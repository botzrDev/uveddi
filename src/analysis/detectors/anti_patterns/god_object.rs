use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use std::collections::HashMap;
use tree_sitter::{Query, QueryCursor};

// --- Queries for identifying language-specific containers (classes, structs) ---
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

const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @name
  body: (field_declaration_list) @body
)
"#;

const RUST_IMPL_QUERY: &str = r#"
(impl_item
  type: (type_identifier) @name
  body: (declaration_list) @body
)
"#;

// --- Queries for counting methods/functions within a container ---
const RUST_FUNCTION_COUNT_QUERY: &str = "(function_item)";
const PYTHON_FUNCTION_COUNT_QUERY: &str = "(function_definition)";
const JAVASCRIPT_FUNCTION_COUNT_QUERY: &str = "(method_definition)";

// --- Queries for counting fields/attributes within a container ---
const RUST_FIELD_COUNT_QUERY: &str = "(field_declaration)";
const PYTHON_FIELD_COUNT_QUERY: &str = r#"(expression_statement (assignment))"#;
const JAVASCRIPT_FIELD_COUNT_QUERY: &str = "(field_definition)";

/// Detects "God Objects" - classes or structs that have too many responsibilities.
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

    /// Scores the severity of a God Object based on method and field counts.
    fn score_severity(&self, method_count: usize, field_count: usize) -> Option<String> {
        let method_excess = method_count.saturating_sub(self.method_threshold);
        let field_excess = field_count.saturating_sub(self.field_threshold);

        // Only consider it an issue if at least one threshold is exceeded.
        if method_excess == 0 && field_excess == 0 {
            return None;
        }

        let total_excess = method_excess + field_excess;
        let severity = match total_excess {
            0..=4 => "Medium",
            5..=10 => "High",
            _ => "Critical",
        };
        Some(severity.to_string())
    }

    /// Creates an `ArchitecturalIssue` if a God Object is detected.
    fn create_issue(
        &self,
        parsed_file: &ParsedFile,
        name: &str,
        name_node: tree_sitter::Node,
        container_node: tree_sitter::Node,
        method_count: usize,
        field_count: usize,
    ) -> Option<ArchitecturalIssue> {
        self.score_severity(method_count, field_count)
            .map(|severity| ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0, // Will be set by the engine
                anti_pattern_type_id: 1, // God Object
                file_path: parsed_file.path.to_str().unwrap_or("").to_string(),
                start_line: Some((name_node.start_position().row + 1) as i32),
                end_line: Some((name_node.end_position().row + 1) as i32),
                severity,
                description: format!(
                    "God Object detected: '{}' has {} methods and {} fields. (Thresholds: methods>{}, fields>{})",
                    name, method_count, field_count, self.method_threshold, self.field_threshold
                ),
                code_snippet: Some(
                    container_node
                        .utf8_text(parsed_file.source.as_bytes())
                        .unwrap_or("")
                        .to_string(),
                ),
                ai_explanation: None,
            })
    }

    /// Analyzes a file for God Objects using direct tree-sitter queries.
    /// This is used for Python and JavaScript where class members are in one block.
    fn analyze_standard(
        &self,
        parsed_file: &ParsedFile,
        container_query_str: &str,
        method_query_str: &str,
        field_query_str: &str,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        let container_query = Query::new(&language, container_query_str)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let method_query = Query::new(&language, method_query_str)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let field_query = Query::new(&language, field_query_str)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&container_query, tree.root_node(), source) {
            let name_node = mat.captures[0].node;
            let body_node = mat.captures[1].node;
            let container_node = name_node.parent().unwrap_or(name_node);

            let name = name_node.utf8_text(source).unwrap_or("Unnamed");

            let mut method_cursor = QueryCursor::new();
            let method_count = method_cursor
                .matches(&method_query, body_node, source)
                .count();

            let mut field_cursor = QueryCursor::new();
            let field_count = field_cursor
                .matches(&field_query, body_node, source)
                .count();

            if let Some(issue) = self.create_issue(
                parsed_file,
                name,
                name_node,
                container_node,
                method_count,
                field_count,
            ) {
                issues.push(issue);
            }
        }
        Ok(issues)
    }

    /// Analyzes a Rust file, which requires correlating `struct` and `impl` blocks.
    fn analyze_rust(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();
        let root_node = tree.root_node();

        // 1. Find all impl blocks and count their methods
        let mut impl_method_counts: HashMap<String, usize> = HashMap::new();
        let impl_query = Query::new(&language, RUST_IMPL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let function_query = Query::new(&language, RUST_FUNCTION_COUNT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&impl_query, root_node, source) {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.get(0), mat.captures.get(1))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let mut method_cursor = QueryCursor::new();
                    let method_count = method_cursor
                        .matches(&function_query, body_node, source)
                        .count();
                    impl_method_counts.insert(name.to_string(), method_count);
                }
            }
        }

        // 2. Find all structs, count their fields, and check against method counts
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let field_query = Query::new(&language, RUST_FIELD_COUNT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut struct_cursor = QueryCursor::new();
        for mat in struct_cursor.matches(&struct_query, root_node, source) {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.get(0), mat.captures.get(1))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                let container_node = name_node.parent().unwrap_or(name_node);

                if let Ok(name) = name_node.utf8_text(source) {
                    let method_count = impl_method_counts.get(name).cloned().unwrap_or(0);

                    let mut field_cursor = QueryCursor::new();
                    let field_count = field_cursor
                        .matches(&field_query, body_node, source)
                        .count();

                    if let Some(issue) = self.create_issue(
                        parsed_file,
                        name,
                        name_node,
                        container_node,
                        method_count,
                        field_count,
                    ) {
                        issues.push(issue);
                    }
                }
            }
        }

        Ok(issues)
    }
}

impl AnalysisDetector for GodObjectDetector {
    fn get_detector_name(&self) -> &'static str {
        "GodObjectDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class or struct that centralizes too many responsibilities, violating the Single Responsibility Principle.".to_string(),
            category: "Abstraction-Based".to_string(),
        }]
    }

    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.analyze_rust(parsed_file),
            SourceLanguage::Python => self.analyze_standard(
                parsed_file,
                PYTHON_CLASS_QUERY,
                PYTHON_FUNCTION_COUNT_QUERY,
                PYTHON_FIELD_COUNT_QUERY,
            ),
            SourceLanguage::JavaScript => self.analyze_standard(
                parsed_file,
                JAVASCRIPT_CLASS_QUERY,
                JAVASCRIPT_FUNCTION_COUNT_QUERY,
                JAVASCRIPT_FIELD_COUNT_QUERY,
            ),
        }
    }
}
