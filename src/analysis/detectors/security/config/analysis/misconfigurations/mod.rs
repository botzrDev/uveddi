//! Misconfiguration detection in configuration files
//!
//! This module detects insecure service configurations, weak encryption
//! settings, exposed debug endpoints, and overly permissive CORS settings.

pub mod analyzer;
pub mod checks;
pub mod database_checker;
pub mod debug_checker;
pub mod patterns;
pub mod rules;
pub mod session_checker;

// Re-export main analyzer
pub use analyzer::MisconfigurationAnalyzer;
pub use database_checker::DatabaseChecker;
pub use debug_checker::DebugChecker;
pub use patterns::PatternBuilder;
pub use rules::{MisconfigurationPattern, MisconfigurationRule};
pub use session_checker::SessionChecker;

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
