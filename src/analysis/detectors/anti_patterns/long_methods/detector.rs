//! Main Long Methods detector implementation

use crate::analysis::detectors::anti_patterns::long_methods::{
    config::LongMethodsConfig,
    extractors::MethodExtractor,
    thresholds::language_thresholds::get_language_thresholds,
    types::{LanguageThresholds, LongMethodsResult, MethodMetrics},
};
use crate::analysis::detectors::base::{
    AnalysisContext, DetectionMetrics, Detector, DetectorConfig, DetectorOutput, Issue, Severity,
};
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::{ParsedFile, SourceLanguage};
use crate::core::logging::{debug, info};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;

/// Long Methods detector implementation
pub struct LongMethodsDetector {
    config: LongMethodsConfig,
}

impl Default for LongMethodsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LongMethodsDetector {
    /// Create a new Long Methods detector with default configuration
    pub fn new() -> Self {
        Self {
            config: <LongMethodsConfig as Default>::default(),
        }
    }

    /// Create detector with custom configuration
    pub fn with_config(config: LongMethodsConfig) -> Self {
        Self { config }
    }

    /// Extract method metrics from a parsed file
    pub(crate) fn extract_method_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        // Check if file should be skipped
        if self
            .config
            .should_skip_file(&parsed_file.file_path.display().to_string())
        {
            debug!("Skipping file: {}", parsed_file.file_path.display());
            return Ok(Vec::new());
        }

        MethodExtractor::extract_metrics(parsed_file)
    }

    /// Calculate severity score for a method
    pub(crate) fn calculate_severity_score(
        &self,
        metrics: &MethodMetrics,
        thresholds: &LanguageThresholds,
    ) -> u32 {
        metrics.calculate_score(thresholds)
    }

    /// Convert severity score to Severity enum
    fn score_to_severity(score: u32) -> Severity {
        match score {
            0..=25 => Severity::Info,
            26..=50 => Severity::Low,
            51..=75 => Severity::Medium,
            76..=90 => Severity::High,
            _ => Severity::Critical,
        }
    }

    /// Create an issue from method metrics
    fn create_issue(&self, metrics: &MethodMetrics, thresholds: &LanguageThresholds) -> Issue {
        let score = self.calculate_severity_score(metrics, thresholds);
        let severity = Self::score_to_severity(score);

        let title = format!("Long method '{}' detected", metrics.name);
        let description = format!(
            "Method '{}' exceeds length thresholds: {} logical lines (max: {}), {} statements (max: {}), complexity {} (max: {})",
            metrics.name,
            metrics.logical_loc,
            thresholds.max_logical_loc,
            metrics.statement_count,
            thresholds.max_statements,
            metrics.cyclomatic_complexity,
            thresholds.max_cyclomatic_complexity
        );

        let suggestions = metrics.get_refactoring_suggestions(thresholds);
        let suggestion_text = if suggestions.is_empty() {
            "Consider breaking this method into smaller, more focused functions".to_string()
        } else {
            suggestions.join("; ")
        };

        Issue::new(
            format!("long_method_{}", metrics.name),
            title,
            description,
            severity,
            metrics.file_path.clone(),
            metrics.start_line,
            metrics.end_line,
        )
        .with_suggestion(suggestion_text)
        .with_metadata(
            "logical_loc".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.logical_loc)),
        )
        .with_metadata(
            "statement_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.statement_count)),
        )
        .with_metadata(
            "cyclomatic_complexity".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.cyclomatic_complexity)),
        )
        .with_metadata(
            "cognitive_complexity".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.cognitive_complexity)),
        )
        .with_metadata(
            "max_nesting_depth".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.max_nesting_depth)),
        )
        .with_metadata(
            "parameter_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.parameter_count)),
        )
        .with_metadata(
            "method_type".to_string(),
            serde_json::Value::String(metrics.method_type.clone()),
        )
        .with_metadata(
            "is_exported".to_string(),
            serde_json::Value::Bool(metrics.is_exported),
        )
    }

    /// Filter issues based on configuration
    fn filter_issues(&self, mut issues: Vec<Issue>) -> Vec<Issue> {
        // Sort by severity (highest first)
        issues.sort_by(|a, b| b.severity.cmp(&a.severity));

        // Apply maximum issues limit
        if let Some(max_issues) = self.config.max_issues {
            issues.truncate(max_issues);
        }

        issues
    }
}

#[async_trait]
impl Detector for LongMethodsDetector {
    type Config = LongMethodsConfig;
    type Output = LongMethodsResult;

    fn name(&self) -> &'static str {
        "long_methods"
    }

    fn category(&self) -> crate::analysis::detectors::base::DetectorCategory {
        crate::analysis::detectors::base::DetectorCategory::AntiPattern
    }

    fn supported_languages(&self) -> &[SourceLanguage] {
        &[
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ]
    }

    async fn detect(&self, context: &AnalysisContext) -> Result<Self::Output, AnalysisError> {
        let mut all_methods = Vec::new();
        let mut all_issues = Vec::new();
        let mut total_methods_analyzed = 0;
        let mut methods_exceeding_thresholds = 0;

        for parsed_file in &context.files {
            let method_metrics = self.extract_method_metrics(parsed_file)?;
            total_methods_analyzed += method_metrics.len();

            if let Some(thresholds) = self.config.get_threshold(parsed_file.language) {
                for metrics in method_metrics {
                    let score = self.calculate_severity_score(&metrics, thresholds);
                    if score >= self.config.min_severity_score {
                        methods_exceeding_thresholds += 1;
                        all_issues.push(self.create_issue(&metrics, thresholds));
                    }
                    all_methods.push(metrics);
                }
            }
        }

        Ok(LongMethodsResult {
            methods: all_methods,
            issues: all_issues,
            total_methods_analyzed,
            methods_exceeding_thresholds,
        })
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DetectorConfig for LongMethodsConfig {
    fn validate(&self) -> Result<(), AnalysisError> {
        if self.adaptive_percentile < 50.0 || self.adaptive_percentile > 99.0 {
            return Err(AnalysisError::ConfigurationError {
                field: "adaptive_percentile".to_string(),
                value: self.adaptive_percentile.to_string(),
                reason: "must be between 50.0 and 99.0".to_string(),
            });
        }

        if self.min_sample_size == 0 {
            return Err(AnalysisError::ConfigurationError {
                field: "min_sample_size".to_string(),
                value: self.min_sample_size.to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        for (language, thresholds) in &self.thresholds {
            if thresholds.max_logical_loc == 0 {
                return Err(AnalysisError::ConfigurationError {
                    field: format!("max_logical_loc[{:?}]", language),
                    value: thresholds.max_logical_loc.to_string(),
                    reason: "must be greater than 0".to_string(),
                });
            }
        }

        Ok(())
    }

    fn merge(&mut self, other: Self) {
        // Merge thresholds, preferring the other config's values
        for (language, threshold) in other.thresholds {
            self.thresholds.insert(language, threshold);
        }

        // Update other settings if they're more permissive
        if other.enable_adaptive_thresholds {
            self.enable_adaptive_thresholds = true;
        }
        if other.adaptive_percentile > self.adaptive_percentile {
            self.adaptive_percentile = other.adaptive_percentile;
        }
        if other.min_sample_size < self.min_sample_size {
            self.min_sample_size = other.min_sample_size;
        }
        if !other.skip_test_files {
            self.skip_test_files = false;
        }
        if !other.skip_generated_files {
            self.skip_generated_files = false;
        }
        if let Some(max) = other.max_issues {
            self.max_issues = Some(max);
        }
        if other.min_severity_score < self.min_severity_score {
            self.min_severity_score = other.min_severity_score;
        }
    }

    fn default() -> Self {
        <LongMethodsConfig as Default>::default()
    }
}

impl DetectorOutput for LongMethodsResult {
    fn severity(&self) -> Severity {
        if self.issues.is_empty() {
            Severity::Info
        } else {
            // Return the highest severity
            self.issues
                .iter()
                .map(|issue| issue.severity.clone())
                .max()
                .unwrap_or(Severity::Info)
        }
    }

    fn issues(&self) -> &[Issue] {
        &self.issues
    }

    fn metrics(&self) -> Option<DetectionMetrics> {
        use std::collections::HashMap;

        Some(DetectionMetrics {
            duration_ms: 0,              // This would need to be tracked properly
            files_analyzed: 1,           // This would need to be tracked properly
            nodes_processed: 0,          // This would need to be tracked
            memory_usage_bytes: Some(0), // This would need to be tracked
            custom_metrics: HashMap::new(),
        })
    }

    fn combine(mut self, other: Self) -> Self {
        self.methods.extend(other.methods);
        self.issues.extend(other.issues);
        self.total_methods_analyzed += other.total_methods_analyzed;
        self.methods_exceeding_thresholds += other.methods_exceeding_thresholds;
        self
    }
}
