//! Unit tests for security detection strategies

use uveddi::analysis::detectors::security::strategies::*;
use uveddi::analysis::detectors::security::types::*;
use uveddi::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use tokio_test;

mod test_helpers;
use test_helpers::*;

#[test]
fn test_vulnerability_pattern_creation() {
    let pattern = VulnerabilityPattern {
        pattern: "eval(".to_string(),
        vulnerability_type: SecurityIssueType::Injection,
        severity: SecuritySeverity::Critical,
        confidence: 0.9,
        description: "Code injection via eval".to_string(),
        remediation: Some("Avoid using eval with user input".to_string()),
    };
    
    assert_eq!(pattern.pattern, "eval(");
    assert_eq!(pattern.vulnerability_type, SecurityIssueType::Injection);
    assert_eq!(pattern.severity, SecuritySeverity::Critical);
    assert_eq!(pattern.confidence, 0.9);
    assert!(pattern.remediation.is_some());
}

#[test]
fn test_deterministic_pattern_matcher_creation() {
    let matcher = DeterministicPatternMatcher::new();
    
    // Should have patterns for supported languages
    assert!(matcher.patterns.contains_key(&SourceLanguage::Rust));
    assert!(matcher.patterns.contains_key(&SourceLanguage::Python));
    assert!(matcher.patterns.contains_key(&SourceLanguage::JavaScript));
    assert!(matcher.patterns.contains_key(&SourceLanguage::TypeScript));
}

#[test]
fn test_rust_patterns_initialization() {
    let matcher = DeterministicPatternMatcher::new();
    let rust_patterns = matcher.patterns.get(&SourceLanguage::Rust).unwrap();
    
    // Should contain memory safety patterns
    assert!(rust_patterns.iter().any(|p| p.pattern.contains("std::ptr")));
    assert!(rust_patterns.iter().any(|p| p.pattern.contains("unwrap")));
    
    // Check specific pattern details
    let unwrap_pattern = rust_patterns.iter()
        .find(|p| p.pattern == ".unwrap()")
        .unwrap();
    assert_eq!(unwrap_pattern.vulnerability_type, SecurityIssueType::ImproperErrorHandling);
    assert_eq!(unwrap_pattern.severity, SecuritySeverity::Medium);
}

#[test]
fn test_python_patterns_initialization() {
    let matcher = DeterministicPatternMatcher::new();
    let python_patterns = matcher.patterns.get(&SourceLanguage::Python).unwrap();
    
    // Should contain Python-specific vulnerability patterns
    assert!(python_patterns.iter().any(|p| p.pattern.contains("pickle.load")));
    assert!(python_patterns.iter().any(|p| p.pattern.contains("shell=True")));
    
    // Check pickle pattern details
    let pickle_pattern = python_patterns.iter()
        .find(|p| p.pattern == "pickle.load")
        .unwrap();
    assert_eq!(pickle_pattern.vulnerability_type, SecurityIssueType::DeserializationVulnerabilities);
    assert_eq!(pickle_pattern.severity, SecuritySeverity::High);
}

#[test]
fn test_javascript_patterns_initialization() {
    let matcher = DeterministicPatternMatcher::new();
    let js_patterns = matcher.patterns.get(&SourceLanguage::JavaScript).unwrap();
    let ts_patterns = matcher.patterns.get(&SourceLanguage::TypeScript).unwrap();
    
    // JavaScript and TypeScript should have similar patterns
    assert_eq!(js_patterns.len(), ts_patterns.len());
    
    // Should contain XSS patterns
    assert!(js_patterns.iter().any(|p| p.pattern.contains("document.write")));
    
    // Check document.write pattern
    let xss_pattern = js_patterns.iter()
        .find(|p| p.pattern.contains("document.write"))
        .unwrap();
    assert_eq!(xss_pattern.vulnerability_type, SecurityIssueType::CrossSiteScripting);
    assert_eq!(xss_pattern.severity, SecuritySeverity::High);
}

#[tokio::test]
async fn test_deterministic_pattern_matcher_analyze_missing_file() {
    let matcher = DeterministicPatternMatcher::new();
    
    let test_file = create_test_parsed_file("/nonexistent/file.py", SourceLanguage::Python);
    
    let result = matcher.analyze(&test_file).await;
    assert!(result.is_err()); // Should fail with IO error
}

#[test]
fn test_config_security_pattern_creation() {
    let pattern = ConfigSecurityPattern {
        key_pattern: "DEBUG".to_string(),
        dangerous_values: vec!["True".to_string(), "true".to_string()],
        vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
        severity: SecuritySeverity::High,
        description: "DEBUG mode enabled in production".to_string(),
        remediation: "Set DEBUG = False in production".to_string(),
    };
    
    assert_eq!(pattern.key_pattern, "DEBUG");
    assert_eq!(pattern.dangerous_values.len(), 2);
    assert_eq!(pattern.vulnerability_type, SecurityIssueType::SecurityMisconfiguration);
    assert_eq!(pattern.severity, SecuritySeverity::High);
}

#[test]
fn test_config_file_analyzer_creation() {
    let analyzer = ConfigFileAnalyzer::new();
    
    // Should have patterns for common config files
    assert!(analyzer.config_patterns.contains_key("settings.py"));
    assert!(analyzer.config_patterns.contains_key("Dockerfile"));
    assert!(analyzer.config_patterns.contains_key("nginx.conf"));
}

#[test]
fn test_django_config_patterns() {
    let analyzer = ConfigFileAnalyzer::new();
    let django_patterns = analyzer.config_patterns.get("settings.py").unwrap();
    
    // Should contain Django security patterns
    assert!(django_patterns.iter().any(|p| p.key_pattern == "DEBUG"));
    assert!(django_patterns.iter().any(|p| p.key_pattern == "SECRET_KEY"));
    
    // Check DEBUG pattern details
    let debug_pattern = django_patterns.iter()
        .find(|p| p.key_pattern == "DEBUG")
        .unwrap();
    assert!(debug_pattern.dangerous_values.contains(&"True".to_string()));
    assert_eq!(debug_pattern.vulnerability_type, SecurityIssueType::SecurityMisconfiguration);
}

#[test]
fn test_docker_config_patterns() {
    let analyzer = ConfigFileAnalyzer::new();
    let docker_patterns = analyzer.config_patterns.get("Dockerfile").unwrap();
    
    // Should contain Docker security patterns
    assert!(docker_patterns.iter().any(|p| p.key_pattern == "USER"));
    
    let user_pattern = docker_patterns.iter()
        .find(|p| p.key_pattern == "USER")
        .unwrap();
    assert!(user_pattern.dangerous_values.contains(&"root".to_string()));
    assert_eq!(user_pattern.vulnerability_type, SecurityIssueType::PrivilegeEscalation);
}

#[test]
fn test_nginx_config_patterns() {
    let analyzer = ConfigFileAnalyzer::new();
    let nginx_patterns = analyzer.config_patterns.get("nginx.conf").unwrap();
    
    // Should contain Nginx security patterns
    assert!(nginx_patterns.iter().any(|p| p.key_pattern == "server_tokens"));
    
    let tokens_pattern = nginx_patterns.iter()
        .find(|p| p.key_pattern == "server_tokens")
        .unwrap();
    assert!(tokens_pattern.dangerous_values.contains(&"on".to_string()));
    assert_eq!(tokens_pattern.vulnerability_type, SecurityIssueType::SecurityMisconfiguration);
}

#[tokio::test]
async fn test_config_file_analyzer_analyze_missing_file() {
    let analyzer = ConfigFileAnalyzer::new();
    
    let test_file = create_test_parsed_file("/nonexistent/settings.py", SourceLanguage::Python);
    
    let result = analyzer.analyze(&test_file).await;
    assert!(result.is_err()); // Should fail with IO error
}

#[test]
fn test_software_composition_analyzer_creation() {
    let analyzer = SoftwareCompositionAnalyzer::new();
    
    // Basic creation test - SCA is a placeholder in the current implementation
    assert!(true);
}

#[tokio::test]
async fn test_sca_analyze_unsupported_file() {
    let analyzer = SoftwareCompositionAnalyzer::new();
    
    let test_file = create_test_parsed_file("/test/random.txt", SourceLanguage::Other);
    
    let result = analyzer.analyze(&test_file).await.unwrap();
    assert_eq!(result.len(), 0); // Should return no issues for unsupported files
}

#[test]
fn test_architectural_correlation_rule_creation() {
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

#[test]
fn test_vulnerability_correlation_engine_creation() {
    let engine = VulnerabilityCorrelationEngine::new();
    
    // Should have correlation rules initialized
    assert!(engine.correlation_rules.len() > 0);
    
    // Should have rules for major vulnerability types
    assert!(engine.correlation_rules.contains_key(&SecurityIssueType::BrokenAccessControl));
    assert!(engine.correlation_rules.contains_key(&SecurityIssueType::Injection));
}

#[test]
fn test_vulnerability_correlation_engine_correlate_issue() {
    let engine = VulnerabilityCorrelationEngine::new();
    
    let location = SecurityLocation::new(PathBuf::from("test.rs"), 10, 20);
    let security_issue = SecurityIssue::new(
        SecurityIssueType::BrokenAccessControl,
        VulnerabilityType::Static,
        "Access Control Issue".to_string(),
        "Test issue".to_string(),
        location,
    ).with_id("test_issue_123".to_string());
    
    let detected_anti_patterns = vec!["God Object".to_string(), "Tight Coupling".to_string()];
    let correlations = engine.correlate_issue(&security_issue, &detected_anti_patterns);
    
    assert!(correlations.len() > 0);
    
    // Should find correlations for God Object and Tight Coupling
    assert!(correlations.iter().any(|c| c.anti_pattern == "God Object"));
    assert!(correlations.iter().any(|c| c.anti_pattern == "Tight Coupling"));
    
    // Check correlation details
    let god_object_correlation = correlations.iter()
        .find(|c| c.anti_pattern == "God Object")
        .unwrap();
    assert_eq!(god_object_correlation.correlation_strength, 0.8);
    assert_eq!(god_object_correlation.amplification_factor, 1.5);
}

#[test]
fn test_vulnerability_correlation_no_match() {
    let engine = VulnerabilityCorrelationEngine::new();
    
    let location = SecurityLocation::new(PathBuf::from("test.rs"), 10, 20);
    let security_issue = SecurityIssue::new(
        SecurityIssueType::BrokenAccessControl,
        VulnerabilityType::Static,
        "Access Control Issue".to_string(),
        "Test issue".to_string(),
        location,
    );
    
    // Anti-patterns that don't correlate with access control
    let detected_anti_patterns = vec!["Magic Values".to_string(), "Dead Code".to_string()];
    let correlations = engine.correlate_issue(&security_issue, &detected_anti_patterns);
    
    // Should have no correlations for these anti-patterns
    assert_eq!(correlations.len(), 0);
}

#[test]
fn test_security_correlation_creation() {
    let correlation = SecurityCorrelation {
        security_issue_id: "issue_123".to_string(),
        anti_pattern: "Leaky Abstraction".to_string(),
        correlation_strength: 0.9,
        amplification_factor: 2.0,
        explanation: "Leaky abstractions expose implementation details".to_string(),
    };
    
    assert_eq!(correlation.security_issue_id, "issue_123");
    assert_eq!(correlation.anti_pattern, "Leaky Abstraction");
    assert_eq!(correlation.correlation_strength, 0.9);
    assert_eq!(correlation.amplification_factor, 2.0);
}

#[test]
fn test_injection_correlation_mapping() {
    let engine = VulnerabilityCorrelationEngine::new();
    
    let injection_rules = engine.correlation_rules.get(&SecurityIssueType::Injection);
    assert!(injection_rules.is_some());
    
    let rules = injection_rules.unwrap();
    // Should have correlation with Leaky Abstraction
    assert!(rules.iter().any(|r| r.anti_pattern == "Leaky Abstraction"));
    
    let leaky_rule = rules.iter()
        .find(|r| r.anti_pattern == "Leaky Abstraction")
        .unwrap();
    assert_eq!(leaky_rule.correlation_strength, 0.9);
    assert_eq!(leaky_rule.amplification_factor, 2.0);
}

#[test]
fn test_pattern_serialization() {
    let pattern = VulnerabilityPattern {
        pattern: "test_pattern".to_string(),
        vulnerability_type: SecurityIssueType::Injection,
        severity: SecuritySeverity::High,
        confidence: 0.8,
        description: "Test pattern".to_string(),
        remediation: Some("Fix it".to_string()),
    };
    
    // Test JSON serialization
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("test_pattern"));
    assert!(json.contains("Injection"));
    
    // Test deserialization
    let deserialized: VulnerabilityPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.pattern, pattern.pattern);
    assert_eq!(deserialized.vulnerability_type, pattern.vulnerability_type);
}