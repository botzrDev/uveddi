//! Debug and logging configuration security checks
//!
//! This module provides specialized checks for debug mode and logging
//! configuration security issues.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use serde_yaml::Value as YamlValue;

/// Checker for debug-related misconfigurations
pub struct DebugChecker {
    config: ConfigSecurityConfig,
}

impl DebugChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for debug mode enabled in production
    pub fn check_debug_in_production(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            let is_production = map.iter().any(|(k, v)| {
                k.as_str()
                    .map_or(false, |key| key.to_lowercase().contains("env"))
                    && v.as_str()
                        .map_or(false, |val| val.to_lowercase().contains("prod"))
            });

            let debug_enabled = map.iter().any(|(k, v)| {
                k.as_str()
                    .map_or(false, |key| key.to_lowercase().contains("debug"))
                    && v.as_bool().unwrap_or(false)
            });

            if is_production && debug_enabled {
                issues.push(
                    ConfigIssue::new(
                        ConfigSeverity::High,
                        0.9,
                        "Debug Mode in Production",
                        "Debug mode is enabled in production environment",
                    )
                    .with_tag("production-debug")
                    .with_remediation("Disable debug mode in production environments")
                    .with_cwe(489),
                );
            }
        }

        Ok(issues)
    }

    /// Check for verbose logging that might expose sensitive data
    pub fn check_verbose_logging(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let key_lower = key_str.to_lowercase();
                    if key_lower.contains("log") && key_lower.contains("level") {
                        if let Some(level) = val.as_str() {
                            if level.to_lowercase() == "debug" || level.to_lowercase() == "trace" {
                                issues.push(
                                    ConfigIssue::new(
                                        ConfigSeverity::Medium,
                                        0.7,
                                        "Verbose Logging Enabled",
                                        "Debug or trace logging may expose sensitive information",
                                    )
                                    .with_tag("logging-security")
                                    .with_remediation(
                                        "Use info or warn level logging in production",
                                    )
                                    .with_cwe(532),
                                );
                            }
                        }
                    }
                }
            }
        }

        issues
    }

    /// Check for development mode indicators
    pub fn check_development_mode(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let key_lower = key_str.to_lowercase();
                    if key_lower.contains("dev") || key_lower.contains("development") {
                        if val.as_bool() == Some(true) {
                            issues.push(
                                ConfigIssue::new(
                                    ConfigSeverity::Medium,
                                    0.6,
                                    "Development Mode Enabled",
                                    "Development mode features are enabled",
                                )
                                .with_tag("development-mode")
                                .with_remediation("Disable development mode in production"),
                            );
                        }
                    }
                }
            }
        }

        issues
    }

    /// Check for error reporting configurations
    pub fn check_error_reporting(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let key_lower = key_str.to_lowercase();
                    if key_lower.contains("error") && key_lower.contains("report") {
                        if val.as_bool() == Some(true) {
                            issues.push(
                                ConfigIssue::new(
                                    ConfigSeverity::Low,
                                    0.5,
                                    "Error Reporting Enabled",
                                    "Detailed error reporting may expose sensitive information",
                                )
                                .with_tag("error-reporting")
                                .with_remediation("Disable detailed error reporting in production")
                                .with_cwe(209),
                            );
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
    fn test_debug_in_production() {
        let config = ConfigSecurityConfig::default();
        let checker = DebugChecker::new(&config);

        let yaml_content = r#"
environment: production
debug: true
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_debug_in_production(&value).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Debug Mode in Production")));
    }

    #[test]
    fn test_verbose_logging() {
        let config = ConfigSecurityConfig::default();
        let checker = DebugChecker::new(&config);

        let yaml_content = r#"
log_level: debug
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_verbose_logging(&value);
        assert!(issues.iter().any(|i| i.title.contains("Verbose Logging")));
    }

    #[test]
    fn test_development_mode() {
        let config = ConfigSecurityConfig::default();
        let checker = DebugChecker::new(&config);

        let yaml_content = r#"
development_mode: true
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_development_mode(&value);
        assert!(issues.iter().any(|i| i.title.contains("Development Mode")));
    }
}
