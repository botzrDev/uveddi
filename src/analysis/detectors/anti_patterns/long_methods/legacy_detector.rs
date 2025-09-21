//! Legacy AnalysisDetector implementation for backward compatibility

use super::{detector::LongMethodsDetector, types::LanguageThresholds};
use crate::analysis::detectors::base::traits::Detector;
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::core::logging::{debug, info};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;

#[async_trait]
impl AnalysisDetector for LongMethodsDetector {
    async fn detect_issues(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        debug!(
            "Long Methods Detector analyzing file: {}",
            file.file_path.display()
        );

        let method_metrics = self.extract_method_metrics(file)?;
        debug!(
            "Long Methods Detector found {} method metrics",
            method_metrics.len()
        );

        let thresholds = self
            .config()
            .get_threshold(file.language)
            .ok_or_else(|| AnalysisError::UnsupportedLanguage(format!("{:?}", file.language)))?;

        for metrics in method_metrics {
            let severity_score = self.calculate_severity_score(&metrics, thresholds);
            debug!(
                "Method '{}': {} logical LOC, severity score: {} (threshold: {})",
                metrics.name, metrics.logical_loc, severity_score, thresholds.max_logical_loc
            );

            if severity_score >= self.config().min_severity_score {
                let mut issue = ArchitecturalIssue::new(
                    0, // analysis_run_id will be set by the engine
                    4, // anti_pattern_type_id for long methods
                    metrics.file_path.clone(),
                    Some(metrics.start_line as i32),
                    format!(
                        "Long method '{}' detected: {} lines, {} statements, complexity {}",
                        metrics.name,
                        metrics.logical_loc,
                        metrics.statement_count,
                        metrics.cyclomatic_complexity
                    ),
                    "LongMethodsDetector".to_string(),
                    Self::get_severity_level(severity_score),
                    format!(
                        "Long method '{}' detected: {} lines, {} statements, complexity {}",
                        metrics.name,
                        metrics.logical_loc,
                        metrics.statement_count,
                        metrics.cyclomatic_complexity
                    ),
                );

                issue.start_line = Some(metrics.start_line as i32);
                issue.end_line = Some(metrics.end_line as i32);
                issue.code_snippet = Some(metrics.code_snippet.clone());
                issue.ai_explanation = Some(format!(
                    "Consider breaking down '{}' into smaller, more focused methods. Current metrics: LOC={}, Statements={}, Complexity={}, Nesting={}",
                    metrics.name, metrics.logical_loc, metrics.statement_count,
                    metrics.cyclomatic_complexity, metrics.max_nesting_depth
                ));

                issues.push(issue);
            }
        }

        info!("Long Methods detector found {} issues", issues.len());
        Ok(issues)
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(4),
            name: "Long Method".to_string(),
            description: "Methods or functions that are excessively long and complex, making them difficult to understand, test, and maintain".to_string(),
            category: "Method-Level".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "LongMethodsDetector"
    }

    fn detect(
        &self,
        _graph: &crate::analysis::graph::dependency::LocalDependencyGraph,
    ) -> Vec<ArchitecturalIssue> {
        // For the graph-based detect method, we return empty for now
        // This method is used for dependency-based analysis
        Vec::new()
    }
}

impl LongMethodsDetector {
    /// Get severity level based on score
    pub(crate) fn get_severity_level(score: u32) -> String {
        match score {
            0..=25 => "info".to_string(),
            26..=50 => "low".to_string(),
            51..=75 => "medium".to_string(),
            76..=90 => "high".to_string(),
            _ => "critical".to_string(),
        }
    }
}
