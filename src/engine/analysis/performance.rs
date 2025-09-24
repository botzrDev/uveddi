//! # Performance Instrumentation
//!
//! Provides performance monitoring and metrics collection for the analysis pipeline.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Performance metrics for analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    /// Total analysis time
    pub total_duration: Duration,

    /// Time spent in parsing phase
    pub parsing_duration: Duration,

    /// Time spent in detector execution
    pub detection_duration: Duration,

    /// Time spent building knowledge graph
    pub knowledge_graph_duration: Duration,

    /// Per-detector execution times
    pub detector_times: HashMap<String, Duration>,

    /// Number of files processed
    pub files_processed: usize,

    /// Number of issues found
    pub issues_found: usize,

    /// Memory usage (if available)
    pub peak_memory_usage: Option<usize>,
}

/// Performance timer for tracking operation durations
pub struct PerformanceTimer {
    start_time: Instant,
    phase_times: HashMap<String, Duration>,
    current_phase: Option<String>,
    current_phase_start: Option<Instant>,
}

/// Instrumentation wrapper for analysis operations
pub struct AnalysisInstrumentation {
    metrics: AnalysisMetrics,
    timer: PerformanceTimer,
    enabled: bool,
}

impl PerformanceTimer {
    /// Create a new performance timer
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            phase_times: HashMap::new(),
            current_phase: None,
            current_phase_start: None,
        }
    }

    /// Start timing a new phase
    pub fn start_phase(&mut self, phase_name: &str) {
        // End current phase if any
        if let (Some(current_phase), Some(phase_start)) =
            (&self.current_phase, self.current_phase_start) {
            let phase_duration = phase_start.elapsed();
            self.phase_times.insert(current_phase.clone(), phase_duration);
        }

        // Start new phase
        self.current_phase = Some(phase_name.to_string());
        self.current_phase_start = Some(Instant::now());
    }

    /// End the current phase
    pub fn end_phase(&mut self) {
        if let (Some(current_phase), Some(phase_start)) =
            (&self.current_phase, self.current_phase_start) {
            let phase_duration = phase_start.elapsed();
            self.phase_times.insert(current_phase.clone(), phase_duration);
            self.current_phase = None;
            self.current_phase_start = None;
        }
    }

    /// Get total elapsed time
    pub fn total_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get phase durations
    pub fn phase_durations(&self) -> &HashMap<String, Duration> {
        &self.phase_times
    }

    /// Get duration for a specific phase
    pub fn phase_duration(&self, phase_name: &str) -> Option<Duration> {
        self.phase_times.get(phase_name).copied()
    }
}

impl AnalysisInstrumentation {
    /// Create new instrumentation instance
    pub fn new(enabled: bool) -> Self {
        Self {
            metrics: AnalysisMetrics::default(),
            timer: PerformanceTimer::new(),
            enabled,
        }
    }

    /// Start timing a phase
    pub fn start_phase(&mut self, phase_name: &str) {
        if self.enabled {
            self.timer.start_phase(phase_name);
        }
    }

    /// End the current phase
    pub fn end_phase(&mut self) {
        if self.enabled {
            self.timer.end_phase();
        }
    }

    /// Record detector execution time
    pub fn record_detector_time(&mut self, detector_name: &str, duration: Duration) {
        if self.enabled {
            self.metrics.detector_times.insert(detector_name.to_string(), duration);
        }
    }

    /// Record file processed
    pub fn record_file_processed(&mut self) {
        if self.enabled {
            self.metrics.files_processed += 1;
        }
    }

    /// Record issues found
    pub fn record_issues_found(&mut self, count: usize) {
        if self.enabled {
            self.metrics.issues_found += count;
        }
    }

    /// Finalize metrics collection
    pub fn finalize(&mut self) -> AnalysisMetrics {
        if !self.enabled {
            return AnalysisMetrics::default();
        }

        self.timer.end_phase();

        // Update metrics from timer
        self.metrics.total_duration = self.timer.total_elapsed();
        self.metrics.parsing_duration = self.timer
            .phase_duration("parsing")
            .unwrap_or_default();
        self.metrics.detection_duration = self.timer
            .phase_duration("detection")
            .unwrap_or_default();
        self.metrics.knowledge_graph_duration = self.timer
            .phase_duration("knowledge_graph")
            .unwrap_or_default();

        // Attempt to get memory usage
        self.metrics.peak_memory_usage = get_peak_memory_usage();

        self.metrics.clone()
    }

    /// Get current metrics snapshot
    pub fn current_metrics(&self) -> &AnalysisMetrics {
        &self.metrics
    }

    /// Check if instrumentation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for AnalysisMetrics {
    fn default() -> Self {
        Self {
            total_duration: Duration::default(),
            parsing_duration: Duration::default(),
            detection_duration: Duration::default(),
            knowledge_graph_duration: Duration::default(),
            detector_times: HashMap::new(),
            files_processed: 0,
            issues_found: 0,
            peak_memory_usage: None,
        }
    }
}

impl AnalysisMetrics {
    /// Calculate throughput (files per second)
    pub fn files_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.files_processed as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Calculate detection rate (issues per file)
    pub fn issues_per_file(&self) -> f64 {
        if self.files_processed > 0 {
            self.issues_found as f64 / self.files_processed as f64
        } else {
            0.0
        }
    }

    /// Get slowest detector
    pub fn slowest_detector(&self) -> Option<(&String, &Duration)> {
        self.detector_times
            .iter()
            .max_by_key(|(_, duration)| *duration)
    }

    /// Generate performance report
    pub fn performance_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Analysis Performance Report ===\n");
        report.push_str(&format!("Total Duration: {:.2}s\n", self.total_duration.as_secs_f64()));
        report.push_str(&format!("Parsing Duration: {:.2}s\n", self.parsing_duration.as_secs_f64()));
        report.push_str(&format!("Detection Duration: {:.2}s\n", self.detection_duration.as_secs_f64()));
        report.push_str(&format!("Knowledge Graph Duration: {:.2}s\n", self.knowledge_graph_duration.as_secs_f64()));
        report.push_str(&format!("Files Processed: {}\n", self.files_processed));
        report.push_str(&format!("Issues Found: {}\n", self.issues_found));
        report.push_str(&format!("Throughput: {:.2} files/sec\n", self.files_per_second()));
        report.push_str(&format!("Detection Rate: {:.2} issues/file\n", self.issues_per_file()));

        if let Some(memory) = self.peak_memory_usage {
            report.push_str(&format!("Peak Memory: {} MB\n", memory / 1_000_000));
        }

        if !self.detector_times.is_empty() {
            report.push_str("\n=== Detector Performance ===\n");
            let mut detector_times: Vec<_> = self.detector_times.iter().collect();
            detector_times.sort_by_key(|(_, duration)| *duration);
            detector_times.reverse(); // Slowest first

            for (name, duration) in detector_times {
                report.push_str(&format!("{}: {:.3}s\n", name, duration.as_secs_f64()));
            }
        }

        report
    }

    /// Export metrics as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Get peak memory usage (platform-specific implementation)
fn get_peak_memory_usage() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmPeak:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }

    None // Not available on this platform or failed to read
}

/// Macro for timing code blocks
#[macro_export]
macro_rules! time_operation {
    ($instrumentation:expr, $phase:expr, $operation:block) => {{
        $instrumentation.start_phase($phase);
        let result = $operation;
        $instrumentation.end_phase();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_performance_timer() {
        let mut timer = PerformanceTimer::new();

        timer.start_phase("phase1");
        sleep(Duration::from_millis(10));
        timer.end_phase();

        timer.start_phase("phase2");
        sleep(Duration::from_millis(20));
        timer.end_phase();

        let phase1_duration = timer.phase_duration("phase1").unwrap();
        let phase2_duration = timer.phase_duration("phase2").unwrap();

        assert!(phase1_duration >= Duration::from_millis(10));
        assert!(phase2_duration >= Duration::from_millis(20));
        assert!(phase2_duration > phase1_duration);
    }

    #[test]
    fn test_analysis_instrumentation() {
        let mut instrumentation = AnalysisInstrumentation::new(true);

        instrumentation.start_phase("parsing");
        sleep(Duration::from_millis(10));
        instrumentation.end_phase();

        instrumentation.record_file_processed();
        instrumentation.record_issues_found(5);
        instrumentation.record_detector_time("TestDetector", Duration::from_millis(5));

        let metrics = instrumentation.finalize();

        assert!(metrics.total_duration >= Duration::from_millis(10));
        assert!(metrics.parsing_duration >= Duration::from_millis(10));
        assert_eq!(metrics.files_processed, 1);
        assert_eq!(metrics.issues_found, 5);
        assert_eq!(metrics.detector_times.len(), 1);
    }

    #[test]
    fn test_metrics_calculations() {
        let metrics = AnalysisMetrics {
            total_duration: Duration::from_secs(2),
            files_processed: 10,
            issues_found: 25,
            ..Default::default()
        };

        assert_eq!(metrics.files_per_second(), 5.0);
        assert_eq!(metrics.issues_per_file(), 2.5);
    }

    #[test]
    fn test_disabled_instrumentation() {
        let mut instrumentation = AnalysisInstrumentation::new(false);

        instrumentation.start_phase("test");
        instrumentation.record_file_processed();
        instrumentation.record_issues_found(10);

        let metrics = instrumentation.finalize();

        assert_eq!(metrics.files_processed, 0);
        assert_eq!(metrics.issues_found, 0);
    }
}