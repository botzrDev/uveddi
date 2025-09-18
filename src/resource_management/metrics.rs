//! Metrics and system information collection

use super::{
    analysis_orchestrator::OrchestrationStats, degradation_manager::DegradationLevel,
    memory_tracker::MemoryStats,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Complete resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage statistics
    pub memory: MemoryStats,
    /// Number of active analyses
    pub active_analyses: usize,
    /// Current degradation level
    pub degradation_level: DegradationLevel,
}

/// System-level metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Timestamp when metrics were collected
    pub timestamp: u64,
    /// CPU usage percentage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Memory metrics
    pub memory: SystemMemoryMetrics,
    /// Disk I/O metrics
    pub disk_io: DiskIoMetrics,
    /// Network metrics
    pub network: NetworkMetrics,
    /// Process-specific metrics
    pub process: ProcessMetrics,
}

/// System memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemoryMetrics {
    /// Total system memory in bytes
    pub total_bytes: u64,
    /// Available system memory in bytes
    pub available_bytes: u64,
    /// Used system memory in bytes
    pub used_bytes: u64,
    /// Memory usage percentage (0.0 to 1.0)
    pub usage_percent: f64,
    /// Swap memory total in bytes
    pub swap_total_bytes: u64,
    /// Swap memory used in bytes
    pub swap_used_bytes: u64,
}

/// Disk I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoMetrics {
    /// Bytes read from disk
    pub bytes_read: u64,
    /// Bytes written to disk
    pub bytes_written: u64,
    /// Number of read operations
    pub read_ops: u64,
    /// Number of write operations
    pub write_ops: u64,
    /// Average read latency in microseconds
    pub avg_read_latency_us: f64,
    /// Average write latency in microseconds
    pub avg_write_latency_us: f64,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes transmitted
    pub bytes_transmitted: u64,
    /// Packets received
    pub packets_received: u64,
    /// Packets transmitted
    pub packets_transmitted: u64,
    /// Number of network errors
    pub errors: u64,
}

/// Process-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Process ID
    pub pid: u32,
    /// CPU time used by process in seconds
    pub cpu_time_seconds: f64,
    /// RSS (Resident Set Size) memory in bytes
    pub memory_rss_bytes: u64,
    /// Virtual memory size in bytes
    pub memory_vsize_bytes: u64,
    /// Number of file descriptors open
    pub open_file_descriptors: u32,
    /// Number of threads
    pub thread_count: u32,
}

/// Time-series metrics storage
#[derive(Debug)]
pub struct MetricsHistory {
    /// Maximum number of data points to store
    max_size: usize,
    /// Resource usage history
    resource_usage: VecDeque<(Instant, ResourceUsage)>,
    /// System metrics history
    system_metrics: VecDeque<(Instant, SystemMetrics)>,
    /// Orchestration statistics history
    orchestration_stats: VecDeque<(Instant, OrchestrationStats)>,
}

impl MetricsHistory {
    /// Creates a new metrics history with specified capacity
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            resource_usage: VecDeque::with_capacity(max_size),
            system_metrics: VecDeque::with_capacity(max_size),
            orchestration_stats: VecDeque::with_capacity(max_size),
        }
    }

    /// Records resource usage metrics
    pub fn record_resource_usage(&mut self, usage: ResourceUsage) {
        self.resource_usage.push_back((Instant::now(), usage));

        if self.resource_usage.len() > self.max_size {
            self.resource_usage.pop_front();
        }
    }

    /// Records system metrics
    pub fn record_system_metrics(&mut self, metrics: SystemMetrics) {
        self.system_metrics.push_back((Instant::now(), metrics));

        if self.system_metrics.len() > self.max_size {
            self.system_metrics.pop_front();
        }
    }

    /// Records orchestration statistics
    pub fn record_orchestration_stats(&mut self, stats: OrchestrationStats) {
        self.orchestration_stats.push_back((Instant::now(), stats));

        if self.orchestration_stats.len() > self.max_size {
            self.orchestration_stats.pop_front();
        }
    }

    /// Gets recent resource usage data points
    pub fn get_resource_usage_history(&self, duration: Duration) -> Vec<(Instant, ResourceUsage)> {
        let cutoff = Instant::now() - duration;
        self.resource_usage
            .iter()
            .filter(|(timestamp, _)| *timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// Gets recent system metrics data points
    pub fn get_system_metrics_history(&self, duration: Duration) -> Vec<(Instant, SystemMetrics)> {
        let cutoff = Instant::now() - duration;
        self.system_metrics
            .iter()
            .filter(|(timestamp, _)| *timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// Calculates average memory usage over a time period
    pub fn average_memory_usage(&self, duration: Duration) -> Option<f64> {
        let history = self.get_resource_usage_history(duration);

        if history.is_empty() {
            return None;
        }

        let sum: f64 = history
            .iter()
            .map(|(_, usage)| usage.memory.usage_percent)
            .sum();

        Some(sum / history.len() as f64)
    }

    /// Gets peak memory usage over a time period
    pub fn peak_memory_usage(&self, duration: Duration) -> Option<f64> {
        let history = self.get_resource_usage_history(duration);

        history
            .iter()
            .map(|(_, usage)| usage.memory.usage_percent)
            .fold(None, |acc, x| Some(acc.unwrap_or(x).max(x)))
    }

    /// Calculates analysis throughput (analyses completed per minute)
    pub fn analysis_throughput(&self, duration: Duration) -> f64 {
        let history = self.get_system_metrics_history(duration);

        if history.len() < 2 {
            return 0.0;
        }

        // This would need to be implemented with actual orchestration stats
        // For now, return a placeholder
        0.0
    }

    /// Clears all historical data
    pub fn clear(&mut self) {
        self.resource_usage.clear();
        self.system_metrics.clear();
        self.orchestration_stats.clear();
    }

    /// Gets the current storage size
    pub fn size(&self) -> usize {
        self.resource_usage.len() + self.system_metrics.len() + self.orchestration_stats.len()
    }
}

/// System metrics collector
#[derive(Clone)]
pub struct SystemMetricsCollector {
    start_time: Instant,
    last_collection: Option<Instant>,
}

impl SystemMetricsCollector {
    /// Creates a new system metrics collector
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_collection: None,
        }
    }

    /// Collects current system metrics
    pub async fn collect(
        &mut self,
    ) -> Result<SystemMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cpu_usage = self.collect_cpu_usage().await?;
        let memory = self.collect_memory_metrics().await?;
        let disk_io = self.collect_disk_io_metrics().await?;
        let network = self.collect_network_metrics().await?;
        let process = self.collect_process_metrics().await?;

        self.last_collection = Some(Instant::now());

        Ok(SystemMetrics {
            timestamp,
            cpu_usage,
            memory,
            disk_io,
            network,
            process,
        })
    }

    async fn collect_cpu_usage(&self) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // This is a simplified implementation
        // In a real system, you would use platform-specific APIs

        #[cfg(target_os = "linux")]
        {
            self.collect_cpu_usage_linux().await
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback implementation
            Ok(0.0)
        }
    }

    #[cfg(target_os = "linux")]
    async fn collect_cpu_usage_linux(
        &self,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Read from /proc/stat for system-wide CPU usage
        let contents = tokio::fs::read_to_string("/proc/stat").await?;
        let line = contents.lines().next().unwrap_or("");

        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 5 {
            return Ok(0.0);
        }

        let user: u64 = fields[1].parse()?;
        let nice: u64 = fields[2].parse()?;
        let system: u64 = fields[3].parse()?;
        let idle: u64 = fields[4].parse()?;

        let total = user + nice + system + idle;
        let used = user + nice + system;

        Ok(used as f64 / total as f64)
    }

    async fn collect_memory_metrics(
        &self,
    ) -> Result<SystemMemoryMetrics, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "linux")]
        {
            self.collect_memory_metrics_linux().await
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback implementation
            Ok(SystemMemoryMetrics {
                total_bytes: 0,
                available_bytes: 0,
                used_bytes: 0,
                usage_percent: 0.0,
                swap_total_bytes: 0,
                swap_used_bytes: 0,
            })
        }
    }

    #[cfg(target_os = "linux")]
    async fn collect_memory_metrics_linux(
        &self,
    ) -> Result<SystemMemoryMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let contents = tokio::fs::read_to_string("/proc/meminfo").await?;

        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        let mut swap_total_kb = 0u64;
        let mut swap_free_kb = 0u64;

        for line in contents.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let value: u64 = parts[1].parse().unwrap_or(0);

                match parts[0] {
                    "MemTotal:" => total_kb = value,
                    "MemAvailable:" => available_kb = value,
                    "SwapTotal:" => swap_total_kb = value,
                    "SwapFree:" => swap_free_kb = value,
                    _ => {}
                }
            }
        }

        let total_bytes = total_kb * 1024;
        let available_bytes = available_kb * 1024;
        let used_bytes = total_bytes - available_bytes;
        let usage_percent = used_bytes as f64 / total_bytes as f64;
        let swap_total_bytes = swap_total_kb * 1024;
        let swap_used_bytes = swap_total_bytes - (swap_free_kb * 1024);

        Ok(SystemMemoryMetrics {
            total_bytes,
            available_bytes,
            used_bytes,
            usage_percent,
            swap_total_bytes,
            swap_used_bytes,
        })
    }

    async fn collect_disk_io_metrics(
        &self,
    ) -> Result<DiskIoMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified implementation - would read from /proc/diskstats on Linux
        Ok(DiskIoMetrics {
            bytes_read: 0,
            bytes_written: 0,
            read_ops: 0,
            write_ops: 0,
            avg_read_latency_us: 0.0,
            avg_write_latency_us: 0.0,
        })
    }

    async fn collect_network_metrics(
        &self,
    ) -> Result<NetworkMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified implementation - would read from /proc/net/dev on Linux
        Ok(NetworkMetrics {
            bytes_received: 0,
            bytes_transmitted: 0,
            packets_received: 0,
            packets_transmitted: 0,
            errors: 0,
        })
    }

    async fn collect_process_metrics(
        &self,
    ) -> Result<ProcessMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let pid = std::process::id();

        #[cfg(target_os = "linux")]
        {
            self.collect_process_metrics_linux(pid).await
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(ProcessMetrics {
                pid,
                cpu_time_seconds: 0.0,
                memory_rss_bytes: 0,
                memory_vsize_bytes: 0,
                open_file_descriptors: 0,
                thread_count: 1,
            })
        }
    }

    #[cfg(target_os = "linux")]
    async fn collect_process_metrics_linux(
        &self,
        pid: u32,
    ) -> Result<ProcessMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Read from /proc/self/stat for process info
        let stat_contents = tokio::fs::read_to_string(format!("/proc/{}/stat", pid)).await?;
        let fields: Vec<&str> = stat_contents.split_whitespace().collect();

        let memory_vsize_bytes = if fields.len() > 22 {
            fields[22].parse().unwrap_or(0)
        } else {
            0
        };

        // Read from /proc/self/status for more detailed info
        let status_contents = tokio::fs::read_to_string(format!("/proc/{}/status", pid)).await?;
        let mut memory_rss_bytes = 0u64;
        let mut thread_count = 1u32;

        for line in status_contents.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                match parts[0] {
                    "VmRSS:" => {
                        let kb: u64 = parts[1].parse().unwrap_or(0);
                        memory_rss_bytes = kb * 1024;
                    }
                    "Threads:" => {
                        thread_count = parts[1].parse().unwrap_or(1);
                    }
                    _ => {}
                }
            }
        }

        // Count open file descriptors
        let fd_dir = format!("/proc/{}/fd", pid);
        let open_file_descriptors = match tokio::fs::read_dir(&fd_dir).await {
            Ok(mut entries) => {
                let mut count = 0;
                while entries.next_entry().await?.is_some() {
                    count += 1;
                }
                count
            }
            Err(_) => 0,
        };

        Ok(ProcessMetrics {
            pid,
            cpu_time_seconds: 0.0, // Would need to calculate from stat fields
            memory_rss_bytes,
            memory_vsize_bytes,
            open_file_descriptors,
            thread_count,
        })
    }
}

impl Default for SystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_history() {
        let mut history = MetricsHistory::new(5);

        // Add some resource usage data
        for i in 0..10 {
            let usage = ResourceUsage {
                memory: MemoryStats {
                    current: i * 100,
                    peak: i * 150,
                    limit: 1000,
                    available: 1000 - (i * 100),
                    usage_percent: (i * 10) as f64,
                    allocation_count: i as usize,
                    component_usage: std::collections::HashMap::new(),
                },
                active_analyses: i as usize,
                degradation_level: DegradationLevel::Normal,
            };

            history.record_resource_usage(usage);
        }

        // Should only keep the last 5 entries
        assert_eq!(history.resource_usage.len(), 5);

        // Check that the most recent data is preserved
        let latest_usage = &history.resource_usage.back().unwrap().1;
        assert_eq!(latest_usage.memory.current, 900);
    }

    #[tokio::test]
    async fn test_system_metrics_collection() {
        let mut collector = SystemMetricsCollector::new();

        // This test may fail on systems without /proc filesystem
        match collector.collect().await {
            Ok(metrics) => {
                assert!(metrics.timestamp > 0);
                assert!(metrics.cpu_usage >= 0.0);
                assert!(metrics.memory.total_bytes >= metrics.memory.used_bytes);
            }
            Err(_) => {
                // Expected on non-Linux systems or systems without proper /proc access
                // The test passes as long as it doesn't panic
            }
        }
    }
}
