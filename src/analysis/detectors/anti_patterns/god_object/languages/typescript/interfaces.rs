//! TypeScript interface analysis

use super::super::super::config::GodObjectConfig;
use super::super::super::metrics::MetricsCalculator;
use super::queries::TYPESCRIPT_INTERFACE_QUERY;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use streaming_iterator::StreamingIterator;
use tracing::debug;

/// TypeScript interface analyzer
pub struct TypeScriptInterfaceAnalyzer<'a> {
    config: &'a GodObjectConfig,
}

impl<'a> TypeScriptInterfaceAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self { config }
    }

    /// Analyze TypeScript interfaces for God Interface patterns
    pub fn analyze_interfaces(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        let interface_query = Query::new(&language, TYPESCRIPT_INTERFACE_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&interface_query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            let name_node = mat.captures[0].node;
            let body_node = mat.captures[1].node;
            let container_node = name_node.parent().unwrap_or(name_node);

            let name = name_node.utf8_text(source).unwrap_or("Unnamed");

            // Count properties in interface
            let property_count = MetricsCalculator::calculate_field_count(parsed_file, body_node)?;

            // Interface-specific threshold (lower than classes)
            let property_threshold = 8;

            debug!(
                "Analyzing TypeScript interface {}: {} properties (threshold: >{})",
                name, property_count, property_threshold
            );

            if property_count > property_threshold {
                let severity = match property_count {
                    0..=12 => "Medium",
                    13..=20 => "High",
                    _ => "Critical",
                };

                let description = format!(
                    "God Interface detected: '{}' has {} properties. Interfaces should be focused and cohesive. (Threshold: >{})",
                    name, property_count, property_threshold
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
