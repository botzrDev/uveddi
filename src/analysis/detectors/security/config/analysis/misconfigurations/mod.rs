//! Misconfiguration detection in configuration files
//!
//! This module detects insecure service configurations, weak encryption
//! settings, exposed debug endpoints, and overly permissive CORS settings.

pub mod analyzer;
pub mod rules;
pub mod patterns;
pub mod checks;

// Re-export main analyzer
pub use analyzer::MisconfigurationAnalyzer;
pub use rules::{MisconfigurationRule, MisconfigurationPattern};
pub use patterns::PatternBuilder;
pub use checks::{DebugChecker, SessionChecker, DatabaseChecker};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_misconfiguration_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();
        // Basic creation test
        assert!(true);
    }
}