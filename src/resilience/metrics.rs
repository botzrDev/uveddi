//! Metrics collection for error tracking and monitoring

use crate::error::RenderingServiceError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_severity: HashMap<String, u64>,
    pub error_rate_per_minute: f64,
    pub last_updated: SystemTime,
    pub time_window_minutes: u32,
}

#[derive(Debug, Clone)]
pub struct ErrorEvent {
    pub timestamp: SystemTime,
    pub category: String,
    pub severity: String,
    pub error_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorTrend {
    pub time_window: String,
    pub error_count: u64,
    pub error_rate: f64,
    pub top_categories: Vec<(String, u64)>,
}

#[derive(Debug, Clone)]
pub enum MetricsFormat {
    Json,
    Prometheus,
    InfluxDb,
}

pub struct MetricsCollector {
    events: Arc<Mutex<Vec<ErrorEvent>>>,
    cleanup_handle: tokio::task::JoinHandle<()>,
    config: MetricsConfig,
}

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub retention_duration: Duration,
    pub trend_window_minutes: u32,
    pub cleanup_interval: Duration,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            retention_duration: Duration::from_secs(3600), // 1 hour
            trend_window_minutes: 5,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl MetricsCollector {
    // Remove duplicate new method

    pub fn with_default_config() -> Self {
        Self::new(MetricsConfig::default())
    }

    pub async fn record_error(&self, error: &RenderingServiceError) {
        let category = error.category().as_str().to_string();
        let severity = error.severity().as_str().to_string();
        let error_type = error.to_string();

        let event = ErrorEvent {
            timestamp: SystemTime::now(),
            category: category.clone(),
            severity: severity.clone(),
            error_type: error_type.clone(),
        };

        // Lock and record the event
        let mut events = self.events.lock().await;
        events.push(event);
        drop(events); // Release lock before logging

        info!(category, severity, error_type, "Recorded error event");

        // No need to trigger cleanup here - it's handled by the background task
    }

    pub fn new(config: MetricsConfig) -> Self {
        let events: Arc<Mutex<Vec<ErrorEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let cleanup_handle = tokio::spawn({
            let events = Arc::clone(&events);
            let config = config.clone();
            async move {
                loop {
                    tokio::time::sleep(config.cleanup_interval).await;
                    let mut events = events.lock().await;
                    let now = SystemTime::now();
                    events.retain(|e| {
                        now.duration_since(e.timestamp).unwrap_or(Duration::ZERO)
                            <= config.retention_duration
                    });

                    // Shrink the vector if it's using too much memory
                    if events.capacity() > events.len() * 2 {
                        events.shrink_to_fit();
                    }
                }
            }
        });

        Self {
            events,
            cleanup_handle,
            config,
        }
    }

    pub async fn calculate_error_rate(&self) -> f64 {
        let now = SystemTime::now();
        let events = self.events.lock().await;
        let one_minute_ago = now - Duration::from_secs(60);

        // Count events in the last minute
        events
            .iter()
            .filter(|e| e.timestamp >= one_minute_ago)
            .count() as f64
    }

    pub async fn get_metrics_snapshot(&self) -> ErrorMetrics {
        // First collect counts without holding the lock during rate calculation
        let (total_errors, errors_by_category, errors_by_severity) = {
            let events = self.events.lock().await;
            let total_errors = events.len() as u64;

            let mut errors_by_category = HashMap::new();
            let mut errors_by_severity = HashMap::new();

            for event in events.iter() {
                *errors_by_category
                    .entry(event.category.clone())
                    .or_insert(0) += 1;
                *errors_by_severity
                    .entry(event.severity.clone())
                    .or_insert(0) += 1;
            }

            (total_errors, errors_by_category, errors_by_severity)
        };

        // Now calculate error rate without holding the lock
        let error_rate_per_minute = self.calculate_error_rate().await;

        ErrorMetrics {
            total_errors,
            errors_by_category,
            errors_by_severity,
            error_rate_per_minute,
            last_updated: SystemTime::now(),
            time_window_minutes: 1,
        }
    }

    pub async fn export_metrics(&self, format: &MetricsFormat) -> String {
        let metrics = self.get_metrics_snapshot().await;
        match format {
            MetricsFormat::Json => self.format_as_json(&metrics),
            MetricsFormat::Prometheus => self.format_as_prometheus(&metrics),
            MetricsFormat::InfluxDb => self.format_as_influxdb(&metrics),
        }
    }

    pub async fn generate_trends(&self) -> Vec<ErrorTrend> {
        let now = SystemTime::now();
        let events = self.events.lock().await;
        let mut trends = Vec::new();
        let window_count = 12; // 12 windows of 5 minutes = 60 minutes

        for i in 0..window_count {
            let window_end = now - Duration::from_secs((i * 5 * 60) as u64);
            let window_start = window_end - Duration::from_secs(5 * 60);

            let window_events: Vec<_> = events
                .iter()
                .filter(|e| e.timestamp >= window_start && e.timestamp < window_end)
                .collect();

            let error_count = window_events.len() as u64;
            let error_rate = error_count as f64 / 5.0; // Errors per minute

            let mut category_counts = HashMap::new();
            for event in &window_events {
                *category_counts.entry(event.category.clone()).or_insert(0) += 1;
            }

            let mut top_categories: Vec<_> = category_counts.into_iter().collect();
            top_categories.sort_by(|a, b| b.1.cmp(&a.1));
            top_categories.truncate(3);

            trends.push(ErrorTrend {
                time_window: format!("{}m-{}m", i * 5, (i + 1) * 5),
                error_count,
                error_rate,
                top_categories,
            });
        }

        trends
    }

    pub async fn reset_metrics(&self) {
        let mut events = self.events.lock().await;
        events.clear();
        info!("Metrics collector has been reset");
    }

    async fn cleanup_old_events(&self) {
        let now = SystemTime::now();
        let mut events = self.events.lock().await;

        events.retain(|e| {
            now.duration_since(e.timestamp).unwrap_or(Duration::ZERO)
                <= self.config.retention_duration
        });

        if events.capacity() > events.len() * 2 {
            events.shrink_to_fit();
        }
    }

    fn format_as_json(&self, metrics: &ErrorMetrics) -> String {
        serde_json::to_string_pretty(metrics).unwrap()
    }

    fn format_as_prometheus(&self, metrics: &ErrorMetrics) -> String {
        let mut output = String::new();
        output.push_str("# HELP uveddi_total_errors Total number of errors\n");
        output.push_str("# TYPE uveddi_total_errors counter\n");
        output.push_str(&format!("uveddi_total_errors {}\n", metrics.total_errors));

        output.push_str("# HELP uveddi_error_rate_per_minute Errors per minute\n");
        output.push_str("# TYPE uveddi_error_rate_per_minute gauge\n");
        output.push_str(&format!(
            "uveddi_error_rate_per_minute {}\n",
            metrics.error_rate_per_minute
        ));

        for (category, count) in &metrics.errors_by_category {
            output.push_str(&format!(
                "uveddi_errors_by_category{{category=\"{}\"}} {}\n",
                category, count
            ));
        }

        for (severity, count) in &metrics.errors_by_severity {
            output.push_str(&format!(
                "uveddi_errors_by_severity{{severity=\"{}\"}} {}\n",
                severity, count
            ));
        }

        output
    }

    fn format_as_influxdb(&self, metrics: &ErrorMetrics) -> String {
        let mut output = String::new();
        let timestamp = metrics
            .last_updated
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        output.push_str(&format!(
            "error_metrics total_errors={},error_rate_per_minute={} {}\n",
            metrics.total_errors, metrics.error_rate_per_minute, timestamp
        ));

        for (category, count) in &metrics.errors_by_category {
            output.push_str(&format!(
                "error_metrics_by_category,category={} count={} {}\n",
                category, count, timestamp
            ));
        }

        for (severity, count) in &metrics.errors_by_severity {
            output.push_str(&format!(
                "error_metrics_by_severity,severity={} count={} {}\n",
                severity, count, timestamp
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RenderingServiceError;
    use std::time::Duration;
    use tokio::task;

    #[tokio::test]
    async fn test_error_recording() {
        let collector = MetricsCollector::with_default_config();
        let error = RenderingServiceError::ServiceUnavailable;

        collector.record_error(&error).await;
        let metrics = collector.get_metrics_snapshot().await;

        assert_eq!(metrics.total_errors, 1);
        assert_eq!(
            *metrics
                .errors_by_category
                .get("service_communication")
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_thread_safety() {
        let collector = Arc::new(MetricsCollector::with_default_config());
        let mut handles = vec![];

        for _ in 0..10 {
            let collector = Arc::clone(&collector);
            handles.push(task::spawn(async move {
                for _ in 0..100 {
                    let error = RenderingServiceError::ServiceUnavailable;
                    collector.record_error(&error).await;
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let metrics = collector.get_metrics_snapshot().await;
        assert_eq!(
            metrics.total_errors, 1000,
            "All 1000 errors should be recorded across threads"
        );
    }

    #[tokio::test]
    async fn test_error_rate_calculation() {
        let collector = MetricsCollector::with_default_config();
        let error = RenderingServiceError::ServiceUnavailable;

        // Record 3 errors in quick succession
        collector.record_error(&error).await;
        collector.record_error(&error).await;
        collector.record_error(&error).await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        let rate = collector.calculate_error_rate().await;
        assert!((rate - 3.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_metrics_export_json() {
        let collector = MetricsCollector::with_default_config();
        let error = RenderingServiceError::ServiceUnavailable;
        collector.record_error(&error).await;

        let json = collector.export_metrics(&MetricsFormat::Json).await;
        assert!(json.contains("\"total_errors\": 1"));
    }

    #[tokio::test]
    async fn test_metrics_export_prometheus() {
        let collector = MetricsCollector::with_default_config();
        let error = RenderingServiceError::ServiceUnavailable;
        collector.record_error(&error).await;

        let prom = collector.export_metrics(&MetricsFormat::Prometheus).await;
        assert!(prom.contains("uveddi_total_errors 1"));
    }

    #[tokio::test]
    async fn test_cleanup_old_events() {
        let config = MetricsConfig {
            retention_duration: Duration::from_millis(100),
            ..MetricsConfig::default()
        };

        let collector = MetricsCollector::new(config);
        let error = RenderingServiceError::ServiceUnavailable;

        collector.record_error(&error).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        collector.cleanup_old_events().await;

        let metrics = collector.get_metrics_snapshot().await;
        assert_eq!(metrics.total_errors, 0);
    }

    #[tokio::test]
    async fn test_memory_efficiency() {
        let config = MetricsConfig {
            retention_duration: Duration::from_millis(100),
            cleanup_interval: Duration::from_millis(100),
            ..MetricsConfig::default()
        };

        let collector = MetricsCollector::new(config);
        // Give time for cleanup task to run
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Record 100 errors quickly
        for _ in 0..100 {
            let error = RenderingServiceError::ServiceUnavailable;
            collector.record_error(&error).await;
        }

        // Wait for cleanup
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Check if old events were cleaned up
        let metrics = collector.get_metrics_snapshot().await;
        assert_eq!(
            metrics.total_errors, 0,
            "All events should be cleaned up after retention period"
        );
    }

    #[tokio::test]
    async fn test_trend_analysis() {
        let collector = MetricsCollector::with_default_config();
        let error = RenderingServiceError::ServiceUnavailable;

        // Record some errors
        for _ in 0..5 {
            collector.record_error(&error).await;
        }

        let trends = collector.generate_trends().await;
        assert!(!trends.is_empty());
        assert_eq!(trends[0].error_count, 5);
    }
}
