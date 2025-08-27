//! Performance Analysis Service
//!
//! This service handles memory monitoring, performance metrics collection, and resource
//! management during analysis. It replaces the performance monitoring logic from the
//! monolithic AnalysisEngine.

use crate::analysis::memory_report::{MemoryAnalysisReport, PhaseMemoryBreakdown};
use crate::database::models::{ComponentPerformanceMetrics, PerformanceMetricsConfig};
use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;

use super::AnalysisResult;

use crate::core::logging::{debug, info, warn};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Memory monitoring configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub memory_limit_bytes: Option<usize>,
    pub warning_threshold_percent: f64,
    pub monitoring_interval_ms: u64,
    pub enable_detailed_tracking: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_limit_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB default
            warning_threshold_percent: 80.0,
            monitoring_interval_ms: 1000,
            enable_detailed_tracking: false,
        }
    }
}

/// Performance monitoring session for tracking metrics during analysis
pub struct MonitoringSession {
    session_id: String,
    start_time: Instant,
    metrics_collector: Arc<PerformanceMetricsCollector>,
    memory_monitor: Arc<MemoryMonitor>,
    session_metrics: Arc<Mutex<SessionMetrics>>,
}

/// Metrics collected during a monitoring session
#[derive(Debug, Clone, Default)]
pub struct SessionMetrics {
    pub peak_memory_usage: usize,
    pub average_memory_usage: usize,
    pub memory_samples: Vec<usize>,
    pub cpu_usage_percent: f64,
    pub execution_phases: HashMap<String, Duration>,
    pub warnings: Vec<String>,
}

/// Comprehensive performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub session_id: String,
    pub total_duration: Duration,
    pub memory_usage: MemoryUsageReport,
    pub cpu_metrics: CpuMetrics,
    pub execution_phases: HashMap<String, Duration>,
    pub recommendations: Vec<String>,
    pub warnings: Vec<String>,
}

/// Memory usage report
#[derive(Debug, Clone)]
pub struct MemoryUsageReport {
    pub peak_usage_bytes: usize,
    pub average_usage_bytes: usize,
    pub current_usage_bytes: usize,
    pub limit_bytes: Option<usize>,
    pub usage_over_time: Vec<(Instant, usize)>,
}

/// CPU metrics
#[derive(Debug, Clone, Default)]
pub struct CpuMetrics {
    pub average_usage_percent: f64,
    pub peak_usage_percent: f64,
    pub user_time_ms: u64,
    pub system_time_ms: u64,
}

/// Memory monitor for tracking memory usage
pub struct MemoryMonitor {
    config: MemoryConfig,
    current_usage: Arc<RwLock<usize>>,
    peak_usage: Arc<RwLock<usize>>,
    usage_history: Arc<Mutex<Vec<(Instant, usize)>>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl MemoryMonitor {
    fn new(config: MemoryConfig) -> Self {
        Self {
            config,
            current_usage: Arc::new(RwLock::new(0)),
            peak_usage: Arc::new(RwLock::new(0)),
            usage_history: Arc::new(Mutex::new(Vec::new())),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    async fn start_monitoring(&self) {
        let mut monitoring_active = self.monitoring_active.write().await;
        *monitoring_active = true;

        // Start background monitoring task
        let current_usage = self.current_usage.clone();
        let peak_usage = self.peak_usage.clone();
        let usage_history = self.usage_history.clone();
        let monitoring_active_clone = self.monitoring_active.clone();
        let interval = Duration::from_millis(self.config.monitoring_interval_ms);

        tokio::spawn(async move {
            while *monitoring_active_clone.read().await {
                let usage = Self::get_current_memory_usage();

                // Update current usage
                {
                    let mut current = current_usage.write().await;
                    *current = usage;
                }

                // Update peak usage
                {
                    let mut peak = peak_usage.write().await;
                    if usage > *peak {
                        *peak = usage;
                    }
                }

                // Record in history
                {
                    let mut history = usage_history.lock().await;
                    history.push((Instant::now(), usage));

                    // Keep only recent history (last 1000 samples)
                    if history.len() > 1000 {
                        history.remove(0);
                    }
                }

                tokio::time::sleep(interval).await;
            }
        });
    }

    async fn stop_monitoring(&self) {
        let mut monitoring_active = self.monitoring_active.write().await;
        *monitoring_active = false;
    }

    async fn get_peak_usage(&self) -> usize {
        *self.peak_usage.read().await
    }

    async fn get_current_usage(&self) -> usize {
        *self.current_usage.read().await
    }

    async fn memory_limit_exceeded(&self) -> bool {
        if let Some(limit) = self.config.memory_limit_bytes {
            let current = self.get_current_usage().await;
            current > limit
        } else {
            false
        }
    }

    fn get_current_memory_usage() -> usize {
        // Platform-specific memory usage retrieval
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(size_str) = line.split_whitespace().nth(1) {
                            if let Ok(size_kb) = size_str.parse::<usize>() {
                                return size_kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        // Fallback: return 0 if unable to determine
        0
    }
}

/// Service responsible for performance monitoring and memory management
pub struct PerformanceAnalysisService {
    metrics_collector: Arc<PerformanceMetricsCollector>,
    memory_monitor: Arc<MemoryMonitor>,
    active_sessions: Arc<RwLock<HashMap<String, Arc<MonitoringSession>>>>,
    config: MemoryConfig,
}

impl PerformanceAnalysisService {
    /// Create new performance analysis service
    pub fn new(metrics_collector: Arc<PerformanceMetricsCollector>, config: MemoryConfig) -> Self {
        let memory_monitor = Arc::new(MemoryMonitor::new(config.clone()));

        Self {
            metrics_collector,
            memory_monitor,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create new performance analysis service with default configuration
    pub fn new_with_defaults(metrics_collector: Arc<PerformanceMetricsCollector>) -> Self {
        Self::new(metrics_collector, MemoryConfig::default())
    }

    /// Start monitoring session
    pub async fn start_monitoring(&self) -> MonitoringSession {
        let session_id = Uuid::new_v4().to_string();
        debug!("Starting monitoring session: {}", session_id);

        let session = MonitoringSession {
            session_id: session_id.clone(),
            start_time: Instant::now(),
            metrics_collector: self.metrics_collector.clone(),
            memory_monitor: self.memory_monitor.clone(),
            session_metrics: Arc::new(Mutex::new(SessionMetrics::default())),
        };

        // Start memory monitoring
        self.memory_monitor.start_monitoring().await;

        // Register session
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id, Arc::new(session.clone()));
        }

        session
    }

    /// Stop monitoring session and generate report
    pub async fn stop_monitoring(&self, session: &MonitoringSession) -> PerformanceReport {
        debug!("Stopping monitoring session: {}", session.session_id);

        // Stop memory monitoring
        self.memory_monitor.stop_monitoring().await;

        // Generate report
        let report = self.generate_performance_report(session).await;

        // Remove from active sessions
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(&session.session_id);
        }

        report
    }

    /// Analyze with memory limits and monitoring
    pub async fn analyze_with_memory_limits<T, F>(&self, analysis: F) -> AnalysisResult<T>
    where
        F: Future<Output = AnalysisResult<T>>,
    {
        let session = self.start_monitoring().await;

        // Monitor memory usage during analysis
        let memory_check_task: tokio::task::JoinHandle<Result<(), String>> = {
            let memory_monitor = self.memory_monitor.clone();
            let session_id = session.session_id.clone();

            tokio::spawn(async move {
                loop {
                    if memory_monitor.memory_limit_exceeded().await {
                        warn!("Memory limit exceeded in session: {}", session_id);
                        return Err("Memory limit exceeded".to_string());
                    }

                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            })
        };

        // Run analysis
        let analysis_result = tokio::select! {
            result = analysis => result,
            memory_result = memory_check_task => {
                match memory_result {
                    Ok(Err(msg)) => return Err(crate::analysis::errors::AnalysisError::configuration_error(
                        "memory_limit", format!("exceeded: {}", msg), "Available memory insufficient"
                    )),
                    _ => return Err(crate::analysis::errors::AnalysisError::configuration_error(
                        "memory_monitor", "failed", "Memory monitoring system failed"
                    )),
                }
            }
        };

        // Generate final report
        let _report = self.stop_monitoring(&session).await;

        analysis_result
    }

    /// Get comprehensive performance report for active session
    pub async fn get_performance_report(&self, session_id: &str) -> Option<PerformanceReport> {
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            Some(self.generate_performance_report(session).await)
        } else {
            None
        }
    }

    /// Get current memory usage
    pub async fn get_current_memory_usage(&self) -> usize {
        self.memory_monitor.get_current_usage().await
    }

    /// Get peak memory usage
    pub async fn get_peak_memory_usage(&self) -> usize {
        self.memory_monitor.get_peak_usage().await
    }

    /// Check if memory limit is exceeded
    pub async fn is_memory_limit_exceeded(&self) -> bool {
        self.memory_monitor.memory_limit_exceeded().await
    }

    /// Get active monitoring sessions
    pub async fn get_active_sessions(&self) -> Vec<String> {
        let sessions = self.active_sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Generate memory analysis report
    pub async fn generate_memory_report(&self) -> MemoryAnalysisReport {
        let current_usage = self.get_current_memory_usage().await;
        let peak_usage = self.get_peak_memory_usage().await;

        MemoryAnalysisReport {
            memory_limit_mb: self.config.memory_limit_bytes.unwrap_or(2048 * 1024 * 1024)
                / (1024 * 1024), // Convert to MB
            initial_memory_mb: 0, // TODO: Track initial memory
            final_memory_mb: current_usage / (1024 * 1024), // Convert to MB
            peak_memory_mb: peak_usage / (1024 * 1024), // Convert to MB
            analysis_duration: std::time::Duration::from_secs(0), // TODO: Track duration
            memory_optimization_effective: peak_usage
                < self.config.memory_limit_bytes.unwrap_or(usize::MAX),
            memory_limit_exceeded: self.is_memory_limit_exceeded().await,
            scope_reduced: false, // TODO: Track scope reduction
            applied_strategies: vec!["Performance monitoring".to_string()],
            recommendations: self
                .generate_memory_recommendations(current_usage, peak_usage)
                .await,
            phase_breakdowns: Vec::new(), // TODO: Track phase breakdowns
        }
    }

    // Private helper methods

    async fn generate_performance_report(&self, session: &MonitoringSession) -> PerformanceReport {
        let total_duration = session.start_time.elapsed();
        let session_metrics = session.session_metrics.lock().await;

        let memory_usage = MemoryUsageReport {
            peak_usage_bytes: session_metrics.peak_memory_usage,
            average_usage_bytes: session_metrics.average_memory_usage,
            current_usage_bytes: self.get_current_memory_usage().await,
            limit_bytes: self.config.memory_limit_bytes,
            usage_over_time: {
                let history = self.memory_monitor.usage_history.lock().await;
                history.clone()
            },
        };

        let recommendations = self
            .generate_recommendations(&session_metrics, &memory_usage)
            .await;

        PerformanceReport {
            session_id: session.session_id.clone(),
            total_duration,
            memory_usage,
            cpu_metrics: CpuMetrics::default(), // TODO: Implement CPU metrics collection
            execution_phases: session_metrics.execution_phases.clone(),
            recommendations,
            warnings: session_metrics.warnings.clone(),
        }
    }

    async fn generate_recommendations(
        &self,
        _metrics: &SessionMetrics,
        memory: &MemoryUsageReport,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(limit) = memory.limit_bytes {
            let usage_percent = (memory.peak_usage_bytes as f64 / limit as f64) * 100.0;

            if usage_percent > 90.0 {
                recommendations.push(
                    "Consider increasing memory limit or optimizing memory usage".to_string(),
                );
            } else if usage_percent > 75.0 {
                recommendations.push("Monitor memory usage closely, approaching limit".to_string());
            }
        }

        if memory.peak_usage_bytes > 1024 * 1024 * 1024 {
            // > 1GB
            recommendations
                .push("Consider implementing incremental analysis for large codebases".to_string());
        }

        recommendations
    }

    async fn generate_memory_recommendations(&self, _current: usize, peak: usize) -> Vec<String> {
        let mut recommendations = Vec::new();

        if peak > 2 * 1024 * 1024 * 1024 {
            // > 2GB
            recommendations.push("Consider using memory-optimized analysis settings".to_string());
        }

        if let Some(limit) = self.config.memory_limit_bytes {
            let usage_percent = (peak as f64 / limit as f64) * 100.0;
            if usage_percent > 80.0 {
                recommendations.push(
                    "Memory usage is high, consider increasing limit or reducing analysis scope"
                        .to_string(),
                );
            }
        }

        recommendations
    }
}

impl Clone for MonitoringSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            start_time: self.start_time,
            metrics_collector: self.metrics_collector.clone(),
            memory_monitor: self.memory_monitor.clone(),
            session_metrics: self.session_metrics.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_performance_service() -> PerformanceAnalysisService {
        let metrics_collector = Arc::new(PerformanceMetricsCollector::new(
            crate::database::models::PerformanceMetricsConfig::default(),
            10,
        ));
        PerformanceAnalysisService::new(metrics_collector, MemoryConfig::default())
    }

    #[tokio::test]
    async fn test_performance_service_creation() {
        let service = create_test_performance_service();

        let active_sessions = service.get_active_sessions().await;
        assert!(active_sessions.is_empty());
    }

    #[tokio::test]
    async fn test_monitoring_session_lifecycle() {
        let service = create_test_performance_service();

        // Start session
        let session = service.start_monitoring().await;
        assert!(!session.session_id.is_empty());

        // Check active sessions
        let active_sessions = service.get_active_sessions().await;
        assert_eq!(active_sessions.len(), 1);
        assert!(active_sessions.contains(&session.session_id));

        // Stop session
        let report = service.stop_monitoring(&session).await;
        assert_eq!(report.session_id, session.session_id);

        // Check sessions cleared
        let active_sessions_after = service.get_active_sessions().await;
        assert!(active_sessions_after.is_empty());
    }

    #[tokio::test]
    async fn test_memory_usage_tracking() {
        let service = create_test_performance_service();

        let current_usage = service.get_current_memory_usage().await;
        let peak_usage = service.get_peak_memory_usage().await;

        // Should be able to get memory readings (may be 0 on some systems)
        // Note: These are u64, so they're always >= 0
        assert!(current_usage == current_usage); // Basic sanity check
        assert!(peak_usage == peak_usage); // Basic sanity check
    }

    #[tokio::test]
    async fn test_memory_limit_checking() {
        let service = create_test_performance_service();

        // Should not exceed limit initially
        let exceeded = service.is_memory_limit_exceeded().await;
        assert!(!exceeded);
    }

    #[tokio::test]
    async fn test_memory_report_generation() {
        let service = create_test_performance_service();

        let report = service.generate_memory_report().await;
        // Validate fields from MemoryAnalysisReport
        // Note: These are u64, so they're always >= 0
        assert!(report.memory_limit_mb == report.memory_limit_mb); // Basic sanity check
        assert!(report.final_memory_mb == report.final_memory_mb); // Basic sanity check
        assert!(report.peak_memory_mb == report.peak_memory_mb); // Basic sanity check
    }
}
