//! Memory metrics collection for Phase 2 with object pool metrics
//! Includes basic memory metrics and object pool performance data

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Basic memory metrics for Phase 1 foundation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicMemoryMetrics {
    /// Type of global allocator being used
    pub allocator_type: String,

    /// Whether memory optimization features are enabled
    pub optimization_enabled: bool,

    /// Target memory limit in bytes
    pub target_memory_bytes: u64,

    /// Current estimated memory usage in bytes
    pub current_memory_bytes: u64,

    /// Peak memory usage observed in bytes
    pub peak_memory_bytes: u64,

    /// Number of times metrics were updated
    pub update_count: u64,

    /// Object pool hit rate (successful reuse from pool)
    pub pool_hit_rate: f64,

    /// Object pool miss rate (had to allocate new object)
    pub pool_miss_rate: f64,

    /// Total number of pool hits
    pub total_pool_hits: u64,

    /// Total number of pool misses
    pub total_pool_misses: u64,
}

impl BasicMemoryMetrics {
    pub fn new(target_memory_bytes: u64) -> Self {
        Self {
            allocator_type: crate::analysis::memory::allocator::get_allocator_info().to_string(),
            optimization_enabled: crate::analysis::memory::allocator::is_optimized_allocator(),
            target_memory_bytes,
            current_memory_bytes: 0,
            peak_memory_bytes: 0,
            update_count: 0,
            pool_hit_rate: 0.0,
            pool_miss_rate: 0.0,
            total_pool_hits: 0,
            total_pool_misses: 0,
        }
    }

    /// Update current memory usage
    pub fn update_memory_usage(&mut self, current_bytes: u64) {
        self.current_memory_bytes = current_bytes;
        self.peak_memory_bytes = self.peak_memory_bytes.max(current_bytes);
        self.update_count += 1;
    }

    /// Record a pool hit (successful object reuse)
    pub fn record_pool_hit(&mut self) {
        self.total_pool_hits += 1;
        self.update_pool_rates();
    }

    /// Record a pool miss (new object allocation)
    pub fn record_pool_miss(&mut self) {
        self.total_pool_misses += 1;
        self.update_pool_rates();
    }

    /// Update pool hit and miss rates
    fn update_pool_rates(&mut self) {
        let total_operations = self.total_pool_hits + self.total_pool_misses;
        if total_operations > 0 {
            self.pool_hit_rate = (self.total_pool_hits as f64) / (total_operations as f64) * 100.0;
            self.pool_miss_rate =
                (self.total_pool_misses as f64) / (total_operations as f64) * 100.0;
        }
    }

    /// Calculate memory usage as percentage of target
    pub fn memory_usage_percentage(&self) -> f64 {
        if self.target_memory_bytes == 0 {
            return 0.0;
        }
        (self.current_memory_bytes as f64 / self.target_memory_bytes as f64) * 100.0
    }

    /// Check if memory usage is within target
    pub fn is_within_target(&self) -> bool {
        self.current_memory_bytes <= self.target_memory_bytes
    }
}

/// Thread-safe memory metrics collector for Phase 1
pub struct BasicMemoryMetricsCollector {
    metrics: Arc<Mutex<BasicMemoryMetrics>>,
}

impl BasicMemoryMetricsCollector {
    pub fn new(target_memory_bytes: u64) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(BasicMemoryMetrics::new(target_memory_bytes))),
        }
    }

    /// Update memory usage metrics
    pub fn update_memory_usage(&self, bytes: u64) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.update_memory_usage(bytes);
        }
    }

    /// Record a pool hit for metrics tracking
    pub fn record_pool_hit(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_pool_hit();
        }
    }

    /// Record a pool miss for metrics tracking
    pub fn record_pool_miss(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_pool_miss();
        }
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> BasicMemoryMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Export metrics for observability
    pub fn export_json(&self) -> serde_json::Value {
        let metrics = self.get_metrics();
        serde_json::json!({
            "memory_optimization_phase1": {
                "allocator_type": metrics.allocator_type,
                "optimization_enabled": metrics.optimization_enabled,
                "target_memory_gb": metrics.target_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                "current_memory_gb": metrics.current_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                "peak_memory_gb": metrics.peak_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                "memory_usage_percent": metrics.memory_usage_percentage(),
                "within_target": metrics.is_within_target(),
                "update_count": metrics.update_count,
                "object_pools": {
                    "hit_rate_percent": metrics.pool_hit_rate,
                    "miss_rate_percent": metrics.pool_miss_rate,
                    "total_hits": metrics.total_pool_hits,
                    "total_misses": metrics.total_pool_misses,
                    "total_operations": metrics.total_pool_hits + metrics.total_pool_misses
                }
            }
        })
    }
}

// Global metrics collector instance for Phase 1
lazy_static::lazy_static! {
    pub static ref BASIC_MEMORY_METRICS: BasicMemoryMetricsCollector = {
        BasicMemoryMetricsCollector::new(8 * 1024 * 1024 * 1024) // 8GB default target
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_memory_metrics() {
        let mut metrics = BasicMemoryMetrics::new(1024 * 1024 * 1024); // 1GB target

        assert_eq!(metrics.target_memory_bytes, 1024 * 1024 * 1024);
        assert_eq!(metrics.current_memory_bytes, 0);
        assert_eq!(metrics.peak_memory_bytes, 0);
        assert_eq!(metrics.update_count, 0);

        metrics.update_memory_usage(512 * 1024 * 1024); // 512MB
        assert_eq!(metrics.current_memory_bytes, 512 * 1024 * 1024);
        assert_eq!(metrics.peak_memory_bytes, 512 * 1024 * 1024);
        assert_eq!(metrics.update_count, 1);
        assert!(metrics.is_within_target());
        assert_eq!(metrics.memory_usage_percentage(), 50.0);

        metrics.update_memory_usage(2 * 1024 * 1024 * 1024); // 2GB - exceeds target
        assert!(!metrics.is_within_target());
        assert_eq!(metrics.memory_usage_percentage(), 200.0);
    }

    #[test]
    fn test_metrics_collector() {
        let collector = BasicMemoryMetricsCollector::new(1024 * 1024 * 1024);

        collector.update_memory_usage(256 * 1024 * 1024);
        let metrics = collector.get_metrics();

        assert_eq!(metrics.current_memory_bytes, 256 * 1024 * 1024);
        assert!(metrics.is_within_target());

        let json = collector.export_json();
        assert!(json["memory_optimization_phase1"]["optimization_enabled"].is_boolean());
        assert!(
            json["memory_optimization_phase1"]["target_memory_gb"]
                .as_f64()
                .unwrap()
                > 0.0
        );
    }
}
