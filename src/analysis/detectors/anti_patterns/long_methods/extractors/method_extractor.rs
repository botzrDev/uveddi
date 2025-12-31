//! Method extraction logic from AST

use super::match_processor::MatchProcessor;
use crate::analysis::detectors::anti_patterns::long_methods::types::MethodMetrics;
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};

/// Extractor for method definitions and their metrics
pub struct MethodExtractor;

impl MethodExtractor {
    /// Extract all method metrics from a parsed file
    pub fn extract_metrics(parsed_file: &ParsedFile) -> Result<Vec<MethodMetrics>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => Self::extract_rust_metrics(parsed_file),
            SourceLanguage::Python => Self::extract_python_metrics(parsed_file),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                Self::extract_javascript_metrics(parsed_file)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Extract metrics for Rust functions
    #[cfg(feature = "tree-sitter")]
    fn extract_rust_metrics(parsed_file: &ParsedFile) -> Result<Vec<MethodMetrics>, AnalysisError> {
        use crate::analysis::detectors::anti_patterns::long_methods::language_support::rust::RustMethodAnalyzer;
        use crate::ast::tree_sitter::QueryCursor;
        use streaming_iterator::StreamingIterator;

        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        let query = RustMethodAnalyzer::create_query(&language)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source);

        let mut metrics = Vec::new();
        while let Some(query_match) = matches.next() {
            if let Some(method_metrics) = MatchProcessor::process_rust_match(
                &query_match,
                source,
                &parsed_file.file_path.display().to_string(),
            )? {
                metrics.push(method_metrics);
            }
        }

        Ok(metrics)
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn extract_rust_metrics(
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        Ok(Vec::new())
    }

    /// Extract metrics for Python functions
    #[cfg(feature = "tree-sitter")]
    fn extract_python_metrics(
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        use crate::analysis::detectors::anti_patterns::long_methods::language_support::python::PythonMethodAnalyzer;
        use crate::ast::tree_sitter::QueryCursor;
        use streaming_iterator::StreamingIterator;

        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        let query = PythonMethodAnalyzer::create_query(&language)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source);

        let mut metrics = Vec::new();
        while let Some(query_match) = matches.next() {
            if let Some(method_metrics) = MatchProcessor::process_python_match(
                &query_match,
                source,
                &parsed_file.file_path.display().to_string(),
            )? {
                metrics.push(method_metrics);
            }
        }

        Ok(metrics)
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn extract_python_metrics(
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        Ok(Vec::new())
    }

    /// Extract metrics for JavaScript/TypeScript functions
    #[cfg(feature = "tree-sitter")]
    fn extract_javascript_metrics(
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        use crate::analysis::detectors::anti_patterns::long_methods::language_support::typescript::TypeScriptMethodAnalyzer;
        use crate::ast::tree_sitter::QueryCursor;
        use streaming_iterator::StreamingIterator;

        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        let query = TypeScriptMethodAnalyzer::create_query(&language)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source);

        let mut metrics = Vec::new();
        while let Some(query_match) = matches.next() {
            if let Some(method_metrics) = MatchProcessor::process_javascript_match(
                &query_match,
                source,
                &parsed_file.file_path.display().to_string(),
            )? {
                metrics.push(method_metrics);
            }
        }

        Ok(metrics)
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn extract_javascript_metrics(
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        Ok(Vec::new())
    }
}
