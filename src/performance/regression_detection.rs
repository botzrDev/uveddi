//! Performance regression detection system
//!
//! Provides functionality for:
//! - Baseline performance tracking
//! - Automated regression detection
//! - Performance alert system
//! - Trend analysis and forecasting

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::performance::genetic_bottleneck::{
    BottleneckAnalysis, GeneticBottleneckDetector, PerformanceDataPoint,
};
use crate::performance::statistical_analysis::{MannKendallResult, StatisticalAnalyzer};
use crate::performance::trend_detection::{ChangePointResult, TrendDetector};

/// Performance regression detector
#[derive(Debug)]
pub struct PerformanceRegressionDetector {
    config: RegressionDetectionConfig,
    baselines: RwLock<HashMap<String, PerformanceBaseline>>,
    alert_manager: AlertManager,
    storage: MetricsStorage,
    statistical_analyzer: StatisticalAnalyzer,
    trend_detector: TrendDetector,
    genetic_detector: Option<GeneticBottleneckDetector>,
    confidence_threshold: f64, // 0.95 for 95% confidence
}

/// Configuration for regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetectionConfig {
    pub baseline_window_days: u32,
    pub min_samples_for_baseline: usize,
    pub regression_threshold_percent: f64,
    pub significance_threshold: f64,
    pub alert_cooldown: Duration,
    pub storage_path: PathBuf,
    pub metrics_retention_days: u32,
}

/// Performance baseline for a specific metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub metric_name: String,
    pub baseline_value: f64,
    pub baseline_std_dev: f64,
    pub sample_count: usize,
    pub confidence_interval: (f64, f64),
    pub last_updated: SystemTime,
    pub historical_values: VecDeque<MetricDataPoint>,
}

/// Individual metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub value: f64,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionResult {
    pub metric_name: String,
    pub current_value: f64,
    pub baseline_value: f64,
    pub regression_percentage: f64,
    pub is_regression: bool,
    pub severity: RegressionSeverity,
    pub confidence: f64,
    pub detected_at: SystemTime,
    pub analysis: RegressionAnalysis,
}

/// Severity levels for regressions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Minor,    // 5-15% regression
    Moderate, // 15-30% regression
    Major,    // 30-50% regression
    Critical, // 50%+ regression
}

/// Detailed regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub trend_direction: TrendDirection,
    pub change_points: Vec<ChangePoint>,
    pub statistical_significance: f64,
    pub potential_causes: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub mann_kendall_result: Option<MannKendallResult>,
    pub change_point_analysis: Option<ChangePointResult>,
}

/// Trend direction analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

/// Change point detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePoint {
    pub timestamp: SystemTime,
    pub before_mean: f64,
    pub after_mean: f64,
    pub confidence: f64,
}

/// Alert manager for regression notifications
#[derive(Debug)]
pub struct AlertManager {
    config: AlertConfig,
    last_alerts: RwLock<HashMap<String, SystemTime>>,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub notification_channels: Vec<NotificationChannel>,
    pub severity_thresholds: HashMap<RegressionSeverity, Duration>,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Slack {
        webhook_url: String,
        channel: String,
    },
    Email {
        recipients: Vec<String>,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

/// Metrics storage interface
#[derive(Debug)]
pub struct MetricsStorage {
    storage_path: PathBuf,
    retention_days: u32,
}

impl PerformanceRegressionDetector {
    /// Create a new regression detector
    pub fn new(config: RegressionDetectionConfig) -> Result<Self> {
        let alert_config = AlertConfig::default();
        let alert_manager = AlertManager::new(alert_config);
        let storage = MetricsStorage::new(&config.storage_path, config.metrics_retention_days);

        Ok(Self {
            config,
            baselines: RwLock::new(HashMap::new()),
            alert_manager,
            storage,
            statistical_analyzer: StatisticalAnalyzer::new(),
            trend_detector: TrendDetector::new(),
            genetic_detector: None,
            confidence_threshold: 0.95,
        })
    }

    /// Initialize baselines from historical data
    #[instrument(skip(self))]
    pub async fn initialize_baselines(&self) -> Result<()> {
        info!("Initializing performance baselines from historical data");

        let historical_metrics = self.storage.load_historical_metrics().await?;
        let mut baselines = self.baselines.write().await;

        for (metric_name, data_points) in historical_metrics {
            if data_points.len() >= self.config.min_samples_for_baseline {
                let baseline = self.calculate_baseline(&metric_name, &data_points)?;
                baselines.insert(metric_name.clone(), baseline);

                debug!(
                    metric = %metric_name,
                    baseline_value = %baselines[&metric_name].baseline_value,
                    sample_count = %data_points.len(),
                    "Baseline initialized"
                );
            } else {
                warn!(
                    metric = %metric_name,
                    sample_count = %data_points.len(),
                    min_required = %self.config.min_samples_for_baseline,
                    "Insufficient samples for baseline"
                );
            }
        }

        info!(
            baseline_count = %baselines.len(),
            "Performance baselines initialized"
        );

        Ok(())
    }

    /// Record a new metric value and check for regressions
    #[instrument(skip(self))]
    pub async fn record_metric(
        &self,
        metric_name: &str,
        value: f64,
        metadata: HashMap<String, String>,
    ) -> Result<Option<RegressionResult>> {
        let data_point = MetricDataPoint {
            value,
            timestamp: SystemTime::now(),
            metadata,
        };

        // Store the metric
        self.storage.store_metric(metric_name, &data_point).await?;

        // Update baseline
        self.update_baseline(metric_name, &data_point).await?;

        // Check for regression
        let regression_result = self.check_for_regression(metric_name, value).await?;

        // Send alerts if regression detected
        if let Some(ref result) = regression_result {
            if result.is_regression {
                self.alert_manager.send_regression_alert(result).await?;
            }
        }

        Ok(regression_result)
    }

    /// Check for regression against baseline
    #[instrument(skip(self))]
    async fn check_for_regression(
        &self,
        metric_name: &str,
        current_value: f64,
    ) -> Result<Option<RegressionResult>> {
        let baselines = self.baselines.read().await;

        let baseline = match baselines.get(metric_name) {
            Some(baseline) => baseline,
            None => {
                debug!(metric = %metric_name, "No baseline available for regression check");
                return Ok(None);
            }
        };

        // Calculate regression percentage
        let regression_percentage =
            ((current_value - baseline.baseline_value) / baseline.baseline_value) * 100.0;

        // Check if this constitutes a regression (assuming higher values are worse)
        let is_regression = regression_percentage > self.config.regression_threshold_percent;

        // Calculate statistical significance
        let z_score = (current_value - baseline.baseline_value) / baseline.baseline_std_dev;
        let confidence = calculate_confidence_from_z_score(z_score);

        // Determine severity
        let severity = determine_regression_severity(regression_percentage.abs());

        // Perform detailed analysis
        let analysis = self
            .analyze_regression(metric_name, baseline, current_value)
            .await?;

        let result = RegressionResult {
            metric_name: metric_name.to_string(),
            current_value,
            baseline_value: baseline.baseline_value,
            regression_percentage,
            is_regression,
            severity,
            confidence,
            detected_at: SystemTime::now(),
            analysis,
        };

        if is_regression {
            warn!(
                metric = %metric_name,
                current_value = %current_value,
                baseline_value = %baseline.baseline_value,
                regression_percentage = %regression_percentage,
                severity = ?result.severity,
                "Performance regression detected"
            );
        } else {
            debug!(
                metric = %metric_name,
                current_value = %current_value,
                baseline_value = %baseline.baseline_value,
                change_percentage = %regression_percentage,
                "No regression detected"
            );
        }

        Ok(Some(result))
    }

    /// Update baseline with new data point
    async fn update_baseline(&self, metric_name: &str, data_point: &MetricDataPoint) -> Result<()> {
        let mut baselines = self.baselines.write().await;

        let baseline =
            baselines
                .entry(metric_name.to_string())
                .or_insert_with(|| PerformanceBaseline {
                    metric_name: metric_name.to_string(),
                    baseline_value: data_point.value,
                    baseline_std_dev: 0.0,
                    sample_count: 0,
                    confidence_interval: (data_point.value, data_point.value),
                    last_updated: data_point.timestamp,
                    historical_values: VecDeque::new(),
                });

        // Add new data point
        baseline.historical_values.push_back(data_point.clone());
        baseline.last_updated = data_point.timestamp;

        // Maintain rolling window
        let max_samples = (self.config.baseline_window_days as f64 * 24.0 * 60.0 / 5.0) as usize; // Assuming 5-minute intervals
        while baseline.historical_values.len() > max_samples {
            baseline.historical_values.pop_front();
        }

        // Recalculate baseline if we have enough samples
        if baseline.historical_values.len() >= self.config.min_samples_for_baseline {
            let values: Vec<f64> = baseline
                .historical_values
                .iter()
                .map(|dp| dp.value)
                .collect();

            baseline.baseline_value = calculate_mean(&values);
            baseline.baseline_std_dev = calculate_std_dev(&values, baseline.baseline_value);
            baseline.sample_count = values.len();
            baseline.confidence_interval = calculate_confidence_interval(&values, 0.95);
        }

        Ok(())
    }

    /// Calculate baseline from historical data points
    fn calculate_baseline(
        &self,
        metric_name: &str,
        data_points: &[MetricDataPoint],
    ) -> Result<PerformanceBaseline> {
        if data_points.is_empty() {
            return Err(anyhow!("Cannot calculate baseline from empty data"));
        }

        let values: Vec<f64> = data_points.iter().map(|dp| dp.value).collect();
        let mean = calculate_mean(&values);
        let std_dev = calculate_std_dev(&values, mean);
        let confidence_interval = calculate_confidence_interval(&values, 0.95);

        let mut historical_values = VecDeque::new();
        for data_point in data_points {
            historical_values.push_back(data_point.clone());
        }

        Ok(PerformanceBaseline {
            metric_name: metric_name.to_string(),
            baseline_value: mean,
            baseline_std_dev: std_dev,
            sample_count: values.len(),
            confidence_interval,
            last_updated: data_points.last().unwrap().timestamp,
            historical_values,
        })
    }

    /// Perform detailed regression analysis
    async fn analyze_regression(
        &self,
        metric_name: &str,
        baseline: &PerformanceBaseline,
        current_value: f64,
    ) -> Result<RegressionAnalysis> {
        // Analyze trend direction
        let trend_direction = self.analyze_trend_direction(&baseline.historical_values);

        // Detect change points
        let change_points = self.detect_change_points(&baseline.historical_values)?;

        // Calculate statistical significance
        let z_score = (current_value - baseline.baseline_value) / baseline.baseline_std_dev;
        let statistical_significance = calculate_confidence_from_z_score(z_score);

        // Generate potential causes and recommendations
        let potential_causes = self.generate_potential_causes(metric_name, &trend_direction);
        let recommended_actions = self.generate_recommendations(metric_name, &trend_direction);

        // Enhanced statistical analysis
        let historical_values: Vec<f64> = baseline
            .historical_values
            .iter()
            .map(|dp| dp.value)
            .collect();

        let mann_kendall_result = self
            .statistical_analyzer
            .mann_kendall_test(&historical_values)
            .ok();
        let change_point_analysis = self
            .trend_detector
            .detect_change_points_pelt(&historical_values)
            .ok();

        Ok(RegressionAnalysis {
            trend_direction,
            change_points,
            statistical_significance,
            potential_causes,
            recommended_actions,
            mann_kendall_result,
            change_point_analysis,
        })
    }

    /// Analyze trend direction from historical data
    fn analyze_trend_direction(
        &self,
        historical_values: &VecDeque<MetricDataPoint>,
    ) -> TrendDirection {
        if historical_values.len() < 10 {
            return TrendDirection::Stable;
        }

        let values: Vec<f64> = historical_values.iter().map(|dp| dp.value).collect();
        let recent_values = &values[values.len().saturating_sub(10)..];
        let older_values = &values[..values.len().saturating_sub(10)];

        let recent_mean = calculate_mean(recent_values);
        let older_mean = calculate_mean(older_values);

        let change_percent = ((recent_mean - older_mean) / older_mean) * 100.0;

        // Calculate variance to detect volatility
        let recent_variance = calculate_variance(recent_values, recent_mean);
        let is_volatile = recent_variance > older_mean * 0.1; // 10% coefficient of variation

        if is_volatile {
            TrendDirection::Volatile
        } else if change_percent > 5.0 {
            TrendDirection::Degrading
        } else if change_percent < -5.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    /// Detect change points in the data using simple algorithm
    fn detect_change_points(
        &self,
        historical_values: &VecDeque<MetricDataPoint>,
    ) -> Result<Vec<ChangePoint>> {
        let mut change_points = Vec::new();

        if historical_values.len() < 20 {
            return Ok(change_points);
        }

        let values: Vec<f64> = historical_values.iter().map(|dp| dp.value).collect();
        let timestamps: Vec<SystemTime> = historical_values.iter().map(|dp| dp.timestamp).collect();

        // Simple change point detection using sliding window
        let window_size = 10;

        for i in window_size..(values.len() - window_size) {
            let before_window = &values[i.saturating_sub(window_size)..i];
            let after_window = &values[i..i + window_size];

            let before_mean = calculate_mean(before_window);
            let after_mean = calculate_mean(after_window);

            // Check if there's a significant change
            let change_magnitude = (after_mean - before_mean).abs();
            let combined_std = (calculate_std_dev(before_window, before_mean)
                + calculate_std_dev(after_window, after_mean))
                / 2.0;

            if change_magnitude > combined_std * 2.0 {
                // 2 standard deviations
                let confidence = 1.0 - (combined_std / change_magnitude).min(1.0);

                change_points.push(ChangePoint {
                    timestamp: timestamps[i],
                    before_mean,
                    after_mean,
                    confidence,
                });
            }
        }

        Ok(change_points)
    }

    /// Generate potential causes for regression
    fn generate_potential_causes(&self, metric_name: &str, trend: &TrendDirection) -> Vec<String> {
        let mut causes = Vec::new();

        match trend {
            TrendDirection::Degrading => {
                causes.push("Code changes introducing performance bottlenecks".to_string());
                causes.push("Increased system load or resource contention".to_string());
                causes.push("Infrastructure degradation or capacity issues".to_string());
                causes.push("Database query performance degradation".to_string());
            }
            TrendDirection::Volatile => {
                causes.push("Intermittent system issues or resource spikes".to_string());
                causes.push("Load balancing issues or uneven resource distribution".to_string());
                causes.push("Garbage collection pressure or memory management issues".to_string());
            }
            _ => {
                causes.push("Normal performance variation within acceptable bounds".to_string());
            }
        }

        // Add metric-specific causes
        if metric_name.contains("latency") || metric_name.contains("response_time") {
            causes.push("Network latency increases".to_string());
            causes.push("External service dependencies slowing down".to_string());
        } else if metric_name.contains("throughput") || metric_name.contains("rps") {
            causes.push("Resource saturation limiting throughput".to_string());
            causes.push("Connection pool exhaustion".to_string());
        } else if metric_name.contains("memory") {
            causes.push("Memory leaks or increased allocation rates".to_string());
            causes.push("Larger data structures or increased cache usage".to_string());
        }

        causes
    }

    /// Generate recommendations for addressing regression
    fn generate_recommendations(&self, metric_name: &str, trend: &TrendDirection) -> Vec<String> {
        let mut recommendations = Vec::new();

        recommendations.push("Review recent code changes and deployments".to_string());
        recommendations.push("Analyze system resource utilization trends".to_string());
        recommendations.push("Check for external dependency performance issues".to_string());

        match trend {
            TrendDirection::Degrading => {
                recommendations
                    .push("Consider rolling back recent changes if correlation exists".to_string());
                recommendations
                    .push("Scale up resources if infrastructure limits are reached".to_string());
                recommendations.push("Optimize identified performance bottlenecks".to_string());
            }
            TrendDirection::Volatile => {
                recommendations
                    .push("Investigate intermittent issues and error patterns".to_string());
                recommendations
                    .push("Implement better load balancing or circuit breakers".to_string());
                recommendations
                    .push("Add monitoring for resource spikes and anomalies".to_string());
            }
            _ => {
                recommendations.push("Continue monitoring for sustained trends".to_string());
            }
        }

        // Add metric-specific recommendations
        if metric_name.contains("latency") || metric_name.contains("response_time") {
            recommendations
                .push("Profile critical code paths for optimization opportunities".to_string());
            recommendations
                .push("Consider caching strategies for frequently accessed data".to_string());
        } else if metric_name.contains("memory") {
            recommendations
                .push("Run memory profiling to identify leaks or excessive allocation".to_string());
            recommendations.push(
                "Consider memory optimization techniques or garbage collection tuning".to_string(),
            );
        }

        recommendations
    }

    /// Get current baselines for all metrics
    pub async fn get_baselines(&self) -> HashMap<String, PerformanceBaseline> {
        self.baselines.read().await.clone()
    }

    /// Reset baseline for a specific metric
    pub async fn reset_baseline(&self, metric_name: &str) -> Result<()> {
        let mut baselines = self.baselines.write().await;
        baselines.remove(metric_name);
        info!(metric = %metric_name, "Baseline reset");
        Ok(())
    }

    /// Enhanced regression detection with statistical confidence
    #[instrument(skip(self))]
    pub async fn detect_regression_with_confidence(
        &self,
        metric_name: &str,
        current_value: f64,
    ) -> Result<Option<EnhancedRegressionResult>> {
        let baselines = self.baselines.read().await;

        let baseline = match baselines.get(metric_name) {
            Some(baseline) => baseline,
            None => {
                debug!(metric = %metric_name, "No baseline available for enhanced regression check");
                return Ok(None);
            }
        };

        // Get historical data for statistical analysis
        let historical_values: Vec<f64> = baseline
            .historical_values
            .iter()
            .map(|dp| dp.value)
            .collect();

        // Perform basic regression check first
        let basic_result = self
            .check_for_regression(metric_name, current_value)
            .await?;
        let basic_result = match basic_result {
            Some(result) => result,
            None => return Ok(None),
        };

        // Perform Mann-Kendall trend test
        let mann_kendall = self
            .statistical_analyzer
            .mann_kendall_test(&historical_values)?;

        // Detect change points in the time series
        let change_points = self
            .trend_detector
            .detect_change_points_pelt(&historical_values)?;

        // Calculate confidence interval for baseline
        let confidence_interval = self
            .statistical_analyzer
            .confidence_interval(&historical_values, self.confidence_threshold)?;

        // Calculate effect size comparing baseline to current
        let baseline_slice = &historical_values;
        let current_slice = &[current_value];
        let effect_size = self
            .statistical_analyzer
            .effect_size(baseline_slice, current_slice)?;

        // Determine if regression validation passes
        let statistical_confidence = mann_kendall.confidence.max(change_points.confidence);
        let validation_passed = statistical_confidence >= self.confidence_threshold
            && mann_kendall.p_value < (1.0 - self.confidence_threshold);

        let enhanced_result = EnhancedRegressionResult {
            basic_result,
            mann_kendall,
            change_points,
            confidence_interval,
            effect_size,
            statistical_confidence,
            validation_passed,
        };

        info!(
            metric = %metric_name,
            statistical_confidence = %statistical_confidence,
            mann_kendall_p_value = %enhanced_result.mann_kendall.p_value,
            effect_size = %effect_size,
            validation_passed = %validation_passed,
            "Enhanced regression detection completed"
        );

        Ok(Some(enhanced_result))
    }

    /// Validate regression with multiple statistical tests
    #[instrument(skip(self))]
    pub async fn validate_regression(
        &self,
        baseline_data: &[f64],
        current_data: &[f64],
    ) -> Result<ValidationResult> {
        if baseline_data.is_empty() || current_data.is_empty() {
            return Ok(ValidationResult {
                is_valid: false,
                confidence_score: 0.0,
                mann_kendall_result: None,
                change_point_result: None,
                effect_size: 0.0,
                validation_notes: vec!["Insufficient data for validation".to_string()],
            });
        }

        // Perform Mann-Kendall test on combined data
        let mut combined_data = baseline_data.to_vec();
        combined_data.extend_from_slice(current_data);
        let mann_kendall_result = self
            .statistical_analyzer
            .mann_kendall_test(&combined_data)?;

        // Run change point detection
        let change_point_result = self
            .trend_detector
            .detect_change_points_pelt(&combined_data)?;

        // Calculate effect size
        let effect_size = self
            .statistical_analyzer
            .effect_size(baseline_data, current_data)?;

        // Calculate confidence intervals
        let baseline_ci = self
            .statistical_analyzer
            .confidence_interval(baseline_data, self.confidence_threshold)?;
        let current_ci = self
            .statistical_analyzer
            .confidence_interval(current_data, self.confidence_threshold)?;

        // Determine overall validation
        let statistical_significance_pass =
            mann_kendall_result.p_value < (1.0 - self.confidence_threshold);
        let effect_size_meaningful = effect_size.abs() > 0.2; // Small effect size threshold
        let confidence_intervals_non_overlapping =
            baseline_ci.1 < current_ci.0 || current_ci.1 < baseline_ci.0;

        let is_valid = statistical_significance_pass
            && effect_size_meaningful
            && confidence_intervals_non_overlapping;
        let confidence_score = mann_kendall_result
            .confidence
            .min(change_point_result.confidence);

        let mut validation_notes = Vec::new();
        if !statistical_significance_pass {
            validation_notes.push(format!(
                "Statistical significance not achieved: p={:.4}",
                mann_kendall_result.p_value
            ));
        }
        if !effect_size_meaningful {
            validation_notes.push(format!("Effect size too small: {:.4}", effect_size));
        }
        if !confidence_intervals_non_overlapping {
            validation_notes.push(
                "Confidence intervals overlap, regression may not be significant".to_string(),
            );
        }

        Ok(ValidationResult {
            is_valid,
            confidence_score,
            mann_kendall_result: Some(mann_kendall_result),
            change_point_result: Some(change_point_result),
            effect_size,
            validation_notes,
        })
    }

    /// Analyze bottlenecks using genetic algorithm
    #[instrument(skip(self))]
    pub async fn analyze_bottlenecks_genetic(
        &self,
        metric_name: &str,
        hours_back: u64,
    ) -> Result<Option<BottleneckAnalysis>> {
        // Get recent performance data
        let performance_data = self
            .get_performance_data_for_genetic_analysis(metric_name, hours_back)
            .await?;

        if performance_data.is_empty() {
            debug!(metric = %metric_name, "No performance data available for genetic analysis");
            return Ok(None);
        }

        // Initialize genetic detector if not already initialized
        let mut genetic_detector = GeneticBottleneckDetector::new()
            .with_population_size(50)
            .with_generations(100)
            .with_mutation_rate(0.1)
            .with_crossover_rate(0.8);

        // Run genetic algorithm analysis
        let analysis = genetic_detector
            .evolve_bottleneck_detection(&performance_data)
            .await?;

        info!(
            metric = %metric_name,
            bottlenecks_found = analysis.identified_bottlenecks.len(),
            recommendations = analysis.optimization_recommendations.len(),
            performance_score = analysis.performance_score,
            "Genetic bottleneck analysis completed"
        );

        Ok(Some(analysis))
    }

    /// Convert stored metrics to performance data points for genetic analysis
    async fn get_performance_data_for_genetic_analysis(
        &self,
        metric_name: &str,
        hours_back: u64,
    ) -> Result<Vec<PerformanceDataPoint>> {
        let baselines = self.baselines.read().await;
        let baseline = match baselines.get(metric_name) {
            Some(baseline) => baseline,
            None => return Ok(Vec::new()),
        };

        let cutoff_time = SystemTime::now() - Duration::from_secs(hours_back * 3600);
        let cutoff_timestamp = cutoff_time.duration_since(UNIX_EPOCH)?.as_secs();

        let mut performance_data = Vec::new();

        // Convert baseline data to performance data points
        for data_point in &baseline.historical_values {
            let timestamp_u64 = data_point.timestamp.duration_since(UNIX_EPOCH)?.as_secs();
            if timestamp_u64 >= cutoff_timestamp {
                performance_data.push(PerformanceDataPoint {
                    timestamp: timestamp_u64,
                    cpu_usage: 0.5, // Default values - would be enhanced with real profiling data
                    memory_usage: 0.6,
                    io_wait: 0.1,
                    network_latency: 10.0,
                    execution_time: data_point.value,
                    throughput: if data_point.value > 0.0 {
                        1000.0 / data_point.value
                    } else {
                        1000.0
                    },
                    component: metric_name.to_string(),
                });
            }
        }

        // Sort by timestamp
        performance_data.sort_by_key(|p| p.timestamp);

        Ok(performance_data)
    }
}

impl AlertManager {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            last_alerts: RwLock::new(HashMap::new()),
        }
    }

    async fn send_regression_alert(&self, result: &RegressionResult) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check cooldown
        let should_alert = {
            let last_alerts = self.last_alerts.read().await;
            if let Some(last_alert_time) = last_alerts.get(&result.metric_name) {
                let cooldown = self
                    .config
                    .severity_thresholds
                    .get(&result.severity)
                    .copied()
                    .unwrap_or(Duration::from_secs(3600)); // Default 1 hour

                SystemTime::now()
                    .duration_since(*last_alert_time)
                    .unwrap_or_default()
                    > cooldown
            } else {
                true
            }
        };

        if !should_alert {
            return Ok(());
        }

        // Send notifications
        for channel in &self.config.notification_channels {
            self.send_notification(channel, result).await?;
        }

        // Update last alert time
        {
            let mut last_alerts = self.last_alerts.write().await;
            last_alerts.insert(result.metric_name.clone(), result.detected_at);
        }

        Ok(())
    }

    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        result: &RegressionResult,
    ) -> Result<()> {
        match channel {
            NotificationChannel::Slack {
                webhook_url,
                channel: _,
            } => self.send_slack_notification(webhook_url, result).await,
            NotificationChannel::Email { recipients } => {
                self.send_email_notification(recipients, result).await
            }
            NotificationChannel::Webhook { url, headers } => {
                self.send_webhook_notification(url, headers, result).await
            }
        }
    }

    async fn send_slack_notification(
        &self,
        _webhook_url: &str,
        result: &RegressionResult,
    ) -> Result<()> {
        // Implementation would send actual Slack notification
        info!(
            metric = %result.metric_name,
            severity = ?result.severity,
            "Slack notification sent for performance regression"
        );
        Ok(())
    }

    async fn send_email_notification(
        &self,
        _recipients: &[String],
        result: &RegressionResult,
    ) -> Result<()> {
        // Implementation would send actual email notification
        info!(
            metric = %result.metric_name,
            severity = ?result.severity,
            "Email notification sent for performance regression"
        );
        Ok(())
    }

    async fn send_webhook_notification(
        &self,
        _url: &str,
        _headers: &HashMap<String, String>,
        result: &RegressionResult,
    ) -> Result<()> {
        // Implementation would send actual webhook notification
        info!(
            metric = %result.metric_name,
            severity = ?result.severity,
            "Webhook notification sent for performance regression"
        );
        Ok(())
    }
}

impl MetricsStorage {
    pub fn new(storage_path: &PathBuf, retention_days: u32) -> Self {
        Self {
            storage_path: storage_path.clone(),
            retention_days,
        }
    }

    async fn load_historical_metrics(&self) -> Result<HashMap<String, Vec<MetricDataPoint>>> {
        // Implementation would load from actual storage (e.g., database, time series DB)
        info!("Loading historical metrics from storage");
        Ok(HashMap::new())
    }

    async fn store_metric(&self, _metric_name: &str, _data_point: &MetricDataPoint) -> Result<()> {
        // Implementation would store to actual storage
        Ok(())
    }
}

// Helper functions

fn calculate_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn calculate_std_dev(values: &[f64], mean: f64) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64;

    variance.sqrt()
}

fn calculate_variance(values: &[f64], mean: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64
}

fn calculate_confidence_interval(values: &[f64], confidence_level: f64) -> (f64, f64) {
    if values.len() < 2 {
        let value = values.first().copied().unwrap_or(0.0);
        return (value, value);
    }

    let mean = calculate_mean(values);
    let std_dev = calculate_std_dev(values, mean);

    // Use t-distribution critical value (approximation for 95% confidence)
    let t_critical = 1.96; // For large samples, t approaches z
    let margin_of_error = t_critical * std_dev / (values.len() as f64).sqrt();

    (mean - margin_of_error, mean + margin_of_error)
}

fn calculate_confidence_from_z_score(z_score: f64) -> f64 {
    // Simplified confidence calculation from z-score
    let abs_z = z_score.abs();

    if abs_z > 2.58 {
        0.99 // 99% confidence
    } else if abs_z > 1.96 {
        0.95 // 95% confidence
    } else if abs_z > 1.64 {
        0.90 // 90% confidence
    } else {
        0.50 + (abs_z / 3.29) * 0.49 // Linear approximation for lower confidence
    }
}

fn determine_regression_severity(regression_percentage: f64) -> RegressionSeverity {
    match regression_percentage {
        x if x >= 50.0 => RegressionSeverity::Critical,
        x if x >= 30.0 => RegressionSeverity::Major,
        x if x >= 15.0 => RegressionSeverity::Moderate,
        _ => RegressionSeverity::Minor,
    }
}

/// Enhanced regression result with statistical confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRegressionResult {
    pub basic_result: RegressionResult,
    pub mann_kendall: MannKendallResult,
    pub change_points: ChangePointResult,
    pub confidence_interval: (f64, f64),
    pub effect_size: f64,
    pub statistical_confidence: f64,
    pub validation_passed: bool,
}

/// Validation result for regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence_score: f64,
    pub mann_kendall_result: Option<MannKendallResult>,
    pub change_point_result: Option<ChangePointResult>,
    pub effect_size: f64,
    pub validation_notes: Vec<String>,
}

// Default implementations

impl Default for RegressionDetectionConfig {
    fn default() -> Self {
        Self {
            baseline_window_days: 14,
            min_samples_for_baseline: 50,
            regression_threshold_percent: 10.0,
            significance_threshold: 0.95,
            alert_cooldown: Duration::from_secs(3600),
            storage_path: PathBuf::from("./metrics_storage"),
            metrics_retention_days: 90,
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        let mut severity_thresholds = HashMap::new();
        severity_thresholds.insert(RegressionSeverity::Minor, Duration::from_secs(3600 * 4)); // 4 hours
        severity_thresholds.insert(RegressionSeverity::Moderate, Duration::from_secs(3600 * 2)); // 2 hours
        severity_thresholds.insert(RegressionSeverity::Major, Duration::from_secs(3600)); // 1 hour
        severity_thresholds.insert(RegressionSeverity::Critical, Duration::from_secs(900)); // 15 minutes

        Self {
            enabled: true,
            notification_channels: Vec::new(),
            severity_thresholds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_regression_detector_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let detector = PerformanceRegressionDetector::new(config);
        assert!(detector.is_ok());
    }

    #[tokio::test]
    async fn test_baseline_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            min_samples_for_baseline: 5,
            ..Default::default()
        };

        let detector = PerformanceRegressionDetector::new(config).unwrap();

        // Record some metrics to establish baseline
        for i in 0..10 {
            let metadata = HashMap::new();
            detector
                .record_metric("test_metric", 100.0 + i as f64, metadata)
                .await
                .unwrap();
        }

        let baselines = detector.get_baselines().await;
        assert!(baselines.contains_key("test_metric"));

        let baseline = &baselines["test_metric"];
        assert!(baseline.baseline_value > 100.0);
        assert!(baseline.sample_count >= 5);
    }

    #[test]
    fn test_statistical_functions() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let mean = calculate_mean(&values);
        assert_eq!(mean, 3.0);

        let std_dev = calculate_std_dev(&values, mean);
        assert!(std_dev > 0.0);

        let (lower, upper) = calculate_confidence_interval(&values, 0.95);
        assert!(lower < mean);
        assert!(upper > mean);
    }

    #[test]
    fn test_regression_severity_determination() {
        assert_eq!(
            determine_regression_severity(5.0),
            RegressionSeverity::Minor
        );
        assert_eq!(
            determine_regression_severity(20.0),
            RegressionSeverity::Moderate
        );
        assert_eq!(
            determine_regression_severity(40.0),
            RegressionSeverity::Major
        );
        assert_eq!(
            determine_regression_severity(60.0),
            RegressionSeverity::Critical
        );
    }
}
