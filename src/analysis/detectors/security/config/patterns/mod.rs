//! Pattern detection for configuration security analysis
//!
//! This module provides pattern matchers for detecting secrets, credentials,
//! and known vulnerability patterns in configuration files.

pub mod secret_patterns;
pub mod vulnerability_patterns;

// Re-export pattern matchers
pub use secret_patterns::SecretPatternMatcher;
pub use vulnerability_patterns::VulnerabilityPatternMatcher;

use super::config::ConfigSecurityConfig;
use super::types::{ConfigIssue, PatternMatch};
use crate::analysis::AnalysisError;

/// Basic pattern matcher interface used by the configuration security detector.
pub trait PatternMatcher: Send + Sync {
    /// Quickly determine if the content matches the underlying pattern set
    fn match_pattern(&self, content: &str) -> Result<bool, AnalysisError>;

    /// Return a descriptive pattern type identifier
    fn get_pattern_type(&self) -> String;
}

/// Trait for configuration-specific pattern matching
pub trait ConfigPatternMatcher: PatternMatcher {
    /// Create a new pattern matcher
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError>
    where
        Self: Sized;

    /// Find all pattern matches in the given content
    fn find_matches(&self, content: &str) -> Result<Vec<PatternMatch>, AnalysisError>;

    /// Convert pattern matches to configuration issues
    fn matches_to_issues(
        &self,
        matches: Vec<PatternMatch>,
    ) -> Result<Vec<ConfigIssue>, AnalysisError>;
}

/// Combined pattern matcher that uses multiple pattern matchers
pub struct CombinedPatternMatcher {
    secret_matcher: SecretPatternMatcher,
    vulnerability_matcher: VulnerabilityPatternMatcher,
}

impl CombinedPatternMatcher {
    /// Create a new combined pattern matcher
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            secret_matcher: SecretPatternMatcher::new(config)?,
            vulnerability_matcher: VulnerabilityPatternMatcher::new(config)?,
        })
    }

    /// Find all patterns in the content
    pub fn find_all_patterns(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Find secret patterns
        let secret_matches = self.secret_matcher.find_matches(content)?;
        issues.extend(self.secret_matcher.matches_to_issues(secret_matches)?);

        // Find vulnerability patterns
        let vuln_matches = self.vulnerability_matcher.find_matches(content)?;
        issues.extend(self.vulnerability_matcher.matches_to_issues(vuln_matches)?);

        Ok(issues)
    }
}

/// Common pattern utilities
pub mod utils {
    use crate::analysis::AnalysisError;
    use regex::Regex;

    /// Compile a regex pattern safely
    pub fn compile_pattern(pattern: &str) -> Result<Regex, AnalysisError> {
        Regex::new(pattern).map_err(|e| AnalysisError::RegexError {
            error: format!("Invalid regex pattern '{}': {}", pattern, e),
        })
    }

    /// Extract context around a match (line_number is 0-indexed)
    pub fn extract_context(content: &str, line_number: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = line_number.saturating_sub(context_lines);
        let end = (line_number + context_lines + 1).min(lines.len());

        lines[start..end].join("\n")
    }

    /// Mask sensitive content in a string
    pub fn mask_sensitive(content: &str, start: usize, length: usize) -> String {
        let mut chars: Vec<char> = content.chars().collect();
        let end = (start + length).min(chars.len());

        for i in start..end {
            if chars[i].is_alphanumeric() {
                chars[i] = '*';
            }
        }

        chars.into_iter().collect()
    }

    /// Check if a line contains a suppression comment
    pub fn has_suppression_comment(line: &str, suppression_patterns: &[String]) -> bool {
        let line_upper = line.to_uppercase();
        suppression_patterns
            .iter()
            .any(|pattern| line_upper.contains(&pattern.to_uppercase()))
    }

    /// Calculate line and column from byte offset
    pub fn offset_to_line_col(content: &str, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in content.char_indices() {
            if i >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;
    use super::*;

    #[test]
    fn test_combined_pattern_matcher() {
        let config = ConfigSecurityConfig::default();
        let matcher = CombinedPatternMatcher::new(&config).unwrap();

        let content = r#"
password: "secret123"
api_url: "http://insecure.api.com"
"#;

        let issues = matcher.find_all_patterns(content).unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_context_extraction() {
        let content = "line1\nline2\nTARGET LINE\nline4\nline5";
        let context = extract_context(content, 2, 1); // line 2 (0-indexed), 1 line context

        assert!(context.contains("line2"));
        assert!(context.contains("TARGET LINE"));
        assert!(context.contains("line4"));
    }

    #[test]
    fn test_sensitive_masking() {
        let content = "password=secret123";
        let masked = mask_sensitive(content, 9, 9); // mask "secret123"
        assert_eq!(masked, "password=*********");
    }

    #[test]
    fn test_suppression_comment_detection() {
        let line = "password=secret # UVEDDI:IGNORE - this is a test";
        let patterns = vec!["UVEDDI:IGNORE".to_string()];
        assert!(has_suppression_comment(line, &patterns));

        let line2 = "password=secret # regular comment";
        assert!(!has_suppression_comment(line2, &patterns));
    }

    #[test]
    fn test_offset_to_line_col() {
        let content = "line1\nline2\ntarget";
        let (line, col) = offset_to_line_col(content, 12); // Points to 't' in "target"
        assert_eq!(line, 3);
        assert_eq!(col, 1);
    }
}
