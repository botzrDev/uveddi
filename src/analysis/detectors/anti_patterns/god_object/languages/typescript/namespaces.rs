//! TypeScript namespace analysis

use super::super::super::config::GodObjectConfig;
use super::queries::TYPESCRIPT_NAMESPACE_QUERY;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use streaming_iterator::StreamingIterator;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use tracing::debug;

/// TypeScript namespace analyzer
pub struct TypeScriptNamespaceAnalyzer<'a> {
    config: &'a GodObjectConfig,
}

impl<'a> TypeScriptNamespaceAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self { config }
    }

    /// Analyze TypeScript namespaces for God Namespace patterns
    pub fn analyze_namespaces(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        let namespace_query = Query::new(&language, TYPESCRIPT_NAMESPACE_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&namespace_query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            let name_node = mat.captures[0].node;
            let body_node = mat.captures[1].node;
            let container_node = name_node.parent().unwrap_or(name_node);

            let name = name_node.utf8_text(source).unwrap_or("Unnamed");

            // Count declarations in namespace
            let declaration_count = body_node.child_count();
            let namespace_threshold = 20; // Namespaces can be larger than classes

            debug!(
                "Analyzing TypeScript namespace {}: {} declarations (threshold: >{})",
                name, declaration_count, namespace_threshold
            );

            if declaration_count > namespace_threshold {
                let severity = match declaration_count {
                    0..=30 => "Medium",
                    31..=50 => "High",
                    _ => "Critical",
                };

                let description = format!(
                    "God Namespace detected: '{}' has {} declarations. Consider splitting into multiple namespaces. (Threshold: >{})",
                    name, declaration_count, namespace_threshold
                );

                let mut issue = ArchitecturalIssue::new(
                    0, // analysis_run_id will be set by the engine
                    1, // anti_pattern_type_id for God Object
                    parsed_file.file_path.display().to_string(),
                    Some((name_node.start_position().row + 1) as i32),
                    description.clone(),
                    "GodObjectDetector".to_string(),
                    severity.to_string(),
                    description.clone(),
                );
                issue.start_line = Some((name_node.start_position().row + 1) as i32);
                issue.end_line = Some((name_node.end_position().row + 1) as i32);
                issue.code_snippet = Some(
                    container_node
                        .utf8_text(parsed_file.source.as_bytes())
                        .unwrap_or("")
                        .to_string(),
                );
                issues.push(issue);
            }
        }

        Ok(issues)
    }
}
