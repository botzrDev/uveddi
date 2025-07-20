//! Performance metrics collection for parallel diagram generation
//!
//! This module implements metrics collection and monitoring for the parallel
//! diagram generation system, tracking throughput, latency, and resource utilization.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Collects and tracks performance metrics for parallel processing
pub struct ParallelMetricsCollector {
    start_time: Instant,
    tasks_completed: AtomicU64,
    tasks_failed: AtomicU64,
    current_queue_size: AtomicUsize,
    total_latency: AtomicU64, // in microseconds
    max_threads_used: AtomicUsize,
    memory_usage: AtomicUsize, // in KB
}

impl ParallelMetricsCollector {
    /// Creates a new metrics collector
    pub fn new() -> Self {
        ParallelMetricsCollector {
            start_time: Instant::now(),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            current_queue_size: AtomicUsize::new(0),
            total_latency: AtomicU64::new(0),
            max_threads_used: AtomicUsize::new(0),
            memory_usage: AtomicUsize::new(0),
        }
    }
    
    /// Track when a task starts processing
    pub fn track_task_started(&self) {
        // Implementation will be added in subsequent steps
    }
    
    /// Track when a task completes successfully
    pub fn track_task_completed(&self, latency: Duration) {
        self.tasks_completed.fetch_add(1, Ordering::SeqCst);
        self.total_latency.fetch_add(
            latency.as_micros() as u64,
            Ordering::SeqCst
        );
    }
    
    /// Track when a task fails
    pub fn track_task_failed(&self) {
        self.tasks_failed.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Track current queue size
    pub fn track_queue_size(&self, size: usize) {
        self.current_queue_size.store(size, Ordering::SeqCst);
    }
    
    /// Track thread usage
    pub fn track_thread_usage(&self, count: usize) {
        let mut max_threads = self.max_threads_used.load(Ordering::SeqCst);
        while count > max_threads {
            match self.max_threads_used.compare_exchange_weak(
                max_threads,
                count,
                Ordering::SeqCst,
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => max_threads = x,
            }
        }
    }
    
    /// Track memory usage
    pub fn track_memory_usage(&self, usage_kb: usize) {
        self.memory_usage.store(usage_kb, Ordering::SeqCst);
    }
    
    /// Calculate current throughput (diagrams per minute)
    pub fn current_throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (self.tasks_completed.load(Ordering::SeqCst) as f64 / elapsed) * 60.0
        } else {
            0.0
        }
    }
    
    /// Calculate average latency per diagram (in milliseconds)
    pub fn average_latency(&self) -> f64 {
        let completed = self.tasks_completed.load(Ordering::SeqCst) as f64;
        if completed > 0.0 {
            (self.total_latency.load(Ordering::SeqCst) as f64 / completed) / 1000.0
        } else {
            0.0
        }
    }
    
    /// Get peak memory usage (in KB)
    pub fn peak_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::SeqCst)
    }
    
    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        format!(
            "Parallel Processing Metrics:\n\
            ---------------------------\n\
            Throughput: {:.2} diagrams/min\n\
            Average Latency: {:.2} ms\n\
            Tasks Completed: {}\n\
            Tasks Failed: {}\n\
            Peak Threads Used: {}\n\
            Peak Memory Usage: {} KB",
            self.current_throughput(),
            self.average_latency(),
            self.tasks_completed.load(Ordering::SeqCst),
            self.tasks_failed.load(Ordering::SeqCst),
            self.max_threads_used.load(Ordering::SeqCst),
            self.peak_memory_usage()
        )
    }
}
