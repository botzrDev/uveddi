//! TOML configuration security analysis
//!
//! This module provides specialized security analysis for TOML configuration
//! files, commonly used in Rust projects (Cargo.toml) and Python (pyproject.toml).

pub mod analyzer;
pub mod build_checker;
pub mod credential_checker;
pub mod dependency_checker;
pub mod python_deps;
pub mod rust_deps;

// Re-export main analyzer
pub use analyzer::TomlAnalyzer;
pub use build_checker::TomlBuildChecker;
pub use credential_checker::TomlCredentialChecker;
pub use dependency_checker::TomlDependencyChecker;
pub use python_deps::PythonDependencyChecker;
pub use rust_deps::RustDependencyChecker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::language_support::LanguageAnalyzer;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_toml_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();
        assert_eq!(
            analyzer.supported_type(),
            crate::analysis::detectors::security::config::types::ConfigType::Toml
        );
    }
}
