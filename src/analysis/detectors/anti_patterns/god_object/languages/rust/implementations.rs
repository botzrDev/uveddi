//! Rust impl block analysis

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::error::ErrorHelpers;
use super::super::super::config::GodObjectConfig;
use super::super::super::metrics::MetricsCalculator;
use super::queries::RUST_IMPL_QUERY;
use std::collections::HashMap;

/// Rust impl block analyzer
pub struct RustImplAnalyzer<'a> {
    config: &'a GodObjectConfig,
}

impl<'a> RustImplAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self { config }
    }

    /// Count methods in impl blocks
    pub fn count_impl_methods(&self, parsed_file: &ParsedFile) -> Result<HashMap<String, usize>, AnalysisError> {
        let mut impl_method_counts: HashMap<String, usize> = HashMap::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            ErrorHelpers::ast_error("impl analysis")
        })?;
        let language = tree.language();

        let impl_query = Query::new(&language, RUST_IMPL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&impl_query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.first(), mat.captures.get(1)) {
                let name_node = name_capture.node;
                let body_node = body_capture.node;

                if let Ok(name) = name_node.utf8_text(source) {
                    let method_count = MetricsCalculator::calculate_method_count(parsed_file, body_node)?;
                    impl_method_counts.insert(name.to_string(), method_count);
                }
            }
        }

        Ok(impl_method_counts)
    }
}