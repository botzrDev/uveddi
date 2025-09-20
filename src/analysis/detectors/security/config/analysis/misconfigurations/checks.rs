//! Configuration security checkers
//!
//! This module provides specialized checkers for different types of
//! configuration security issues.

pub mod debug_checker;
pub mod session_checker;
pub mod database_checker;

// Re-export checkers
pub use debug_checker::DebugChecker;
pub use session_checker::SessionChecker;
pub use database_checker::DatabaseChecker;

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