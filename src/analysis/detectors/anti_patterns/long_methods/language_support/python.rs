//! Python-specific method analysis

use crate::analysis::AnalysisError;
#[cfg(feature = "tree-sitter")]
use crate::ast::tree_sitter::{Language, Node, Query};

/// Python-specific method analyzer
pub struct PythonMethodAnalyzer;

impl PythonMethodAnalyzer {
    /// Tree-sitter query for Python functions
    pub const FUNCTION_QUERY: &'static str = r#"
(function_definition
  name: (identifier) @name
  body: (block) @body) @function

(class_definition
  body: (block
    (function_definition
      name: (identifier) @name
      body: (block) @body) @function))
"#;

    /// Create Python function query
    #[cfg(feature = "tree-sitter")]
    pub fn create_query(language: &Language) -> Result<Query, AnalysisError> {
        Query::new(language, Self::FUNCTION_QUERY).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Failed to create Python function query: {}",
                e
            ))
        })
    }

    /// Count parameters in a Python function
    #[cfg(feature = "tree-sitter")]
    pub fn count_parameters(node: &Node, source: &[u8]) -> Result<u32, AnalysisError> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "parameters" {
                    let param_count = cursor.node().child_count() as u32;

                    // Subtract 1 for 'self' parameter if present
                    if param_count > 0 {
                        if let Some(first_param) = cursor.node().child(0) {
                            if let Ok(param_text) = first_param.utf8_text(source) {
                                if param_text.trim() == "self" {
                                    return Ok(param_count - 1);
                                }
                            }
                        }
                    }
                    return Ok(param_count);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(0)
    }

    /// Check if a Python function is exported (not starting with underscore)
    #[cfg(feature = "tree-sitter")]
    pub fn is_exported(node: &Node, source: &[u8]) -> bool {
        // In Python, functions starting with underscore are considered private
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "identifier" {
                    if let Ok(name) = cursor.node().utf8_text(source) {
                        return !name.starts_with('_');
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        false
    }

    /// Determine Python method type
    #[cfg(feature = "tree-sitter")]
    pub fn determine_method_type(node: &Node, source: &[u8]) -> String {
        // Check if it's an async function
        if Self::is_async_function(node, source) {
            if Self::is_in_class(node) {
                return "async_method".to_string();
            } else {
                return "async_function".to_string();
            }
        }

        // Check if it's in a class
        if Self::is_in_class(node) {
            // Check for special methods
            if let Ok(name) = Self::get_function_name(node, source) {
                if name.starts_with("__") && name.ends_with("__") {
                    return "dunder_method".to_string();
                }
                if name == "setUp" || name == "tearDown" || name.starts_with("test_") {
                    return "test_method".to_string();
                }
            }
            return "method".to_string();
        }

        "function".to_string()
    }

    /// Check if function is async
    #[cfg(feature = "tree-sitter")]
    fn is_async_function(node: &Node, source: &[u8]) -> bool {
        if let Ok(text) = node.utf8_text(source) {
            return text.trim_start().starts_with("async def");
        }
        false
    }

    /// Check if function is in a class
    #[cfg(feature = "tree-sitter")]
    fn is_in_class(node: &Node) -> bool {
        let mut cursor = node.walk();
        while cursor.goto_parent() {
            if cursor.node().kind() == "class_definition" {
                return true;
            }
        }
        false
    }

    /// Get function name
    #[cfg(feature = "tree-sitter")]
    fn get_function_name(node: &Node, source: &[u8]) -> Result<String, AnalysisError> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "identifier" {
                    if let Ok(name) = cursor.node().utf8_text(source) {
                        return Ok(name.to_string());
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Err(AnalysisError::AntiPatternDetectionError(
            "Could not find function name".to_string(),
        ))
    }

    /// Extract Python decorators
    #[cfg(feature = "tree-sitter")]
    pub fn extract_decorators(node: &Node, source: &[u8]) -> Vec<String> {
        let mut decorators = Vec::new();
        let mut cursor = node.walk();

        // Look for decorators before the function
        if cursor.goto_parent() {
            let mut sibling_cursor = cursor;
            while sibling_cursor.goto_previous_sibling() {
                if sibling_cursor.node().kind() == "decorated_definition" {
                    // Look for decorator nodes within
                    Self::collect_decorators(&sibling_cursor.node(), source, &mut decorators);
                    break;
                } else if sibling_cursor.node().kind() == "decorator" {
                    if let Ok(decorator) = sibling_cursor.node().utf8_text(source) {
                        decorators.push(decorator.to_string());
                    }
                } else if !matches!(sibling_cursor.node().kind(), "comment" | "newline") {
                    break; // Stop at first non-decorator, non-comment
                }
            }
        }

        decorators.reverse(); // Reverse to get original order
        decorators
    }

    #[cfg(feature = "tree-sitter")]
    fn collect_decorators(node: &Node, source: &[u8], decorators: &mut Vec<String>) {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "decorator" {
                    if let Ok(decorator) = child.utf8_text(source) {
                        decorators.push(decorator.to_string());
                    }
                }
            }
        }
    }

    /// Analyze Python-specific patterns
    #[cfg(feature = "tree-sitter")]
    pub fn analyze_python_patterns(node: &Node, source: &[u8]) -> PythonPatternAnalysis {
        let mut analysis = PythonPatternAnalysis::default();

        Self::count_pattern_usage(node, source, &mut analysis);

        analysis
    }

    #[cfg(feature = "tree-sitter")]
    fn count_pattern_usage(node: &Node, _source: &[u8], analysis: &mut PythonPatternAnalysis) {
        let node_kind = node.kind();

        match node_kind {
            "with_statement" => analysis.with_statements += 1,
            "try_statement" => analysis.try_statements += 1,
            "except_clause" => analysis.except_clauses += 1,
            "finally_clause" => analysis.finally_clauses += 1,
            "list_comprehension" => analysis.list_comprehensions += 1,
            "dictionary_comprehension" => analysis.dict_comprehensions += 1,
            "set_comprehension" => analysis.set_comprehensions += 1,
            "generator_expression" => analysis.generator_expressions += 1,
            "lambda" => analysis.lambda_expressions += 1,
            "yield" => analysis.yield_statements += 1,
            "yield_from_statement" => analysis.yield_from_statements += 1,
            "await" => analysis.await_expressions += 1,
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

/// Analysis of Python-specific patterns
#[derive(Debug, Clone, Default)]
pub struct PythonPatternAnalysis {
    pub with_statements: u32,
    pub try_statements: u32,
    pub except_clauses: u32,
    pub finally_clauses: u32,
    pub list_comprehensions: u32,
    pub dict_comprehensions: u32,
    pub set_comprehensions: u32,
    pub generator_expressions: u32,
    pub lambda_expressions: u32,
    pub yield_statements: u32,
    pub yield_from_statements: u32,
    pub await_expressions: u32,
}

impl PythonPatternAnalysis {
    /// Calculate pattern complexity score
    pub fn complexity_score(&self) -> u32 {
        // Comprehensions can reduce complexity compared to loops
        let comprehension_benefit = (self.list_comprehensions +
                                   self.dict_comprehensions +
                                   self.set_comprehensions).saturating_sub(2);

        // Exception handling adds complexity
        let exception_penalty = (self.try_statements + self.except_clauses) * 2;

        // Generators and async add complexity
        let async_penalty = self.await_expressions + self.yield_statements;

        // Lambda expressions add complexity
        let lambda_penalty = self.lambda_expressions;

        exception_penalty + async_penalty + lambda_penalty - comprehension_benefit
    }
}