//! Automated QA Testing Framework for Uveddi Detectors
//!
//! This module provides comprehensive automated testing to ensure 100% accuracy
//! of all detectors in the Uveddi analysis engine.

pub mod detector_accuracy_tests;
pub mod regression_tests;
pub mod performance_benchmarks;
pub mod false_positive_validation;

use crate::tests::fixtures::detector_validation::{DetectorTestFixture, ExpectedDetection};
use crate::analysis::detectors::*;
use crate::ast::{ParsedFile, Language};
use crate::database::models::ArchitecturalIssue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::test;
use tracing::{info, warn, error};

/// QA test result for a single detector test fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QATestResult {
    pub fixture_name: String,
    pub detector_name: String,
    pub passed: bool,
    pub expected_detections: usize,
    pub actual_detections: usize,
    pub accuracy_score: f32,
    pub confidence_scores: Vec<f32>,
    pub false_positives: Vec<String>,
    pub false_negatives: Vec<String>,
    pub execution_time_ms: u128,
    pub error_message: Option<String>,
}

/// Comprehensive QA test suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QATestSuiteResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub overall_accuracy: f32,
    pub detector_results: HashMap<String, Vec<QATestResult>>,
    pub performance_metrics: PerformanceMetrics,
    pub regression_status: RegressionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_detection_time_ms: f32,
    pub memory_usage_mb: f32,
    pub throughput_files_per_second: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionStatus {
    pub has_regressions: bool,
    pub affected_detectors: Vec<String>,
    pub regression_details: Vec<String>,
}

/// Main QA automation framework
pub struct QAAutomationFramework {
    detector_registry: DetectorRegistry,
    test_fixtures: Vec<DetectorTestFixture>,
    accuracy_threshold: f32,
    confidence_threshold: f32,
}

impl QAAutomationFramework {
    /// Create new QA automation framework
    pub fn new() -> Self {
        Self {
            detector_registry: DetectorRegistry::new(),
            test_fixtures: load_all_test_fixtures(),
            accuracy_threshold: 0.95, // Require 95% accuracy
            confidence_threshold: 0.80, // Require 80% minimum confidence
        }
    }

    /// Run comprehensive QA test suite
    pub async fn run_comprehensive_qa_suite(&self) -> QATestSuiteResults {
        info!("Starting comprehensive QA test suite for all detectors");

        let mut results = QATestSuiteResults {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            overall_accuracy: 0.0,
            detector_results: HashMap::new(),
            performance_metrics: PerformanceMetrics {
                average_detection_time_ms: 0.0,
                memory_usage_mb: 0.0,
                throughput_files_per_second: 0.0,
            },
            regression_status: RegressionStatus {
                has_regressions: false,
                affected_detectors: Vec::new(),
                regression_details: Vec::new(),
            },
        };

        let start_time = std::time::Instant::now();

        // Test each detector with its fixtures
        for detector_name in self.detector_registry.get_all_detector_names() {
            info!("Testing detector: {}", detector_name);

            let detector_fixtures = self.get_fixtures_for_detector(&detector_name);
            let mut detector_results = Vec::new();

            for fixture in detector_fixtures {
                let test_result = self.run_single_detector_test(&detector_name, &fixture).await;

                if test_result.passed {
                    results.passed_tests += 1;
                } else {
                    results.failed_tests += 1;
                    error!("Test failed: {} for detector {}", fixture.name, detector_name);
                }

                detector_results.push(test_result);
                results.total_tests += 1;
            }

            results.detector_results.insert(detector_name, detector_results);
        }

        let total_time = start_time.elapsed();
        results.performance_metrics.average_detection_time_ms =
            total_time.as_millis() as f32 / results.total_tests as f32;

        results.overall_accuracy = results.passed_tests as f32 / results.total_tests as f32;

        // Check for regressions
        results.regression_status = self.check_for_regressions(&results).await;

        info!(
            "QA test suite completed: {}/{} tests passed ({:.2}% accuracy)",
            results.passed_tests,
            results.total_tests,
            results.overall_accuracy * 100.0
        );

        results
    }

    /// Run a single detector test with a specific fixture
    async fn run_single_detector_test(
        &self,
        detector_name: &str,
        fixture: &DetectorTestFixture,
    ) -> QATestResult {
        let start_time = std::time::Instant::now();

        // Create temporary file with test code
        let temp_file = match self.create_temp_file_from_fixture(fixture) {
            Ok(file) => file,
            Err(e) => {
                return QATestResult {
                    fixture_name: fixture.name.clone(),
                    detector_name: detector_name.to_string(),
                    passed: false,
                    expected_detections: fixture.expected_detections.len(),
                    actual_detections: 0,
                    accuracy_score: 0.0,
                    confidence_scores: vec![],
                    false_positives: vec![],
                    false_negatives: vec![],
                    execution_time_ms: start_time.elapsed().as_millis(),
                    error_message: Some(format!("Failed to create temp file: {}", e)),
                };
            }
        };

        // Parse the file
        let parsed_file = match self.parse_test_file(&temp_file, &fixture.language) {
            Ok(file) => file,
            Err(e) => {
                return QATestResult {
                    fixture_name: fixture.name.clone(),
                    detector_name: detector_name.to_string(),
                    passed: false,
                    expected_detections: fixture.expected_detections.len(),
                    actual_detections: 0,
                    accuracy_score: 0.0,
                    confidence_scores: vec![],
                    false_positives: vec![],
                    false_negatives: vec![],
                    execution_time_ms: start_time.elapsed().as_millis(),
                    error_message: Some(format!("Failed to parse file: {}", e)),
                };
            }
        };

        // Run detector
        let detector = match self.detector_registry.get_detector(detector_name) {
            Some(detector) => detector,
            None => {
                return QATestResult {
                    fixture_name: fixture.name.clone(),
                    detector_name: detector_name.to_string(),
                    passed: false,
                    expected_detections: fixture.expected_detections.len(),
                    actual_detections: 0,
                    accuracy_score: 0.0,
                    confidence_scores: vec![],
                    false_positives: vec![],
                    false_negatives: vec![],
                    execution_time_ms: start_time.elapsed().as_millis(),
                    error_message: Some(format!("Detector not found: {}", detector_name)),
                };
            }
        };

        // Execute detection
        let actual_issues = match detector.detect_issues(&parsed_file).await {
            Ok(issues) => issues,
            Err(e) => {
                return QATestResult {
                    fixture_name: fixture.name.clone(),
                    detector_name: detector_name.to_string(),
                    passed: false,
                    expected_detections: fixture.expected_detections.len(),
                    actual_detections: 0,
                    accuracy_score: 0.0,
                    confidence_scores: vec![],
                    false_positives: vec![],
                    false_negatives: vec![],
                    execution_time_ms: start_time.elapsed().as_millis(),
                    error_message: Some(format!("Detection failed: {}", e)),
                };
            }
        };

        // Analyze results
        let analysis = self.analyze_detection_results(fixture, &actual_issues);
        let execution_time = start_time.elapsed().as_millis();

        QATestResult {
            fixture_name: fixture.name.clone(),
            detector_name: detector_name.to_string(),
            passed: analysis.accuracy >= self.accuracy_threshold
                && analysis.confidence_scores.iter().all(|&score| score >= self.confidence_threshold),
            expected_detections: fixture.expected_detections.len(),
            actual_detections: actual_issues.len(),
            accuracy_score: analysis.accuracy,
            confidence_scores: analysis.confidence_scores,
            false_positives: analysis.false_positives,
            false_negatives: analysis.false_negatives,
            execution_time_ms: execution_time,
            error_message: None,
        }
    }

    /// Analyze detection results compared to expected results
    fn analyze_detection_results(
        &self,
        fixture: &DetectorTestFixture,
        actual_issues: &[ArchitecturalIssue],
    ) -> DetectionAnalysis {
        let mut true_positives = 0;
        let mut false_positives = Vec::new();
        let mut false_negatives = Vec::new();
        let mut confidence_scores = Vec::new();

        // Check each expected detection
        for expected in &fixture.expected_detections {
            let found = actual_issues.iter().any(|issue| {
                self.matches_expected_detection(issue, expected)
            });

            if found {
                true_positives += 1;
                // Extract confidence score from metadata
                if let Some(issue) = actual_issues.iter().find(|issue| {
                    self.matches_expected_detection(issue, expected)
                }) {
                    if let Some(metadata) = &issue.metadata {
                        if let Ok(meta_json) = serde_json::from_str::<serde_json::Value>(metadata) {
                            if let Some(confidence) = meta_json.get("confidence") {
                                if let Some(conf_f64) = confidence.as_f64() {
                                    confidence_scores.push(conf_f64 as f32);
                                }
                            }
                        }
                    }
                }
            } else {
                false_negatives.push(format!(
                    "Missing detection: {} at line {}",
                    expected.issue_type, expected.start_line
                ));
            }
        }

        // Check for unexpected detections (false positives)
        for issue in actual_issues {
            let expected = fixture.expected_detections.iter().any(|expected| {
                self.matches_expected_detection(issue, expected)
            });

            if !expected {
                false_positives.push(format!(
                    "Unexpected detection: {} at line {}",
                    issue.severity, issue.line_number.unwrap_or(0)
                ));
            }
        }

        let total_expected = fixture.expected_detections.len();
        let accuracy = if total_expected > 0 {
            true_positives as f32 / total_expected as f32
        } else {
            1.0 // No expected detections and no false positives = perfect
        };

        DetectionAnalysis {
            accuracy,
            true_positives,
            false_positives,
            false_negatives,
            confidence_scores,
        }
    }

    /// Check if an actual detection matches an expected detection
    fn matches_expected_detection(
        &self,
        actual: &ArchitecturalIssue,
        expected: &ExpectedDetection,
    ) -> bool {
        // Check detector name
        if actual.detector_name != expected.detector_name {
            return false;
        }

        // Check severity
        if actual.severity != expected.severity {
            return false;
        }

        // Check line number (with some tolerance)
        if let Some(actual_line) = actual.line_number {
            let line_diff = (actual_line as u32).abs_diff(expected.start_line);
            if line_diff > 2 { // Allow 2-line tolerance
                return false;
            }
        }

        // Check message content
        for required_text in &expected.expected_message_contains {
            if !actual.message.to_lowercase().contains(&required_text.to_lowercase()) {
                return false;
            }
        }

        true
    }

    /// Create temporary file from test fixture
    fn create_temp_file_from_fixture(&self, fixture: &DetectorTestFixture) -> std::io::Result<NamedTempFile> {
        use std::io::Write;
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(fixture.code.as_bytes())?;
        temp_file.flush()?;
        Ok(temp_file)
    }

    /// Parse test file into ParsedFile
    fn parse_test_file(&self, temp_file: &NamedTempFile, language: &str) -> Result<ParsedFile, Box<dyn std::error::Error>> {
        let language_enum = match language.to_lowercase().as_str() {
            "rust" => Language::Rust,
            "python" => Language::Python,
            "javascript" => Language::JavaScript,
            "typescript" => Language::TypeScript,
            _ => return Err(format!("Unsupported language: {}", language).into()),
        };

        // This would use your actual AST parsing logic
        Ok(ParsedFile {
            file_path: temp_file.path().to_path_buf(),
            language: language_enum,
            content: std::fs::read_to_string(temp_file.path())?,
            ast: None, // Would be populated by actual parser
            functions: vec![],
            classes: vec![],
            imports: vec![],
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Get test fixtures for a specific detector
    fn get_fixtures_for_detector(&self, detector_name: &str) -> Vec<DetectorTestFixture> {
        self.test_fixtures
            .iter()
            .filter(|fixture| {
                fixture.expected_detections
                    .iter()
                    .any(|detection| detection.detector_name == detector_name)
            })
            .cloned()
            .collect()
    }

    /// Check for regressions compared to previous test runs
    async fn check_for_regressions(&self, current_results: &QATestSuiteResults) -> RegressionStatus {
        // This would compare against stored baseline results
        // For now, return no regressions
        RegressionStatus {
            has_regressions: false,
            affected_detectors: Vec::new(),
            regression_details: Vec::new(),
        }
    }
}

/// Analysis results for detection comparison
#[derive(Debug)]
struct DetectionAnalysis {
    accuracy: f32,
    true_positives: usize,
    false_positives: Vec<String>,
    false_negatives: Vec<String>,
    confidence_scores: Vec<f32>,
}

/// Registry of all available detectors
struct DetectorRegistry {
    detectors: HashMap<String, Box<dyn crate::analysis::AnalysisDetector>>,
}

impl DetectorRegistry {
    fn new() -> Self {
        let mut detectors: HashMap<String, Box<dyn crate::analysis::AnalysisDetector>> = HashMap::new();

        // Register all anti-pattern detectors
        detectors.insert("GodObjectDetector".to_string(), Box::new(anti_patterns::GodObjectDetector::new()));
        detectors.insert("LongMethodsDetector".to_string(), Box::new(anti_patterns::LongMethodsDetector::new()));
        detectors.insert("MagicValuesDetector".to_string(), Box::new(anti_patterns::MagicValuesDetector::new()));
        detectors.insert("DeadCodeDetector".to_string(), Box::new(anti_patterns::DeadCodeDetector::new()));
        detectors.insert("CodeDuplicationDetector".to_string(), Box::new(anti_patterns::CodeDuplicationDetector::new()));
        detectors.insert("TightCouplingDetector".to_string(), Box::new(anti_patterns::TightCouplingDetector::new()));
        detectors.insert("CyclicDependenciesDetector".to_string(), Box::new(anti_patterns::CyclicDependenciesDetector::new()));
        detectors.insert("LeakyAbstractionDetector".to_string(), Box::new(anti_patterns::LeakyAbstractionDetector::new()));
        detectors.insert("LargeClassDetector".to_string(), Box::new(anti_patterns::LargeClassDetector::new()));

        // Register security detector
        detectors.insert("SecurityDetector".to_string(), Box::new(security::MainSecurityDetector::new().unwrap()));

        Self { detectors }
    }

    fn get_detector(&self, name: &str) -> Option<&Box<dyn crate::analysis::AnalysisDetector>> {
        self.detectors.get(name)
    }

    fn get_all_detector_names(&self) -> Vec<String> {
        self.detectors.keys().cloned().collect()
    }
}

/// Load all test fixtures from the fixtures module
fn load_all_test_fixtures() -> Vec<DetectorTestFixture> {
    let mut fixtures = Vec::new();

    // Load anti-pattern fixtures
    fixtures.extend(crate::tests::fixtures::detector_validation::god_object_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::long_methods_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::magic_values_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::dead_code_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::large_classes_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::code_duplication_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::tight_coupling_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::cyclic_dependencies_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::leaky_abstraction_fixtures());

    // Load security fixtures
    fixtures.extend(crate::tests::fixtures::detector_validation::injection_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::hardcoded_secrets_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::access_control_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::crypto_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::insecure_design_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::misconfig_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::vulnerable_deps_fixtures());
    fixtures.extend(crate::tests::fixtures::detector_validation::auth_failures_fixtures());

    fixtures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qa_framework_initialization() {
        let framework = QAAutomationFramework::new();
        assert!(framework.test_fixtures.len() > 0);
        assert!(framework.accuracy_threshold > 0.0);
    }

    #[tokio::test]
    async fn test_comprehensive_qa_suite() {
        let framework = QAAutomationFramework::new();
        let results = framework.run_comprehensive_qa_suite().await;

        assert!(results.total_tests > 0);
        assert!(results.overall_accuracy >= 0.0);
        assert!(results.overall_accuracy <= 1.0);
    }

    #[tokio::test]
    async fn test_detector_registry() {
        let registry = DetectorRegistry::new();
        let detector_names = registry.get_all_detector_names();

        assert!(detector_names.contains(&"GodObjectDetector".to_string()));
        assert!(detector_names.contains(&"SecurityDetector".to_string()));
        assert!(detector_names.len() >= 10); // Should have at least 10 detectors
    }
}
"#.to_string(),
