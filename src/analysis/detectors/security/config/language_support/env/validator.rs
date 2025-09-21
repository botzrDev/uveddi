//! Environment variable validation and common issue detection
//!
//! This module provides validation for environment variables and
//! detects common security and configuration issues.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use super::parser::EnvParser;
use crate::analysis::AnalysisError;
use std::collections::HashSet;

/// Validator for environment variable security issues
pub struct EnvValidator {
    config: ConfigSecurityConfig,
}

impl EnvValidator {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for common environment variable issues
    pub fn check_common_issues(
        &self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for default/weak values
        if self.is_default_value(value) {
            issues.push(
                utils::create_config_issue(
                    ConfigSeverity::Medium,
                    "Default Environment Value".to_string(),
                    format!(
                        "Environment variable '{}' has a default/placeholder value",
                        key
                    ),
                    Some(line_number),
                    "Replace with actual production values".to_string(),
                    vec!["environment".to_string(), "default".to_string()],
                )
                .with_cwe(1188),
            );
        }

        // Check for empty values where they shouldn't be
        if self.is_sensitive_key(key) && value.is_empty() {
            issues.push(
                utils::create_config_issue(
                    ConfigSeverity::High,
                    "Empty Sensitive Value".to_string(),
                    format!("Sensitive environment variable '{}' is empty", key),
                    Some(line_number),
                    "Provide a proper value for sensitive configuration".to_string(),
                    vec!["environment".to_string(), "empty-value".to_string()],
                )
                .with_cwe(1188),
            );
        }

        // Check for insecure URLs
        if self.is_url_value(value) && value.starts_with("http://") {
            issues.push(
                utils::create_config_issue(
                    ConfigSeverity::Medium,
                    "Insecure HTTP URL".to_string(),
                    format!("Environment variable '{}' contains an HTTP URL", key),
                    Some(line_number),
                    "Use HTTPS URLs for secure communication".to_string(),
                    vec!["environment".to_string(), "http".to_string()],
                )
                .with_cwe(319),
            );
        }

        // Check for localhost in production-like keys
        if self.is_production_key(key) && self.contains_localhost(value) {
            issues.push(
                utils::create_config_issue(
                    ConfigSeverity::Medium,
                    "Localhost in Production Config".to_string(),
                    format!(
                        "Production environment variable '{}' contains localhost",
                        key
                    ),
                    Some(line_number),
                    "Use production hostnames instead of localhost".to_string(),
                    vec!["environment".to_string(), "localhost".to_string()],
                )
                .with_cwe(1188),
            );
        }

        // Check for debug/development flags
        if self.is_debug_key(key) && self.is_debug_enabled(value) {
            issues.push(
                utils::create_config_issue(
                    ConfigSeverity::Low,
                    "Debug Mode Enabled".to_string(),
                    format!("Debug mode enabled via '{}'", key),
                    Some(line_number),
                    "Disable debug mode in production environments".to_string(),
                    vec!["environment".to_string(), "debug".to_string()],
                )
                .with_cwe(489),
            );
        }

        issues
    }

    /// Check for structural issues in the entire file
    pub fn check_structural_issues(&self, content: &str) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for duplicate variables
        let duplicates = EnvParser::find_duplicates(content);
        for (var_name, line_numbers) in duplicates {
            issues.push(
                utils::create_config_issue(
                    ConfigSeverity::Medium,
                    "Duplicate Environment Variable".to_string(),
                    format!("Variable '{}' is defined multiple times", var_name),
                    Some(line_numbers[0]),
                    "Remove duplicate variable definitions".to_string(),
                    vec!["environment".to_string(), "duplicate".to_string()],
                )
                .with_cwe(1188),
            );
        }

        // Check for missing required variables
        issues.extend(self.check_missing_required_vars(content));

        // Check for inconsistent naming conventions
        issues.extend(self.check_naming_conventions(content));

        issues
    }

    fn is_default_value(&self, value: &str) -> bool {
        let default_values = [
            "default",
            "example",
            "changeme",
            "replace-me",
            "your-value-here",
            "todo",
            "fixme",
            "placeholder",
            "sample",
            "demo",
            "test",
            "localhost",
            "127.0.0.1",
            "http://localhost",
            "http://example.com",
        ];

        let value_lower = value.to_lowercase();
        default_values
            .iter()
            .any(|&default| value_lower.contains(default))
    }

    fn is_sensitive_key(&self, key: &str) -> bool {
        let sensitive_indicators = [
            "password",
            "secret",
            "key",
            "token",
            "auth",
            "credential",
            "private",
            "secure",
            "salt",
            "hash",
        ];

        let key_lower = key.to_lowercase();
        sensitive_indicators
            .iter()
            .any(|&indicator| key_lower.contains(indicator))
    }

    fn is_url_value(&self, value: &str) -> bool {
        value.starts_with("http://") || value.starts_with("https://")
    }

    fn is_production_key(&self, key: &str) -> bool {
        let prod_indicators = ["prod", "production", "live", "release"];
        let key_lower = key.to_lowercase();
        prod_indicators
            .iter()
            .any(|&indicator| key_lower.contains(indicator))
    }

    fn contains_localhost(&self, value: &str) -> bool {
        value.contains("localhost") || value.contains("127.0.0.1")
    }

    fn is_debug_key(&self, key: &str) -> bool {
        let debug_indicators = ["debug", "verbose", "trace", "dev", "development"];
        let key_lower = key.to_lowercase();
        debug_indicators
            .iter()
            .any(|&indicator| key_lower.contains(indicator))
    }

    fn is_debug_enabled(&self, value: &str) -> bool {
        let enable_values = ["true", "1", "yes", "on", "enabled"];
        let value_lower = value.to_lowercase();
        enable_values.contains(&value_lower.as_str())
    }

    fn check_missing_required_vars(&self, content: &str) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();
        let vars = EnvParser::extract_all_vars(content);
        let var_names: HashSet<String> = vars.iter().map(|(name, _, _)| name.clone()).collect();

        // Common required variables for web applications
        let common_required = [
            ("NODE_ENV", "Node.js environment"),
            ("DATABASE_URL", "Database connection"),
            ("SECRET_KEY", "Application secret key"),
        ];

        // Only check if we have some indication this is a web app config
        let has_web_indicators = var_names.iter().any(|name| {
            let name_lower = name.to_lowercase();
            name_lower.contains("port")
                || name_lower.contains("host")
                || name_lower.contains("url")
                || name_lower.contains("database")
        });

        if has_web_indicators {
            for (required_var, description) in &common_required {
                if !var_names.contains(*required_var) {
                    issues.push(
                        utils::create_config_issue(
                            ConfigSeverity::Low,
                            "Missing Required Variable".to_string(),
                            format!(
                                "Missing common variable '{}' for {}",
                                required_var, description
                            ),
                            None,
                            format!(
                                "Consider adding {} if needed for your application",
                                required_var
                            )
                            .to_string(),
                            vec!["environment".to_string(), "missing".to_string()],
                        )
                        .with_cwe(1188),
                    );
                }
            }
        }

        issues
    }

    fn check_naming_conventions(&self, content: &str) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();
        let vars = EnvParser::extract_all_vars(content);

        let mut naming_styles = std::collections::HashMap::new();
        for (name, _, line_num) in &vars {
            if name.contains('-') {
                *naming_styles.entry("kebab-case").or_insert(0) += 1;
            } else if name.chars().any(|c| c.is_lowercase()) {
                *naming_styles.entry("mixed-case").or_insert(0) += 1;
            } else {
                *naming_styles.entry("UPPER_SNAKE_CASE").or_insert(0) += 1;
            }
        }

        // If multiple naming styles are used, suggest consistency
        if naming_styles.len() > 1 {
            let total_vars = vars.len();
            if total_vars > 5 {
                // Only suggest for larger files
                issues.push(utils::create_config_issue(
                    ConfigSeverity::Info,
                    "Inconsistent Naming Convention".to_string(),
                    "Environment variables use inconsistent naming styles".to_string(),
                    None,
                    "Consider using consistent UPPER_SNAKE_CASE for all environment variables"
                        .to_string(),
                    vec!["environment".to_string(), "naming".to_string()],
                ));
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_value_detection() {
        let config = ConfigSecurityConfig::default();
        let validator = EnvValidator::new(&config);

        let issues = validator.check_common_issues("API_KEY", "changeme", 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Environment Value")));
    }

    #[test]
    fn test_empty_sensitive_value() {
        let config = ConfigSecurityConfig::default();
        let validator = EnvValidator::new(&config);

        let issues = validator.check_common_issues("SECRET_KEY", "", 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Empty Sensitive Value")));
    }

    #[test]
    fn test_insecure_url_detection() {
        let config = ConfigSecurityConfig::default();
        let validator = EnvValidator::new(&config);

        let issues = validator.check_common_issues("API_URL", "http://api.example.com", 1);
        assert!(issues.iter().any(|i| i.title.contains("Insecure HTTP URL")));
    }

    #[test]
    fn test_localhost_in_production() {
        let config = ConfigSecurityConfig::default();
        let validator = EnvValidator::new(&config);

        let issues = validator.check_common_issues("PROD_DATABASE_HOST", "localhost", 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Localhost in Production")));
    }

    #[test]
    fn test_debug_mode_detection() {
        let config = ConfigSecurityConfig::default();
        let validator = EnvValidator::new(&config);

        let issues = validator.check_common_issues("DEBUG", "true", 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Debug Mode Enabled")));
    }

    #[test]
    fn test_duplicate_detection() {
        let config = ConfigSecurityConfig::default();
        let validator = EnvValidator::new(&config);

        let content = r#"
KEY1=value1
KEY2=value2
KEY1=duplicate
"#;

        let issues = validator.check_structural_issues(content);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Duplicate Environment Variable")));
    }
}
