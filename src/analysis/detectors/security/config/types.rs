//! Core type definitions for configuration security detection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration file types that can be analyzed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigType {
    /// YAML configuration files
    Yaml,
    /// JSON configuration files
    Json,
    /// TOML configuration files
    Toml,
    /// Environment variable files (.env)
    Environment,
}

/// Severity levels for configuration security issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigSeverity {
    /// Critical security vulnerabilities
    Critical,
    /// High-risk security issues
    High,
    /// Medium-risk security issues
    Medium,
    /// Low-risk security issues
    Low,
    /// Informational findings
    Info,
}

/// Configuration security issue
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigIssue {
    /// Issue severity
    pub severity: ConfigSeverity,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Issue title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Line number where issue was found
    pub line_number: Option<usize>,
    /// Column number where issue was found
    pub column_number: Option<usize>,
    /// Code snippet showing the issue
    pub code_snippet: Option<String>,
    /// Remediation guidance
    pub remediation: Option<String>,
    /// External references
    pub references: Vec<String>,
    /// OWASP category
    pub owasp_category: Option<String>,
    /// CWE identifier
    pub cwe_id: Option<u32>,
    /// Additional tags
    pub tags: Vec<String>,
}

/// Context information for configuration analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigAnalysisContext {
    /// Configuration file type
    pub config_type: ConfigType,
    /// File path being analyzed
    pub file_path: String,
    /// Configuration schema if available
    pub schema: Option<String>,
    /// Environment context (dev, staging, prod)
    pub environment: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Pattern match result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Matched pattern identifier
    pub pattern_id: String,
    /// Matched text
    pub matched_text: String,
    /// Match confidence
    pub confidence: f64,
    /// Line number of match
    pub line_number: usize,
    /// Column number of match
    pub column_number: usize,
    /// Context around the match
    pub context: String,
}

/// Analysis result for a configuration file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigAnalysisResult {
    /// Analysis context
    pub context: ConfigAnalysisContext,
    /// Discovered issues
    pub issues: Vec<ConfigIssue>,
    /// Pattern matches found
    pub pattern_matches: Vec<PatternMatch>,
    /// Analysis metadata
    pub analysis_metadata: HashMap<String, String>,
}

impl ConfigIssue {
    /// Create a new configuration issue
    pub fn new(
        severity: ConfigSeverity,
        confidence: f64,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            confidence,
            title: title.into(),
            description: description.into(),
            line_number: None,
            column_number: None,
            code_snippet: None,
            remediation: None,
            references: Vec::new(),
            owasp_category: None,
            cwe_id: None,
            tags: Vec::new(),
        }
    }

    /// Set location information
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line_number = Some(line);
        self.column_number = Some(column);
        self
    }

    /// Set code snippet
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.code_snippet = Some(snippet.into());
        self
    }

    /// Set remediation guidance
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set OWASP category
    pub fn with_owasp(mut self, category: impl Into<String>) -> Self {
        self.owasp_category = Some(category.into());
        self
    }

    /// Set CWE identifier
    pub fn with_cwe(mut self, cwe_id: u32) -> Self {
        self.cwe_id = Some(cwe_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_issue_builder() {
        let issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.9,
            "Hardcoded Password",
            "Password found in configuration"
        )
        .with_location(10, 15)
        .with_snippet("password: 'secret123'")
        .with_remediation("Use environment variables for sensitive data")
        .with_tag("credential")
        .with_owasp("A02:2021 - Cryptographic Failures")
        .with_cwe(798);

        assert_eq!(issue.severity, ConfigSeverity::High);
        assert_eq!(issue.confidence, 0.9);
        assert_eq!(issue.line_number, Some(10));
        assert_eq!(issue.column_number, Some(15));
        assert!(issue.tags.contains(&"credential".to_string()));
        assert_eq!(issue.cwe_id, Some(798));
    }

    #[test]
    fn test_config_type_serialization() {
        let config_type = ConfigType::Yaml;
        let serialized = serde_json::to_string(&config_type).unwrap();
        let deserialized: ConfigType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config_type, deserialized);
    }
}