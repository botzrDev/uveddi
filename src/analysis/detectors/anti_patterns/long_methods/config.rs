//! Configuration for long methods detector

use crate::analysis::detectors::anti_patterns::long_methods::types::LanguageThresholds;
use crate::ast::tree_sitter_impl::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the Long Methods detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongMethodsConfig {
    /// Language-specific thresholds
    pub thresholds: HashMap<SourceLanguage, LanguageThresholds>,
    /// Enable adaptive threshold calculation
    pub enable_adaptive_thresholds: bool,
    /// Percentile for adaptive thresholds (50-99)
    pub adaptive_percentile: f64,
    /// Minimum sample size for adaptive thresholds
    pub min_sample_size: usize,
    /// Skip analysis for test files
    pub skip_test_files: bool,
    /// Skip analysis for generated files
    pub skip_generated_files: bool,
    /// Maximum number of issues to report
    pub max_issues: Option<usize>,
    /// Minimum severity level to report
    pub min_severity_score: u32,
}

impl Default for LongMethodsConfig {
    fn default() -> Self {
        use crate::analysis::detectors::anti_patterns::long_methods::thresholds::language_thresholds::create_threshold_map;

        Self {
            thresholds: create_threshold_map(),
            enable_adaptive_thresholds: false,
            adaptive_percentile: 90.0,
            min_sample_size: 50,
            skip_test_files: true,
            skip_generated_files: true,
            max_issues: Some(100),
            min_severity_score: 25,
        }
    }
}

impl LongMethodsConfig {
    /// Create a strict configuration for code review
    pub fn strict() -> Self {
        let mut config = Self::default();

        // Make thresholds more strict
        for threshold in config.thresholds.values_mut() {
            threshold.max_logical_loc = (threshold.max_logical_loc as f32 * 0.7) as u32;
            threshold.max_cyclomatic_complexity = (threshold.max_cyclomatic_complexity as f32 * 0.8) as u32;
            threshold.max_cognitive_complexity = (threshold.max_cognitive_complexity as f32 * 0.8) as u32;
        }

        config.min_severity_score = 10;
        config.max_issues = None;
        config
    }

    /// Create a lenient configuration for legacy code
    pub fn lenient() -> Self {
        let mut config = Self::default();

        // Make thresholds more lenient
        for threshold in config.thresholds.values_mut() {
            threshold.max_logical_loc = (threshold.max_logical_loc as f32 * 1.5) as u32;
            threshold.max_cyclomatic_complexity = (threshold.max_cyclomatic_complexity as f32 * 1.3) as u32;
            threshold.max_cognitive_complexity = (threshold.max_cognitive_complexity as f32 * 1.3) as u32;
        }

        config.min_severity_score = 50;
        config.max_issues = Some(50);
        config
    }

    /// Get threshold for a specific language
    pub fn get_threshold(&self, language: SourceLanguage) -> Option<&LanguageThresholds> {
        self.thresholds.get(&language)
    }

    /// Set custom threshold for a language
    pub fn set_threshold(&mut self, language: SourceLanguage, threshold: LanguageThresholds) {
        self.thresholds.insert(language, threshold);
    }

    /// Check if a file should be skipped
    pub fn should_skip_file(&self, file_path: &str) -> bool {
        if self.skip_test_files && is_test_file(file_path) {
            return true;
        }

        if self.skip_generated_files && is_generated_file(file_path) {
            return true;
        }

        false
    }
}

/// Check if a file is a test file
fn is_test_file(file_path: &str) -> bool {
    file_path.contains("/test/") ||
    file_path.contains("/tests/") ||
    file_path.contains("_test.") ||
    file_path.contains(".test.") ||
    file_path.ends_with("_spec.rs") ||
    file_path.ends_with("_test.rs") ||
    file_path.ends_with(".test.js") ||
    file_path.ends_with(".spec.js") ||
    file_path.ends_with("_test.py") ||
    file_path.ends_with("test_")
}

/// Check if a file is generated
fn is_generated_file(file_path: &str) -> bool {
    file_path.contains("/generated/") ||
    file_path.contains("/gen/") ||
    file_path.contains(".generated.") ||
    file_path.contains(".pb.") ||  // Protocol buffers
    file_path.contains(".g.")       // Generated files
}