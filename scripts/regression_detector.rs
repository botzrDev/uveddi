#!/usr/bin/env cargo -Zscript
//! Automated Regression Detection System for UV-91
//!
//! Comprehensive regression detection system that automatically monitors
//! performance baselines, detects regressions, and generates actionable reports.
//! Integrates with CI/CD pipelines for continuous performance monitoring.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

/// Main regression detection system
#[derive(Debug)]
pub struct RegressionDetector {
    config: RegressionConfig,
    baseline_storage: BaselineStorage,
    performance_history: PerformanceHistory,
    alert_system: AlertSystem,
    report_generator: ReportGenerator,
}

/// Configuration for regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    pub baseline_path: PathBuf,
    pub benchmark_command: String,
    pub regression_thresholds: RegressionThresholds,
    pub monitoring_intervals: MonitoringIntervals,
    pub alert_settings: AlertSettings,
    pub ci_integration: CiIntegrationConfig,
    pub statistical_config: StatisticalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionThresholds {
    pub pipeline_latency_threshold_percent: f64,
    pub memory_usage_threshold_percent: f64,
    pub cache_hit_rate_threshold_percent: f64,
    pub throughput_threshold_percent: f64,
    pub error_rate_threshold_percent: f64,
    pub critical_threshold_percent: f64,
    pub major_threshold_percent: f64,
    pub minor_threshold_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringIntervals {
    pub continuous_monitoring_seconds: u64,
    pub baseline_update_days: u64,
    pub history_retention_days: u64,
    pub alert_cooldown_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSettings {
    pub enabled: bool,
    pub email_notifications: Vec<String>,
    pub slack_webhook: Option<String>,
    pub github_issue_creation: bool,
    pub severity_filters: Vec<RegressionSeverity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiIntegrationConfig {
    pub enabled: bool,
    pub fail_build_on_critical: bool,
    pub fail_build_on_major: bool,
    pub comment_on_pr: bool,
    pub create_performance_badge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalConfig {
    pub confidence_level: f64,
    pub minimum_samples: usize,
    pub outlier_detection_method: OutlierDetectionMethod,
    pub trend_analysis_window: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierDetectionMethod {
    StandardDeviation,
    InterquartileRange,
    ModifiedZScore,
    Isolation,
}

/// Performance measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    pub timestamp: DateTime<Utc>,
    pub git_commit: String,
    pub branch: String,
    pub environment: EnvironmentInfo,
    pub metrics: PerformanceMetrics,
    pub benchmark_duration: Duration,
    pub measurement_metadata: MeasurementMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub memory_gb: f64,
    pub rust_version: String,
    pub build_mode: String,
    pub compiler_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub pipeline_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cache_hit_rate: f64,
    pub throughput_operations_per_second: f64,
    pub error_rate: f64,
    pub custom_metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementMetadata {
    pub benchmark_variant: String,
    pub file_count: usize,
    pub warmup_iterations: u32,
    pub measurement_iterations: u32,
    pub statistical_confidence: f64,
}

/// Regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionResult {
    pub timestamp: DateTime<Utc>,
    pub baseline_commit: String,
    pub current_commit: String,
    pub regressions_detected: Vec<DetectedRegression>,
    pub overall_severity: RegressionSeverity,
    pub performance_summary: PerformanceSummary,
    pub statistical_analysis: StatisticalAnalysis,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedRegression {
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub change_percent: f64,
    pub severity: RegressionSeverity,
    pub statistical_significance: f64,
    pub trend_analysis: TrendAnalysis,
    pub potential_causes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegressionSeverity {
    None,
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub overall_performance_change: f64,
    pub best_performing_metrics: Vec<String>,
    pub worst_performing_metrics: Vec<String>,
    pub stability_score: f64,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    pub confidence_interval: (f64, f64),
    pub p_value: f64,
    pub effect_size: f64,
    pub statistical_power: f64,
    pub outliers_detected: Vec<OutlierInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub trend_direction: TrendDirection,
    pub trend_strength: f64,
    pub trend_duration_days: u64,
    pub forecast_next_week: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierInfo {
    pub measurement_timestamp: DateTime<Utc>,
    pub metric_name: String,
    pub value: f64,
    pub outlier_score: f64,
    pub detection_method: OutlierDetectionMethod,
}

/// Storage for baseline data
#[derive(Debug)]
pub struct BaselineStorage {
    storage_path: PathBuf,
    current_baseline: Option<PerformanceMeasurement>,
    baseline_history: Vec<PerformanceMeasurement>,
}

/// Performance history tracking
#[derive(Debug)]
pub struct PerformanceHistory {
    measurements: VecDeque<PerformanceMeasurement>,
    max_history_size: usize,
    trend_cache: HashMap<String, TrendAnalysis>,
}

/// Alert system for notifications
#[derive(Debug)]
pub struct AlertSystem {
    config: AlertSettings,
    last_alert_times: HashMap<String, DateTime<Utc>>,
}

/// Report generation system
#[derive(Debug)]
pub struct ReportGenerator {
    template_engine: ReportTemplateEngine,
    output_formats: Vec<OutputFormat>,
}

#[derive(Debug)]
pub enum OutputFormat {
    Html,
    Markdown,
    Json,
    Pdf,
    Slack,
    Email,
}

#[derive(Debug)]
pub struct ReportTemplateEngine {
    templates: HashMap<String, String>,
}

impl RegressionDetector {
    pub fn new(config: RegressionConfig) -> Result<Self> {
        let baseline_storage = BaselineStorage::new(&config.baseline_path)?;
        let performance_history = PerformanceHistory::new(10000); // Keep 10k measurements
        let alert_system = AlertSystem::new(config.alert_settings.clone());
        let report_generator = ReportGenerator::new()?;

        Ok(Self {
            config,
            baseline_storage,
            performance_history,
            alert_system,
            report_generator,
        })
    }

    /// Run regression detection for current state
    pub async fn detect_regressions(&mut self) -> Result<RegressionResult> {
        println!("🔍 Starting regression detection...");

        // Run benchmarks to get current performance
        let current_measurement = self.run_performance_measurement().await?;
        
        // Store in history
        self.performance_history.add_measurement(current_measurement.clone());

        // Get baseline for comparison
        let baseline = self.baseline_storage.get_current_baseline()
            .context("No baseline available for comparison")?;

        // Perform regression analysis
        let regression_result = self.analyze_performance_change(&baseline, &current_measurement).await?;

        // Generate alerts if regressions detected
        if !regression_result.regressions_detected.is_empty() {
            self.alert_system.send_alerts(&regression_result).await?;
        }

        // Generate and save reports
        self.report_generator.generate_reports(&regression_result).await?;

        // Update baseline if conditions are met
        if self.should_update_baseline(&current_measurement, &baseline) {
            self.baseline_storage.update_baseline(current_measurement)?;
        }

        println!("✅ Regression detection completed");
        Ok(regression_result)
    }

    /// Run continuous monitoring
    pub async fn start_continuous_monitoring(&mut self) -> Result<()> {
        println!("📊 Starting continuous performance monitoring...");

        let interval = Duration::from_secs(self.config.monitoring_intervals.continuous_monitoring_seconds);
        
        loop {
            match self.detect_regressions().await {
                Ok(result) => {
                    println!("Monitoring cycle completed: {} regressions detected", 
                            result.regressions_detected.len());
                    
                    if result.overall_severity >= RegressionSeverity::Major {
                        println!("⚠️  Major regression detected! Check reports for details.");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Monitoring cycle failed: {}", e);
                }
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Run performance measurement
    async fn run_performance_measurement(&self) -> Result<PerformanceMeasurement> {
        println!("📈 Running performance benchmarks...");

        let start_time = std::time::Instant::now();
        
        // Run benchmark command
        let benchmark_output = Command::new("sh")
            .arg("-c")
            .arg(&self.config.benchmark_command)
            .output()
            .context("Failed to run benchmark command")?;

        if !benchmark_output.status.success() {
            return Err(anyhow::anyhow!(
                "Benchmark command failed: {}",
                String::from_utf8_lossy(&benchmark_output.stderr)
            ));
        }

        let benchmark_duration = start_time.elapsed();

        // Parse benchmark results
        let benchmark_results = String::from_utf8(benchmark_output.stdout)
            .context("Invalid UTF-8 in benchmark output")?;

        let metrics = self.parse_benchmark_results(&benchmark_results)?;

        // Collect environment info
        let environment = self.collect_environment_info()?;

        // Get git information
        let git_commit = self.get_git_commit()?;
        let branch = self.get_git_branch()?;

        Ok(PerformanceMeasurement {
            timestamp: Utc::now(),
            git_commit,
            branch,
            environment,
            metrics,
            benchmark_duration,
            measurement_metadata: MeasurementMetadata {
                benchmark_variant: "enterprise_comprehensive".to_string(),
                file_count: 1000,
                warmup_iterations: 5,
                measurement_iterations: 20,
                statistical_confidence: 0.95,
            },
        })
    }

    /// Parse benchmark results from output
    fn parse_benchmark_results(&self, output: &str) -> Result<PerformanceMetrics> {
        // This is a simplified parser - in production, implement robust parsing
        // based on your actual benchmark output format
        
        let mut metrics = PerformanceMetrics {
            pipeline_latency_ms: 0.0,
            memory_usage_mb: 0.0,
            cache_hit_rate: 0.0,
            throughput_operations_per_second: 0.0,
            error_rate: 0.0,
            custom_metrics: HashMap::new(),
        };

        // Parse key metrics from benchmark output
        for line in output.lines() {
            if line.contains("Pipeline latency:") {
                if let Some(value) = self.extract_numeric_value(line, "ms") {
                    metrics.pipeline_latency_ms = value;
                }
            } else if line.contains("Memory usage:") {
                if let Some(value) = self.extract_numeric_value(line, "MB") {
                    metrics.memory_usage_mb = value;
                }
            } else if line.contains("Cache hit rate:") {
                if let Some(value) = self.extract_numeric_value(line, "%") {
                    metrics.cache_hit_rate = value;
                }
            } else if line.contains("Throughput:") {
                if let Some(value) = self.extract_numeric_value(line, "ops/s") {
                    metrics.throughput_operations_per_second = value;
                }
            } else if line.contains("Error rate:") {
                if let Some(value) = self.extract_numeric_value(line, "%") {
                    metrics.error_rate = value;
                }
            }
        }

        Ok(metrics)
    }

    /// Extract numeric value from text
    fn extract_numeric_value(&self, text: &str, unit: &str) -> Option<f64> {
        text.split_whitespace()
            .find_map(|word| {
                if word.ends_with(unit) {
                    word.trim_end_matches(unit).parse().ok()
                } else {
                    word.parse().ok()
                }
            })
    }

    /// Analyze performance change between baseline and current
    async fn analyze_performance_change(
        &self,
        baseline: &PerformanceMeasurement,
        current: &PerformanceMeasurement,
    ) -> Result<RegressionResult> {
        let mut detected_regressions = Vec::new();

        // Analyze each metric
        let metric_changes = vec![
            ("pipeline_latency_ms", baseline.metrics.pipeline_latency_ms, current.metrics.pipeline_latency_ms, self.config.regression_thresholds.pipeline_latency_threshold_percent),
            ("memory_usage_mb", baseline.metrics.memory_usage_mb, current.metrics.memory_usage_mb, self.config.regression_thresholds.memory_usage_threshold_percent),
            ("cache_hit_rate", baseline.metrics.cache_hit_rate, current.metrics.cache_hit_rate, self.config.regression_thresholds.cache_hit_rate_threshold_percent),
            ("throughput_operations_per_second", baseline.metrics.throughput_operations_per_second, current.metrics.throughput_operations_per_second, self.config.regression_thresholds.throughput_threshold_percent),
            ("error_rate", baseline.metrics.error_rate, current.metrics.error_rate, self.config.regression_thresholds.error_rate_threshold_percent),
        ];

        for (metric_name, baseline_value, current_value, threshold) in metric_changes {
            if baseline_value > 0.0 {
                let change_percent = ((current_value - baseline_value) / baseline_value) * 100.0;
                
                // Determine if this is a regression based on metric type
                let is_regression = match metric_name {
                    "pipeline_latency_ms" | "memory_usage_mb" | "error_rate" => change_percent > threshold,
                    "cache_hit_rate" | "throughput_operations_per_second" => change_percent < -threshold,
                    _ => false,
                };

                if is_regression {
                    let severity = self.calculate_regression_severity(change_percent.abs());
                    let trend_analysis = self.performance_history.get_trend_analysis(metric_name);
                    let statistical_significance = self.calculate_statistical_significance(
                        metric_name, baseline_value, current_value
                    );

                    detected_regressions.push(DetectedRegression {
                        metric_name: metric_name.to_string(),
                        baseline_value,
                        current_value,
                        change_percent,
                        severity,
                        statistical_significance,
                        trend_analysis,
                        potential_causes: self.identify_potential_causes(metric_name, change_percent),
                    });
                }
            }
        }

        // Calculate overall severity
        let overall_severity = detected_regressions.iter()
            .map(|r| &r.severity)
            .max()
            .cloned()
            .unwrap_or(RegressionSeverity::None);

        // Generate performance summary
        let performance_summary = self.generate_performance_summary(&baseline.metrics, &current.metrics);

        // Perform statistical analysis
        let statistical_analysis = self.perform_statistical_analysis(baseline, current).await?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&detected_regressions);

        Ok(RegressionResult {
            timestamp: Utc::now(),
            baseline_commit: baseline.git_commit.clone(),
            current_commit: current.git_commit.clone(),
            regressions_detected: detected_regressions,
            overall_severity,
            performance_summary,
            statistical_analysis,
            recommendations,
        })
    }

    /// Calculate regression severity based on percentage change
    fn calculate_regression_severity(&self, change_percent: f64) -> RegressionSeverity {
        if change_percent >= self.config.regression_thresholds.critical_threshold_percent {
            RegressionSeverity::Critical
        } else if change_percent >= self.config.regression_thresholds.major_threshold_percent {
            RegressionSeverity::Major
        } else if change_percent >= self.config.regression_thresholds.minor_threshold_percent {
            RegressionSeverity::Minor
        } else {
            RegressionSeverity::None
        }
    }

    /// Calculate statistical significance of the change
    fn calculate_statistical_significance(&self, metric_name: &str, baseline: f64, current: f64) -> f64 {
        // Simplified statistical significance calculation
        // In production, implement proper statistical tests based on historical data
        
        let historical_data = self.performance_history.get_metric_history(metric_name);
        if historical_data.len() < self.config.statistical_config.minimum_samples {
            return 0.5; // Low confidence with insufficient data
        }

        // Calculate standard deviation of historical data
        let mean: f64 = historical_data.iter().sum::<f64>() / historical_data.len() as f64;
        let variance: f64 = historical_data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / historical_data.len() as f64;
        let std_dev = variance.sqrt();

        // Calculate z-score
        let z_score = (current - baseline).abs() / std_dev;
        
        // Convert to confidence level (simplified)
        match z_score {
            z if z >= 2.58 => 0.99,  // 99% confidence
            z if z >= 1.96 => 0.95,  // 95% confidence
            z if z >= 1.64 => 0.90,  // 90% confidence
            z if z >= 1.28 => 0.80,  // 80% confidence
            _ => 0.50,               // Low confidence
        }
    }

    /// Identify potential causes for regression
    fn identify_potential_causes(&self, metric_name: &str, change_percent: f64) -> Vec<String> {
        let mut causes = Vec::new();

        match metric_name {
            "pipeline_latency_ms" => {
                if change_percent > 20.0 {
                    causes.push("Possible algorithmic changes or increased computational complexity".to_string());
                    causes.push("New dependencies with performance overhead".to_string());
                    causes.push("Resource contention or system load changes".to_string());
                }
            }
            "memory_usage_mb" => {
                if change_percent > 15.0 {
                    causes.push("Memory leaks or inefficient memory management".to_string());
                    causes.push("Increased data structure sizes or new caching".to_string());
                    causes.push("Changes in memory allocation patterns".to_string());
                }
            }
            "cache_hit_rate" => {
                if change_percent < -10.0 {
                    causes.push("Cache invalidation logic changes".to_string());
                    causes.push("Increased data variability reducing cache effectiveness".to_string());
                    causes.push("Cache size limitations or eviction policy changes".to_string());
                }
            }
            "throughput_operations_per_second" => {
                if change_percent < -10.0 {
                    causes.push("Bottlenecks in processing pipeline".to_string());
                    causes.push("Increased serialization or I/O overhead".to_string());
                    causes.push("Reduced parallelization or concurrency".to_string());
                }
            }
            "error_rate" => {
                if change_percent > 5.0 {
                    causes.push("New error conditions or reduced error handling".to_string());
                    causes.push("Environmental changes affecting reliability".to_string());
                    causes.push("Edge cases not handled in recent changes".to_string());
                }
            }
            _ => {}
        }

        if causes.is_empty() {
            causes.push("Performance change detected - investigate recent code changes".to_string());
        }

        causes
    }

    /// Generate performance summary
    fn generate_performance_summary(&self, baseline: &PerformanceMetrics, current: &PerformanceMetrics) -> PerformanceSummary {
        let mut metric_changes = HashMap::new();
        
        // Calculate all metric changes
        if baseline.pipeline_latency_ms > 0.0 {
            metric_changes.insert("pipeline_latency_ms", (current.pipeline_latency_ms - baseline.pipeline_latency_ms) / baseline.pipeline_latency_ms * 100.0);
        }
        if baseline.memory_usage_mb > 0.0 {
            metric_changes.insert("memory_usage_mb", (current.memory_usage_mb - baseline.memory_usage_mb) / baseline.memory_usage_mb * 100.0);
        }
        if baseline.cache_hit_rate > 0.0 {
            metric_changes.insert("cache_hit_rate", (current.cache_hit_rate - baseline.cache_hit_rate) / baseline.cache_hit_rate * 100.0);
        }
        if baseline.throughput_operations_per_second > 0.0 {
            metric_changes.insert("throughput_operations_per_second", (current.throughput_operations_per_second - baseline.throughput_operations_per_second) / baseline.throughput_operations_per_second * 100.0);
        }
        if baseline.error_rate >= 0.0 {
            metric_changes.insert("error_rate", (current.error_rate - baseline.error_rate) / (baseline.error_rate + 0.01) * 100.0); // Add small epsilon to avoid division by zero
        }

        // Find best and worst performing metrics
        let mut sorted_changes: Vec<_> = metric_changes.iter().collect();
        sorted_changes.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));

        let best_performing_metrics = sorted_changes.iter()
            .take(2)
            .map(|(name, _)| name.to_string())
            .collect();

        let worst_performing_metrics = sorted_changes.iter()
            .rev()
            .take(2)
            .map(|(name, _)| name.to_string())
            .collect();

        // Calculate overall performance change
        let overall_performance_change = metric_changes.values().sum::<f64>() / metric_changes.len() as f64;

        // Calculate stability and reliability scores
        let stability_score = self.calculate_stability_score(&metric_changes);
        let reliability_score = self.calculate_reliability_score(current);

        PerformanceSummary {
            overall_performance_change,
            best_performing_metrics,
            worst_performing_metrics,
            stability_score,
            reliability_score,
        }
    }

    /// Calculate stability score based on metric variance
    fn calculate_stability_score(&self, metric_changes: &HashMap<&str, f64>) -> f64 {
        if metric_changes.is_empty() {
            return 1.0;
        }

        let variance: f64 = metric_changes.values()
            .map(|&change| change.abs())
            .map(|abs_change| abs_change.powi(2))
            .sum::<f64>() / metric_changes.len() as f64;

        // Convert variance to stability score (0-1, higher is more stable)
        (1.0 / (1.0 + variance / 100.0)).max(0.0).min(1.0)
    }

    /// Calculate reliability score based on error rate and other factors
    fn calculate_reliability_score(&self, metrics: &PerformanceMetrics) -> f64 {
        // Simple reliability calculation based on error rate
        (1.0 - metrics.error_rate / 100.0).max(0.0).min(1.0)
    }

    /// Perform statistical analysis
    async fn perform_statistical_analysis(
        &self,
        baseline: &PerformanceMeasurement,
        current: &PerformanceMeasurement,
    ) -> Result<StatisticalAnalysis> {
        // Simplified statistical analysis - in production, implement robust statistical methods
        
        let confidence_interval = (baseline.metrics.pipeline_latency_ms * 0.95, baseline.metrics.pipeline_latency_ms * 1.05);
        let p_value = 0.05; // Placeholder
        let effect_size = (current.metrics.pipeline_latency_ms - baseline.metrics.pipeline_latency_ms).abs() / baseline.metrics.pipeline_latency_ms;
        let statistical_power = 0.80; // Placeholder
        let outliers_detected = Vec::new(); // Placeholder

        Ok(StatisticalAnalysis {
            confidence_interval,
            p_value,
            effect_size,
            statistical_power,
            outliers_detected,
        })
    }

    /// Generate actionable recommendations
    fn generate_recommendations(&self, regressions: &[DetectedRegression]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if regressions.is_empty() {
            recommendations.push("No performance regressions detected. Continue monitoring.".to_string());
            return recommendations;
        }

        // Critical regressions
        let critical_count = regressions.iter().filter(|r| r.severity == RegressionSeverity::Critical).count();
        if critical_count > 0 {
            recommendations.push(format!(
                "URGENT: {} critical performance regressions detected. Immediate investigation required.",
                critical_count
            ));
        }

        // Major regressions
        let major_count = regressions.iter().filter(|r| r.severity == RegressionSeverity::Major).count();
        if major_count > 0 {
            recommendations.push(format!(
                "WARNING: {} major performance regressions detected. Schedule investigation within 24 hours.",
                major_count
            ));
        }

        // Specific metric recommendations
        for regression in regressions {
            match regression.metric_name.as_str() {
                "pipeline_latency_ms" => {
                    recommendations.push("Pipeline latency regression: Profile recent changes for algorithmic complexity increases.".to_string());
                }
                "memory_usage_mb" => {
                    recommendations.push("Memory usage regression: Check for memory leaks and optimize data structures.".to_string());
                }
                "cache_hit_rate" => {
                    recommendations.push("Cache performance regression: Review cache invalidation logic and size limits.".to_string());
                }
                "throughput_operations_per_second" => {
                    recommendations.push("Throughput regression: Investigate bottlenecks and parallelization opportunities.".to_string());
                }
                "error_rate" => {
                    recommendations.push("Error rate regression: Review error handling and edge case coverage.".to_string());
                }
                _ => {}
            }
        }

        recommendations.push("Consider running additional targeted benchmarks to isolate the regression source.".to_string());
        recommendations.push("Review recent code changes and their potential performance impact.".to_string());

        recommendations
    }

    /// Check if baseline should be updated
    fn should_update_baseline(&self, current: &PerformanceMeasurement, baseline: &PerformanceMeasurement) -> bool {
        // Update baseline if:
        // 1. Current measurement is significantly better across multiple metrics
        // 2. Enough time has passed since last baseline update
        // 3. Current measurement is stable (low variance)

        let time_since_baseline = current.timestamp.signed_duration_since(baseline.timestamp);
        let days_since_baseline = time_since_baseline.num_days() as u64;

        if days_since_baseline >= self.config.monitoring_intervals.baseline_update_days {
            return true;
        }

        // Check if current is significantly better
        let improvements = [
            current.metrics.pipeline_latency_ms < baseline.metrics.pipeline_latency_ms * 0.95,
            current.metrics.memory_usage_mb < baseline.metrics.memory_usage_mb * 0.95,
            current.metrics.cache_hit_rate > baseline.metrics.cache_hit_rate * 1.05,
            current.metrics.throughput_operations_per_second > baseline.metrics.throughput_operations_per_second * 1.05,
            current.metrics.error_rate < baseline.metrics.error_rate * 0.95,
        ];

        improvements.iter().filter(|&&improved| improved).count() >= 3
    }

    /// Collect environment information
    fn collect_environment_info(&self) -> Result<EnvironmentInfo> {
        Ok(EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            cpu_model: "Unknown".to_string(), // In production, get actual CPU info
            cpu_cores: num_cpus::get() as u32,
            memory_gb: 16.0, // In production, get actual memory info
            rust_version: "1.70.0".to_string(), // In production, get actual version
            build_mode: if cfg!(debug_assertions) { "debug" } else { "release" }.to_string(),
            compiler_flags: vec!["--release".to_string()],
        })
    }

    /// Get current git commit
    fn get_git_commit(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .context("Failed to get git commit")?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Get current git branch
    fn get_git_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get git branch")?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
}

// Implementation for helper structs

impl BaselineStorage {
    fn new(storage_path: &Path) -> Result<Self> {
        std::fs::create_dir_all(storage_path)?;
        
        let mut storage = Self {
            storage_path: storage_path.to_path_buf(),
            current_baseline: None,
            baseline_history: Vec::new(),
        };
        
        // Load existing baseline if available
        storage.load_current_baseline()?;
        
        Ok(storage)
    }

    fn load_current_baseline(&mut self) -> Result<()> {
        let baseline_file = self.storage_path.join("current_baseline.json");
        if baseline_file.exists() {
            let content = std::fs::read_to_string(&baseline_file)?;
            self.current_baseline = Some(serde_json::from_str(&content)?);
        }
        Ok(())
    }

    fn get_current_baseline(&self) -> Option<&PerformanceMeasurement> {
        self.current_baseline.as_ref()
    }

    fn update_baseline(&mut self, measurement: PerformanceMeasurement) -> Result<()> {
        // Save previous baseline to history
        if let Some(previous) = &self.current_baseline {
            self.baseline_history.push(previous.clone());
        }

        // Update current baseline
        self.current_baseline = Some(measurement.clone());

        // Save to disk
        let baseline_file = self.storage_path.join("current_baseline.json");
        let content = serde_json::to_string_pretty(&measurement)?;
        std::fs::write(&baseline_file, content)?;

        println!("📊 Baseline updated to commit: {}", measurement.git_commit);
        Ok(())
    }
}

impl PerformanceHistory {
    fn new(max_size: usize) -> Self {
        Self {
            measurements: VecDeque::with_capacity(max_size),
            max_history_size: max_size,
            trend_cache: HashMap::new(),
        }
    }

    fn add_measurement(&mut self, measurement: PerformanceMeasurement) {
        if self.measurements.len() >= self.max_history_size {
            self.measurements.pop_front();
        }
        self.measurements.push_back(measurement);
        
        // Invalidate trend cache
        self.trend_cache.clear();
    }

    fn get_trend_analysis(&mut self, metric_name: &str) -> TrendAnalysis {
        if let Some(cached) = self.trend_cache.get(metric_name) {
            return cached.clone();
        }

        let trend = self.calculate_trend(metric_name);
        self.trend_cache.insert(metric_name.to_string(), trend.clone());
        trend
    }

    fn calculate_trend(&self, metric_name: &str) -> TrendAnalysis {
        let values = self.get_metric_history(metric_name);
        
        if values.len() < 3 {
            return TrendAnalysis {
                trend_direction: TrendDirection::Stable,
                trend_strength: 0.0,
                trend_duration_days: 0,
                forecast_next_week: values.last().copied().unwrap_or(0.0),
            };
        }

        // Simple linear regression for trend analysis
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        let slope = if denominator != 0.0 { numerator / denominator } else { 0.0 };
        
        let trend_direction = match slope {
            s if s > 0.1 => TrendDirection::Improving,
            s if s < -0.1 => TrendDirection::Degrading,
            _ => TrendDirection::Stable,
        };

        let trend_strength = slope.abs();
        let forecast_next_week = values.last().unwrap_or(&0.0) + slope * 7.0;

        TrendAnalysis {
            trend_direction,
            trend_strength,
            trend_duration_days: values.len() as u64,
            forecast_next_week,
        }
    }

    fn get_metric_history(&self, metric_name: &str) -> Vec<f64> {
        self.measurements.iter()
            .map(|m| match metric_name {
                "pipeline_latency_ms" => m.metrics.pipeline_latency_ms,
                "memory_usage_mb" => m.metrics.memory_usage_mb,
                "cache_hit_rate" => m.metrics.cache_hit_rate,
                "throughput_operations_per_second" => m.metrics.throughput_operations_per_second,
                "error_rate" => m.metrics.error_rate,
                _ => 0.0,
            })
            .collect()
    }
}

impl AlertSystem {
    fn new(config: AlertSettings) -> Self {
        Self {
            config,
            last_alert_times: HashMap::new(),
        }
    }

    async fn send_alerts(&mut self, result: &RegressionResult) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check if we should send alerts (cooldown period)
        let now = Utc::now();
        let cooldown = chrono::Duration::minutes(self.config.monitoring_intervals.alert_cooldown_minutes as i64);

        // Filter regressions by severity
        let filtered_regressions: Vec<_> = result.regressions_detected.iter()
            .filter(|r| self.config.severity_filters.contains(&r.severity))
            .collect();

        if filtered_regressions.is_empty() {
            return Ok();
        }

        for regression in &filtered_regressions {
            let alert_key = format!("{}-{:?}", regression.metric_name, regression.severity);
            
            if let Some(last_alert) = self.last_alert_times.get(&alert_key) {
                if now.signed_duration_since(*last_alert) < cooldown {
                    continue; // Skip this alert due to cooldown
                }
            }

            // Send alert
            self.send_alert_notification(regression, result).await?;
            self.last_alert_times.insert(alert_key, now);
        }

        Ok(())
    }

    async fn send_alert_notification(&self, regression: &DetectedRegression, result: &RegressionResult) -> Result<()> {
        let alert_message = format!(
            "🚨 Performance Regression Detected!\n\
            Metric: {}\n\
            Change: {:.2}% (from {:.2} to {:.2})\n\
            Severity: {:?}\n\
            Commit: {}\n\
            Statistical Significance: {:.2}%\n\
            Trend: {:?}",
            regression.metric_name,
            regression.change_percent,
            regression.baseline_value,
            regression.current_value,
            regression.severity,
            result.current_commit,
            regression.statistical_significance * 100.0,
            regression.trend_analysis.trend_direction
        );

        println!("📧 Alert: {}", alert_message);

        // In production, implement actual notification sending:
        // - Email notifications
        // - Slack messages
        // - GitHub issue creation
        // - etc.

        Ok(())
    }
}

impl ReportGenerator {
    fn new() -> Result<Self> {
        let template_engine = ReportTemplateEngine::new()?;
        let output_formats = vec![OutputFormat::Html, OutputFormat::Markdown, OutputFormat::Json];

        Ok(Self {
            template_engine,
            output_formats,
        })
    }

    async fn generate_reports(&self, result: &RegressionResult) -> Result<()> {
        for format in &self.output_formats {
            self.generate_report(result, format).await?;
        }
        Ok(())
    }

    async fn generate_report(&self, result: &RegressionResult, format: &OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => {
                let json_content = serde_json::to_string_pretty(result)?;
                let filename = format!("regression_report_{}.json", result.timestamp.format("%Y%m%d_%H%M%S"));
                std::fs::write(&filename, json_content)?;
                println!("📄 Generated JSON report: {}", filename);
            }
            OutputFormat::Markdown => {
                let markdown_content = self.generate_markdown_report(result);
                let filename = format!("regression_report_{}.md", result.timestamp.format("%Y%m%d_%H%M%S"));
                std::fs::write(&filename, markdown_content)?;
                println!("📄 Generated Markdown report: {}", filename);
            }
            OutputFormat::Html => {
                let html_content = self.generate_html_report(result);
                let filename = format!("regression_report_{}.html", result.timestamp.format("%Y%m%d_%H%M%S"));
                std::fs::write(&filename, html_content)?;
                println!("📄 Generated HTML report: {}", filename);
            }
            _ => {
                println!("📄 Report format {:?} not yet implemented", format);
            }
        }
        Ok(())
    }

    fn generate_markdown_report(&self, result: &RegressionResult) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("# Performance Regression Report\n\n"));
        report.push_str(&format!("**Generated:** {}\n", result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("**Baseline Commit:** {}\n", result.baseline_commit));
        report.push_str(&format!("**Current Commit:** {}\n", result.current_commit));
        report.push_str(&format!("**Overall Severity:** {:?}\n\n", result.overall_severity));

        if result.regressions_detected.is_empty() {
            report.push_str("## ✅ No Regressions Detected\n\n");
            report.push_str("All performance metrics are within acceptable ranges.\n\n");
        } else {
            report.push_str(&format!("## ⚠️ Detected Regressions ({})\n\n", result.regressions_detected.len()));
            
            for regression in &result.regressions_detected {
                report.push_str(&format!("### {} ({:?})\n\n", regression.metric_name, regression.severity));
                report.push_str(&format!("- **Change:** {:.2}% (from {:.2} to {:.2})\n", 
                    regression.change_percent, regression.baseline_value, regression.current_value));
                report.push_str(&format!("- **Statistical Significance:** {:.2}%\n", 
                    regression.statistical_significance * 100.0));
                report.push_str(&format!("- **Trend:** {:?} (strength: {:.2})\n", 
                    regression.trend_analysis.trend_direction, regression.trend_analysis.trend_strength));
                
                if !regression.potential_causes.is_empty() {
                    report.push_str("- **Potential Causes:**\n");
                    for cause in &regression.potential_causes {
                        report.push_str(&format!("  - {}\n", cause));
                    }
                }
                report.push_str("\n");
            }
        }

        report.push_str("## 📊 Performance Summary\n\n");
        report.push_str(&format!("- **Overall Performance Change:** {:.2}%\n", result.performance_summary.overall_performance_change));
        report.push_str(&format!("- **Stability Score:** {:.2}\n", result.performance_summary.stability_score));
        report.push_str(&format!("- **Reliability Score:** {:.2}\n\n", result.performance_summary.reliability_score));

        if !result.recommendations.is_empty() {
            report.push_str("## 🎯 Recommendations\n\n");
            for (i, recommendation) in result.recommendations.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, recommendation));
            }
        }

        report
    }

    fn generate_html_report(&self, result: &RegressionResult) -> String {
        // Simple HTML report - in production, use proper templating
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Performance Regression Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .severity-critical {{ color: #dc3545; }}
        .severity-major {{ color: #fd7e14; }}
        .severity-minor {{ color: #ffc107; }}
        .severity-none {{ color: #28a745; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>Performance Regression Report</h1>
    <p><strong>Generated:</strong> {}</p>
    <p><strong>Baseline Commit:</strong> {}</p>
    <p><strong>Current Commit:</strong> {}</p>
    <p><strong>Overall Severity:</strong> <span class="severity-{:?}">{:?}</span></p>
    
    <h2>Detected Regressions</h2>
    {}
    
    <h2>Performance Summary</h2>
    <p><strong>Overall Performance Change:</strong> {:.2}%</p>
    <p><strong>Stability Score:</strong> {:.2}</p>
    <p><strong>Reliability Score:</strong> {:.2}</p>
    
    <h2>Recommendations</h2>
    <ol>
        {}
    </ol>
</body>
</html>"#,
            result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            result.baseline_commit,
            result.current_commit,
            result.overall_severity,
            result.overall_severity,
            if result.regressions_detected.is_empty() {
                "<p>✅ No regressions detected.</p>".to_string()
            } else {
                result.regressions_detected.iter()
                    .map(|r| format!("<p><strong>{}:</strong> {:.2}% change ({:?})</p>", 
                        r.metric_name, r.change_percent, r.severity))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            result.performance_summary.overall_performance_change,
            result.performance_summary.stability_score,
            result.performance_summary.reliability_score,
            result.recommendations.iter()
                .map(|r| format!("<li>{}</li>", r))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl ReportTemplateEngine {
    fn new() -> Result<Self> {
        Ok(Self {
            templates: HashMap::new(),
        })
    }
}

// Default configurations
impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            baseline_path: PathBuf::from("./performance_baselines"),
            benchmark_command: "cargo bench --bench enterprise_performance".to_string(),
            regression_thresholds: RegressionThresholds {
                pipeline_latency_threshold_percent: 10.0,
                memory_usage_threshold_percent: 15.0,
                cache_hit_rate_threshold_percent: 5.0,
                throughput_threshold_percent: 10.0,
                error_rate_threshold_percent: 1.0,
                critical_threshold_percent: 30.0,
                major_threshold_percent: 20.0,
                minor_threshold_percent: 10.0,
            },
            monitoring_intervals: MonitoringIntervals {
                continuous_monitoring_seconds: 3600, // 1 hour
                baseline_update_days: 7,
                history_retention_days: 90,
                alert_cooldown_minutes: 30,
            },
            alert_settings: AlertSettings {
                enabled: true,
                email_notifications: vec!["team@example.com".to_string()],
                slack_webhook: None,
                github_issue_creation: false,
                severity_filters: vec![
                    RegressionSeverity::Critical,
                    RegressionSeverity::Major,
                ],
            },
            ci_integration: CiIntegrationConfig {
                enabled: true,
                fail_build_on_critical: true,
                fail_build_on_major: false,
                comment_on_pr: true,
                create_performance_badge: true,
            },
            statistical_config: StatisticalConfig {
                confidence_level: 0.95,
                minimum_samples: 10,
                outlier_detection_method: OutlierDetectionMethod::StandardDeviation,
                trend_analysis_window: 30,
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = RegressionConfig::default();
    let mut detector = RegressionDetector::new(config)?;
    
    println!("🚀 Starting regression detection system...");
    
    // Run single detection
    match detector.detect_regressions().await {
        Ok(result) => {
            println!("✅ Regression detection completed successfully");
            println!("   Regressions detected: {}", result.regressions_detected.len());
            println!("   Overall severity: {:?}", result.overall_severity);
        }
        Err(e) => {
            eprintln!("❌ Regression detection failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}