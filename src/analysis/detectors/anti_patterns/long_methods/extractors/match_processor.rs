//! Query match processing for method extraction

use crate::analysis::detectors::anti_patterns::long_methods::analyzers::{
    ComplexityAnalyzer, LineCounter, NestingAnalyzer,
};
use crate::analysis::detectors::anti_patterns::long_methods::types::MethodMetrics;
use crate::analysis::AnalysisError;
#[cfg(feature = "tree-sitter")]
use crate::ast::tree_sitter::{Node, QueryMatch};

/// Helper for processing tree-sitter query matches
pub struct MatchProcessor;

impl MatchProcessor {
    /// Process a Rust query match
    #[cfg(feature = "tree-sitter")]
    pub fn process_rust_match(
        query_match: &QueryMatch,
        source: &[u8],
        file_path: &str,
    ) -> Result<Option<MethodMetrics>, AnalysisError> {
        if let (Some(function_capture), Some(name_capture), Some(body_capture)) = (
            query_match.captures.get(0),
            query_match.captures.get(1),
            query_match.captures.get(2),
        ) {
            let function_node = function_capture.node;
            let name_node = name_capture.node;
            let body_node = body_capture.node;

            if let Ok(name) = name_node.utf8_text(source) {
                let metrics = Self::calculate_method_metrics(
                    name,
                    &function_node,
                    &body_node,
                    source,
                    file_path,
                    "function",
                )?;
                return Ok(Some(metrics));
            }
        }
        Ok(None)
    }

    /// Process a Python query match
    #[cfg(feature = "tree-sitter")]
    pub fn process_python_match(
        query_match: &QueryMatch,
        source: &[u8],
        file_path: &str,
    ) -> Result<Option<MethodMetrics>, AnalysisError> {
        if let (Some(function_capture), Some(name_capture), Some(body_capture)) = (
            query_match.captures.get(0),
            query_match.captures.get(1),
            query_match.captures.get(2),
        ) {
            let function_node = function_capture.node;
            let name_node = name_capture.node;
            let body_node = body_capture.node;

            if let Ok(name) = name_node.utf8_text(source) {
                let metrics = Self::calculate_method_metrics(
                    name,
                    &function_node,
                    &body_node,
                    source,
                    file_path,
                    "function",
                )?;
                return Ok(Some(metrics));
            }
        }
        Ok(None)
    }

    /// Process a JavaScript query match
    #[cfg(feature = "tree-sitter")]
    pub fn process_javascript_match(
        query_match: &QueryMatch,
        source: &[u8],
        file_path: &str,
    ) -> Result<Option<MethodMetrics>, AnalysisError> {
        if let (Some(function_capture), Some(body_capture)) = (
            query_match.captures.get(0),
            query_match.captures.get(2),
        ) {
            let function_node = function_capture.node;
            let body_node = body_capture.node;

            // Name might be optional for arrow functions
            let name = if let Some(name_capture) = query_match.captures.get(1) {
                name_capture.node.utf8_text(source).unwrap_or("anonymous")
            } else {
                "anonymous"
            };

            let metrics = Self::calculate_method_metrics(
                name,
                &function_node,
                &body_node,
                source,
                file_path,
                "function",
            )?;
            return Ok(Some(metrics));
        }
        Ok(None)
    }

    /// Calculate comprehensive metrics for a method
    #[cfg(feature = "tree-sitter")]
    fn calculate_method_metrics(
        name: &str,
        function_node: &Node,
        body_node: &Node,
        source: &[u8],
        file_path: &str,
        method_type: &str,
    ) -> Result<MethodMetrics, AnalysisError> {
        let logical_loc = LineCounter::calculate_logical_loc(function_node, source);
        let statement_count = LineCounter::count_statements(body_node, source)?;
        let max_nesting_depth = NestingAnalyzer::calculate_max_nesting_depth(body_node, source);
        let cyclomatic_complexity = ComplexityAnalyzer::calculate_cyclomatic(body_node, source)?;
        let cognitive_complexity = ComplexityAnalyzer::calculate_cognitive(body_node, source)?;

        let parameter_count = Self::count_parameters(function_node, source)?;
        let is_exported = Self::check_if_exported(function_node, source);
        let code_snippet = Self::extract_code_snippet(function_node, source, 5);

        Ok(MethodMetrics {
            name: name.to_string(),
            file_path: file_path.to_string(),
            start_line: (function_node.start_position().row + 1) as u32,
            end_line: (function_node.end_position().row + 1) as u32,
            logical_loc,
            statement_count,
            parameter_count,
            max_nesting_depth,
            cyclomatic_complexity,
            cognitive_complexity,
            is_exported,
            code_snippet,
            method_type: method_type.to_string(),
        })
    }

    /// Count parameters in a function
    #[cfg(feature = "tree-sitter")]
    fn count_parameters(node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        // This is a simplified implementation
        // Real implementation would be language-specific
        Ok(0)
    }

    /// Check if a function is exported/public
    #[cfg(feature = "tree-sitter")]
    fn check_if_exported(node: &Node, source: &[u8]) -> bool {
        // Look for visibility modifiers
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

    /// Extract a code snippet for display
    #[cfg(feature = "tree-sitter")]
    fn extract_code_snippet(node: &Node, source: &[u8], max_lines: usize) -> String {
        if let Ok(text) = node.utf8_text(source) {
            let lines: Vec<&str> = text.lines().take(max_lines).collect();
            let mut snippet = lines.join("\n");
            if text.lines().count() > max_lines {
                snippet.push_str("\n...");
            }
            snippet
        } else {
            "Failed to extract snippet".to_string()
        }
    }
}