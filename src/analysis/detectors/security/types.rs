//! Core types and structures for security analysis
//!
//! This module defines the fundamental data structures used throughout
//! the security detection system, including vulnerability types, security
//! issues, confidence scoring, and metadata structures.

use crate::ast::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Security vulnerability severity levels following industry standards
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecuritySeverity {
    /// Informational - no immediate security impact
    Info,
    /// Low severity - limited security impact
    Low,
    /// Medium severity - moderate security impact
    Medium,
    /// High severity - significant security impact requiring attention
    High,
    /// Critical severity - immediate security threat requiring urgent action
    Critical,
}

impl SecuritySeverity {
    /// Convert severity to numerical score for calculations (0.0 - 1.0)
    pub fn score(&self) -> f64 {
        match self {
            SecuritySeverity::Info => 0.1,
            SecuritySeverity::Low => 0.3,
            SecuritySeverity::Medium => 0.5,
            SecuritySeverity::High => 0.7,
            SecuritySeverity::Critical => 0.9,
        }
    }

    /// Convert numerical score to severity level
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.8 => SecuritySeverity::Critical,
            s if s >= 0.6 => SecuritySeverity::High,
            s if s >= 0.4 => SecuritySeverity::Medium,
            s if s >= 0.2 => SecuritySeverity::Low,
            _ => SecuritySeverity::Info,
        }
    }
}

impl ToString for SecuritySeverity {
    fn to_string(&self) -> String {
        match self {
            SecuritySeverity::Info => "Info".to_string(),
            SecuritySeverity::Low => "Low".to_string(),
            SecuritySeverity::Medium => "Medium".to_string(),
            SecuritySeverity::High => "High".to_string(),
            SecuritySeverity::Critical => "Critical".to_string(),
        }
    }
}

impl Default for SecuritySeverity {
    fn default() -> Self {
        SecuritySeverity::Medium
    }
}

/// Types of security vulnerabilities following OWASP classifications
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityIssueType {
    // OWASP Top 10 2021
    BrokenAccessControl,
    CryptographicFailures,
    Injection,
    InsecureDesign,
    SecurityMisconfiguration,
    VulnerableComponents,
    AuthenticationFailures,
    SoftwareDataIntegrityFailures,
    SecurityLoggingFailures,
    ServerSideRequestForgery,

    // Additional specific vulnerability types
    HardcodedSecrets,
    InsecureRandomness,
    PathTraversal,
    CrossSiteScripting,
    CrossSiteRequestForgery,
    BusinessLogicErrors,
    InsufficientInputValidation,
    ImproperErrorHandling,
    InsecureCommunication,
    PrivilegeEscalation,

    // Language-specific issues
    BufferOverflow,
    UseAfterFree,
    NullPointerDereference,
    RaceCondition,
    DeserializationVulnerabilities,

    // Custom/Unknown category
    Custom(String),
}

impl SecurityIssueType {
    /// Get OWASP category for this issue type
    pub fn owasp_category(&self) -> Option<&'static str> {
        match self {
            SecurityIssueType::BrokenAccessControl => Some("A01:2021"),
            SecurityIssueType::CryptographicFailures => Some("A02:2021"),
            SecurityIssueType::Injection => Some("A03:2021"),
            SecurityIssueType::InsecureDesign => Some("A04:2021"),
            SecurityIssueType::SecurityMisconfiguration => Some("A05:2021"),
            SecurityIssueType::VulnerableComponents => Some("A06:2021"),
            SecurityIssueType::AuthenticationFailures => Some("A07:2021"),
            SecurityIssueType::SoftwareDataIntegrityFailures => Some("A08:2021"),
            SecurityIssueType::SecurityLoggingFailures => Some("A09:2021"),
            SecurityIssueType::ServerSideRequestForgery => Some("A10:2021"),
            _ => None,
        }
    }

    /// Get default severity for this issue type
    pub fn default_severity(&self) -> SecuritySeverity {
        match self {
            SecurityIssueType::BrokenAccessControl => SecuritySeverity::High,
            SecurityIssueType::Injection => SecuritySeverity::Critical,
            SecurityIssueType::CryptographicFailures => SecuritySeverity::High,
            SecurityIssueType::HardcodedSecrets => SecuritySeverity::High,
            SecurityIssueType::BufferOverflow => SecuritySeverity::Critical,
            SecurityIssueType::UseAfterFree => SecuritySeverity::Critical,
            SecurityIssueType::CrossSiteScripting => SecuritySeverity::High,
            SecurityIssueType::PathTraversal => SecuritySeverity::High,
            SecurityIssueType::SecurityMisconfiguration => SecuritySeverity::Medium,
            SecurityIssueType::InsecureRandomness => SecuritySeverity::Medium,
            SecurityIssueType::ImproperErrorHandling => SecuritySeverity::Low,
            _ => SecuritySeverity::Medium,
        }
    }
}

impl ToString for SecurityIssueType {
    fn to_string(&self) -> String {
        match self {
            SecurityIssueType::BrokenAccessControl => "Broken Access Control".to_string(),
            SecurityIssueType::CryptographicFailures => "Cryptographic Failures".to_string(),
            SecurityIssueType::Injection => "Injection".to_string(),
            SecurityIssueType::InsecureDesign => "Insecure Design".to_string(),
            SecurityIssueType::SecurityMisconfiguration => "Security Misconfiguration".to_string(),
            SecurityIssueType::VulnerableComponents => "Vulnerable Components".to_string(),
            SecurityIssueType::AuthenticationFailures => "Authentication Failures".to_string(),
            SecurityIssueType::SoftwareDataIntegrityFailures => "Software Data Integrity Failures".to_string(),
            SecurityIssueType::SecurityLoggingFailures => "Security Logging Failures".to_string(),
            SecurityIssueType::ServerSideRequestForgery => "Server Side Request Forgery".to_string(),
            SecurityIssueType::HardcodedSecrets => "Hardcoded Secrets".to_string(),
            SecurityIssueType::InsecureRandomness => "Insecure Randomness".to_string(),
            SecurityIssueType::PathTraversal => "Path Traversal".to_string(),
            SecurityIssueType::CrossSiteScripting => "Cross-Site Scripting (XSS)".to_string(),
            SecurityIssueType::CrossSiteRequestForgery => "Cross-Site Request Forgery (CSRF)".to_string(),
            SecurityIssueType::BusinessLogicErrors => "Business Logic Errors".to_string(),
            SecurityIssueType::InsufficientInputValidation => "Insufficient Input Validation".to_string(),
            SecurityIssueType::ImproperErrorHandling => "Improper Error Handling".to_string(),
            SecurityIssueType::InsecureCommunication => "Insecure Communication".to_string(),
            SecurityIssueType::PrivilegeEscalation => "Privilege Escalation".to_string(),
            SecurityIssueType::BufferOverflow => "Buffer Overflow".to_string(),
            SecurityIssueType::UseAfterFree => "Use After Free".to_string(),
            SecurityIssueType::NullPointerDereference => "Null Pointer Dereference".to_string(),
            SecurityIssueType::RaceCondition => "Race Condition".to_string(),
            SecurityIssueType::DeserializationVulnerabilities => "Deserialization Vulnerabilities".to_string(),
            SecurityIssueType::Custom(name) => name.clone(),
        }
    }
}

/// Location information for a security issue in source code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityLocation {
    pub file_path: PathBuf,
    pub start_line: i32,
    pub end_line: i32,
    pub start_column: Option<i32>,
    pub end_column: Option<i32>,
    pub function_name: Option<String>,
    pub class_name: Option<String>,
    pub module_name: Option<String>,
}

impl SecurityLocation {
    pub fn new(file_path: PathBuf, start_line: i32, end_line: i32) -> Self {
        Self {
            file_path,
            start_line,
            end_line,
            start_column: None,
            end_column: None,
            function_name: None,
            class_name: None,
            module_name: None,
        }
    }

    pub fn with_columns(mut self, start_col: i32, end_col: i32) -> Self {
        self.start_column = Some(start_col);
        self.end_column = Some(end_col);
        self
    }

    pub fn with_context(mut self, function: Option<String>, class: Option<String>, module: Option<String>) -> Self {
        self.function_name = function;
        self.class_name = class;
        self.module_name = module;
        self
    }
}

/// Metadata about a vulnerability including CWE mapping and references
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VulnerabilityMetadata {
    pub cwe_id: Option<String>,
    pub cvss_score: Option<f64>,
    pub references: Vec<String>,
    pub tags: Vec<String>,
    pub first_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    pub false_positive_indicators: Vec<String>,
    pub architectural_context: Option<String>,
}

impl VulnerabilityMetadata {
    pub fn new() -> Self {
        Self {
            cwe_id: None,
            cvss_score: None,
            references: Vec::new(),
            tags: Vec::new(),
            first_seen: Some(chrono::Utc::now()),
            last_updated: Some(chrono::Utc::now()),
            false_positive_indicators: Vec::new(),
            architectural_context: None,
        }
    }

    pub fn with_cwe(mut self, cwe_id: String) -> Self {
        self.cwe_id = Some(cwe_id);
        self
    }

    pub fn with_cvss(mut self, score: f64) -> Self {
        self.cvss_score = Some(score);
        self
    }

    pub fn with_references(mut self, refs: Vec<String>) -> Self {
        self.references = refs;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Default for VulnerabilityMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Vulnerability type enumeration for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VulnerabilityType {
    /// Static analysis detected vulnerability
    Static,
    /// Dynamic analysis detected vulnerability
    Dynamic,
    /// Dependency vulnerability from SCA
    Dependency,
    /// Configuration vulnerability
    Configuration,
    /// AI-inferred vulnerability
    AiInferred,
    /// Hybrid detection (multiple methods)
    Hybrid,
}

impl ToString for VulnerabilityType {
    fn to_string(&self) -> String {
        match self {
            VulnerabilityType::Static => "Static".to_string(),
            VulnerabilityType::Dynamic => "Dynamic".to_string(),
            VulnerabilityType::Dependency => "Dependency".to_string(),
            VulnerabilityType::Configuration => "Configuration".to_string(),
            VulnerabilityType::AiInferred => "AI-Inferred".to_string(),
            VulnerabilityType::Hybrid => "Hybrid".to_string(),
        }
    }
}

/// Main security issue structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub id: Option<String>,
    pub issue_type: SecurityIssueType,
    pub vulnerability_type: VulnerabilityType,
    pub severity: SecuritySeverity,
    pub confidence_score: f64,
    pub title: String,
    pub description: String,
    pub location: SecurityLocation,
    pub language: Option<SourceLanguage>,
    pub remediation: Option<String>,
    pub context: HashMap<String, serde_json::Value>,
    pub metadata: VulnerabilityMetadata,
    pub detected_by: Vec<String>, // List of detectors that found this issue
    pub correlation_id: Option<String>, // Link to architectural anti-patterns
}

impl SecurityIssue {
    pub fn new(
        issue_type: SecurityIssueType,
        vulnerability_type: VulnerabilityType,
        title: String,
        description: String,
        location: SecurityLocation,
    ) -> Self {
        let severity = issue_type.default_severity();
        Self {
            id: None,
            issue_type,
            vulnerability_type,
            severity,
            confidence_score: 0.5,
            title,
            description,
            location,
            language: None,
            remediation: None,
            context: HashMap::new(),
            metadata: VulnerabilityMetadata::new(),
            detected_by: Vec::new(),
            correlation_id: None,
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

    pub fn with_language(mut self, language: SourceLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_remediation(mut self, remediation: String) -> Self {
        self.remediation = Some(remediation);
        self
    }

    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.insert(key, value);
        self
    }

    pub fn with_metadata(mut self, metadata: VulnerabilityMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_detector(mut self, detector_name: String) -> Self {
        if !self.detected_by.contains(&detector_name) {
            self.detected_by.push(detector_name);
        }
        self
    }

    pub fn with_correlation(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Check if this is a high-confidence finding
    pub fn is_high_confidence(&self) -> bool {
        self.confidence_score >= 0.8
    }

    /// Check if this is a critical or high severity issue
    pub fn is_urgent(&self) -> bool {
        matches!(self.severity, SecuritySeverity::Critical | SecuritySeverity::High)
    }

    /// Convert from a generic vulnerability structure
    pub fn from_vulnerability(
        vulnerability: crate::analysis::detectors::security::owasp::OwaspVulnerability,
        file_path: &std::path::Path,
        language: SourceLanguage,
    ) -> Result<Self, crate::analysis::AnalysisError> {
        let location = SecurityLocation::new(
            file_path.to_path_buf(),
            vulnerability.location.start_line,
            vulnerability.location.end_line,
        );

        Ok(SecurityIssue::new(
            vulnerability.issue_type,
            VulnerabilityType::Static,
            vulnerability.title,
            vulnerability.description,
            location,
        )
        .with_language(language)
        .with_confidence(vulnerability.confidence_score)
        .with_severity(vulnerability.severity)
        .with_remediation(vulnerability.remediation.unwrap_or_default())
        .with_metadata(vulnerability.metadata))
    }

    /// Get risk score combining severity and confidence
    pub fn risk_score(&self) -> f64 {
        self.severity.score() * self.confidence_score
    }

    /// Check if issue should be suppressed based on context
    pub fn should_suppress(&self, suppression_rules: &HashMap<String, Vec<String>>) -> bool {
        // Check if file path matches suppression patterns
        if let Some(patterns) = suppression_rules.get("file_patterns") {
            let file_str = self.location.file_path.to_string_lossy();
            if patterns.iter().any(|pattern| file_str.contains(pattern)) {
                return true;
            }
        }

        // Check if issue type is suppressed
        if let Some(types) = suppression_rules.get("issue_types") {
            if types.contains(&self.issue_type.to_string()) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_security_severity_scoring() {
        assert_eq!(SecuritySeverity::Critical.score(), 0.9);
        assert_eq!(SecuritySeverity::High.score(), 0.7);
        assert_eq!(SecuritySeverity::Medium.score(), 0.5);
        assert_eq!(SecuritySeverity::Low.score(), 0.3);
        assert_eq!(SecuritySeverity::Info.score(), 0.1);
    }

    #[test]
    fn test_security_severity_from_score() {
        assert_eq!(SecuritySeverity::from_score(0.9), SecuritySeverity::Critical);
        assert_eq!(SecuritySeverity::from_score(0.7), SecuritySeverity::High);
        assert_eq!(SecuritySeverity::from_score(0.5), SecuritySeverity::Medium);
        assert_eq!(SecuritySeverity::from_score(0.3), SecuritySeverity::Low);
        assert_eq!(SecuritySeverity::from_score(0.1), SecuritySeverity::Info);
    }

    #[test]
    fn test_security_issue_type_owasp_mapping() {
        assert_eq!(
            SecurityIssueType::Injection.owasp_category(),
            Some("A03:2021")
        );
        assert_eq!(
            SecurityIssueType::BrokenAccessControl.owasp_category(),
            Some("A01:2021")
        );
        assert_eq!(SecurityIssueType::HardcodedSecrets.owasp_category(), None);
    }

    #[test]
    fn test_security_issue_creation() {
        let location = SecurityLocation::new(
            PathBuf::from("test.rs"),
            10,
            15,
        );

        let issue = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "SQL Injection Vulnerability".to_string(),
            "User input is directly concatenated into SQL query".to_string(),
            location,
        )
        .with_language(SourceLanguage::Rust)
        .with_confidence(0.9)
        .with_remediation("Use parameterized queries".to_string());

        assert_eq!(issue.issue_type, SecurityIssueType::Injection);
        assert_eq!(issue.vulnerability_type, VulnerabilityType::Static);
        assert_eq!(issue.confidence_score, 0.9);
        assert_eq!(issue.language, Some(SourceLanguage::Rust));
        assert!(issue.is_high_confidence());
        assert!(issue.is_urgent());
    }

    #[test]
    fn test_risk_score_calculation() {
        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);
        
        let issue = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "Test".to_string(),
            "Test".to_string(),
            location,
        )
        .with_severity(SecuritySeverity::Critical)
        .with_confidence(0.8);

        assert_eq!(issue.risk_score(), 0.9 * 0.8); // Critical severity (0.9) * confidence (0.8)
    }

    #[test]
    fn test_vulnerability_metadata() {
        let metadata = VulnerabilityMetadata::new()
            .with_cwe("CWE-89".to_string())
            .with_cvss(7.5)
            .with_references(vec!["https://owasp.org".to_string()])
            .with_tags(vec!["injection".to_string(), "sql".to_string()]);

        assert_eq!(metadata.cwe_id, Some("CWE-89".to_string()));
        assert_eq!(metadata.cvss_score, Some(7.5));
        assert_eq!(metadata.references.len(), 1);
        assert_eq!(metadata.tags.len(), 2);
    }
}