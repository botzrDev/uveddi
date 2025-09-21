//! YAML/JSON security validation and analysis
//!
//! This module provides the main analyzer implementation for YAML and JSON
//! configuration files with comprehensive security validation.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity, ConfigType};
use super::super::{utils, LanguageAnalyzer};
use super::parser::YamlParser;
use super::patterns::YamlPatternMatcher;
use crate::analysis::AnalysisError;
use serde_yaml::Value as YamlValue;

/// Main analyzer for YAML and JSON configuration files
pub struct YamlAnalyzer {
    config: ConfigSecurityConfig,
    pattern_matcher: YamlPatternMatcher,
}

impl LanguageAnalyzer for YamlAnalyzer {
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let pattern_matcher = YamlPatternMatcher::new(config)?;

        Ok(Self {
            config: config.clone(),
            pattern_matcher,
        })
    }

    fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Parse the content
        let yaml_value = YamlParser::parse_content(content)?;

        // Apply security patterns
        issues.extend(self.pattern_matcher.apply_rules(&yaml_value, content));

        // Perform structural analysis
        issues.extend(self.analyze_structure(&yaml_value, content)?);

        Ok(issues)
    }

    fn supported_type(&self) -> ConfigType {
        ConfigType::Yaml
    }

    fn validate_syntax(&self, content: &str) -> Result<(), AnalysisError> {
        YamlParser::validate_syntax(content)
    }
}

impl YamlAnalyzer {
    /// Analyze the structural aspects of the configuration
    fn analyze_structure(
        &self,
        value: &YamlValue,
        _content: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for overly complex structures
        let depth = Self::calculate_depth(value);
        if depth > 10 {
            issues.push(utils::create_config_issue(
                ConfigSeverity::Info,
                "Complex Configuration Structure",
                format!("Configuration has deep nesting (depth: {})", depth),
                None,
                "Consider flattening the configuration structure for better maintainability",
                vec!["structure".to_string(), "maintainability".to_string()],
            ));
        }

        // Check for very large configurations
        let node_count = Self::count_nodes(value);
        if node_count > 1000 {
            issues.push(utils::create_config_issue(
                ConfigSeverity::Info,
                "Large Configuration File",
                format!("Configuration contains {} nodes", node_count),
                None,
                "Consider splitting large configurations into smaller, focused files",
                vec!["structure".to_string(), "maintainability".to_string()],
            ));
        }

        // Check for duplicate keys at the same level
        issues.extend(self.check_duplicate_keys(value)?);

        Ok(issues)
    }

    /// Calculate the maximum depth of the YAML/JSON structure
    fn calculate_depth(value: &YamlValue) -> usize {
        match value {
            YamlValue::Mapping(map) => {
                1 + map.values().map(Self::calculate_depth).max().unwrap_or(0)
            }
            YamlValue::Sequence(seq) => {
                1 + seq.iter().map(Self::calculate_depth).max().unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// Count the total number of nodes in the structure
    fn count_nodes(value: &YamlValue) -> usize {
        match value {
            YamlValue::Mapping(map) => 1 + map.values().map(Self::count_nodes).sum::<usize>(),
            YamlValue::Sequence(seq) => 1 + seq.iter().map(Self::count_nodes).sum::<usize>(),
            _ => 1,
        }
    }

    /// Check for potential duplicate key issues
    fn check_duplicate_keys(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
        self.check_duplicate_keys_recursive(value, "", &mut issues);
        Ok(issues)
    }

    fn check_duplicate_keys_recursive(
        &self,
        value: &YamlValue,
        path: &str,
        issues: &mut Vec<ConfigIssue>,
    ) {
        match value {
            YamlValue::Mapping(map) => {
                // Check for case-insensitive duplicates
                let mut seen_keys: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();

                for key in map.keys() {
                    if let Some(key_str) = key.as_str() {
                        let key_lower = key_str.to_lowercase();
                        if let Some(existing_key) = seen_keys.get(&key_lower) {
                            if existing_key != key_str {
                                let current_path = if path.is_empty() {
                                    key_str.to_string()
                                } else {
                                    format!("{}.{}", path, key_str)
                                };

                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::Medium,
                                    "Case-Insensitive Duplicate Keys",
                                    format!(
                                        "Keys '{}' and '{}' differ only in case at path: {}",
                                        existing_key, key_str, current_path
                                    ),
                                    None,
                                    "Use consistent key naming to avoid confusion",
                                    vec!["structure".to_string(), "naming".to_string()],
                                ));
                            }
                        } else {
                            seen_keys.insert(key_lower, key_str.to_string());
                        }
                    }
                }

                // Recurse into nested structures
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
                        let new_path = if path.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", path, key_str)
                        };
                        self.check_duplicate_keys_recursive(val, &new_path, issues);
                    }
                }
            }
            YamlValue::Sequence(seq) => {
                for (index, item) in seq.iter().enumerate() {
                    let new_path = if path.is_empty() {
                        format!("[{}]", index)
                    } else {
                        format!("{}[{}]", path, index)
                    };
                    self.check_duplicate_keys_recursive(item, &new_path, issues);
                }
            }
            _ => {}
        }
    }

    /// Check for environment-specific configurations that might be insecure
    pub fn check_environment_config(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            // Look for environment-specific sections
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let key_lower = key_str.to_lowercase();

                    // Check for hardcoded production configs
                    if key_lower.contains("prod") || key_lower.contains("production") {
                        if let YamlValue::Mapping(env_map) = val {
                            for (env_key, env_val) in env_map {
                                if let Some(env_key_str) = env_key.as_str() {
                                    if utils::is_sensitive_key(env_key_str) {
                                        if let Some(env_val_str) = env_val.as_str() {
                                            if YamlParser::looks_like_credential(env_val_str) {
                                                issues.push(utils::create_config_issue(
                                                    ConfigSeverity::Critical,
                                                    "Production Credential in Config",
                                                    format!("Production credential found in configuration at {}.{}", key_str, env_key_str),
                                                    None,
                                                    "Never store production credentials in configuration files",
                                                    vec!["production".to_string(), "credential".to_string()],
                                                ).with_cwe(798));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = YamlAnalyzer::new(&config).unwrap();
        assert_eq!(analyzer.supported_type(), ConfigType::Yaml);
    }

    #[test]
    fn test_basic_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = YamlAnalyzer::new(&config).unwrap();

        let yaml_content = r#"
database:
  password: "SuperSecret123!"
"#;

        let issues = analyzer.analyze(yaml_content).unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_syntax_validation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = YamlAnalyzer::new(&config).unwrap();

        assert!(analyzer.validate_syntax("key: value").is_ok());
        assert!(analyzer.validate_syntax(r#"{"key": "value"}"#).is_ok());
        assert!(analyzer.validate_syntax("key: value: invalid").is_err());
    }
}
