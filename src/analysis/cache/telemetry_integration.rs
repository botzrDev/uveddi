#![cfg(feature = "ast-cache")]

//! Cache Telemetry Integration
//!
//! This module integrates cache metrics with Uveddi's telemetry systems,
//! providing comprehensive monitoring and alerting for cache performance.

#[cfg(feature = "prometheus")]
use crate::analysis::cache::metrics::CacheMetrics;
use crate::analysis::cache::{EnhancedCacheStats, FileWatcherStats};
use crate::core::logging::{debug, info, warn};
// REMOVED: monitoring module removed for CLI release
// use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;

// Stub for removed monitoring feature
pub struct PerformanceMetricsCollector;

use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tokio::time::interval;

/// Cache telemetry configuration
#[derive(Debug, Clone)]
/// Configuration options for cachetelemetry.
pub struct CacheTelemetryConfig {
    /// Enable telemetry collection
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval_seconds: u64,
    /// Enable detailed metrics (may impact performance)
    pub detailed_metrics: bool,
    /// Enable alerting for cache performance issues
    pub alerting_enabled: bool,
    /// Threshold for low cache hit rate alerts (percentage)
    pub hit_rate_alert_threshold: f64,
    /// Threshold for high memory usage alerts (bytes)
    pub memory_alert_threshold: Option<usize>,
}

impl Default for CacheTelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_seconds: 60, // 1 minute
            detailed_metrics: false,
            alerting_enabled: true,
            hit_rate_alert_threshold: 0.5,                    // 50%
            memory_alert_threshold: Some(1024 * 1024 * 1024), // 1GB
        }
    }
}

/// Comprehensive cache metrics for telemetry
#[derive(Debug, Clone)]
/// Performance and quality metrics for cachetelemetry.
pub struct CacheTelemetryMetrics {
    pub timestamp: SystemTime,
    pub cache_stats: Option<EnhancedCacheStats>,
    pub file_watcher_stats: Option<FileWatcherStats>,
    pub performance_metrics: CachePerformanceMetrics,
    pub alerts: Vec<CacheAlert>,
}

/// Cache performance metrics
#[derive(Debug, Clone)]
/// Performance and quality metrics for cacheperformance.
pub struct CachePerformanceMetrics {
    pub overall_hit_rate: f64,
    pub ast_hit_rate: f64,
    pub results_hit_rate: f64,
    pub total_memory_usage_bytes: usize,
    pub ast_memory_usage_bytes: usize,
    pub results_memory_usage_bytes: usize,
    pub total_entries: usize,
    pub eviction_rate_per_minute: f64,
    pub invalidation_rate_per_minute: f64,
}

/// Cache alert types
#[derive(Debug, Clone)]
/// Represents cache alert in the system.
pub struct CacheAlert {
    pub alert_type: CacheAlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: SystemTime,
    pub details: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Enumeration of cachealerttype variants.
pub enum CacheAlertType {
    LowHitRate,
    HighMemoryUsage,
    ExcessiveEvictions,
    FileWatcherOverload,
    CacheSystemFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Enumeration of alertseverity variants.
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Cache telemetry collector
pub struct CacheTelemetryCollector {
    config: CacheTelemetryConfig,
    #[cfg(feature = "prometheus")]
    prometheus_metrics: Option<Arc<CacheMetrics>>,
    performance_collector: Option<Arc<PerformanceMetricsCollector>>,
    metrics_history: Arc<RwLock<Vec<CacheTelemetryMetrics>>>,
    active_alerts: Arc<RwLock<Vec<CacheAlert>>>,
    _collection_handle: Option<tokio::task::JoinHandle<()>>,
}

impl CacheTelemetryCollector {
    /// Create new telemetry collector
    pub fn new(config: CacheTelemetryConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "prometheus")]
            prometheus_metrics: None,
            performance_collector: None,
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            _collection_handle: None,
        }
    }

    /// Set Prometheus metrics collector
    #[cfg(feature = "prometheus")]
    /// Creates a new instance with custom prometheus metrics.
    pub fn with_prometheus_metrics(mut self, metrics: Arc<CacheMetrics>) -> Self {
        self.prometheus_metrics = Some(metrics);
        self
    }

    /// Set performance metrics collector
    pub fn with_performance_collector(
        mut self,
        collector: Arc<PerformanceMetricsCollector>,
    ) -> Self {
        self.performance_collector = Some(collector);
        self
    }

    /// Start telemetry collection
    pub async fn start_collection(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            info!("Cache telemetry collection is disabled");
            return Ok(());
        }

        info!("Starting cache telemetry collection");

        let config = self.config.clone();
        #[cfg(feature = "prometheus")]
        let prometheus_metrics = self.prometheus_metrics.clone();
        let performance_collector = self.performance_collector.clone();
        let metrics_history = Arc::clone(&self.metrics_history);
        let active_alerts = Arc::clone(&self.active_alerts);

        let handle = tokio::spawn(async move {
            let mut interval_timer =
                interval(Duration::from_secs(config.collection_interval_seconds));

            loop {
                interval_timer.tick().await;

                let metrics = Self::collect_metrics_static(
                    &config,
                    #[cfg(feature = "prometheus")]
                    prometheus_metrics.as_ref(),
                    performance_collector.as_ref(),
                )
                .await;

                // Store metrics in history
                {
                    let mut history = metrics_history.write().await;
                    history.push(metrics.clone());

                    // Keep only last 1000 entries to prevent memory leaks
                    if history.len() > 1000 {
                        let drain_count = history.len() - 1000;
                        history.drain(..drain_count);
                    }
                }

                // Process alerts
                if config.alerting_enabled {
                    let alerts = Self::process_alerts(&config, &metrics).await;
                    if !alerts.is_empty() {
                        let mut active = active_alerts.write().await;
                        active.extend(alerts.clone());

                        // Log alerts
                        for alert in alerts {
                            match alert.severity {
                                AlertSeverity::Info => info!("Cache Alert: {}", alert.message),
                                AlertSeverity::Warning => warn!("Cache Alert: {}", alert.message),
                                AlertSeverity::Error | AlertSeverity::Critical => {
                                    warn!(
                                        "Cache Alert [{}]: {}",
                                        format!("{:?}", alert.severity),
                                        alert.message
                                    );
                                }
                            }
                        }
                    }
                }

                debug!("Cache telemetry collection completed");
            }
        });

        self._collection_handle = Some(handle);
        Ok(())
    }

    /// Collect current cache metrics
    async fn collect_metrics_static(
        config: &CacheTelemetryConfig,
        #[cfg(feature = "prometheus")] prometheus_metrics: Option<&Arc<CacheMetrics>>,
        performance_collector: Option<&Arc<PerformanceMetricsCollector>>,
    ) -> CacheTelemetryMetrics {
        let timestamp = SystemTime::now();

        // Collect performance metrics
        let performance_metrics = CachePerformanceMetrics {
            overall_hit_rate: 0.0, // Would be calculated from cache stats
            ast_hit_rate: 0.0,
            results_hit_rate: 0.0,
            total_memory_usage_bytes: 0,
            ast_memory_usage_bytes: 0,
            results_memory_usage_bytes: 0,
            total_entries: 0,
            eviction_rate_per_minute: 0.0,
            invalidation_rate_per_minute: 0.0,
        };

        // Collect Prometheus metrics if available
        #[cfg(feature = "prometheus")]
        if let Some(_prometheus) = prometheus_metrics {
            // Would integrate with actual Prometheus metrics
        }

        // Collect performance collector metrics if available
        if let Some(_perf) = performance_collector {
            // Would integrate with performance collector
        }

        CacheTelemetryMetrics {
            timestamp,
            cache_stats: None,        // Would be populated with actual cache stats
            file_watcher_stats: None, // Would be populated with actual file watcher stats
            performance_metrics,
            alerts: Vec::new(),
        }
    }

    /// Process and generate alerts based on metrics
    async fn process_alerts(
        config: &CacheTelemetryConfig,
        metrics: &CacheTelemetryMetrics,
    ) -> Vec<CacheAlert> {
        let mut alerts = Vec::new();
        let timestamp = SystemTime::now();

        // Check hit rate
        if metrics.performance_metrics.overall_hit_rate < config.hit_rate_alert_threshold {
            alerts.push(CacheAlert {
                alert_type: CacheAlertType::LowHitRate,
                message: format!(
                    "Cache hit rate is low: {:.1}% (threshold: {:.1}%)",
                    metrics.performance_metrics.overall_hit_rate * 100.0,
                    config.hit_rate_alert_threshold * 100.0
                ),
                severity: AlertSeverity::Warning,
                timestamp,
                details: {
                    let mut map = HashMap::new();
                    map.insert(
                        "current_hit_rate".to_string(),
                        json!(metrics.performance_metrics.overall_hit_rate),
                    );
                    map.insert(
                        "threshold".to_string(),
                        json!(config.hit_rate_alert_threshold),
                    );
                    map.insert(
                        "ast_hit_rate".to_string(),
                        json!(metrics.performance_metrics.ast_hit_rate),
                    );
                    map.insert(
                        "results_hit_rate".to_string(),
                        json!(metrics.performance_metrics.results_hit_rate),
                    );
                    map
                },
            });
        }

        // Check memory usage
        if let Some(memory_threshold) = config.memory_alert_threshold {
            if metrics.performance_metrics.total_memory_usage_bytes > memory_threshold {
                alerts.push(CacheAlert {
                    alert_type: CacheAlertType::HighMemoryUsage,
                    message: format!(
                        "Cache memory usage is high: {} MB (threshold: {} MB)",
                        metrics.performance_metrics.total_memory_usage_bytes / (1024 * 1024),
                        memory_threshold / (1024 * 1024)
                    ),
                    severity: AlertSeverity::Warning,
                    timestamp,
                    details: {
                        let mut map = HashMap::new();
                        map.insert(
                            "current_memory_bytes".to_string(),
                            json!(metrics.performance_metrics.total_memory_usage_bytes),
                        );
                        map.insert("threshold_bytes".to_string(), json!(memory_threshold));
                        map.insert(
                            "ast_memory_bytes".to_string(),
                            json!(metrics.performance_metrics.ast_memory_usage_bytes),
                        );
                        map.insert(
                            "results_memory_bytes".to_string(),
                            json!(metrics.performance_metrics.results_memory_usage_bytes),
                        );
                        map
                    },
                });
            }
        }

        // Check eviction rate
        if metrics.performance_metrics.eviction_rate_per_minute > 50.0 {
            alerts.push(CacheAlert {
                alert_type: CacheAlertType::ExcessiveEvictions,
                message: format!(
                    "High cache eviction rate: {:.1} evictions per minute",
                    metrics.performance_metrics.eviction_rate_per_minute
                ),
                severity: AlertSeverity::Info,
                timestamp,
                details: {
                    let mut map = HashMap::new();
                    map.insert(
                        "eviction_rate_per_minute".to_string(),
                        json!(metrics.performance_metrics.eviction_rate_per_minute),
                    );
                    map
                },
            });
        }

        // Check file watcher overload
        if let Some(file_watcher_stats) = &metrics.file_watcher_stats {
            if file_watcher_stats.pending_changes_count > 100 {
                alerts.push(CacheAlert {
                    alert_type: CacheAlertType::FileWatcherOverload,
                    message: format!(
                        "File watcher has many pending changes: {}",
                        file_watcher_stats.pending_changes_count
                    ),
                    severity: AlertSeverity::Warning,
                    timestamp,
                    details: {
                        let mut map = HashMap::new();
                        map.insert(
                            "pending_changes".to_string(),
                            json!(file_watcher_stats.pending_changes_count),
                        );
                        map.insert(
                            "watched_files".to_string(),
                            json!(file_watcher_stats.watched_file_count),
                        );
                        map.insert(
                            "polling_interval_ms".to_string(),
                            json!(file_watcher_stats.polling_interval_ms),
                        );
                        map
                    },
                });
            }
        }

        alerts
    }

    /// Get recent cache metrics
    pub async fn get_recent_metrics(&self, count: usize) -> Vec<CacheTelemetryMetrics> {
        let history = self.metrics_history.read().await;
        history.iter().rev().take(count).cloned().collect()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<CacheAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.clone()
    }

    /// Clear resolved alerts
    pub async fn clear_alerts(&self, alert_type: Option<CacheAlertType>) {
        let mut alerts = self.active_alerts.write().await;
        if let Some(alert_type_filter) = alert_type {
            alerts.retain(|alert| alert.alert_type != alert_type_filter);
        } else {
            alerts.clear();
        }
    }

    /// Generate telemetry report
    pub async fn generate_report(&self) -> CacheTelemetryReport {
        let recent_metrics = self.get_recent_metrics(100).await;
        let active_alerts = self.get_active_alerts().await;

        let summary = if !recent_metrics.is_empty() {
            let latest = &recent_metrics[0];
            CacheSummary {
                overall_hit_rate: latest.performance_metrics.overall_hit_rate,
                total_memory_usage_mb: latest.performance_metrics.total_memory_usage_bytes
                    / (1024 * 1024),
                total_entries: latest.performance_metrics.total_entries,
                active_alerts_count: active_alerts.len(),
            }
        } else {
            CacheSummary {
                overall_hit_rate: 0.0,
                total_memory_usage_mb: 0,
                total_entries: 0,
                active_alerts_count: active_alerts.len(),
            }
        };

        let recommendations = self.generate_recommendations(&recent_metrics).await;

        CacheTelemetryReport {
            generated_at: SystemTime::now(),
            summary,
            recent_metrics,
            active_alerts,
            recommendations,
        }
    }

    /// Generate performance recommendations
    async fn generate_recommendations(&self, metrics: &[CacheTelemetryMetrics]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.is_empty() {
            return recommendations;
        }

        let latest = &metrics[0];

        // Hit rate recommendations
        if latest.performance_metrics.overall_hit_rate < 0.7 {
            recommendations
                .push("Consider increasing cache size limits to improve hit rates".to_string());
        }

        // Memory recommendations
        if latest.performance_metrics.total_memory_usage_bytes > 512 * 1024 * 1024 {
            recommendations.push("High memory usage detected - consider enabling compression or reducing cache sizes".to_string());
        }

        // Eviction recommendations
        if latest.performance_metrics.eviction_rate_per_minute > 10.0 {
            recommendations.push("High eviction rate - consider adjusting eviction policy or increasing memory limits".to_string());
        }

        // File watcher recommendations
        if let Some(file_stats) = &latest.file_watcher_stats {
            if file_stats.polling_interval_ms < 500 {
                recommendations.push("Very frequent file polling detected - consider increasing polling interval to reduce CPU usage".to_string());
            }
        }

        recommendations
    }

    /// Stop telemetry collection
    pub async fn stop_collection(&mut self) {
        info!("Stopping cache telemetry collection");

        if let Some(handle) = self._collection_handle.take() {
            handle.abort();
        }

        info!("Cache telemetry collection stopped");
    }
}

/// Cache telemetry report
#[derive(Debug, Clone)]
/// Represents cache telemetry report in the system.
pub struct CacheTelemetryReport {
    pub generated_at: SystemTime,
    pub summary: CacheSummary,
    pub recent_metrics: Vec<CacheTelemetryMetrics>,
    pub active_alerts: Vec<CacheAlert>,
    pub recommendations: Vec<String>,
}

/// Cache summary statistics
#[derive(Debug, Clone)]
/// Represents cache summary in the system.
pub struct CacheSummary {
    pub overall_hit_rate: f64,
    pub total_memory_usage_mb: usize,
    pub total_entries: usize,
    pub active_alerts_count: usize,
}

impl CacheTelemetryReport {
    /// Convert report to JSON for API responses
    pub fn to_json(&self) -> Value {
        json!({
            "generated_at": self.generated_at.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            "summary": {
                "overall_hit_rate": self.summary.overall_hit_rate,
                "total_memory_usage_mb": self.summary.total_memory_usage_mb,
                "total_entries": self.summary.total_entries,
                "active_alerts_count": self.summary.active_alerts_count
            },
            "recent_metrics_count": self.recent_metrics.len(),
            "active_alerts": self.active_alerts.iter().map(|alert| {
                json!({
                    "type": format!("{:?}", alert.alert_type),
                    "message": alert.message,
                    "severity": format!("{:?}", alert.severity),
                    "timestamp": alert.timestamp.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    "details": alert.details
                })
            }).collect::<Vec<_>>(),
            "recommendations": self.recommendations
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_collector_creation() {
        let config = CacheTelemetryConfig::default();
        let collector = CacheTelemetryCollector::new(config);

        assert!(collector._collection_handle.is_none());
    }

    #[tokio::test]
    async fn test_alert_processing() {
        let config = CacheTelemetryConfig {
            alerting_enabled: true,
            hit_rate_alert_threshold: 0.8,                   // 80%
            memory_alert_threshold: Some(100 * 1024 * 1024), // 100MB
            ..Default::default()
        };

        let metrics = CacheTelemetryMetrics {
            timestamp: SystemTime::now(),
            cache_stats: None,
            file_watcher_stats: None,
            performance_metrics: CachePerformanceMetrics {
                overall_hit_rate: 0.5, // 50% - below threshold
                ast_hit_rate: 0.5,
                results_hit_rate: 0.5,
                total_memory_usage_bytes: 200 * 1024 * 1024, // 200MB - above threshold
                ast_memory_usage_bytes: 100 * 1024 * 1024,
                results_memory_usage_bytes: 100 * 1024 * 1024,
                total_entries: 1000,
                eviction_rate_per_minute: 5.0,
                invalidation_rate_per_minute: 2.0,
            },
            alerts: Vec::new(),
        };

        let alerts = CacheTelemetryCollector::process_alerts(&config, &metrics).await;

        assert_eq!(alerts.len(), 2); // Should have hit rate and memory alerts
        assert!(alerts
            .iter()
            .any(|a| a.alert_type == CacheAlertType::LowHitRate));
        assert!(alerts
            .iter()
            .any(|a| a.alert_type == CacheAlertType::HighMemoryUsage));
    }

    #[tokio::test]
    async fn test_report_generation() {
        let config = CacheTelemetryConfig::default();
        let mut collector = CacheTelemetryCollector::new(config);

        let report = collector.generate_report().await;
        let json_report = report.to_json();

        assert!(json_report.is_object());
        assert!(json_report.get("summary").is_some());
        assert!(json_report.get("recommendations").is_some());
    }
}
