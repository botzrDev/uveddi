//! Integration layer between Criterion.rs and statistical regression detection
//!
//! This module provides:
//! - Seamless integration of Criterion benchmarks with statistical analysis
//! - Automatic baseline management for Criterion results
//! - Enhanced regression detection with statistical confidence
//! - Performance report generation combining both systems

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};

use crate::performance::{
    performance_reports::{PerformanceReport, PerformanceReportGenerator},
    BaselineConfig, BaselineType, BenchmarkBaselineManager, PerformanceRegressionDetector,
    RegressionDetectionConfig, StatisticalAnalyzer, TrendDetector,
};

/// Criterion.rs integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionIntegrationConfig {
    pub baseline_storage_path: PathBuf,
    pub enable_automatic_baseline_updates: bool,
    pub regression_threshold_percent: f64,
    pub statistical_confidence_threshold: f64,
    pub min_samples_for_analysis: usize,
    pub report_output_directory: PathBuf,
    pub generate_html_reports: bool,
    pub generate_json_reports: bool,
}

impl Default for CriterionIntegrationConfig {
    fn default() -> Self {
        Self {
            baseline_storage_path: PathBuf::from("target/criterion/baselines.json"),
            enable_automatic_baseline_updates: false,
            regression_threshold_percent: 5.0,
            statistical_confidence_threshold: 0.95,
            min_samples_for_analysis: 10,
            report_output_directory: PathBuf::from("target/criterion/reports"),
            generate_html_reports: true,
            generate_json_reports: true,
        }
    }
}

/// Criterion benchmark result extracted for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionBenchmarkResult {
    pub benchmark_name: String,
    pub measurements: Vec<f64>,
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub median_ns: f64,
    pub mad_ns: f64,
    pub sample_count: usize,
    pub timestamp: SystemTime,
    pub throughput: Option<CriterionThroughput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionThroughput {
    pub elements_per_second: f64,
    pub bytes_per_second: Option<f64>,
}

/// Integration result combining Criterion and statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedBenchmarkResult {
    pub criterion_result: CriterionBenchmarkResult,
    pub baseline_comparison: Option<crate::performance::BaselineComparison>,
    pub statistical_analysis: IntegratedStatisticalAnalysis,
    pub regression_verdict: RegressionVerdict,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedStatisticalAnalysis {
    pub mann_kendall_result: Option<crate::performance::MannKendallResult>,
    pub change_point_analysis: Option<crate::performance::ChangePointResult>,
    pub confidence_interval_95: (f64, f64),
    pub effect_size: Option<f64>,
    pub trend_stability: f64,
    pub performance_percentiles: PerformancePercentiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePercentiles {
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionVerdict {
    Pass,
    PassWithWarning { warning: String },
    Fail { reason: String },
    InsufficientData,
}

/// Main integration manager
pub struct CriterionIntegrationManager {
    config: CriterionIntegrationConfig,
    baseline_manager: BenchmarkBaselineManager,
    statistical_analyzer: StatisticalAnalyzer,
    trend_detector: TrendDetector,
    regression_detector: Option<PerformanceRegressionDetector>,
    report_generator: PerformanceReportGenerator,
}

impl CriterionIntegrationManager {
    /// Create a new integration manager
    pub async fn new(config: CriterionIntegrationConfig) -> Result<Self> {
        // Create baseline manager
        let baseline_config = BaselineConfig {
            regression_threshold_percent: config.regression_threshold_percent,
            improvement_threshold_percent: config.regression_threshold_percent,
            statistical_confidence_threshold: config.statistical_confidence_threshold,
            min_samples_for_baseline: config.min_samples_for_analysis,
            max_baseline_age_days: 30,
            enable_automatic_baseline_updates: config.enable_automatic_baseline_updates,
        };

        let baseline_manager =
            BenchmarkBaselineManager::new(&config.baseline_storage_path, baseline_config).await?;

        // Create regression detector if needed
        let regression_detector = if std::env::var("ENABLE_REGRESSION_DETECTION").is_ok() {
            let regression_config = RegressionDetectionConfig::default();
            PerformanceRegressionDetector::new(regression_config).ok()
        } else {
            None
        };

        Ok(Self {
            config,
            baseline_manager,
            statistical_analyzer: StatisticalAnalyzer::new(),
            trend_detector: TrendDetector::new(),
            regression_detector,
            report_generator: PerformanceReportGenerator::new()?,
        })
    }

    /// Process a Criterion benchmark result
    pub async fn process_benchmark_result(
        &mut self,
        benchmark_result: CriterionBenchmarkResult,
    ) -> Result<IntegratedBenchmarkResult> {
        info!(
            "Processing Criterion benchmark: {}",
            benchmark_result.benchmark_name
        );

        // Perform statistical analysis
        let statistical_analysis = self.analyze_benchmark_statistics(&benchmark_result).await?;

        // Compare against baseline if available
        let baseline_comparison = self.compare_against_baseline(&benchmark_result).await?;

        // Determine regression verdict
        let regression_verdict = self.determine_regression_verdict(
            &benchmark_result,
            &baseline_comparison,
            &statistical_analysis,
        );

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &benchmark_result,
            &baseline_comparison,
            &regression_verdict,
        );

        let result = IntegratedBenchmarkResult {
            criterion_result: benchmark_result,
            baseline_comparison,
            statistical_analysis,
            regression_verdict,
            recommendations,
        };

        // Log result summary
        self.log_result_summary(&result);

        Ok(result)
    }

    /// Analyze benchmark statistics
    async fn analyze_benchmark_statistics(
        &self,
        benchmark_result: &CriterionBenchmarkResult,
    ) -> Result<IntegratedStatisticalAnalysis> {
        let measurements = &benchmark_result.measurements;

        // Perform Mann-Kendall trend test if sufficient data
        let mann_kendall_result = if measurements.len() >= 10 {
            self.statistical_analyzer
                .mann_kendall_test(measurements)
                .ok()
        } else {
            None
        };

        // Perform change point detection if sufficient data
        let change_point_analysis = if measurements.len() >= 20 {
            self.trend_detector
                .detect_change_points_pelt(measurements)
                .ok()
        } else {
            None
        };

        // Calculate confidence interval
        let confidence_interval_95 = self
            .statistical_analyzer
            .confidence_interval(measurements, 0.95)
            .unwrap_or((benchmark_result.mean_ns, benchmark_result.mean_ns));

        // Calculate trend stability
        let trend_stability = if benchmark_result.std_dev_ns > 0.0 {
            1.0 / (1.0 + (benchmark_result.std_dev_ns / benchmark_result.mean_ns).min(10.0))
        } else {
            1.0
        };

        // Calculate performance percentiles
        let performance_percentiles = self.calculate_percentiles(measurements);

        Ok(IntegratedStatisticalAnalysis {
            mann_kendall_result,
            change_point_analysis,
            confidence_interval_95,
            effect_size: None, // Will be calculated during baseline comparison
            trend_stability,
            performance_percentiles,
        })
    }

    /// Compare benchmark against existing baseline
    async fn compare_against_baseline(
        &mut self,
        benchmark_result: &CriterionBenchmarkResult,
    ) -> Result<Option<crate::performance::BaselineComparison>> {
        let baseline_type = BaselineType::Criterion {
            mean_ns: benchmark_result.mean_ns,
            std_dev_ns: benchmark_result.std_dev_ns,
            median_ns: benchmark_result.median_ns,
        };

        match self
            .baseline_manager
            .compare_against_baseline(
                &benchmark_result.benchmark_name,
                &benchmark_result.measurements,
                baseline_type,
            )
            .await
        {
            Ok(comparison) => {
                // Update baseline if configured to do so
                if self.config.enable_automatic_baseline_updates {
                    self.maybe_update_baseline(benchmark_result, &comparison)
                        .await?;
                }
                Ok(Some(comparison))
            }
            Err(e) => {
                debug!(
                    "No baseline available for {}: {}",
                    benchmark_result.benchmark_name, e
                );

                // Create initial baseline if we have enough data
                if benchmark_result.measurements.len() >= self.config.min_samples_for_analysis {
                    self.create_initial_baseline(benchmark_result).await?;
                }

                Ok(None)
            }
        }
    }

    /// Maybe update baseline based on comparison results
    async fn maybe_update_baseline(
        &mut self,
        benchmark_result: &CriterionBenchmarkResult,
        comparison: &crate::performance::BaselineComparison,
    ) -> Result<()> {
        // Only update if the result is acceptable and has high confidence
        let should_update = match &comparison.recommendation {
            crate::performance::BaselineRecommendation::Accept => {
                comparison.statistical_confidence > 0.8
            }
            _ => false,
        };

        if should_update {
            let baseline_type = BaselineType::Criterion {
                mean_ns: benchmark_result.mean_ns,
                std_dev_ns: benchmark_result.std_dev_ns,
                median_ns: benchmark_result.median_ns,
            };

            self.baseline_manager
                .update_baseline(
                    &benchmark_result.benchmark_name,
                    benchmark_result.measurements.clone(),
                    baseline_type,
                )
                .await?;

            info!("Updated baseline for {}", benchmark_result.benchmark_name);
        }

        Ok(())
    }

    /// Create initial baseline for new benchmark
    async fn create_initial_baseline(
        &mut self,
        benchmark_result: &CriterionBenchmarkResult,
    ) -> Result<()> {
        let baseline_type = BaselineType::Criterion {
            mean_ns: benchmark_result.mean_ns,
            std_dev_ns: benchmark_result.std_dev_ns,
            median_ns: benchmark_result.median_ns,
        };

        self.baseline_manager
            .create_baseline(
                &benchmark_result.benchmark_name,
                benchmark_result.measurements.clone(),
                baseline_type,
            )
            .await?;

        info!(
            "Created initial baseline for {}",
            benchmark_result.benchmark_name
        );
        Ok(())
    }

    /// Calculate performance percentiles
    fn calculate_percentiles(&self, measurements: &[f64]) -> PerformancePercentiles {
        let mut sorted = measurements.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let percentile = |p: f64| -> f64 {
            if sorted.is_empty() {
                return 0.0;
            }
            let index = p * (sorted.len() - 1) as f64;
            let lower = index.floor() as usize;
            let upper = index.ceil() as usize;

            if lower == upper {
                sorted[lower]
            } else {
                let weight = index - lower as f64;
                sorted[lower] * (1.0 - weight) + sorted[upper] * weight
            }
        };

        PerformancePercentiles {
            p50: percentile(0.50),
            p75: percentile(0.75),
            p90: percentile(0.90),
            p95: percentile(0.95),
            p99: percentile(0.99),
        }
    }

    /// Determine overall regression verdict
    fn determine_regression_verdict(
        &self,
        benchmark_result: &CriterionBenchmarkResult,
        baseline_comparison: &Option<crate::performance::BaselineComparison>,
        statistical_analysis: &IntegratedStatisticalAnalysis,
    ) -> RegressionVerdict {
        // Check if we have sufficient data
        if benchmark_result.measurements.len() < self.config.min_samples_for_analysis {
            return RegressionVerdict::InsufficientData;
        }

        // No baseline means we can't detect regression
        let comparison = match baseline_comparison {
            Some(comp) => comp,
            None => return RegressionVerdict::Pass, // No baseline, so pass by default
        };

        // Check statistical confidence
        if comparison.statistical_confidence < self.config.statistical_confidence_threshold {
            return RegressionVerdict::PassWithWarning {
                warning: format!(
                    "Low statistical confidence ({:.2})",
                    comparison.statistical_confidence
                ),
            };
        }

        // Check for regression based on baseline comparison
        match &comparison.recommendation {
            crate::performance::BaselineRecommendation::Accept => {
                // Additional checks for warnings
                if statistical_analysis.trend_stability < 0.5 {
                    RegressionVerdict::PassWithWarning {
                        warning: "High performance variability detected".to_string(),
                    }
                } else {
                    RegressionVerdict::Pass
                }
            }
            crate::performance::BaselineRecommendation::Investigate { reasons } => {
                RegressionVerdict::PassWithWarning {
                    warning: reasons.join("; "),
                }
            }
            crate::performance::BaselineRecommendation::Reject { reasons } => {
                RegressionVerdict::Fail {
                    reason: reasons.join("; "),
                }
            }
            crate::performance::BaselineRecommendation::RequireManualReview => {
                RegressionVerdict::PassWithWarning {
                    warning: "Manual review required due to statistical uncertainty".to_string(),
                }
            }
        }
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(
        &self,
        benchmark_result: &CriterionBenchmarkResult,
        baseline_comparison: &Option<crate::performance::BaselineComparison>,
        regression_verdict: &RegressionVerdict,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Sample size recommendations
        if benchmark_result.measurements.len() < 50 {
            recommendations
                .push("Consider increasing sample size for more reliable results".to_string());
        }

        // Variability recommendations
        let cv = benchmark_result.std_dev_ns / benchmark_result.mean_ns;
        if cv > 0.1 {
            recommendations.push(format!(
                "High coefficient of variation ({:.1}%) suggests unstable performance. Consider investigating environmental factors.",
                cv * 100.0
            ));
        }

        // Baseline-specific recommendations
        if let Some(comparison) = baseline_comparison {
            if comparison.statistical_confidence < 0.8 {
                recommendations.push("Low statistical confidence. Consider collecting more data or improving test environment stability".to_string());
            }

            if let Some(mk) = &comparison
                .current_baseline
                .statistical_summary
                .mann_kendall_result
            {
                if mk.p_value < 0.01 {
                    recommendations.push(format!(
                        "Strong statistical evidence of {} trend detected",
                        format!("{:?}", mk.trend).to_lowercase()
                    ));
                }
            }
        }

        // Verdict-specific recommendations
        match regression_verdict {
            RegressionVerdict::Fail { reason } => {
                recommendations.push(format!("Action required: {}", reason));
                recommendations
                    .push("Review recent code changes and system configuration".to_string());
            }
            RegressionVerdict::PassWithWarning { warning } => {
                recommendations.push(format!("Monitor closely: {}", warning));
            }
            RegressionVerdict::InsufficientData => {
                recommendations.push("Increase sample size or measurement duration".to_string());
            }
            _ => {}
        }

        recommendations
    }

    /// Log result summary
    fn log_result_summary(&self, result: &IntegratedBenchmarkResult) {
        let benchmark_name = &result.criterion_result.benchmark_name;

        match &result.regression_verdict {
            RegressionVerdict::Pass => {
                info!("✅ {} passed performance validation", benchmark_name);
            }
            RegressionVerdict::PassWithWarning { warning } => {
                warn!("⚠️  {} passed with warning: {}", benchmark_name, warning);
            }
            RegressionVerdict::Fail { reason } => {
                error!(
                    "❌ {} failed performance validation: {}",
                    benchmark_name, reason
                );
            }
            RegressionVerdict::InsufficientData => {
                warn!("📊 {} has insufficient data for validation", benchmark_name);
            }
        }

        if let Some(comparison) = &result.baseline_comparison {
            debug!(
                "{}: {:.2}% change, {:.2} confidence, {} samples",
                benchmark_name,
                comparison.comparison_result.performance_change_percent,
                comparison.statistical_confidence,
                result.criterion_result.sample_count
            );
        }
    }

    /// Process multiple benchmark results and generate comprehensive report
    pub async fn process_benchmark_suite(
        &mut self,
        benchmark_results: Vec<CriterionBenchmarkResult>,
    ) -> Result<(Vec<IntegratedBenchmarkResult>, Option<PerformanceReport>)> {
        info!(
            "Processing benchmark suite with {} benchmarks",
            benchmark_results.len()
        );

        let mut integrated_results = Vec::new();
        let mut baseline_comparisons = Vec::new();

        // Process each benchmark
        for benchmark_result in benchmark_results {
            let integrated_result = self.process_benchmark_result(benchmark_result).await?;

            if let Some(ref comparison) = integrated_result.baseline_comparison {
                baseline_comparisons.push(comparison.clone());
            }

            integrated_results.push(integrated_result);
        }

        // Generate comprehensive report if enabled
        let performance_report =
            if self.config.generate_html_reports || self.config.generate_json_reports {
                let report = self
                    .report_generator
                    .generate_report(
                        baseline_comparisons,
                        None, // Historical data would be loaded here
                    )
                    .await?;

                // Export reports
                if self.config.generate_html_reports {
                    let html_path = self
                        .config
                        .report_output_directory
                        .join("performance_report.html");
                    self.report_generator
                        .export_html(&report, &html_path)
                        .await?;
                    info!("Generated HTML report: {}", html_path.display());
                }

                if self.config.generate_json_reports {
                    let json_path = self
                        .config
                        .report_output_directory
                        .join("performance_report.json");
                    self.report_generator
                        .export_json(&report, &json_path)
                        .await?;
                    info!("Generated JSON report: {}", json_path.display());
                }

                Some(report)
            } else {
                None
            };

        // Summary logging
        let (pass_count, warning_count, fail_count) =
            integrated_results
                .iter()
                .fold((0, 0, 0), |(pass, warn, fail), result| {
                    match result.regression_verdict {
                        RegressionVerdict::Pass => (pass + 1, warn, fail),
                        RegressionVerdict::PassWithWarning { .. } => (pass, warn + 1, fail),
                        RegressionVerdict::Fail { .. } => (pass, warn, fail + 1),
                        RegressionVerdict::InsufficientData => (pass, warn, fail),
                    }
                });

        info!(
            "Benchmark suite completed: {} passed, {} warnings, {} failures",
            pass_count, warning_count, fail_count
        );

        Ok((integrated_results, performance_report))
    }

    /// Helper method to extract Criterion results from benchmark run
    /// This would typically be called by a custom Criterion measurement or profiler
    pub fn extract_criterion_result(
        benchmark_name: &str,
        measurements: Vec<f64>,
        throughput: Option<CriterionThroughput>,
    ) -> CriterionBenchmarkResult {
        let mean_ns = measurements.iter().sum::<f64>() / measurements.len() as f64;

        let variance = measurements
            .iter()
            .map(|x| (x - mean_ns).powi(2))
            .sum::<f64>()
            / measurements.len() as f64;
        let std_dev_ns = variance.sqrt();

        let mut sorted = measurements.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_ns = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        // Calculate MAD (Median Absolute Deviation)
        let deviations: Vec<f64> = measurements.iter().map(|x| (x - median_ns).abs()).collect();
        let mut sorted_deviations = deviations;
        sorted_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mad_ns = if sorted_deviations.len() % 2 == 0 {
            (sorted_deviations[sorted_deviations.len() / 2 - 1]
                + sorted_deviations[sorted_deviations.len() / 2])
                / 2.0
        } else {
            sorted_deviations[sorted_deviations.len() / 2]
        };

        CriterionBenchmarkResult {
            benchmark_name: benchmark_name.to_string(),
            measurements: measurements.clone(),
            mean_ns,
            std_dev_ns,
            median_ns,
            mad_ns,
            sample_count: measurements.len(),
            timestamp: SystemTime::now(),
            throughput,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_criterion_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config = CriterionIntegrationConfig {
            baseline_storage_path: temp_dir.path().join("baselines.json"),
            report_output_directory: temp_dir.path().join("reports"),
            ..Default::default()
        };

        let mut manager = CriterionIntegrationManager::new(config).await.unwrap();

        // Simulate Criterion benchmark result
        let measurements = (0..100)
            .map(|i| 1000.0 + i as f64 * 0.1)
            .collect::<Vec<_>>();
        let benchmark_result = CriterionIntegrationManager::extract_criterion_result(
            "test_benchmark",
            measurements,
            None,
        );

        // Process first result (should create baseline)
        let result1 = manager
            .process_benchmark_result(benchmark_result.clone())
            .await
            .unwrap();
        assert!(result1.baseline_comparison.is_none()); // No baseline exists yet

        // Process second result (should compare against baseline)
        let mut measurements2 = (0..100)
            .map(|i| 1010.0 + i as f64 * 0.1)
            .collect::<Vec<_>>();
        measurements2[50] = 2000.0; // Add some variance

        let benchmark_result2 = CriterionIntegrationManager::extract_criterion_result(
            "test_benchmark",
            measurements2,
            None,
        );

        let result2 = manager
            .process_benchmark_result(benchmark_result2)
            .await
            .unwrap();
        assert!(result2.baseline_comparison.is_some()); // Should have baseline comparison
    }
}
