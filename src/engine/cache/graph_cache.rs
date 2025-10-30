//! # Graph Cache Module
//!
//! Specialized cache for knowledge graph data including relationships,
//! dependency graphs, and computed graph queries.

use crate::engine::cache::metrics::CacheMetricsCollector;
use crate::engine::knowledge_graph::builder::GraphStats;
use crate::engine::knowledge_graph::relations::GraphRelation;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// Cache for graph-specific data with TTL and invalidation
#[derive(Debug)]
/// Graph structure representing cache relationships.
pub struct GraphCache {
    /// Cached graph relations by file/context
    relations: HashMap<String, CachedRelations>,
    /// Cached dependency computations
    dependencies: HashMap<String, CachedDependencies>,
    /// Cached graph statistics
    statistics: HashMap<String, CachedStats>,
    /// Cached query results
    query_results: HashMap<String, CachedQueryResult>,
    /// Metrics collector
    metrics: CacheMetricsCollector,
    /// Cache configuration
    config: GraphCacheConfig,
}

/// Cached graph relations with metadata
#[derive(Debug, Clone)]
/// Represents cached relations in the system.
pub struct CachedRelations {
    pub relations: Vec<GraphRelation>,
    pub cached_at: Instant,
    pub file_hash: u64,
    pub dependency_hash: u64,
}

/// Cached dependency computation
#[derive(Debug, Clone)]
/// Represents cached dependencies in the system.
pub struct CachedDependencies {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
    pub cycles: Vec<Vec<String>>,
    pub cached_at: Instant,
    pub invalidation_key: String,
}

/// Cached graph statistics
#[derive(Debug, Clone)]
/// Represents cached stats in the system.
pub struct CachedStats {
    pub stats: GraphStats,
    pub cached_at: Instant,
    pub computation_time_ms: u64,
}

/// Cached query result
#[derive(Debug, Clone)]
/// Result of cached query operation.
pub struct CachedQueryResult {
    pub result: QueryResultData,
    pub cached_at: Instant,
    pub query_hash: u64,
}

/// Query result data types
#[derive(Debug, Clone)]
pub enum QueryResultData {
    /// Path query results
    Paths(Vec<Vec<String>>),
    /// Node query results
    Nodes(Vec<String>),
    /// Relation query results
    Relations(Vec<GraphRelation>),
    /// Boolean query results
    Boolean(bool),
}

/// Graph cache configuration
#[derive(Debug, Clone)]
/// Configuration options for graphcache.
///
/// Provides sensible defaults that can be overridden via builder methods
/// or loaded from configuration files.
pub struct GraphCacheConfig {
    /// Maximum entries per cache type
    pub max_entries: usize,
    /// Time-to-live for cached entries
    pub ttl: Duration,
    /// Enable automatic cleanup
    pub enable_cleanup: bool,
    /// Cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for GraphCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(3600), // 1 hour
            enable_cleanup: true,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl GraphCache {
    /// Create a new graph cache
    pub fn new(config: GraphCacheConfig) -> Self {
        Self {
            relations: HashMap::new(),
            dependencies: HashMap::new(),
            statistics: HashMap::new(),
            query_results: HashMap::new(),
            metrics: CacheMetricsCollector::new(),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(GraphCacheConfig::default())
    }

    /// Cache graph relations for a file
    pub fn cache_relations(
        &mut self,
        cache_key: String,
        relations: Vec<GraphRelation>,
        file_hash: u64,
    ) {
        let cached = CachedRelations {
            relations,
            cached_at: Instant::now(),
            file_hash,
            dependency_hash: self.compute_dependency_hash(&cache_key),
        };

        self.relations.insert(cache_key, cached);
        self.metrics
            .record_cache_operation("graph_relations", "write");
        self.enforce_size_limits();
    }

    /// Get cached relations
    pub fn get_relations(
        &mut self,
        cache_key: &str,
        current_file_hash: u64,
    ) -> Option<Vec<GraphRelation>> {
        if let Some(cached) = self.relations.get(cache_key) {
            // Check if cache is still valid
            if self.is_cache_valid(cached.cached_at) && cached.file_hash == current_file_hash {
                self.metrics.record_hit("graph_relations");
                return Some(cached.relations.clone());
            }
        }
        // Remove stale entry if invalid
        self.relations.remove(cache_key);
        self.metrics.record_miss("graph_relations");
        None
    }

    /// Cache dependency computation
    pub fn cache_dependencies(
        &mut self,
        cache_key: String,
        nodes: Vec<String>,
        edges: Vec<(String, String)>,
        cycles: Vec<Vec<String>>,
    ) {
        let cached = CachedDependencies {
            nodes,
            edges,
            cycles,
            cached_at: Instant::now(),
            invalidation_key: cache_key.clone(),
        };

        self.dependencies.insert(cache_key, cached);
        self.metrics
            .record_cache_operation("graph_dependencies", "write");
        self.enforce_size_limits();
    }

    /// Get cached dependencies
    pub fn get_dependencies(&mut self, cache_key: &str) -> Option<CachedDependencies> {
        if let Some(cached) = self.dependencies.get(cache_key) {
            if self.is_cache_valid(cached.cached_at) {
                self.metrics.record_hit("graph_dependencies");
                return Some(cached.clone());
            }
        }
        // Remove stale entry if invalid
        self.dependencies.remove(cache_key);
        self.metrics.record_miss("graph_dependencies");
        None
    }

    /// Cache graph statistics
    pub fn cache_stats(&mut self, cache_key: String, stats: GraphStats, computation_time_ms: u64) {
        let cached = CachedStats {
            stats,
            cached_at: Instant::now(),
            computation_time_ms,
        };

        self.statistics.insert(cache_key, cached);
        self.metrics.record_cache_operation("graph_stats", "write");
        self.enforce_size_limits();
    }

    /// Get cached statistics
    pub fn get_stats(&mut self, cache_key: &str) -> Option<GraphStats> {
        if let Some(cached) = self.statistics.get(cache_key) {
            if self.is_cache_valid(cached.cached_at) {
                self.metrics.record_hit("graph_stats");
                return Some(cached.stats.clone());
            }
        }
        // Remove stale entry if invalid
        self.statistics.remove(cache_key);
        self.metrics.record_miss("graph_stats");
        None
    }

    /// Cache query result
    pub fn cache_query_result(&mut self, query_hash: u64, result: QueryResultData) {
        let cache_key = format!("query_{:x}", query_hash);
        let cached = CachedQueryResult {
            result,
            cached_at: Instant::now(),
            query_hash,
        };

        self.query_results.insert(cache_key, cached);
        self.metrics
            .record_cache_operation("graph_queries", "write");
        self.enforce_size_limits();
    }

    /// Get cached query result
    pub fn get_query_result(&mut self, query_hash: u64) -> Option<QueryResultData> {
        let cache_key = format!("query_{:x}", query_hash);
        if let Some(cached) = self.query_results.get(&cache_key) {
            if self.is_cache_valid(cached.cached_at) && cached.query_hash == query_hash {
                self.metrics.record_hit("graph_queries");
                return Some(cached.result.clone());
            }
        }
        // Remove stale entry if invalid
        self.query_results.remove(&cache_key);
        self.metrics.record_miss("graph_queries");
        None
    }

    /// Invalidate cache entries based on file changes
    pub fn invalidate_file(&mut self, file_path: &str) {
        // Remove all entries related to this file
        let file_prefix = format!("file:{}", file_path);

        self.relations
            .retain(|key, _| !key.starts_with(&file_prefix));
        self.dependencies
            .retain(|key, _| !key.starts_with(&file_prefix));
        self.statistics
            .retain(|key, _| !key.starts_with(&file_prefix));

        self.metrics
            .record_cache_operation("cache", "invalidate_file");
    }

    /// Invalidate all caches
    pub fn invalidate_all(&mut self) {
        self.relations.clear();
        self.dependencies.clear();
        self.statistics.clear();
        self.query_results.clear();
        self.metrics.record_cache_operation("cache", "clear_all");
    }

    /// Get cache usage statistics
    pub fn get_cache_stats(&self) -> GraphCacheStats {
        GraphCacheStats {
            relations_count: self.relations.len(),
            dependencies_count: self.dependencies.len(),
            statistics_count: self.statistics.len(),
            query_results_count: self.query_results.len(),
            total_entries: self.relations.len()
                + self.dependencies.len()
                + self.statistics.len()
                + self.query_results.len(),
        }
    }

    /// Check if cache entry is still valid
    fn is_cache_valid(&self, cached_at: Instant) -> bool {
        cached_at.elapsed() < self.config.ttl
    }

    /// Compute dependency hash for cache invalidation
    fn compute_dependency_hash(&self, cache_key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        cache_key.hash(&mut hasher);
        SystemTime::now().hash(&mut hasher);
        hasher.finish()
    }

    /// Enforce cache size limits
    fn enforce_size_limits(&mut self) {
        if self.relations.len() > self.config.max_entries {
            self.evict_oldest_relations();
        }
        if self.dependencies.len() > self.config.max_entries {
            self.evict_oldest_dependencies();
        }
        if self.statistics.len() > self.config.max_entries {
            self.evict_oldest_statistics();
        }
        if self.query_results.len() > self.config.max_entries {
            self.evict_oldest_queries();
        }
    }

    /// Evict oldest relations entries
    fn evict_oldest_relations(&mut self) {
        let mut entries: Vec<_> = self
            .relations
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at))
            .collect();
        entries.sort_by_key(|(_, cached_at)| *cached_at);

        let to_remove = entries.len() - (self.config.max_entries * 3 / 4);
        for (key, _) in entries.into_iter().take(to_remove) {
            self.relations.remove(&key);
        }
    }

    /// Evict oldest dependencies entries
    fn evict_oldest_dependencies(&mut self) {
        let mut entries: Vec<_> = self
            .dependencies
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at))
            .collect();
        entries.sort_by_key(|(_, cached_at)| *cached_at);

        let to_remove = entries.len() - (self.config.max_entries * 3 / 4);
        for (key, _) in entries.into_iter().take(to_remove) {
            self.dependencies.remove(&key);
        }
    }

    /// Evict oldest statistics entries
    fn evict_oldest_statistics(&mut self) {
        let mut entries: Vec<_> = self
            .statistics
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at))
            .collect();
        entries.sort_by_key(|(_, cached_at)| *cached_at);

        let to_remove = entries.len() - (self.config.max_entries * 3 / 4);
        for (key, _) in entries.into_iter().take(to_remove) {
            self.statistics.remove(&key);
        }
    }

    /// Evict oldest query results
    fn evict_oldest_queries(&mut self) {
        let mut entries: Vec<_> = self
            .query_results
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at))
            .collect();
        entries.sort_by_key(|(_, cached_at)| *cached_at);

        let to_remove = entries.len() - (self.config.max_entries * 3 / 4);
        for (key, _) in entries.into_iter().take(to_remove) {
            self.query_results.remove(&key);
        }
    }

    /// Get cache metrics
    pub fn get_metrics(&self) -> &CacheMetricsCollector {
        &self.metrics
    }
}

/// Graph cache statistics
#[derive(Debug)]
/// Graph structure representing cache stats relationships.
pub struct GraphCacheStats {
    pub relations_count: usize,
    pub dependencies_count: usize,
    pub statistics_count: usize,
    pub query_results_count: usize,
    pub total_entries: usize,
}

/// Thread-safe graph cache wrapper
pub type SharedGraphCache = Arc<Mutex<GraphCache>>;

/// Create a shared graph cache
pub fn create_shared_graph_cache(config: GraphCacheConfig) -> SharedGraphCache {
    Arc::new(Mutex::new(GraphCache::new(config)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::knowledge_graph::relations::RelationType;

    #[test]
    fn test_graph_cache_relations() {
        let mut cache = GraphCache::with_default_config();

        let relations = vec![GraphRelation {
            from: "test::func".to_string(),
            to: "std::vec::Vec".to_string(),
            relation_type: RelationType::Uses,
            file_path: "test.rs".to_string(),
        }];

        // Cache relations
        cache.cache_relations("test_key".to_string(), relations.clone(), 12345);

        // Retrieve relations
        let cached = cache.get_relations("test_key", 12345).unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].from, "test::func");

        // Test cache miss with different hash
        let missed = cache.get_relations("test_key", 54321);
        assert!(missed.is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = GraphCache::with_default_config();

        let relations = vec![GraphRelation {
            from: "file:test.rs::func".to_string(),
            to: "std::vec::Vec".to_string(),
            relation_type: RelationType::Uses,
            file_path: "test.rs".to_string(),
        }];

        cache.cache_relations("file:test.rs::key".to_string(), relations, 12345);

        // Verify cache entry exists
        assert!(cache.get_relations("file:test.rs::key", 12345).is_some());

        // Invalidate file
        cache.invalidate_file("test.rs");

        // Verify cache entry is gone
        assert!(cache.get_relations("file:test.rs::key", 12345).is_none());
    }

    #[test]
    fn test_cache_size_limits() {
        let config = GraphCacheConfig {
            max_entries: 2,
            ttl: Duration::from_secs(3600),
            enable_cleanup: false,
            cleanup_interval: Duration::from_secs(300),
        };

        let mut cache = GraphCache::new(config);

        // Add entries beyond limit
        for i in 0..5 {
            let relations = vec![GraphRelation {
                from: format!("func_{}", i),
                to: "std::vec::Vec".to_string(),
                relation_type: RelationType::Uses,
                file_path: "test.rs".to_string(),
            }];
            cache.cache_relations(format!("key_{}", i), relations, i as u64);
        }

        // Should have evicted oldest entries
        assert!(cache.relations.len() <= 2);
    }
}
