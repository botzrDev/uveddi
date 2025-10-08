//! YAML/JSON security patterns and rules
//!
//! This module contains security patterns specific to YAML and JSON
//! configuration files, including credential detection and structural analysis.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use super::container_patterns::ContainerPatternChecker;
use super::database_patterns::DatabasePatternChecker;
use super::parser::YamlParser;
use crate::analysis::AnalysisError;
use serde_yaml::Value as YamlValue;

/// Security rule for YAML/JSON analysis
pub struct YamlSecurityRule {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub checker: Box<dyn Fn(&YamlValue, &str) -> Vec<ConfigIssue> + Send + Sync>,
}

/// Pattern matcher for YAML/JSON security issues
pub struct YamlPatternMatcher {
    rules: Vec<YamlSecurityRule>,
}

impl YamlPatternMatcher {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let rules = Self::build_security_rules();
        Ok(Self { rules })
    }

    pub fn apply_rules(&self, value: &YamlValue, content: &str) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();
        for rule in &self.rules {
            issues.extend((rule.checker)(value, content));
        }
        issues
    }

    fn build_security_rules() -> Vec<YamlSecurityRule> {
        vec![
            // Credential exposure rule
            YamlSecurityRule {
                name: "Credential Exposure".to_string(),
                description: "Hardcoded credentials detected in YAML/JSON".to_string(),
                severity: ConfigSeverity::High,
                checker: Box::new(|value, _content| Self::check_credentials(value)),
            },
            // Insecure service configuration
            YamlSecurityRule {
                name: "Insecure Service Configuration".to_string(),
                description: "Insecure service settings detected".to_string(),
                severity: ConfigSeverity::Medium,
                checker: Box::new(|value, _content| Self::check_service_security(value)),
            },
            // Database security
            YamlSecurityRule {
                name: "Database Security Issues".to_string(),
                description: "Insecure database configuration detected".to_string(),
                severity: ConfigSeverity::High,
                checker: Box::new(|value, _content| {
                    DatabasePatternChecker::check_database_security(value)
                }),
            },
            // Network security
            YamlSecurityRule {
                name: "Network Security Issues".to_string(),
                description: "Insecure network configuration detected".to_string(),
                severity: ConfigSeverity::Medium,
                checker: Box::new(|value, _content| Self::check_network_security(value)),
            },
            // Container security (Kubernetes/Docker)
            YamlSecurityRule {
                name: "Container Security Issues".to_string(),
                description: "Insecure container configuration detected".to_string(),
                severity: ConfigSeverity::High,
                checker: Box::new(|value, _content| {
                    ContainerPatternChecker::check_container_security(value)
                }),
            },
        ]
    }

    fn check_credentials(value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();
        Self::check_credentials_recursive(value, "", &mut issues);
        issues
    }

    fn check_credentials_recursive(value: &YamlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
                        let new_path = if path.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", path, key_str)
                        };

                        // Check if this is a credential field
                        if utils::is_sensitive_key(key_str) {
                            if let Some(val_str) = val.as_str() {
                                if YamlParser::looks_like_credential(val_str) {
                                    issues.push(
                                        utils::create_config_issue(
                                            ConfigSeverity::High,
                                            "Hardcoded Credential in YAML/JSON",
                                            format!("Found credential at path: {}", new_path),
                                            None,
                                            "Use environment variables or secure secret management",
                                            vec!["credential".to_string(), "yaml".to_string()],
                                        )
                                        .with_cwe(798),
                                    );
                                }
                            }
                        }

                        Self::check_credentials_recursive(val, &new_path, issues);
                    }
                }
            }
            YamlValue::Sequence(seq) => {
                for (index, item) in seq.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    Self::check_credentials_recursive(item, &new_path, issues);
                }
            }
            _ => {}
        }
    }

    fn check_service_security(value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            // Check for debug settings
            Self::check_debug_settings(map, &mut issues);

            // Check for logging configuration
            Self::check_logging_settings(map, &mut issues);

            // Check for CORS settings
            Self::check_cors_settings(map, &mut issues);
        }

        issues
    }

    fn check_debug_settings(map: &serde_yaml::Mapping, issues: &mut Vec<ConfigIssue>) {
        for (key, val) in map {
            if let Some(key_str) = key.as_str() {
                if key_str.to_lowercase().contains("debug") {
                    if val.as_bool() == Some(true) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::Medium,
                                "Debug Mode Enabled",
                                "Debug mode is enabled which may expose sensitive information",
                                None,
                                "Disable debug mode in production environments",
                                vec!["debug".to_string(), "configuration".to_string()],
                            )
                            .with_cwe(489),
                        );
                    }
                }
            }
        }
    }

    fn check_logging_settings(map: &serde_yaml::Mapping, issues: &mut Vec<ConfigIssue>) {
        for (key, val) in map {
            if let Some(key_str) = key.as_str() {
                let key_lower = key_str.to_lowercase();
                if key_lower.contains("log") && key_lower.contains("level") {
                    if let Some(level_str) = val.as_str() {
                        let level_lower = level_str.to_lowercase();
                        if level_lower == "debug" || level_lower == "trace" {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::Low,
                                    "Verbose Logging Enabled",
                                    "Verbose logging may expose sensitive information in logs",
                                    None,
                                    "Use INFO or WARN log level in production",
                                    vec![
                                        "logging".to_string(),
                                        "information-disclosure".to_string(),
                                    ],
                                )
                                .with_cwe(532),
                            );
                        }
                    }
                }
            }
        }
    }

    fn check_cors_settings(map: &serde_yaml::Mapping, issues: &mut Vec<ConfigIssue>) {
        for (key, val) in map {
            if let Some(key_str) = key.as_str() {
                if key_str.to_lowercase().contains("cors") {
                    if let YamlValue::Mapping(cors_config) = val {
                        for (cors_key, cors_val) in cors_config {
                            if let Some(cors_key_str) = cors_key.as_str() {
                                if cors_key_str.to_lowercase().contains("origin") {
                                    if cors_val.as_str() == Some("*") {
                                        issues.push(utils::create_config_issue(
                                            ConfigSeverity::High,
                                            "Permissive CORS Policy",
                                            "CORS allows all origins which may enable cross-origin attacks",
                                            None,
                                            "Restrict CORS to specific trusted origins",
                                            vec!["cors".to_string(), "web-security".to_string()],
                                        ).with_cwe(942));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Database security checking is handled by DatabasePatternChecker

    fn check_network_security(value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            // Check for insecure bindings
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let key_lower = key_str.to_lowercase();
                    if key_lower.contains("bind")
                        || key_lower.contains("listen")
                        || key_lower == "host"
                    {
                        if val.as_str() == Some("0.0.0.0") {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::Medium,
                                    "Insecure Network Binding",
                                    "Service bound to all interfaces (0.0.0.0)",
                                    None,
                                    "Bind to specific interfaces instead of 0.0.0.0",
                                    vec!["network".to_string(), "binding".to_string()],
                                )
                                .with_cwe(1188),
                            );
                        }
                    }
                }
            }
        }

        issues
    }

    // Container security checking is handled by ContainerPatternChecker
}

#[cfg(test)]
mod tests {
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;
    use super::*;

    #[test]
    fn test_pattern_detection() {
        let config = ConfigSecurityConfig::default();
        let matcher = YamlPatternMatcher::new(&config).unwrap();

        let yaml_content = r#"
database:
  password: "Xy9$kL2mN8pQ"
app:
  debug: true
cors:
  allowed_origins: "*"
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        let issues = matcher.apply_rules(&parsed, yaml_content);

        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("Credential")
            || i.title.contains("Debug")
            || i.title.contains("CORS")));
    }
}
