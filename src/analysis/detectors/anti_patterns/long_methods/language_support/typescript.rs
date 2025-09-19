//! TypeScript/JavaScript-specific method analysis

use crate::analysis::AnalysisError;
#[cfg(feature = "tree-sitter")]
use crate::ast::tree_sitter::{Language, Node, Query};

/// TypeScript/JavaScript-specific method analyzer
pub struct TypeScriptMethodAnalyzer;

impl TypeScriptMethodAnalyzer {
    /// Tree-sitter query for JavaScript/TypeScript functions
    pub const FUNCTION_QUERY: &'static str = r#"
(function_declaration
  name: (identifier) @name
  body: (statement_block) @body) @function

(method_definition
  name: (property_name) @name
  body: (statement_block) @body) @function

(arrow_function
  body: (statement_block) @body) @function

(function_expression
  name: (identifier)? @name
  body: (statement_block) @body) @function
"#;

    /// Create JavaScript/TypeScript function query
    #[cfg(feature = "tree-sitter")]
    pub fn create_query(language: &Language) -> Result<Query, AnalysisError> {
        Query::new(language, Self::FUNCTION_QUERY).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Failed to create JavaScript/TypeScript function query: {}",
                e
            ))
        })
    }

    /// Count parameters in a JavaScript/TypeScript function
    #[cfg(feature = "tree-sitter")]
    pub fn count_parameters(node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "formal_parameters" {
                    return Ok(cursor.node().child_count() as u32);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(0)
    }

    /// Check if a JavaScript/TypeScript function is exported
    #[cfg(feature = "tree-sitter")]
    pub fn is_exported(node: &Node, source: &[u8]) -> bool {
        // Check for export keyword
        let mut cursor = node.walk();
        if cursor.goto_parent() {
            loop {
                if cursor.node().kind() == "export_statement" {
                    return true;
                }
                if cursor.node().kind() == "lexical_declaration" {
                    // Check for 'export' in the source text
                    if let Ok(text) = cursor.node().utf8_text(source) {
                        if text.trim().starts_with("export") {
                            return true;
                        }
                    }
                }
                if !cursor.goto_previous_sibling() {
                    break;
                }
            }
        }
        false
    }

    /// Determine JavaScript/TypeScript method type
    #[cfg(feature = "tree-sitter")]
    pub fn determine_method_type(node: &Node, source: &[u8]) -> String {
        let node_kind = node.kind();

        // Check for async
        let is_async = Self::is_async_function(node, source);

        match node_kind {
            "arrow_function" => {
                if is_async {
                    "async_arrow".to_string()
                } else {
                    "arrow_function".to_string()
                }
            }
            "method_definition" => {
                if is_async {
                    "async_method".to_string()
                } else {
                    Self::classify_method(node, source)
                }
            }
            "function_expression" => {
                if is_async {
                    "async_function_expression".to_string()
                } else {
                    "function_expression".to_string()
                }
            }
            "function_declaration" => {
                if is_async {
                    "async_function".to_string()
                } else {
                    "function".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }

    /// Classify method types (constructor, getter, setter, etc.)
    #[cfg(feature = "tree-sitter")]
    fn classify_method(node: &Node, source: &[u8]) -> String {
        // Check for method kind
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "get" => return "getter".to_string(),
                    "set" => return "setter".to_string(),
                    "static" => return "static_method".to_string(),
                    "property_name" => {
                        if let Ok(name) = child.utf8_text(source) {
                            if name == "constructor" {
                                return "constructor".to_string();
                            }
                        }
                    }
                    _ => {}
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        "method".to_string()
    }

    /// Check if function is async
    #[cfg(feature = "tree-sitter")]
    fn is_async_function(node: &Node, source: &[u8]) -> bool {
        if let Ok(text) = node.utf8_text(source) {
            return text.trim_start().starts_with("async");
        }
        false
    }

    /// Extract TypeScript type annotations
    #[cfg(feature = "tree-sitter")]
    pub fn extract_type_annotations(node: &Node, source: &[u8]) -> TypeAnnotations {
        let mut annotations = TypeAnnotations::default();

        Self::collect_type_info(node, source, &mut annotations);

        annotations
    }

    #[cfg(feature = "tree-sitter")]
    fn collect_type_info(node: &Node, source: &[u8], annotations: &mut TypeAnnotations) {
        let node_kind = node.kind();

        match node_kind {
            "type_annotation" => {
                if let Ok(type_text) = node.utf8_text(source) {
                    annotations.parameter_types.push(type_text.to_string());
                }
            }
            "return_type" => {
                if let Ok(return_type) = node.utf8_text(source) {
                    annotations.return_type = Some(return_type.to_string());
                }
            }
            "generic_type" => {
                annotations.has_generics = true;
            }
            _ => {}
        }

        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                Self::collect_type_info(&child, source, annotations);
            }
        }
    }

    /// Analyze JavaScript/TypeScript-specific patterns
    #[cfg(feature = "tree-sitter")]
    pub fn analyze_js_patterns(node: &Node, source: &[u8]) -> JSPatternAnalysis {
        let mut analysis = JSPatternAnalysis::default();

        Self::count_pattern_usage(node, source, &mut analysis);

        analysis
    }

    #[cfg(feature = "tree-sitter")]
    fn count_pattern_usage(node: &Node, _source: &[u8], analysis: &mut JSPatternAnalysis) {
        let node_kind = node.kind();

        match node_kind {
            "try_statement" => analysis.try_statements += 1,
            "catch_clause" => analysis.catch_clauses += 1,
            "finally_clause" => analysis.finally_clauses += 1,
            "await_expression" => analysis.await_expressions += 1,
            "yield_expression" => analysis.yield_expressions += 1,
            "arrow_function" => analysis.arrow_functions += 1,
            "function_expression" => analysis.function_expressions += 1,
            "call_expression" => analysis.call_expressions += 1,
            "new_expression" => analysis.new_expressions += 1,
            "template_literal" => analysis.template_literals += 1,
            "spread_element" => analysis.spread_operators += 1,
            "destructuring_pattern" => analysis.destructuring_patterns += 1,
            "class_declaration" => analysis.class_declarations += 1,
            _ => {}
        }

        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                Self::count_pattern_usage(&child, _source, analysis);
            }
        }
    }
}

/// TypeScript type annotations
#[derive(Debug, Clone, Default)]
pub struct TypeAnnotations {
    pub parameter_types: Vec<String>,
    pub return_type: Option<String>,
    pub has_generics: bool,
}

/// Analysis of JavaScript/TypeScript-specific patterns
#[derive(Debug, Clone, Default)]
pub struct JSPatternAnalysis {
    pub try_statements: u32,
    pub catch_clauses: u32,
    pub finally_clauses: u32,
    pub await_expressions: u32,
    pub yield_expressions: u32,
    pub arrow_functions: u32,
    pub function_expressions: u32,
    pub call_expressions: u32,
    pub new_expressions: u32,
    pub template_literals: u32,
    pub spread_operators: u32,
    pub destructuring_patterns: u32,
    pub class_declarations: u32,
}

impl JSPatternAnalysis {
    /// Calculate pattern complexity score
    pub fn complexity_score(&self) -> u32 {
        // Exception handling adds complexity
        let exception_penalty = (self.try_statements + self.catch_clauses) * 2;

        // Async patterns add complexity
        let async_penalty = self.await_expressions + self.yield_expressions;

        // Higher-order functions add some complexity
        let hof_penalty = (self.arrow_functions + self.function_expressions).saturating_sub(2);

        // Destructuring can reduce complexity (up to a point)
        let destructuring_benefit = self.destructuring_patterns.min(3);

        exception_penalty + async_penalty + hof_penalty - destructuring_benefit
    }
}