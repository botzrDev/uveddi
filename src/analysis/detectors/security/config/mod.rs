//! Configuration security analysis module
//!
//! This module provides comprehensive security analysis for configuration files
//! including credential exposure detection, misconfiguration analysis, and
//! compliance validation across multiple configuration formats.

pub mod analysis;
pub mod config;
pub mod detector;
pub mod language_support;
pub mod patterns;
pub mod settings;
pub mod types;
pub mod validation;

// Re-export public API
pub use crate::analysis::detectors::security::agents::config::{AgentConfig, MultiAgentConfig};
pub use config::ConfigSecurityConfig;
pub use detector::ConfigSecurityDetector;
pub use settings::{FalsePositiveConfig, LanguageConfig, SecurityConfig, TaintAnalysisConfig};
pub use types::{
    ConfigAnalysisContext, ConfigAnalysisResult, ConfigIssue, ConfigSeverity, ConfigType,
    PatternMatch,
};

// Re-export analysis components
pub use analysis::{
    CredentialAnalyzer, DefaultsAnalyzer, MisconfigurationAnalyzer, PermissionAnalyzer,
};

// Re-export language support
pub use language_support::{EnvAnalyzer, TomlAnalyzer, YamlAnalyzer};

// Re-export pattern matchers
pub use patterns::{SecretPatternMatcher, VulnerabilityPatternMatcher};

// Re-export validators
pub use validation::{ComplianceValidator, PolicyValidator};

use crate::analysis::AnalysisError;
use std::path::PathBuf;

/// Convenience function to analyze a configuration file
pub async fn analyze_config_file(
    file_path: &PathBuf,
    content: &str,
    config: Option<ConfigSecurityConfig>,
) -> Result<Vec<crate::analysis::detectors::security::types::SecurityIssue>, AnalysisError> {
    let config = config.unwrap_or_default();
    let detector = ConfigSecurityDetector::new(config)?;
    detector.analyze_config(file_path, content).await
}

/// Check if a file is a supported configuration file
pub fn is_supported_config_file(file_path: &PathBuf) -> bool {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        matches!(
            extension.to_lowercase().as_str(),
            "yaml" | "yml" | "json" | "toml" | "env"
        )
    } else {
        false
    }
}

/// Get the configuration type for a file
pub fn get_config_type(file_path: &PathBuf) -> Option<ConfigType> {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => Some(ConfigType::Yaml),
            "json" => Some(ConfigType::Json),
            "toml" => Some(ConfigType::Toml),
            "env" => Some(ConfigType::Environment),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_supported_config_file() {
        assert!(is_supported_config_file(&PathBuf::from("config.yaml")));
        assert!(is_supported_config_file(&PathBuf::from("Cargo.toml")));
        assert!(is_supported_config_file(&PathBuf::from(".env")));
        assert!(!is_supported_config_file(&PathBuf::from("main.rs")));
    }

    #[test]
    fn test_get_config_type() {
        assert_eq!(
            get_config_type(&PathBuf::from("config.yaml")),
            Some(ConfigType::Yaml)
        );
        assert_eq!(
            get_config_type(&PathBuf::from("package.json")),
            Some(ConfigType::Json)
        );
        assert_eq!(
            get_config_type(&PathBuf::from("Cargo.toml")),
            Some(ConfigType::Toml)
        );
        assert_eq!(
            get_config_type(&PathBuf::from(".env")),
            Some(ConfigType::Environment)
        );
        assert_eq!(get_config_type(&PathBuf::from("main.rs")), None);
    }

    #[tokio::test]
    async fn test_analyze_config_file_convenience() {
        let yaml_content = r#"
database:
  password: "test123"
"#;

        let file_path = PathBuf::from("/test/config.yaml");
        let result = analyze_config_file(&file_path, yaml_content, None).await;
        assert!(result.is_ok());
    }
}
