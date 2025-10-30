//! Rust-specific method analysis

use crate::analysis::detectors::anti_patterns::long_methods::types::MethodMetrics;
use crate::analysis::AnalysisError;
#[cfg(feature = "tree-sitter")]
use crate::ast::tree_sitter::{Language, Node, Query};

/// Rust-specific method analyzer
pub struct RustMethodAnalyzer;

impl RustMethodAnalyzer {
    /// Tree-sitter query for Rust functions
    pub const FUNCTION_QUERY: &'static str = r#"
[
(function_item
  name: (identifier) @name
  body: (block) @body) @function

(impl_item
  body: (declaration_list
    (function_item
      name: (identifier) @name
      body: (block) @body) @function))
]
"#;

    /// Create Rust function query
    #[cfg(feature = "tree-sitter")]
    pub fn create_query(language: &Language) -> Result<Query, AnalysisError> {
        Query::new(language, Self::FUNCTION_QUERY).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Failed to create Rust function query: {}",
                e
            ))
        })
    }

    /// Count parameters in a Rust function
    #[cfg(feature = "tree-sitter")]
    pub fn count_parameters(node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "parameters" {
                    let params_node = cursor.node();
                    let mut param_cursor = params_node.walk();
                    let mut param_count = 0;

                    if param_cursor.goto_first_child() {
                        loop {
                            if param_cursor.node().kind() == "parameter" {
                                param_count += 1;
                            }
                            if !param_cursor.goto_next_sibling() {
                                break;
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

    /// Check if a Rust function is exported
    #[cfg(feature = "tree-sitter")]
    pub fn is_exported(node: &Node, source: &[u8]) -> bool {
        let mut cursor = node.walk();
        if cursor.goto_parent() {
            loop {
                if cursor.node().kind() == "visibility_modifier" {
                    if let Ok(vis) = cursor.node().utf8_text(source) {
                        return vis.contains("pub");
                    }
                }
                if !cursor.goto_previous_sibling() {
                    break;
                }
            }
        }
        false
    }

    /// Determine Rust method type
    #[cfg(feature = "tree-sitter")]
    pub fn determine_method_type(node: &Node, source: &[u8]) -> String {
        // Check if it's an async function
        if Self::is_async_function(node, source) {
            return "async_function".to_string();
        }

        // Check if it's in an impl block
        if Self::is_in_impl_block(node) {
            return "method".to_string();
        }

        // Check if it's a closure
        if node.kind() == "closure_expression" {
            return "closure".to_string();
        }

        "function".to_string()
    }

    /// Check if function is async
    #[cfg(feature = "tree-sitter")]
    fn is_async_function(node: &Node, source: &[u8]) -> bool {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "async" {
                    return true;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        // Also check source text directly
        if let Ok(text) = node.utf8_text(source) {
            return text.trim_start().starts_with("async");
        }

        false
    }

    /// Check if function is in an impl block
    #[cfg(feature = "tree-sitter")]
    fn is_in_impl_block(node: &Node) -> bool {
        let mut cursor = node.walk();
        while cursor.goto_parent() {
            if cursor.node().kind() == "impl_item" {
                return true;
            }
        }
        false
    }

    /// Extract Rust-specific attributes
    #[cfg(feature = "tree-sitter")]
    pub fn extract_attributes(node: &Node, source: &[u8]) -> Vec<String> {
        let mut attributes = Vec::new();
        let mut cursor = node.walk();

        // Look for attributes before the function
        if cursor.goto_parent() {
            let mut sibling_cursor = cursor;
            while sibling_cursor.goto_previous_sibling() {
                if sibling_cursor.node().kind() == "attribute_item" {
                    if let Ok(attr) = sibling_cursor.node().utf8_text(source) {
                        attributes.push(attr.to_string());
                    }
                } else {
                    break; // Stop at first non-attribute
                }
            }
        }

        attributes.reverse(); // Reverse to get original order
        attributes
    }

    /// Check for common Rust patterns that affect complexity
    #[cfg(feature = "tree-sitter")]
    pub fn analyze_rust_patterns(node: &Node, source: &[u8]) -> RustPatternAnalysis {
        let mut analysis = RustPatternAnalysis::default();

        Self::count_pattern_usage(node, source, &mut analysis);

        analysis
    }

    #[cfg(feature = "tree-sitter")]
    fn count_pattern_usage(node: &Node, _source: &[u8], analysis: &mut RustPatternAnalysis) {
        let node_kind = node.kind();

        match node_kind {
            "match_expression" => analysis.match_expressions += 1,
            "if_let_expression" => analysis.if_let_expressions += 1,
            "while_let_expression" => analysis.while_let_expressions += 1,
            "for_expression" => analysis.for_loops += 1,
            "loop_expression" => analysis.infinite_loops += 1,
            "closure_expression" => analysis.closures += 1,
            "macro_invocation" => analysis.macro_invocations += 1,
            "await_expression" => analysis.await_expressions += 1,
            "try_expression" => analysis.try_expressions += 1,
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

/// Analysis of Rust-specific patterns
#[derive(Debug, Clone, Default)]
pub struct RustPatternAnalysis {
    pub match_expressions: u32,
    pub if_let_expressions: u32,
    pub while_let_expressions: u32,
    pub for_loops: u32,
    pub infinite_loops: u32,
    pub closures: u32,
    pub macro_invocations: u32,
    pub await_expressions: u32,
    pub try_expressions: u32,
}

impl RustPatternAnalysis {
    /// Calculate pattern complexity score
    pub fn complexity_score(&self) -> u32 {
        // Match expressions can reduce complexity compared to if-else chains
        let match_benefit = self.match_expressions.saturating_sub(2);

        // Closures and macros add complexity
        let closure_penalty = self.closures * 2;
        let macro_penalty = self.macro_invocations;

        // Async patterns add complexity
        let async_penalty = self.await_expressions;

        closure_penalty + macro_penalty + async_penalty - match_benefit
    }
}
