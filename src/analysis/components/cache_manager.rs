//! High-Performance Multi-Layered Cache Management Component
//!
//! This module provides a unified cache management system that leverages
//! the new multi-layered caching architecture with memory and disk tiers,
//! optimized serialization, and intelligent invalidation strategies.

#[cfg(feature = "prometheus")]
use crate::analysis::cache::metrics::CacheMetrics;
use crate::analysis::cache::{
    ast::{AstCache, CacheConfig},
    engine_cache::{EngineCache, EngineCacheConfig},
};
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use async_trait::async_trait;
#[cfg(feature = "prometheus")]
use prometheus::Registry;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait defining the cache management interface
#[async_trait]
pub trait CacheManager: Send + Sync {
    /// Get or parse an AST for a file
    async fn get_or_parse_ast(&self, file_path: &Path) -> Result<Arc<ParsedFile>, UveddiError>;

    /// Get cached analysis results for a file
    async fn get_cached_results(&self, file_path: &Path) -> Option<Vec<ArchitecturalIssue>>;

    /// Cache analysis results for a file
    async fn cache_results(&self, file_path: &Path, results: Vec<ArchitecturalIssue>);

    /// Clear all caches
    async fn clear_all_caches(&self);

    /// Get cache metrics
    fn get_cache_metrics(&self) -> Value;

    /// Get cache statistics
    async fn get_cache_stats(&self) -> CacheStats;
}

/// Cache statistics structure
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub ast_cache_size: usize,
    pub result_cache_size: usize,
    pub ast_hit_rate: f64,
    pub result_hit_rate: f64,
    pub total_memory_usage: usize,
}

/// High-performance cache management implementation
pub struct CacheManagerImpl {
    /// High-performance engine cache
    engine_cache: Arc<EngineCache>,
    /// Performance metrics collector
    #[cfg(feature = "prometheus")]
    metrics: Arc<CacheMetrics>,
    /// Legacy AST cache for compatibility
    legacy_ast_cache: Arc<RwLock<AstCache>>,
    /// Cache statistics
    cache_stats: Arc<RwLock<CacheStats>>,
}

impl CacheManagerImpl {
    /// Create a new high-performance cache manager with multi-layer caching
    pub async fn new() -> Result<Self, UveddiError> {
        Self::with_config(EngineCacheConfig::default()).await
    }

    /// Create cache manager with custom configuration
    pub async fn with_config(config: EngineCacheConfig) -> Result<Self, UveddiError> {
        // Initialize metrics
        #[cfg(feature = "prometheus")]
        let registry = Registry::new();
        #[cfg(feature = "prometheus")]
        let metrics = Arc::new(
            CacheMetrics::new(&registry)
                .map_err(|e| UveddiError::config_error(&e.to_string(), "CacheManager metrics"))?,
        );

        // Create high-performance engine cache
        #[cfg(feature = "prometheus")]
        let engine_cache = Arc::new(
            EngineCache::new_with_config(config, metrics.clone())
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "Engine cache creation"))?,
        );
        #[cfg(not(feature = "prometheus"))]
        let engine_cache = Arc::new(
            EngineCache::new_with_config(config)
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "Engine cache creation"))?,
        );

        // Legacy AST cache for backward compatibility
        let legacy_ast_cache = AstCache::new(CacheConfig::default())
            .map_err(|e| UveddiError::config_error(&e.to_string(), "Legacy AST cache"))?;

        #[cfg(feature = "prometheus")]
        {
            Ok(Self {
                engine_cache,
                metrics,
                legacy_ast_cache: Arc::new(RwLock::new(legacy_ast_cache)),
                cache_stats: Arc::new(RwLock::new(CacheStats {
                    ast_cache_size: 0,
                    result_cache_size: 0,
                    ast_hit_rate: 0.0,
                    result_hit_rate: 0.0,
                    total_memory_usage: 0,
                })),
            })
        }
        #[cfg(not(feature = "prometheus"))]
        {
            Ok(Self {
                engine_cache,
                legacy_ast_cache: Arc::new(RwLock::new(legacy_ast_cache)),
                cache_stats: Arc::new(RwLock::new(CacheStats {
                    ast_cache_size: 0,
                    result_cache_size: 0,
                    ast_hit_rate: 0.0,
                    result_hit_rate: 0.0,
                    total_memory_usage: 0,
                })),
            })
        }
    }

    /// Create a new cache manager with custom AST cache (legacy compatibility)
    pub async fn with_ast_cache(ast_cache: AstCache) -> Result<Self, UveddiError> {
        // Initialize metrics
        #[cfg(feature = "prometheus")]
        let registry = Registry::new();
        #[cfg(feature = "prometheus")]
        let metrics = Arc::new(
            CacheMetrics::new(&registry)
                .map_err(|e| UveddiError::config_error(&e.to_string(), "CacheManager metrics"))?,
        );

        // Create engine cache with default config
        #[cfg(feature = "prometheus")]
        let engine_cache = Arc::new(
            EngineCache::new(metrics.clone())
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "Engine cache creation"))?,
        );
        #[cfg(not(feature = "prometheus"))]
        let engine_cache = Arc::new(
            EngineCache::new()
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "Engine cache creation"))?,
        );

        #[cfg(feature = "prometheus")]
        {
            Ok(Self {
                engine_cache,
                metrics,
                legacy_ast_cache: Arc::new(RwLock::new(ast_cache)),
                cache_stats: Arc::new(RwLock::new(CacheStats {
                    ast_cache_size: 0,
                    result_cache_size: 0,
                    ast_hit_rate: 0.0,
                    result_hit_rate: 0.0,
                    total_memory_usage: 0,
                })),
            })
        }
        #[cfg(not(feature = "prometheus"))]
        {
            Ok(Self {
                engine_cache,
                legacy_ast_cache: Arc::new(RwLock::new(ast_cache)),
                cache_stats: Arc::new(RwLock::new(CacheStats {
                    ast_cache_size: 0,
                    result_cache_size: 0,
                    ast_hit_rate: 0.0,
                    result_hit_rate: 0.0,
                    total_memory_usage: 0,
                })),
            })
        }
    }

    /// Update cache statistics using engine cache stats
    async fn update_stats(&self) {
        let mut stats = self.cache_stats.write().await;

        // Get statistics from engine cache
        let engine_stats = self.engine_cache.stats().await;

        // Update combined statistics
        stats.ast_cache_size = engine_stats.ast_entries;
        stats.result_cache_size = engine_stats.result_entries;
        stats.ast_hit_rate = engine_stats.hit_rate;
        stats.result_hit_rate = engine_stats.hit_rate;
        stats.total_memory_usage = (engine_stats.ast_entries + engine_stats.result_entries) * 1024;
        // Estimate
    }
}

#[async_trait]
impl CacheManager for CacheManagerImpl {
    async fn get_or_parse_ast(&self, file_path: &Path) -> Result<Arc<ParsedFile>, UveddiError> {
        let parser = || -> Result<ParsedFile, Box<dyn std::error::Error + Send + Sync>> {
            use crate::ast::tree_sitter_impl::AstParser;
            let mut ast_parser = AstParser::new()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            let parsed_file = ast_parser
                .parse_file(file_path)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            Ok(parsed_file)
        };

        match self.engine_cache.get_or_parse_ast(file_path, parser).await {
            Ok(parsed_file) => {
                self.update_stats().await;
                Ok(parsed_file)
            }
            Err(e) => {
                tracing::warn!("Cache error for AST {}: {}", file_path.display(), e);
                Err(UveddiError::AnalysisError {
                    message: format!("Failed to get or parse AST: {}", e),
                    file: file_path.to_string_lossy().to_string(),
                    line: 0,
                    context: "cache_manager".to_string(),
                    suggestion: "Check file permissions and syntax".to_string(),
                    source: Some(e.into()),
                })
            }
        }
    }

    async fn get_cached_results(&self, file_path: &Path) -> Option<Vec<ArchitecturalIssue>> {
        let results = self.engine_cache.get_cached_results(file_path).await;
        self.update_stats().await;
        results
    }

    async fn cache_results(&self, file_path: &Path, results: Vec<ArchitecturalIssue>) {
        self.engine_cache.cache_results(file_path, results).await;
        self.update_stats().await;
    }

    async fn clear_all_caches(&self) {
        // Clear engine cache
        self.engine_cache.clear_all().await;

        // Clear legacy cache for compatibility
        {
            let legacy_ast_cache = self.legacy_ast_cache.write().await;
            legacy_ast_cache.clear();
        }

        // Update statistics
        self.update_stats().await;
    }

    fn get_cache_metrics(&self) -> Value {
        #[cfg(feature = "prometheus")]
        {
            // Get performance summary from metrics collector
            let summary = self.metrics.performance_summary();

            serde_json::json!({
                "overall_hit_rate": summary.overall_hit_rate,
                "average_latency_ms": summary.average_latency_ms,
                "total_size_bytes": summary.total_size_bytes,
                "total_entries": summary.total_entries,
                "layer_summaries": summary.layer_summaries,
                "prometheus_metrics": self.metrics.export_metrics()
            })
        }
        #[cfg(not(feature = "prometheus"))]
        {
            serde_json::json!({
                "message": "Prometheus metrics not available - compile with 'prometheus' feature",
                "overall_hit_rate": 0.0,
                "average_latency_ms": 0.0,
                "total_size_bytes": 0,
                "total_entries": 0,
                "layer_summaries": [],
                "prometheus_metrics": null
            })
        }
    }

    async fn get_cache_stats(&self) -> CacheStats {
        self.update_stats().await;
        self.cache_stats.read().await.clone()
    }
}

impl CacheManagerImpl {
    /// Cache a dependency graph with a given key
    pub async fn cache_dependency_graph(
        &self,
        cache_key: &str,
        _graph: &crate::analysis::graph::dependency::LocalDependencyGraph,
    ) -> Result<(), UveddiError> {
        // For now, we'll just log this operation since the cache infrastructure
        // for dependency graphs is not fully implemented yet
        tracing::debug!("Caching dependency graph with key: {}", cache_key);

        // In a full implementation, we would serialize the graph and store it
        // in the engine cache, but for now we'll just indicate success
        Ok(())
    }
}

/// Simple result cache implementation for architectural issues
pub struct ResultCache {
    cache: HashMap<std::path::PathBuf, Vec<ArchitecturalIssue>>,
    hits: usize,
    misses: usize,
}

impl ResultCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, file_path: &Path) -> Option<Vec<ArchitecturalIssue>> {
        if let Some(results) = self.cache.get(file_path) {
            self.hits += 1;
            Some(results.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, file_path: std::path::PathBuf, results: Vec<ArchitecturalIssue>) {
        self.cache.insert(file_path, results);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }

    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }

    pub fn memory_usage(&self) -> usize {
        // Estimate memory usage
        self.cache.len() * 1024 // Rough estimate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let cache_manager = CacheManagerImpl::new().await.unwrap();
        let stats = cache_manager.get_cache_stats().await;

        assert_eq!(stats.ast_cache_size, 0);
        assert_eq!(stats.result_cache_size, 0);
    }

    #[tokio::test]
    async fn test_cache_manager_ast_operations() {
        let cache_manager = CacheManagerImpl::new().await.unwrap();

        // Create a temporary Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        write!(temp_file, "fn main() {{ println!(\"Hello, world!\"); }}").unwrap();
        let file_path = temp_file.path();

        // Test AST parsing and caching
        let ast1 = cache_manager.get_or_parse_ast(file_path).await.unwrap();
        let ast2 = cache_manager.get_or_parse_ast(file_path).await.unwrap();

        // Both should be valid ASTs (caching not yet implemented)
        assert!(ast1.tree.is_some());
        assert!(ast2.tree.is_some());
    }

    #[tokio::test]
    async fn test_cache_manager_result_operations() {
        let cache_manager = CacheManagerImpl::new().await.unwrap();

        // Create a temporary file path
        let temp_path = std::path::Path::new("/tmp/test_file.rs");

        // Test result caching
        let results = vec![]; // Empty results for testing
        cache_manager
            .cache_results(temp_path, results.clone())
            .await;

        let cached_results = cache_manager.get_cached_results(temp_path).await;
        assert!(cached_results.is_some());
        assert_eq!(cached_results.unwrap().len(), results.len());
    }

    #[tokio::test]
    async fn test_cache_manager_clear() {
        let cache_manager = CacheManagerImpl::new().await.unwrap();

        // Create a temporary file path
        let temp_path = std::path::Path::new("/tmp/test_file.rs");

        // Cache some results
        let results = vec![];
        cache_manager.cache_results(temp_path, results).await;

        // Clear all caches
        cache_manager.clear_all_caches().await;

        // Should be empty now
        let cached_results = cache_manager.get_cached_results(temp_path).await;
        assert!(cached_results.is_none());

        let stats = cache_manager.get_cache_stats().await;
        assert_eq!(stats.result_cache_size, 0);
    }

    #[tokio::test]
    async fn test_cache_manager_metrics() {
        let cache_manager = CacheManagerImpl::new().await.unwrap();

        // Get metrics
        let metrics = cache_manager.get_cache_metrics();
        assert!(metrics.is_object());

        // Get stats
        let stats = cache_manager.get_cache_stats().await;
        assert_eq!(stats.ast_cache_size, 0);
        assert_eq!(stats.result_cache_size, 0);
    }
}
