//! Configuration security checkers
//!
//! This module provides specialized checkers for different types of
//! configuration security issues.

// Re-export checkers for convenient access
pub use super::debug_checker::DebugChecker;
pub use super::session_checker::SessionChecker;
pub use super::database_checker::DatabaseChecker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_checker_creation() {
        let config = ConfigSecurityConfig::default();

        let _debug_checker = DebugChecker::new(&config);
        let _session_checker = SessionChecker::new(&config);
        let _database_checker = DatabaseChecker::new(&config);

        // Basic creation test
        assert!(true);
    }
}
