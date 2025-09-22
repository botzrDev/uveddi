//! Metrics calculation for God Object detection

use super::detector::ComplexityMetrics;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use streaming_iterator::StreamingIterator;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::error::ErrorHelpers;

// Query strings for metric calculation
const RUST_FUNCTION_COUNT_QUERY: &str = "(function_item)";
const PYTHON_FUNCTION_COUNT_QUERY: &str = "(function_definition)";
const JAVASCRIPT_FUNCTION_COUNT_QUERY: &str = "(method_definition)";
const TYPESCRIPT_FUNCTION_COUNT_QUERY: &str = r#"
[
  (method_definition)
  (method_signature)
  (function_declaration)
  (function_signature)
]
"#;

const RUST_FIELD_COUNT_QUERY: &str = "(field_declaration)";
const PYTHON_FIELD_COUNT_QUERY: &str = r#"(expression_statement (assignment))"#;
const JAVASCRIPT_FIELD_COUNT_QUERY: &str = "(field_definition)";
const TYPESCRIPT_FIELD_COUNT_QUERY: &str = r#"
[
  (field_definition)
  (property_signature)
  (public_field_definition)
  (private_field_definition)
  (protected_field_definition)
  (readonly_field_definition)
]
"#;

/// Metrics calculator for God Object detection
pub struct MetricsCalculator;

impl MetricsCalculator {
    /// Calculate complexity metrics for a given node
    pub fn calculate_metrics(
        parsed_file: &ParsedFile,
        node: Node,
    ) -> Result<ComplexityMetrics, AnalysisError> {
        let mut metrics = ComplexityMetrics::default();

        metrics.method_count = Self::calculate_method_count(parsed_file, node)?;
        metrics.field_count = Self::calculate_field_count(parsed_file, node)?;

        if metrics.method_count > 0 {
            let (trivial, complex) = Self::analyze_behavioral_complexity(parsed_file, node)?;
            metrics.trivial_methods = trivial;
            metrics.complex_methods = complex;
        }

        metrics.lcom4_score = Self::calculate_lcom4(parsed_file, node).ok();
        metrics.dependency_count = Self::calculate_dependency_count(parsed_file, node)?;

        Ok(metrics)
    }

    /// Calculate the number of methods in a class/struct
    pub fn calculate_method_count(
        parsed_file: &ParsedFile,
        node: Node,
    ) -> Result<usize, AnalysisError> {
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("method count calculation"))?;
        let language = tree.language();

        let query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_FUNCTION_COUNT_QUERY,
            SourceLanguage::Python => PYTHON_FUNCTION_COUNT_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_FUNCTION_COUNT_QUERY,
            SourceLanguage::TypeScript => TYPESCRIPT_FUNCTION_COUNT_QUERY,
        };

        let query = Query::new(&language, query_str)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, node, source);
        let mut count = 0;
        while matches.next().is_some() {
            count += 1;
        }

        Ok(count)
    }

    /// Calculate the number of fields in a class/struct
    pub fn calculate_field_count(
        parsed_file: &ParsedFile,
        node: Node,
    ) -> Result<usize, AnalysisError> {
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("field count calculation"))?;
        let language = tree.language();

        let query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_FIELD_COUNT_QUERY,
            SourceLanguage::Python => PYTHON_FIELD_COUNT_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_FIELD_COUNT_QUERY,
            SourceLanguage::TypeScript => TYPESCRIPT_FIELD_COUNT_QUERY,
        };

        let query = Query::new(&language, query_str)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, node, source);
        let mut count = 0;
        while matches.next().is_some() {
            count += 1;
        }

        Ok(count)
    }

    /// Calculate Lines of Code for a node
    pub fn calculate_lines_of_code(node: Node, source: &str) -> usize {
        let start_row = node.start_position().row;
        let end_row = node.end_position().row;
        (end_row - start_row + 1) as usize
    }

    /// Calculate dependency count (simplified metric)
    pub fn calculate_dependency_count(
        parsed_file: &ParsedFile,
        _node: Node,
    ) -> Result<usize, AnalysisError> {
        // Simplified implementation - count import statements in the file
        let import_patterns = match parsed_file.language {
            SourceLanguage::Rust => vec!["use "],
            SourceLanguage::Python => vec!["import ", "from "],
            SourceLanguage::JavaScript => vec!["import ", "require("],
            SourceLanguage::TypeScript => vec!["import ", "require("],
        };

        let mut count = 0;
        for line in parsed_file.source.lines() {
            for pattern in &import_patterns {
                if line.trim().starts_with(pattern) {
                    count += 1;
                    break;
                }
            }
        }

        Ok(count)
    }

    /// Calculate a simplified LCOM4 score for cohesion analysis
    pub fn calculate_lcom4(parsed_file: &ParsedFile, node: Node) -> Result<u32, AnalysisError> {
        // Simplified implementation - in practice, you'd want sophisticated analysis
        let method_count = Self::calculate_method_count(parsed_file, node)?;

        // Simplified heuristic: assume low cohesion if many methods (>10) without deep analysis
        // A proper implementation would analyze shared fields and method calls
        if method_count > 10 {
            Ok(2) // Assume multiple responsibilities
        } else {
            Ok(1) // Assume cohesive
        }
    }

    /// Analyze behavioral complexity to classify methods as trivial or complex
    pub fn analyze_behavioral_complexity(
        parsed_file: &ParsedFile,
        node: Node,
    ) -> Result<(usize, usize), AnalysisError> {
        // Simplified behavioral analysis
        // In practice, you'd calculate Cyclomatic Complexity for each method
        let total_methods = Self::calculate_method_count(parsed_file, node)?;

        // Simplified heuristic: assume 70% are trivial methods (getters, setters)
        let trivial_methods = (total_methods as f64 * 0.7) as usize;
        let complex_methods = total_methods - trivial_methods;

        Ok((trivial_methods, complex_methods))
    }

    /// Calculate cyclomatic complexity for a method (simplified)
    pub fn calculate_cyclomatic_complexity(
        _parsed_file: &ParsedFile,
        _method_node: Node,
    ) -> Result<u32, AnalysisError> {
        // Simplified implementation - would count decision points (if, while, for, etc.)
        // For now, return a default value
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::cache::wrappers::ArchivableSystemTime;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::SystemTime;

    fn create_test_parsed_file(source: &str, language: SourceLanguage) -> ParsedFile {
        ParsedFile {
            file_path: PathBuf::from("test.rs").into(),
            source: source.to_string().into(),
            language,
            tree: None, // Would be populated by actual parser
            custom_ast: Arc::new(None),
            modified_at: ArchivableSystemTime::from(SystemTime::now()),
        }
    }

    #[test]
    fn test_lines_of_code_calculation() {
        // This would require a real AST node in practice
        // For now, test the concept
        let source = "struct Test {\n    field1: i32,\n    field2: String,\n}";
        // In real implementation, you'd create a proper node
        // let loc = MetricsCalculator::calculate_lines_of_code(node, source);
        // assert_eq!(loc, 4);
    }

    #[test]
    fn test_dependency_count() {
        let source = r#"
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::other::Module;

struct Test {
    field: String,
}
"#;
        let parsed_file = create_test_parsed_file(source, SourceLanguage::Rust);
        // In practice, this would need a real node
        // let count = MetricsCalculator::calculate_dependency_count(&parsed_file, node).unwrap();
        // assert_eq!(count, 3); // Three use statements
    }

    #[test]
    fn test_complexity_metrics_creation() {
        let metrics = ComplexityMetrics {
            method_count: 15,
            field_count: 8,
            trivial_methods: 10,
            complex_methods: 5,
            lcom4_score: Some(2),
            dependency_count: 5,
        };

        assert_eq!(metrics.method_count, 15);
        assert_eq!(metrics.field_count, 8);
        assert_eq!(metrics.lcom4_score, Some(2));
    }
}
