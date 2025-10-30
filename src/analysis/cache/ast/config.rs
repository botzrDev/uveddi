use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the AST cache system
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for cache.
pub struct CacheConfig {
    /// Maximum number of entries to keep in memory
    pub max_memory_entries: usize,
    /// Maximum memory usage in megabytes
    pub max_memory_size_mb: usize,
    /// Whether to enable disk-based caching
    pub enable_disk_cache: bool,
    /// Path for disk cache storage
    pub disk_cache_path: PathBuf,
    /// Whether to enable memory mapping for large ASTs
    pub enable_memory_mapping: bool,
    /// Whether to use LRU eviction policy
    pub lru_eviction_enabled: bool,
    /// Whether to collect performance metrics
    pub cache_metrics_enabled: bool,

    /// Whether zero-copy caching is enabled
    #[cfg(feature = "memory-optimization")]
    pub enable_zero_copy: bool,

    /// Directory for zero-copy cache files
    #[cfg(feature = "memory-optimization")]
    pub zero_copy_cache_dir: PathBuf,

    /// Threshold for using zero-copy cache (file size in bytes)
    #[cfg(feature = "memory-optimization")]
    pub zero_copy_threshold_bytes: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_entries: 10000,
            max_memory_size_mb: 500,
            enable_disk_cache: true,
            disk_cache_path: PathBuf::from("./cache/ast"),
            enable_memory_mapping: true,
            lru_eviction_enabled: true,
            cache_metrics_enabled: true,

            #[cfg(feature = "memory-optimization")]
            enable_zero_copy: true,
            #[cfg(feature = "memory-optimization")]
            zero_copy_cache_dir: PathBuf::from("./cache/zero_copy"),
            #[cfg(feature = "memory-optimization")]
            zero_copy_threshold_bytes: 10 * 1024, // 10KB threshold
        }
    }
}
