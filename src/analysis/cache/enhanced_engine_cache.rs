//! Enhanced Engine Cache with Advanced Eviction Policies
//!
//! This module provides an improved cache implementation with:
//! - Configurable eviction policies (LRU, LFU, FIFO, TTL, Memory-based)
//! - Memory usage tracking and limits
//! - Better performance metrics
//! - Background maintenance tasks

use crate::analysis::cache::eviction::{EvictionManager, EvictionPolicy, EvictionStats};
#[cfg(feature = "prometheus")]
use crate::analysis::cache::metrics::CacheMetrics;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

/// Enhanced cache configuration with eviction policies
#[derive(Debug, Clone)]
/// Configuration options for enhancedcache.
pub struct EnhancedCacheConfig {
    /// AST cache configuration
    pub ast_config: CacheLayerConfig,
    /// Results cache configuration
    pub results_config: CacheLayerConfig,
    /// Background maintenance interval
    pub maintenance_interval: Duration,
    /// Enable automatic background maintenance
    pub auto_maintenance: bool,
}

/// Configuration for individual cache layers
#[derive(Debug, Clone)]
/// Configuration options for cachelayer.
pub struct CacheLayerConfig {
    /// Eviction policy to use
    pub eviction_policy: EvictionPolicy,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<usize>,
    /// Maximum number of entries
    pub max_entries: Option<usize>,
    /// TTL for entries (if using TTL policy)
    pub ttl_seconds: Option<u64>,
}

impl Default for EnhancedCacheConfig {
    fn default() -> Self {
        Self {
            ast_config: CacheLayerConfig {
                eviction_policy: EvictionPolicy::LRU,
                max_memory_bytes: Some(256 * 1024 * 1024), // 256MB
                max_entries: Some(1000),
                ttl_seconds: None,
            },
            results_config: CacheLayerConfig {
                eviction_policy: EvictionPolicy::LRU,
                max_memory_bytes: Some(128 * 1024 * 1024), // 128MB
                max_entries: Some(2000),
                ttl_seconds: Some(3600), // 1 hour
            },
            maintenance_interval: Duration::from_secs(300), // 5 minutes
            auto_maintenance: true,
        }
    }
}

/// Enhanced engine cache with advanced eviction policies
pub struct EnhancedEngineCache {
    /// AST cache storage
    ast_cache: Arc<RwLock<HashMap<String, Arc<ParsedFile>>>>,
    /// AST cache eviction manager
    ast_eviction: Arc<RwLock<EvictionManager<String>>>,
    /// Results cache storage
    results_cache: Arc<RwLock<HashMap<String, CachedResult>>>,
    /// Results cache eviction manager
    results_eviction: Arc<RwLock<EvictionManager<String>>>,
    /// Performance metrics
    #[cfg(feature = "prometheus")]
    metrics: Arc<CacheMetrics>,
    /// Configuration
    config: EnhancedCacheConfig,
    /// Background maintenance task handle
    _maintenance_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Cached result with metadata
#[derive(Debug, Clone)]
struct CachedResult {
    data: Vec<ArchitecturalIssue>,
    timestamp: Instant,
}

impl CachedResult {
    fn new(data: Vec<ArchitecturalIssue>) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
        }
    }

    fn estimated_size(&self) -> usize {
        // Rough estimation: 1KB per architectural issue
        self.data.len() * 1024
    }
}

impl EnhancedEngineCache {
    /// Create new enhanced cache with default configuration
    #[cfg(feature = "prometheus")]
    pub async fn new(
        metrics: Arc<CacheMetrics>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::new_with_config(EnhancedCacheConfig::default(), metrics).await
    }

    #[cfg(not(feature = "prometheus"))]
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::new_with_config(EnhancedCacheConfig::default()).await
    }

    /// Create new enhanced cache with custom configuration
    #[cfg(feature = "prometheus")]
    pub async fn new_with_config(
        config: EnhancedCacheConfig,
        metrics: Arc<CacheMetrics>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ast_eviction = create_eviction_manager(&config.ast_config);
        let results_eviction = create_eviction_manager(&config.results_config);

        let mut cache = Self {
            ast_cache: Arc::new(RwLock::new(HashMap::new())),
            ast_eviction: Arc::new(RwLock::new(ast_eviction)),
            results_cache: Arc::new(RwLock::new(HashMap::new())),
            results_eviction: Arc::new(RwLock::new(results_eviction)),
            metrics,
            config: config.clone(),
            _maintenance_handle: None,
        };

        // Start background maintenance task if enabled
        if config.auto_maintenance {
            cache.start_maintenance_task().await;
        }

        Ok(cache)
    }

    #[cfg(not(feature = "prometheus"))]
    pub async fn new_with_config(
        config: EnhancedCacheConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ast_eviction = create_eviction_manager(&config.ast_config);
        let results_eviction = create_eviction_manager(&config.results_config);

        let mut cache = Self {
            ast_cache: Arc::new(RwLock::new(HashMap::new())),
            ast_eviction: Arc::new(RwLock::new(ast_eviction)),
            results_cache: Arc::new(RwLock::new(HashMap::new())),
            results_eviction: Arc::new(RwLock::new(results_eviction)),
            config: config.clone(),
            _maintenance_handle: None,
        };

        // Start background maintenance task if enabled
        if config.auto_maintenance {
            cache.start_maintenance_task().await;
        }

        Ok(cache)
    }

    /// Start background maintenance task
    async fn start_maintenance_task(&mut self) {
        let ast_cache = Arc::clone(&self.ast_cache);
        let ast_eviction = Arc::clone(&self.ast_eviction);
        let results_cache = Arc::clone(&self.results_cache);
        let results_eviction = Arc::clone(&self.results_eviction);
        let maintenance_interval = self.config.maintenance_interval;

        #[cfg(feature = "prometheus")]
        let metrics = Arc::clone(&self.metrics);

        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(maintenance_interval);

            loop {
                interval_timer.tick().await;

                // Perform maintenance on both caches
                Self::perform_maintenance_static(
                    &ast_cache,
                    &ast_eviction,
                    "ast",
                    #[cfg(feature = "prometheus")]
                    &metrics,
                )
                .await;

                Self::perform_maintenance_static(
                    &results_cache,
                    &results_eviction,
                    "results",
                    #[cfg(feature = "prometheus")]
                    &metrics,
                )
                .await;

                tracing::debug!("Cache maintenance completed");
            }
        });

        self._maintenance_handle = Some(handle);
    }

    /// Get or compute AST for a file
    pub async fn get_or_parse_ast<F>(
        &self,
        file_path: &Path,
        parser: F,
    ) -> Result<Arc<ParsedFile>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Result<ParsedFile, Box<dyn std::error::Error + Send + Sync>>,
    {
        let start_time = Instant::now();
        let cache_key = file_path.to_string_lossy().to_string();

        // Try to get from cache first
        {
            let cache = self.ast_cache.read().await;
            if let Some(cached_ast) = cache.get(&cache_key) {
                // Update eviction manager
                let mut eviction = self.ast_eviction.write().await;
                eviction.record_access(&cache_key, estimate_parsed_file_size(cached_ast));
                drop(eviction);

                let duration = start_time.elapsed();
                #[cfg(feature = "prometheus")]
                self.metrics.record_hit("ast", duration);
                return Ok(cached_ast.clone());
            }
        }

        // Cache miss - parse the file
        let parsed_file = parser()?;
        let parsed_arc = Arc::new(parsed_file);
        let estimated_size = estimate_parsed_file_size(&parsed_arc);

        // Check if we need to evict before inserting
        {
            let mut eviction = self.ast_eviction.write().await;
            eviction.record_access(&cache_key, estimated_size);

            if eviction.needs_eviction() {
                let to_evict = eviction.select_for_eviction(1);
                drop(eviction);

                let mut cache = self.ast_cache.write().await;
                for key in to_evict {
                    cache.remove(&key);
                    let mut eviction = self.ast_eviction.write().await;
                    eviction.remove_entry(&key);
                    #[cfg(feature = "prometheus")]
                    self.metrics
                        .record_eviction("ast", estimated_size.try_into().unwrap_or(0));
                }
            }
        }

        // Cache the result
        {
            let mut cache = self.ast_cache.write().await;
            cache.insert(cache_key, parsed_arc.clone());
        }

        let duration = start_time.elapsed();
        #[cfg(feature = "prometheus")]
        {
            self.metrics.record_miss("ast", duration);
            self.metrics
                .record_insertion("ast", estimated_size.try_into().unwrap_or(0));
        }

        Ok(parsed_arc)
    }

    /// Get cached analysis results
    pub async fn get_cached_results(&self, file_path: &Path) -> Option<Vec<ArchitecturalIssue>> {
        let start_time = Instant::now();
        let cache_key = file_path.to_string_lossy().to_string();

        let cache = self.results_cache.read().await;
        if let Some(cached_result) = cache.get(&cache_key) {
            // Update eviction manager
            let mut eviction = self.results_eviction.write().await;
            eviction.record_access(&cache_key, cached_result.estimated_size());
            drop(eviction);

            let duration = start_time.elapsed();
            #[cfg(feature = "prometheus")]
            self.metrics.record_hit("results", duration);
            return Some(cached_result.data.clone());
        }

        let duration = start_time.elapsed();
        #[cfg(feature = "prometheus")]
        self.metrics.record_miss("results", duration);
        None
    }

    /// Cache analysis results
    pub async fn cache_results(&self, file_path: &Path, results: Vec<ArchitecturalIssue>) {
        let cache_key = file_path.to_string_lossy().to_string();
        let cached_result = CachedResult::new(results);
        let estimated_size = cached_result.estimated_size();

        // Check if we need to evict before inserting
        {
            let mut eviction = self.results_eviction.write().await;
            eviction.record_access(&cache_key, estimated_size);

            if eviction.needs_eviction() {
                let to_evict = eviction.select_for_eviction(1);
                drop(eviction);

                let mut cache = self.results_cache.write().await;
                for key in to_evict {
                    cache.remove(&key);
                    let mut eviction = self.results_eviction.write().await;
                    eviction.remove_entry(&key);
                    #[cfg(feature = "prometheus")]
                    self.metrics
                        .record_eviction("results", estimated_size.try_into().unwrap_or(0));
                }
            }
        }

        // Cache the result
        {
            let mut cache = self.results_cache.write().await;
            cache.insert(cache_key, cached_result);
        }

        #[cfg(feature = "prometheus")]
        self.metrics
            .record_insertion("results", estimated_size.try_into().unwrap_or(0));
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        {
            let mut ast_cache = self.ast_cache.write().await;
            ast_cache.clear();
        }

        {
            let mut results_cache = self.results_cache.write().await;
            results_cache.clear();
        }

        // Reset eviction managers
        {
            let mut ast_eviction = self.ast_eviction.write().await;
            *ast_eviction = create_eviction_manager(&self.config.ast_config);
        }

        {
            let mut results_eviction = self.results_eviction.write().await;
            *results_eviction = create_eviction_manager(&self.config.results_config);
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> EnhancedCacheStats {
        let ast_cache = self.ast_cache.read().await;
        let results_cache = self.results_cache.read().await;
        let ast_eviction = self.ast_eviction.read().await;
        let results_eviction = self.results_eviction.read().await;

        EnhancedCacheStats {
            ast_stats: CacheLayerStats {
                entries: ast_cache.len(),
                memory_usage_bytes: ast_eviction.current_memory_usage(),
                eviction_stats: ast_eviction.get_stats(),
            },
            results_stats: CacheLayerStats {
                entries: results_cache.len(),
                memory_usage_bytes: results_eviction.current_memory_usage(),
                eviction_stats: results_eviction.get_stats(),
            },
            #[cfg(feature = "prometheus")]
            hit_rate: self.metrics.hit_rate(),
            #[cfg(not(feature = "prometheus"))]
            hit_rate: 0.0,
        }
    }

    /// Perform manual maintenance
    pub async fn maintain(&self) {
        Self::perform_maintenance_static(
            &self.ast_cache,
            &self.ast_eviction,
            "ast",
            #[cfg(feature = "prometheus")]
            &self.metrics,
        )
        .await;

        Self::perform_maintenance_static(
            &self.results_cache,
            &self.results_eviction,
            "results",
            #[cfg(feature = "prometheus")]
            &self.metrics,
        )
        .await;
    }

    /// Static method for performing maintenance (used by background task)
    async fn perform_maintenance_static<T>(
        cache: &Arc<RwLock<HashMap<String, T>>>,
        eviction: &Arc<RwLock<EvictionManager<String>>>,
        cache_type: &str,
        #[cfg(feature = "prometheus")] metrics: &Arc<CacheMetrics>,
    ) {
        let mut eviction_guard = eviction.write().await;

        if eviction_guard.needs_eviction() {
            let to_evict = eviction_guard.select_for_eviction(10); // Evict up to 10 at once
            let evicted_count = to_evict.len();

            drop(eviction_guard);

            if evicted_count > 0 {
                let mut cache_guard = cache.write().await;
                let mut eviction_guard = eviction.write().await;

                for key in to_evict {
                    cache_guard.remove(&key);
                    if let Some(size) = eviction_guard.remove_entry(&key) {
                        #[cfg(feature = "prometheus")]
                        metrics.record_eviction(cache_type, size.try_into().unwrap_or(0));
                    }
                }

                tracing::info!(
                    "Evicted {} entries from {} cache",
                    evicted_count,
                    cache_type
                );
            }
        }
    }
}

/// Statistics for individual cache layers
#[derive(Debug, Clone)]
/// Represents cache layer stats in the system.
pub struct CacheLayerStats {
    pub entries: usize,
    pub memory_usage_bytes: usize,
    pub eviction_stats: EvictionStats,
}

/// Enhanced cache statistics
#[derive(Debug, Clone)]
/// Represents enhanced cache stats in the system.
pub struct EnhancedCacheStats {
    pub ast_stats: CacheLayerStats,
    pub results_stats: CacheLayerStats,
    pub hit_rate: f64,
}

/// Create eviction manager from configuration
fn create_eviction_manager(config: &CacheLayerConfig) -> EvictionManager<String> {
    let mut manager = EvictionManager::new(config.eviction_policy);

    if let Some(max_memory) = config.max_memory_bytes {
        manager = manager.with_max_memory(max_memory);
    }

    if let Some(max_entries) = config.max_entries {
        manager = manager.with_max_entries(max_entries);
    }

    if let Some(ttl_seconds) = config.ttl_seconds {
        manager = manager.with_ttl(ttl_seconds);
    }

    manager
}

/// Estimate memory usage of a ParsedFile
fn estimate_parsed_file_size(parsed_file: &ParsedFile) -> usize {
    // Rough estimation based on source length and tree size
    let base_size = parsed_file.source.len();
    let tree_size = if parsed_file.tree.is_some() {
        base_size * 2 // Tree is roughly 2x source size
    } else {
        0
    };
    base_size + tree_size + 1024 // Add overhead
}
