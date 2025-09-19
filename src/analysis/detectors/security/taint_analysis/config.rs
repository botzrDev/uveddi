//! Configuration for taint analysis detector

// Re-export the main config from parent module
pub use crate::analysis::detectors::security::config::TaintAnalysisConfig;

/// Default configuration values for taint analysis
impl Default for TaintAnalysisConfig {
    fn default() -> Self {
        Self::production()
    }
}