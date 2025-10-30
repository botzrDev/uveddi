//! Pattern matching utilities for misconfiguration detection
//!
//! This module provides utilities for building and matching configuration patterns.

use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::rules::{MisconfigurationPattern, MisconfigurationRule};
use crate::analysis::detectors::security::config::language_support::utils as lang_utils;
use crate::analysis::detectors::security::config::patterns::utils;
use crate::analysis::AnalysisError;
use regex::Regex;

/// Pattern builder for misconfiguration detection
pub struct PatternBuilder;

impl PatternBuilder {
    /// Check a line against a misconfiguration rule
    pub fn check_line_against_rule(
        line: &str,
        line_number: usize,
        rule: &MisconfigurationRule,
    ) -> Result<Option<ConfigIssue>, AnalysisError> {
        let matches = match &rule.pattern {
            MisconfigurationPattern::Regex(regex) => regex.is_match(line),
            MisconfigurationPattern::KeyValue {
                key_pattern,
                value_pattern,
            } => Self::check_key_value_pattern(line, key_pattern, value_pattern.as_ref()),
            MisconfigurationPattern::Structured(_) => false, // Handled separately
        };

        if matches {
            let confidence = Self::calculate_confidence(rule, line);

            let mut issue = lang_utils::create_config_issue(
                rule.severity.clone(),
                rule.name.clone(),
                rule.description.clone(),
                Some(line_number),
                rule.remediation.clone(),
                vec!["misconfiguration".to_string()],
            );

            if let Some(cwe_id) = rule.cwe_id {
                issue = issue.with_cwe(cwe_id);
            }

            // Add confidence score and OWASP category as metadata
            if confidence < 0.8 {
                issue.severity = match issue.severity {
                    ConfigSeverity::Critical => ConfigSeverity::High,
                    ConfigSeverity::High => ConfigSeverity::Medium,
                    ConfigSeverity::Medium => ConfigSeverity::Low,
                    other => other,
                };
            }

            Ok(Some(issue))
        } else {
            Ok(None)
        }
    }

    fn check_key_value_pattern(
        line: &str,
        key_pattern: &Regex,
        value_pattern: Option<&Regex>,
    ) -> bool {
        if !key_pattern.is_match(line) {
            return false;
        }

        if let Some(value_regex) = value_pattern {
            // Look for key-value separators and check the value part
            if let Some(separator_pos) = line.find([':', '=']) {
                let value_part = &line[separator_pos + 1..].trim();
                return value_regex.is_match(value_part);
            }
        }

        true
    }

    fn calculate_confidence(rule: &MisconfigurationRule, line: &str) -> f64 {
        let mut confidence: f64 = 0.7; // Base confidence

        // Increase confidence for exact matches
        match &rule.pattern {
            MisconfigurationPattern::Regex(regex) => {
                if regex.as_str().len() > 10 {
                    confidence += 0.1; // More specific patterns are more confident
                }
            }
            MisconfigurationPattern::KeyValue {
                key_pattern,
                value_pattern,
            } => {
                confidence += 0.1; // Key-value patterns are generally more specific

                if value_pattern.is_some() {
                    confidence += 0.1; // Having both key and value patterns increases confidence
                }
            }
            MisconfigurationPattern::Structured(_) => {
                confidence += 0.2; // Structured patterns are most specific
            }
        }

        // Decrease confidence for common words that might be false positives
        let common_false_positives = ["comment", "example", "test", "demo"];
        let line_lower = line.to_lowercase();
        for &false_positive in &common_false_positives {
            if line_lower.contains(false_positive) {
                confidence -= 0.2;
                break;
            }
        }

        // Increase confidence if in a configuration section
        if line.trim().starts_with('#') {
            confidence -= 0.3; // Comments are less likely to be actual config
        } else if line.contains(':') || line.contains('=') {
            confidence += 0.1; // Looks like actual configuration
        }

        confidence.max(0.1).min(1.0)
    }

    /// Build regex patterns for common configuration formats
    pub fn build_common_patterns() -> Result<Vec<(String, Regex)>, AnalysisError> {
        let patterns = vec![
            (
                "yaml_key_value".to_string(),
                Regex::new(r"^\s*([^:]+):\s*(.+)$")?,
            ),
            (
                "ini_key_value".to_string(),
                Regex::new(r"^\s*([^=]+)=\s*(.+)$")?,
            ),
            (
                "json_key_value".to_string(),
                Regex::new(r#"^\s*"([^"]+)"\s*:\s*"?([^",}]+)"?[,}]?$"#)?,
            ),
            (
                "environment_var".to_string(),
                Regex::new(r"^([A-Z_][A-Z0-9_]*)\s*=\s*(.+)$")?,
            ),
            (
                "xml_element".to_string(),
                Regex::new(r"<([^>]+)>([^<]*)</\1>")?,
            ),
        ];

        Ok(patterns)
    }

    /// Extract key-value pairs from different configuration formats
    pub fn extract_key_value(line: &str) -> Option<(String, String)> {
        // YAML format: key: value
        if let Some(captures) = Regex::new(r"^\s*([^:]+):\s*(.+)$").ok()?.captures(line) {
            return Some((
                captures.get(1)?.as_str().trim().to_string(),
                captures.get(2)?.as_str().trim().to_string(),
            ));
        }

        // INI format: key=value
        if let Some(captures) = Regex::new(r"^\s*([^=]+)=\s*(.+)$").ok()?.captures(line) {
            return Some((
                captures.get(1)?.as_str().trim().to_string(),
                captures.get(2)?.as_str().trim().to_string(),
            ));
        }

        // JSON format: "key": "value"
        if let Some(captures) = Regex::new(r#"^\s*"([^"]+)"\s*:\s*"?([^",}]+)"?[,}]?$"#)
            .ok()?
            .captures(line)
        {
            return Some((
                captures.get(1)?.as_str().to_string(),
                captures.get(2)?.as_str().to_string(),
            ));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::rules::MisconfigurationRule;
    use super::*;

    #[test]
    fn test_key_value_extraction() {
        assert_eq!(
            PatternBuilder::extract_key_value("debug: true"),
            Some(("debug".to_string(), "true".to_string()))
        );

        assert_eq!(
            PatternBuilder::extract_key_value("ssl_enabled=false"),
            Some(("ssl_enabled".to_string(), "false".to_string()))
        );

        assert_eq!(
            PatternBuilder::extract_key_value(r#""cors": "*""#),
            Some(("cors".to_string(), "*".to_string()))
        );
    }

    #[test]
    fn test_pattern_matching() {
        let rules = MisconfigurationRule::build_default_rules().unwrap();
        let debug_rule = rules
            .iter()
            .find(|r| r.name == "Debug Mode Enabled")
            .unwrap();

        let result = PatternBuilder::check_line_against_rule("debug: true", 1, debug_rule).unwrap();
        assert!(result.is_some());

        let result =
            PatternBuilder::check_line_against_rule("debug: false", 1, debug_rule).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_confidence_calculation() {
        let rules = MisconfigurationRule::build_default_rules().unwrap();
        let debug_rule = rules
            .iter()
            .find(|r| r.name == "Debug Mode Enabled")
            .unwrap();

        let confidence = PatternBuilder::calculate_confidence(debug_rule, "debug: true");
        assert!(confidence > 0.5);

        let confidence_comment = PatternBuilder::calculate_confidence(debug_rule, "# debug: true");
        assert!(confidence_comment < confidence);
    }

    #[test]
    fn test_common_patterns() {
        let patterns = PatternBuilder::build_common_patterns().unwrap();
        assert!(!patterns.is_empty());
        assert!(patterns.len() >= 5);
    }
}
