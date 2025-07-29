//! Simplified cache implementation for the analysis engine
//!
//! This provides a high-performance cache specifically tailored for the
//! analysis engine's needs, avoiding complex serialization issues.

use crate::analysis::cache::metrics::CacheMetrics;
use crate::analysis::cache::wrappers::ArchivableSystemTime;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::ArchitecturalIssue;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// High-performance engine-specific cache
pub struct EngineCache {
    /// AST cache with LRU eviction
    ast_cache: Arc<RwLock<LruCache<String, Arc<ParsedFile>>>>,
    /// Analysis results cache
    result_cache: Arc<RwLock<HashMap<String, CachedResult>>>,
    /// Performance metrics
    metrics: Arc<CacheMetrics>,
    /// Configuration
    config: EngineCacheConfig,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct EngineCacheConfig {
    pub ast_capacity: usize,
    pub result_capacity: usize,
    pub result_ttl: Duration,
}

impl Default for EngineCacheConfig {
    fn default() -> Self {
        Self {
            ast_capacity: 1000,
            result_capacity: 2000,
            result_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Cached result with metadata
#[derive(Debug, Clone)]
struct CachedResult {
    data: Vec<ArchitecturalIssue>,
    timestamp: Instant,
    access_count: u64,
}

impl CachedResult {
    fn new(data: Vec<ArchitecturalIssue>) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            access_count: 0,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }

    fn touch(&mut self) {
        self.access_count += 1;
        self.timestamp = Instant::now();
    }
}

impl EngineCache {
    /// Create new engine cache with default configuration
    pub async fn new(
        metrics: Arc<CacheMetrics>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::new_with_config(EngineCacheConfig::default(), metrics).await
    }

    /// Create new engine cache with custom configuration
    pub async fn new_with_config(
        config: EngineCacheConfig,
        metrics: Arc<CacheMetrics>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ast_capacity =
            NonZeroUsize::new(config.ast_capacity).ok_or("AST cache capacity must be > 0")?;

        let ast_cache = Arc::new(RwLock::new(LruCache::new(ast_capacity)));
        let result_cache = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            ast_cache,
            result_cache,
            metrics,
            config,
        })
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
            let mut cache = self.ast_cache.write().await;
            if let Some(cached_ast) = cache.get(&cache_key) {
                let duration = start_time.elapsed();
                self.metrics.record_hit("ast", duration);
                return Ok(cached_ast.clone());
            }
        }

        // Cache miss - parse the file
        let parsed_file = parser()?;
        let parsed_arc = Arc::new(parsed_file);

        // Cache the result
        {
            let mut cache = self.ast_cache.write().await;
            cache.put(cache_key, parsed_arc.clone());
        }

        let duration = start_time.elapsed();
        self.metrics.record_miss("ast", duration);
        self.metrics.record_insertion("ast", 1024); // Estimated size

        Ok(parsed_arc)
    }

    /// Get cached analysis results
    pub async fn get_cached_results(&self, file_path: &Path) -> Option<Vec<ArchitecturalIssue>> {
        let start_time = Instant::now();
        let cache_key = file_path.to_string_lossy().to_string();

        let mut cache = self.result_cache.write().await;

        // Clean up expired entries while we're here
        let ttl = self.config.result_ttl;
        cache.retain(|_, result| !result.is_expired(ttl));

        if let Some(cached_result) = cache.get_mut(&cache_key) {
            if !cached_result.is_expired(ttl) {
                cached_result.touch();
                let duration = start_time.elapsed();
                self.metrics.record_hit("results", duration);
                return Some(cached_result.data.clone());
            } else {
                // Remove expired entry
                cache.remove(&cache_key);
            }
        }

        let duration = start_time.elapsed();
        self.metrics.record_miss("results", duration);
        None
    }

    /// Cache analysis results
    pub async fn cache_results(&self, file_path: &Path, results: Vec<ArchitecturalIssue>) {
        let cache_key = file_path.to_string_lossy().to_string();
        let cached_result = CachedResult::new(results);

        let mut cache = self.result_cache.write().await;

        // Enforce capacity limits
        while cache.len() >= self.config.result_capacity {
            // Find and remove the least recently used entry
            if let Some((lru_key, _)) = cache
                .iter()
                .min_by_key(|(_, result)| result.timestamp)
                .map(|(key, result)| (key.clone(), result.timestamp))
            {
                cache.remove(&lru_key);
                self.metrics.record_eviction("results", 1024);
            } else {
                break;
            }
        }

        cache.insert(cache_key, cached_result);
        self.metrics.record_insertion("results", 1024);
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        {
            let mut ast_cache = self.ast_cache.write().await;
            ast_cache.clear();
        }

        {
            let mut result_cache = self.result_cache.write().await;
            result_cache.clear();
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> EngineCacheStats {
        let ast_cache = self.ast_cache.read().await;
        let result_cache = self.result_cache.read().await;

        EngineCacheStats {
            ast_entries: ast_cache.len(),
            ast_capacity: self.config.ast_capacity,
            result_entries: result_cache.len(),
            result_capacity: self.config.result_capacity,
            hit_rate: self.metrics.hit_rate(),
        }
    }

    /// Perform maintenance (cleanup expired entries)
    pub async fn maintain(&self) {
        let mut result_cache = self.result_cache.write().await;
        let ttl = self.config.result_ttl;
        let before_count = result_cache.len();

        result_cache.retain(|_, result| !result.is_expired(ttl));

        let cleaned_count = before_count - result_cache.len();
        if cleaned_count > 0 {
            log::info!("Cleaned up {} expired cache entries", cleaned_count);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct EngineCacheStats {
    pub ast_entries: usize,
    pub ast_capacity: usize,
    pub result_entries: usize,
    pub result_capacity: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
    use prometheus::Registry;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_parsed_file() -> ParsedFile {
        ParsedFile {
            file_path: Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            tree: None,
            source: Arc::new("fn main() {}".to_string()),
            custom_ast: Arc::new(None),
            modified_at: ArchivableSystemTime(std::time::SystemTime::now()),
        }
    }

    #[tokio::test]
    async fn test_engine_cache_ast_operations() {
        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache = EngineCache::new(metrics).await.unwrap();

        let test_path = std::path::Path::new("test.rs");
        let parser = || Ok(create_test_parsed_file());

        // First call should miss and parse
        let result1 = cache.get_or_parse_ast(test_path, parser).await.unwrap();
        assert_eq!(result1.language, SourceLanguage::Rust);

        // Second call should hit cache
        let parser2 = || panic!("Should not be called - cache hit expected");
        let result2 = cache.get_or_parse_ast(test_path, parser2).await.unwrap();
        assert_eq!(result2.language, SourceLanguage::Rust);

        // Results should be the same Arc
        assert!(Arc::ptr_eq(&result1, &result2));
    }

    #[tokio::test]
    async fn test_engine_cache_result_operations() {
        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache = EngineCache::new(metrics).await.unwrap();

        let test_path = std::path::Path::new("test.rs");
        let test_results = vec![];

        // Cache some results
        cache.cache_results(test_path, test_results.clone()).await;

        // Retrieve cached results
        let cached = cache.get_cached_results(test_path).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), test_results.len());

        // Clear cache
        cache.clear_all().await;

        // Should be empty now
        let empty = cache.get_cached_results(test_path).await;
        assert!(empty.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache = EngineCache::new(metrics).await.unwrap();

        let stats = cache.stats().await;
        assert_eq!(stats.ast_entries, 0);
        assert_eq!(stats.result_entries, 0);

        // Add some entries
        let test_path = std::path::Path::new("test.rs");
        cache.cache_results(test_path, vec![]).await;

        let stats = cache.stats().await;
        assert_eq!(stats.result_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = EngineCacheConfig {
            ast_capacity: 100,
            result_capacity: 100,
            result_ttl: Duration::from_millis(10), // Very short TTL
        };

        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache = EngineCache::new_with_config(config, metrics).await.unwrap();

        let test_path = std::path::Path::new("test.rs");
        cache.cache_results(test_path, vec![]).await;

        // Should be cached
        assert!(cache.get_cached_results(test_path).await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Should be expired
        assert!(cache.get_cached_results(test_path).await.is_none());
    }
}
