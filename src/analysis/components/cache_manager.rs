//! Centralized Cache Management Component
//!
//! This module provides a unified cache management system that consolidates
//! AST caching, result caching, and cache eviction policies into a single
//! component as required by the facade pattern architecture.

use crate::analysis::cache::ast::{AstCache, CacheConfig};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait defining the cache management interface
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

/// Centralized cache management implementation
pub struct CacheManagerImpl {
    ast_cache: Arc<RwLock<AstCache>>,
    result_cache: Arc<RwLock<ResultCache>>,
    cache_stats: Arc<RwLock<CacheStats>>,
}

impl CacheManagerImpl {
    /// Create a new cache manager with default configuration
    pub fn new() -> Result<Self, UveddiError> {
        let ast_cache = AstCache::new(CacheConfig::default())
            .map_err(|e| UveddiError::config_error(&e.to_string(), "CacheManager::new"))?;
        
        Ok(Self {
            ast_cache: Arc::new(RwLock::new(ast_cache)),
            result_cache: Arc::new(RwLock::new(ResultCache::new())),
            cache_stats: Arc::new(RwLock::new(CacheStats {
                ast_cache_size: 0,
                result_cache_size: 0,
                ast_hit_rate: 0.0,
                result_hit_rate: 0.0,
                total_memory_usage: 0,
            })),
        })
    }
    
    /// Create a new cache manager with custom AST cache
    pub fn with_ast_cache(ast_cache: AstCache) -> Self {
        Self {
            ast_cache: Arc::new(RwLock::new(ast_cache)),
            result_cache: Arc::new(RwLock::new(ResultCache::new())),
            cache_stats: Arc::new(RwLock::new(CacheStats {
                ast_cache_size: 0,
                result_cache_size: 0,
                ast_hit_rate: 0.0,
                result_hit_rate: 0.0,
                total_memory_usage: 0,
            })),
        }
    }
    
    /// Update cache statistics
    async fn update_stats(&self) {
        let mut stats = self.cache_stats.write().await;
        let ast_cache = self.ast_cache.read().await;
        let result_cache = self.result_cache.read().await;
        
        // Use basic metrics since hit_rate and memory_usage don't exist on AstCache
        stats.ast_cache_size = 0; // Will be updated when we implement metrics
        stats.result_cache_size = result_cache.size();
        stats.ast_hit_rate = 0.0; // Will be updated when we implement metrics
        stats.result_hit_rate = result_cache.hit_rate();
        stats.total_memory_usage = result_cache.memory_usage();
    }
}

impl CacheManager for CacheManagerImpl {
    async fn get_or_parse_ast(&self, file_path: &Path) -> Result<Arc<ParsedFile>, UveddiError> {
        // For now, delegate to the AstProvider pattern until we can integrate properly
        // This is a placeholder implementation
        use crate::ast::tree_sitter_impl::AstParser;
        let mut ast_parser = AstParser::new()?;
        let parsed_file = ast_parser.parse_file(file_path)?;
        
        // Update statistics
        self.update_stats().await;
        
        Ok(Arc::new(parsed_file))
    }
    
    async fn get_cached_results(&self, file_path: &Path) -> Option<Vec<ArchitecturalIssue>> {
        let mut result_cache = self.result_cache.write().await;
        let result = result_cache.get(file_path);
        
        // Update statistics
        self.update_stats().await;
        
        result
    }
    
    async fn cache_results(&self, file_path: &Path, results: Vec<ArchitecturalIssue>) {
        let mut result_cache = self.result_cache.write().await;
        result_cache.insert(file_path.to_path_buf(), results);
        
        // Update statistics
        self.update_stats().await;
    }
    
    async fn clear_all_caches(&self) {
        let mut ast_cache = self.ast_cache.write().await;
        let mut result_cache = self.result_cache.write().await;
        
        ast_cache.clear();
        result_cache.clear();
        
        // Update statistics
        self.update_stats().await;
    }
    
    fn get_cache_metrics(&self) -> Value {
        // Return combined metrics from both caches
        let ast_cache_metrics = {
            // We need to handle this synchronously for the trait
            // In a real implementation, this would be redesigned to be async
            serde_json::json!({
                "ast_cache": "metrics_placeholder",
                "result_cache": "metrics_placeholder",
                "combined": true
            })
        };
        
        ast_cache_metrics
    }
    
    async fn get_cache_stats(&self) -> CacheStats {
        self.update_stats().await;
        self.cache_stats.read().await.clone()
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
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_cache_manager_creation() {
        let cache_manager = CacheManagerImpl::new().unwrap();
        let stats = cache_manager.get_cache_stats().await;
        
        assert_eq!(stats.ast_cache_size, 0);
        assert_eq!(stats.result_cache_size, 0);
    }
    
    #[tokio::test]
    async fn test_cache_manager_ast_operations() {
        let cache_manager = CacheManagerImpl::new().unwrap();
        
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
        let cache_manager = CacheManagerImpl::new().unwrap();
        
        // Create a temporary file path
        let temp_path = std::path::Path::new("/tmp/test_file.rs");
        
        // Test result caching
        let results = vec![]; // Empty results for testing
        cache_manager.cache_results(temp_path, results.clone()).await;
        
        let cached_results = cache_manager.get_cached_results(temp_path).await;
        assert!(cached_results.is_some());
        assert_eq!(cached_results.unwrap().len(), results.len());
    }
    
    #[tokio::test]
    async fn test_cache_manager_clear() {
        let cache_manager = CacheManagerImpl::new().unwrap();
        
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
        let cache_manager = CacheManagerImpl::new().unwrap();
        
        // Get metrics
        let metrics = cache_manager.get_cache_metrics();
        assert!(metrics.is_object());
        
        // Get stats
        let stats = cache_manager.get_cache_stats().await;
        assert_eq!(stats.ast_cache_size, 0);
        assert_eq!(stats.result_cache_size, 0);
    }
}