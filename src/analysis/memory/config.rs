//! Memory optimization configuration for UV-210/UV-26
//! Provides centralized configuration for all memory optimization features

use serde::{Deserialize, Serialize};
use crate::analysis::memory::allocator::AllocationStrategy;

/// Comprehensive memory optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimizationConfig {
    /// Whether memory optimization features are enabled
    pub enabled: bool,

    /// Target maximum memory usage in bytes (default: 8GB)
    pub target_max_memory_bytes: usize,

    /// Object pool configuration
    pub object_pools: ObjectPoolConfig,

    /// Arena allocation configuration
    pub arena_allocation: ArenaConfig,

    /// AST cache optimization configuration
    pub ast_cache_optimization: AstCacheOptimizationConfig,

    /// AI memory optimization configuration
    pub ai_memory_optimization: AiMemoryConfig,

    /// Memory monitoring configuration
    pub monitoring: MemoryMonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPoolConfig {
    /// Whether object pooling is enabled
    pub enabled: bool,

    /// Default pool capacity for detector configurations
    pub detector_pool_capacity: usize,

    /// Default pool capacity for temporary objects (strings, vectors, etc.)
    pub temporary_pool_capacity: usize,

    /// Number of shards for concurrent access (0 = auto-detect based on CPU count)
    pub shard_count: usize,

    /// Allocation strategy for pools
    #[serde(skip)] // Skip serialization for complex enum
    pub allocation_strategy: AllocationStrategy,

    /// Pre-population percentage (0-100) for pools
    pub pre_population_percentage: f64,

    /// Maximum number of objects to keep in each pool
    pub max_pool_size: usize,

    /// Minimum utilization threshold (%) before pool is considered under-utilized
    pub min_utilization_threshold: f64,

    /// Maximum utilization threshold (%) before pool is considered over-utilized
    pub max_utilization_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaConfig {
    /// Whether arena allocation is enabled
    pub enabled: bool,

    /// Default arena size in MB for single-file analysis
    pub default_arena_size_mb: usize,

    /// Maximum number of concurrent arenas for batch processing
    pub max_concurrent_arenas: usize,

    /// Whether to automatically reset arenas between file analyses
    pub auto_reset_between_files: bool,

    /// Arena size limit warning threshold (percentage)
    pub size_warning_threshold: f64,

    /// Enable arena memory efficiency monitoring
    pub enable_efficiency_monitoring: bool,

    /// Pre-allocate arenas for common file types
    pub pre_allocate_common_arenas: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstCacheOptimizationConfig {
    /// Whether zero-copy AST caching is enabled
    pub zero_copy_enabled: bool,

    /// Whether memory mapping is enabled for large ASTs
    pub memory_mapping_enabled: bool,

    /// Threshold size for using memory mapping (bytes)
    pub memory_mapping_threshold_bytes: usize,

    /// Directory for zero-copy cache storage
    pub zero_copy_cache_directory: String,

    /// Maximum number of cached AST entries in memory
    pub max_cached_asts: usize,

    /// Maximum memory usage for AST cache in MB
    pub max_ast_cache_memory_mb: usize,

    /// Whether to enable AST cache metrics collection
    pub enable_ast_cache_metrics: bool,

    /// Whether to enable LRU eviction for AST cache
    pub enable_lru_eviction: bool,

    /// Threshold file size for using zero-copy cache (bytes)
    pub zero_copy_threshold_bytes: usize,

    /// Whether to enable concurrent AST cache access
    pub enable_concurrent_access: bool,

    /// Whether to enable AST cache warming for common files
    pub enable_cache_warming: bool,

    /// Cache warming concurrency level
    pub cache_warming_concurrency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMemoryConfig {
    /// Whether AI memory optimization is enabled
    pub enabled: bool,

    /// Maximum AI context size in bytes
    pub max_context_size_bytes: usize,

    /// Chunk size for processing large issue sets
    pub analysis_chunk_size: usize,

    /// Whether to use streaming context management
    pub streaming_context: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMonitoringConfig {
    /// Whether memory monitoring is enabled
    pub enabled: bool,

    /// Interval for memory usage sampling in milliseconds
    pub sampling_interval_ms: u64,

    /// Whether to export metrics for observability
    pub export_metrics: bool,
}

impl Default for MemoryOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_max_memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            object_pools: ObjectPoolConfig::default(),
            arena_allocation: ArenaConfig::default(),
            ast_cache_optimization: AstCacheOptimizationConfig::default(),
            ai_memory_optimization: AiMemoryConfig::default(),
            monitoring: MemoryMonitoringConfig::default(),
        }
    }
}

impl Default for ObjectPoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detector_pool_capacity: 100,
            temporary_pool_capacity: 200,
            shard_count: 0, // Auto-detect
            allocation_strategy: AllocationStrategy::default(),
            pre_population_percentage: 25.0,
            max_pool_size: 1000,
            min_utilization_threshold: 20.0,
            max_utilization_threshold: 80.0,
        }
    }
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_arena_size_mb: 32, // 32MB default
            max_concurrent_arenas: 8,  // Allow 8 concurrent file analyses
            auto_reset_between_files: true,
            size_warning_threshold: 85.0, // Warn at 85% capacity
            enable_efficiency_monitoring: true,
            pre_allocate_common_arenas: false, // Disable by default
        }
    }
}

impl Default for AstCacheOptimizationConfig {
    fn default() -> Self {
        Self {
            zero_copy_enabled: true,
            memory_mapping_enabled: true,
            memory_mapping_threshold_bytes: 10 * 1024 * 1024, // 10MB threshold
            zero_copy_cache_directory: "./cache/zero_copy".to_string(),
            max_cached_asts: 10000,
            max_ast_cache_memory_mb: 500,
            enable_ast_cache_metrics: true,
            enable_lru_eviction: true,
            zero_copy_threshold_bytes: 10 * 1024, // 10KB threshold
            enable_concurrent_access: true,
            enable_cache_warming: false,
            cache_warming_concurrency: 4,
        }
    }
}

impl Default for AiMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_context_size_bytes: 512 * 1024 * 1024, // 512MB max context
            analysis_chunk_size: 50, // Process 50 issues at a time
            streaming_context: true,
        }
    }
}

impl Default for MemoryMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_interval_ms: 1000, // Sample every second
            export_metrics: true,
        }
    }
}

impl MemoryOptimizationConfig {
    /// Create configuration optimized for large codebases
    pub fn large_codebase() -> Self {
        Self {
            target_max_memory_bytes: 6 * 1024 * 1024 * 1024, // 6GB for large codebases
            object_pools: ObjectPoolConfig {
                detector_pool_capacity: 200,
                temporary_pool_capacity: 400,
                pre_population_percentage: 30.0,
                max_pool_size: 2000,
                ..Default::default()
            },
            arena_allocation: ArenaConfig {
                default_arena_size_mb: 64, // 64MB for large codebases
                max_concurrent_arenas: 16, // More concurrent arenas
                ..Default::default()
            },
            ai_memory_optimization: AiMemoryConfig {
                max_context_size_bytes: 256 * 1024 * 1024, // Smaller context for large codebases
                analysis_chunk_size: 25, // Smaller chunks
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create configuration optimized for small projects
    pub fn small_project() -> Self {
        Self {
            target_max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB for small projects
            object_pools: ObjectPoolConfig {
                detector_pool_capacity: 50,
                temporary_pool_capacity: 100,
                pre_population_percentage: 20.0,
                max_pool_size: 500,
                ..Default::default()
            },
            arena_allocation: ArenaConfig {
                default_arena_size_mb: 16, // 16MB for small projects
                max_concurrent_arenas: 4,   // Fewer concurrent arenas
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.target_max_memory_bytes < 1024 * 1024 * 1024 {
            return Err("Target memory must be at least 1GB".to_string());
        }

        if self.object_pools.detector_pool_capacity == 0 {
            return Err("Detector pool capacity must be greater than 0".to_string());
        }

        if self.arena_allocation.default_arena_size_mb == 0 {
            return Err("Default arena size must be greater than 0 MB".to_string());
        }

        if self.arena_allocation.max_concurrent_arenas == 0 {
            return Err("Maximum concurrent arenas must be greater than 0".to_string());
        }

        if self.arena_allocation.size_warning_threshold < 50.0 || self.arena_allocation.size_warning_threshold > 100.0 {
            return Err("Arena size warning threshold must be between 50% and 100%".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MemoryOptimizationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.target_max_memory_bytes, 8 * 1024 * 1024 * 1024);
        assert!(config.object_pools.enabled);
        assert!(config.arena_allocation.enabled);
    }

    #[test]
    fn test_large_codebase_config() {
        let config = MemoryOptimizationConfig::large_codebase();
        assert_eq!(config.target_max_memory_bytes, 6 * 1024 * 1024 * 1024);
        assert_eq!(config.object_pools.detector_pool_capacity, 200);
        assert_eq!(config.arena_allocation.default_arena_size_mb, 64);
        assert_eq!(config.arena_allocation.max_concurrent_arenas, 16);
        assert_eq!(config.ai_memory_optimization.analysis_chunk_size, 25);
    }

    #[test]
    fn test_small_project_config() {
        let config = MemoryOptimizationConfig::small_project();
        assert_eq!(config.target_max_memory_bytes, 2 * 1024 * 1024 * 1024);
        assert_eq!(config.object_pools.detector_pool_capacity, 50);
        assert_eq!(config.arena_allocation.default_arena_size_mb, 16);
        assert_eq!(config.arena_allocation.max_concurrent_arenas, 4);
    }

    #[test]
    fn test_config_validation() {
        let mut config = MemoryOptimizationConfig::default();
        assert!(config.validate().is_ok());

        config.target_max_memory_bytes = 512 * 1024 * 1024; // 512MB - too small
        assert!(config.validate().is_err());

        config.target_max_memory_bytes = 2 * 1024 * 1024 * 1024; // 2GB - ok
        config.object_pools.detector_pool_capacity = 0;
        assert!(config.validate().is_err());
    }
}