//! Database Monitoring and Health Check System
//!
//! This module provides comprehensive monitoring, alerting, and health checking
//! capabilities for the scalable database system.

use super::connection::providers::{DatabaseHealthStatus, DatabaseMetrics, DatabaseProvider};
use super::scalable_manager::{LoadBalancerStats, ScalableDatabase};
use crate::error::{Result, UveddiError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Database monitoring system
pub struct DatabaseMonitor {
    database: Arc<ScalableDatabase>,
    config: MonitoringConfig,
    metrics_history: Arc<RwLock<MetricsHistory>>,
    alert_sender: broadcast::Sender<Alert>,
    _alert_receiver: broadcast::Receiver<Alert>,
}

impl DatabaseMonitor {
    /// Create a new database monitor
    pub fn new(database: Arc<ScalableDatabase>, config: MonitoringConfig) -> Self {
        let (alert_sender, alert_receiver) = broadcast::channel(1000);

        Self {
            database,
            config,
            metrics_history: Arc::new(RwLock::new(MetricsHistory::new())),
            alert_sender,
            _alert_receiver: alert_receiver,
        }
    }

    /// Start monitoring background tasks
    pub fn start_monitoring(&self) -> MonitoringHandle {
        let database = self.database.clone();
        let config = self.config.clone();
        let metrics_history = self.metrics_history.clone();
        let alert_sender = self.alert_sender.clone();

        // Health check task
        let health_check_handle = {
            let database = database.clone();
            let alert_sender = alert_sender.clone();
            let interval_duration = config.health_check_interval;

            tokio::spawn(async move {
                let mut interval = interval(interval_duration);

                loop {
                    interval.tick().await;

                    match Self::perform_health_check(&database, &alert_sender).await {
                        Ok(_) => debug!("Health check completed successfully"),
                        Err(e) => error!("Health check failed: {}", e),
                    }
                }
            })
        };

        // Metrics collection task
        let metrics_collection_handle = {
            let database = database.clone();
            let metrics_history = metrics_history.clone();
            let alert_sender = alert_sender.clone();
            let config = config.clone();
            let interval_duration = config.metrics_collection_interval;

            tokio::spawn(async move {
                let mut interval = interval(interval_duration);

                loop {
                    interval.tick().await;

                    match Self::collect_metrics(&database, &metrics_history, &alert_sender, &config)
                        .await
                    {
                        Ok(_) => debug!("Metrics collection completed successfully"),
                        Err(e) => error!("Metrics collection failed: {}", e),
                    }
                }
            })
        };

        // Cleanup task
        let cleanup_handle = {
            let database = database.clone();
            let metrics_history = metrics_history.clone();
            let interval_duration = config.cleanup_interval;
            let retention_period = config.metrics_retention_period;

            tokio::spawn(async move {
                let mut interval = interval(interval_duration);

                loop {
                    interval.tick().await;

                    // Cleanup expired connections
                    if let Err(e) = database.cleanup().await {
                        warn!("Database cleanup failed: {}", e);
                    }

                    // Cleanup old metrics
                    let mut history = metrics_history.write().await;
                    history.cleanup_old_metrics(retention_period);
                }
            })
        };

        MonitoringHandle {
            health_check_handle,
            metrics_collection_handle,
            cleanup_handle,
        }
    }

    /// Perform database health check
    async fn perform_health_check(
        database: &ScalableDatabase,
        alert_sender: &broadcast::Sender<Alert>,
    ) -> Result<()> {
        let health_status = database.get_health_status().await?;

        // Check if database is unhealthy
        if !health_status.is_healthy {
            let alert = Alert {
                severity: AlertSeverity::Critical,
                alert_type: AlertType::DatabaseUnhealthy,
                message: "Database health check failed".to_string(),
                timestamp: SystemTime::now(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert(
                        "active_connections".to_string(),
                        health_status.active_connections.to_string(),
                    );
                    map.insert(
                        "pool_utilization".to_string(),
                        format!("{:.2}%", health_status.pool_utilization * 100.0),
                    );
                    map.insert(
                        "error_count".to_string(),
                        health_status.error_count.to_string(),
                    );
                    map
                },
            };

            let _ = alert_sender.send(alert);
        }

        // Check pool utilization
        if health_status.pool_utilization > 0.9 {
            let alert = Alert {
                severity: AlertSeverity::Warning,
                alert_type: AlertType::HighPoolUtilization,
                message: format!(
                    "High pool utilization: {:.1}%",
                    health_status.pool_utilization * 100.0
                ),
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
            };

            let _ = alert_sender.send(alert);
        }

        // Check query response time
        if health_status.average_query_time > Duration::from_millis(1000) {
            let alert = Alert {
                severity: AlertSeverity::Warning,
                alert_type: AlertType::SlowQueries,
                message: format!("Average query time: {:?}", health_status.average_query_time),
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
            };

            let _ = alert_sender.send(alert);
        }

        Ok(())
    }

    /// Collect database metrics
    async fn collect_metrics(
        database: &ScalableDatabase,
        metrics_history: &Arc<RwLock<MetricsHistory>>,
        alert_sender: &broadcast::Sender<Alert>,
        config: &MonitoringConfig,
    ) -> Result<()> {
        let timestamp = SystemTime::now();
        let health_status = database.get_health_status().await?;
        let load_balancer_stats = database.get_load_balancer_stats();

        let snapshot = MetricsSnapshot {
            timestamp,
            health_status,
            load_balancer_stats,
            custom_metrics: Self::collect_custom_metrics(database).await,
        };

        // Store metrics
        {
            let mut history = metrics_history.write().await;
            history.add_snapshot(snapshot.clone());
        }

        // Check for anomalies
        Self::check_metrics_anomalies(&snapshot, alert_sender, config).await;

        Ok(())
    }

    /// Collect custom metrics specific to the application
    async fn collect_custom_metrics(database: &ScalableDatabase) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        // Example: Get recent analysis runs count
        if let Ok(recent_runs) = database.get_recent_analysis_runs(10).await {
            metrics.insert("recent_analysis_runs".to_string(), recent_runs.len() as f64);
        }

        // Add more custom metrics as needed
        metrics.insert(
            "uptime_seconds".to_string(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as f64,
        );

        metrics
    }

    /// Check metrics for anomalies and send alerts
    async fn check_metrics_anomalies(
        snapshot: &MetricsSnapshot,
        alert_sender: &broadcast::Sender<Alert>,
        config: &MonitoringConfig,
    ) {
        // Check error rate
        let total_queries = snapshot.custom_metrics.get("total_queries").unwrap_or(&0.0);
        let failed_queries = snapshot.health_status.error_count as f64;

        if *total_queries > 0.0 {
            let error_rate = failed_queries / total_queries;
            if error_rate > config.error_rate_threshold {
                let alert = Alert {
                    severity: AlertSeverity::Critical,
                    alert_type: AlertType::HighErrorRate,
                    message: format!("High error rate: {:.2}%", error_rate * 100.0),
                    timestamp: SystemTime::now(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("error_rate".to_string(), format!("{:.4}", error_rate));
                        map.insert("total_queries".to_string(), total_queries.to_string());
                        map.insert("failed_queries".to_string(), failed_queries.to_string());
                        map
                    },
                };

                let _ = alert_sender.send(alert);
            }
        }

        // Check for connection leaks
        if snapshot.health_status.active_connections > config.max_connections_threshold {
            let alert = Alert {
                severity: AlertSeverity::Warning,
                alert_type: AlertType::ConnectionLeak,
                message: format!(
                    "High number of active connections: {}",
                    snapshot.health_status.active_connections
                ),
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
            };

            let _ = alert_sender.send(alert);
        }
    }

    /// Get current database metrics
    pub async fn get_current_metrics(&self) -> Result<MetricsSnapshot> {
        let timestamp = SystemTime::now();
        let health_status = self.database.get_health_status().await?;
        let load_balancer_stats = self.database.get_load_balancer_stats();
        let custom_metrics = Self::collect_custom_metrics(&self.database).await;

        Ok(MetricsSnapshot {
            timestamp,
            health_status,
            load_balancer_stats,
            custom_metrics,
        })
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self, duration: Duration) -> Vec<MetricsSnapshot> {
        let history = self.metrics_history.read().await;
        history.get_snapshots_since(duration)
    }

    /// Subscribe to alerts
    pub fn subscribe_to_alerts(&self) -> broadcast::Receiver<Alert> {
        self.alert_sender.subscribe()
    }

    /// Generate monitoring report
    pub async fn generate_report(&self, duration: Duration) -> MonitoringReport {
        let snapshots = self.get_metrics_history(duration).await;

        if snapshots.is_empty() {
            return MonitoringReport::default();
        }

        let total_snapshots = snapshots.len();
        let healthy_snapshots = snapshots
            .iter()
            .filter(|s| s.health_status.is_healthy)
            .count();
        let uptime_percentage = (healthy_snapshots as f64 / total_snapshots as f64) * 100.0;

        let avg_response_time = if !snapshots.is_empty() {
            snapshots
                .iter()
                .map(|s| s.health_status.average_query_time)
                .sum::<Duration>()
                / snapshots.len() as u32
        } else {
            Duration::ZERO
        };

        let max_connections = snapshots
            .iter()
            .map(|s| s.health_status.active_connections)
            .max()
            .unwrap_or(0);

        let total_errors = snapshots
            .last()
            .map(|s| s.health_status.error_count)
            .unwrap_or(0);

        MonitoringReport {
            period: duration,
            uptime_percentage,
            average_response_time: avg_response_time,
            peak_connections: max_connections,
            total_errors,
            health_checks_performed: total_snapshots as u64,
            alerts_generated: 0, // Would need to track this separately
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub health_check_interval: Duration,
    pub metrics_collection_interval: Duration,
    pub cleanup_interval: Duration,
    pub metrics_retention_period: Duration,
    pub error_rate_threshold: f64,
    pub max_connections_threshold: u32,
    pub response_time_threshold: Duration,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            metrics_collection_interval: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            metrics_retention_period: Duration::from_secs(86400), // 24 hours
            error_rate_threshold: 0.05,                 // 5%
            max_connections_threshold: 100,
            response_time_threshold: Duration::from_millis(1000),
        }
    }
}

/// Monitoring task handles
pub struct MonitoringHandle {
    pub health_check_handle: tokio::task::JoinHandle<()>,
    pub metrics_collection_handle: tokio::task::JoinHandle<()>,
    pub cleanup_handle: tokio::task::JoinHandle<()>,
}

impl MonitoringHandle {
    /// Stop all monitoring tasks
    pub fn stop(self) {
        self.health_check_handle.abort();
        self.metrics_collection_handle.abort();
        self.cleanup_handle.abort();
    }
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: SystemTime,
    pub health_status: DatabaseHealthStatus,
    pub load_balancer_stats: LoadBalancerStats,
    pub custom_metrics: HashMap<String, f64>,
}

/// Metrics history storage
struct MetricsHistory {
    snapshots: Vec<MetricsSnapshot>,
    max_snapshots: usize,
}

impl MetricsHistory {
    fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: 10000, // Keep last 10k snapshots
        }
    }

    fn add_snapshot(&mut self, snapshot: MetricsSnapshot) {
        self.snapshots.push(snapshot);

        // Keep only recent snapshots
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots
                .drain(0..self.snapshots.len() - self.max_snapshots);
        }
    }

    fn get_snapshots_since(&self, duration: Duration) -> Vec<MetricsSnapshot> {
        let cutoff = SystemTime::now() - duration;

        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.timestamp >= cutoff)
            .cloned()
            .collect()
    }

    fn cleanup_old_metrics(&mut self, retention_period: Duration) {
        let cutoff = SystemTime::now() - retention_period;
        self.snapshots
            .retain(|snapshot| snapshot.timestamp >= cutoff);
    }
}

/// Alert system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    DatabaseUnhealthy,
    HighPoolUtilization,
    SlowQueries,
    HighErrorRate,
    ConnectionLeak,
    DiskSpaceLow,
    MemoryUsageHigh,
}

/// Monitoring report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub period: Duration,
    pub uptime_percentage: f64,
    pub average_response_time: Duration,
    pub peak_connections: u32,
    pub total_errors: u64,
    pub health_checks_performed: u64,
    pub alerts_generated: u64,
}

impl Default for MonitoringReport {
    fn default() -> Self {
        Self {
            period: Duration::ZERO,
            uptime_percentage: 0.0,
            average_response_time: Duration::ZERO,
            peak_connections: 0,
            total_errors: 0,
            health_checks_performed: 0,
            alerts_generated: 0,
        }
    }
}

/// Alert handler trait for custom alert processing
pub trait AlertHandler: Send + Sync {
    fn handle_alert(&self, alert: Alert) -> Result<()>;
}

/// Console alert handler (logs alerts to console)
pub struct ConsoleAlertHandler;

impl AlertHandler for ConsoleAlertHandler {
    fn handle_alert(&self, alert: Alert) -> Result<()> {
        match alert.severity {
            AlertSeverity::Info => info!(
                "[ALERT] {}: {}",
                format!("{:?}", alert.alert_type),
                alert.message
            ),
            AlertSeverity::Warning => warn!(
                "[ALERT] {}: {}",
                format!("{:?}", alert.alert_type),
                alert.message
            ),
            AlertSeverity::Critical => error!(
                "[ALERT] {}: {}",
                format!("{:?}", alert.alert_type),
                alert.message
            ),
        }
        Ok(())
    }
}

/// Multi-handler for processing alerts with multiple handlers
pub struct MultiAlertHandler {
    handlers: Vec<Box<dyn AlertHandler>>,
}

impl MultiAlertHandler {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(mut self, handler: Box<dyn AlertHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Process alerts from a receiver
    pub async fn process_alerts(self, mut alert_receiver: broadcast::Receiver<Alert>) {
        while let Ok(alert) = alert_receiver.recv().await {
            for handler in &self.handlers {
                if let Err(e) = handler.handle_alert(alert.clone()) {
                    error!("Alert handler failed: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_history_cleanup() {
        let mut history = MetricsHistory::new();
        let now = SystemTime::now();

        // Add some old metrics
        for i in 0..5 {
            let timestamp = now - Duration::from_secs(3600 * (i + 1)); // 1-5 hours ago
            let snapshot = MetricsSnapshot {
                timestamp,
                health_status: DatabaseHealthStatus {
                    is_healthy: true,
                    active_connections: 10,
                    pool_utilization: 0.5,
                    last_successful_query: 0,
                    error_count: 0,
                    average_query_time: Duration::from_millis(100),
                },
                load_balancer_stats: LoadBalancerStats {
                    total_providers: 1,
                    healthy_providers: 1,
                    total_requests: 100,
                    current_provider: 0,
                },
                custom_metrics: HashMap::new(),
            };
            history.add_snapshot(snapshot);
        }

        assert_eq!(history.snapshots.len(), 5);

        // Cleanup metrics older than 2 hours
        // Note: cleanup uses SystemTime::now() which is slightly later than test's `now`,
        // so only the 1-hour-old snapshot reliably remains (2-hour boundary is a race)
        history.cleanup_old_metrics(Duration::from_secs(7200));
        assert_eq!(history.snapshots.len(), 1);
    }
}
