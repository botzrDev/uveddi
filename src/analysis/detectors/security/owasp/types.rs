//! Core types for OWASP security vulnerability detection
//!
//! This module defines the fundamental types and enums used across the OWASP
//! detector modules for consistent vulnerability representation and analysis.

use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::ast::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OWASP Top 10 2021 categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwaspCategory {
    /// A01:2021 – Broken Access Control
    BrokenAccessControl,
    /// A02:2021 – Cryptographic Failures
    CryptographicFailures,
    /// A03:2021 – Injection
    Injection,
    /// A04:2021 – Insecure Design
    InsecureDesign,
    /// A05:2021 – Security Misconfiguration
    SecurityMisconfiguration,
    /// A06:2021 – Vulnerable and Outdated Components
    VulnerableComponents,
    /// A07:2021 – Identification and Authentication Failures
    AuthenticationFailures,
    /// A08:2021 – Software and Data Integrity Failures
    DataIntegrityFailures,
    /// A09:2021 – Security Logging and Monitoring Failures
    LoggingFailures,
    /// A10:2021 – Server-Side Request Forgery (SSRF)
    ServerSideRequestForgery,
}

impl OwaspCategory {
    /// Get the official OWASP identifier
    pub fn identifier(&self) -> &'static str {
        match self {
            OwaspCategory::BrokenAccessControl => "A01:2021",
            OwaspCategory::CryptographicFailures => "A02:2021",
            OwaspCategory::Injection => "A03:2021",
            OwaspCategory::InsecureDesign => "A04:2021",
            OwaspCategory::SecurityMisconfiguration => "A05:2021",
            OwaspCategory::VulnerableComponents => "A06:2021",
            OwaspCategory::AuthenticationFailures => "A07:2021",
            OwaspCategory::DataIntegrityFailures => "A08:2021",
            OwaspCategory::LoggingFailures => "A09:2021",
            OwaspCategory::ServerSideRequestForgery => "A10:2021",
        }
    }

    /// Get description of the vulnerability category
    pub fn description(&self) -> &'static str {
        match self {
            OwaspCategory::BrokenAccessControl => "Access control enforces policy such that users cannot act outside of their intended permissions",
            OwaspCategory::CryptographicFailures => "Failures related to cryptography which often leads to sensitive data exposure",
            OwaspCategory::Injection => "User data is not validated, filtered, or sanitized by the application",
            OwaspCategory::InsecureDesign => "Risks related to design and architectural flaws",
            OwaspCategory::SecurityMisconfiguration => "Missing appropriate security hardening across the application stack",
            OwaspCategory::VulnerableComponents => "Components, such as libraries, frameworks, and other software modules",
            OwaspCategory::AuthenticationFailures => "Application functions related to authentication and session management",
            OwaspCategory::DataIntegrityFailures => "Code and infrastructure that does not protect against integrity violations",
            OwaspCategory::LoggingFailures => "Insufficient logging and monitoring, coupled with missing or ineffective integration",
            OwaspCategory::ServerSideRequestForgery => "SSRF flaws occur whenever a web application is fetching a remote resource",
        }
    }

    /// Get related architectural anti-patterns
    pub fn related_anti_patterns(&self) -> Vec<&'static str> {
        match self {
            OwaspCategory::BrokenAccessControl => vec!["God Object", "Tight Coupling"],
            OwaspCategory::CryptographicFailures => vec!["Leaky Abstraction", "Magic Values"],
            OwaspCategory::Injection => vec!["Leaky Abstraction", "Tight Coupling"],
            OwaspCategory::InsecureDesign => vec!["God Object", "Tight Coupling", "Dead Code"],
            OwaspCategory::SecurityMisconfiguration => vec!["Magic Values", "Dead Code"],
            OwaspCategory::VulnerableComponents => vec!["Tight Coupling", "Dead Code"],
            OwaspCategory::AuthenticationFailures => vec!["God Object", "Tight Coupling"],
            OwaspCategory::DataIntegrityFailures => vec!["Leaky Abstraction"],
            OwaspCategory::LoggingFailures => vec!["Dead Code", "Magic Values"],
            OwaspCategory::ServerSideRequestForgery => vec!["Leaky Abstraction"],
        }
    }
}

/// Represents an OWASP vulnerability finding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwaspVulnerability {
    pub category: OwaspCategory,
    pub issue_type: SecurityIssueType,
    pub title: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence_score: f64,
    pub location: SecurityLocation,
    pub remediation: Option<String>,
    pub metadata: VulnerabilityMetadata,
    pub architectural_correlation: Vec<String>,
}

impl OwaspVulnerability {
    pub fn new(
        category: OwaspCategory,
        issue_type: SecurityIssueType,
        title: String,
        description: String,
        location: SecurityLocation,
    ) -> Self {
        Self {
            category,
            issue_type,
            title,
            description,
            severity: SecuritySeverity::Medium,
            confidence_score: 0.5,
            location,
            remediation: None,
            metadata: VulnerabilityMetadata::new(),
            architectural_correlation: Vec::new(),
        }
    }

    pub fn with_severity(mut self, severity: SecuritySeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence_score = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_remediation(mut self, remediation: String) -> Self {
        self.remediation = Some(remediation);
        self
    }

    pub fn with_metadata(mut self, metadata: VulnerabilityMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_architectural_correlation(mut self, correlations: Vec<String>) -> Self {
        self.architectural_correlation = correlations;
        self
    }
}

/// Trait for category-specific OWASP detectors
#[async_trait::async_trait]
pub trait OwaspCategoryDetector: Send + Sync {
    async fn detect(
        &self,
        file: &crate::ast::ParsedFile,
    ) -> Result<Vec<OwaspVulnerability>, crate::analysis::AnalysisError>;
}

/// Pattern for analyzing OWASP vulnerabilities in different languages
#[derive(Debug, Clone)]
pub struct OwaspAnalysisPattern {
    pub pattern: String,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub languages: Vec<SourceLanguage>,
}

impl OwaspAnalysisPattern {
    pub fn new(
        pattern: String,
        description: String,
        confidence: f64,
        severity: SecuritySeverity,
    ) -> Self {
        Self {
            pattern,
            description,
            confidence,
            severity,
            languages: Vec::new(),
        }
    }

    pub fn with_languages(mut self, languages: Vec<SourceLanguage>) -> Self {
        self.languages = languages;
        self
    }

    pub fn supports_language(&self, language: &SourceLanguage) -> bool {
        self.languages.is_empty() || self.languages.contains(language)
    }
}

/// Configuration for OWASP analyzer behavior
#[derive(Debug, Clone)]
pub struct OwaspAnalyzerConfig {
    pub confidence_threshold: f64,
    pub enabled_categories: Vec<OwaspCategory>,
    pub language_patterns: HashMap<SourceLanguage, Vec<OwaspAnalysisPattern>>,
    pub false_positive_reduction: bool,
}

impl Default for OwaspAnalyzerConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.5,
            enabled_categories: vec![
                OwaspCategory::BrokenAccessControl,
                OwaspCategory::CryptographicFailures,
                OwaspCategory::Injection,
                OwaspCategory::InsecureDesign,
                OwaspCategory::SecurityMisconfiguration,
                OwaspCategory::VulnerableComponents,
                OwaspCategory::AuthenticationFailures,
                OwaspCategory::DataIntegrityFailures,
                OwaspCategory::LoggingFailures,
                OwaspCategory::ServerSideRequestForgery,
            ],
            language_patterns: HashMap::new(),
            false_positive_reduction: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_owasp_category_properties() {
        let category = OwaspCategory::Injection;
        assert_eq!(category.identifier(), "A03:2021");
        assert!(!category.description().is_empty());
        assert!(!category.related_anti_patterns().is_empty());
    }

    #[test]
    fn test_owasp_vulnerability_creation() {
        let location = SecurityLocation::new(PathBuf::from("test.rs"), 10, 10);

        let vuln = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "SQL Injection".to_string(),
            "Potential SQL injection vulnerability".to_string(),
            location,
        )
        .with_confidence(0.8)
        .with_severity(SecuritySeverity::Critical);

        assert_eq!(vuln.category, OwaspCategory::Injection);
        assert_eq!(vuln.confidence_score, 0.8);
        assert_eq!(vuln.severity, SecuritySeverity::Critical);
    }

    #[test]
    fn test_owasp_analysis_pattern() {
        let pattern = OwaspAnalysisPattern::new(
            "sql_query".to_string(),
            "SQL query pattern".to_string(),
            0.7,
            SecuritySeverity::High,
        )
        .with_languages(vec![SourceLanguage::Rust, SourceLanguage::Python]);

        assert!(pattern.supports_language(&SourceLanguage::Rust));
        assert!(pattern.supports_language(&SourceLanguage::Python));
        assert!(!pattern.supports_language(&SourceLanguage::JavaScript));
    }

    #[test]
    fn test_default_config() {
        let config = OwaspAnalyzerConfig::default();
        assert_eq!(config.confidence_threshold, 0.5);
        assert_eq!(config.enabled_categories.len(), 10);
        assert!(config.false_positive_reduction);
    }
}
