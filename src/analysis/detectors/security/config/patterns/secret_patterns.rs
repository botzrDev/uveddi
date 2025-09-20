//! Secret and credential pattern detection
//!
//! This module provides comprehensive pattern matching for detecting various
//! types of secrets and credentials in configuration files.

use crate::analysis::AnalysisError;
use crate::core::patterns::PatternMatcher;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity, PatternMatch};
use super::{ConfigPatternMatcher, utils};
use regex::Regex;
use std::collections::HashMap;

/// Pattern matcher for detecting secrets and credentials
pub struct SecretPatternMatcher {
    patterns: HashMap<String, SecretPattern>,
    config: ConfigSecurityConfig,
    suppression_patterns: Vec<String>,
}

/// Individual secret pattern definition
struct SecretPattern {
    name: String,
    regex: Regex,
    severity: ConfigSeverity,
    confidence: f64,
    description: String,
    remediation: String,
    cwe_id: Option<u32>,
    tags: Vec<String>,
}

impl ConfigPatternMatcher for SecretPatternMatcher {
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let patterns = Self::build_secret_patterns()?;
        let suppression_patterns = vec![
            "UVEDDI:IGNORE".to_string(),
            "SECURITY:OK".to_string(),
            "FALSE-POSITIVE".to_string(),
        ];

        Ok(Self {
            patterns,
            config: config.clone(),
            suppression_patterns,
        })
    }

    fn find_matches(&self, content: &str) -> Result<Vec<PatternMatch>, AnalysisError> {
        let mut matches = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;

            // Skip suppressed lines
            if utils::has_suppression_comment(line, &self.suppression_patterns) {
                continue;
            }

            for (pattern_id, pattern) in &self.patterns {
                for regex_match in pattern.regex.find_iter(line) {
                    let match_start = regex_match.start();
                    let match_end = regex_match.end();
                    let matched_text = regex_match.as_str().to_string();

                    let confidence = self.calculate_match_confidence(pattern, &matched_text, line);

                    if confidence >= self.config.confidence_threshold {
                        matches.push(PatternMatch {
                            pattern_id: pattern_id.clone(),
                            matched_text,
                            confidence,
                            line_number,
                            column_number: match_start + 1,
                            context: utils::extract_context(content, line_number, 2),
                        });
                    }
                }
            }
        }

        Ok(matches)
    }

    fn matches_to_issues(&self, matches: Vec<PatternMatch>) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for pattern_match in matches {
            if let Some(pattern) = self.patterns.get(&pattern_match.pattern_id) {
                let masked_snippet = utils::mask_sensitive(
                    &pattern_match.context,
                    pattern_match.column_number.saturating_sub(1),
                    pattern_match.matched_text.len(),
                );

                let mut issue = ConfigIssue::new(
                    pattern.severity,
                    pattern_match.confidence,
                    format!("Secret Pattern: {}", pattern.name),
                    pattern.description.clone(),
                )
                .with_location(pattern_match.line_number, pattern_match.column_number)
                .with_snippet(masked_snippet)
                .with_remediation(pattern.remediation.clone());

                for tag in &pattern.tags {
                    issue = issue.with_tag(tag.clone());
                }

                if let Some(cwe_id) = pattern.cwe_id {
                    issue = issue.with_cwe(cwe_id);
                }

                issues.push(issue);
            }
        }

        Ok(issues)
    }
}

impl PatternMatcher for SecretPatternMatcher {
    fn match_pattern(&self, content: &str) -> Result<bool, AnalysisError> {
        let matches = self.find_matches(content)?;
        Ok(!matches.is_empty())
    }

    fn get_pattern_type(&self) -> String {
        "secret_credential".to_string()
    }
}

impl SecretPatternMatcher {
    fn build_secret_patterns() -> Result<HashMap<String, SecretPattern>, AnalysisError> {
        let mut patterns = HashMap::new();

        // AWS Credentials
        patterns.insert("aws_access_key".to_string(), SecretPattern {
            name: "AWS Access Key ID".to_string(),
            regex: utils::compile_pattern(r"(?i)(aws_access_key_id|AKIA[0-9A-Z]{16})")?,
            severity: ConfigSeverity::Critical,
            confidence: 0.95,
            description: "AWS Access Key ID detected in configuration".to_string(),
            remediation: "Use AWS IAM roles or environment variables for AWS credentials".to_string(),
            cwe_id: Some(798),
            tags: vec!["aws".to_string(), "cloud".to_string(), "credential".to_string()],
        });

        patterns.insert("aws_secret_key".to_string(), SecretPattern {
            name: "AWS Secret Access Key".to_string(),
            regex: utils::compile_pattern(r"(?i)(aws_secret_access_key|[A-Za-z0-9/+=]{40})")?,
            severity: ConfigSeverity::Critical,
            confidence: 0.90,
            description: "AWS Secret Access Key detected in configuration".to_string(),
            remediation: "Use AWS IAM roles or AWS Secrets Manager for credentials".to_string(),
            cwe_id: Some(798),
            tags: vec!["aws".to_string(), "cloud".to_string(), "credential".to_string()],
        });

        // Generic API Keys
        patterns.insert("api_key".to_string(), SecretPattern {
            name: "API Key".to_string(),
            regex: utils::compile_pattern(r#"(?i)(api[_-]?key|secret[_-]?key)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_-]{20,})['"]?"#)?,
            severity: ConfigSeverity::High,
            confidence: 0.85,
            description: "API key detected in configuration".to_string(),
            remediation: "Store API keys in environment variables or secure secret management".to_string(),
            cwe_id: Some(798),
            tags: vec!["api-key".to_string(), "credential".to_string()],
        });

        // Database Passwords
        patterns.insert("database_password".to_string(), SecretPattern {
            name: "Database Password".to_string(),
            regex: utils::compile_pattern(r#"(?i)(password|passwd|pwd)[\s]*[:=][\s]*['"]?([^'\s\n]{6,})['"]?"#)?,
            severity: ConfigSeverity::High,
            confidence: 0.80,
            description: "Database password detected in configuration".to_string(),
            remediation: "Use environment variables or database credential management systems".to_string(),
            cwe_id: Some(798),
            tags: vec!["database".to_string(), "password".to_string(), "credential".to_string()],
        });

        // JWT Secrets
        patterns.insert("jwt_secret".to_string(), SecretPattern {
            name: "JWT Secret".to_string(),
            regex: utils::compile_pattern(r#"(?i)(jwt[_-]?secret|token[_-]?secret)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_+-=]{20,})['"]?"#)?,
            severity: ConfigSeverity::High,
            confidence: 0.88,
            description: "JWT signing secret detected in configuration".to_string(),
            remediation: "Use cryptographically strong, random JWT secrets stored securely".to_string(),
            cwe_id: Some(798),
            tags: vec!["jwt".to_string(), "token".to_string(), "secret".to_string()],
        });

        // Private Keys
        patterns.insert("private_key".to_string(), SecretPattern {
            name: "Private Key".to_string(),
            regex: utils::compile_pattern(r"-----BEGIN[A-Z\s]*PRIVATE KEY-----")?,
            severity: ConfigSeverity::Critical,
            confidence: 0.99,
            description: "Private key detected in configuration".to_string(),
            remediation: "Store private keys in secure key management systems, never in configuration files".to_string(),
            cwe_id: Some(798),
            tags: vec!["private-key".to_string(), "encryption".to_string(), "credential".to_string()],
        });

        // OAuth Tokens
        patterns.insert("oauth_token".to_string(), SecretPattern {
            name: "OAuth Token".to_string(),
            regex: utils::compile_pattern(r#"(?i)(access[_-]?token|bearer[_-]?token|oauth[_-]?token)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_.-]{32,})['"]?"#)?,
            severity: ConfigSeverity::High,
            confidence: 0.85,
            description: "OAuth access token detected in configuration".to_string(),
            remediation: "Use OAuth refresh token flow and store tokens securely".to_string(),
            cwe_id: Some(798),
            tags: vec!["oauth".to_string(), "token".to_string(), "credential".to_string()],
        });

        // Database Connection Strings
        patterns.insert("db_connection_string".to_string(), SecretPattern {
            name: "Database Connection String".to_string(),
            regex: utils::compile_pattern(r"(?i)(mongodb|mysql|postgresql|postgres|mssql)://[^/]*:[^@]*@")?,
            severity: ConfigSeverity::Critical,
            confidence: 0.95,
            description: "Database connection string with embedded credentials detected".to_string(),
            remediation: "Use connection strings without embedded credentials and store credentials separately".to_string(),
            cwe_id: Some(798),
            tags: vec!["database".to_string(), "connection-string".to_string(), "credential".to_string()],
        });

        // Slack Tokens
        patterns.insert("slack_token".to_string(), SecretPattern {
            name: "Slack Token".to_string(),
            regex: utils::compile_pattern(r"xox[baprs]-[0-9]{12}-[0-9]{12}-[a-zA-Z0-9]{24}")?,
            severity: ConfigSeverity::High,
            confidence: 0.95,
            description: "Slack API token detected in configuration".to_string(),
            remediation: "Store Slack tokens in environment variables or secure token management".to_string(),
            cwe_id: Some(798),
            tags: vec!["slack".to_string(), "token".to_string(), "api".to_string()],
        });

        // GitHub Tokens
        patterns.insert("github_token".to_string(), SecretPattern {
            name: "GitHub Token".to_string(),
            regex: utils::compile_pattern(r"(?i)(github[_-]?token|gh[ps]_[a-zA-Z0-9]{36})")?,
            severity: ConfigSeverity::High,
            confidence: 0.90,
            description: "GitHub access token detected in configuration".to_string(),
            remediation: "Use GitHub Apps or store tokens in secure secret management".to_string(),
            cwe_id: Some(798),
            tags: vec!["github".to_string(), "token".to_string(), "git".to_string()],
        });

        Ok(patterns)
    }

    fn calculate_match_confidence(&self, pattern: &SecretPattern, matched_text: &str, line: &str) -> f64 {
        let mut confidence = pattern.confidence;

        // Reduce confidence for common test/example values
        let test_indicators = ["test", "example", "demo", "placeholder", "xxx", "***", "changeme"];
        if test_indicators.iter().any(|&indicator| {
            matched_text.to_lowercase().contains(indicator) || line.to_lowercase().contains(indicator)
        }) {
            confidence *= 0.3;
        }

        // Reduce confidence for obviously fake values
        if matched_text.chars().all(|c| c == 'x' || c == '*' || c == '0') || matched_text.len() < 6 {
            confidence *= 0.1;
        }

        // Increase confidence for production-like contexts
        let prod_indicators = ["prod", "production", "live", "release"];
        if prod_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator)) {
            confidence = (confidence * 1.2).min(1.0);
        }

        // Adjust confidence based on entropy
        let entropy = self.calculate_entropy(matched_text);
        if entropy > 3.5 {
            confidence = (confidence * 1.1).min(1.0);
        } else if entropy < 2.0 {
            confidence *= 0.7;
        }

        confidence
    }

    fn calculate_entropy(&self, text: &str) -> f64 {
        let mut char_counts = HashMap::new();
        for ch in text.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let text_len = text.len() as f64;
        char_counts.values()
            .map(|&count| {
                let p = count as f64 / text_len;
                -p * p.log2()
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_key_detection() {
        let config = ConfigSecurityConfig::default();
        let matcher = SecretPatternMatcher::new(&config).unwrap();

        let content = r#"
aws_access_key_id: AKIAIOSFODNN7EXAMPLE
aws_secret_access_key: wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
"#;

        let matches = matcher.find_matches(content).unwrap();
        assert!(!matches.is_empty());

        let issues = matcher.matches_to_issues(matches).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("AWS Access Key")));
        assert!(issues.iter().any(|i| i.title.contains("AWS Secret")));
    }

    #[test]
    fn test_private_key_detection() {
        let config = ConfigSecurityConfig::default();
        let matcher = SecretPatternMatcher::new(&config).unwrap();

        let content = r#"
private_key: |
  -----BEGIN RSA PRIVATE KEY-----
  MIIEpAIBAAKCAQEA...
  -----END RSA PRIVATE KEY-----
"#;

        let matches = matcher.find_matches(content).unwrap();
        assert!(!matches.is_empty());

        let issues = matcher.matches_to_issues(matches).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Private Key")));
    }

    #[test]
    fn test_suppression_comments() {
        let config = ConfigSecurityConfig::default();
        let matcher = SecretPatternMatcher::new(&config).unwrap();

        let content = r#"
password: secret123 # UVEDDI:IGNORE - this is a test password
api_key: real_secret_key
"#;

        let matches = matcher.find_matches(content).unwrap();
        // Should only find the real API key, not the suppressed password
        assert_eq!(matches.len(), 1);
        assert!(matches[0].pattern_id.contains("api_key"));
    }

    #[test]
    fn test_confidence_calculation() {
        let config = ConfigSecurityConfig::default();
        let matcher = SecretPatternMatcher::new(&config).unwrap();

        // Test password should have low confidence
        let test_content = r#"password: "test123""#;
        let matches = matcher.find_matches(test_content).unwrap();
        if !matches.is_empty() {
            assert!(matches[0].confidence < 0.5);
        }

        // Real-looking password should have higher confidence
        let real_content = r#"prod_password: "Xy9$kL2mN8pQ""#;
        let matches = matcher.find_matches(real_content).unwrap();
        if !matches.is_empty() {
            assert!(matches[0].confidence > 0.7);
        }
    }

    #[test]
    fn test_entropy_calculation() {
        let config = ConfigSecurityConfig::default();
        let matcher = SecretPatternMatcher::new(&config).unwrap();

        // Low entropy (repeated characters)
        let low_entropy = matcher.calculate_entropy("aaaaaaa");
        assert!(low_entropy < 1.0);

        // High entropy (random-looking)
        let high_entropy = matcher.calculate_entropy("Xy9$kL2mN8pQ");
        assert!(high_entropy > 3.0);
    }
}