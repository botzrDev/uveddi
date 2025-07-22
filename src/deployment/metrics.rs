//! Deployment metrics collection and analysis
//!
//! Provides functionality for collecting and analyzing deployment metrics including:
//! - Deployment success/failure rates
//! - Performance impact tracking
//! - MTTR/MTBF calculations
//! - Deployment frequency analytics

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use super::{DeploymentMetadata, DeploymentStatus, DeploymentStrategy, Environment};

/// Deployment metrics collector
#[derive(Debug)]
pub struct DeploymentMetricsCollector {
    metrics: RwLock<DeploymentMetrics>,
    config: MetricsConfig,
}

/// Deployment metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub retention_days: u32,
    pub aggregation_intervals: Vec<Duration>,
    pub performance_thresholds: PerformanceThresholds,
    pub alerting_enabled: bool,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub deployment_duration_warning: Duration,
    pub deployment_duration_critical: Duration,
    pub success_rate_warning: f64,
    pub success_rate_critical: f64,
    pub rollback_rate_warning: f64,
    pub rollback_rate_critical: f64,
}

/// Comprehensive deployment metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    pub total_deployments: u64,
    pub successful_deployments: u64,
    pub failed_deployments: u64,
    pub rollbacks: u64,
    pub deployment_history: Vec<DeploymentRecord>,
    pub strategy_metrics: HashMap<DeploymentStrategy, StrategyMetrics>,
    pub environment_metrics: HashMap<Environment, EnvironmentMetrics>,
    pub performance_metrics: PerformanceMetrics,
    pub frequency_metrics: FrequencyMetrics,
    pub reliability_metrics: ReliabilityMetrics,
}

/// Individual deployment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub metadata: DeploymentMetadata,
    pub duration: Duration,
    pub performance_impact: PerformanceImpact,
    pub issues: Vec<DeploymentIssue>,
    pub rollback_info: Option<RollbackInfo>,
}

/// Strategy-specific metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub total_deployments: u64,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub rollback_rate: f64,
    pub failure_reasons: HashMap<String, u64>,
}

/// Environment-specific metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnvironmentMetrics {
    pub total_deployments: u64,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub uptime_impact: Duration,
    pub last_deployment: Option<SystemTime>,
}

/// Performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub mttr: Duration,            // Mean Time To Recovery
    pub mtbf: Duration,            // Mean Time Between Failures
    pub mttd: Duration,            // Mean Time To Deployment
    pub lead_time: Duration,       // Deployment lead time
    pub cycle_time: Duration,      // Cycle time
    pub deployment_frequency: f64, // Deployments per day
    pub change_failure_rate: f64,  // Percentage of deployments causing failures
}

/// Frequency metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FrequencyMetrics {
    pub daily_deployments: HashMap<String, u64>, // Date -> count
    pub weekly_deployments: HashMap<String, u64>, // Week -> count
    pub monthly_deployments: HashMap<String, u64>, // Month -> count
    pub peak_deployment_hours: HashMap<u8, u64>, // Hour -> count
    pub deployment_trends: Vec<TrendPoint>,
}

/// Reliability metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    pub availability: f64,
    pub error_budget: f64,
    pub error_budget_consumed: f64,
    pub sli_metrics: HashMap<String, f64>, // Service Level Indicators
    pub slo_compliance: HashMap<String, f64>, // Service Level Objectives
}

/// Performance impact of deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    pub response_time_change: f64,  // Percentage change
    pub error_rate_change: f64,     // Percentage change
    pub throughput_change: f64,     // Percentage change
    pub resource_usage_change: f64, // Percentage change
    pub downtime: Duration,
}

/// Deployment issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub time_to_resolve: Option<Duration>,
    pub root_cause: Option<String>,
}

/// Types of deployment issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    HealthCheckFailure,
    PerformanceDegradation,
    ConfigurationError,
    DependencyFailure,
    ResourceExhaustion,
    NetworkIssue,
    UserError,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Rollback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub triggered_at: SystemTime,
    pub trigger_reason: String,
    pub rollback_duration: Duration,
    pub automatic: bool,
    pub success: bool,
}

/// Trend point for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub timestamp: SystemTime,
    pub value: f64,
    pub metric_name: String,
}

/// Deployment analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentAnalytics {
    pub summary: AnalyticsSummary,
    pub trends: Vec<TrendAnalysis>,
    pub recommendations: Vec<String>,
    pub benchmarks: BenchmarkComparison,
    pub forecasts: Vec<ForecastPoint>,
}

/// Analytics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub period: DateRange,
    pub total_deployments: u64,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub deployment_frequency: f64,
    pub change_failure_rate: f64,
    pub mttr: Duration,
    pub key_insights: Vec<String>,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub metric_name: String,
    pub trend_direction: TrendDirection,
    pub confidence: f64,
    pub rate_of_change: f64,
    pub significance: TrendSignificance,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

/// Trend significance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendSignificance {
    High,
    Medium,
    Low,
    Insignificant,
}

/// Benchmark comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub industry_percentile: f64,
    pub compared_metrics: HashMap<String, BenchmarkMetric>,
    pub improvement_opportunities: Vec<String>,
}

/// Benchmark metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetric {
    pub current_value: f64,
    pub industry_median: f64,
    pub industry_p90: f64,
    pub industry_p95: f64,
    pub performance_rating: PerformanceRating,
}

/// Performance rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceRating {
    Elite,
    High,
    Medium,
    Low,
}

/// Forecast point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub timestamp: SystemTime,
    pub metric_name: String,
    pub predicted_value: f64,
    pub confidence_interval: (f64, f64),
}

/// Date range for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: SystemTime,
    pub end: SystemTime,
}

impl DeploymentMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::with_config(MetricsConfig::default())
    }

    /// Create metrics collector with custom configuration
    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            metrics: RwLock::new(DeploymentMetrics::default()),
            config,
        }
    }

    /// Record deployment start
    #[instrument(skip(self))]
    pub async fn record_deployment_start(&self, metadata: &DeploymentMetadata) -> Result<()> {
        debug!(
            deployment_id = %metadata.id,
            strategy = ?metadata.config.strategy,
            environment = ?metadata.config.environment,
            "Recording deployment start"
        );

        let mut metrics = self.metrics.write().await;
        metrics.total_deployments += 1;

        // Initialize deployment record
        let deployment_record = DeploymentRecord {
            metadata: metadata.clone(),
            duration: Duration::from_secs(0), // Will be updated on completion
            performance_impact: PerformanceImpact::default(),
            issues: Vec::new(),
            rollback_info: None,
        };

        metrics.deployment_history.push(deployment_record);

        // Update frequency metrics
        self.update_frequency_metrics(&mut metrics, metadata.timestamp)
            .await;

        Ok(())
    }

    /// Record deployment success
    #[instrument(skip(self))]
    pub async fn record_deployment_success(&self, metadata: &DeploymentMetadata) -> Result<()> {
        info!(
            deployment_id = %metadata.id,
            duration = ?metadata.timestamp.elapsed().unwrap_or_default(),
            "Recording deployment success"
        );

        let mut metrics = self.metrics.write().await;
        metrics.successful_deployments += 1;

        // Update deployment record
        if let Some(record) = metrics
            .deployment_history
            .iter_mut()
            .find(|r| r.metadata.id == metadata.id)
        {
            record.duration = metadata.timestamp.elapsed().unwrap_or_default();
        }

        // Update strategy metrics
        self.update_strategy_metrics(
            &mut metrics,
            &metadata.config.strategy,
            true,
            metadata.timestamp.elapsed().unwrap_or_default(),
        )
        .await;

        // Update environment metrics
        self.update_environment_metrics(
            &mut metrics,
            &metadata.config.environment,
            true,
            metadata.timestamp.elapsed().unwrap_or_default(),
        )
        .await;

        // Update performance metrics
        self.update_performance_metrics(&mut metrics).await;

        Ok(())
    }

    /// Record deployment failure
    #[instrument(skip(self))]
    pub async fn record_deployment_failure(
        &self,
        metadata: &DeploymentMetadata,
        failure_reason: String,
    ) -> Result<()> {
        info!(
            deployment_id = %metadata.id,
            failure_reason = %failure_reason,
            "Recording deployment failure"
        );

        let mut metrics = self.metrics.write().await;
        metrics.failed_deployments += 1;

        // Update deployment record
        if let Some(record) = metrics
            .deployment_history
            .iter_mut()
            .find(|r| r.metadata.id == metadata.id)
        {
            record.duration = metadata.timestamp.elapsed().unwrap_or_default();
            record.issues.push(DeploymentIssue {
                issue_type: IssueType::ConfigurationError, // Default type
                severity: IssueSeverity::High,
                description: failure_reason.clone(),
                time_to_resolve: None,
                root_cause: Some(failure_reason.clone()),
            });
        }

        // Update strategy metrics
        self.update_strategy_metrics(
            &mut metrics,
            &metadata.config.strategy,
            false,
            metadata.timestamp.elapsed().unwrap_or_default(),
        )
        .await;

        // Update environment metrics
        self.update_environment_metrics(
            &mut metrics,
            &metadata.config.environment,
            false,
            metadata.timestamp.elapsed().unwrap_or_default(),
        )
        .await;

        Ok(())
    }

    /// Record rollback
    #[instrument(skip(self))]
    pub async fn record_rollback(
        &self,
        deployment_id: &str,
        rollback_info: RollbackInfo,
    ) -> Result<()> {
        info!(
            deployment_id = %deployment_id,
            trigger_reason = %rollback_info.trigger_reason,
            automatic = %rollback_info.automatic,
            "Recording rollback"
        );

        let mut metrics = self.metrics.write().await;
        metrics.rollbacks += 1;

        // Update deployment record
        if let Some(record) = metrics
            .deployment_history
            .iter_mut()
            .find(|r| r.metadata.id == deployment_id)
        {
            record.rollback_info = Some(rollback_info);
        }

        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> DeploymentMetrics {
        (*self.metrics.read().await).clone()
    }

    /// Generate analytics report
    #[instrument(skip(self))]
    pub async fn generate_analytics(&self, period: DateRange) -> Result<DeploymentAnalytics> {
        let metrics = self.metrics.read().await;

        // Filter deployments within the period
        let period_deployments: Vec<&DeploymentRecord> = metrics
            .deployment_history
            .iter()
            .filter(|record| {
                record.metadata.timestamp >= period.start && record.metadata.timestamp <= period.end
            })
            .collect();

        // Calculate summary
        let total_deployments = period_deployments.len() as u64;
        let successful_deployments = period_deployments
            .iter()
            .filter(|record| record.metadata.status == DeploymentStatus::Verified)
            .count() as u64;

        let success_rate = if total_deployments > 0 {
            (successful_deployments as f64 / total_deployments as f64) * 100.0
        } else {
            0.0
        };

        let average_duration = if !period_deployments.is_empty() {
            let total_duration: Duration = period_deployments
                .iter()
                .map(|record| record.duration)
                .sum();
            total_duration / period_deployments.len() as u32
        } else {
            Duration::from_secs(0)
        };

        let period_days = period
            .end
            .duration_since(period.start)
            .unwrap_or_default()
            .as_secs() as f64
            / (24.0 * 3600.0);
        let deployment_frequency = if period_days > 0.0 {
            total_deployments as f64 / period_days
        } else {
            0.0
        };

        let change_failure_rate = if total_deployments > 0 {
            let failures = period_deployments
                .iter()
                .filter(|record| !record.issues.is_empty() || record.rollback_info.is_some())
                .count() as u64;
            (failures as f64 / total_deployments as f64) * 100.0
        } else {
            0.0
        };

        let summary = AnalyticsSummary {
            period,
            total_deployments,
            success_rate,
            average_duration,
            deployment_frequency,
            change_failure_rate,
            mttr: metrics.performance_metrics.mttr,
            key_insights: self
                .generate_key_insights(&period_deployments, &metrics)
                .await,
        };

        // Generate trend analysis
        let trends = self.analyze_trends(&period_deployments).await;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&summary, &trends).await;

        // Generate benchmarks
        let benchmarks = self.generate_benchmark_comparison(&summary).await;

        // Generate forecasts
        let forecasts = self.generate_forecasts(&period_deployments).await;

        Ok(DeploymentAnalytics {
            summary,
            trends,
            recommendations,
            benchmarks,
            forecasts,
        })
    }

    /// Update frequency metrics
    async fn update_frequency_metrics(
        &self,
        metrics: &mut DeploymentMetrics,
        timestamp: SystemTime,
    ) {
        let date_key = format!(
            "{}",
            timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                / (24 * 3600)
        );
        *metrics
            .frequency_metrics
            .daily_deployments
            .entry(date_key)
            .or_insert(0) += 1;

        // Update hourly distribution
        let hour = (timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 3600)
            % 24;
        *metrics
            .frequency_metrics
            .peak_deployment_hours
            .entry(hour as u8)
            .or_insert(0) += 1;
    }

    /// Update strategy metrics
    async fn update_strategy_metrics(
        &self,
        metrics: &mut DeploymentMetrics,
        strategy: &DeploymentStrategy,
        success: bool,
        duration: Duration,
    ) {
        let strategy_metrics = metrics
            .strategy_metrics
            .entry(strategy.clone())
            .or_default();

        strategy_metrics.total_deployments += 1;

        if success {
            strategy_metrics.success_rate = (strategy_metrics.success_rate
                * (strategy_metrics.total_deployments - 1) as f64
                + 100.0)
                / strategy_metrics.total_deployments as f64;
        } else {
            strategy_metrics.success_rate = (strategy_metrics.success_rate
                * (strategy_metrics.total_deployments - 1) as f64)
                / strategy_metrics.total_deployments as f64;
        }

        // Update average duration
        let total_duration = strategy_metrics.average_duration
            * (strategy_metrics.total_deployments - 1) as u32
            + duration;
        strategy_metrics.average_duration =
            total_duration / strategy_metrics.total_deployments as u32;
    }

    /// Update environment metrics
    async fn update_environment_metrics(
        &self,
        metrics: &mut DeploymentMetrics,
        environment: &Environment,
        success: bool,
        duration: Duration,
    ) {
        let env_metrics = metrics
            .environment_metrics
            .entry(environment.clone())
            .or_default();

        env_metrics.total_deployments += 1;
        env_metrics.last_deployment = Some(SystemTime::now());

        if success {
            env_metrics.success_rate =
                (env_metrics.success_rate * (env_metrics.total_deployments - 1) as f64 + 100.0)
                    / env_metrics.total_deployments as f64;
        } else {
            env_metrics.success_rate = (env_metrics.success_rate
                * (env_metrics.total_deployments - 1) as f64)
                / env_metrics.total_deployments as f64;
        }

        // Update average duration
        let total_duration =
            env_metrics.average_duration * (env_metrics.total_deployments - 1) as u32 + duration;
        env_metrics.average_duration = total_duration / env_metrics.total_deployments as u32;
    }

    /// Update performance metrics
    async fn update_performance_metrics(&self, metrics: &mut DeploymentMetrics) {
        // Calculate MTTR (Mean Time To Recovery)
        let total_recovery_time: Duration = metrics
            .deployment_history
            .iter()
            .filter_map(|record| record.rollback_info.as_ref().map(|r| r.rollback_duration))
            .sum();

        let rollback_count = metrics.rollbacks.max(1); // Avoid division by zero
        metrics.performance_metrics.mttr = total_recovery_time / rollback_count as u32;

        // Calculate deployment frequency
        let days_since_first_deployment =
            if let Some(first_deployment) = metrics.deployment_history.first() {
                SystemTime::now()
                    .duration_since(first_deployment.metadata.timestamp)
                    .unwrap_or_default()
                    .as_secs() as f64
                    / (24.0 * 3600.0)
            } else {
                1.0
            };

        metrics.performance_metrics.deployment_frequency =
            metrics.total_deployments as f64 / days_since_first_deployment.max(1.0);

        // Calculate change failure rate
        let failures = metrics
            .deployment_history
            .iter()
            .filter(|record| !record.issues.is_empty() || record.rollback_info.is_some())
            .count() as u64;

        metrics.performance_metrics.change_failure_rate = if metrics.total_deployments > 0 {
            (failures as f64 / metrics.total_deployments as f64) * 100.0
        } else {
            0.0
        };
    }

    /// Generate key insights
    async fn generate_key_insights(
        &self,
        deployments: &[&DeploymentRecord],
        metrics: &DeploymentMetrics,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        // Success rate insight
        let successful = deployments
            .iter()
            .filter(|record| record.metadata.status == DeploymentStatus::Verified)
            .count();
        let success_rate = if !deployments.is_empty() {
            (successful as f64 / deployments.len() as f64) * 100.0
        } else {
            0.0
        };

        if success_rate >= 95.0 {
            insights.push("Excellent deployment success rate above 95%".to_string());
        } else if success_rate < 80.0 {
            insights.push("Deployment success rate below 80% needs attention".to_string());
        }

        // Deployment frequency insight
        if metrics.performance_metrics.deployment_frequency > 1.0 {
            insights.push("High deployment frequency indicates good CI/CD maturity".to_string());
        } else if metrics.performance_metrics.deployment_frequency < 0.1 {
            insights.push("Low deployment frequency may indicate process bottlenecks".to_string());
        }

        // Rollback rate insight
        let rollback_rate = if metrics.total_deployments > 0 {
            (metrics.rollbacks as f64 / metrics.total_deployments as f64) * 100.0
        } else {
            0.0
        };

        if rollback_rate > 10.0 {
            insights.push("High rollback rate indicates need for better testing".to_string());
        }

        insights
    }

    /// Analyze trends
    async fn analyze_trends(&self, _deployments: &[&DeploymentRecord]) -> Vec<TrendAnalysis> {
        // In a real implementation, this would perform statistical analysis
        // on deployment metrics over time to identify trends
        vec![
            TrendAnalysis {
                metric_name: "success_rate".to_string(),
                trend_direction: TrendDirection::Improving,
                confidence: 0.85,
                rate_of_change: 2.5,
                significance: TrendSignificance::Medium,
            },
            TrendAnalysis {
                metric_name: "deployment_duration".to_string(),
                trend_direction: TrendDirection::Stable,
                confidence: 0.92,
                rate_of_change: 0.1,
                significance: TrendSignificance::Low,
            },
        ]
    }

    /// Generate recommendations
    async fn generate_recommendations(
        &self,
        summary: &AnalyticsSummary,
        trends: &[TrendAnalysis],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if summary.success_rate < 95.0 {
            recommendations.push(
                "Improve deployment success rate by enhancing testing and validation".to_string(),
            );
        }

        if summary.change_failure_rate > 15.0 {
            recommendations.push(
                "Reduce change failure rate by implementing better quality gates".to_string(),
            );
        }

        if summary.deployment_frequency < 0.5 {
            recommendations
                .push("Increase deployment frequency to improve delivery speed".to_string());
        }

        for trend in trends {
            if trend.trend_direction == TrendDirection::Declining
                && trend.significance != TrendSignificance::Insignificant
            {
                recommendations.push(format!("Address declining trend in {}", trend.metric_name));
            }
        }

        if summary.mttr > Duration::from_secs(1800) {
            // 30 minutes
            recommendations.push(
                "Improve mean time to recovery by automating rollback procedures".to_string(),
            );
        }

        recommendations
    }

    /// Generate benchmark comparison
    async fn generate_benchmark_comparison(
        &self,
        summary: &AnalyticsSummary,
    ) -> BenchmarkComparison {
        // Industry benchmarks (these would come from real industry data)
        let mut compared_metrics = HashMap::new();

        compared_metrics.insert(
            "deployment_frequency".to_string(),
            BenchmarkMetric {
                current_value: summary.deployment_frequency,
                industry_median: 0.5,
                industry_p90: 2.0,
                industry_p95: 5.0,
                performance_rating: if summary.deployment_frequency >= 2.0 {
                    PerformanceRating::High
                } else if summary.deployment_frequency >= 0.5 {
                    PerformanceRating::Medium
                } else {
                    PerformanceRating::Low
                },
            },
        );

        compared_metrics.insert(
            "success_rate".to_string(),
            BenchmarkMetric {
                current_value: summary.success_rate,
                industry_median: 85.0,
                industry_p90: 95.0,
                industry_p95: 98.0,
                performance_rating: if summary.success_rate >= 95.0 {
                    PerformanceRating::High
                } else if summary.success_rate >= 85.0 {
                    PerformanceRating::Medium
                } else {
                    PerformanceRating::Low
                },
            },
        );

        let industry_percentile =
            (summary.success_rate + summary.deployment_frequency * 10.0) / 2.0;

        BenchmarkComparison {
            industry_percentile,
            compared_metrics,
            improvement_opportunities: vec![
                "Implement automated testing to improve success rate".to_string(),
                "Optimize deployment pipeline to increase frequency".to_string(),
            ],
        }
    }

    /// Generate forecasts
    async fn generate_forecasts(&self, _deployments: &[&DeploymentRecord]) -> Vec<ForecastPoint> {
        // In a real implementation, this would use time series analysis
        // to predict future metrics based on historical data
        vec![ForecastPoint {
            timestamp: SystemTime::now() + Duration::from_secs(30 * 24 * 3600), // 30 days
            metric_name: "deployment_frequency".to_string(),
            predicted_value: 1.2,
            confidence_interval: (0.8, 1.6),
        }]
    }
}

impl Default for DeploymentMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            retention_days: 90,
            aggregation_intervals: vec![
                Duration::from_secs(3600),          // 1 hour
                Duration::from_secs(24 * 3600),     // 1 day
                Duration::from_secs(7 * 24 * 3600), // 1 week
            ],
            performance_thresholds: PerformanceThresholds::default(),
            alerting_enabled: true,
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            deployment_duration_warning: Duration::from_secs(900), // 15 minutes
            deployment_duration_critical: Duration::from_secs(1800), // 30 minutes
            success_rate_warning: 90.0,
            success_rate_critical: 80.0,
            rollback_rate_warning: 5.0,
            rollback_rate_critical: 10.0,
        }
    }
}

impl Default for PerformanceImpact {
    fn default() -> Self {
        Self {
            response_time_change: 0.0,
            error_rate_change: 0.0,
            throughput_change: 0.0,
            resource_usage_change: 0.0,
            downtime: Duration::from_secs(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = DeploymentMetricsCollector::new();
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_deployments, 0);
    }

    #[tokio::test]
    async fn test_record_deployment_success() {
        let collector = DeploymentMetricsCollector::new();

        let metadata = DeploymentMetadata {
            id: "test-deployment".to_string(),
            version: "v1.0.0".to_string(),
            timestamp: SystemTime::now(),
            triggered_by: "test".to_string(),
            commit_sha: "abc123".to_string(),
            config: super::super::DeploymentConfig {
                strategy: DeploymentStrategy::BlueGreen,
                environment: Environment::Production,
                image_tag: "test:latest".to_string(),
                replicas: 3,
                health_check_timeout: Duration::from_secs(30),
                rollback_enabled: true,
                canary_weight: None,
                validation_tests: vec![],
            },
            status: DeploymentStatus::Verified,
        };

        collector.record_deployment_start(&metadata).await.unwrap();
        collector
            .record_deployment_success(&metadata)
            .await
            .unwrap();

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_deployments, 1);
        assert_eq!(metrics.successful_deployments, 1);
    }
}
