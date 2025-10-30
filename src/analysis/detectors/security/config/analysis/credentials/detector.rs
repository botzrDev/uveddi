//! Main credential detection logic

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::{scanner, validator, CredentialPattern};
use crate::analysis::AnalysisError;
use regex::Regex;

/// Analyzes configuration files for credential exposure
pub struct CredentialAnalyzer {
    patterns: Vec<CredentialPattern>,
    config: ConfigSecurityConfig,
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
        issues.extend(scanner::analyze_structured_credentials(content)?);

        Ok(issues)
    }

    fn build_credential_patterns() -> Result<Vec<CredentialPattern>, AnalysisError> {
        let compile = |pattern: &str| -> Result<Regex, AnalysisError> {
            Regex::new(pattern).map_err(|e| {
                AnalysisError::DetectionError(format!(
                    "Invalid credential detection regex '{}': {}",
                    pattern, e
                ))
            })
        };

        let patterns = vec![
            // API Keys
            CredentialPattern {
                name: "AWS Access Key".to_string(),
                regex: compile(r"(?i)(aws_access_key_id|AKIA[0-9A-Z]{16})")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.95,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CredentialPattern {
                name: "AWS Secret Key".to_string(),
                regex: compile(r"(?i)(aws_secret_access_key|[A-Za-z0-9/+=]{40})")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.9,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            CredentialPattern {
                name: "Generic API Key".to_string(),
                regex: compile(
                    r#"(?i)(api[_-]?key|secret[_-]?key)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_-]{20,})['"]?"#,
                )?,
                severity: ConfigSeverity::High,
                confidence_base: 0.85,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // Database Credentials
            CredentialPattern {
                name: "Database Password".to_string(),
                regex: compile(
                    r#"(?i)(password|passwd|pwd)[\s]*[:=][\s]*['"]?([^'\s\n]{6,})['"]?"#,
                )?,
                severity: ConfigSeverity::High,
                confidence_base: 0.8,
                cwe_id: Some(798),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            CredentialPattern {
                name: "Database Connection String".to_string(),
                regex: compile(r"(?i)(mongodb|mysql|postgresql|postgres)://[^/]*:[^@]*@")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.95,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // Private Keys
            CredentialPattern {
                name: "Private Key".to_string(),
                regex: compile(r"-----BEGIN[A-Z\s]*PRIVATE KEY-----")?,
                severity: ConfigSeverity::Critical,
                confidence_base: 0.99,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // JWT Tokens
            CredentialPattern {
                name: "JWT Token".to_string(),
                regex: compile(r"eyJ[A-Za-z0-9_/+-]*\.[A-Za-z0-9_/+-]*\.[A-Za-z0-9_/+-]*")?,
                severity: ConfigSeverity::High,
                confidence_base: 0.9,
                cwe_id: Some(798),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },
            // OAuth Tokens
            CredentialPattern {
                name: "OAuth Token".to_string(),
                regex: Regex::new(
                    r#"(?i)(access[_-]?token|bearer[_-]?token)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_.-]{32,})['"]?"#,
                )?,
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
        let confidence = validator::calculate_confidence(pattern, matched_text, line);

        ConfigIssue::new(
            pattern.severity,
            confidence,
            format!("Hardcoded Credential: {}", pattern.name),
            format!("Found {} in configuration file. This exposes sensitive credentials.", pattern.name),
        )
        .with_location(line_number, column_number)
        .with_snippet(Self::create_safe_snippet(line, matched_text))
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

    fn create_safe_snippet(line: &str, matched_text: &str) -> String {
        // Mask sensitive data in snippet
        let masked = "*".repeat(matched_text.len().min(8));
        line.replace(matched_text, &masked)
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
}
