//! Cache performance metrics and logging
//!
//! Provides comprehensive metrics collection and reporting for cache performance,
//! including hit rates, latency measurements, and memory usage tracking.

use super::{AnalysisCache, AstCache};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};

/// Comprehensive cache metrics collector
#[derive(Debug)]
/// Performance and operational metrics for cache  collector.
pub struct CacheMetricsCollector {
    /// AST cache metrics
    #[cfg(feature = "ast-cache")]
    ast_cache: Option<Arc<Mutex<AstCache>>>,

    /// Analysis cache metrics
    #[cfg(feature = "analysis-cache")]
    analysis_cache: Option<Arc<Mutex<AnalysisCache>>>,

    /// Performance metrics
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,

    /// Metrics collection enabled
    enabled: bool,
}

/// Performance metrics for cache operations
#[derive(Debug, Default)]
/// Performance and operational metrics for performance.
pub struct PerformanceMetrics {
    /// Cache operation timing
    pub operation_times: CacheOperationTimes,

    /// Cache effectiveness metrics
    pub effectiveness: CacheEffectiveness,

    /// Memory usage tracking
    pub memory_usage: MemoryUsageMetrics,

    /// Error tracking
    pub error_counts: ErrorCounts,
}

/// Timing metrics for different cache operations
#[derive(Debug, Default)]
/// Represents cache operation times in the system.
pub struct CacheOperationTimes {
    /// AST cache operation times
    pub ast_cache_get_ns: RunningAverage,
    pub ast_cache_put_ns: RunningAverage,
    pub ast_cache_eviction_ns: RunningAverage,

    /// Analysis cache operation times
    pub analysis_cache_get_ns: RunningAverage,
    pub analysis_cache_put_ns: RunningAverage,
    pub analysis_cache_eviction_ns: RunningAverage,

    /// File system operation times
    pub file_metadata_check_ns: RunningAverage,
}

/// Cache effectiveness metrics
#[derive(Debug, Default)]
/// Represents cache effectiveness in the system.
pub struct CacheEffectiveness {
    /// AST cache effectiveness
    pub ast_hit_rate: f64,
    pub ast_total_requests: u64,
    pub ast_cache_hits: u64,
    pub ast_cache_misses: u64,

    /// Analysis cache effectiveness
    pub analysis_hit_rate: f64,
    pub analysis_total_requests: u64,
    pub analysis_cache_hits: u64,
    pub analysis_cache_misses: u64,

    /// Time saved by caching
    pub total_time_saved_ms: u64,
    pub average_time_saved_per_hit_ms: f64,
}

/// Memory usage metrics
#[derive(Debug, Default)]
/// Performance and operational metrics for memory usage.
pub struct MemoryUsageMetrics {
    /// AST cache memory usage
    pub ast_cache_memory_bytes: usize,
    pub ast_cache_entries: usize,
    pub ast_cache_peak_memory_bytes: usize,

    /// Analysis cache memory usage
    pub analysis_cache_memory_bytes: usize,
    pub analysis_cache_entries: usize,
    pub analysis_cache_peak_memory_bytes: usize,

    /// Memory usage over time
    pub memory_usage_history: Vec<MemorySnapshot>,
}

/// Point-in-time memory usage snapshot
#[derive(Debug, Clone)]
/// Represents memory snapshot in the system.
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub total_memory_bytes: usize,
    pub ast_cache_memory_bytes: usize,
    pub analysis_cache_memory_bytes: usize,
}

/// Error count tracking
#[derive(Debug, Default)]
/// Error information for counts failures.
pub struct ErrorCounts {
    pub cache_lock_failures: u64,
    pub file_access_errors: u64,
    pub serialization_errors: u64,
    pub eviction_errors: u64,
}

/// Running average calculator for performance metrics
#[derive(Debug)]
/// Represents running average in the system.
pub struct RunningAverage {
    sum: f64,
    count: u64,
    min: f64,
    max: f64,
}

impl Default for RunningAverage {
    fn default() -> Self {
        Self {
            sum: 0.0,
            count: 0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }
}

impl RunningAverage {
    /// Add a new sample
    pub fn add_sample(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    /// Get the current average
    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    /// Get the minimum value
    pub fn min(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.min
        }
    }

    /// Get the maximum value
    pub fn max(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.max
        }
    }

    /// Get the sample count
    pub fn count(&self) -> u64 {
        self.count
    }
}

impl CacheMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "ast-cache")]
            ast_cache: None,
            #[cfg(feature = "analysis-cache")]
            analysis_cache: None,
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            enabled: true,
        }
    }

    /// Set the AST cache to monitor
    #[cfg(feature = "ast-cache")]
    /// Caches with ast in the analysis pipeline.
    ///
    /// # Arguments
    ///
    /// - `cache`: Cache instance
    pub fn with_ast_cache(mut self, cache: Arc<Mutex<AstCache>>) -> Self {
        self.ast_cache = Some(cache);
        self
    }

    /// Set the analysis cache to monitor
    #[cfg(feature = "analysis-cache")]
    /// Caches with analysis in the analysis pipeline.
    ///
    /// # Arguments
    ///
    /// - `cache`: Cache instance
    pub fn with_analysis_cache(mut self, cache: Arc<Mutex<AnalysisCache>>) -> Self {
        self.analysis_cache = Some(cache);
        self
    }

    /// Enable or disable metrics collection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("Cache metrics collection enabled");
        } else {
            info!("Cache metrics collection disabled");
        }
    }

    /// Record an AST cache operation
    pub fn record_ast_cache_operation(&self, operation: CacheOperation, duration: Duration) {
        if !self.enabled {
            return;
        }

        let duration_ns = duration.as_nanos() as f64;

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            match operation {
                CacheOperation::Get => {
                    metrics
                        .operation_times
                        .ast_cache_get_ns
                        .add_sample(duration_ns);
                    debug!("AST cache GET took {:.2}μs", duration.as_micros());
                }
                CacheOperation::Put => {
                    metrics
                        .operation_times
                        .ast_cache_put_ns
                        .add_sample(duration_ns);
                    debug!("AST cache PUT took {:.2}μs", duration.as_micros());
                }
                CacheOperation::Eviction => {
                    metrics
                        .operation_times
                        .ast_cache_eviction_ns
                        .add_sample(duration_ns);
                    debug!("AST cache EVICTION took {:.2}μs", duration.as_micros());
                }
            }
        }
    }

    /// Record an analysis cache operation
    pub fn record_analysis_cache_operation(&self, operation: CacheOperation, duration: Duration) {
        if !self.enabled {
            return;
        }

        let duration_ns = duration.as_nanos() as f64;

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            match operation {
                CacheOperation::Get => {
                    metrics
                        .operation_times
                        .analysis_cache_get_ns
                        .add_sample(duration_ns);
                    debug!("Analysis cache GET took {:.2}μs", duration.as_micros());
                }
                CacheOperation::Put => {
                    metrics
                        .operation_times
                        .analysis_cache_put_ns
                        .add_sample(duration_ns);
                    debug!("Analysis cache PUT took {:.2}μs", duration.as_micros());
                }
                CacheOperation::Eviction => {
                    metrics
                        .operation_times
                        .analysis_cache_eviction_ns
                        .add_sample(duration_ns);
                    debug!("Analysis cache EVICTION took {:.2}μs", duration.as_micros());
                }
            }
        }
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self, cache_type: CacheType) {
        if !self.enabled {
            return;
        }

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            match cache_type {
                CacheType::Ast => {
                    metrics.effectiveness.ast_cache_hits += 1;
                    metrics.effectiveness.ast_total_requests += 1;
                    metrics.effectiveness.ast_hit_rate = metrics.effectiveness.ast_cache_hits
                        as f64
                        / metrics.effectiveness.ast_total_requests as f64;
                }
                CacheType::Analysis => {
                    metrics.effectiveness.analysis_cache_hits += 1;
                    metrics.effectiveness.analysis_total_requests += 1;
                    metrics.effectiveness.analysis_hit_rate =
                        metrics.effectiveness.analysis_cache_hits as f64
                            / metrics.effectiveness.analysis_total_requests as f64;
                }
            }
        }
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self, cache_type: CacheType) {
        if !self.enabled {
            return;
        }

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            match cache_type {
                CacheType::Ast => {
                    metrics.effectiveness.ast_cache_misses += 1;
                    metrics.effectiveness.ast_total_requests += 1;
                    metrics.effectiveness.ast_hit_rate = metrics.effectiveness.ast_cache_hits
                        as f64
                        / metrics.effectiveness.ast_total_requests as f64;
                }
                CacheType::Analysis => {
                    metrics.effectiveness.analysis_cache_misses += 1;
                    metrics.effectiveness.analysis_total_requests += 1;
                    metrics.effectiveness.analysis_hit_rate =
                        metrics.effectiveness.analysis_cache_hits as f64
                            / metrics.effectiveness.analysis_total_requests as f64;
                }
            }
        }
    }

    /// Record time saved by cache hit
    pub fn record_time_saved(&self, time_saved: Duration) {
        if !self.enabled {
            return;
        }

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            let time_saved_ms = time_saved.as_millis() as u64;
            metrics.effectiveness.total_time_saved_ms += time_saved_ms;

            let total_hits =
                metrics.effectiveness.ast_cache_hits + metrics.effectiveness.analysis_cache_hits;
            if total_hits > 0 {
                metrics.effectiveness.average_time_saved_per_hit_ms =
                    metrics.effectiveness.total_time_saved_ms as f64 / total_hits as f64;
            }
        }
    }

    /// Update memory usage metrics
    pub fn update_memory_metrics(&self) {
        if !self.enabled {
            return;
        }

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            let mut total_memory = 0;
            let mut ast_memory = 0;
            let mut analysis_memory = 0;

            // Collect AST cache metrics
            #[cfg(feature = "ast-cache")]
            if let Some(ref cache) = self.ast_cache {
                if let Ok(cache_guard) = cache.lock() {
                    let stats = cache_guard.stats();
                    ast_memory = stats.memory_bytes;
                    total_memory += ast_memory;

                    metrics.memory_usage.ast_cache_memory_bytes = ast_memory;
                    metrics.memory_usage.ast_cache_entries =
                        stats.hits as usize + stats.misses as usize;

                    if ast_memory > metrics.memory_usage.ast_cache_peak_memory_bytes {
                        metrics.memory_usage.ast_cache_peak_memory_bytes = ast_memory;
                    }
                }
            }

            // Collect Analysis cache metrics
            #[cfg(feature = "analysis-cache")]
            if let Some(ref cache) = self.analysis_cache {
                if let Ok(cache_guard) = cache.lock() {
                    let stats = cache_guard.stats();
                    analysis_memory = stats.total_files_cached * 1024; // Rough estimate
                    total_memory += analysis_memory;

                    metrics.memory_usage.analysis_cache_memory_bytes = analysis_memory;
                    metrics.memory_usage.analysis_cache_entries = stats.total_files_cached;

                    if analysis_memory > metrics.memory_usage.analysis_cache_peak_memory_bytes {
                        metrics.memory_usage.analysis_cache_peak_memory_bytes = analysis_memory;
                    }
                }
            }

            // Create memory snapshot
            let snapshot = MemorySnapshot {
                timestamp: Instant::now(),
                total_memory_bytes: total_memory,
                ast_cache_memory_bytes: ast_memory,
                analysis_cache_memory_bytes: analysis_memory,
            };

            metrics.memory_usage.memory_usage_history.push(snapshot);

            // Keep only recent history (last 100 snapshots)
            if metrics.memory_usage.memory_usage_history.len() > 100 {
                metrics.memory_usage.memory_usage_history.remove(0);
            }
        }
    }

    /// Record an error
    pub fn record_error(&self, error_type: CacheErrorType) {
        if !self.enabled {
            return;
        }

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            match error_type {
                CacheErrorType::LockFailure => {
                    metrics.error_counts.cache_lock_failures += 1;
                    warn!("Cache lock failure occurred");
                }
                CacheErrorType::FileAccess => {
                    metrics.error_counts.file_access_errors += 1;
                    warn!("File access error in cache operation");
                }
                CacheErrorType::Serialization => {
                    metrics.error_counts.serialization_errors += 1;
                    warn!("Serialization error in cache operation");
                }
                CacheErrorType::Eviction => {
                    metrics.error_counts.eviction_errors += 1;
                    warn!("Cache eviction error occurred");
                }
            }
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> Option<PerformanceMetrics> {
        if let Ok(metrics) = self.performance_metrics.lock() {
            Some(metrics.clone())
        } else {
            None
        }
    }

    /// Print comprehensive cache metrics report
    pub fn print_metrics_report(&self) {
        if let Some(metrics) = self.get_metrics() {
            info!("=== Cache Performance Report ===");

            // AST Cache metrics
            info!("AST Cache:");
            info!(
                "  Hit Rate: {:.2}%",
                metrics.effectiveness.ast_hit_rate * 100.0
            );
            info!(
                "  Total Requests: {}",
                metrics.effectiveness.ast_total_requests
            );
            info!("  Cache Hits: {}", metrics.effectiveness.ast_cache_hits);
            info!("  Cache Misses: {}", metrics.effectiveness.ast_cache_misses);
            info!(
                "  Memory Usage: {} bytes",
                metrics.memory_usage.ast_cache_memory_bytes
            );
            info!(
                "  Peak Memory: {} bytes",
                metrics.memory_usage.ast_cache_peak_memory_bytes
            );
            info!(
                "  Avg GET time: {:.2}μs",
                metrics.operation_times.ast_cache_get_ns.average() / 1000.0
            );
            info!(
                "  Avg PUT time: {:.2}μs",
                metrics.operation_times.ast_cache_put_ns.average() / 1000.0
            );

            // Analysis Cache metrics
            info!("Analysis Cache:");
            info!(
                "  Hit Rate: {:.2}%",
                metrics.effectiveness.analysis_hit_rate * 100.0
            );
            info!(
                "  Total Requests: {}",
                metrics.effectiveness.analysis_total_requests
            );
            info!(
                "  Cache Hits: {}",
                metrics.effectiveness.analysis_cache_hits
            );
            info!(
                "  Cache Misses: {}",
                metrics.effectiveness.analysis_cache_misses
            );
            info!(
                "  Memory Usage: {} bytes",
                metrics.memory_usage.analysis_cache_memory_bytes
            );
            info!(
                "  Peak Memory: {} bytes",
                metrics.memory_usage.analysis_cache_peak_memory_bytes
            );
            info!(
                "  Avg GET time: {:.2}μs",
                metrics.operation_times.analysis_cache_get_ns.average() / 1000.0
            );
            info!(
                "  Avg PUT time: {:.2}μs",
                metrics.operation_times.analysis_cache_put_ns.average() / 1000.0
            );

            // Overall effectiveness
            info!("Overall Performance:");
            info!(
                "  Total Time Saved: {}ms",
                metrics.effectiveness.total_time_saved_ms
            );
            info!(
                "  Avg Time Saved per Hit: {:.2}ms",
                metrics.effectiveness.average_time_saved_per_hit_ms
            );

            // Error counts
            if metrics.error_counts.cache_lock_failures > 0
                || metrics.error_counts.file_access_errors > 0
                || metrics.error_counts.serialization_errors > 0
                || metrics.error_counts.eviction_errors > 0
            {
                info!("Errors:");
                info!(
                    "  Lock Failures: {}",
                    metrics.error_counts.cache_lock_failures
                );
                info!(
                    "  File Access Errors: {}",
                    metrics.error_counts.file_access_errors
                );
                info!(
                    "  Serialization Errors: {}",
                    metrics.error_counts.serialization_errors
                );
                info!(
                    "  Eviction Errors: {}",
                    metrics.error_counts.eviction_errors
                );
            }

            info!("=== End Cache Report ===");
        } else {
            warn!("Could not access cache metrics");
        }
    }

    /// Record a cache operation (simple version for service manager)
    pub fn record_cache_operation(&self, cache_type: &str, operation: &str) {
        if !self.enabled {
            return;
        }

        match (cache_type, operation) {
            ("ast", "hit") => self.record_cache_hit(CacheType::Ast),
            ("ast", "miss") => self.record_cache_miss(CacheType::Ast),
            ("analysis", "hit") => self.record_cache_hit(CacheType::Analysis),
            ("analysis", "miss") => self.record_cache_miss(CacheType::Analysis),
            ("ast", "access") | ("analysis", "access") => {
                // Generic access recorded for service manager
                debug!("Cache {} accessed", cache_type);
            }
            _ => {
                warn!("Unknown cache operation: {} {}", cache_type, operation);
            }
        }
    }

    /// Record a cache hit (simple version for tests)
    pub fn record_hit(&mut self, cache_type: &str) {
        match cache_type {
            "ast" => self.record_cache_hit(CacheType::Ast),
            "analysis" => self.record_cache_hit(CacheType::Analysis),
            _ => warn!("Unknown cache type: {}", cache_type),
        }
    }

    /// Record a cache miss (simple version for tests)
    pub fn record_miss(&mut self, cache_type: &str) {
        match cache_type {
            "ast" => self.record_cache_miss(CacheType::Ast),
            "analysis" => self.record_cache_miss(CacheType::Analysis),
            _ => warn!("Unknown cache type: {}", cache_type),
        }
    }

    /// Get simplified metrics for testing
    pub fn get_simplified_metrics(&self) -> SimplifiedMetrics {
        if let Ok(metrics) = self.performance_metrics.lock() {
            SimplifiedMetrics {
                total_hits: metrics.effectiveness.ast_cache_hits
                    + metrics.effectiveness.analysis_cache_hits,
                total_misses: metrics.effectiveness.ast_cache_misses
                    + metrics.effectiveness.analysis_cache_misses,
                ast_hits: metrics.effectiveness.ast_cache_hits,
                ast_misses: metrics.effectiveness.ast_cache_misses,
                analysis_hits: metrics.effectiveness.analysis_cache_hits,
                analysis_misses: metrics.effectiveness.analysis_cache_misses,
            }
        } else {
            SimplifiedMetrics::default()
        }
    }

    /// Generate a comprehensive metrics report
    pub fn generate_report(&self) -> String {
        if let Ok(metrics) = self.performance_metrics.lock() {
            format!(
                "Cache Metrics Report:\n\
                 AST Cache - Hits: {}, Misses: {}, Hit Rate: {:.2}%\n\
                 Analysis Cache - Hits: {}, Misses: {}, Hit Rate: {:.2}%\n\
                 Average GET times - AST: {:.2}μs, Analysis: {:.2}μs",
                metrics.effectiveness.ast_cache_hits,
                metrics.effectiveness.ast_cache_misses,
                metrics.effectiveness.ast_hit_rate * 100.0,
                metrics.effectiveness.analysis_cache_hits,
                metrics.effectiveness.analysis_cache_misses,
                metrics.effectiveness.analysis_hit_rate * 100.0,
                metrics.operation_times.ast_cache_get_ns.average() / 1000.0,
                metrics.operation_times.analysis_cache_get_ns.average() / 1000.0
            )
        } else {
            "Unable to generate metrics report - lock failed".to_string()
        }
    }

    /// Get hit rate for a specific cache type
    pub fn hit_rate(&self, cache_name: &str) -> f64 {
        if let Ok(metrics) = self.performance_metrics.lock() {
            match cache_name {
                "ast" => metrics.effectiveness.ast_hit_rate,
                "analysis" => metrics.effectiveness.analysis_hit_rate,
                _ => {
                    // For unknown cache types, return average of all caches
                    let total_hits = metrics.effectiveness.ast_cache_hits
                        + metrics.effectiveness.analysis_cache_hits;
                    let total_requests = metrics.effectiveness.ast_total_requests
                        + metrics.effectiveness.analysis_total_requests;
                    if total_requests > 0 {
                        total_hits as f64 / total_requests as f64
                    } else {
                        0.0
                    }
                }
            }
        } else {
            0.0
        }
    }
}

impl Default for CacheMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// Supporting types for the metrics system

/// Type of cache operation being measured
#[derive(Debug, Clone, Copy)]
pub enum CacheOperation {
    Get,
    Put,
    Eviction,
}

/// Type of cache being measured
#[derive(Debug, Clone, Copy)]
pub enum CacheType {
    Ast,
    Analysis,
}

/// Type of cache error that occurred
#[derive(Debug, Clone, Copy)]
pub enum CacheErrorType {
    LockFailure,
    FileAccess,
    Serialization,
    Eviction,
}

/// Simplified metrics structure for testing
#[derive(Debug, Default)]
/// Performance and operational metrics for simplified.
pub struct SimplifiedMetrics {
    pub total_hits: u64,
    pub total_misses: u64,
    pub ast_hits: u64,
    pub ast_misses: u64,
    pub analysis_hits: u64,
    pub analysis_misses: u64,
}

impl Clone for PerformanceMetrics {
    fn clone(&self) -> Self {
        Self {
            operation_times: CacheOperationTimes {
                ast_cache_get_ns: RunningAverage {
                    sum: self.operation_times.ast_cache_get_ns.sum,
                    count: self.operation_times.ast_cache_get_ns.count,
                    min: self.operation_times.ast_cache_get_ns.min,
                    max: self.operation_times.ast_cache_get_ns.max,
                },
                ast_cache_put_ns: RunningAverage {
                    sum: self.operation_times.ast_cache_put_ns.sum,
                    count: self.operation_times.ast_cache_put_ns.count,
                    min: self.operation_times.ast_cache_put_ns.min,
                    max: self.operation_times.ast_cache_put_ns.max,
                },
                ast_cache_eviction_ns: RunningAverage {
                    sum: self.operation_times.ast_cache_eviction_ns.sum,
                    count: self.operation_times.ast_cache_eviction_ns.count,
                    min: self.operation_times.ast_cache_eviction_ns.min,
                    max: self.operation_times.ast_cache_eviction_ns.max,
                },
                analysis_cache_get_ns: RunningAverage {
                    sum: self.operation_times.analysis_cache_get_ns.sum,
                    count: self.operation_times.analysis_cache_get_ns.count,
                    min: self.operation_times.analysis_cache_get_ns.min,
                    max: self.operation_times.analysis_cache_get_ns.max,
                },
                analysis_cache_put_ns: RunningAverage {
                    sum: self.operation_times.analysis_cache_put_ns.sum,
                    count: self.operation_times.analysis_cache_put_ns.count,
                    min: self.operation_times.analysis_cache_put_ns.min,
                    max: self.operation_times.analysis_cache_put_ns.max,
                },
                analysis_cache_eviction_ns: RunningAverage {
                    sum: self.operation_times.analysis_cache_eviction_ns.sum,
                    count: self.operation_times.analysis_cache_eviction_ns.count,
                    min: self.operation_times.analysis_cache_eviction_ns.min,
                    max: self.operation_times.analysis_cache_eviction_ns.max,
                },
                file_metadata_check_ns: RunningAverage {
                    sum: self.operation_times.file_metadata_check_ns.sum,
                    count: self.operation_times.file_metadata_check_ns.count,
                    min: self.operation_times.file_metadata_check_ns.min,
                    max: self.operation_times.file_metadata_check_ns.max,
                },
            },
            effectiveness: self.effectiveness.clone(),
            memory_usage: self.memory_usage.clone(),
            error_counts: self.error_counts.clone(),
        }
    }
}

impl Clone for CacheEffectiveness {
    fn clone(&self) -> Self {
        Self {
            ast_hit_rate: self.ast_hit_rate,
            ast_total_requests: self.ast_total_requests,
            ast_cache_hits: self.ast_cache_hits,
            ast_cache_misses: self.ast_cache_misses,
            analysis_hit_rate: self.analysis_hit_rate,
            analysis_total_requests: self.analysis_total_requests,
            analysis_cache_hits: self.analysis_cache_hits,
            analysis_cache_misses: self.analysis_cache_misses,
            total_time_saved_ms: self.total_time_saved_ms,
            average_time_saved_per_hit_ms: self.average_time_saved_per_hit_ms,
        }
    }
}

impl Clone for MemoryUsageMetrics {
    fn clone(&self) -> Self {
        Self {
            ast_cache_memory_bytes: self.ast_cache_memory_bytes,
            ast_cache_entries: self.ast_cache_entries,
            ast_cache_peak_memory_bytes: self.ast_cache_peak_memory_bytes,
            analysis_cache_memory_bytes: self.analysis_cache_memory_bytes,
            analysis_cache_entries: self.analysis_cache_entries,
            analysis_cache_peak_memory_bytes: self.analysis_cache_peak_memory_bytes,
            memory_usage_history: self.memory_usage_history.clone(),
        }
    }
}

impl Clone for ErrorCounts {
    fn clone(&self) -> Self {
        Self {
            cache_lock_failures: self.cache_lock_failures,
            file_access_errors: self.file_access_errors,
            serialization_errors: self.serialization_errors,
            eviction_errors: self.eviction_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running_average() {
        let mut avg = RunningAverage::default();

        avg.add_sample(10.0);
        assert_eq!(avg.average(), 10.0);
        assert_eq!(avg.count(), 1);

        avg.add_sample(20.0);
        assert_eq!(avg.average(), 15.0);
        assert_eq!(avg.count(), 2);

        assert_eq!(avg.min(), 10.0);
        assert_eq!(avg.max(), 20.0);
    }

    #[test]
    fn test_metrics_collector_creation() {
        let collector = CacheMetricsCollector::new();
        assert!(collector.enabled);

        let metrics = collector.get_metrics().unwrap();
        assert_eq!(metrics.effectiveness.ast_total_requests, 0);
        assert_eq!(metrics.effectiveness.analysis_total_requests, 0);
    }

    #[test]
    fn test_cache_hit_recording() {
        let collector = CacheMetricsCollector::new();

        collector.record_cache_hit(CacheType::Ast);
        collector.record_cache_hit(CacheType::Ast);
        collector.record_cache_miss(CacheType::Ast);

        let metrics = collector.get_metrics().unwrap();
        assert_eq!(metrics.effectiveness.ast_cache_hits, 2);
        assert_eq!(metrics.effectiveness.ast_cache_misses, 1);
        assert_eq!(metrics.effectiveness.ast_total_requests, 3);
        assert!((metrics.effectiveness.ast_hit_rate - 2.0 / 3.0).abs() < 0.001);
    }
}
