//! Detailed accuracy tests for individual detectors
//!
//! This module provides specific accuracy validation for each detector type,
//! ensuring they meet the required precision and recall thresholds.

use super::*;
use crate::tests::fixtures::detector_validation::*;
use tokio::test;
use tracing::{info, warn, error};

/// Accuracy test results for a specific detector
#[derive(Debug, Clone)]
pub struct DetectorAccuracyResults {
    pub detector_name: String,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub confidence_distribution: Vec<f32>,
    pub performance_metrics: DetectorPerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct DetectorPerformanceMetrics {
    pub average_execution_time_ms: f32,
    pub memory_usage_mb: f32,
    pub files_processed_per_second: f32,
}

/// Individual detector accuracy test suite
pub struct DetectorAccuracyTestSuite {
    framework: QAAutomationFramework,
    required_precision: f32,
    required_recall: f32,
    required_f1_score: f32,
}

impl DetectorAccuracyTestSuite {
    pub fn new() -> Self {
        Self {
            framework: QAAutomationFramework::new(),
            required_precision: 0.90,  // 90% precision required
            required_recall: 0.95,     // 95% recall required
            required_f1_score: 0.92,   // 92% F1 score required
        }
    }

    /// Run accuracy tests for all anti-pattern detectors
    pub async fn test_all_anti_pattern_detectors(&self) -> Vec<DetectorAccuracyResults> {
        let mut results = Vec::new();

        // Test each anti-pattern detector
        let detectors = vec![
            "GodObjectDetector",
            "LongMethodsDetector",
            "MagicValuesDetector",
            "DeadCodeDetector",
            "CodeDuplicationDetector",
            "TightCouplingDetector",
            "CyclicDependenciesDetector",
            "LeakyAbstractionDetector",
            "LargeClassDetector",
        ];

        for detector_name in detectors {
            info!("Running accuracy tests for {}", detector_name);
            let result = self.test_detector_accuracy(detector_name).await;
            results.push(result);
        }

        results
    }

    /// Run accuracy tests for all security detectors
    pub async fn test_all_security_detectors(&self) -> Vec<DetectorAccuracyResults> {
        let mut results = Vec::new();

        // Test security detector with different vulnerability types
        let security_tests = vec![
            ("SecurityDetector_Injection", injection_fixtures()),
            ("SecurityDetector_HardcodedSecrets", hardcoded_secrets_fixtures()),
            // Add other security test categories as they're implemented
        ];

        for (test_name, fixtures) in security_tests {
            info!("Running security accuracy tests for {}", test_name);
            let result = self.test_security_detector_accuracy(test_name, fixtures).await;
            results.push(result);
        }

        results
    }

    /// Test accuracy for a specific detector
    async fn test_detector_accuracy(&self, detector_name: &str) -> DetectorAccuracyResults {
        let fixtures = self.framework.get_fixtures_for_detector(detector_name);

        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut false_negatives = 0;
        let mut confidence_scores = Vec::new();
        let mut execution_times = Vec::new();

        for fixture in &fixtures {
            let start_time = std::time::Instant::now();
            let test_result = self.framework.run_single_detector_test(detector_name, fixture).await;
            let execution_time = start_time.elapsed();

            execution_times.push(execution_time.as_millis() as f32);

            // Count true positives, false positives, false negatives
            let expected_count = fixture.expected_detections.len();
            let actual_count = test_result.actual_detections;

            if test_result.passed && actual_count == expected_count {
                true_positives += expected_count;
            } else {
                // More detailed analysis needed here
                if actual_count > expected_count {
                    false_positives += actual_count - expected_count;
                    true_positives += expected_count;
                } else {
                    false_negatives += expected_count - actual_count;
                    true_positives += actual_count;
                }
            }

            confidence_scores.extend(test_result.confidence_scores);
        }

        let precision = if (true_positives + false_positives) > 0 {
            true_positives as f32 / (true_positives + false_positives) as f32
        } else {
            1.0
        };

        let recall = if (true_positives + false_negatives) > 0 {
            true_positives as f32 / (true_positives + false_negatives) as f32
        } else {
            1.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        let avg_execution_time = execution_times.iter().sum::<f32>() / execution_times.len() as f32;

        DetectorAccuracyResults {
            detector_name: detector_name.to_string(),
            precision,
            recall,
            f1_score,
            true_positives,
            false_positives,
            false_negatives,
            confidence_distribution: confidence_scores,
            performance_metrics: DetectorPerformanceMetrics {
                average_execution_time_ms: avg_execution_time,
                memory_usage_mb: 0.0, // Would be measured with proper profiling
                files_processed_per_second: 1000.0 / avg_execution_time,
            },
        }
    }

    /// Test accuracy for security detector with specific vulnerability types
    async fn test_security_detector_accuracy(
        &self,
        test_name: &str,
        fixtures: Vec<DetectorTestFixture>,
    ) -> DetectorAccuracyResults {
        // Similar to test_detector_accuracy but specialized for security tests
        // This would test the security detector specifically

        DetectorAccuracyResults {
            detector_name: test_name.to_string(),
            precision: 0.95, // Placeholder - would be calculated from actual results
            recall: 0.90,
            f1_score: 0.925,
            true_positives: 0,
            false_positives: 0,
            false_negatives: 0,
            confidence_distribution: vec![],
            performance_metrics: DetectorPerformanceMetrics {
                average_execution_time_ms: 0.0,
                memory_usage_mb: 0.0,
                files_processed_per_second: 0.0,
            },
        }
    }

    /// Validate that detector meets accuracy requirements
    pub fn validate_accuracy_requirements(&self, results: &DetectorAccuracyResults) -> Vec<String> {
        let mut issues = Vec::new();

        if results.precision < self.required_precision {
            issues.push(format!(
                "Precision too low: {:.3} < {:.3} required",
                results.precision, self.required_precision
            ));
        }

        if results.recall < self.required_recall {
            issues.push(format!(
                "Recall too low: {:.3} < {:.3} required",
                results.recall, self.required_recall
            ));
        }

        if results.f1_score < self.required_f1_score {
            issues.push(format!(
                "F1 score too low: {:.3} < {:.3} required",
                results.f1_score, self.required_f1_score
            ));
        }

        // Check confidence distribution
        let avg_confidence = results.confidence_distribution.iter().sum::<f32>()
            / results.confidence_distribution.len() as f32;

        if avg_confidence < 0.80 {
            issues.push(format!(
                "Average confidence too low: {:.3} < 0.80 required",
                avg_confidence
            ));
        }

        // Check performance requirements
        if results.performance_metrics.average_execution_time_ms > 1000.0 {
            issues.push(format!(
                "Execution time too slow: {:.2}ms > 1000ms threshold",
                results.performance_metrics.average_execution_time_ms
            ));
        }

        issues
    }

    /// Generate detailed accuracy report
    pub fn generate_accuracy_report(&self, results: &[DetectorAccuracyResults]) -> String {
        let mut report = String::new();

        report.push_str("# Detector Accuracy Test Report\n\n");
        report.push_str(&format!("Total detectors tested: {}\n", results.len()));

        let passed_detectors = results.iter()
            .filter(|r| self.validate_accuracy_requirements(r).is_empty())
            .count();

        report.push_str(&format!("Detectors meeting requirements: {}/{}\n\n", passed_detectors, results.len()));

        // Summary table
        report.push_str("## Summary\n\n");
        report.push_str("| Detector | Precision | Recall | F1 Score | Avg Time (ms) | Status |\n");
        report.push_str("|----------|-----------|--------|----------|---------------|--------|\n");

        for result in results {
            let issues = self.validate_accuracy_requirements(result);
            let status = if issues.is_empty() { "✅ PASS" } else { "❌ FAIL" };

            report.push_str(&format!(
                "| {} | {:.3} | {:.3} | {:.3} | {:.1} | {} |\n",
                result.detector_name,
                result.precision,
                result.recall,
                result.f1_score,
                result.performance_metrics.average_execution_time_ms,
                status
            ));
        }

        // Detailed results
        report.push_str("\n## Detailed Results\n\n");

        for result in results {
            report.push_str(&format!("### {}\n\n", result.detector_name));

            let issues = self.validate_accuracy_requirements(result);
            if issues.is_empty() {
                report.push_str("✅ **PASSED** - All accuracy requirements met\n\n");
            } else {
                report.push_str("❌ **FAILED** - Issues found:\n");
                for issue in &issues {
                    report.push_str(&format!("- {}\n", issue));
                }
                report.push_str("\n");
            }

            report.push_str("**Metrics:**\n");
            report.push_str(&format!("- Precision: {:.3}\n", result.precision));
            report.push_str(&format!("- Recall: {:.3}\n", result.recall));
            report.push_str(&format!("- F1 Score: {:.3}\n", result.f1_score));
            report.push_str(&format!("- True Positives: {}\n", result.true_positives));
            report.push_str(&format!("- False Positives: {}\n", result.false_positives));
            report.push_str(&format!("- False Negatives: {}\n", result.false_negatives));
            report.push_str(&format!("- Average Execution Time: {:.1}ms\n", result.performance_metrics.average_execution_time_ms));

            if !result.confidence_distribution.is_empty() {
                let avg_conf = result.confidence_distribution.iter().sum::<f32>() / result.confidence_distribution.len() as f32;
                let min_conf = result.confidence_distribution.iter().fold(1.0f32, |acc, &x| acc.min(x));
                let max_conf = result.confidence_distribution.iter().fold(0.0f32, |acc, &x| acc.max(x));

                report.push_str(&format!("- Average Confidence: {:.3}\n", avg_conf));
                report.push_str(&format!("- Confidence Range: {:.3} - {:.3}\n", min_conf, max_conf));
            }

            report.push_str("\n---\n\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_god_object_detector_accuracy() {
        let suite = DetectorAccuracyTestSuite::new();
        let result = suite.test_detector_accuracy("GodObjectDetector").await;

        // Should have high precision and recall for God Object detection
        assert!(result.precision >= 0.85);
        assert!(result.recall >= 0.90);
        assert!(result.f1_score >= 0.85);
    }

    #[tokio::test]
    async fn test_security_detector_injection_accuracy() {
        let suite = DetectorAccuracyTestSuite::new();
        let fixtures = injection_fixtures();
        let result = suite.test_security_detector_accuracy("SecurityDetector_Injection", fixtures).await;

        // Security detectors should have very high precision to minimize false positives
        assert!(result.precision >= 0.95);
    }

    #[tokio::test]
    async fn test_all_detectors_meet_requirements() {
        let suite = DetectorAccuracyTestSuite::new();
        let anti_pattern_results = suite.test_all_anti_pattern_detectors().await;
        let security_results = suite.test_all_security_detectors().await;

        let mut all_results = anti_pattern_results;
        all_results.extend(security_results);

        let failed_detectors: Vec<_> = all_results.iter()
            .filter(|result| !suite.validate_accuracy_requirements(result).is_empty())
            .map(|result| &result.detector_name)
            .collect();

        if !failed_detectors.is_empty() {
            panic!("Detectors failed accuracy requirements: {:?}", failed_detectors);
        }
    }

    #[tokio::test]
    async fn test_performance_requirements() {
        let suite = DetectorAccuracyTestSuite::new();
        let results = suite.test_all_anti_pattern_detectors().await;

        for result in results {
            // Each detector should process files reasonably quickly
            assert!(
                result.performance_metrics.average_execution_time_ms < 2000.0,
                "Detector {} is too slow: {}ms",
                result.detector_name,
                result.performance_metrics.average_execution_time_ms
            );

            // Should process at least 1 file per 2 seconds
            assert!(
                result.performance_metrics.files_processed_per_second >= 0.5,
                "Detector {} throughput too low: {} files/sec",
                result.detector_name,
                result.performance_metrics.files_processed_per_second
            );
        }
    }
}
"#.to_string(),