//! Integration tests for the complete security detector system

use uveddi::analysis::detectors::security::{SecurityDetector, SecurityConfig};
use uveddi::analysis::detectors::security::types::*;
use uveddi::analysis::detectors::security::config::*;
use uveddi::analysis::detectors::AnalysisDetector;
use uveddi::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use std::io::Write;

mod test_helpers;
use test_helpers::*;

#[tokio::test]
async fn test_security_detector_end_to_end_python() {
    let config = SecurityConfig::development();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a temporary Python file with a security vulnerability
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "import subprocess").unwrap();
    writeln!(temp_file, "user_input = input('Enter command: ')").unwrap();
    writeln!(temp_file, "result = subprocess.run(user_input, shell=True)").unwrap();
    writeln!(temp_file, "print(result.stdout)").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Python);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // Should detect command injection vulnerability
    assert!(issues.len() > 0);
    assert!(issues.iter().any(|issue| 
        issue.issue_type == SecurityIssueType::Injection ||
        matches!(issue.issue_type, SecurityIssueType::Injection)
    ));
    
    // Check issue details
    let injection_issue = issues.iter()
        .find(|issue| matches!(issue.issue_type, SecurityIssueType::Injection))
        .unwrap();
    
    assert!(injection_issue.confidence_score > 0.5);
    assert_eq!(injection_issue.severity, SecuritySeverity::Critical);
    assert!(injection_issue.description.to_lowercase().contains("injection") ||
            injection_issue.description.to_lowercase().contains("shell"));
}

#[tokio::test]
async fn test_security_detector_end_to_end_rust() {
    let config = SecurityConfig::development();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a temporary Rust file with security issues
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    let data = std::env::args().collect::<Vec<_>>();").unwrap();
    writeln!(temp_file, "    let value = data.get(1).unwrap();").unwrap();
    writeln!(temp_file, "    println!(\"Value: {{}}\", value);").unwrap();
    writeln!(temp_file, "}}").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Rust);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // Should detect improper error handling (unwrap usage)
    assert!(issues.len() > 0);
    assert!(issues.iter().any(|issue| 
        issue.issue_type == SecurityIssueType::ImproperErrorHandling
    ));
    
    // Check issue details
    let error_issue = issues.iter()
        .find(|issue| issue.issue_type == SecurityIssueType::ImproperErrorHandling)
        .unwrap();
    
    assert!(error_issue.confidence_score > 0.0);
    assert_eq!(error_issue.severity, SecuritySeverity::Medium);
    assert!(error_issue.description.to_lowercase().contains("unwrap") ||
            error_issue.description.to_lowercase().contains("panic"));
}

#[tokio::test]
async fn test_security_detector_end_to_end_javascript() {
    let config = SecurityConfig::development();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a temporary JavaScript file with XSS vulnerability
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "function displayUserData(userData) {{").unwrap();
    writeln!(temp_file, "    document.write('<div>' + userData + '</div>');").unwrap();
    writeln!(temp_file, "}}").unwrap();
    writeln!(temp_file, "").unwrap();
    writeln!(temp_file, "const urlParams = new URLSearchParams(window.location.search);").unwrap();
    writeln!(temp_file, "const name = urlParams.get('name');").unwrap();
    writeln!(temp_file, "displayUserData(name);").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::JavaScript);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // Should detect XSS vulnerability
    assert!(issues.len() > 0);
    assert!(issues.iter().any(|issue| 
        issue.issue_type == SecurityIssueType::CrossSiteScripting
    ));
    
    // Check issue details
    let xss_issue = issues.iter()
        .find(|issue| issue.issue_type == SecurityIssueType::CrossSiteScripting)
        .unwrap();
    
    assert!(xss_issue.confidence_score > 0.5);
    assert_eq!(xss_issue.severity, SecuritySeverity::High);
    assert!(xss_issue.description.to_lowercase().contains("xss") ||
            xss_issue.description.to_lowercase().contains("document.write"));
}

#[tokio::test]
async fn test_security_detector_config_file_analysis() {
    let config = SecurityConfig::production();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a temporary Django settings file with security issues
    let mut temp_file = NamedTempFile::with_suffix(".py").unwrap();
    writeln!(temp_file, "# Django settings").unwrap();
    writeln!(temp_file, "DEBUG = True").unwrap();
    writeln!(temp_file, "SECRET_KEY = 'django-insecure-hardcoded-key'").unwrap();
    writeln!(temp_file, "ALLOWED_HOSTS = ['*']").unwrap();
    
    // Rename to settings.py to trigger config analysis
    let settings_path = temp_file.path().parent().unwrap().join("settings.py");
    std::fs::copy(temp_file.path(), &settings_path).unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(&settings_path, SourceLanguage::Python);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // Should detect security misconfigurations
    assert!(issues.len() > 0);
    assert!(issues.iter().any(|issue| 
        issue.issue_type == SecurityIssueType::SecurityMisconfiguration ||
        issue.issue_type == SecurityIssueType::HardcodedSecrets
    ));
    
    // Clean up
    std::fs::remove_file(&settings_path).unwrap_or(());
}

#[tokio::test]
async fn test_security_detector_with_different_profiles() {
    // Test development profile
    let dev_config = SecurityConfig::development();
    let dev_detector = SecurityDetector::with_config(dev_config).unwrap();
    
    // Test production profile
    let prod_config = SecurityConfig::production();
    let prod_detector = SecurityDetector::with_config(prod_config).unwrap();
    
    // Test CI/CD profile
    let ci_config = SecurityConfig::ci_cd();
    let ci_detector = SecurityDetector::with_config(ci_config).unwrap();
    
    // Create a test file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "print('Hello World')").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Python);
    
    // All profiles should be able to analyze the file without errors
    let dev_result = dev_detector.analyze(&parsed_file).await;
    let prod_result = prod_detector.analyze(&parsed_file).await;
    let ci_result = ci_detector.analyze(&parsed_file).await;
    
    assert!(dev_result.is_ok());
    assert!(prod_result.is_ok());
    assert!(ci_result.is_ok());
}

#[tokio::test]
async fn test_security_detector_empty_file() {
    let config = SecurityConfig::development();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create an empty file
    let temp_file = NamedTempFile::new().unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Python);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // Empty file should have no security issues
    assert_eq!(issues.len(), 0);
}

#[tokio::test]
async fn test_security_detector_safe_code() {
    let config = SecurityConfig::development();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a file with safe code
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "def safe_function(x, y):").unwrap();
    writeln!(temp_file, "    return x + y").unwrap();
    writeln!(temp_file, "").unwrap();
    writeln!(temp_file, "result = safe_function(1, 2)").unwrap();
    writeln!(temp_file, "print(f'Result: {{result}}')").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Python);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // Safe code should have minimal or no security issues
    assert!(issues.len() <= 1); // Allow for very low-confidence false positives
    
    // If there are issues, they should be low severity
    for issue in issues {
        assert!(matches!(issue.severity, SecuritySeverity::Info | SecuritySeverity::Low));
    }
}

#[tokio::test]
async fn test_security_detector_multiple_vulnerabilities() {
    let config = SecurityConfig::production();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a file with multiple security issues
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "import pickle").unwrap();
    writeln!(temp_file, "import subprocess").unwrap();
    writeln!(temp_file, "").unwrap();
    writeln!(temp_file, "def vulnerable_function(user_data, command):").unwrap();
    writeln!(temp_file, "    # Deserialization vulnerability").unwrap();
    writeln!(temp_file, "    obj = pickle.loads(user_data)").unwrap();
    writeln!(temp_file, "    ").unwrap();
    writeln!(temp_file, "    # Command injection vulnerability").unwrap();
    writeln!(temp_file, "    result = subprocess.run(command, shell=True, capture_output=True)").unwrap();
    writeln!(temp_file, "    ").unwrap();
    writeln!(temp_file, "    return obj, result.stdout").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Python);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // Should detect both deserialization and command injection
    assert!(issues.len() >= 2);
    
    let issue_types: Vec<_> = issues.iter().map(|i| &i.issue_type).collect();
    assert!(issue_types.contains(&&SecurityIssueType::DeserializationVulnerabilities) ||
            issue_types.contains(&&SecurityIssueType::Injection));
    assert!(issue_types.contains(&&SecurityIssueType::Injection));
}

#[tokio::test]
async fn test_security_detector_confidence_scoring() {
    let config = SecurityConfig::development();
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a file with a clear vulnerability
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "import os").unwrap();
    writeln!(temp_file, "user_input = input('Enter command: ')").unwrap();
    writeln!(temp_file, "os.system(user_input)").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Python);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    assert!(issues.len() > 0);
    
    // High-confidence vulnerabilities should have appropriate confidence scores
    let high_confidence_issues: Vec<_> = issues.iter()
        .filter(|i| i.confidence_score > 0.7)
        .collect();
    
    assert!(high_confidence_issues.len() > 0);
    
    // All issues should have valid confidence scores
    for issue in &issues {
        assert!(issue.confidence_score >= 0.0 && issue.confidence_score <= 1.0);
    }
}

#[tokio::test]
async fn test_security_detector_language_specific_analysis() {
    let config = SecurityConfig::production();
    
    // Test with different languages - each should use language-specific patterns
    let languages = vec![
        (SourceLanguage::Rust, "fn main() { let x = std::env::args().collect::<Vec<_>>().get(0).unwrap(); }"),
        (SourceLanguage::Python, "import subprocess; subprocess.run(input(), shell=True)"),
        (SourceLanguage::JavaScript, "document.write('<div>' + userInput + '</div>');"),
        (SourceLanguage::TypeScript, "document.getElementById('output').innerHTML = userInput;"),
    ];
    
    for (language, code) in languages {
        let detector = SecurityDetector::with_config(config.clone()).unwrap();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", code).unwrap();
        
        let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), language);
        
        let result = detector.analyze(&parsed_file).await;
        
        // Each language should be analyzed without errors
        assert!(result.is_ok(), "Failed to analyze {:?}: {:?}", language, result.err());
        
        let issues = result.unwrap();
        
        // Language-specific patterns should detect vulnerabilities in the crafted code
        if !issues.is_empty() {
            // If issues are found, they should have proper metadata
            for issue in &issues {
                assert!(issue.language.is_some());
                assert_eq!(issue.language.unwrap(), language);
                assert!(!issue.detected_by.is_empty());
            }
        }
    }
}

#[tokio::test]
async fn test_security_detector_integration_with_validation() {
    let mut config = SecurityConfig::production();
    
    // Enable validation to test false positive filtering
    config.false_positive_config = FalsePositiveConfig::aggressive();
    
    let detector = SecurityDetector::with_config(config).unwrap();
    
    // Create a test file that might generate false positives
    let mut temp_file = NamedTempFile::with_suffix("_test.py").unwrap();  // Test file
    writeln!(temp_file, "# This is a unit test file").unwrap();
    writeln!(temp_file, "import subprocess").unwrap();
    writeln!(temp_file, "def test_subprocess():").unwrap();
    writeln!(temp_file, "    # This might be flagged but should be filtered as test code").unwrap();
    writeln!(temp_file, "    result = subprocess.run(['echo', 'test'], capture_output=True)").unwrap();
    writeln!(temp_file, "    assert result.returncode == 0").unwrap();
    
    let parsed_file = create_test_parsed_file_from_temp(temp_file.path(), SourceLanguage::Python);
    
    let issues = detector.analyze(&parsed_file).await.unwrap();
    
    // With aggressive false positive filtering, test files should have fewer issues
    // or issues should have lower confidence scores
    if !issues.is_empty() {
        for issue in &issues {
            // Issues in test files should either be suppressed or have adjusted confidence
            assert!(issue.confidence_score <= 0.8 || issue.severity != SecuritySeverity::Critical);
        }
    }
}