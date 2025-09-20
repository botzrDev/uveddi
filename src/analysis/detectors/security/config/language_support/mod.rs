//! Language-specific configuration analysis support
//!
//! This module provides specialized analysis for different configuration
//! file formats including YAML, TOML, JSON, and environment files.

pub mod env;
pub mod toml;
pub mod yaml;

// Re-export analyzers
pub use env::EnvAnalyzer;
pub use toml::TomlAnalyzer;
pub use yaml::YamlAnalyzer;

use crate::analysis::AnalysisError;
use super::config::ConfigSecurityConfig;
use super::types::{ConfigIssue, ConfigType};

/// Trait for language-specific configuration analyzers
pub trait LanguageAnalyzer {
    /// Create a new analyzer instance
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError>
    where
        Self: Sized;

    /// Analyze configuration content for security issues
    fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError>;

    /// Get the supported configuration type
    fn supported_type(&self) -> ConfigType;

    /// Validate the syntax of the configuration
    fn validate_syntax(&self, content: &str) -> Result<(), AnalysisError>;
}

/// Factory function to create the appropriate analyzer for a configuration type
pub fn create_analyzer(
    config_type: ConfigType,
    config: &ConfigSecurityConfig,
) -> Result<Box<dyn LanguageAnalyzer>, AnalysisError> {
    match config_type {
        ConfigType::Yaml | ConfigType::Json => {
            Ok(Box::new(YamlAnalyzer::new(config)?))
        }
        ConfigType::Toml => {
            Ok(Box::new(TomlAnalyzer::new(config)?))
        }
        ConfigType::Environment => {
            Ok(Box::new(EnvAnalyzer::new(config)?))
        }
    }
}

/// Common utilities for language analyzers
pub mod utils {
    use super::super::types::{ConfigIssue, ConfigSeverity};

    /// Extract string values from nested configuration structures
    pub fn extract_string_values(value: &serde_json::Value, path: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        extract_strings_recursive(value, path, &mut results);
        results
    }

    fn extract_strings_recursive(
        value: &serde_json::Value,
        current_path: &str,
        results: &mut Vec<(String, String)>,
    ) {
        match value {
            serde_json::Value::String(s) => {
                results.push((current_path.to_string(), s.clone()));
            }
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let new_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", current_path, key)
                    };
                    extract_strings_recursive(val, &new_path, results);
                }
            }
            serde_json::Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", current_path, index);
                    extract_strings_recursive(val, &new_path, results);
                }
            }
            _ => {}
        }
    }

    /// Check if a configuration key suggests sensitive data
    pub fn is_sensitive_key(key: &str) -> bool {
        let sensitive_patterns = [
            "password", "passwd", "pwd", "secret", "key", "token", "auth",
            "credential", "private", "api_key", "access_key", "private_key",
            "cert", "certificate", "ssl", "tls", "oauth", "bearer",
        ];

        let key_lower = key.to_lowercase();
        sensitive_patterns.iter().any(|&pattern| key_lower.contains(pattern))
    }

    /// Create a standardized configuration issue
    pub fn create_config_issue(
        severity: ConfigSeverity,
        title: impl Into<String>,
        description: impl Into<String>,
        line_number: Option<usize>,
        remediation: impl Into<String>,
        tags: Vec<String>,
    ) -> ConfigIssue {
        let mut issue = ConfigIssue::new(severity, 0.8, title, description);

        if let Some(line) = line_number {
            issue = issue.with_location(line, 0);
        }

        issue = issue.with_remediation(remediation);

        for tag in tags {
            issue = issue.with_tag(tag);
        }

        issue
    }

    /// Calculate confidence based on context
    pub fn calculate_context_confidence(base_confidence: f64, context: &str) -> f64 {
        let mut confidence = base_confidence;

        // Reduce confidence for test/example contexts
        let test_indicators = ["test", "example", "demo", "sample", "dev", "development"];
        if test_indicators.iter().any(|&indicator| context.to_lowercase().contains(indicator)) {
            confidence *= 0.5;
        }

        // Increase confidence for production contexts
        let prod_indicators = ["prod", "production", "live", "release"];
        if prod_indicators.iter().any(|&indicator| context.to_lowercase().contains(indicator)) {
            confidence = (confidence * 1.2).min(1.0);
        }

        confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;

    #[test]
    fn test_create_analyzer_factory() {
        let config = ConfigSecurityConfig::default();

        let yaml_analyzer = create_analyzer(ConfigType::Yaml, &config).unwrap();
        assert_eq!(yaml_analyzer.supported_type(), ConfigType::Yaml);

        let toml_analyzer = create_analyzer(ConfigType::Toml, &config).unwrap();
        assert_eq!(toml_analyzer.supported_type(), ConfigType::Toml);

        let env_analyzer = create_analyzer(ConfigType::Environment, &config).unwrap();
        assert_eq!(env_analyzer.supported_type(), ConfigType::Environment);
    }

    #[test]
    fn test_sensitive_key_detection() {
        assert!(is_sensitive_key("password"));
        assert!(is_sensitive_key("api_key"));
        assert!(is_sensitive_key("JWT_SECRET"));
        assert!(is_sensitive_key("database_password"));
        assert!(!is_sensitive_key("timeout"));
        assert!(!is_sensitive_key("max_connections"));
    }

    #[test]
    fn test_context_confidence_calculation() {
        assert_eq!(calculate_context_confidence(0.8, "normal config"), 0.8);
        assert!(calculate_context_confidence(0.8, "test environment") < 0.8);
        assert!(calculate_context_confidence(0.8, "production database") > 0.8);
    }

    #[test]
    fn test_string_value_extraction() {
        let json_str = r#"
        {
            "database": {
                "password": "secret123",
                "hosts": ["host1", "host2"]
            },
            "api_key": "key456"
        }"#;

        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let strings = extract_string_values(&value, "");

        assert!(strings.iter().any(|(path, val)| path == "database.password" && val == "secret123"));
        assert!(strings.iter().any(|(path, val)| path == "api_key" && val == "key456"));
        assert!(strings.iter().any(|(path, val)| path == "database.hosts[0]" && val == "host1"));
    }
}