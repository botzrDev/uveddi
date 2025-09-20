//! Credential exposure detection in configuration files
//!
//! This module detects hardcoded passwords, API keys, database connection
//! strings, cloud service credentials, and certificate/private keys in
//! configuration files.

use crate::analysis::AnalysisError;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity};
use regex::Regex;
use std::collections::HashMap;

/// Analyzes configuration files for credential exposure
pub struct CredentialAnalyzer {
    patterns: Vec<CredentialPattern>,
    config: ConfigSecurityConfig,
}

/// Pattern for detecting different types of credentials
struct CredentialPattern {
    name: String,
    regex: Regex,
    severity: ConfigSeverity,
    confidence_base: f64,
    cwe_id: Option<u32>,
    owasp_category: Option<String>,
}

impl CredentialAnalyzer {
    /// Create a new credential analyzer
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let patterns = Self::build_credential_patterns()?;

        Ok(Self {
            patterns,
            config: config.clone(),
        })
    }

    /// Analyze content for credential exposure
    pub fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if let Some(captures) = pattern.regex.captures(line) {
                    if let Some(matched) = captures.get(0) {
                        let issue = self.create_credential_issue(
                            pattern,
                            matched.as_str(),
                            line,
                            line_num + 1,
                            matched.start(),
                        );
                        issues.push(issue);
                    }
                }
            }
        }

        // Additional context-aware analysis
        issues.extend(self.analyze_structured_credentials(content)?);

        Ok(issues)
    }

    fn build_credential_patterns() -> Result<Vec<CredentialPattern>, AnalysisError> {
        let patterns = vec![
            // API Keys
            CredentialPattern {
                name: "AWS Access Key".to_string(),
                regex: Regex::new(r"(?i)(aws_access_key_id|AKIA[0-9A-Z]{16})")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.95,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CredentialPattern {
                name: "AWS Secret Key".to_string(),
                regex: Regex::new(r"(?i)(aws_secret_access_key|[A-Za-z0-9/+=]{40})")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.9,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CredentialPattern {
                name: "Generic API Key".to_string(),
                regex: Regex::new(r#"(?i)(api[_-]?key|secret[_-]?key)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_-]{20,})['"]?"#)?,
                severity: ConfigSeverity::High,
                confidence_base: 0.85,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // Database Credentials
            CredentialPattern {
                name: "Database Password".to_string(),
                regex: Regex::new(r#"(?i)(password|passwd|pwd)[\s]*[:=][\s]*['"]?([^'\s\n]{6,})['"]?"#)?,
                severity: ConfigSeverity::High,
                confidence_base: 0.8,
                cwe_id: Some(798),
                owasp_category: Some("A07:2021 - Identification and Authentication Failures".to_string()),
            },
            CredentialPattern {
                name: "Database Connection String".to_string(),
                regex: Regex::new(r"(?i)(mongodb|mysql|postgresql|postgres)://[^/]*:[^@]*@")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.95,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // Private Keys
            CredentialPattern {
                name: "Private Key".to_string(),
                regex: Regex::new(r"-----BEGIN[A-Z\s]*PRIVATE KEY-----")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.99,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // JWT Tokens
            CredentialPattern {
                name: "JWT Token".to_string(),
                regex: Regex::new(r"eyJ[A-Za-z0-9_/+-]*\.[A-Za-z0-9_/+-]*\.[A-Za-z0-9_/+-]*")?,
                severity: ConfigSeverity::High,
                confidence_base: 0.9,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // OAuth Tokens
            CredentialPattern {
                name: "OAuth Token".to_string(),
                regex: Regex::new(r#"(?i)(access[_-]?token|bearer[_-]?token)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_.-]{32,})['"]?"#)?,
                severity: ConfigSeverity::High,
                confidence_base: 0.85,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
        ];

        Ok(patterns)
    }

    fn create_credential_issue(
        &self,
        pattern: &CredentialPattern,
        matched_text: &str,
        line: &str,
        line_number: usize,
        column_number: usize,
    ) -> ConfigIssue {
        let confidence = self.calculate_confidence(pattern, matched_text, line);

        ConfigIssue::new(
            pattern.severity,
            confidence,
            format!("Hardcoded Credential: {}", pattern.name),
            format!("Found {} in configuration file. This exposes sensitive credentials.", pattern.name),
        )
        .with_location(line_number, column_number)
        .with_snippet(self.create_safe_snippet(line, matched_text))
        .with_remediation(format!(
            "Remove hardcoded {} and use environment variables or secure secret management instead. \
            Consider using tools like HashiCorp Vault, AWS Secrets Manager, or Kubernetes secrets.",
            pattern.name.to_lowercase()
        ))
        .with_tag("credential-exposure")
        .with_tag("hardcoded-secret")
        .with_owasp(pattern.owasp_category.clone().unwrap_or_default())
        .with_cwe(pattern.cwe_id.unwrap_or(798))
    }

    fn calculate_confidence(&self, pattern: &CredentialPattern, matched_text: &str, line: &str) -> f64 {
        let mut confidence = pattern.confidence_base;

        // Reduce confidence for common test/example values
        let test_indicators = ["test", "example", "demo", "placeholder", "xxx", "***"];
        if test_indicators.iter().any(|&indicator| {
            matched_text.to_lowercase().contains(indicator) || line.to_lowercase().contains(indicator)
        }) {
            confidence *= 0.3;
        }

        // Reduce confidence for obviously fake values
        if matched_text.chars().all(|c| c == 'x' || c == '*' || c == '0') {
            confidence *= 0.1;
        }

        // Increase confidence for production-like contexts
        let prod_indicators = ["prod", "production", "live", "release"];
        if prod_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator)) {
            confidence = (confidence * 1.2).min(1.0);
        }

        confidence
    }

    fn create_safe_snippet(&self, line: &str, matched_text: &str) -> String {
        // Mask sensitive data in snippet
        let masked = "*".repeat(matched_text.len().min(8));
        line.replace(matched_text, &masked)
    }

    fn analyze_structured_credentials(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Look for structured credential patterns
        // e.g., username/password pairs, connection objects
        if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(content) {
            issues.extend(self.analyze_yaml_credentials(&yaml_value)?);
        }

        Ok(issues)
    }

    fn analyze_yaml_credentials(&self, value: &serde_yaml::Value) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
                        // Check for credential-related keys
                        if self.is_credential_key(key_str) {
                            if let Some(val_str) = val.as_str() {
                                if self.looks_like_credential(val_str) {
                                    issues.push(ConfigIssue::new(
                                        ConfigSeverity::High,
                                        0.8,
                                        "Structured Credential Found",
                                        format!("Found credential in structured configuration: {}", key_str),
                                    )
                                    .with_tag("structured-credential")
                                    .with_cwe(798));
                                }
                            }
                        }
                    }

                    // Recursively check nested structures
                    issues.extend(self.analyze_yaml_credentials(val)?);
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    issues.extend(self.analyze_yaml_credentials(item)?);
                }
            }
            _ => {}
        }

        Ok(issues)
    }

    fn is_credential_key(&self, key: &str) -> bool {
        let credential_keys = [
            "password", "passwd", "pwd", "secret", "key", "token", "auth",
            "credential", "private_key", "api_key", "access_key", "secret_key"
        ];

        let key_lower = key.to_lowercase();
        credential_keys.iter().any(|&cred_key| key_lower.contains(cred_key))
    }

    fn looks_like_credential(&self, value: &str) -> bool {
        // Skip obviously fake or empty values
        if value.is_empty() || value.len() < 6 {
            return false;
        }

        let fake_indicators = ["test", "example", "demo", "placeholder", "xxx", "***", "changeme"];
        if fake_indicators.iter().any(|&indicator| value.to_lowercase().contains(indicator)) {
            return false;
        }

        // Look for credential-like patterns
        let has_mixed_case = value.chars().any(|c| c.is_uppercase()) && value.chars().any(|c| c.is_lowercase());
        let has_numbers = value.chars().any(|c| c.is_numeric());
        let has_special = value.chars().any(|c| !c.is_alphanumeric());
        let reasonable_length = value.len() >= 8 && value.len() <= 256;

        (has_mixed_case || has_numbers || has_special) && reasonable_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = CredentialAnalyzer::new(&config).unwrap();

        let content = r#"
api_key: "sk-1234567890abcdef"
aws_access_key_id: "AKIAIOSFODNN7EXAMPLE"
"#;

        let issues = analyzer.analyze(content).unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|i| i.title.contains("API Key")));
        assert!(issues.iter().any(|i| i.title.contains("AWS Access Key")));
    }

    #[test]
    fn test_database_credential_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = CredentialAnalyzer::new(&config).unwrap();

        let content = r#"
database:
  password: "secretpassword123"
  connection: "postgresql://user:password@localhost/db"
"#;

        let issues = analyzer.analyze(content).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("Database")));
    }

    #[test]
    fn test_private_key_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = CredentialAnalyzer::new(&config).unwrap();

        let content = r#"
private_key: |
  -----BEGIN RSA PRIVATE KEY-----
  MIIEpAIBAAKCAQEA...
  -----END RSA PRIVATE KEY-----
"#;

        let issues = analyzer.analyze(content).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("Private Key")));
    }

    #[test]
    fn test_confidence_calculation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = CredentialAnalyzer::new(&config).unwrap();

        // Test password should have low confidence
        let test_content = r#"password: "test123""#;
        let issues = analyzer.analyze(test_content).unwrap();
        if !issues.is_empty() {
            assert!(issues[0].confidence < 0.5);
        }

        // Production password should have higher confidence
        let prod_content = r#"prod_password: "Xy9$kL2mN8pQ""#;
        let issues = analyzer.analyze(prod_content).unwrap();
        if !issues.is_empty() {
            assert!(issues[0].confidence > 0.7);
        }
    }
}