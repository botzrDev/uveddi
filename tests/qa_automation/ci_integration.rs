//! CI/CD integration for automated QA validation
//!
//! This module provides integration with continuous integration systems
//! to automatically run QA tests on every code change.

use super::*;
use crate::tests::qa_automation::{
    detector_accuracy_tests::DetectorAccuracyTestSuite,
    regression_tests::RegressionTestSuite,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::test;
use tracing::{info, warn, error};

/// CI/CD integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIIntegrationConfig {
    pub run_on_pr: bool,
    pub run_on_main_branch: bool,
    pub fail_on_regressions: bool,
    pub fail_on_accuracy_drop: bool,
    pub accuracy_threshold: f32,
    pub performance_threshold_ms: f32,
    pub confidence_threshold: f32,
    pub generate_reports: bool,
    pub report_output_path: String,
}

impl Default for CIIntegrationConfig {
    fn default() -> Self {
        Self {
            run_on_pr: true,
            run_on_main_branch: true,
            fail_on_regressions: true,
            fail_on_accuracy_drop: true,
            accuracy_threshold: 0.90,
            performance_threshold_ms: 2000.0,
            confidence_threshold: 0.80,
            generate_reports: true,
            report_output_path: "target/qa-reports".to_string(),
        }
    }
}

/// CI test execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CITestResults {
    pub overall_status: CIStatus,
    pub accuracy_results: Vec<crate::tests::qa_automation::detector_accuracy_tests::DetectorAccuracyResults>,
    pub regression_results: Vec<crate::tests::qa_automation::regression_tests::RegressionTestResult>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub summary: CITestSummary,
    pub artifacts: Vec<CIArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CIStatus {
    Success,
    WarningsOnly,
    Failed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub detector_name: String,
    pub issue_type: String,
    pub threshold: f32,
    pub actual_value: f32,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CITestSummary {
    pub total_detectors_tested: usize,
    pub detectors_passed: usize,
    pub detectors_failed: usize,
    pub regressions_found: usize,
    pub critical_issues: usize,
    pub warnings: usize,
    pub execution_time_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIArtifact {
    pub name: String,
    pub path: String,
    pub artifact_type: ArtifactType,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    AccuracyReport,
    RegressionReport,
    PerformanceReport,
    TestResults,
    Coverage,
}

/// Main CI integration orchestrator
pub struct CIIntegration {
    config: CIIntegrationConfig,
    accuracy_suite: DetectorAccuracyTestSuite,
    regression_suite: RegressionTestSuite,
}

impl CIIntegration {
    pub fn new() -> Self {
        Self::with_config(CIIntegrationConfig::default())
    }

    pub fn with_config(config: CIIntegrationConfig) -> Self {
        Self {
            config,
            accuracy_suite: DetectorAccuracyTestSuite::new(),
            regression_suite: RegressionTestSuite::new(),
        }
    }

    /// Main CI test execution entry point
    pub async fn run_ci_tests(&self) -> CITestResults {
        info!("Starting CI QA test execution");
        let start_time = std::time::Instant::now();

        let mut results = CITestResults {
            overall_status: CIStatus::Success,
            accuracy_results: Vec::new(),
            regression_results: Vec::new(),
            performance_issues: Vec::new(),
            summary: CITestSummary {
                total_detectors_tested: 0,
                detectors_passed: 0,
                detectors_failed: 0,
                regressions_found: 0,
                critical_issues: 0,
                warnings: 0,
                execution_time_seconds: 0.0,
            },
            artifacts: Vec::new(),
        };

        // Run accuracy tests
        info!("Running detector accuracy tests");
        let mut accuracy_results = self.accuracy_suite.test_all_anti_pattern_detectors().await;
        accuracy_results.extend(self.accuracy_suite.test_all_security_detectors().await);

        // Analyze accuracy results
        let (passed_detectors, failed_detectors, performance_issues) =
            self.analyze_accuracy_results(&accuracy_results);

        results.accuracy_results = accuracy_results;
        results.performance_issues = performance_issues;
        results.summary.total_detectors_tested = passed_detectors + failed_detectors;
        results.summary.detectors_passed = passed_detectors;
        results.summary.detectors_failed = failed_detectors;

        // Run regression tests
        info!("Running regression tests");
        let regression_results = self.regression_suite.run_regression_tests().await;
        let critical_regressions = regression_results.iter()
            .filter(|r| matches!(r.severity, crate::tests::qa_automation::regression_tests::RegressionSeverity::Critical))
            .count();

        results.regression_results = regression_results.clone();
        results.summary.regressions_found = regression_results.len();
        results.summary.critical_issues = critical_regressions;

        // Determine overall status
        results.overall_status = self.determine_ci_status(&results);

        // Generate reports and artifacts
        if self.config.generate_reports {
            results.artifacts = self.generate_ci_artifacts(&results).await;
        }

        results.summary.execution_time_seconds = start_time.elapsed().as_secs_f32();

        info!(
            "CI tests completed: {} status, {}/{} detectors passed, {} regressions",
            format!("{:?}", results.overall_status),
            results.summary.detectors_passed,
            results.summary.total_detectors_tested,
            results.summary.regressions_found
        );

        results
    }

    /// Analyze accuracy results and classify issues
    fn analyze_accuracy_results(
        &self,
        accuracy_results: &[crate::tests::qa_automation::detector_accuracy_tests::DetectorAccuracyResults],
    ) -> (usize, usize, Vec<PerformanceIssue>) {
        let mut passed = 0;
        let mut failed = 0;
        let mut performance_issues = Vec::new();

        for result in accuracy_results {
            let issues = self.accuracy_suite.validate_accuracy_requirements(result);

            if issues.is_empty() {
                passed += 1;
            } else {
                failed += 1;
            }

            // Check performance thresholds
            if result.performance_metrics.average_execution_time_ms > self.config.performance_threshold_ms {
                performance_issues.push(PerformanceIssue {
                    detector_name: result.detector_name.clone(),
                    issue_type: "slow_execution".to_string(),
                    threshold: self.config.performance_threshold_ms,
                    actual_value: result.performance_metrics.average_execution_time_ms,
                    severity: if result.performance_metrics.average_execution_time_ms > self.config.performance_threshold_ms * 2.0 {
                        "critical".to_string()
                    } else {
                        "warning".to_string()
                    },
                });
            }

            // Check accuracy thresholds
            if result.precision < self.config.accuracy_threshold {
                performance_issues.push(PerformanceIssue {
                    detector_name: result.detector_name.clone(),
                    issue_type: "low_precision".to_string(),
                    threshold: self.config.accuracy_threshold,
                    actual_value: result.precision,
                    severity: "critical".to_string(),
                });
            }

            if result.recall < self.config.accuracy_threshold {
                performance_issues.push(PerformanceIssue {
                    detector_name: result.detector_name.clone(),
                    issue_type: "low_recall".to_string(),
                    threshold: self.config.accuracy_threshold,
                    actual_value: result.recall,
                    severity: "critical".to_string(),
                });
            }

            // Check confidence thresholds
            let avg_confidence = result.confidence_distribution.iter().sum::<f32>()
                / result.confidence_distribution.len() as f32;

            if avg_confidence < self.config.confidence_threshold {
                performance_issues.push(PerformanceIssue {
                    detector_name: result.detector_name.clone(),
                    issue_type: "low_confidence".to_string(),
                    threshold: self.config.confidence_threshold,
                    actual_value: avg_confidence,
                    severity: "warning".to_string(),
                });
            }
        }

        (passed, failed, performance_issues)
    }

    /// Determine overall CI status based on results
    fn determine_ci_status(&self, results: &CITestResults) -> CIStatus {
        // Check for critical failures
        if results.summary.detectors_failed > 0 && self.config.fail_on_accuracy_drop {
            return CIStatus::Failed;
        }

        let critical_regressions = results.regression_results.iter()
            .filter(|r| matches!(r.severity, crate::tests::qa_automation::regression_tests::RegressionSeverity::Critical))
            .count();

        if critical_regressions > 0 && self.config.fail_on_regressions {
            return CIStatus::Failed;
        }

        let critical_performance_issues = results.performance_issues.iter()
            .filter(|issue| issue.severity == "critical")
            .count();

        if critical_performance_issues > 0 {
            return CIStatus::Failed;
        }

        // Check for warnings
        let warnings = results.performance_issues.iter()
            .filter(|issue| issue.severity == "warning")
            .count();

        let minor_regressions = results.regression_results.iter()
            .filter(|r| matches!(r.severity, crate::tests::qa_automation::regression_tests::RegressionSeverity::Minor))
            .count();

        if warnings > 0 || minor_regressions > 0 {
            return CIStatus::WarningsOnly;
        }

        CIStatus::Success
    }

    /// Generate CI artifacts (reports, test results)
    async fn generate_ci_artifacts(&self, results: &CITestResults) -> Vec<CIArtifact> {
        let mut artifacts = Vec::new();

        // Create output directory
        if let Err(e) = tokio::fs::create_dir_all(&self.config.report_output_path).await {
            error!("Failed to create report directory: {}", e);
            return artifacts;
        }

        // Generate accuracy report
        let accuracy_report = self.accuracy_suite.generate_accuracy_report(&results.accuracy_results);
        let accuracy_report_path = format!("{}/accuracy_report.md", self.config.report_output_path);

        if let Ok(_) = tokio::fs::write(&accuracy_report_path, &accuracy_report).await {
            artifacts.push(CIArtifact {
                name: "Detector Accuracy Report".to_string(),
                path: accuracy_report_path,
                artifact_type: ArtifactType::AccuracyReport,
                size_bytes: accuracy_report.len() as u64,
            });
        }

        // Generate regression report
        let regression_report = self.regression_suite.generate_regression_report(&results.regression_results);
        let regression_report_path = format!("{}/regression_report.md", self.config.report_output_path);

        if let Ok(_) = tokio::fs::write(&regression_report_path, &regression_report).await {
            artifacts.push(CIArtifact {
                name: "Regression Test Report".to_string(),
                path: regression_report_path,
                artifact_type: ArtifactType::RegressionReport,
                size_bytes: regression_report.len() as u64,
            });
        }

        // Generate performance report
        let performance_report = self.generate_performance_report(results);
        let performance_report_path = format!("{}/performance_report.md", self.config.report_output_path);

        if let Ok(_) = tokio::fs::write(&performance_report_path, &performance_report).await {
            artifacts.push(CIArtifact {
                name: "Performance Report".to_string(),
                path: performance_report_path,
                artifact_type: ArtifactType::PerformanceReport,
                size_bytes: performance_report.len() as u64,
            });
        }

        // Generate JSON test results for CI systems
        let json_results = serde_json::to_string_pretty(results).unwrap_or_default();
        let json_path = format!("{}/test_results.json", self.config.report_output_path);

        if let Ok(_) = tokio::fs::write(&json_path, &json_results).await {
            artifacts.push(CIArtifact {
                name: "Test Results JSON".to_string(),
                path: json_path,
                artifact_type: ArtifactType::TestResults,
                size_bytes: json_results.len() as u64,
            });
        }

        artifacts
    }

    /// Generate performance report
    fn generate_performance_report(&self, results: &CITestResults) -> String {
        let mut report = String::new();

        report.push_str("# Detector Performance Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Performance summary
        report.push_str("## Performance Summary\n\n");

        let total_issues = results.performance_issues.len();
        let critical_issues = results.performance_issues.iter()
            .filter(|issue| issue.severity == "critical")
            .count();
        let warning_issues = results.performance_issues.iter()
            .filter(|issue| issue.severity == "warning")
            .count();

        report.push_str(&format!("- Total performance issues: {}\n", total_issues));
        report.push_str(&format!("- Critical issues: {} 🚨\n", critical_issues));
        report.push_str(&format!("- Warning issues: {} ⚠️\n\n", warning_issues));

        if total_issues == 0 {
            report.push_str("✅ **No performance issues detected!**\n\n");
            return report;
        }

        // Detailed issues
        report.push_str("## Performance Issues\n\n");

        for issue in &results.performance_issues {
            let severity_icon = match issue.severity.as_str() {
                "critical" => "🚨",
                "warning" => "⚠️",
                _ => "ℹ️",
            };

            report.push_str(&format!("### {} {} - {}\n\n", severity_icon, issue.detector_name, issue.issue_type));
            report.push_str(&format!("- **Threshold**: {:.3}\n", issue.threshold));
            report.push_str(&format!("- **Actual**: {:.3}\n", issue.actual_value));
            report.push_str(&format!("- **Severity**: {}\n\n", issue.severity));
        }

        // Performance metrics table
        report.push_str("## Detector Performance Metrics\n\n");
        report.push_str("| Detector | Avg Time (ms) | Precision | Recall | F1 Score | Status |\n");
        report.push_str("|----------|---------------|-----------|--------|----------|--------|\n");

        for result in &results.accuracy_results {
            let status = if results.performance_issues.iter()
                .any(|issue| issue.detector_name == result.detector_name && issue.severity == "critical") {
                "🚨 CRITICAL"
            } else if results.performance_issues.iter()
                .any(|issue| issue.detector_name == result.detector_name && issue.severity == "warning") {
                "⚠️ WARNING"
            } else {
                "✅ OK"
            };

            report.push_str(&format!(
                "| {} | {:.1} | {:.3} | {:.3} | {:.3} | {} |\n",
                result.detector_name,
                result.performance_metrics.average_execution_time_ms,
                result.precision,
                result.recall,
                result.f1_score,
                status
            ));
        }

        report
    }

    /// Check if CI should run based on current context
    pub fn should_run_ci_tests(&self) -> bool {
        // Check if running in CI environment
        let is_ci = std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok();

        if !is_ci {
            return true; // Always run locally
        }

        // Check branch context
        let current_branch = std::env::var("GITHUB_REF_NAME")
            .or_else(|_| std::env::var("CI_COMMIT_REF_NAME"))
            .unwrap_or_default();

        let is_main_branch = current_branch == "main" || current_branch == "master";
        let is_pr = std::env::var("GITHUB_EVENT_NAME").map(|e| e == "pull_request").unwrap_or(false);

        (is_main_branch && self.config.run_on_main_branch) ||
        (is_pr && self.config.run_on_pr)
    }

    /// Entry point for CI systems
    pub async fn ci_main() -> i32 {
        tracing_subscriber::fmt::init();

        let ci = CIIntegration::new();

        if !ci.should_run_ci_tests() {
            info!("Skipping CI tests based on configuration");
            return 0;
        }

        let results = ci.run_ci_tests().await;

        // Print summary for CI logs
        println!("=== QA Test Results ===");
        println!("Status: {:?}", results.overall_status);
        println!("Detectors tested: {}", results.summary.total_detectors_tested);
        println!("Detectors passed: {}", results.summary.detectors_passed);
        println!("Detectors failed: {}", results.summary.detectors_failed);
        println!("Regressions found: {}", results.summary.regressions_found);
        println!("Critical issues: {}", results.summary.critical_issues);
        println!("Execution time: {:.1}s", results.summary.execution_time_seconds);

        if !results.artifacts.is_empty() {
            println!("\nGenerated artifacts:");
            for artifact in &results.artifacts {
                println!("- {} ({})", artifact.name, artifact.path);
            }
        }

        match results.overall_status {
            CIStatus::Success => {
                println!("✅ All tests passed!");
                0
            }
            CIStatus::WarningsOnly => {
                println!("⚠️  Tests passed with warnings");
                0 // Don't fail CI for warnings
            }
            CIStatus::Failed => {
                println!("❌ Tests failed!");
                1
            }
            CIStatus::Error => {
                println!("💥 Test execution error!");
                2
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ci_integration_creation() {
        let ci = CIIntegration::new();
        assert!(ci.config.accuracy_threshold > 0.0);
    }

    #[tokio::test]
    async fn test_should_run_ci_tests() {
        let ci = CIIntegration::new();
        // Should always run in non-CI environments
        assert!(ci.should_run_ci_tests());
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let ci = CIIntegration::new();
        let results = CITestResults {
            overall_status: CIStatus::Success,
            accuracy_results: vec![],
            regression_results: vec![],
            performance_issues: vec![
                PerformanceIssue {
                    detector_name: "TestDetector".to_string(),
                    issue_type: "slow_execution".to_string(),
                    threshold: 1000.0,
                    actual_value: 1500.0,
                    severity: "warning".to_string(),
                }
            ],
            summary: CITestSummary {
                total_detectors_tested: 1,
                detectors_passed: 1,
                detectors_failed: 0,
                regressions_found: 0,
                critical_issues: 0,
                warnings: 1,
                execution_time_seconds: 10.0,
            },
            artifacts: vec![],
        };

        let report = ci.generate_performance_report(&results);
        assert!(report.contains("Performance Report"));
        assert!(report.contains("TestDetector"));
    }
}
"#.to_string(),