//! Resource monitoring and alerting system

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::{
    analysis_orchestrator::OrchestrationStats,
    error::{ResourceError, ResourceResult},
    memory_tracker::MemoryTracker,
    metrics::{MetricsHistory, ResourceUsage, SystemMetrics, SystemMetricsCollector},
    resource_config::{MonitoringConfig, ResourceConfig},
};

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Resource alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub resource_type: String,
    pub message: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub timestamp: u64,
    pub acknowledged: bool,
}

/// Alert thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Memory usage warning threshold (0.0-1.0)
    pub memory_warning: f64,
    /// Memory usage critical threshold (0.0-1.0)
    pub memory_critical: f64,
    /// CPU usage warning threshold (0.0-1.0)
    pub cpu_warning: f64,
    /// CPU usage critical threshold (0.0-1.0)
    pub cpu_critical: f64,
    /// Analysis queue size warning threshold
    pub queue_warning: usize,
    /// Analysis queue size critical threshold
    pub queue_critical: usize,
    /// Analysis failure rate warning threshold (failures per minute)
    pub failure_rate_warning: f64,
    /// Analysis failure rate critical threshold (failures per minute)
    pub failure_rate_critical: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            memory_warning: 0.7,
            memory_critical: 0.9,
            cpu_warning: 0.8,
            cpu_critical: 0.95,
            queue_warning: 20,
            queue_critical: 50,
            failure_rate_warning: 5.0,
            failure_rate_critical: 10.0,
        }
    }
}

/// Resource monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Current resource usage
    pub current_usage: ResourceUsage,
    /// System metrics
    pub system_metrics: Option<SystemMetrics>,
    /// Analysis orchestration statistics
    pub orchestration_stats: OrchestrationStats,
    /// Active alerts
    pub active_alerts: Vec<ResourceAlert>,
    /// Monitoring uptime in seconds
    pub uptime_seconds: u64,
    /// Last monitoring update timestamp
    pub last_update: u64,
}

/// Resource monitoring and alerting system
pub struct ResourceMonitor {
    memory_tracker: Arc<MemoryTracker>,
    config: Arc<RwLock<ResourceConfig>>,
    system_collector: SystemMetricsCollector,
    metrics_history: Arc<Mutex<MetricsHistory>>,
    active_alerts: Arc<Mutex<Vec<ResourceAlert>>>,
    alert_thresholds: Arc<RwLock<AlertThresholds>>,
    monitoring_start: Instant,
    last_metrics_collection: Arc<Mutex<Option<Instant>>>,
    alert_callbacks: Arc<Mutex<Vec<AlertCallback>>>,
}

/// Callback function type for alert notifications
type AlertCallback = Box<dyn Fn(&ResourceAlert) + Send + Sync>;

impl ResourceMonitor {
    /// Creates a new resource monitor
    pub fn new(memory_tracker: Arc<MemoryTracker>, config: Arc<RwLock<ResourceConfig>>) -> Self {
        let monitoring_config = config.blocking_read().monitoring.clone();

        Self {
            memory_tracker,
            config,
            system_collector: SystemMetricsCollector::new(),
            metrics_history: Arc::new(Mutex::new(MetricsHistory::new(
                monitoring_config.history_size,
            ))),
            active_alerts: Arc::new(Mutex::new(Vec::new())),
            alert_thresholds: Arc::new(RwLock::new(AlertThresholds::default())),
            monitoring_start: Instant::now(),
            last_metrics_collection: Arc::new(Mutex::new(None)),
            alert_callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Starts resource monitoring in the background
    pub async fn start_monitoring(&self) -> ResourceResult<()> {
        let config = self.config.read().await;
        if !config.monitoring.enabled {
            return Ok(());
        }

        let interval_duration = Duration::from_secs(config.monitoring.interval_seconds);
        drop(config);

        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.monitoring_loop(interval_duration).await;
        });

        Ok(())
    }

    /// Registers a callback for alert notifications
    pub fn register_alert_callback<F>(&self, callback: F)
    where
        F: Fn(&ResourceAlert) + Send + Sync + 'static,
    {
        self.alert_callbacks
            .lock()
            .unwrap()
            .push(Box::new(callback));
    }

    /// Updates alert thresholds
    pub async fn update_alert_thresholds(&self, thresholds: AlertThresholds) {
        *self.alert_thresholds.write().await = thresholds;
    }

    /// Gets current resource metrics
    pub async fn get_current_metrics(&self) -> ResourceResult<ResourceMetrics> {
        // Collect current resource usage
        let memory_stats = self.memory_tracker.get_usage_stats();
        let current_usage = ResourceUsage {
            memory: memory_stats,
            active_analyses: 0, // Would be provided by orchestrator
            degradation_level: super::degradation_manager::DegradationLevel::Normal, // Would be provided by degradation manager
        };

        // Try to collect system metrics
        let system_metrics = match self.system_collector.clone().collect().await {
            Ok(metrics) => Some(metrics),
            Err(_) => None, // System metrics collection failed, continue without them
        };

        let orchestration_stats = OrchestrationStats::default(); // Would be provided by orchestrator
        let active_alerts = self.active_alerts.lock().unwrap().clone();

        let uptime_seconds = self.monitoring_start.elapsed().as_secs();
        let last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(ResourceMetrics {
            current_usage,
            system_metrics,
            orchestration_stats,
            active_alerts,
            uptime_seconds,
            last_update,
        })
    }

    /// Gets historical metrics for a time period
    pub fn get_metrics_history(
        &self,
        duration: Duration,
    ) -> (Vec<ResourceUsage>, Vec<SystemMetrics>) {
        let history = self.metrics_history.lock().unwrap();
        let resource_history = history
            .get_resource_usage_history(duration)
            .into_iter()
            .map(|(_, usage)| usage)
            .collect();
        let system_history = history
            .get_system_metrics_history(duration)
            .into_iter()
            .map(|(_, metrics)| metrics)
            .collect();

        (resource_history, system_history)
    }

    /// Acknowledges an alert by ID
    pub fn acknowledge_alert(&self, alert_id: &str) -> ResourceResult<()> {
        let mut alerts = self.active_alerts.lock().unwrap();

        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(ResourceError::ResourceUnavailable(format!(
                "Alert with ID {} not found",
                alert_id
            )))
        }
    }

    /// Clears acknowledged alerts older than specified duration
    pub fn clear_acknowledged_alerts(&self, older_than: Duration) {
        let mut alerts = self.active_alerts.lock().unwrap();
        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - older_than.as_secs();

        alerts.retain(|alert| !alert.acknowledged || alert.timestamp > cutoff_time);
    }

    /// Triggers memory pressure response
    pub async fn trigger_memory_pressure_response(&self) -> ResourceResult<()> {
        let memory_stats = self.memory_tracker.get_usage_stats();

        let alert = ResourceAlert {
            id: format!("memory_pressure_{}", chrono::Utc::now().timestamp()),
            severity: if memory_stats.usage_percent > 90.0 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            },
            resource_type: "memory".to_string(),
            message: format!(
                "Memory usage is high: {:.1}% ({} MB used of {} MB)",
                memory_stats.usage_percent,
                memory_stats.current / 1024 / 1024,
                memory_stats.limit / 1024 / 1024
            ),
            current_value: memory_stats.usage_percent,
            threshold_value: if memory_stats.usage_percent > 90.0 {
                90.0
            } else {
                70.0
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            acknowledged: false,
        };

        self.trigger_alert(alert).await;
        Ok(())
    }

    /// Triggers emergency cleanup when memory is critically low
    pub async fn trigger_emergency_cleanup(&self) -> ResourceResult<()> {
        let alert = ResourceAlert {
            id: format!("emergency_cleanup_{}", chrono::Utc::now().timestamp()),
            severity: AlertSeverity::Emergency,
            resource_type: "memory".to_string(),
            message: "Emergency memory cleanup triggered - system at critical resource levels"
                .to_string(),
            current_value: self.memory_tracker.get_memory_pressure(),
            threshold_value: 0.95,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            acknowledged: false,
        };

        self.trigger_alert(alert).await;

        // Perform emergency cleanup actions
        // This would integrate with other components to:
        // - Cancel non-critical analyses
        // - Clear caches
        // - Release temporary resources

        Ok(())
    }

    async fn monitoring_loop(&self, interval_duration: Duration) {
        let mut interval = interval(interval_duration);

        loop {
            interval.tick().await;

            if let Err(e) = self.collect_and_check_metrics().await {
                eprintln!("Error during monitoring: {}", e);
            }
        }
    }

    async fn collect_and_check_metrics(&self) -> ResourceResult<()> {
        // Collect resource usage
        let memory_stats = self.memory_tracker.get_usage_stats();
        let current_usage = ResourceUsage {
            memory: memory_stats,
            active_analyses: 0, // Would be provided by orchestrator
            degradation_level: super::degradation_manager::DegradationLevel::Normal,
        };

        // Record in history
        {
            let mut history = self.metrics_history.lock().unwrap();
            history.record_resource_usage(current_usage.clone());
        }

        // Collect system metrics if enabled
        let config = self.config.read().await;
        if config.monitoring.enable_metrics_export {
            match self.system_collector.clone().collect().await {
                Ok(system_metrics) => {
                    let mut history = self.metrics_history.lock().unwrap();
                    history.record_system_metrics(system_metrics);
                }
                Err(e) => {
                    eprintln!("Failed to collect system metrics: {}", e);
                }
            }
        }

        if config.monitoring.log_usage {
            info!(
                "Resource usage - Memory: {:.1}% ({}/{} MB), Active analyses: {}",
                current_usage.memory.usage_percent,
                current_usage.memory.current / 1024 / 1024,
                current_usage.memory.limit / 1024 / 1024,
                current_usage.active_analyses
            );
        }

        drop(config);

        // Check thresholds and generate alerts
        self.check_thresholds(&current_usage).await?;

        // Update last collection time
        *self.last_metrics_collection.lock().unwrap() = Some(Instant::now());

        Ok(())
    }

    async fn check_thresholds(&self, usage: &ResourceUsage) -> ResourceResult<()> {
        let thresholds = self.alert_thresholds.read().await;

        // Check memory thresholds
        if usage.memory.usage_percent / 100.0 > thresholds.memory_critical {
            let alert = ResourceAlert {
                id: format!("memory_critical_{}", chrono::Utc::now().timestamp()),
                severity: AlertSeverity::Critical,
                resource_type: "memory".to_string(),
                message: format!(
                    "Memory usage critical: {:.1}% exceeds {:.1}% threshold",
                    usage.memory.usage_percent,
                    thresholds.memory_critical * 100.0
                ),
                current_value: usage.memory.usage_percent / 100.0,
                threshold_value: thresholds.memory_critical,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                acknowledged: false,
            };

            self.trigger_alert(alert).await;
        } else if usage.memory.usage_percent / 100.0 > thresholds.memory_warning {
            let alert = ResourceAlert {
                id: format!("memory_warning_{}", chrono::Utc::now().timestamp()),
                severity: AlertSeverity::Warning,
                resource_type: "memory".to_string(),
                message: format!(
                    "Memory usage warning: {:.1}% exceeds {:.1}% threshold",
                    usage.memory.usage_percent,
                    thresholds.memory_warning * 100.0
                ),
                current_value: usage.memory.usage_percent / 100.0,
                threshold_value: thresholds.memory_warning,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                acknowledged: false,
            };

            self.trigger_alert(alert).await;
        }

        Ok(())
    }

    async fn trigger_alert(&self, alert: ResourceAlert) {
        // Add to active alerts
        {
            let mut alerts = self.active_alerts.lock().unwrap();

            // Remove similar alerts to prevent spam
            alerts.retain(|existing| {
                existing.resource_type != alert.resource_type
                    || existing.severity != alert.severity
                    || existing.timestamp < alert.timestamp - 300 // 5 minutes
            });

            alerts.push(alert.clone());
        }

        // Notify callbacks
        let callbacks = self.alert_callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(&alert);
        }
    }
}

impl Clone for ResourceMonitor {
    fn clone(&self) -> Self {
        Self {
            memory_tracker: self.memory_tracker.clone(),
            config: self.config.clone(),
            system_collector: SystemMetricsCollector::new(),
            metrics_history: self.metrics_history.clone(),
            active_alerts: self.active_alerts.clone(),
            alert_thresholds: self.alert_thresholds.clone(),
            monitoring_start: self.monitoring_start,
            last_metrics_collection: self.last_metrics_collection.clone(),
            alert_callbacks: self.alert_callbacks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Arc<RwLock<ResourceConfig>> {
        Arc::new(RwLock::new(ResourceConfig::testing()))
    }

    #[tokio::test]
    async fn test_resource_monitor_creation() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker, config);

        let metrics = monitor.get_current_metrics().await.unwrap();
        assert_eq!(metrics.active_alerts.len(), 0);
        assert!(metrics.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker, config);

        // Manually add an alert
        let alert = ResourceAlert {
            id: "test_alert".to_string(),
            severity: AlertSeverity::Warning,
            resource_type: "memory".to_string(),
            message: "Test alert".to_string(),
            current_value: 0.8,
            threshold_value: 0.7,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            acknowledged: false,
        };

        monitor.active_alerts.lock().unwrap().push(alert);

        // Acknowledge the alert
        assert!(monitor.acknowledge_alert("test_alert").is_ok());

        let alerts = monitor.active_alerts.lock().unwrap();
        assert!(alerts[0].acknowledged);
    }

    #[tokio::test]
    async fn test_memory_pressure_alert() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker.clone(), config);

        // Allocate memory to trigger pressure
        let _guard = memory_tracker.allocate("test", 800).unwrap();

        // Trigger memory pressure response
        monitor.trigger_memory_pressure_response().await.unwrap();

        let alerts = monitor.active_alerts.lock().unwrap();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].resource_type, "memory");
    }
}
