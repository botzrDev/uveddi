use serde::{Deserialize, Serialize};

/// Performance metrics for cache operations
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Performance and quality metrics for cache.
pub struct CacheMetrics {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub memory_usage_bytes: usize,
    pub average_lookup_time_ms: f64,
    pub hit_rate: f64,
    pub memory_mapped_entries: u64,
}

impl CacheMetrics {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            evictions: 0,
            memory_usage_bytes: 0,
            average_lookup_time_ms: 0.0,
            hit_rate: 0.0,
            memory_mapped_entries: 0,
        }
    }

    /// Performs update hit rate operation.
    pub fn update_hit_rate(&mut self) {
        if self.total_requests > 0 {
            self.hit_rate = (self.cache_hits as f64 / self.total_requests as f64) * 100.0;
        }
    }
}
