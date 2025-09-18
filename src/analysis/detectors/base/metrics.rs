//! Base metrics collection utilities

use super::types::DetectionMetrics;
use std::time::Instant;

/// A utility for collecting metrics during detection
pub struct MetricsCollector {
    start_time: Instant,
    metrics: DetectionMetrics,
}

impl MetricsCollector {
    /// Creates a new metrics collector
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: DetectionMetrics::new(),
        }
    }

    /// Records that a file was analyzed
    pub fn record_file_analyzed(&mut self) {
        self.metrics.files_analyzed += 1;
    }

    /// Records multiple files analyzed
    pub fn record_files_analyzed(&mut self, count: usize) {
        self.metrics.files_analyzed += count;
    }

    /// Records that nodes were processed
    pub fn record_nodes_processed(&mut self, count: usize) {
        self.metrics.nodes_processed += count;
    }

    /// Records memory usage
    pub fn record_memory_usage(&mut self, bytes: usize) {
        self.metrics.memory_usage_bytes = Some(bytes);
    }

    /// Adds a custom metric
    pub fn add_custom_metric(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metrics.custom_metrics.insert(key.into(), value);
    }

    /// Finalizes collection and returns the metrics
    pub fn finish(mut self) -> DetectionMetrics {
        self.metrics.duration_ms = self.start_time.elapsed().as_millis() as u64;
        self.metrics
    }

    /// Gets the current metrics without finalizing
    pub fn current_metrics(&self) -> &DetectionMetrics {
        &self.metrics
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Base metrics that can be extended by specific detectors
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BaseMetrics {
    /// Core detection metrics
    pub detection: DetectionMetrics,
    /// Processing statistics
    pub processing_stats: ProcessingStats,
}

/// Statistics about the processing phase
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessingStats {
    /// Number of patterns matched
    pub patterns_matched: usize,
    /// Number of false positives filtered out
    pub false_positives_filtered: usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: Option<usize>,
}

impl BaseMetrics {
    /// Creates new base metrics
    pub fn new(detection: DetectionMetrics) -> Self {
        Self {
            detection,
            processing_stats: ProcessingStats {
                patterns_matched: 0,
                false_positives_filtered: 0,
                cache_hit_ratio: 0.0,
                peak_memory_bytes: None,
            },
        }
    }

    /// Combines two metrics instances
    pub fn combine(mut self, other: Self) -> Self {
        // Combine detection metrics
        self.detection.duration_ms += other.detection.duration_ms;
        self.detection.files_analyzed += other.detection.files_analyzed;
        self.detection.nodes_processed += other.detection.nodes_processed;

        // Combine memory usage (take maximum)
        self.detection.memory_usage_bytes = match (
            self.detection.memory_usage_bytes,
            other.detection.memory_usage_bytes,
        ) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        // Merge custom metrics
        for (key, value) in other.detection.custom_metrics {
            self.detection.custom_metrics.insert(key, value);
        }

        // Combine processing stats
        self.processing_stats.patterns_matched += other.processing_stats.patterns_matched;
        self.processing_stats.false_positives_filtered +=
            other.processing_stats.false_positives_filtered;

        // Average cache hit ratios (weighted by files analyzed)
        let total_files = self.detection.files_analyzed;
        if total_files > 0 {
            let self_weight = (self.detection.files_analyzed - other.detection.files_analyzed) as f64 / total_files as f64;
            let other_weight = other.detection.files_analyzed as f64 / total_files as f64;
            self.processing_stats.cache_hit_ratio =
                self.processing_stats.cache_hit_ratio * self_weight +
                other.processing_stats.cache_hit_ratio * other_weight;
        }

        // Take maximum peak memory
        self.processing_stats.peak_memory_bytes = match (
            self.processing_stats.peak_memory_bytes,
            other.processing_stats.peak_memory_bytes,
        ) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        self
    }
}