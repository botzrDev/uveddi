//! Performance monitoring and optimization for the knowledge library
//!
//! This module provides comprehensive performance monitoring, optimization hints,
//! and runtime profiling capabilities for the knowledge library system.

use std::sync::atomic::{AtomicU64, AtomicUsize, AtomicF64, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::collections::{HashMap, VecDeque};
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// Ring buffer for storing recent performance measurements
pub struct RingBuffer<T> {
    buffer: RwLock<VecDeque<T>>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }
    
    pub fn push(&self, value: T) {
        let mut buffer = self.buffer.write();
        if buffer.len() >= self.capacity {
            buffer.pop_front();
        }
        buffer.push_back(value);
    }
    
    pub fn average(&self) -> f64 
    where 
        T: Clone + Into<f64>
    {
        let buffer = self.buffer.read();
        if buffer.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = buffer.iter().cloned().map(|x| x.into()).sum();
        sum / buffer.len() as f64
    }
}

/// Comprehensive performance metrics for the knowledge library
#[derive(Debug)]
pub struct PerformanceMetrics {
    // Loading performance
    pub load_time_ms: AtomicU64,
    pub decompression_time_ms: AtomicU64,
    pub index_load_time_ms: AtomicU64,
    
    // Runtime performance
    pub lookup_times: RingBuffer<u64>,
    pub cache_hit_rate: AtomicF64,
    pub memory_usage_bytes: AtomicUsize,
    
    // Error tracking
    pub load_failures: AtomicU64,
    pub lookup_failures: AtomicU64,
    pub validation_failures: AtomicU64,
    
    // Throughput metrics
    pub total_lookups: AtomicU64,
    pub concurrent_operations: AtomicUsize,
    pub peak_concurrent_operations: AtomicUsize,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            load_time_ms: AtomicU64::new(0),
            decompression_time_ms: AtomicU64::new(0),
            index_load_time_ms: AtomicU64::new(0),
            lookup_times: RingBuffer::new(1000), // Keep last 1000 measurements
            cache_hit_rate: AtomicF64::new(0.0),
            memory_usage_bytes: AtomicUsize::new(0),
            load_failures: AtomicU64::new(0),
            lookup_failures: AtomicU64::new(0),
            validation_failures: AtomicU64::new(0),
            total_lookups: AtomicU64::new(0),
            concurrent_operations: AtomicUsize::new(0),
            peak_concurrent_operations: AtomicUsize::new(0),
        }
    }
    
    pub fn record_lookup_time(&self, duration_ns: u64) {
        self.lookup_times.push(duration_ns / 1_000_000); // Convert to milliseconds
        self.total_lookups.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_concurrent_ops(&self) -> usize {
        let current = self.concurrent_operations.fetch_add(1, Ordering::Relaxed) + 1;
        
        // Update peak if necessary
        let mut peak = self.peak_concurrent_operations.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_concurrent_operations.compare_exchange_weak(
                peak, current, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
        
        current
    }
    
    pub fn decrement_concurrent_ops(&self) {
        self.concurrent_operations.fetch_sub(1, Ordering::Relaxed);
    }
    
    pub fn average_lookup_time_ms(&self) -> f64 {
        self.lookup_times.average()
    }
}

/// Performance profiler for detailed analysis
pub struct PerformanceProfiler {
    enabled: bool,
    sampling_interval: Duration,
    profiles: RwLock<Vec<ProfileSnapshot>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileSnapshot {
    pub timestamp: u64,
    pub memory_usage_mb: usize,
    pub active_operations: usize,
    pub cache_hit_rate: f64,
    pub average_lookup_time_ms: f64,
    pub throughput_ops_per_sec: f64,
}

impl PerformanceProfiler {
    pub fn new(enabled: bool, sampling_interval: Duration) -> Self {
        Self {
            enabled,
            sampling_interval,
            profiles: RwLock::new(Vec::new()),
        }
    }
    
    pub fn take_snapshot(&self, metrics: &PerformanceMetrics) -> ProfileSnapshot {
        ProfileSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            memory_usage_mb: metrics.memory_usage_bytes.load(Ordering::Relaxed) / (1024 * 1024),
            active_operations: metrics.concurrent_operations.load(Ordering::Relaxed),
            cache_hit_rate: metrics.cache_hit_rate.load(Ordering::Relaxed),
            average_lookup_time_ms: metrics.average_lookup_time_ms(),
            throughput_ops_per_sec: self.calculate_throughput(metrics),
        }
    }
    
    fn calculate_throughput(&self, metrics: &PerformanceMetrics) -> f64 {
        // Simple throughput calculation based on recent operations
        let total_ops = metrics.total_lookups.load(Ordering::Relaxed);
        if total_ops == 0 {
            return 0.0;
        }
        
        // Estimate based on average lookup time
        let avg_time_ms = metrics.average_lookup_time_ms();
        if avg_time_ms > 0.0 {
            1000.0 / avg_time_ms // ops per second
        } else {
            0.0
        }
    }
    
    pub fn start_profiling(&self, metrics: Arc<PerformanceMetrics>) {
        if !self.enabled {
            return;
        }
        
        let profiler = Arc::new(self);
        let interval = self.sampling_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                
                let snapshot = profiler.take_snapshot(&metrics);
                let mut profiles = profiler.profiles.write();
                profiles.push(snapshot);
                
                // Keep only last 1000 profiles to prevent memory growth
                if profiles.len() > 1000 {
                    profiles.remove(0);
                }
            }
        });
    }
    
    pub fn get_profiles(&self) -> Vec<ProfileSnapshot> {
        self.profiles.read().clone()
    }
}

/// Performance optimization hints and suggestions
#[derive(Debug, Clone, Serialize)]
pub struct OptimizationHints {
    pub suggestions: Vec<OptimizationSuggestion>,
    pub performance_score: f64,
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizationSuggestion {
    pub category: OptimizationCategory,
    pub priority: Priority,
    pub description: String,
    pub expected_improvement: String,
    pub implementation_effort: EffortLevel,
}

#[derive(Debug, Clone, Serialize)]
pub enum OptimizationCategory {
    Memory,
    Caching,
    Concurrency,
    Decompression,
    Indexing,
}

#[derive(Debug, Clone, Serialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize)]
pub enum EffortLevel {
    Trivial,
    Low,
    Medium,
    High,
    Extensive,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceBottleneck {
    pub component: String,
    pub metric: String,
    pub current_value: f64,
    pub target_value: f64,
    pub severity: BottleneckSeverity,
}

#[derive(Debug, Clone, Serialize)]
pub enum BottleneckSeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

/// Main performance monitoring and optimization system
pub struct KnowledgePerformanceMonitor {
    metrics: Arc<PerformanceMetrics>,
    profiler: Option<PerformanceProfiler>,
    optimization_hints: RwLock<OptimizationHints>,
    performance_targets: PerformanceTargets,
}

#[derive(Debug, Clone)]
pub struct PerformanceTargets {
    pub max_load_time_ms: u64,        // Target: 100ms
    pub max_lookup_time_ms: f64,      // Target: 1ms
    pub max_memory_usage_mb: usize,   // Target: 100MB
    pub min_throughput_ops_sec: f64,  // Target: 1000 ops/sec
    pub min_cache_hit_rate: f64,      // Target: 95%
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            max_load_time_ms: 100,
            max_lookup_time_ms: 1.0,
            max_memory_usage_mb: 100,
            min_throughput_ops_sec: 1000.0,
            min_cache_hit_rate: 0.95,
        }
    }
}

/// Comprehensive performance report
#[derive(Debug, Serialize)]
pub struct PerformanceReport {
    pub summary: PerformanceSummary,
    pub metrics: PerformanceSnapshot,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub trends: PerformanceTrends,
    pub compliance: TargetCompliance,
}

#[derive(Debug, Serialize)]
pub struct PerformanceSummary {
    pub overall_score: f64,
    pub performance_grade: PerformanceGrade,
    pub key_insights: Vec<String>,
    pub critical_issues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

#[derive(Debug, Serialize)]
pub struct PerformanceSnapshot {
    pub load_time_ms: u64,
    pub average_lookup_time_ms: f64,
    pub memory_usage_mb: usize,
    pub cache_hit_rate: f64,
    pub throughput_ops_sec: f64,
    pub concurrent_operations: usize,
    pub error_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct PerformanceTrends {
    pub lookup_time_trend: TrendDirection,
    pub memory_usage_trend: TrendDirection,
    pub throughput_trend: TrendDirection,
    pub error_rate_trend: TrendDirection,
}

#[derive(Debug, Serialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct TargetCompliance {
    pub load_time_compliant: bool,
    pub lookup_time_compliant: bool,
    pub memory_usage_compliant: bool,
    pub throughput_compliant: bool,
    pub cache_hit_rate_compliant: bool,
    pub overall_compliance_score: f64,
}

impl KnowledgePerformanceMonitor {
    pub fn new(enable_profiling: bool) -> Self {
        let profiler = if enable_profiling {
            Some(PerformanceProfiler::new(true, Duration::from_secs(5)))
        } else {
            None
        };
        
        Self {
            metrics: Arc::new(PerformanceMetrics::new()),
            profiler,
            optimization_hints: RwLock::new(OptimizationHints {
                suggestions: Vec::new(),
                performance_score: 100.0,
                bottlenecks: Vec::new(),
            }),
            performance_targets: PerformanceTargets::default(),
        }
    }
    
    pub fn get_metrics(&self) -> Arc<PerformanceMetrics> {
        Arc::clone(&self.metrics)
    }
    
    /// Record lookup performance
    pub fn record_lookup(&self, pattern_id: &str, duration_ns: u64) {
        self.metrics.record_lookup_time(duration_ns);
        
        // Check if lookup time exceeds target
        let duration_ms = duration_ns as f64 / 1_000_000.0;
        if duration_ms > self.performance_targets.max_lookup_time_ms {
            tracing::warn!(
                "Slow lookup detected: pattern={}, duration={}ms (target: {}ms)",
                pattern_id, duration_ms, self.performance_targets.max_lookup_time_ms
            );
        }
    }
    
    /// Generate comprehensive performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let snapshot = self.create_performance_snapshot();
        let compliance = self.check_target_compliance(&snapshot);
        let bottlenecks = self.identify_bottlenecks(&snapshot);
        let suggestions = self.generate_optimization_suggestions(&snapshot, &bottlenecks);
        
        let overall_score = self.calculate_performance_score(&snapshot, &compliance);
        let grade = self.determine_performance_grade(overall_score);
        
        PerformanceReport {
            summary: PerformanceSummary {
                overall_score,
                performance_grade: grade,
                key_insights: self.generate_key_insights(&snapshot, &compliance),
                critical_issues: self.identify_critical_issues(&bottlenecks),
            },
            metrics: snapshot,
            optimization_suggestions: suggestions,
            bottlenecks,
            trends: self.analyze_trends(),
            compliance,
        }
    }
    
    /// Suggest optimizations based on usage patterns
    pub fn suggest_optimizations(&self) -> Vec<OptimizationSuggestion> {
        let snapshot = self.create_performance_snapshot();
        let bottlenecks = self.identify_bottlenecks(&snapshot);
        self.generate_optimization_suggestions(&snapshot, &bottlenecks)
    }
    
    fn create_performance_snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot {
            load_time_ms: self.metrics.load_time_ms.load(Ordering::Relaxed),
            average_lookup_time_ms: self.metrics.average_lookup_time_ms(),
            memory_usage_mb: self.metrics.memory_usage_bytes.load(Ordering::Relaxed) / (1024 * 1024),
            cache_hit_rate: self.metrics.cache_hit_rate.load(Ordering::Relaxed),
            throughput_ops_sec: self.calculate_throughput(),
            concurrent_operations: self.metrics.concurrent_operations.load(Ordering::Relaxed),
            error_rate: self.calculate_error_rate(),
        }
    }
    
    fn calculate_throughput(&self) -> f64 {
        let avg_time_ms = self.metrics.average_lookup_time_ms();
        if avg_time_ms > 0.0 {
            1000.0 / avg_time_ms
        } else {
            0.0
        }
    }
    
    fn calculate_error_rate(&self) -> f64 {
        let total_ops = self.metrics.total_lookups.load(Ordering::Relaxed);
        let total_errors = self.metrics.lookup_failures.load(Ordering::Relaxed) +
                          self.metrics.validation_failures.load(Ordering::Relaxed);
        
        if total_ops > 0 {
            total_errors as f64 / total_ops as f64
        } else {
            0.0
        }
    }
    
    fn check_target_compliance(&self, snapshot: &PerformanceSnapshot) -> TargetCompliance {
        let load_time_compliant = snapshot.load_time_ms <= self.performance_targets.max_load_time_ms;
        let lookup_time_compliant = snapshot.average_lookup_time_ms <= self.performance_targets.max_lookup_time_ms;
        let memory_usage_compliant = snapshot.memory_usage_mb <= self.performance_targets.max_memory_usage_mb;
        let throughput_compliant = snapshot.throughput_ops_sec >= self.performance_targets.min_throughput_ops_sec;
        let cache_hit_rate_compliant = snapshot.cache_hit_rate >= self.performance_targets.min_cache_hit_rate;
        
        let compliant_count = [
            load_time_compliant,
            lookup_time_compliant,
            memory_usage_compliant,
            throughput_compliant,
            cache_hit_rate_compliant,
        ].iter().filter(|&&x| x).count();
        
        let overall_compliance_score = compliant_count as f64 / 5.0;
        
        TargetCompliance {
            load_time_compliant,
            lookup_time_compliant,
            memory_usage_compliant,
            throughput_compliant,
            cache_hit_rate_compliant,
            overall_compliance_score,
        }
    }
    
    fn identify_bottlenecks(&self, snapshot: &PerformanceSnapshot) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Check load time
        if snapshot.load_time_ms > self.performance_targets.max_load_time_ms {
            bottlenecks.push(PerformanceBottleneck {
                component: "Loading".to_string(),
                metric: "Load Time".to_string(),
                current_value: snapshot.load_time_ms as f64,
                target_value: self.performance_targets.max_load_time_ms as f64,
                severity: self.determine_bottleneck_severity(
                    snapshot.load_time_ms as f64,
                    self.performance_targets.max_load_time_ms as f64,
                    false,
                ),
            });
        }
        
        // Check lookup time
        if snapshot.average_lookup_time_ms > self.performance_targets.max_lookup_time_ms {
            bottlenecks.push(PerformanceBottleneck {
                component: "Lookup".to_string(),
                metric: "Average Lookup Time".to_string(),
                current_value: snapshot.average_lookup_time_ms,
                target_value: self.performance_targets.max_lookup_time_ms,
                severity: self.determine_bottleneck_severity(
                    snapshot.average_lookup_time_ms,
                    self.performance_targets.max_lookup_time_ms,
                    false,
                ),
            });
        }
        
        // Check memory usage
        if snapshot.memory_usage_mb > self.performance_targets.max_memory_usage_mb {
            bottlenecks.push(PerformanceBottleneck {
                component: "Memory".to_string(),
                metric: "Memory Usage".to_string(),
                current_value: snapshot.memory_usage_mb as f64,
                target_value: self.performance_targets.max_memory_usage_mb as f64,
                severity: self.determine_bottleneck_severity(
                    snapshot.memory_usage_mb as f64,
                    self.performance_targets.max_memory_usage_mb as f64,
                    false,
                ),
            });
        }
        
        // Check throughput
        if snapshot.throughput_ops_sec < self.performance_targets.min_throughput_ops_sec {
            bottlenecks.push(PerformanceBottleneck {
                component: "Throughput".to_string(),
                metric: "Operations per Second".to_string(),
                current_value: snapshot.throughput_ops_sec,
                target_value: self.performance_targets.min_throughput_ops_sec,
                severity: self.determine_bottleneck_severity(
                    snapshot.throughput_ops_sec,
                    self.performance_targets.min_throughput_ops_sec,
                    true,
                ),
            });
        }
        
        bottlenecks
    }
    
    fn determine_bottleneck_severity(&self, current: f64, target: f64, higher_is_better: bool) -> BottleneckSeverity {
        let ratio = if higher_is_better {
            target / current.max(0.1) // Avoid division by zero
        } else {
            current / target.max(0.1)
        };
        
        match ratio {
            r if r >= 3.0 => BottleneckSeverity::Critical,
            r if r >= 2.0 => BottleneckSeverity::Severe,
            r if r >= 1.5 => BottleneckSeverity::Moderate,
            _ => BottleneckSeverity::Minor,
        }
    }
    
    fn generate_optimization_suggestions(&self, snapshot: &PerformanceSnapshot, bottlenecks: &[PerformanceBottleneck]) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        
        for bottleneck in bottlenecks {
            match bottleneck.component.as_str() {
                "Loading" => {
                    suggestions.push(OptimizationSuggestion {
                        category: OptimizationCategory::Decompression,
                        priority: Priority::High,
                        description: "Optimize decompression pipeline with parallel processing".to_string(),
                        expected_improvement: "50-70% reduction in load time".to_string(),
                        implementation_effort: EffortLevel::Medium,
                    });
                }
                "Lookup" => {
                    suggestions.push(OptimizationSuggestion {
                        category: OptimizationCategory::Indexing,
                        priority: Priority::High,
                        description: "Implement perfect hash functions for O(1) lookups".to_string(),
                        expected_improvement: "90% reduction in lookup time".to_string(),
                        implementation_effort: EffortLevel::High,
                    });
                }
                "Memory" => {
                    suggestions.push(OptimizationSuggestion {
                        category: OptimizationCategory::Memory,
                        priority: Priority::Medium,
                        description: "Enable memory mapping for large datasets".to_string(),
                        expected_improvement: "60-80% reduction in memory usage".to_string(),
                        implementation_effort: EffortLevel::Low,
                    });
                }
                "Throughput" => {
                    suggestions.push(OptimizationSuggestion {
                        category: OptimizationCategory::Concurrency,
                        priority: Priority::High,
                        description: "Implement lock-free data structures for concurrent access".to_string(),
                        expected_improvement: "3-5x improvement in throughput".to_string(),
                        implementation_effort: EffortLevel::High,
                    });
                }
                _ => {}
            }
        }
        
        // Add general suggestions based on metrics
        if snapshot.cache_hit_rate < 0.9 {
            suggestions.push(OptimizationSuggestion {
                category: OptimizationCategory::Caching,
                priority: Priority::Medium,
                description: "Increase cache size and implement intelligent prefetching".to_string(),
                expected_improvement: "Improve cache hit rate to >95%".to_string(),
                implementation_effort: EffortLevel::Low,
            });
        }
        
        suggestions
    }
    
    fn calculate_performance_score(&self, snapshot: &PerformanceSnapshot, compliance: &TargetCompliance) -> f64 {
        // Base score from compliance
        let mut score = compliance.overall_compliance_score * 100.0;
        
        // Adjust based on how close to targets we are
        if snapshot.load_time_ms <= self.performance_targets.max_load_time_ms / 2 {
            score += 5.0; // Bonus for being well under target
        }
        
        if snapshot.average_lookup_time_ms <= self.performance_targets.max_lookup_time_ms / 2.0 {
            score += 5.0;
        }
        
        if snapshot.cache_hit_rate > 0.98 {
            score += 5.0; // Bonus for excellent cache performance
        }
        
        // Penalty for high error rate
        if snapshot.error_rate > 0.01 {
            score -= 10.0;
        }
        
        score.max(0.0).min(100.0)
    }
    
    fn determine_performance_grade(&self, score: f64) -> PerformanceGrade {
        match score {
            s if s >= 90.0 => PerformanceGrade::Excellent,
            s if s >= 80.0 => PerformanceGrade::Good,
            s if s >= 60.0 => PerformanceGrade::Fair,
            s if s >= 40.0 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Critical,
        }
    }
    
    fn generate_key_insights(&self, snapshot: &PerformanceSnapshot, compliance: &TargetCompliance) -> Vec<String> {
        let mut insights = Vec::new();
        
        if compliance.overall_compliance_score == 1.0 {
            insights.push("All performance targets are being met".to_string());
        } else {
            insights.push(format!(
                "{}% of performance targets are being met",
                (compliance.overall_compliance_score * 100.0) as u32
            ));
        }
        
        if snapshot.cache_hit_rate > 0.95 {
            insights.push("Excellent cache performance detected".to_string());
        } else if snapshot.cache_hit_rate < 0.8 {
            insights.push("Cache performance needs improvement".to_string());
        }
        
        if snapshot.concurrent_operations > 50 {
            insights.push("High concurrency workload detected".to_string());
        }
        
        insights
    }
    
    fn identify_critical_issues(&self, bottlenecks: &[PerformanceBottleneck]) -> Vec<String> {
        bottlenecks
            .iter()
            .filter(|b| matches!(b.severity, BottleneckSeverity::Critical | BottleneckSeverity::Severe))
            .map(|b| format!("{} {} is {} target", b.component, b.metric, 
                if b.current_value > b.target_value { "above" } else { "below" }))
            .collect()
    }
    
    fn analyze_trends(&self) -> PerformanceTrends {
        // Simplified trend analysis - in a real implementation,
        // this would analyze historical data from the profiler
        PerformanceTrends {
            lookup_time_trend: TrendDirection::Stable,
            memory_usage_trend: TrendDirection::Stable,
            throughput_trend: TrendDirection::Stable,
            error_rate_trend: TrendDirection::Stable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.load_time_ms.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_lookups.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_ring_buffer() {
        let buffer = RingBuffer::new(3);
        buffer.push(10u64);
        buffer.push(20u64);
        buffer.push(30u64);
        assert_eq!(buffer.average(), 20.0);
        
        buffer.push(40u64); // Should replace 10
        assert_eq!(buffer.average(), 30.0);
    }
    
    #[test]
    fn test_performance_monitor() {
        let monitor = KnowledgePerformanceMonitor::new(false);
        monitor.record_lookup("test_pattern", 500_000); // 0.5ms
        
        let report = monitor.generate_report();
        assert!(report.metrics.average_lookup_time_ms < 1.0);
        assert!(matches!(report.summary.performance_grade, PerformanceGrade::Excellent | PerformanceGrade::Good));
    }
    
    #[tokio::test]
    async fn test_concurrent_operations_tracking() {
        let metrics = PerformanceMetrics::new();
        
        // Simulate concurrent operations
        let count1 = metrics.increment_concurrent_ops();
        let count2 = metrics.increment_concurrent_ops();
        
        assert_eq!(count1, 1);
        assert_eq!(count2, 2);
        assert_eq!(metrics.peak_concurrent_operations.load(Ordering::Relaxed), 2);
        
        metrics.decrement_concurrent_ops();
        assert_eq!(metrics.concurrent_operations.load(Ordering::Relaxed), 1);
    }
}