//! Environment variable file security analysis
//!
//! This module provides specialized security analysis for .env files and
//! other environment variable configuration files.

pub mod analyzer;
pub mod parser;
pub mod secret_patterns;
pub mod validator;

// Re-export main analyzer
pub use analyzer::EnvAnalyzer;
pub use parser::EnvParser;
pub use secret_patterns::EnvSecretPatternChecker;
pub use validator::EnvValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::language_support::LanguageAnalyzer;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_env_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();
        assert_eq!(
            analyzer.supported_type(),
            crate::analysis::detectors::security::config::types::ConfigType::Environment
        );
    }
}
