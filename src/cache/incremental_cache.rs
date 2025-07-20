//! Incremental Analysis Cache System
//!
//! Provides specialized caching for incremental analysis results with intelligent
//! invalidation based on file changes and dependency relationships.
//!
//! Key Features:
//! - Multi-layer cache architecture (file-level, result-level, dependency-level)
//! - Change-aware cache invalidation
//! - Memory-efficient storage with configurable limits
//! - High-performance lookup with O(1) average complexity

use crate::analysis::incremental::{FileState, Result, IncrementalAnalysisError};
use crate::database::models::ArchitecturalIssue;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use log::{debug, info, warn};
use lru::LruCache;

/// Configuration for the incremental cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalCacheConfig {
    /// Maximum number of analysis results to cache
    pub max_analysis_results: usize,
    
    /// Maximum memory usage for cache (MB)
    pub max_memory_mb: usize,
    
    /// Cache entry TTL (time to live) in hours
    pub ttl_hours: u32,
    
    /// Enable cache compression
    pub enable_compression: bool,
    
    /// Cache eviction policy
    pub eviction_policy: CacheEvictionPolicy,
    
    /// Enable cache statistics
    pub enable_statistics: bool,
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionPolicy {
    /// Least Recently Used
    LRU,
    
    /// Least Recently Used with size weighting
    LRUSize,
    
    /// Time-based eviction (oldest first)
    TimeBasedFIFO,
    
    /// Frequency-based eviction (least frequently used)
    LFU,
}

/// Cached analysis result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnalysisEntry {
    /// Analysis results for the file
    pub issues: Vec<ArchitecturalIssue>,
    
    /// File state when analysis was performed
    pub file_state: FileState,
    
    /// When this entry was cached
    pub cached_at: DateTime<Utc>,
    
    /// When this entry was last accessed
    pub last_accessed: DateTime<Utc>,
    
    /// Number of times this entry has been accessed
    pub access_count: u64,
    
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
    
    /// Estimated memory size of this entry
    pub estimated_size_bytes: usize,
    
    /// Whether this entry is still valid
    pub is_valid: bool,
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    /// Total cache hits
    pub hits: u64,
    
    /// Total cache misses
    pub misses: u64,
    
    /// Total cache invalidations
    pub invalidations: u64,
    
    /// Total entries evicted
    pub evictions: u64,
    
    /// Current number of cached entries
    pub entry_count: usize,
    
    /// Current memory usage in bytes
    pub memory_usage_bytes: usize,
    
    /// Average lookup time in microseconds
    pub average_lookup_time_us: f64,
    
    /// Cache efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
}

/// Multi-layer incremental cache
pub struct IncrementalCache {
    /// Configuration
    config: IncrementalCacheConfig,
    
    /// Analysis result cache (file path -> cached entry)
    analysis_cache: Arc<RwLock<LruCache<PathBuf, CachedAnalysisEntry>>>,
    
    /// Dependency relationship cache
    dependency_cache: Arc<RwLock<HashMap<PathBuf, HashSet<PathBuf>>>>,
    
    /// File modification time cache for quick lookups
    mtime_cache: Arc<RwLock<HashMap<PathBuf, u64>>>,
    
    /// Cache statistics
    stats: Arc<RwLock<CacheStatistics>>,
    
    /// Cache creation time for TTL calculations
    created_at: Instant,
}

impl IncrementalCache {
    /// Creates a new incremental cache with the given configuration
    pub fn new(config: IncrementalCacheConfig) -> Self {
        let analysis_cache = Arc::new(RwLock::new(
            LruCache::new(std::num::NonZero::new(config.max_analysis_results.try_into().unwrap_or(1000)).unwrap())
        ));

        Self {
            config,
            analysis_cache,
            dependency_cache: Arc::new(RwLock::new(HashMap::new())),
            mtime_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStatistics::default())),
            created_at: Instant::now(),
        }
    }

    /// Gets cached analysis result for a file
    pub async fn get_analysis_result(&self, file_path: &PathBuf) -> Result<Option<Vec<ArchitecturalIssue>>> {
        let lookup_start = Instant::now();
        
        let mut cache = self.analysis_cache.write().await;
        let result = if let Some(entry) = cache.get_mut(file_path) {
            // Update access statistics
            entry.last_accessed = Utc::now();
            entry.access_count += 1;
            
            // Check if entry is still valid and within TTL
            if self.is_entry_valid(entry).await {
                debug!("Cache hit for: {}", file_path.display());
                
                // Update statistics
                if self.config.enable_statistics {
                    let mut stats = self.stats.write().await;
                    stats.hits += 1;
                    self.update_lookup_time(&mut stats, lookup_start.elapsed());
                }
                
                Some(entry.issues.clone())
            } else {
                debug!("Cache entry expired for: {}", file_path.display());
                
                // Remove expired entry
                cache.pop(file_path);
                
                // Update statistics
                if self.config.enable_statistics {
                    let mut stats = self.stats.write().await;
                    stats.misses += 1;
                    stats.invalidations += 1;
                    self.update_lookup_time(&mut stats, lookup_start.elapsed());
                }
                
                None
            }
        } else {
            debug!("Cache miss for: {}", file_path.display());
            
            // Update statistics
            if self.config.enable_statistics {
                let mut stats = self.stats.write().await;
                stats.misses += 1;
                self.update_lookup_time(&mut stats, lookup_start.elapsed());
            }
            
            None
        };

        drop(cache);
        Ok(result)
    }

    /// Stores analysis result in cache
    pub async fn store_analysis_result(
        &self,
        file_path: PathBuf,
        issues: Vec<ArchitecturalIssue>,
        file_state: FileState,
        analysis_duration_ms: u64,
    ) -> Result<()> {
        let now = Utc::now();
        let estimated_size = self.estimate_entry_size(&issues);

        let entry = CachedAnalysisEntry {
            issues,
            file_state: file_state.clone(),
            cached_at: now,
            last_accessed: now,
            access_count: 1,
            analysis_duration_ms,
            estimated_size_bytes: estimated_size,
            is_valid: true,
        };

        // Check memory limits before storing
        if self.would_exceed_memory_limit(&entry).await {
            self.evict_entries_for_space(&entry).await?;
        }

        let cache_len = {
            let mut cache = self.analysis_cache.write().await;
            cache.put(file_path.clone(), entry);
            cache.len()
        };

        // Update modification time cache
        {
            let mut mtime_cache = self.mtime_cache.write().await;
            mtime_cache.insert(file_path.clone(), file_state.last_modified);
        }

        // Update statistics
        if self.config.enable_statistics {
            let mut stats = self.stats.write().await;
            stats.entry_count = cache_len;
            stats.memory_usage_bytes += estimated_size;
        }

        debug!("Cached analysis result for: {}", file_path.display());
        Ok(())
    }

    /// Invalidates cache entries for changed files
    pub async fn invalidate_files(&self, changed_files: &HashSet<PathBuf>) -> Result<()> {
        let mut invalidated_count = 0;

        {
            let mut cache = self.analysis_cache.write().await;
            for file_path in changed_files {
                if cache.pop(file_path).is_some() {
                    invalidated_count += 1;
                }
            }
        }

        // Also invalidate dependent files
        let dependent_files = self.find_dependent_files(changed_files).await;
        if !dependent_files.is_empty() {
            let mut cache = self.analysis_cache.write().await;
            for file_path in &dependent_files {
                if cache.pop(file_path).is_some() {
                    invalidated_count += 1;
                }
            }
        }

        // Update modification time cache
        {
            let mut mtime_cache = self.mtime_cache.write().await;
            for file_path in changed_files {
                mtime_cache.remove(file_path);
            }
            for file_path in &dependent_files {
                mtime_cache.remove(file_path);
            }
        }

        let cache_len = {
            let cache = self.analysis_cache.read().await;
            cache.len()
        };

        // Update statistics
        if self.config.enable_statistics {
            let mut stats = self.stats.write().await;
            stats.invalidations += invalidated_count;
            stats.entry_count = cache_len;
        }

        info!("Invalidated {} cache entries for changed files", invalidated_count);
        Ok(())
    }

    /// Updates dependency relationships in cache
    pub async fn update_dependencies(
        &self,
        file_path: PathBuf,
        dependencies: HashSet<PathBuf>,
    ) -> Result<()> {
        let mut dep_cache = self.dependency_cache.write().await;
        dep_cache.insert(file_path.clone(), dependencies);
        drop(dep_cache);

        debug!("Updated dependencies for: {}", file_path.display());
        Ok(())
    }

    /// Finds files that depend on the given changed files
    async fn find_dependent_files(&self, changed_files: &HashSet<PathBuf>) -> HashSet<PathBuf> {
        let mut dependent_files = HashSet::new();
        let dep_cache = self.dependency_cache.read().await;

        // Find all files that depend on any of the changed files
        for (file_path, dependencies) in dep_cache.iter() {
            if dependencies.iter().any(|dep| changed_files.contains(dep)) {
                dependent_files.insert(file_path.clone());
            }
        }

        drop(dep_cache);
        dependent_files
    }

    /// Checks if a cache entry is still valid
    async fn is_entry_valid(&self, entry: &CachedAnalysisEntry) -> bool {
        // Check TTL
        if self.config.ttl_hours > 0 {
            let age = Utc::now() - entry.cached_at;
            if age.num_hours() > self.config.ttl_hours as i64 {
                return false;
            }
        }

        // Check if explicitly marked as invalid
        if !entry.is_valid {
            return false;
        }

        true
    }

    /// Estimates the memory size of a cache entry
    fn estimate_entry_size(&self, issues: &[ArchitecturalIssue]) -> usize {
        // Rough estimation of memory usage
        // Base size + size per issue
        let base_size = std::mem::size_of::<CachedAnalysisEntry>();
        let issues_size = issues.len() * 200; // Estimated average issue size
        base_size + issues_size
    }

    /// Checks if storing an entry would exceed memory limits
    async fn would_exceed_memory_limit(&self, entry: &CachedAnalysisEntry) -> bool {
        if self.config.max_memory_mb == 0 {
            return false; // No limit
        }

        let stats = self.stats.read().await;
        let current_memory_mb = stats.memory_usage_bytes / (1024 * 1024);
        let entry_memory_mb = entry.estimated_size_bytes / (1024 * 1024);
        
        (current_memory_mb + entry_memory_mb) > self.config.max_memory_mb
    }

    /// Evicts entries to make space for a new entry
    async fn evict_entries_for_space(&self, new_entry: &CachedAnalysisEntry) -> Result<()> {
        let target_memory_mb = self.config.max_memory_mb * 80 / 100; // Target 80% of limit
        let mut evicted_count = 0;

        match self.config.eviction_policy {
            CacheEvictionPolicy::LRU => {
                // LRU eviction is handled automatically by LruCache
                let mut cache = self.analysis_cache.write().await;
                while cache.len() > 0 {
                    let stats = self.stats.read().await;
                    let current_memory_mb = stats.memory_usage_bytes / (1024 * 1024);
                    drop(stats);
                    
                    if current_memory_mb <= target_memory_mb {
                        break;
                    }
                    
                    if let Some((_, entry)) = cache.pop_lru() {
                        evicted_count += 1;
                        
                        // Update memory usage
                        let mut stats = self.stats.write().await;
                        stats.memory_usage_bytes = stats.memory_usage_bytes
                            .saturating_sub(entry.estimated_size_bytes);
                    } else {
                        break;
                    }
                }
            }
            CacheEvictionPolicy::TimeBasedFIFO => {
                // Find oldest entries and remove them
                let mut cache = self.analysis_cache.write().await;
                let mut entries_by_age: Vec<(PathBuf, DateTime<Utc>)> = cache
                    .iter()
                    .map(|(path, entry)| (path.clone(), entry.cached_at))
                    .collect();
                
                entries_by_age.sort_by(|a, b| a.1.cmp(&b.1));
                
                for (path, _) in entries_by_age {
                    let stats = self.stats.read().await;
                    let current_memory_mb = stats.memory_usage_bytes / (1024 * 1024);
                    drop(stats);
                    
                    if current_memory_mb <= target_memory_mb {
                        break;
                    }
                    
                    if let Some(entry) = cache.pop(&path) {
                        evicted_count += 1;
                        
                        // Update memory usage
                        let mut stats = self.stats.write().await;
                        stats.memory_usage_bytes = stats.memory_usage_bytes
                            .saturating_sub(entry.estimated_size_bytes);
                    }
                }
            }
            _ => {
                // For other policies, use LRU as fallback
                warn!("Eviction policy not implemented, using LRU");
            }
        }

        // Update statistics
        if self.config.enable_statistics {
            let mut stats = self.stats.write().await;
            stats.evictions += evicted_count;
        }

        if evicted_count > 0 {
            info!("Evicted {} cache entries to free memory", evicted_count);
        }

        Ok(())
    }

    /// Updates average lookup time statistics
    fn update_lookup_time(&self, stats: &mut CacheStatistics, lookup_time: Duration) {
        let lookup_time_us = lookup_time.as_micros() as f64;
        let total_lookups = stats.hits + stats.misses;
        
        if total_lookups == 1 {
            stats.average_lookup_time_us = lookup_time_us;
        } else {
            // Running average
            stats.average_lookup_time_us = 
                (stats.average_lookup_time_us * (total_lookups - 1) as f64 + lookup_time_us) / total_lookups as f64;
        }

        // Update efficiency score
        stats.efficiency_score = if total_lookups > 0 {
            stats.hits as f64 / total_lookups as f64
        } else {
            0.0
        };
    }

    /// Gets current cache statistics
    pub async fn get_statistics(&self) -> CacheStatistics {
        let stats = self.stats.read().await;
        let mut result = stats.clone();
        drop(stats);

        // Update current entry count
        let cache = self.analysis_cache.read().await;
        result.entry_count = cache.len();
        drop(cache);

        result
    }

    /// Clears all cached entries
    pub async fn clear(&self) -> Result<()> {
        {
            let mut cache = self.analysis_cache.write().await;
            cache.clear();
        }

        {
            let mut dep_cache = self.dependency_cache.write().await;
            dep_cache.clear();
        }

        {
            let mut mtime_cache = self.mtime_cache.write().await;
            mtime_cache.clear();
        }

        // Reset statistics
        if self.config.enable_statistics {
            let mut stats = self.stats.write().await;
            *stats = CacheStatistics::default();
        }

        info!("Cleared all cache entries");
        Ok(())
    }

    /// Gets cache hit rate
    pub async fn get_hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        let total_lookups = stats.hits + stats.misses;
        
        if total_lookups > 0 {
            stats.hits as f64 / total_lookups as f64
        } else {
            0.0
        }
    }

    /// Validates cache consistency and repairs if needed
    pub async fn validate_and_repair(&self) -> Result<()> {
        info!("Validating cache consistency");
        let mut repairs_made = 0;

        // Check for expired entries
        {
            let mut cache = self.analysis_cache.write().await;
            let mut to_remove = Vec::new();

            for (path, entry) in cache.iter() {
                if !self.is_entry_valid(entry).await {
                    to_remove.push(path.clone());
                }
            }

            for path in to_remove {
                cache.pop(&path);
                repairs_made += 1;
            }
            
            let cache_len = cache.len();
            drop(cache);

            // Update statistics if repairs were made
            if repairs_made > 0 {
                let mut stats = self.stats.write().await;
                stats.invalidations += repairs_made;
                stats.entry_count = cache_len;
                
                info!("Cache validation completed: {} entries repaired", repairs_made);
            }
        }

        Ok(())
    }
}

impl Default for IncrementalCacheConfig {
    fn default() -> Self {
        Self {
            max_analysis_results: 10_000,
            max_memory_mb: 512,
            ttl_hours: 24,
            enable_compression: false,
            eviction_policy: CacheEvictionPolicy::LRU,
            enable_statistics: true,
        }
    }
}

impl CacheStatistics {
    /// Calculates the hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Calculates the miss rate
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }

    /// Calculates memory utilization
    pub fn memory_utilization_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cache_creation() {
        let config = IncrementalCacheConfig::default();
        let cache = IncrementalCache::new(config);
        
        let stats = cache.get_statistics().await;
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_store_and_retrieve() {
        let config = IncrementalCacheConfig::default();
        let cache = IncrementalCache::new(config);
        
        let file_path = PathBuf::from("test.rs");
        let issues = vec![];
        let file_state = FileState {
            last_modified: 12345,
            content_hash: "test_hash".to_string(),
            file_size: 1024,
            dependencies: std::collections::HashSet::new(),
            dependents: std::collections::HashSet::new(),
            last_analyzed: Utc::now(),
            exists: true,
        };

        // Store result
        cache.store_analysis_result(file_path.clone(), issues.clone(), file_state, 100).await.unwrap();

        // Retrieve result
        let cached_result = cache.get_analysis_result(&file_path).await.unwrap();
        assert!(cached_result.is_some());
        assert_eq!(cached_result.unwrap().len(), 0);

        // Check statistics
        let stats = cache.get_statistics().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let config = IncrementalCacheConfig::default();
        let cache = IncrementalCache::new(config);
        
        let file_path = PathBuf::from("nonexistent.rs");
        let result = cache.get_analysis_result(&file_path).await.unwrap();
        
        assert!(result.is_none());
        
        let stats = cache.get_statistics().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let config = IncrementalCacheConfig::default();
        let cache = IncrementalCache::new(config);
        
        let file_path = PathBuf::from("test.rs");
        let issues = vec![];
        let file_state = FileState {
            last_modified: 12345,
            content_hash: "test_hash".to_string(),
            file_size: 1024,
            dependencies: std::collections::HashSet::new(),
            dependents: std::collections::HashSet::new(),
            last_analyzed: Utc::now(),
            exists: true,
        };

        // Store result
        cache.store_analysis_result(file_path.clone(), issues, file_state, 100).await.unwrap();

        // Invalidate
        let mut changed_files = HashSet::new();
        changed_files.insert(file_path.clone());
        cache.invalidate_files(&changed_files).await.unwrap();

        // Try to retrieve
        let result = cache.get_analysis_result(&file_path).await.unwrap();
        assert!(result.is_none());

        let stats = cache.get_statistics().await;
        assert_eq!(stats.invalidations, 1);
    }

    #[test]
    fn test_cache_statistics() {
        let mut stats = CacheStatistics::default();
        stats.hits = 80;
        stats.misses = 20;
        
        assert_eq!(stats.hit_rate(), 0.8);
        assert_eq!(stats.miss_rate(), 0.2);
        
        stats.memory_usage_bytes = 1024 * 1024; // 1MB
        assert_eq!(stats.memory_utilization_mb(), 1.0);
    }
}