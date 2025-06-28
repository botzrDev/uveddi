use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use tree_sitter::{Query, QueryCursor};

const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @name
  body: (field_declaration_list) @body
)
"#;

const RUST_IMPL_QUERY: &str = r#"
(impl_item
  (type_identifier) @name
  body: (associated_type) @body
)
"#;

const PYTHON_CLASS_QUERY: &str = r#"
(class_definition
  name: (identifier) @name
  body: (block) @body
)
"#;

const JAVASCRIPT_CLASS_QUERY: &str = r#"
(class_declaration
  name: (identifier) @name
  body: (class_body) @body
)
"#;

const FUNCTION_COUNT_QUERY: &str = r#"
(function_item)
(function_definition)
(method_definition)
"#;

const FIELD_COUNT_QUERY: &str = r#"
(field_declaration)
(attribute)
(public_field_definition)
"#;

const PYTHON_METHOD_QUERY: &str = r#"
(function_definition)
// Static and class methods (decorators)
(decorated_definition
  decorator: (decorator) @decorator
  definition: (function_definition) @method
)
"#;

const PYTHON_FIELD_QUERY: &str = r#"
// Class-level assignments
(expression_statement
  (assignment
    left: (attribute) @field
    right: (_)
  )
)
// Instance fields in __init__
(function_definition
  name: (identifier) @init_name
  body: (block
    (expression_statement
      (assignment
        left: (attribute) @field
        right: (_)
      )
    )
  )
  (#eq? @init_name "__init__")
)
"#;

const JAVASCRIPT_METHOD_QUERY: &str = r#"
(method_definition)
// Static methods
(method_definition
  static: true
)
"#;

const JAVASCRIPT_FIELD_QUERY: &str = r#"
// Class fields
(public_field_definition)
(field_definition)
// Fields set in constructor
(method_definition
  name: (property_identifier) @ctor_name
  body: (statement_block
    (expression_statement
      (assignment_expression
        left: (member_expression
          object: (this)
          property: (property_identifier) @field
        )
        right: (_)
      )
    )
  )
  (#eq? @ctor_name "constructor")
)
"#;

pub struct GodObjectDetector {
    method_threshold: usize,
    field_threshold: usize,
}

impl GodObjectDetector {
    pub fn new(method_threshold: usize, field_threshold: usize) -> Self {
        Self {
            method_threshold,
            field_threshold,
        }
    }

    fn analyze_node(
        &self,
        node: tree_sitter::Node,
        parsed_file: &ParsedFile,
        issues: &mut Vec<ArchitecturalIssue>,
    ) -> Result<(), AnalysisError> {
        let name_node = node.child_by_field_name("name").unwrap_or(node);
        let body_node = node.child_by_field_name("body").unwrap_or(node);

        let name = name_node
            .utf8_text(parsed_file.source.as_bytes())
            .unwrap_or("Unnamed");

        let (function_query, field_query) = match parsed_file.language {
            SourceLanguage::Rust => (FUNCTION_COUNT_QUERY, FIELD_COUNT_QUERY),
            SourceLanguage::Python => (PYTHON_METHOD_QUERY, PYTHON_FIELD_QUERY),
            SourceLanguage::JavaScript => (JAVASCRIPT_METHOD_QUERY, JAVASCRIPT_FIELD_QUERY),
        };

        let mut cursor = QueryCursor::new();
        let function_query_obj = Query::new(parsed_file.tree.language(), function_query)
            .map_err(|e| AnalysisError::Generic(e.to_string()))?;
        let field_query_obj = Query::new(parsed_file.tree.language(), field_query)
            .map_err(|e| AnalysisError::Generic(e.to_string()))?;
        let method_count = cursor
            .matches(&function_query_obj, body_node, parsed_file.source.as_bytes())
            .count();
        let field_count = cursor
            .matches(&field_query_obj, body_node, parsed_file.source.as_bytes())
            .count();

        if method_count > self.method_threshold || field_count > self.field_threshold {
            issues.push(ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0, // Will be set by the engine
                anti_pattern_type_id: 1, // God Object
                file_path: parsed_file.path.to_str().unwrap_or("").to_string(),
                start_line: Some((name_node.start_position().row + 1) as i32),
                end_line: Some((name_node.end_position().row + 1) as i32),
                severity: "High".to_string(),
                description: format!(
                    "God Object detected: '{}' has {} methods and {} fields. (Thresholds: methods={}, fields={})",
                    name, method_count, field_count, self.method_threshold, self.field_threshold
                ),
                code_snippet: Some(node.utf8_text(parsed_file.source.as_bytes()).unwrap_or("").to_string()),
                ai_explanation: None,
            });
        }

        Ok(())
    }
}

impl AnalysisDetector for GodObjectDetector {
    fn get_detector_name(&self) -> &'static str {
        "GodObjectDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that does too much.".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let (query_str, _lang) = match parsed_file.language {
            SourceLanguage::Rust => (RUST_STRUCT_QUERY, "Rust"),
            SourceLanguage::Python => (PYTHON_CLASS_QUERY, "Python"),
            SourceLanguage::JavaScript => (JAVASCRIPT_CLASS_QUERY, "JavaScript"),
        };

        let query = Query::new(parsed_file.tree.language(), query_str)
            .map_err(|e| AnalysisError::Generic(e.to_string()))?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(
            &query,
            parsed_file.tree.root_node(),
            parsed_file.source.as_bytes(),
        );

        let mut issues = Vec::new();
        for mat in matches {
            for capture in mat.captures {
                self.analyze_node(capture.node, parsed_file, &mut issues)?;
            }
        }

        // Special handling for Rust `impl` blocks
        if parsed_file.language == SourceLanguage::Rust {
            let impl_query = Query::new(parsed_file.tree.language(), RUST_IMPL_QUERY)
                .map_err(|e| AnalysisError::Generic(e.to_string()))?;
            let impl_matches = cursor.matches(
                &impl_query,
                parsed_file.tree.root_node(),
                parsed_file.source.as_bytes(),
            );
            for mat in impl_matches {
                for capture in mat.captures {
                    self.analyze_node(capture.node, parsed_file, &mut issues)?;
                }
            }
        }

        Ok(issues)
    }
}