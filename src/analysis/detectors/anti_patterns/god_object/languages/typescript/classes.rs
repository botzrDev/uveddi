//! TypeScript class analysis

use super::super::super::config::GodObjectConfig;
use super::super::super::detector::{ComplexityMetrics, DetectedPattern};
use super::super::super::metrics::MetricsCalculator;
use super::patterns::TypeScriptPatternDetector;
use super::queries::TYPESCRIPT_CLASS_QUERY;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use std::collections::HashSet;
use streaming_iterator::StreamingIterator;
use tracing::debug;

/// TypeScript class analyzer
pub struct TypeScriptClassAnalyzer<'a> {
    config: &'a GodObjectConfig,
    pattern_detector: TypeScriptPatternDetector<'a>,
}

impl<'a> TypeScriptClassAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self {
            config,
            pattern_detector: TypeScriptPatternDetector::new(config),
        }
    }

    /// Analyze TypeScript classes for God Object patterns
    pub fn analyze_classes(
        &self,
        parsed_file: &ParsedFile,
        detected_frameworks: &HashSet<String>,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        let class_query = Query::new(&language, TYPESCRIPT_CLASS_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&class_query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            let name_node = mat.captures[0].node;
            let body_node = mat
                .captures
                .iter()
                .find(|c| c.node.kind() == "class_body")
                .map(|c| c.node)
                .unwrap_or(mat.captures[1].node);
            let container_node = name_node.parent().unwrap_or(name_node);

            let name = name_node.utf8_text(source).unwrap_or("Unnamed");

            // Calculate metrics
            let method_count = MetricsCalculator::calculate_method_count(parsed_file, body_node)?;
            let field_count = MetricsCalculator::calculate_field_count(parsed_file, body_node)?;

            let method_threshold = self.config.get_method_threshold(parsed_file.language);
            let field_threshold = self.config.get_field_threshold(parsed_file.language);

            debug!(
                "Analyzing TypeScript class {}: {} methods, {} fields (thresholds: >{}, >{})",
                name, method_count, field_count, method_threshold, field_threshold
            );

            // Only proceed if thresholds are exceeded
            if method_count <= method_threshold && field_count <= field_threshold {
                continue;
            }

            // TypeScript-specific Pattern Recognition
            let excluded_pattern = if self.config.recognize_patterns {
                self.pattern_detector.detect_patterns(
                    name,
                    method_count,
                    field_count,
                    detected_frameworks,
                )
            } else {
                None
            };

            // Create metrics and issue
            let mut metrics = MetricsCalculator::calculate_metrics(parsed_file, container_node)?;
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
