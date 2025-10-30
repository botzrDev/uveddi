//! Cryptographic default detection
//!
//! This module detects default encryption keys, salts, JWT secrets,
//! and other cryptographic defaults in configuration files.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use std::collections::HashSet;

/// Pattern for detecting cryptographic default values
pub struct CryptoDefaultPattern {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub values: HashSet<String>,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Checker for cryptographic default security issues
pub struct CryptoDefaultChecker {
    patterns: Vec<CryptoDefaultPattern>,
}

impl CryptoDefaultChecker {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let patterns = Self::build_crypto_patterns();
        Ok(Self { patterns })
    }

    pub fn check_line(&self, line: &str, line_number: usize) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Look for crypto-related keys
        if Self::contains_crypto_key(line) {
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

    fn build_crypto_patterns() -> Vec<CryptoDefaultPattern> {
        vec![
            CryptoDefaultPattern {
                name: "Default Encryption Key".to_string(),
                description: "Default or weak encryption key detected".to_string(),
                severity: ConfigSeverity::Critical,
                values: [
                    "secret",
                    "secretkey",
                    "mySecretKey",
                    "defaultkey",
                    "changeme",
                    "key123",
                    "your-secret-key",
                    "your_secret_key",
                    "replace-me",
                    "example-key",
                    "test-key",
                    "development-key",
                    "demo-key",
                    "sample-key",
                    "encryption-key",
                    "default-encryption-key",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Generate strong, random encryption keys unique to each environment."
                    .to_string(),
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CryptoDefaultPattern {
                name: "Default Salt Value".to_string(),
                description: "Default or weak salt value detected".to_string(),
                severity: ConfigSeverity::High,
                values: [
                    "salt",
                    "mysalt",
                    "defaultsalt",
                    "pepper",
                    "seasoning",
                    "somesalt",
                    "your-salt",
                    "changethis",
                    "example-salt",
                    "test-salt",
                    "demo-salt",
                    "salt123",
                    "salted",
                    "saltvalue",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Use cryptographically random salt values unique to each password."
                    .to_string(),
                cwe_id: Some(916),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CryptoDefaultPattern {
                name: "Default JWT Secret".to_string(),
                description: "Default or weak JWT signing secret detected".to_string(),
                severity: ConfigSeverity::Critical,
                values: [
                    "jwt-secret",
                    "jwtSecret",
                    "your-jwt-secret",
                    "secret-key",
                    "supersecret",
                    "jwtsecretkey",
                    "my-jwt-secret",
                    "change-me",
                    "jwt_secret_key",
                    "jwt-key",
                    "token-secret",
                    "signing-key",
                    "jwt_secret",
                    "jsonwebtoken",
                    "jwt-signing-key",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Use a cryptographically strong, random JWT signing secret."
                    .to_string(),
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CryptoDefaultPattern {
                name: "Default Token/Secret".to_string(),
                description: "Default or placeholder token/secret detected".to_string(),
                severity: ConfigSeverity::High,
                values: [
                    "your-token-here",
                    "your_token_here",
                    "insert-token-here",
                    "replace-with-token",
                    "your-api-key",
                    "your_api_key",
                    "token123",
                    "secret123",
                    "api-key-here",
                    "put-your-key-here",
                    "example-token",
                    "test-token",
                    "demo-token",
                    "sample-secret",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation:
                    "Replace placeholder values with actual tokens/secrets from secure storage."
                        .to_string(),
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CryptoDefaultPattern {
                name: "Weak Cryptographic Value".to_string(),
                description: "Weak or predictable cryptographic value detected".to_string(),
                severity: ConfigSeverity::Medium,
                values: [
                    "abc123",
                    "123abc",
                    "test123",
                    "key1",
                    "key2",
                    "key3",
                    "simple",
                    "basic",
                    "easy",
                    "weak",
                    "default",
                    "standard",
                    "common",
                    "public",
                    "open",
                    "shared",
                    "temp",
                    "temporary",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation:
                    "Use cryptographically strong, random values for all cryptographic purposes."
                        .to_string(),
                cwe_id: Some(330),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
        ]
    }

    fn contains_crypto_key(line: &str) -> bool {
        let line_lower = line.to_lowercase();
        let crypto_keys = [
            "key",
            "secret",
            "token",
            "salt",
            "pepper",
            "encrypt",
            "decrypt",
            "crypto",
            "cipher",
            "hash",
            "jwt",
            "signature",
            "signing",
            "hmac",
            "aes",
            "rsa",
            "private",
            "public",
        ];

        crypto_keys.iter().any(|&key| line_lower.contains(key))
    }

    fn line_contains_value(line: &str, value: &str) -> bool {
        let line_lower = line.to_lowercase();
        let value_lower = value.to_lowercase();

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
    fn test_default_encryption_key_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = CryptoDefaultChecker::new(&config).unwrap();

        let line = "encryption_key: secretkey";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Encryption Key")));
    }

    #[test]
    fn test_default_salt_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = CryptoDefaultChecker::new(&config).unwrap();

        let line = "password_salt: \"salt\"";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Salt Value")));
    }

    #[test]
    fn test_jwt_secret_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = CryptoDefaultChecker::new(&config).unwrap();

        let line = "jwt_secret: jwt-secret";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default JWT Secret")));
    }

    #[test]
    fn test_placeholder_token_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = CryptoDefaultChecker::new(&config).unwrap();

        let line = "api_token: your-token-here";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Token/Secret")));
    }

    #[test]
    fn test_non_crypto_line_ignored() {
        let config = ConfigSecurityConfig::default();
        let checker = CryptoDefaultChecker::new(&config).unwrap();

        let line = "timeout: 30";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_strong_crypto_value_no_issues() {
        let config = ConfigSecurityConfig::default();
        let checker = CryptoDefaultChecker::new(&config).unwrap();

        let line = "encryption_key: 4f8a2b1c9d3e7f6a8b5c2d9e1f4a7b3c";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }
}
