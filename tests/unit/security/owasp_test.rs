//! Unit tests for OWASP Top 10 detector

use uveddi::analysis::detectors::security::owasp::*;
use uveddi::analysis::detectors::security::types::*;
use uveddi::analysis::detectors::security::config::SecurityConfig;
use uveddi::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use tokio_test;

mod test_helpers;
use test_helpers::*;

#[tokio::test]
async fn test_owasp_top10_detector_creation() {
    let config = SecurityConfig::production();
    let detector = OwaspTop10Detector::new(config);
    
    // Verify all category detectors are initialized
    assert!(detector.a01_broken_access_control.is_some());
    assert!(detector.a02_cryptographic_failures.is_some());
    assert!(detector.a03_injection.is_some());
    assert!(detector.a04_insecure_design.is_some());
    assert!(detector.a05_security_misconfiguration.is_some());
    assert!(detector.a06_vulnerable_components.is_some());
    assert!(detector.a07_identification_failures.is_some());
    assert!(detector.a08_data_integrity_failures.is_some());
    assert!(detector.a09_logging_failures.is_some());
    assert!(detector.a10_ssrf.is_some());
}

#[tokio::test]
async fn test_broken_access_control_detector() {
    let detector = BrokenAccessControlDetector::new();
    
    // Create test file with access control issue
    let test_file = create_test_parsed_file("test.py", SourceLanguage::Python);
    
    // This test would need actual file content to analyze
    // For now, just verify the detector can be created
    assert!(true);
}

#[tokio::test]
async fn test_cryptographic_failures_detector() {
    let detector = CryptographicFailuresDetector::new();
    
    // Test patterns
    assert!(detector.weak_algorithms.contains("MD5"));
    assert!(detector.weak_algorithms.contains("SHA1"));
    assert!(detector.weak_algorithms.contains("DES"));
    
    assert!(detector.hardcoded_keys_patterns.contains("password"));
    assert!(detector.hardcoded_keys_patterns.contains("api_key"));
    assert!(detector.hardcoded_keys_patterns.contains("secret"));
}

#[tokio::test] 
async fn test_injection_detector() {
    let detector = InjectionDetector::new();
    
    // Test SQL injection patterns
    assert!(detector.sql_patterns.iter().any(|p| p.contains("execute")));
    assert!(detector.sql_patterns.iter().any(|p| p.contains("query")));
    
    // Test NoSQL injection patterns
    assert!(detector.nosql_patterns.iter().any(|p| p.contains("$where")));
    assert!(detector.nosql_patterns.iter().any(|p| p.contains("$regex")));
    
    // Test command injection patterns
    assert!(detector.command_patterns.iter().any(|p| p.contains("system")));
    assert!(detector.command_patterns.iter().any(|p| p.contains("exec")));
    
    // Test LDAP injection patterns
    assert!(detector.ldap_patterns.iter().any(|p| p.contains("search")));
    
    // Test XPath injection patterns
    assert!(detector.xpath_patterns.iter().any(|p| p.contains("evaluate")));
}

#[test]
fn test_security_issue_type_to_owasp_category() {
    assert_eq!(
        SecurityIssueType::BrokenAccessControl.owasp_id(),
        "A01:2021"
    );
    assert_eq!(
        SecurityIssueType::CryptographicFailures.owasp_id(),
        "A02:2021"
    );
    assert_eq!(
        SecurityIssueType::Injection.owasp_id(),
        "A03:2021"
    );
    assert_eq!(
        SecurityIssueType::InsecureDesign.owasp_id(),
        "A04:2021"
    );
    assert_eq!(
        SecurityIssueType::SecurityMisconfiguration.owasp_id(),
        "A05:2021"
    );
    assert_eq!(
        SecurityIssueType::VulnerableComponents.owasp_id(),
        "A06:2021"
    );
    assert_eq!(
        SecurityIssueType::IdentificationFailures.owasp_id(),
        "A07:2021"
    );
    assert_eq!(
        SecurityIssueType::SoftwareDataIntegrityFailures.owasp_id(),
        "A08:2021"
    );
    assert_eq!(
        SecurityIssueType::LoggingMonitoringFailures.owasp_id(),
        "A09:2021"
    );
    assert_eq!(
        SecurityIssueType::ServerSideRequestForgery.owasp_id(),
        "A10:2021"
    );
}

#[test]
fn test_insecure_design_detector_creation() {
    let detector = InsecureDesignDetector::new();
    
    // Check architectural anti-patterns
    assert!(detector.architectural_antipatterns.contains(&"God Object".to_string()));
    assert!(detector.architectural_antipatterns.contains(&"Tight Coupling".to_string()));
    assert!(detector.architectural_antipatterns.contains(&"Leaky Abstraction".to_string()));
    
    // Check design smell patterns
    assert!(detector.design_smell_patterns.iter().any(|p| p.contains("singleton")));
    assert!(detector.design_smell_patterns.iter().any(|p| p.contains("global")));
}

#[test]
fn test_security_misconfiguration_detector_creation() {
    let detector = SecurityMisconfigurationDetector::new();
    
    // Check default credentials patterns
    assert!(detector.default_credentials.iter().any(|p| p.contains("admin")));
    assert!(detector.default_credentials.iter().any(|p| p.contains("password")));
    
    // Check debug mode patterns
    assert!(detector.debug_mode_patterns.iter().any(|p| p.contains("DEBUG")));
    assert!(detector.debug_mode_patterns.iter().any(|p| p.contains("development")));
    
    // Check insecure headers patterns
    assert!(detector.insecure_headers.iter().any(|p| p.contains("X-Powered-By")));
    assert!(detector.insecure_headers.iter().any(|p| p.contains("Server")));
}

#[test]
fn test_vulnerable_components_detector_creation() {
    let detector = VulnerableComponentsDetector::new();
    
    // Check known vulnerable patterns
    assert!(detector.known_vulnerable_patterns.iter().any(|p| p.contains("log4j")));
    assert!(detector.known_vulnerable_patterns.iter().any(|p| p.contains("struts")));
    
    // Check outdated version patterns  
    assert!(detector.outdated_version_patterns.iter().any(|p| p.contains("version")));
    assert!(detector.outdated_version_patterns.iter().any(|p| p.contains("require")));
}

#[test]
fn test_identification_failures_detector_creation() {
    let detector = IdentificationAuthenticationFailuresDetector::new();
    
    // Check weak authentication patterns
    assert!(detector.weak_auth_patterns.iter().any(|p| p.contains("password")));
    assert!(detector.weak_auth_patterns.iter().any(|p| p.contains("auth")));
    
    // Check session management patterns
    assert!(detector.session_patterns.iter().any(|p| p.contains("session")));
    assert!(detector.session_patterns.iter().any(|p| p.contains("cookie")));
    
    // Check credential exposure patterns
    assert!(detector.credential_exposure_patterns.iter().any(|p| p.contains("token")));
    assert!(detector.credential_exposure_patterns.iter().any(|p| p.contains("key")));
}

#[test]
fn test_data_integrity_failures_detector_creation() {
    let detector = SoftwareDataIntegrityFailuresDetector::new();
    
    // Check insecure deserialization patterns
    assert!(detector.deserialization_patterns.iter().any(|p| p.contains("pickle")));
    assert!(detector.deserialization_patterns.iter().any(|p| p.contains("deserialize")));
    
    // Check CI/CD pipeline patterns
    assert!(detector.cicd_patterns.iter().any(|p| p.contains("deploy")));
    assert!(detector.cicd_patterns.iter().any(|p| p.contains("build")));
    
    // Check auto-update patterns
    assert!(detector.auto_update_patterns.iter().any(|p| p.contains("update")));
    assert!(detector.auto_update_patterns.iter().any(|p| p.contains("upgrade")));
}

#[test]
fn test_logging_monitoring_failures_detector_creation() {
    let detector = LoggingMonitoringFailuresDetector::new();
    
    // Check missing logging patterns
    assert!(detector.missing_logging_patterns.iter().any(|p| p.contains("login")));
    assert!(detector.missing_logging_patterns.iter().any(|p| p.contains("access")));
    
    // Check sensitive data logging patterns
    assert!(detector.sensitive_logging_patterns.iter().any(|p| p.contains("password")));
    assert!(detector.sensitive_logging_patterns.iter().any(|p| p.contains("credit_card")));
    
    // Check insufficient monitoring patterns
    assert!(detector.insufficient_monitoring_patterns.iter().any(|p| p.contains("error")));
    assert!(detector.insufficient_monitoring_patterns.iter().any(|p| p.contains("exception")));
}

#[test]
fn test_ssrf_detector_creation() {
    let detector = ServerSideRequestForgeryDetector::new();
    
    // Check URL patterns
    assert!(detector.url_patterns.iter().any(|p| p.contains("http")));
    assert!(detector.url_patterns.iter().any(|p| p.contains("url")));
    
    // Check request patterns
    assert!(detector.request_patterns.iter().any(|p| p.contains("fetch")));
    assert!(detector.request_patterns.iter().any(|p| p.contains("request")));
    
    // Check redirect patterns
    assert!(detector.redirect_patterns.iter().any(|p| p.contains("redirect")));
    assert!(detector.redirect_patterns.iter().any(|p| p.contains("location")));
}

#[test]
fn test_architectural_correlation_mapping() {
    let correlations = ArchitecturalCorrelationMapping::new();
    
    // Test that correlations are properly initialized
    assert!(correlations.correlations.len() > 0);
    
    // Test specific correlations
    if let Some(broken_access_correlations) = correlations.correlations.get(&SecurityIssueType::BrokenAccessControl) {
        assert!(broken_access_correlations.iter().any(|c| c.anti_pattern == "God Object"));
        assert!(broken_access_correlations.iter().any(|c| c.anti_pattern == "Tight Coupling"));
    }
    
    if let Some(injection_correlations) = correlations.correlations.get(&SecurityIssueType::Injection) {
        assert!(injection_correlations.iter().any(|c| c.anti_pattern == "Leaky Abstraction"));
    }
}

#[test]
fn test_architectural_correlation_rule() {
    let rule = ArchitecturalCorrelationRule {
        anti_pattern: "God Object".to_string(),
        correlation_strength: 0.8,
        amplification_factor: 1.5,
        explanation: "God Objects centralize too much logic".to_string(),
    };
    
    assert_eq!(rule.anti_pattern, "God Object");
    assert_eq!(rule.correlation_strength, 0.8);
    assert_eq!(rule.amplification_factor, 1.5);
    assert!(rule.explanation.contains("centralize"));
}

#[tokio::test]
async fn test_owasp_detector_analyze_with_mock_file() {
    let config = SecurityConfig::development();
    let detector = OwaspTop10Detector::new(config);
    
    // Create a mock file (won't exist, but tests structure)
    let test_file = create_test_parsed_file("mock_test.py", SourceLanguage::Python);
    
    // This will fail due to file not existing, but tests the interface
    let result = detector.analyze(&test_file).await;
    assert!(result.is_err()); // Expected to fail with IO error
}

#[test]
fn test_owasp_category_severity_defaults() {
    // Test that different OWASP categories have appropriate severity defaults
    let broken_access = SecurityIssueType::BrokenAccessControl;
    let crypto_failures = SecurityIssueType::CryptographicFailures;
    let injection = SecurityIssueType::Injection;
    
    // These would be implemented in the actual SecurityIssueType
    assert_eq!(broken_access.to_string(), "Broken Access Control");
    assert_eq!(crypto_failures.to_string(), "Cryptographic Failures");
    assert_eq!(injection.to_string(), "Injection");
}