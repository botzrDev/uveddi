use super::*;
use crate::analysis::detectors::security::config::FalsePositiveConfig;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
async fn validation_engine_filters_and_scores() {
    let engine = ValidationEngine::new(FalsePositiveConfig::moderate()).unwrap();

    let issue = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Test issue".to_string(),
        SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
    )
    .with_confidence(0.8);

    let validated = engine.validate_issues(vec![issue]).await.unwrap();
    assert!(!validated.is_empty());
    assert!(validated[0].confidence_score >= 0.6);
}

#[tokio::test]
async fn false_positive_mitigator_preserves_enabled_stages() {
    let config = ValidationConfig::from_false_positive(FalsePositiveConfig::aggressive());
    let mitigator = FalsePositiveMitigator::new(config.clone()).unwrap();
    assert!(mitigator.config().enable_input_validation);

    let issue = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Test issue".to_string(),
        SecurityLocation::new(PathBuf::from("src/main.rs"), 1, 20),
    );

    let filtered = mitigator.filter_issues(vec![issue]).await.unwrap();
    assert!(filtered.len() <= 1);
}

#[tokio::test]
async fn confidence_calculator_respects_severity() {
    let calculator = ConfidenceCalculator::new();

    let issue = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Test issue".to_string(),
        SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
    )
    .with_severity(SecuritySeverity::Critical);

    let confidence = calculator.calculate_confidence(&issue).await.unwrap();
    assert!(confidence.final_score > 0.0);
    assert!(confidence.final_score <= 1.0);
}

#[tokio::test]
async fn cross_validation_combines_matching_findings() {
    let engine = ValidationEngine::new(FalsePositiveConfig::moderate()).unwrap();

    let issue1 = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Test issue".to_string(),
        SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
    )
    .with_detector("Detector1".to_string());

    let issue2 = SecurityIssue::new(
        SecurityIssueType::Injection,
        VulnerabilityType::Static,
        "SQL Injection".to_string(),
        "Test issue".to_string(),
        SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
    )
    .with_detector("Detector2".to_string());

    let mut results = HashMap::new();
    results.insert("Detector1".to_string(), vec![issue1]);
    results.insert("Detector2".to_string(), vec![issue2]);

    let validated = engine.cross_validate(results).await.unwrap();
    assert_eq!(validated.len(), 1);
    assert_eq!(validated[0].detected_by.len(), 2);
}
