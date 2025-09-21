//! Configuration types for OWASP security analysis

use super::types::OwaspCategory;
use serde::{Deserialize, Serialize};

/// Configuration for the OWASP detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwaspConfig {
    /// Minimum confidence threshold for reporting vulnerabilities
    pub confidence_threshold: f64,
    /// Enable/disable specific OWASP categories
    pub enabled_categories: Vec<OwaspCategory>,
    /// Enable AI-enhanced analysis
    pub enable_ai_analysis: bool,
    /// Enable false positive reduction
    pub enable_false_positive_reduction: bool,
    /// Maximum analysis time per file (seconds)
    pub max_analysis_time: u64,
}

impl Default for OwaspConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            enabled_categories: vec![
                OwaspCategory::BrokenAccessControl,
                OwaspCategory::CryptographicFailures,
                OwaspCategory::Injection,
                OwaspCategory::InsecureDesign,
                OwaspCategory::SecurityMisconfiguration,
                OwaspCategory::VulnerableComponents,
                OwaspCategory::AuthenticationFailures,
                OwaspCategory::DataIntegrityFailures,
                OwaspCategory::LoggingFailures,
                OwaspCategory::ServerSideRequestForgery,
            ],
            enable_ai_analysis: false,
            enable_false_positive_reduction: true,
            max_analysis_time: 300,
        }
    }
}

impl OwaspConfig {
    /// Create a new configuration with high sensitivity
    pub fn high_sensitivity() -> Self {
        Self {
            confidence_threshold: 0.3,
            ..Default::default()
        }
    }

    /// Create a new configuration with low false positives
    pub fn low_false_positives() -> Self {
        Self {
            confidence_threshold: 0.9,
            enable_false_positive_reduction: true,
            ..Default::default()
        }
    }

    /// Check if a category is enabled
    pub fn is_category_enabled(&self, category: &OwaspCategory) -> bool {
        self.enabled_categories.contains(category)
    }
}
