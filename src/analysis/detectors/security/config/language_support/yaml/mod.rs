//! YAML and JSON configuration security analysis
//!
//! This module provides specialized security analysis for YAML and JSON
//! configuration files, including schema validation and structural analysis.

pub mod container_patterns;
pub mod database_patterns;
pub mod kubernetes_patterns;
pub mod parser;
pub mod patterns;
pub mod validator;

// Re-export main analyzer and key types
pub use container_patterns::ContainerPatternChecker;
pub use database_patterns::DatabasePatternChecker;
pub use kubernetes_patterns::KubernetesPatternChecker;
pub use parser::YamlParser;
pub use patterns::{YamlPatternMatcher, YamlSecurityRule};
pub use validator::YamlAnalyzer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::language_support::LanguageAnalyzer;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_yaml_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = YamlAnalyzer::new(&config).unwrap();
        assert_eq!(
            analyzer.supported_type(),
            crate::analysis::detectors::security::config::types::ConfigType::Yaml
        );
    }
}
