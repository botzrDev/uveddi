//! Default password detection
//!
//! This module detects common default passwords and weak password patterns
//! in configuration files.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use std::collections::HashSet;

/// Pattern for detecting default password values
pub struct PasswordDefaultPattern {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub values: HashSet<String>,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Checker for default password security issues
pub struct PasswordDefaultChecker {
    patterns: Vec<PasswordDefaultPattern>,
}

impl PasswordDefaultChecker {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let patterns = Self::build_password_patterns();
        Ok(Self { patterns })
    }

    pub fn check_line(&self, line: &str, line_number: usize) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Look for password-like keys
        if Self::contains_password_key(line) {
            for pattern in &self.patterns {
                for default_value in &pattern.values {
                    if Self::line_contains_value(line, default_value) {
                        let issue = ConfigIssue::new(
                            pattern.severity,
                            0.95, // High confidence for exact matches
                            pattern.name.clone(),
                            format!("{}: {}", pattern.description, default_value),
                        )
                        .with_location(line_number, 1)
                        .with_snippet(line.to_string())
                        .with_remediation(pattern.remediation.clone());

                        let mut final_issue = if let Some(cwe_id) = pattern.cwe_id {
                            issue.with_cwe(cwe_id)
                        } else {
                            issue
                        };

                        if let Some(ref owasp_cat) = pattern.owasp_category {
                            final_issue = final_issue.with_tag(owasp_cat.clone());
                        }

                        issues.push(final_issue);
                        break; // Only report one issue per line
                    }
                }
            }
        }

        issues
    }

    fn build_password_patterns() -> Vec<PasswordDefaultPattern> {
        vec![
            PasswordDefaultPattern {
                name: "Default Password".to_string(),
                description: "Common default password detected".to_string(),
                severity: ConfigSeverity::Critical,
                values: [
                    "admin",
                    "password",
                    "123456",
                    "qwerty",
                    "letmein",
                    "welcome",
                    "monkey",
                    "dragon",
                    "default",
                    "changeme",
                    "secret",
                    "root",
                    "guest",
                    "user",
                    "test",
                    "demo",
                    "pass",
                    "pwd",
                    "login",
                    "temp",
                    "temporary",
                    "example",
                    "sample",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Replace default passwords with strong, unique passwords.".to_string(),
                cwe_id: Some(521),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            PasswordDefaultPattern {
                name: "Weak Default Password".to_string(),
                description: "Weak or predictable default password detected".to_string(),
                severity: ConfigSeverity::High,
                values: [
                    "password123",
                    "admin123",
                    "test123",
                    "user123",
                    "123123",
                    "111111",
                    "000000",
                    "qwerty123",
                    "abc123",
                    "password1",
                    "admin1",
                    "guest1",
                    "root123",
                    "12345",
                    "54321",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation:
                    "Use strong passwords with mixed case, numbers, and special characters."
                        .to_string(),
                cwe_id: Some(521),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            PasswordDefaultPattern {
                name: "Empty Password".to_string(),
                description: "Empty or blank password detected".to_string(),
                severity: ConfigSeverity::Critical,
                values: ["", " ", "   ", "\t", "\n"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                remediation: "Set a strong password - empty passwords are never acceptable."
                    .to_string(),
                cwe_id: Some(521),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            PasswordDefaultPattern {
                name: "Placeholder Password".to_string(),
                description: "Placeholder password text detected".to_string(),
                severity: ConfigSeverity::High,
                values: [
                    "your-password",
                    "your_password",
                    "enter-password",
                    "replace-me",
                    "change-this",
                    "password-here",
                    "insert-password",
                    "put-password-here",
                    "your-password-here",
                    "set-password",
                    "configure-password",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Replace placeholder text with actual strong passwords.".to_string(),
                cwe_id: Some(521),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
        ]
    }

    fn contains_password_key(line: &str) -> bool {
        let line_lower = line.to_lowercase();
        let password_keys = [
            "password",
            "passwd",
            "pwd",
            "pass",
            "secret",
            "key",
            "token",
            "auth",
            "credential",
            "cred",
            "login",
        ];

        password_keys.iter().any(|&key| line_lower.contains(key))
    }

    fn line_contains_value(line: &str, value: &str) -> bool {
        let line_lower = line.to_lowercase();
        let value_lower = value.to_lowercase();

        // For empty values, only check for empty string literals
        if value.is_empty() || value.trim().is_empty() {
            return line_lower.contains(": \"\"")
                || line_lower.contains(": ''")
                || line_lower.contains(":\"\"")
                || line_lower.contains(":''");
        }

        // Check for exact matches with common delimiters
        let patterns = [
            format!("\"{}\"", value_lower), // "value"
            format!("'{}'", value_lower),   // 'value'
            format!(": {}", value_lower),   // : value
            format!("= {}", value_lower),   // = value
            format!(":{}", value_lower),    // :value
            format!("={}", value_lower),    // =value
        ];

        patterns.iter().any(|pattern| line_lower.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_password_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PasswordDefaultChecker::new(&config).unwrap();

        let line = "password: admin";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Default Password")));
    }

    #[test]
    fn test_weak_password_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PasswordDefaultChecker::new(&config).unwrap();

        let line = "user_password: \"123456\"";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Default Password")));
    }

    #[test]
    fn test_empty_password_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PasswordDefaultChecker::new(&config).unwrap();

        let line = "admin_password: \"\"";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Empty Password")));
    }

    #[test]
    fn test_placeholder_password_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PasswordDefaultChecker::new(&config).unwrap();

        let line = "secret: your-password-here";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Placeholder Password")));
    }

    #[test]
    fn test_non_password_line_ignored() {
        let config = ConfigSecurityConfig::default();
        let checker = PasswordDefaultChecker::new(&config).unwrap();

        let line = "timeout: 30";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_strong_password_no_issues() {
        let config = ConfigSecurityConfig::default();
        let checker = PasswordDefaultChecker::new(&config).unwrap();

        let line = "password: Xy9$kL2mN8pQ!";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }
}
