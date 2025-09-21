//! TOML credential detection
//!
//! This module detects hardcoded credentials and secrets in TOML files.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use crate::analysis::AnalysisError;
use toml::Value as TomlValue;

/// Credential checker for TOML files
pub struct TomlCredentialChecker {
    config: ConfigSecurityConfig,
}

impl TomlCredentialChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for hardcoded credentials in TOML
    pub fn check_credentials(&self, value: &TomlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
        self.check_credentials_recursive(value, "", &mut issues);
        Ok(issues)
    }

    fn check_credentials_recursive(
        &self,
        value: &TomlValue,
        path: &str,
        issues: &mut Vec<ConfigIssue>,
    ) {
        match value {
            TomlValue::Table(table) => {
                for (key, val) in table {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };

                    if utils::is_sensitive_key(key) {
                        if let TomlValue::String(val_str) = val {
                            if self.looks_like_credential(val_str) {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::High,
                                    "Hardcoded Credential in TOML",
                                    format!("Found credential at path: {}", new_path),
                                    None,
                                    "Use environment variables or secure configuration management",
                                    vec!["credential".to_string(), "toml".to_string()],
                                ).with_cwe(798));
                            }
                        }
                    }

                    self.check_credentials_recursive(val, &new_path, issues);
                }
            }
            TomlValue::Array(array) => {
                for (index, item) in array.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    self.check_credentials_recursive(item, &new_path, issues);
                }
            }
            _ => {}
        }
    }

    fn looks_like_credential(&self, value: &str) -> bool {
        // Skip obviously fake values
        if value.is_empty() || value.len() < 6 {
            return false;
        }

        let fake_indicators = ["test", "example", "demo", "placeholder", "xxx", "***"];
        if fake_indicators
            .iter()
            .any(|&indicator| value.to_lowercase().contains(indicator))
        {
            return false;
        }

        // Look for credential-like characteristics
        let has_special_chars = value.chars().any(|c| !c.is_alphanumeric());
        let has_mixed_case =
            value.chars().any(|c| c.is_uppercase()) && value.chars().any(|c| c.is_lowercase());
        let reasonable_length = value.len() >= 8 && value.len() <= 512;

        (has_special_chars || has_mixed_case) && reasonable_length
    }

    /// Check for specific database configuration credentials
    pub fn check_database_credentials(&self, value: &TomlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let TomlValue::Table(table) = value {
            // Check for database sections
            for (section_name, section_value) in table {
                if section_name.to_lowercase().contains("database")
                    || section_name.to_lowercase().contains("db")
                {
                    if let TomlValue::Table(db_table) = section_value {
                        self.check_db_table(db_table, section_name, &mut issues);
                    }
                }
            }
        }

        issues
    }

    fn check_db_table(
        &self,
        db_table: &toml::value::Table,
        section_name: &str,
        issues: &mut Vec<ConfigIssue>,
    ) {
        for (key, value) in db_table {
            let key_lower = key.to_lowercase();

            // Check for hardcoded database URLs with credentials
            if key_lower.contains("url") || key_lower.contains("connection") {
                if let TomlValue::String(url_str) = value {
                    if self.contains_credentials_in_url(url_str) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::Critical,
                                "Database URL with Credentials",
                                format!(
                                    "Database URL contains embedded credentials in {}.{}",
                                    section_name, key
                                ),
                                None,
                                "Use connection URLs without embedded credentials",
                                vec![
                                    "database".to_string(),
                                    "credential".to_string(),
                                    "toml".to_string(),
                                ],
                            )
                            .with_cwe(798),
                        );
                    }
                }
            }

            // Check for password fields
            if key_lower.contains("password") || key_lower.contains("pwd") {
                if let TomlValue::String(pwd_str) = value {
                    if !pwd_str.is_empty() && !self.is_placeholder_value(pwd_str) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::High,
                                "Hardcoded Database Password",
                                format!("Database password found in {}.{}", section_name, key),
                                None,
                                "Use environment variables for database passwords",
                                vec![
                                    "database".to_string(),
                                    "password".to_string(),
                                    "toml".to_string(),
                                ],
                            )
                            .with_cwe(798),
                        );
                    }
                }
            }

            // Check for API keys in database config
            if key_lower.contains("api") && key_lower.contains("key") {
                if let TomlValue::String(api_key) = value {
                    if self.looks_like_credential(api_key) {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::High,
                                "Hardcoded API Key in Database Config",
                                format!(
                                    "API key found in database configuration {}.{}",
                                    section_name, key
                                ),
                                None,
                                "Use environment variables for API keys",
                                vec![
                                    "database".to_string(),
                                    "api-key".to_string(),
                                    "toml".to_string(),
                                ],
                            )
                            .with_cwe(798),
                        );
                    }
                }
            }
        }
    }

    fn contains_credentials_in_url(&self, url: &str) -> bool {
        // Look for patterns like: protocol://user:password@host
        let credential_patterns = [
            "://.*:.*@", // Basic auth pattern
            "postgres://.*:.*@",
            "mysql://.*:.*@",
            "mongodb://.*:.*@",
        ];

        credential_patterns.iter().any(|pattern| {
            regex::Regex::new(pattern)
                .map(|re| re.is_match(url))
                .unwrap_or(false)
        })
    }

    fn is_placeholder_value(&self, value: &str) -> bool {
        let placeholders = [
            "changeme",
            "replace",
            "your-password",
            "password-here",
            "todo",
            "fixme",
            "example",
            "placeholder",
        ];

        let value_lower = value.to_lowercase();
        placeholders
            .iter()
            .any(|&placeholder| value_lower.contains(placeholder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlCredentialChecker::new(&config);

        let toml_content = r#"
password = "SuperSecret123!"
api_key = "sk-1234567890abcdef"
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_credentials(&parsed).unwrap();
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Hardcoded Credential")));
    }

    #[test]
    fn test_database_url_with_credentials() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlCredentialChecker::new(&config);

        let toml_content = r#"
[database]
url = "postgres://user:password@localhost/db"
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_database_credentials(&parsed);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Database URL with Credentials")));
    }

    #[test]
    fn test_placeholder_values_ignored() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlCredentialChecker::new(&config);

        let toml_content = r#"
password = "changeme"
api_key = "your-api-key-here"
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_credentials(&parsed).unwrap();
        // Should not detect placeholder values as real credentials
        assert!(issues.is_empty());
    }

    #[test]
    fn test_looks_like_credential() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlCredentialChecker::new(&config);

        assert!(checker.looks_like_credential("Xy9$kL2mN8pQ"));
        assert!(!checker.looks_like_credential("test"));
        assert!(!checker.looks_like_credential("example"));
        assert!(!checker.looks_like_credential(""));
    }
}
