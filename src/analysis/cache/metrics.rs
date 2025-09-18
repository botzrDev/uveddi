//! Cache performance metrics and monitoring instrumentation
//!
//! This module provides comprehensive metrics collection and monitoring
//! for cache performance, including hit rates, latencies, memory usage,
//! and real-time performance analytics.

#[cfg(feature = "prometheus")]
use prometheus::{Histogram, IntCounter, IntGauge, Registry};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Comprehensive cache metrics collector
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    // Prometheus metrics
    pub cache_hits: IntCounter,
    pub cache_misses: IntCounter,
    pub cache_evictions: IntCounter,
    pub cache_size_bytes: IntGauge,
    pub cache_entry_count: IntGauge,
    pub cache_operation_duration: Histogram,
    pub serialization_duration: Histogram,
    pub deserialization_duration: Histogram,
    pub disk_read_duration: Histogram,
    pub disk_write_duration: Histogram,

    // Internal metrics
    internal_metrics: Arc<Mutex<InternalMetrics>>,

    // Registry for exporting metrics
    registry: Arc<Registry>,
}

#[derive(Debug, Default)]
struct InternalMetrics {
    total_operations: u64,
    total_hit_time_ns: u64,
    total_miss_time_ns: u64,
    total_serialization_bytes: u64,
    cache_layer_stats: HashMap<String, LayerStats>,
    recent_operations: Vec<OperationRecord>,
}

#[derive(Debug, Clone)]
struct LayerStats {
    hits: u64,
    misses: u64,
    evictions: u64,
    size_bytes: u64,
    entry_count: u64,
}

#[derive(Debug, Clone)]
struct OperationRecord {
    timestamp: Instant,
    operation_type: OperationType,
    layer: String,
    duration: Duration,
    success: bool,
}

#[derive(Debug, Clone)]
enum OperationType {
    Get,
    Set,
    Eviction,
    Serialization,
    Deserialization,
}

impl CacheMetrics {
    /// Create new cache metrics with custom registry
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let registry = Arc::new(registry.clone());
        let cache_hits = IntCounter::new("cache_hits_total", "Total cache hits")?;
        registry.register(Box::new(cache_hits.clone()))?;

        let cache_misses = IntCounter::new("cache_misses_total", "Total cache misses")?;
        registry.register(Box::new(cache_misses.clone()))?;

        let cache_evictions = IntCounter::new("cache_evictions_total", "Total cache evictions")?;
        registry.register(Box::new(cache_evictions.clone()))?;

        let cache_size_bytes = IntGauge::new("cache_size_bytes", "Current cache size in bytes")?;
        registry.register(Box::new(cache_size_bytes.clone()))?;

        let cache_entry_count =
            IntGauge::new("cache_entry_count", "Current number of cache entries")?;
        registry.register(Box::new(cache_entry_count.clone()))?;

        let cache_operation_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "cache_operation_duration_seconds",
                "Cache operation duration",
            )
            .buckets(vec![
                0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0,
            ]),
        )?;
        registry.register(Box::new(cache_operation_duration.clone()))?;

        let serialization_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "cache_serialization_duration_seconds",
                "Serialization duration",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
        )?;
        registry.register(Box::new(serialization_duration.clone()))?;

        let deserialization_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "cache_deserialization_duration_seconds",
                "Deserialization duration",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
        )?;
        registry.register(Box::new(deserialization_duration.clone()))?;

        let disk_read_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "cache_disk_read_duration_seconds",
                "Disk read duration",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        )?;
        registry.register(Box::new(disk_read_duration.clone()))?;

        let disk_write_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "cache_disk_write_duration_seconds",
                "Disk write duration",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        )?;
        registry.register(Box::new(disk_write_duration.clone()))?;

        Ok(Self {
            cache_hits,
            cache_misses,
            cache_evictions,
            cache_size_bytes,
            cache_entry_count,
            cache_operation_duration,
            serialization_duration,
            deserialization_duration,
            disk_read_duration,
            disk_write_duration,
            internal_metrics: Arc::new(Mutex::new(InternalMetrics::default())),
            registry,
        })
    }

    /// Create metrics with default registry
    pub fn with_default_registry() -> Result<Self, prometheus::Error> {
        Self::new(&prometheus::default_registry())
    }

    /// Record a cache hit
    pub fn record_hit(&self, layer: &str, duration: Duration) {
        self.cache_hits.inc();
        self.cache_operation_duration
            .observe(duration.as_secs_f64());

        if let Ok(mut metrics) = self.internal_metrics.lock() {
            metrics.total_operations += 1;
            metrics.total_hit_time_ns += duration.as_nanos() as u64;

            let layer_stats = metrics
                .cache_layer_stats
                .entry(layer.to_string())
                .or_default();
            layer_stats.hits += 1;

            metrics.recent_operations.push(OperationRecord {
                timestamp: Instant::now(),
                operation_type: OperationType::Get,
                layer: layer.to_string(),
                duration,
                success: true,
            });

            // Keep only recent operations (last 1000)
            if metrics.recent_operations.len() > 1000 {
                metrics.recent_operations.drain(0..100);
            }
        }
    }

    /// Record a cache miss
    pub fn record_miss(&self, layer: &str, duration: Duration) {
        self.cache_misses.inc();
        self.cache_operation_duration
            .observe(duration.as_secs_f64());

        if let Ok(mut metrics) = self.internal_metrics.lock() {
            metrics.total_operations += 1;
            metrics.total_miss_time_ns += duration.as_nanos() as u64;

            let layer_stats = metrics
                .cache_layer_stats
                .entry(layer.to_string())
                .or_default();
            layer_stats.misses += 1;

            metrics.recent_operations.push(OperationRecord {
                timestamp: Instant::now(),
                operation_type: OperationType::Get,
                layer: layer.to_string(),
                duration,
                success: false,
            });
        }
    }

    /// Record cache eviction
    pub fn record_eviction(&self, layer: &str, bytes_freed: u64) {
        self.cache_evictions.inc();
        self.cache_size_bytes.sub(bytes_freed as i64);
        self.cache_entry_count.dec();

        if let Ok(mut metrics) = self.internal_metrics.lock() {
            let layer_stats = metrics
                .cache_layer_stats
                .entry(layer.to_string())
                .or_default();
            layer_stats.evictions += 1;
            layer_stats.size_bytes = layer_stats.size_bytes.saturating_sub(bytes_freed);
            layer_stats.entry_count = layer_stats.entry_count.saturating_sub(1);
        }
    }

    /// Record cache entry insertion
    pub fn record_insertion(&self, layer: &str, bytes_added: u64) {
        self.cache_size_bytes.add(bytes_added as i64);
        self.cache_entry_count.inc();

        if let Ok(mut metrics) = self.internal_metrics.lock() {
            let layer_stats = metrics
                .cache_layer_stats
                .entry(layer.to_string())
                .or_default();
            layer_stats.size_bytes += bytes_added;
            layer_stats.entry_count += 1;
        }
    }

    /// Record serialization operation
    pub fn record_serialization(&self, duration: Duration, bytes: u64) {
        self.serialization_duration.observe(duration.as_secs_f64());

        if let Ok(mut metrics) = self.internal_metrics.lock() {
            metrics.total_serialization_bytes += bytes;

            metrics.recent_operations.push(OperationRecord {
                timestamp: Instant::now(),
                operation_type: OperationType::Serialization,
                layer: "serialization".to_string(),
                duration,
                success: true,
            });
        }
    }

    /// Record deserialization operation
    pub fn record_deserialization(&self, duration: Duration) {
        self.deserialization_duration
            .observe(duration.as_secs_f64());

        if let Ok(mut metrics) = self.internal_metrics.lock() {
            metrics.recent_operations.push(OperationRecord {
                timestamp: Instant::now(),
                operation_type: OperationType::Deserialization,
                layer: "deserialization".to_string(),
                duration,
                success: true,
            });
        }
    }

    /// Record disk read operation
    pub fn record_disk_read(&self, duration: Duration) {
        self.disk_read_duration.observe(duration.as_secs_f64());
    }

    /// Record disk write operation
    pub fn record_disk_write(&self, duration: Duration) {
        self.disk_write_duration.observe(duration.as_secs_f64());
    }

    /// Get current hit rate across all layers
    pub fn hit_rate(&self) -> f64 {
        let hits = self.cache_hits.get() as f64;
        let misses = self.cache_misses.get() as f64;
        let total = hits + misses;

        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }

    /// Get hit rate for specific layer
    pub fn layer_hit_rate(&self, layer: &str) -> f64 {
        if let Ok(metrics) = self.internal_metrics.lock() {
            if let Some(stats) = metrics.cache_layer_stats.get(layer) {
                let total = stats.hits + stats.misses;
                if total > 0 {
                    return stats.hits as f64 / total as f64;
                }
            }
        }
        0.0
    }

    /// Get average operation latency
    pub fn average_latency(&self) -> Duration {
        if let Ok(metrics) = self.internal_metrics.lock() {
            if metrics.total_operations > 0 {
                let total_ns = metrics.total_hit_time_ns + metrics.total_miss_time_ns;
                return Duration::from_nanos(total_ns / metrics.total_operations);
            }
        }
        Duration::from_nanos(0)
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> PerformanceSummary {
        let hit_rate = self.hit_rate();
        let avg_latency = self.average_latency();
        let cache_size = self.cache_size_bytes.get() as u64;
        let entry_count = self.cache_entry_count.get() as u64;

        let mut layer_summaries = HashMap::new();
        if let Ok(metrics) = self.internal_metrics.lock() {
            for (layer, stats) in &metrics.cache_layer_stats {
                layer_summaries.insert(
                    layer.clone(),
                    LayerSummary {
                        hit_rate: if stats.hits + stats.misses > 0 {
                            stats.hits as f64 / (stats.hits + stats.misses) as f64
                        } else {
                            0.0
                        },
                        size_bytes: stats.size_bytes,
                        entry_count: stats.entry_count,
                        evictions: stats.evictions,
                    },
                );
            }
        }

        PerformanceSummary {
            overall_hit_rate: hit_rate,
            average_latency_ms: avg_latency.as_millis() as f64,
            total_size_bytes: cache_size,
            total_entries: entry_count,
            layer_summaries,
        }
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> String {
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder
            .encode_to_string(&metric_families)
            .unwrap_or_default()
    }
}

impl Default for LayerStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            size_bytes: 0,
            entry_count: 0,
        }
    }
}

/// Performance summary for reporting
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub overall_hit_rate: f64,
    pub average_latency_ms: f64,
    pub total_size_bytes: u64,
    pub total_entries: u64,
    pub layer_summaries: HashMap<String, LayerSummary>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LayerSummary {
    pub hit_rate: f64,
    pub size_bytes: u64,
    pub entry_count: u64,
    pub evictions: u64,
}

/// Real-time cache monitor with alerting
pub struct CacheMonitor {
    metrics: Arc<CacheMetrics>,
    alert_thresholds: AlertThresholds,
    monitoring_interval: Duration,
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub min_hit_rate: f64,
    pub max_latency_ms: f64,
    pub max_memory_usage_bytes: u64,
    pub max_eviction_rate: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            min_hit_rate: 0.8,                          // 80% minimum hit rate
            max_latency_ms: 10.0,                       // 10ms maximum latency
            max_memory_usage_bytes: 1024 * 1024 * 1024, // 1GB maximum
            max_eviction_rate: 0.1,                     // 10% maximum eviction rate
        }
    }
}

impl CacheMonitor {
    pub fn new(metrics: Arc<CacheMetrics>) -> Self {
        Self {
            metrics,
            alert_thresholds: AlertThresholds::default(),
            monitoring_interval: Duration::from_secs(30),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.alert_thresholds = thresholds;
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.monitoring_interval = interval;
        self
    }

    /// Start monitoring in background
    pub async fn start_monitoring(&self) {
        let mut running = self.running.write().await;
        if *running {
            return; // Already running
        }
        *running = true;

        let metrics = self.metrics.clone();
        let thresholds = self.alert_thresholds.clone();
        let interval = self.monitoring_interval;
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            loop {
                let should_continue = {
                    let running = running_flag.read().await;
                    *running
                };

                if !should_continue {
                    break;
                }

                let summary = metrics.performance_summary();
                Self::check_alerts(&summary, &thresholds).await;

                tokio::time::sleep(interval).await;
            }
        });
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    async fn check_alerts(summary: &PerformanceSummary, thresholds: &AlertThresholds) {
        // Check hit rate
        if summary.overall_hit_rate < thresholds.min_hit_rate {
            tracing::warn!(
                "Cache hit rate below threshold: {:.2}% < {:.2}%",
                summary.overall_hit_rate * 100.0,
                thresholds.min_hit_rate * 100.0
            );
        }

        // Check latency
        if summary.average_latency_ms > thresholds.max_latency_ms {
            tracing::warn!(
                "Cache latency above threshold: {:.2}ms > {:.2}ms",
                summary.average_latency_ms,
                thresholds.max_latency_ms
            );
        }

        // Check memory usage
        if summary.total_size_bytes > thresholds.max_memory_usage_bytes {
            tracing::warn!(
                "Cache memory usage above threshold: {}MB > {}MB",
                summary.total_size_bytes / (1024 * 1024),
                thresholds.max_memory_usage_bytes / (1024 * 1024)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    use std::time::Duration;

    #[test]
    fn test_cache_metrics_creation() {
        let registry = Registry::new();
        let metrics = CacheMetrics::new(&registry).unwrap();

        assert_eq!(metrics.cache_hits.get(), 0);
        assert_eq!(metrics.cache_misses.get(), 0);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let registry = Registry::new();
        let metrics = CacheMetrics::new(&registry).unwrap();

        // Record some hits and misses
        metrics.record_hit("L1", Duration::from_millis(1));
        metrics.record_hit("L1", Duration::from_millis(1));
        metrics.record_miss("L1", Duration::from_millis(5));

        let hit_rate = metrics.hit_rate();
        assert!((hit_rate - 0.666).abs() < 0.01); // ~66.7%
    }

    #[test]
    fn test_layer_specific_metrics() {
        let registry = Registry::new();
        let metrics = CacheMetrics::new(&registry).unwrap();

        metrics.record_hit("L1", Duration::from_millis(1));
        metrics.record_miss("L1", Duration::from_millis(5));
        metrics.record_hit("L2", Duration::from_millis(10));

        let l1_hit_rate = metrics.layer_hit_rate("L1");
        let l2_hit_rate = metrics.layer_hit_rate("L2");

        assert!((l1_hit_rate - 0.5).abs() < 0.01); // 50% for L1
        assert!((l2_hit_rate - 1.0).abs() < 0.01); // 100% for L2
    }

    #[test]
    fn test_performance_summary() {
        let registry = Registry::new();
        let metrics = CacheMetrics::new(&registry).unwrap();

        metrics.record_hit("L1", Duration::from_millis(2));
        metrics.record_insertion("L1", 1024);

        let summary = metrics.performance_summary();
        assert_eq!(summary.total_entries, 1);
        assert_eq!(summary.total_size_bytes, 1024);
        assert!(summary.overall_hit_rate > 0.0);
    }

    #[tokio::test]
    async fn test_cache_monitor() {
        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());

        let monitor = CacheMonitor::new(metrics.clone());

        // Start monitoring
        monitor.start_monitoring().await;

        // Record some metrics
        metrics.record_hit("L1", Duration::from_millis(1));

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop monitoring
        monitor.stop_monitoring().await;
    }

    #[test]
    fn test_metrics_export() {
        let registry = Registry::new();
        let metrics = CacheMetrics::new(&registry).unwrap();

        metrics.record_hit("L1", Duration::from_millis(1));

        let exported = metrics.export_metrics();
        assert!(exported.contains("cache_hits_total"));
    }
}
