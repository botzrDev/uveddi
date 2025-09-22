use serde::{Deserialize, Serialize};

/// Configuration for the SQL injection detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlInjectionDetectorConfig {
    /// Enable generic pattern-based detection (regex heuristics).
    pub enable_pattern_analysis: bool,
    /// Enable deeper analysis of dynamic parameter construction.
    pub enable_parameter_analysis: bool,
    /// Enable sanitization validation heuristics.
    pub enable_sanitizer_validation: bool,
}

impl Default for SqlInjectionDetectorConfig {
    fn default() -> Self {
        Self {
            enable_pattern_analysis: true,
            enable_parameter_analysis: true,
            enable_sanitizer_validation: true,
        }
    }
}
