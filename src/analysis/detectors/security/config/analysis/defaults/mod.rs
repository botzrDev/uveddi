//! Insecure defaults detection in configuration files
//!
//! This module detects the use of insecure default values, weak default
//! passwords, default encryption keys, and other dangerous default settings.

pub mod analyzer;
pub mod password_defaults;
pub mod crypto_defaults;
pub mod database_defaults;

// Re-export main analyzer
pub use analyzer::DefaultsAnalyzer;
pub use password_defaults::PasswordDefaultChecker;
pub use crypto_defaults::CryptoDefaultChecker;
pub use database_defaults::DatabaseDefaultChecker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_defaults_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = DefaultsAnalyzer::new(&config).unwrap();
        // Basic creation test
    }
}