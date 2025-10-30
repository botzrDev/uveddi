//! Secret and credential pattern detection
//!
//! This module provides comprehensive pattern matching for detecting various
//! types of secrets and credentials in configuration files.

pub mod api_keys;
pub mod certificates;
pub mod credentials;

// Re-export main types
pub use api_keys::build_api_key_patterns;
pub use certificates::build_certificate_patterns;
pub use credentials::build_credential_patterns;

use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity, PatternMatch};
use super::{utils, ConfigPatternMatcher, PatternMatcher};
use crate::analysis::AnalysisError;
use regex::Regex;
use std::collections::HashMap;

/// Individual secret pattern definition
pub struct SecretPattern {
    pub name: String,
    pub regex: Regex,
    pub severity: ConfigSeverity,
    pub confidence: f64,
    pub description: String,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub tags: Vec<String>,
}

/// Pattern matcher for detecting secrets and credentials
pub struct SecretPatternMatcher {
    patterns: HashMap<String, SecretPattern>,
    config: ConfigSecurityConfig,
    suppression_patterns: Vec<String>,
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

    fn matches_to_issues(
        &self,
        matches: Vec<PatternMatch>,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
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

        // Add API key patterns
        patterns.extend(build_api_key_patterns()?);

        // Add credential patterns
        patterns.extend(build_credential_patterns()?);

        // Add certificate patterns
        patterns.extend(build_certificate_patterns()?);

        Ok(patterns)
    }

    fn calculate_match_confidence(
        &self,
        pattern: &SecretPattern,
        matched_text: &str,
        line: &str,
    ) -> f64 {
        let mut confidence = pattern.confidence;

        // Reduce confidence for common test/example values
        let test_indicators = [
            "test",
            "example",
            "demo",
            "placeholder",
            "xxx",
            "***",
            "changeme",
        ];
        if test_indicators.iter().any(|&indicator| {
            matched_text.to_lowercase().contains(indicator)
                || line.to_lowercase().contains(indicator)
        }) {
            confidence *= 0.3;
        }

        // Reduce confidence for obviously fake values
        if matched_text
            .chars()
            .all(|c| c == 'x' || c == '*' || c == '0')
            || matched_text.len() < 6
        {
            confidence *= 0.1;
        }

        // Increase confidence for production-like contexts
        let prod_indicators = ["prod", "production", "live", "release"];
        if prod_indicators
            .iter()
            .any(|&indicator| line.to_lowercase().contains(indicator))
        {
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
        char_counts
            .values()
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
