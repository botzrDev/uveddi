//! Rust struct analysis

use super::super::super::config::GodObjectConfig;
use super::super::super::detector::{ComplexityMetrics, DetectedPattern};
use super::super::super::metrics::MetricsCalculator;
use super::patterns::RustPatternDetector;
use super::queries::RUST_STRUCT_QUERY;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tracing::debug;

/// Rust struct analyzer
pub struct RustStructAnalyzer<'a> {
    config: &'a GodObjectConfig,
    pattern_detector: RustPatternDetector<'a>,
}

impl<'a> RustStructAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self {
            config,
            pattern_detector: RustPatternDetector::new(config),
        }
    }

    /// Analyze structs for God Object patterns
    pub fn analyze_structs(
        &self,
        parsed_file: &ParsedFile,
        impl_method_counts: &HashMap<String, usize>,
        detected_frameworks: &[String],
        derive_attributes: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();
        let root_node = tree.root_node();

        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut struct_cursor = QueryCursor::new();
        let mut matches = struct_cursor.matches(&struct_query, root_node, source);

        while let Some(mat) = matches.next() {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.first(), mat.captures.get(1))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                let container_node = name_node.parent().unwrap_or(name_node);

                if let Ok(name) = name_node.utf8_text(source) {
                    let method_count = impl_method_counts.get(name).cloned().unwrap_or(0);
                    let field_count =
                        MetricsCalculator::calculate_field_count(parsed_file, body_node)?;

                    let method_threshold = self.config.get_method_threshold(parsed_file.language);
                    let field_threshold = self.config.get_field_threshold(parsed_file.language);

                    // Only proceed if thresholds are exceeded
                    if method_count <= method_threshold && field_count <= field_threshold {
                        continue;
                    }

                    // Pattern Recognition
                    let excluded_pattern = if self.config.recognize_patterns {
                        self.pattern_detector.detect_patterns(
                            name,
                            method_count,
                            field_count,
                            detected_frameworks,
                            derive_attributes,
                        )
                    } else {
                        None
                    };

                    // Create metrics and issue
                    let mut metrics =
                        MetricsCalculator::calculate_metrics(parsed_file, container_node)?;
                    metrics.method_count = method_count;
                    metrics.field_count = field_count;

                    if let Some(issue) = self.create_issue(
                        parsed_file,
                        name,
                        name_node,
                        container_node,
                        &metrics,
                        excluded_pattern,
                    ) {
                        issues.push(issue);
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Create an ArchitecturalIssue for a detected God Object
    fn create_issue(
        &self,
        parsed_file: &ParsedFile,
        name: &str,
        name_node: Node,
        container_node: Node,
        metrics: &ComplexityMetrics,
        excluded_pattern: Option<DetectedPattern>,
    ) -> Option<ArchitecturalIssue> {
        // If excluded by pattern recognition, return None
        if excluded_pattern.is_some() {
            debug!(
                "Excluding '{}' due to detected pattern: {:?}",
                name, excluded_pattern
            );
            return None;
        }

        let method_threshold = self.config.get_method_threshold(parsed_file.language);
        let field_threshold = self.config.get_field_threshold(parsed_file.language);

        let method_excess = metrics.method_count.saturating_sub(method_threshold);
        let field_excess = metrics.field_count.saturating_sub(field_threshold);

        // Only consider it an issue if at least one threshold is exceeded
        if method_excess == 0 && field_excess == 0 {
            return None;
        }

        let total_excess = method_excess + field_excess;
        let severity = match total_excess {
            0..=4 => "Medium",
            5..=10 => "High",
            _ => "Critical",
        };

        let mut description = format!(
            "God Object detected: '{}' has {} methods and {} fields. (Thresholds: methods>{}, fields>{})",
            name, metrics.method_count, metrics.field_count, method_threshold, field_threshold
        );

        if let Some(lcom4) = metrics.lcom4_score {
            description.push_str(&format!(
                " LCOM4 score: {} (>1 indicates low cohesion)",
                lcom4
            ));
        }

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

        Some(issue)
    }
}
