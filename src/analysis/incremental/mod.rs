//! Incremental Analysis System for UV-91 Phase 2
//!
//! Implements intelligent incremental analysis to achieve 50%+ reduction in re-analysis time
//! for enterprise codebases by tracking file changes and dependencies.
//!
//! Key Components:
//! - Change Detection: Track file modifications and validate changes
//! - Dependency Tracking: Build and maintain dependency graphs
//! - State Management: Persist analysis state between runs
//! - Incremental Engine: Orchestrate selective re-analysis

/// Analysis components for change detector.
pub mod change_detector;
/// Analysis components for dependency tracker.
pub mod dependency_tracker;
/// Analysis components for incremental engine.
pub mod incremental_engine;
/// Analysis components for state manager.
pub mod state_manager;

pub use change_detector::{ChangeDetector, ChangeSet, FileState};
pub use dependency_tracker::{ChangeImpact, DependencyGraph, DependencyTracker};
pub use incremental_engine::{IncrementalAnalysisEngine, IncrementalConfig};
pub use state_manager::{IncrementalState, IncrementalStateManager};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core change detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for incrementalanalysis.
///
/// Provides sensible defaults that can be overridden via builder methods
/// or loaded from configuration files.
pub struct IncrementalAnalysisConfig {
    /// Enable incremental analysis (can be disabled for debugging)
    pub enabled: bool,

    /// State persistence configuration
    pub state_file_path: Option<PathBuf>,

    /// Maximum age for cached analysis results (in hours)
    pub max_cache_age_hours: u32,

    /// Force full analysis threshold (percentage of files changed)
    pub full_analysis_threshold: f32,

    /// Enable dependency change propagation
    pub enable_dependency_propagation: bool,

    /// Change detection settings
    pub change_detection: ChangeDetectionConfig,

    /// Performance optimization settings
    pub performance: PerformanceConfig,
}

/// Change detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for changedetection.
///
/// Provides sensible defaults that can be overridden via builder methods
/// or loaded from configuration files.
pub struct ChangeDetectionConfig {
    /// Use content hash for change detection (more accurate but slower)
    pub use_content_hash: bool,

    /// Hash algorithm to use (blake3, sha256, etc.)
    pub hash_algorithm: String,

    /// Check file modification time as primary change indicator
    pub check_mtime: bool,

    /// Ignore certain file patterns for change detection
    pub ignore_patterns: Vec<String>,
}

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for performance.
///
/// Provides sensible defaults that can be overridden via builder methods
/// or loaded from configuration files.
pub struct PerformanceConfig {
    /// Maximum number of files to process in parallel
    pub max_parallel_files: usize,

    /// Batch size for dependency analysis
    pub dependency_batch_size: usize,

    /// Enable memory optimization for large codebases
    pub enable_memory_optimization: bool,

    /// Cache size limits
    pub cache_limits: CacheLimits,
}

/// Cache size and memory limits
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for cachelimits.
pub struct CacheLimits {
    /// Maximum number of file states to keep in memory
    pub max_file_states: usize,

    /// Maximum memory usage for incremental state (MB)
    pub max_memory_mb: usize,

    /// Maximum dependency graph size (nodes)
    pub max_dependency_nodes: usize,
}

impl Default for IncrementalAnalysisConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            state_file_path: Some(PathBuf::from(".uveddi/incremental_state.json")),
            max_cache_age_hours: 24,
            full_analysis_threshold: 0.30, // 30% of files changed triggers full analysis
            enable_dependency_propagation: true,
            change_detection: ChangeDetectionConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for ChangeDetectionConfig {
    fn default() -> Self {
        Self {
            use_content_hash: true,
            hash_algorithm: "blake3".to_string(),
            check_mtime: true,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.bak".to_string(),
                ".git/*".to_string(),
                "target/*".to_string(),
                "node_modules/*".to_string(),
            ],
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_parallel_files: num_cpus::get().max(4),
            dependency_batch_size: 100,
            enable_memory_optimization: true,
            cache_limits: CacheLimits::default(),
        }
    }
}

impl Default for CacheLimits {
    fn default() -> Self {
        Self {
            max_file_states: 50_000,
            max_memory_mb: 1024, // 1GB default limit
            max_dependency_nodes: 100_000,
        }
    }
}

/// Overall incremental analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for incrementalanalysisresult.
pub struct IncrementalAnalysisResult {
    /// Whether incremental analysis was used
    pub was_incremental: bool,

    /// Number of files that were re-analyzed
    pub files_reanalyzed: usize,

    /// Total number of files in the project
    pub total_files: usize,

    /// Time saved compared to full analysis (estimated)
    pub time_saved_ms: u64,

    /// Analysis timestamp
    pub analysis_time: DateTime<Utc>,

    /// Change detection summary
    pub change_summary: ChangeSummary,

    /// Performance metrics
    pub performance_metrics: IncrementalPerformanceMetrics,
}

/// Summary of detected changes
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for changesummary.
pub struct ChangeSummary {
    /// Files that were directly changed
    pub changed_files: Vec<PathBuf>,

    /// Files that were affected by dependency changes
    pub affected_files: Vec<PathBuf>,

    /// New files that were added
    pub new_files: Vec<PathBuf>,

    /// Files that were deleted
    pub deleted_files: Vec<PathBuf>,

    /// Percentage of files changed
    pub change_percentage: f32,
}

/// Performance metrics for incremental analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for incrementalperformancemetrics.
pub struct IncrementalPerformanceMetrics {
    /// Time spent on change detection
    pub change_detection_time_ms: u64,

    /// Time spent on dependency analysis
    pub dependency_analysis_time_ms: u64,

    /// Time spent on actual re-analysis
    pub reanalysis_time_ms: u64,

    /// Time spent on state persistence
    pub state_persistence_time_ms: u64,

    /// Memory usage peak during incremental analysis
    pub peak_memory_usage_mb: f64,

    /// Cache hit rate for various components
    pub cache_hit_rates: CacheHitRates,
}

impl Default for IncrementalPerformanceMetrics {
    fn default() -> Self {
        Self {
            change_detection_time_ms: 0,
            dependency_analysis_time_ms: 0,
            reanalysis_time_ms: 0,
            state_persistence_time_ms: 0,
            peak_memory_usage_mb: 0.0,
            cache_hit_rates: CacheHitRates::default(),
        }
    }
}

/// Cache hit rates for different cache types
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for cachehitrates.
pub struct CacheHitRates {
    /// File state cache hit rate
    pub file_state_cache: f64,

    /// Dependency cache hit rate
    pub dependency_cache: f64,

    /// Analysis result cache hit rate
    pub analysis_result_cache: f64,

    /// AST cache hit rate
    pub ast_cache: f64,
}

impl Default for CacheHitRates {
    fn default() -> Self {
        Self {
            file_state_cache: 0.0,
            dependency_cache: 0.0,
            analysis_result_cache: 0.0,
            ast_cache: 0.0,
        }
    }
}

/// Error types for incremental analysis
#[derive(thiserror::Error, Debug)]
pub enum IncrementalAnalysisError {
    #[error("Change detection failed: {message}")]
    ChangeDetectionError { message: String },

    #[error("Dependency tracking error: {message}")]
    DependencyTrackingError { message: String },

    #[error("State persistence error: {message}")]
    StatePersistenceError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("IO error during incremental analysis: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Analysis engine error: {0}")]
    AnalysisEngineError(#[from] crate::error::UveddiError),
}

/// Result type for incremental analysis operations
pub type Result<T> = std::result::Result<T, IncrementalAnalysisError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IncrementalAnalysisConfig::default();
        assert!(config.enabled);
        assert!(config.enable_dependency_propagation);
        assert_eq!(config.full_analysis_threshold, 0.30);
        assert_eq!(config.change_detection.hash_algorithm, "blake3");
    }

    #[test]
    fn test_cache_limits() {
        let limits = CacheLimits::default();
        assert_eq!(limits.max_memory_mb, 1024);
        assert_eq!(limits.max_file_states, 50_000);
        assert_eq!(limits.max_dependency_nodes, 100_000);
    }

    #[test]
    fn test_performance_config() {
        let perf = PerformanceConfig::default();
        assert!(perf.max_parallel_files >= 4);
        assert_eq!(perf.dependency_batch_size, 100);
        assert!(perf.enable_memory_optimization);
    }
}
