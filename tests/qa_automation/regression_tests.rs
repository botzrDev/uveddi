//! Regression testing framework for Uveddi detectors
//!
//! This module ensures that detector accuracy doesn't regress over time
//! and that changes to the codebase don't break existing functionality.

use super::*;
use crate::tests::fixtures::detector_validation::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::test;
use tracing::{info, warn, error};

/// Baseline results for regression testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResults {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub detector_results: HashMap<String, DetectorBaseline>,
    pub overall_accuracy: f32,
}

/// Baseline metrics for a specific detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorBaseline {
    pub detector_name: String,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
    pub average_confidence: f32,
    pub execution_time_ms: f32,
    pub test_cases_passed: usize,
    pub total_test_cases: usize,
}

/// Regression test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionTestResult {
    pub detector_name: String,
    pub has_regression: bool,
    pub regression_type: RegressionType,
    pub baseline_metric: f32,
    pub current_metric: f32,
    pub change_percentage: f32,
    pub severity: RegressionSeverity,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionType {
    PrecisionDrop,
    RecallDrop,
    F1ScoreDrop,
    PerformanceDegradation,
    TestFailureIncrease,
    ConfidenceDrop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Critical,  // >10% degradation
    Major,     // 5-10% degradation
    Minor,     // 2-5% degradation
    Negligible, // <2% degradation
}

/// Comprehensive regression testing suite
pub struct RegressionTestSuite {
    baseline_path: String,
    accuracy_threshold: f32,     // Max allowed accuracy drop (5%)
    performance_threshold: f32,  // Max allowed performance degradation (20%)
    confidence_threshold: f32,   // Max allowed confidence drop (10%)
}

impl RegressionTestSuite {
    pub fn new() -> Self {
        Self {
            baseline_path: "tests/baselines/detector_baselines.json".to_string(),
            accuracy_threshold: 0.05,  // 5% max accuracy drop
            performance_threshold: 0.20, // 20% max performance degradation
            confidence_threshold: 0.10,  // 10% max confidence drop
        }
    }

    /// Run comprehensive regression test suite
    pub async fn run_regression_tests(&self) -> Vec<RegressionTestResult> {
        info!("Starting regression test suite");

        let baseline = match self.load_baseline().await {
            Ok(baseline) => baseline,
            Err(e) => {
                warn!("Failed to load baseline: {}. Creating new baseline.", e);
                return self.create_new_baseline().await;
            }
        };

        let current_results = self.run_current_tests().await;
        let regression_results = self.compare_against_baseline(&baseline, &current_results).await;

        // Save current results as new baseline if no major regressions
        let has_major_regressions = regression_results.iter()
            .any(|r| matches!(r.severity, RegressionSeverity::Critical | RegressionSeverity::Major));

        if !has_major_regressions {
            if let Err(e) = self.save_baseline(&current_results).await {
                warn!("Failed to save new baseline: {}", e);
            }
        }

        info!("Regression tests completed: {} results", regression_results.len());
        regression_results
    }

    /// Load baseline results from file
    async fn load_baseline(&self) -> Result<BaselineResults, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(&self.baseline_path).await?;
        let baseline: BaselineResults = serde_json::from_str(&content)?;
        Ok(baseline)
    }

    /// Save current results as new baseline
    async fn save_baseline(&self, current_results: &HashMap<String, DetectorBaseline>) -> Result<(), Box<dyn std::error::Error>> {
        let baseline = BaselineResults {
            timestamp: chrono::Utc::now(),
            version: self.get_current_version(),
            detector_results: current_results.clone(),
            overall_accuracy: self.calculate_overall_accuracy(current_results),
        };

        // Ensure directory exists
        if let Some(parent) = Path::new(&self.baseline_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(&baseline)?;
        tokio::fs::write(&self.baseline_path, content).await?;

        info!("Saved new baseline with {} detector results", baseline.detector_results.len());
        Ok(())
    }

    /// Run current detector tests to get latest metrics
    async fn run_current_tests(&self) -> HashMap<String, DetectorBaseline> {
        let accuracy_suite = DetectorAccuracyTestSuite::new();

        let mut current_results = HashMap::new();

        // Test anti-pattern detectors
        let anti_pattern_results = accuracy_suite.test_all_anti_pattern_detectors().await;
        for result in anti_pattern_results {
            let baseline = DetectorBaseline {
                detector_name: result.detector_name.clone(),
                precision: result.precision,
                recall: result.recall,
                f1_score: result.f1_score,
                average_confidence: result.confidence_distribution.iter().sum::<f32>()
                    / result.confidence_distribution.len() as f32,
                execution_time_ms: result.performance_metrics.average_execution_time_ms,
                test_cases_passed: result.true_positives,
                total_test_cases: result.true_positives + result.false_negatives,
            };
            current_results.insert(result.detector_name, baseline);
        }

        // Test security detectors
        let security_results = accuracy_suite.test_all_security_detectors().await;
        for result in security_results {
            let baseline = DetectorBaseline {
                detector_name: result.detector_name.clone(),
                precision: result.precision,
                recall: result.recall,
                f1_score: result.f1_score,
                average_confidence: result.confidence_distribution.iter().sum::<f32>()
                    / result.confidence_distribution.len() as f32,
                execution_time_ms: result.performance_metrics.average_execution_time_ms,
                test_cases_passed: result.true_positives,
                total_test_cases: result.true_positives + result.false_negatives,
            };
            current_results.insert(result.detector_name, baseline);
        }

        current_results
    }

    /// Compare current results against baseline to detect regressions
    async fn compare_against_baseline(
        &self,
        baseline: &BaselineResults,
        current: &HashMap<String, DetectorBaseline>,
    ) -> Vec<RegressionTestResult> {
        let mut regression_results = Vec::new();

        for (detector_name, current_result) in current {
            if let Some(baseline_result) = baseline.detector_results.get(detector_name) {
                // Check for precision regression
                let precision_change = (current_result.precision - baseline_result.precision) / baseline_result.precision;
                if precision_change < -self.accuracy_threshold {
                    regression_results.push(RegressionTestResult {
                        detector_name: detector_name.clone(),
                        has_regression: true,
                        regression_type: RegressionType::PrecisionDrop,
                        baseline_metric: baseline_result.precision,
                        current_metric: current_result.precision,
                        change_percentage: precision_change * 100.0,
                        severity: self.classify_severity(precision_change.abs()),
                        details: format!(
                            "Precision dropped from {:.3} to {:.3} ({:.1}% decrease)",
                            baseline_result.precision, current_result.precision, precision_change.abs() * 100.0
                        ),
                    });
                }

                // Check for recall regression
                let recall_change = (current_result.recall - baseline_result.recall) / baseline_result.recall;
                if recall_change < -self.accuracy_threshold {
                    regression_results.push(RegressionTestResult {
                        detector_name: detector_name.clone(),
                        has_regression: true,
                        regression_type: RegressionType::RecallDrop,
                        baseline_metric: baseline_result.recall,
                        current_metric: current_result.recall,
                        change_percentage: recall_change * 100.0,
                        severity: self.classify_severity(recall_change.abs()),
                        details: format!(
                            "Recall dropped from {:.3} to {:.3} ({:.1}% decrease)",
                            baseline_result.recall, current_result.recall, recall_change.abs() * 100.0
                        ),
                    });
                }

                // Check for F1 score regression
                let f1_change = (current_result.f1_score - baseline_result.f1_score) / baseline_result.f1_score;
                if f1_change < -self.accuracy_threshold {
                    regression_results.push(RegressionTestResult {
                        detector_name: detector_name.clone(),
                        has_regression: true,
                        regression_type: RegressionType::F1ScoreDrop,
                        baseline_metric: baseline_result.f1_score,
                        current_metric: current_result.f1_score,
                        change_percentage: f1_change * 100.0,
                        severity: self.classify_severity(f1_change.abs()),
                        details: format!(
                            "F1 score dropped from {:.3} to {:.3} ({:.1}% decrease)",
                            baseline_result.f1_score, current_result.f1_score, f1_change.abs() * 100.0
                        ),
                    });
                }

                // Check for performance regression
                let perf_change = (current_result.execution_time_ms - baseline_result.execution_time_ms) / baseline_result.execution_time_ms;
                if perf_change > self.performance_threshold {
                    regression_results.push(RegressionTestResult {
                        detector_name: detector_name.clone(),
                        has_regression: true,
                        regression_type: RegressionType::PerformanceDegradation,
                        baseline_metric: baseline_result.execution_time_ms,
                        current_metric: current_result.execution_time_ms,
                        change_percentage: perf_change * 100.0,
                        severity: self.classify_severity(perf_change),
                        details: format!(
                            "Execution time increased from {:.1}ms to {:.1}ms ({:.1}% slower)",
                            baseline_result.execution_time_ms, current_result.execution_time_ms, perf_change * 100.0
                        ),
                    });
                }

                // Check for confidence regression
                let conf_change = (current_result.average_confidence - baseline_result.average_confidence) / baseline_result.average_confidence;
                if conf_change < -self.confidence_threshold {
                    regression_results.push(RegressionTestResult {
                        detector_name: detector_name.clone(),
                        has_regression: true,
                        regression_type: RegressionType::ConfidenceDrop,
                        baseline_metric: baseline_result.average_confidence,
                        current_metric: current_result.average_confidence,
                        change_percentage: conf_change * 100.0,
                        severity: self.classify_severity(conf_change.abs()),
                        details: format!(
                            "Average confidence dropped from {:.3} to {:.3} ({:.1}% decrease)",
                            baseline_result.average_confidence, current_result.average_confidence, conf_change.abs() * 100.0
                        ),
                    });
                }
            } else {
                warn!("New detector found (not in baseline): {}", detector_name);
            }
        }

        regression_results
    }

    /// Create new baseline when none exists
    async fn create_new_baseline(&self) -> Vec<RegressionTestResult> {
        info!("Creating new baseline results");

        let current_results = self.run_current_tests().await;

        if let Err(e) = self.save_baseline(&current_results).await {
            error!("Failed to save new baseline: {}", e);
        }

        // Return empty regression results since this is the first run
        Vec::new()
    }

    /// Classify regression severity based on percentage change
    fn classify_severity(&self, change_percentage: f32) -> RegressionSeverity {
        if change_percentage >= 0.10 { // 10%+
            RegressionSeverity::Critical
        } else if change_percentage >= 0.05 { // 5-10%
            RegressionSeverity::Major
        } else if change_percentage >= 0.02 { // 2-5%
            RegressionSeverity::Minor
        } else {
            RegressionSeverity::Negligible
        }
    }

    /// Get current version from environment or default
    fn get_current_version(&self) -> String {
        std::env::var("UVEDDI_VERSION").unwrap_or_else(|_| "0.9.0-alpha".to_string())
    }

    /// Calculate overall accuracy across all detectors
    fn calculate_overall_accuracy(&self, results: &HashMap<String, DetectorBaseline>) -> f32 {
        if results.is_empty() {
            return 0.0;
        }

        let total_f1: f32 = results.values().map(|r| r.f1_score).sum();
        total_f1 / results.len() as f32
    }

    /// Generate comprehensive regression test report
    pub fn generate_regression_report(&self, results: &[RegressionTestResult]) -> String {
        let mut report = String::new();

        report.push_str("# Detector Regression Test Report\n\n");
        report.push_str(&format!("Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("Version: {}\n\n", self.get_current_version()));

        let critical_regressions = results.iter().filter(|r| matches!(r.severity, RegressionSeverity::Critical)).count();
        let major_regressions = results.iter().filter(|r| matches!(r.severity, RegressionSeverity::Major)).count();
        let minor_regressions = results.iter().filter(|r| matches!(r.severity, RegressionSeverity::Minor)).count();

        report.push_str("## Summary\n\n");
        report.push_str(&format!("- Total regressions detected: {}\n", results.len()));
        report.push_str(&format!("- Critical regressions: {} 🚨\n", critical_regressions));
        report.push_str(&format!("- Major regressions: {} ⚠️\n", major_regressions));
        report.push_str(&format!("- Minor regressions: {} ℹ️\n\n", minor_regressions));

        if results.is_empty() {
            report.push_str("✅ **No regressions detected!** All detectors are performing at or above baseline levels.\n\n");
            return report;
        }

        // Group by severity
        report.push_str("## Regressions by Severity\n\n");

        for severity in [RegressionSeverity::Critical, RegressionSeverity::Major, RegressionSeverity::Minor] {
            let severity_results: Vec<_> = results.iter().filter(|r| matches!(r.severity, severity)).collect();

            if severity_results.is_empty() {
                continue;
            }

            let severity_name = match severity {
                RegressionSeverity::Critical => "🚨 Critical",
                RegressionSeverity::Major => "⚠️ Major",
                RegressionSeverity::Minor => "ℹ️ Minor",
                RegressionSeverity::Negligible => "Negligible",
            };

            report.push_str(&format!("### {}\n\n", severity_name));

            for result in severity_results {
                report.push_str(&format!("**{}**\n", result.detector_name));
                report.push_str(&format!("- Type: {:?}\n", result.regression_type));
                report.push_str(&format!("- Change: {:.1}%\n", result.change_percentage));
                report.push_str(&format!("- Baseline: {:.3}\n", result.baseline_metric));
                report.push_str(&format!("- Current: {:.3}\n", result.current_metric));
                report.push_str(&format!("- Details: {}\n\n", result.details));
            }
        }

        report.push_str("## Recommendations\n\n");

        if critical_regressions > 0 {
            report.push_str("🚨 **Critical regressions detected!** Consider:\n");
            report.push_str("- Rolling back recent changes\n");
            report.push_str("- Investigating detector implementation changes\n");
            report.push_str("- Reviewing training data or model updates\n\n");
        }

        if major_regressions > 0 {
            report.push_str("⚠️ **Major regressions detected!** Consider:\n");
            report.push_str("- Reviewing recent algorithm changes\n");
            report.push_str("- Checking for test fixture updates\n");
            report.push_str("- Validating configuration changes\n\n");
        }

        if minor_regressions > 0 {
            report.push_str("ℹ️ **Minor regressions detected.** These may be acceptable but should be monitored.\n\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regression_suite_creation() {
        let suite = RegressionTestSuite::new();
        assert!(suite.accuracy_threshold > 0.0);
        assert!(suite.performance_threshold > 0.0);
    }

    #[tokio::test]
    async fn test_severity_classification() {
        let suite = RegressionTestSuite::new();

        assert!(matches!(suite.classify_severity(0.15), RegressionSeverity::Critical));
        assert!(matches!(suite.classify_severity(0.07), RegressionSeverity::Major));
        assert!(matches!(suite.classify_severity(0.03), RegressionSeverity::Minor));
        assert!(matches!(suite.classify_severity(0.01), RegressionSeverity::Negligible));
    }

    #[tokio::test]
    async fn test_baseline_creation() {
        let suite = RegressionTestSuite::new();
        let results = suite.create_new_baseline().await;

        // First run should have no regressions
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_overall_accuracy_calculation() {
        let suite = RegressionTestSuite::new();
        let mut results = HashMap::new();

        results.insert("Detector1".to_string(), DetectorBaseline {
            detector_name: "Detector1".to_string(),
            precision: 0.9,
            recall: 0.9,
            f1_score: 0.9,
            average_confidence: 0.85,
            execution_time_ms: 100.0,
            test_cases_passed: 9,
            total_test_cases: 10,
        });

        results.insert("Detector2".to_string(), DetectorBaseline {
            detector_name: "Detector2".to_string(),
            precision: 0.8,
            recall: 0.8,
            f1_score: 0.8,
            average_confidence: 0.75,
            execution_time_ms: 150.0,
            test_cases_passed: 8,
            total_test_cases: 10,
        });

        let overall_accuracy = suite.calculate_overall_accuracy(&results);
        assert_eq!(overall_accuracy, 0.85); // (0.9 + 0.8) / 2
    }
}
"#.to_string(),