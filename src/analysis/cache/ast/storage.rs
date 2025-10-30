use super::config::CacheConfig;
use super::metrics::CacheMetrics;
use crate::analysis::errors::AnalysisError;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, SystemTime};
use tracing::{debug, error, info, warn};

#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;

#[cfg(feature = "memory-optimization")]
use crate::analysis::memory::zero_copy::ZeroCopyAstCache;

/// Cached AST entry with metadata
#[derive(Debug, Clone)]
/// Data structure for cachedast.
pub struct CachedAST {
    #[cfg(feature = "tree-sitter")]
    pub ast: Option<Arc<Tree>>,
    #[cfg(not(feature = "tree-sitter"))]
    pub ast: Option<Arc<CacheableAst>>,
    pub file_hash: String,
    pub last_modified: SystemTime,
    pub access_count: u64,
    pub last_accessed: Instant,
    pub memory_size_bytes: usize,
    pub language: String,
    pub is_memory_mapped: bool,
}

/// Fallback AST representation when tree-sitter is disabled
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for cacheableast.
pub struct CacheableAst {
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
    pub language: String,
}

/// Thread-safe AST cache with LRU eviction and performance metrics
pub struct AstCache {
    pub(crate) cache: Arc<RwLock<HashMap<PathBuf, CachedAST>>>,
    pub(crate) lru_order: Arc<Mutex<Vec<PathBuf>>>,
    pub(crate) config: CacheConfig,
    pub(crate) metrics: Arc<Mutex<CacheMetrics>>,
    pub(crate) memory_usage: Arc<Mutex<usize>>,
    #[cfg(feature = "memory-optimization")]
    pub(crate) zero_copy_cache: Option<ZeroCopyAstCache>,
}

impl AstCache {
    /// Creates a new AST cache with the given configuration
    pub fn new(config: CacheConfig) -> Result<Self, std::io::Error> {
        if config.enable_disk_cache {
            std::fs::create_dir_all(&config.disk_cache_path)?;
        }

        info!(
            "Initializing AST cache with config: max_entries={}, max_memory={}MB, disk_cache={}",
            config.max_memory_entries, config.max_memory_size_mb, config.enable_disk_cache
        );

        #[cfg(feature = "memory-optimization")]
        let zero_copy_cache = if config.enable_zero_copy {
            match ZeroCopyAstCache::new(config.zero_copy_cache_dir.clone()) {
                Ok(cache) => {
                    info!("Zero-copy AST cache enabled");
                    Some(cache)
                }
                Err(e) => {
                    warn!("Failed to initialize zero-copy cache: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            lru_order: Arc::new(Mutex::new(Vec::new())),
            config,
            metrics: Arc::new(Mutex::new(CacheMetrics::new())),
            memory_usage: Arc::new(Mutex::new(0)),
            #[cfg(feature = "memory-optimization")]
            zero_copy_cache,
        })
    }

    /// Creates a new AST cache with default configuration for backward compatibility
    pub fn with_capacity(capacity: usize, cache_dir: PathBuf) -> Result<Self, std::io::Error> {
        let mut config = CacheConfig {
            max_memory_entries: capacity,
            disk_cache_path: cache_dir.clone(),
            ..Default::default()
        };

        #[cfg(feature = "memory-optimization")]
        {
            config.zero_copy_cache_dir = cache_dir.join("zero_copy");
        }

        Self::new(config)
    }

    pub(crate) fn safe_cache_read(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<HashMap<PathBuf, CachedAST>>, AnalysisError> {
        self.cache
            .read()
            .map_err(|_| AnalysisError::lock_error("Cache read lock poisoned"))
    }

    pub(crate) fn safe_cache_write(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<HashMap<PathBuf, CachedAST>>, AnalysisError> {
        self.cache
            .write()
            .map_err(|_| AnalysisError::lock_error("Cache write lock poisoned"))
    }

    pub(crate) fn safe_metrics_lock(&self) -> Result<std::sync::MutexGuard<CacheMetrics>, AnalysisError> {
        self.metrics
            .lock()
            .map_err(|_| AnalysisError::lock_error("Metrics mutex poisoned"))
    }

    pub(crate) fn safe_memory_usage_lock(&self) -> Result<std::sync::MutexGuard<usize>, AnalysisError> {
        self.memory_usage
            .lock()
            .map_err(|_| AnalysisError::lock_error("Memory usage mutex poisoned"))
    }

    pub(crate) fn safe_lru_order_lock(&self) -> Result<std::sync::MutexGuard<Vec<PathBuf>>, AnalysisError> {
        self.lru_order
            .lock()
            .map_err(|_| AnalysisError::lock_error("LRU order mutex poisoned"))
    }

    #[cfg(feature = "tree-sitter")]
    #[inline]
    /// Performs get operation.
    pub fn get(&self, path: &Path) -> Option<Arc<Tree>> {
        let start_time = Instant::now();
        if self.update_request_count().is_err() {
            return None;
        }

        let file_modified = match fs::metadata(path).and_then(|m| m.modified()) {
            Ok(modified) => modified,
            Err(_) => {
                debug!("Failed to get file metadata for: {:?}", path);
                self.record_miss();
                return None;
            }
        };

        let cache_result = {
            let Ok(cache) = self.safe_cache_read() else {
                error!("Failed to acquire cache read lock for: {:?}", path);
                return None;
            };
            cache.get(path).cloned()
        };

        if let Some(mut cached_ast) = cache_result {
            if file_modified <= cached_ast.last_modified {
                if let Ok(current_hash) = self.calculate_file_hash(path) {
                    if current_hash != cached_ast.file_hash {
                        self.invalidate_entry(path);
                        self.record_miss();
                        debug!("Cache invalidated due to hash mismatch for: {:?}", path);
                        return None;
                    }
                }

                cached_ast.access_count += 1;
                cached_ast.last_accessed = Instant::now();

                if let Ok(mut cache) = self.safe_cache_write() {
                    cache.insert(path.to_path_buf(), cached_ast.clone());
                }

                self.update_lru_order(path);
                self.record_hit(start_time.elapsed().as_millis() as f64);
                debug!("Cache hit for: {:?}", path);
                return cached_ast.ast;
            } else {
                self.invalidate_entry(path);
            }
        }

        self.record_miss();
        debug!("Cache miss for: {:?}", path);
        None
    }

    #[cfg(not(feature = "tree-sitter"))]
    /// Performs get operation.
    pub fn get(&self, path: &Path) -> Option<Arc<CacheableAst>> {
        let start_time = Instant::now();
        if self.update_request_count().is_err() {
            return None;
        }

        let file_modified = match fs::metadata(path).and_then(|m| m.modified()) {
            Ok(modified) => modified,
            Err(_) => {
                debug!("Failed to get file metadata for: {:?}", path);
                self.record_miss();
                return None;
            }
        };

        let cache_result = {
            let Ok(cache) = self.safe_cache_read() else {
                error!("Failed to acquire cache read lock for: {:?}", path);
                return None;
            };
            cache.get(path).cloned()
        };

        if let Some(mut cached_ast) = cache_result {
            if file_modified <= cached_ast.last_modified {
                cached_ast.access_count += 1;
                cached_ast.last_accessed = Instant::now();

                if let Ok(mut cache) = self.safe_cache_write() {
                    cache.insert(path.to_path_buf(), cached_ast.clone());
                }

                self.update_lru_order(path);
                self.record_hit(start_time.elapsed().as_millis() as f64);
                debug!("Cache hit for: {:?}", path);
                return cached_ast.ast;
            } else {
                self.invalidate_entry(path);
            }
        }

        self.record_miss();
        debug!("Cache miss for: {:?}", path);
        None
    }

    /// Performs size operation.
    pub fn size(&self) -> usize {
        self.cache.read().map(|cache| cache.len()).unwrap_or(0)
    }

    /// Performs memory usage operation.
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
            .lock()
            .map(|usage| *usage)
            .unwrap_or(0)
    }

    pub(crate) fn update_request_count(&self) -> Result<(), AnalysisError> {
        let mut metrics = self.safe_metrics_lock()?;
        metrics.total_requests += 1;
        Ok(())
    }

    pub(crate) fn record_hit(&self, lookup_time_ms: f64) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.cache_hits += 1;
            metrics.average_lookup_time_ms =
                (metrics.average_lookup_time_ms + lookup_time_ms) / 2.0;
            metrics.update_hit_rate();
        }
    }

    pub(crate) fn record_miss(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.cache_misses += 1;
            metrics.update_hit_rate();
        }
    }

    pub(crate) fn update_memory_usage(&self, delta: isize) {
        if let Ok(mut memory_usage) = self.memory_usage.lock() {
            if delta.is_positive() {
                *memory_usage += delta as usize;
            } else {
                *memory_usage = memory_usage.saturating_sub(delta.unsigned_abs());
            }
        }
    }

    fn calculate_file_hash(&self, path: &Path) -> Result<String, std::io::Error> {
        let content = fs::read(path)?;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }
}

impl Clone for AstCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            lru_order: Arc::clone(&self.lru_order),
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
            memory_usage: Arc::clone(&self.memory_usage),
            #[cfg(feature = "memory-optimization")]
            zero_copy_cache: self.zero_copy_cache.clone(),
        }
    }
}
