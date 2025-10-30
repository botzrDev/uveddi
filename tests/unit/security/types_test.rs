//! Unit tests for security types module

use uveddi::analysis::detectors::security::types::*;
use std::path::PathBuf;

#[test]
fn test_security_severity_ordering() {
    assert!(SecuritySeverity::Critical > SecuritySeverity::High);
    assert!(SecuritySeverity::High > SecuritySeverity::Medium);
    assert!(SecuritySeverity::Medium > SecuritySeverity::Low);
    assert!(SecuritySeverity::Low > SecuritySeverity::Info);
}

#[test]
fn test_security_severity_score() {
    assert_eq!(SecuritySeverity::Critical.score(), 5);
    assert_eq!(SecuritySeverity::High.score(), 4);
    assert_eq!(SecuritySeverity::Medium.score(), 3);
    assert_eq!(SecuritySeverity::Low.score(), 2);
    assert_eq!(SecuritySeverity::Info.score(), 1);
}

#[test]
fn test_security_issue_type_display() {
    assert_eq!(SecurityIssueType::BrokenAccessControl.to_string(), "Broken Access Control");
    assert_eq!(SecurityIssueType::CryptographicFailures.to_string(), "Cryptographic Failures");
    assert_eq!(SecurityIssueType::Injection.to_string(), "Injection");
    assert_eq!(SecurityIssueType::InsecureDesign.to_string(), "Insecure Design");
}

#[test]
fn test_security_issue_type_owasp_id() {
    assert_eq!(SecurityIssueType::BrokenAccessControl.owasp_id(), "A01:2021");
    assert_eq!(SecurityIssueType::CryptographicFailures.owasp_id(), "A02:2021");
    assert_eq!(SecurityIssueType::Injection.owasp_id(), "A03:2021");
    assert_eq!(SecurityIssueType::InsecureDesign.owasp_id(), "A04:2021");
    assert_eq!(SecurityIssueType::SecurityMisconfiguration.owasp_id(), "A05:2021");
}

#[test]
fn test_security_location_creation() {
    let path = PathBuf::from("/test/file.rs");
    let location = SecurityLocation::new(path.clone(), 10, 20);
    
    assert_eq!(location.file_path, path);
    assert_eq!(location.start_line, 10);
    assert_eq!(location.end_line, 20);
    assert!(location.column_start.is_none());
    assert!(location.column_end.is_none());
}

#[test]
fn test_security_location_with_columns() {
    let path = PathBuf::from("/test/file.rs");
    let location = SecurityLocation::new(path.clone(), 10, 20)
        .with_columns(5, 15);
    
    assert_eq!(location.column_start, Some(5));
    assert_eq!(location.column_end, Some(15));
}

#[test]
fn test_security_issue_creation() {
    let location = SecurityLocation::new(PathBuf::from("/test/file.rs"), 10, 20);
    let issue = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Potential SQL injection vulnerability".to_string(),
        location.clone(),
    );
    
    assert_eq!(issue.issue_type, SecurityIssueType::Injection);
    assert_eq!(issue.vulnerability_type, VulnerabilityType::Static);
    assert_eq!(issue.title, "SQL Injection");
    assert_eq!(issue.description, "Potential SQL injection vulnerability");
    assert_eq!(issue.location, location);
    assert!(issue.id.is_some());
    assert_eq!(issue.severity, SecuritySeverity::High); // Default for injection
    assert_eq!(issue.confidence_score, 0.5); // Default
}

#[test]
fn test_security_issue_builder_pattern() {
    let location = SecurityLocation::new(PathBuf::from("/test/file.rs"), 10, 20);
    let issue = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Potential SQL injection vulnerability".to_string(),
        location,
    )
    .with_severity(SecuritySeverity::Critical)
    .with_confidence(0.95)
    .with_cwe_id("CWE-89".to_string())
    .with_owasp_category("A03:2021".to_string())
    .with_remediation("Use parameterized queries".to_string())
    .with_detector("TaintAnalysis".to_string())
    .with_language(crate::ast::SourceLanguage::Python)
    .with_architectural_correlation(vec!["Leaky Abstraction".to_string()]);
    
    assert_eq!(issue.severity, SecuritySeverity::Critical);
    assert_eq!(issue.confidence_score, 0.95);
    assert_eq!(issue.cwe_id, Some("CWE-89".to_string()));
    assert_eq!(issue.owasp_category, Some("A03:2021".to_string()));
    assert_eq!(issue.remediation_advice, Some("Use parameterized queries".to_string()));
    assert_eq!(issue.detected_by, vec!["TaintAnalysis".to_string()]);
    assert_eq!(issue.language, Some(crate::ast::SourceLanguage::Python));
    assert_eq!(issue.architectural_correlations, vec!["Leaky Abstraction".to_string()]);
}

#[test]
fn test_vulnerability_metadata_creation() {
    let metadata = VulnerabilityMetadata {
        cve_id: Some("CVE-2021-12345".to_string()),
        cvss_score: Some(7.5),
        exploit_available: true,
        patch_available: true,
        first_detected: chrono::Utc::now(),
        last_seen: chrono::Utc::now(),
        references: vec!["https://example.com/advisory".to_string()],
    };
    
    assert_eq!(metadata.cve_id, Some("CVE-2021-12345".to_string()));
    assert_eq!(metadata.cvss_score, Some(7.5));
    assert!(metadata.exploit_available);
    assert!(metadata.patch_available);
    assert_eq!(metadata.references.len(), 1);
}

#[test]
fn test_vulnerability_type_variants() {
    let static_vuln = VulnerabilityType::Static;
    let dynamic_vuln = VulnerabilityType::Dynamic;
    let config_vuln = VulnerabilityType::Configuration;
    let dependency_vuln = VulnerabilityType::Dependency;
    let composite_vuln = VulnerabilityType::Composite;
    
    // Test that all variants can be created
    assert!(matches!(static_vuln, VulnerabilityType::Static));
    assert!(matches!(dynamic_vuln, VulnerabilityType::Dynamic));
    assert!(matches!(config_vuln, VulnerabilityType::Configuration));
    assert!(matches!(dependency_vuln, VulnerabilityType::Dependency));
    assert!(matches!(composite_vuln, VulnerabilityType::Composite));
}

#[test]
fn test_security_issue_type_from_string() {
    use std::str::FromStr;
    
    // This test would require implementing FromStr for SecurityIssueType
    // For now, we'll test the display implementation
    let issue_type = SecurityIssueType::ServerSideRequestForgery;
    let display_string = issue_type.to_string();
    assert_eq!(display_string, "Server-Side Request Forgery");
}

#[test]
fn test_security_issue_equality() {
    let location = SecurityLocation::new(PathBuf::from("/test/file.rs"), 10, 20);
    let issue1 = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Potential SQL injection vulnerability".to_string(),
        location.clone(),
    );
    
    let issue2 = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Potential SQL injection vulnerability".to_string(),
        location,
    );
    
    // Issues with different IDs should not be equal
    assert_ne!(issue1.id, issue2.id);
}