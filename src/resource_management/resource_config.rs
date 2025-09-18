//! Resource configuration and limits management

use super::error::{ResourceError, ResourceResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main resource configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Memory limits
    pub memory: MemoryLimits,
    /// CPU and processing limits
    pub processing: ProcessingLimits,
    /// File handling limits
    pub file_handling: FileHandlingLimits,
    /// Monitoring settings
    pub monitoring: MonitoringConfig,
    /// Graceful degradation settings
    pub degradation: DegradationConfig,
}

/// Memory-related limits and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Memory pressure threshold for warnings (0.0-1.0)
    pub warning_threshold: f64,
    /// Memory pressure threshold for critical alerts (0.0-1.0)
    pub critical_threshold: f64,
    /// Enable memory pressure monitoring
    pub enable_monitoring: bool,
    /// Memory allocation chunk size for large operations
    pub allocation_chunk_size: u64,
}

/// Processing and CPU limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingLimits {
    /// Maximum number of concurrent analyses
    pub max_concurrent_analyses: usize,
    /// Maximum number of file handles
    pub max_file_handles: usize,
    /// Maximum CPU usage percentage (0.0-1.0)
    pub max_cpu_usage: f64,
    /// Analysis timeout duration
    pub analysis_timeout: Duration,
    /// Background processing thread count
    pub background_threads: usize,
}

/// File handling limits and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHandlingLimits {
    /// Maximum file size for in-memory processing (bytes)
    pub max_file_size_memory: u64,
    /// Chunk size for streaming large files (bytes)
    pub streaming_chunk_size: usize,
    /// Maximum number of files processed concurrently
    pub max_concurrent_files: usize,
    /// Enable streaming for large files
    pub enable_streaming: bool,
    /// File type filters (empty = all types allowed)
    pub allowed_file_types: Vec<String>,
}

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable resource monitoring
    pub enabled: bool,
    /// Monitoring interval in seconds
    pub interval_seconds: u64,
    /// Number of historical data points to keep
    pub history_size: usize,
    /// Enable metrics export (for Prometheus, etc.)
    pub enable_metrics_export: bool,
    /// Log resource usage periodically
    pub log_usage: bool,
}

/// Graceful degradation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationConfig {
    /// Enable graceful degradation
    pub enabled: bool,
    /// Light degradation threshold (0.0-1.0)
    pub light_threshold: f64,
    /// Heavy degradation threshold (0.0-1.0)
    pub heavy_threshold: f64,
    /// Emergency mode threshold (0.0-1.0)
    pub emergency_threshold: f64,
    /// Auto-recovery enabled
    pub auto_recovery: bool,
    /// Recovery threshold (when to return to normal)
    pub recovery_threshold: f64,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            memory: MemoryLimits::default(),
            processing: ProcessingLimits::default(),
            file_handling: FileHandlingLimits::default(),
            monitoring: MonitoringConfig::default(),
            degradation: DegradationConfig::default(),
        }
    }
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            warning_threshold: 0.7,
            critical_threshold: 0.9,
            enable_monitoring: true,
            allocation_chunk_size: 64 * 1024 * 1024, // 64MB
        }
    }
}

impl Default for ProcessingLimits {
    fn default() -> Self {
        Self {
            max_concurrent_analyses: 10,
            max_file_handles: 1000,
            max_cpu_usage: 0.8,
            analysis_timeout: Duration::from_secs(300), // 5 minutes
            background_threads: num_cpus::get().max(4),
        }
    }
}

impl Default for FileHandlingLimits {
    fn default() -> Self {
        Self {
            max_file_size_memory: 100 * 1024 * 1024, // 100MB
            streaming_chunk_size: 8 * 1024 * 1024,   // 8MB
            max_concurrent_files: 50,
            enable_streaming: true,
            allowed_file_types: vec![
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "jsx".to_string(),
            ],
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 5,
            history_size: 100,
            enable_metrics_export: true,
            log_usage: false,
        }
    }
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            light_threshold: 0.7,
            heavy_threshold: 0.8,
            emergency_threshold: 0.9,
            auto_recovery: true,
            recovery_threshold: 0.6,
        }
    }
}

impl ResourceConfig {
    /// Validates the configuration for consistency and correctness
    pub fn validate(&self) -> ResourceResult<()> {
        // Memory validation
        if self.memory.max_memory_bytes == 0 {
            return Err(ResourceError::InvalidConfiguration(
                "max_memory_bytes must be greater than 0".to_string(),
            ));
        }

        if self.memory.warning_threshold >= self.memory.critical_threshold {
            return Err(ResourceError::InvalidConfiguration(
                "warning_threshold must be less than critical_threshold".to_string(),
            ));
        }

        if self.memory.critical_threshold > 1.0 || self.memory.warning_threshold < 0.0 {
            return Err(ResourceError::InvalidConfiguration(
                "thresholds must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Processing validation
        if self.processing.max_concurrent_analyses == 0 {
            return Err(ResourceError::InvalidConfiguration(
                "max_concurrent_analyses must be greater than 0".to_string(),
            ));
        }

        if self.processing.max_cpu_usage > 1.0 || self.processing.max_cpu_usage <= 0.0 {
            return Err(ResourceError::InvalidConfiguration(
                "max_cpu_usage must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Degradation validation
        if self.degradation.enabled {
            let thresholds = [
                self.degradation.light_threshold,
                self.degradation.heavy_threshold,
                self.degradation.emergency_threshold,
            ];

            for i in 1..thresholds.len() {
                if thresholds[i] <= thresholds[i - 1] {
                    return Err(ResourceError::InvalidConfiguration(
                        "degradation thresholds must be in ascending order".to_string(),
                    ));
                }
            }

            if self.degradation.recovery_threshold >= self.degradation.light_threshold {
                return Err(ResourceError::InvalidConfiguration(
                    "recovery_threshold must be less than light_threshold".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Creates a configuration optimized for development
    pub fn development() -> Self {
        Self {
            memory: MemoryLimits {
                max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB for development
                warning_threshold: 0.8,
                critical_threshold: 0.95,
                enable_monitoring: true,
                allocation_chunk_size: 32 * 1024 * 1024, // 32MB
            },
            processing: ProcessingLimits {
                max_concurrent_analyses: 5, // Lower for development
                max_file_handles: 500,
                max_cpu_usage: 0.7, // Leave more for IDE, etc.
                analysis_timeout: Duration::from_secs(60),
                background_threads: 2,
            },
            file_handling: FileHandlingLimits {
                max_file_size_memory: 50 * 1024 * 1024, // 50MB
                streaming_chunk_size: 4 * 1024 * 1024,  // 4MB
                max_concurrent_files: 25,
                enable_streaming: true,
                allowed_file_types: vec!["rs".to_string()], // Rust only for dev
            },
            monitoring: MonitoringConfig {
                enabled: true,
                interval_seconds: 10, // Less frequent monitoring
                history_size: 50,
                enable_metrics_export: false, // No metrics export in dev
                log_usage: true,              // Enable logging for development
            },
            degradation: DegradationConfig {
                enabled: true,
                light_threshold: 0.8,
                heavy_threshold: 0.9,
                emergency_threshold: 0.95,
                auto_recovery: true,
                recovery_threshold: 0.7,
            },
        }
    }

    /// Creates a configuration optimized for production
    pub fn production() -> Self {
        Self {
            memory: MemoryLimits {
                max_memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB for production
                warning_threshold: 0.7,
                critical_threshold: 0.85,
                enable_monitoring: true,
                allocation_chunk_size: 128 * 1024 * 1024, // 128MB
            },
            processing: ProcessingLimits {
                max_concurrent_analyses: 20, // Higher for production
                max_file_handles: 2000,
                max_cpu_usage: 0.9, // Can use more CPU in production
                analysis_timeout: Duration::from_secs(600), // 10 minutes
                background_threads: num_cpus::get() * 2,
            },
            file_handling: FileHandlingLimits {
                max_file_size_memory: 200 * 1024 * 1024, // 200MB
                streaming_chunk_size: 16 * 1024 * 1024,  // 16MB
                max_concurrent_files: 100,
                enable_streaming: true,
                allowed_file_types: vec![], // All types allowed
            },
            monitoring: MonitoringConfig {
                enabled: true,
                interval_seconds: 5,
                history_size: 200,
                enable_metrics_export: true,
                log_usage: false, // Less logging in production
            },
            degradation: DegradationConfig {
                enabled: true,
                light_threshold: 0.7,
                heavy_threshold: 0.8,
                emergency_threshold: 0.9,
                auto_recovery: true,
                recovery_threshold: 0.6,
            },
        }
    }

    /// Creates a configuration for testing with very low limits
    pub fn testing() -> Self {
        Self {
            memory: MemoryLimits {
                max_memory_bytes: 100 * 1024 * 1024, // 100MB
                warning_threshold: 0.5,
                critical_threshold: 0.8,
                enable_monitoring: true,
                allocation_chunk_size: 1024 * 1024, // 1MB
            },
            processing: ProcessingLimits {
                max_concurrent_analyses: 2,
                max_file_handles: 50,
                max_cpu_usage: 0.5,
                analysis_timeout: Duration::from_secs(10),
                background_threads: 1,
            },
            file_handling: FileHandlingLimits {
                max_file_size_memory: 1024 * 1024, // 1MB
                streaming_chunk_size: 64 * 1024,   // 64KB
                max_concurrent_files: 5,
                enable_streaming: true,
                allowed_file_types: vec!["rs".to_string()],
            },
            monitoring: MonitoringConfig {
                enabled: true,
                interval_seconds: 1,
                history_size: 10,
                enable_metrics_export: false,
                log_usage: true,
            },
            degradation: DegradationConfig {
                enabled: true,
                light_threshold: 0.5,
                heavy_threshold: 0.7,
                emergency_threshold: 0.8,
                auto_recovery: true,
                recovery_threshold: 0.4,
            },
        }
    }
}

/// Resource limits that can be updated at runtime
pub struct ResourceLimits {
    pub memory_limit: u64,
    pub concurrent_analyses: usize,
    pub file_handles: usize,
    pub analysis_timeout: Duration,
}

impl From<&ResourceConfig> for ResourceLimits {
    fn from(config: &ResourceConfig) -> Self {
        Self {
            memory_limit: config.memory.max_memory_bytes,
            concurrent_analyses: config.processing.max_concurrent_analyses,
            file_handles: config.processing.max_file_handles,
            analysis_timeout: config.processing.analysis_timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = ResourceConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_development_config_validation() {
        let config = ResourceConfig::development();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_production_config_validation() {
        let config = ResourceConfig::production();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_thresholds() {
        let mut config = ResourceConfig::default();
        config.memory.warning_threshold = 0.9;
        config.memory.critical_threshold = 0.8;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_degradation_thresholds() {
        let mut config = ResourceConfig::default();
        config.degradation.light_threshold = 0.8;
        config.degradation.heavy_threshold = 0.7;

        assert!(config.validate().is_err());
    }
}
