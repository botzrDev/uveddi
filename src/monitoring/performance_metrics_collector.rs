//! PerformanceMetricsCollector for adaptive, component-level metrics (UV-2)
//!
//! Collects execution time, memory usage, and other metrics per component.
//! Supports adaptive sampling, async storage, and configuration.

use crate::database::models::{ComponentPerformanceMetrics, PerformanceMetricsConfig};
use crate::monitoring::memory_monitor::MemoryMonitor;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde_json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

pub struct PerformanceMetricsCollector {
    config: PerformanceMetricsConfig,
    buffer: Arc<Mutex<Vec<ComponentPerformanceMetrics>>>,
    memory_monitor: Arc<Mutex<MemoryMonitor>>,
    total_components: usize,
    sampled_components: usize,
}

impl PerformanceMetricsCollector {
    pub fn new(config: PerformanceMetricsConfig, total_components: usize) -> Self {
        Self {
            config,
            buffer: Arc::new(Mutex::new(Vec::new())),
            memory_monitor: Arc::new(Mutex::new(MemoryMonitor::new())),
            total_components,
            sampled_components: 0,
        }
    }

    /// Determines if the current component should be sampled (adaptive)
    pub fn should_sample(&mut self, component_index: usize, has_issue: bool) -> bool {
        if !self.config.enabled {
            return false;
        }
        if self.config.adaptive_sampling {
            if self.total_components < 1000 {
                return true;
            }
            if has_issue {
                return true;
            }
            // Sample every Nth component
            let rate = (1.0 / self.config.sampling_rate).ceil() as usize;
            component_index % rate == 0
        } else {
            let rate = (1.0 / self.config.sampling_rate).ceil() as usize;
            component_index % rate == 0
        }
    }

    /// Capture current memory usage
    pub fn capture_memory_snapshot(&self) -> u64 {
        self.memory_monitor
            .lock()
            .unwrap()
            .get_current_memory_usage()
    }

    /// Record metrics for a component (buffered for async storage)
    pub fn record_component_metrics(&self, metrics: ComponentPerformanceMetrics) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.push(metrics);
        // TODO: Trigger async flush if buffer size exceeds config.buffer_size
    }

    /// Record analysis-level metrics (for overall analysis tracking)
    pub fn record_analysis_metrics(&self, issues_found: usize, files_analyzed: usize) {
        // This can be used for overall analysis tracking
        // For now, we'll just log the metrics
        log::info!(
            "Analysis completed: {} issues found in {} files",
            issues_found,
            files_analyzed
        );
    }

    /// Emit/export current metrics (returns Result for error handling)
    pub fn emit_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just export as JSON and log
        let json_output = self.export_json();
        log::debug!("Performance metrics: {}", json_output);
        Ok(())
    }

    /// Flush metrics buffer to persistent storage (to be implemented)
    pub async fn flush(&self) {
        // TODO: Implement async batch write to database
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        serde_json::to_string(&*buffer).unwrap_or_default()
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        let mut output = String::new();
        for metric in buffer.iter() {
            output.push_str(&format!(
                "component_execution_time_ms{{component_id=\"{}\"}} {}\n",
                metric.component_id, metric.execution_time_ms
            ));
            output.push_str(&format!(
                "component_memory_usage_bytes{{component_id=\"{}\"}} {}\n",
                metric.component_id, metric.memory_usage_bytes
            ));
            // Add other metrics as needed
        }
        output
    }

    /// Export metrics in InfluxDB line protocol
    pub fn export_influxdb(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        let mut output = String::new();
        for metric in buffer.iter() {
            output.push_str(&format!(
                "component_performance,component_id={} execution_time_ms={},memory_usage_bytes={},ast_parse_time_ms={},symbol_resolution_time_ms={},dependency_extraction_time_ms={} {}\n",
                metric.component_id,
                metric.execution_time_ms,
                metric.memory_usage_bytes,
                metric.ast_parse_time_ms.unwrap_or(0),
                metric.symbol_resolution_time_ms.unwrap_or(0),
                metric.dependency_extraction_time_ms.unwrap_or(0),
                metric.timestamp.timestamp()
            ));
        }
        output
    }
}

pub struct AsyncMetricsStorage {
    buffer: Arc<Mutex<Vec<ComponentPerformanceMetrics>>>,
    db_connection: Arc<Mutex<Connection>>,
    flush_handle: Option<tokio::task::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    flush_interval_seconds: u64,
    buffer_size: usize,
}

impl AsyncMetricsStorage {
    pub fn new(
        db_path: &str,
        buffer: Arc<Mutex<Vec<ComponentPerformanceMetrics>>>,
        flush_interval_seconds: u64,
        buffer_size: usize,
    ) -> Self {
        let conn = Connection::open(db_path).expect("Failed to open metrics DB");
        Self {
            buffer,
            db_connection: Arc::new(Mutex::new(conn)),
            flush_handle: None,
            running: Arc::new(AtomicBool::new(true)),
            flush_interval_seconds,
            buffer_size,
        }
    }

    pub fn start_flush_loop(&mut self) {
        let buffer = Arc::clone(&self.buffer);
        let db_connection = Arc::clone(&self.db_connection);
        let running = Arc::clone(&self.running);
        let interval = self.flush_interval_seconds;
        let buffer_size = self.buffer_size;
        self.flush_handle = Some(tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                sleep(Duration::from_secs(interval)).await;
                let mut buf = buffer.lock().unwrap();
                if buf.len() >= buffer_size || !buf.is_empty() {
                    // TODO: Batch write metrics to DB
                    let conn = db_connection.lock().unwrap();
                    for metric in buf.drain(..) {
                        let _ = conn.execute(
                            "INSERT INTO performance_metrics (component_id, analysis_run_id, execution_time_ms, memory_usage_bytes, ast_parse_time_ms, symbol_resolution_time_ms, dependency_extraction_time_ms, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            rusqlite::params![
                                metric.component_id,
                                metric.analysis_run_id,
                                metric.execution_time_ms,
                                metric.memory_usage_bytes,
                                metric.ast_parse_time_ms,
                                metric.symbol_resolution_time_ms,
                                metric.dependency_extraction_time_ms,
                                metric.timestamp.timestamp(),
                            ],
                        );
                    }
                }
            }
        }));
    }

    pub async fn flush_all(&self) {
        let mut buf = self.buffer.lock().unwrap();
        let conn = self.db_connection.lock().unwrap();
        for metric in buf.drain(..) {
            let _ = conn.execute(
                "INSERT INTO performance_metrics (component_id, analysis_run_id, execution_time_ms, memory_usage_bytes, ast_parse_time_ms, symbol_resolution_time_ms, dependency_extraction_time_ms, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    metric.component_id,
                    metric.analysis_run_id,
                    metric.execution_time_ms,
                    metric.memory_usage_bytes,
                    metric.ast_parse_time_ms,
                    metric.symbol_resolution_time_ms,
                    metric.dependency_extraction_time_ms,
                    metric.timestamp.timestamp(),
                ],
            );
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.flush_handle.take() {
            handle.abort();
        }
    }
}
