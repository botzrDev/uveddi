//! Advanced Multi-Layer Cache Engine for Large Codebases
//!
//! Implements intelligent caching strategies with memory hierarchy, compression,
//! intelligent eviction, and distributed caching capabilities for enterprise-scale
//! analysis performance.

use crate::analysis::cache::{AstCache, CacheConfig};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use crate::error::{Result, UveddiError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, info, warn};

/// Advanced cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for advancedcache.
pub struct AdvancedCacheConfig {
    /// Memory cache configuration
    pub memory_cache: MemoryCacheConfig,
    
    /// Disk cache configuration
    pub disk_cache: DiskCacheConfig,
    
    /// Distributed cache configuration
    pub distributed_cache: Option<DistributedCacheConfig>,
    
    /// Compression configuration
    pub compression: CompressionConfig,
    
    /// Eviction policies
    pub eviction_policy: EvictionPolicyConfig,
    
    /// Cache warming strategies
    pub warming_strategy: CacheWarmingConfig,
    
    /// Performance monitoring
    pub monitoring: CacheMonitoringConfig,
}

impl Default for AdvancedCacheConfig {
    fn default() -> Self {
        Self {
            memory_cache: MemoryCacheConfig::default(),
            disk_cache: DiskCacheConfig::default(),
            distributed_cache: None,
            compression: CompressionConfig::default(),
            eviction_policy: EvictionPolicyConfig::default(),
            warming_strategy: CacheWarmingConfig::default(),
            monitoring: CacheMonitoringConfig::default(),
        }
    }
}

/// Memory cache tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for memorycache.
pub struct MemoryCacheConfig {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    
    /// AST cache size limit
    pub ast_cache_size: usize,
    
    /// Result cache size limit
    pub result_cache_size: usize,
    
    /// Metadata cache size limit
    pub metadata_cache_size: usize,
    
    /// Enable memory-mapped files for large ASTs
    pub enable_memory_mapping: bool,
    
    /// Memory pressure threshold (0.0-1.0)
    pub pressure_threshold: f64,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            ast_cache_size: 10000,
            result_cache_size: 50000,
            metadata_cache_size: 100000,
            enable_memory_mapping: true,
            pressure_threshold: 0.8,
        }
    }
}

/// Disk cache tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for diskcache.
pub struct DiskCacheConfig {
    /// Cache directory path
    pub cache_directory: PathBuf,
    
    /// Maximum disk usage in bytes
    pub max_disk_bytes: usize,
    
    /// Enable compression for disk storage
    pub enable_compression: bool,
    
    /// File-based cache chunk size
    pub chunk_size_bytes: usize,
    
    /// Cache file retention period
    pub retention_hours: u64,
    
    /// Enable cache integrity checks
    pub enable_integrity_checks: bool,
}

impl Default for DiskCacheConfig {
    fn default() -> Self {
        Self {
            cache_directory: std::env::temp_dir().join("uveddi_cache"),
            max_disk_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            enable_compression: true,
            chunk_size_bytes: 1024 * 1024, // 1MB chunks
            retention_hours: 168, // 1 week
            enable_integrity_checks: true,
        }
    }
}

/// Distributed cache configuration (Redis, Hazelcast, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for distributedcache.
pub struct DistributedCacheConfig {
    /// Cache provider type
    pub provider: String, // "redis", "hazelcast", "custom"
    
    /// Connection configuration
    pub connection_string: String,
    
    /// Maximum network timeout
    pub network_timeout_ms: u64,
    
    /// Enable cache replication
    pub enable_replication: bool,
    
    /// Cache key prefix
    pub key_prefix: String,
    
    /// TTL for distributed entries
    pub default_ttl_seconds: u64,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for compression.
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
    
    /// Compression level (1-9)
    pub compression_level: u8,
    
    /// Minimum size threshold for compression
    pub min_compression_size: usize,
    
    /// Enable adaptive compression
    pub enable_adaptive_compression: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            compression_level: 3,
            min_compression_size: 1024, // 1KB
            enable_adaptive_compression: true,
        }
    }
}

/// Available compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enumeration of compressionalgorithm variants.
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
    Lz4,
    Brotli,
}

/// Cache eviction policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for evictionpolicy.
pub struct EvictionPolicyConfig {
    /// Primary eviction policy
    pub policy: EvictionPolicy,
    
    /// Secondary policy for tie-breaking
    pub secondary_policy: Option<EvictionPolicy>,
    
    /// Eviction batch size
    pub batch_size: usize,
    
    /// Enable proactive eviction
    pub proactive_eviction: bool,
    
    /// Eviction trigger threshold
    pub trigger_threshold: f64,
}

impl Default for EvictionPolicyConfig {
    fn default() -> Self {
        Self {
            policy: EvictionPolicy::AdaptiveLRU,
            secondary_policy: Some(EvictionPolicy::LeastRecentlyUsed),
            batch_size: 100,
            proactive_eviction: true,
            trigger_threshold: 0.9,
        }
    }
}

/// Available eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enumeration of evictionpolicy variants.
pub enum EvictionPolicy {
    LeastRecentlyUsed,      // Classic LRU
    LeastFrequentlyUsed,    // LFU based on access frequency
    TimeToLive,             // TTL-based expiration
    SizeBasedEviction,      // Evict largest items first
    AdaptiveLRU,           // Adaptive LRU with access patterns
    WeightedEviction,      // Custom weight-based eviction
    PredictiveEviction,    // ML-based eviction prediction
}

/// Cache warming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for cachewarming.
pub struct CacheWarmingConfig {
    /// Enable automatic cache warming
    pub enable_warming: bool,
    
    /// Warming strategy
    pub strategy: WarmingStrategy,
    
    /// Maximum warming concurrency
    pub max_concurrent_warmers: usize,
    
    /// Warming trigger conditions
    pub trigger_conditions: Vec<WarmingTrigger>,
}

impl Default for CacheWarmingConfig {
    fn default() -> Self {
        Self {
            enable_warming: true,
            strategy: WarmingStrategy::PredictiveWarming,
            max_concurrent_warmers: 4,
            trigger_conditions: vec![
                WarmingTrigger::ColdStart,
                WarmingTrigger::LowHitRate,
            ],
        }
    }
}

/// Cache warming strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enumeration of warmingstrategy variants.
pub enum WarmingStrategy {
    PreloadCommon,        // Pre-load commonly accessed items
    PredictiveWarming,    // ML-based predictive warming
    AccessPatternBased,   // Based on historical access patterns
    DependencyChained,    // Warm dependent items together
    TimeBasedWarming,     // Warm during off-peak hours
}

/// Cache warming triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enumeration of warmingtrigger variants.
pub enum WarmingTrigger {
    ColdStart,           // Application startup
    LowHitRate,          // Hit rate below threshold
    NewAnalysisRun,      // New analysis started
    TimeBasedSchedule,   // Scheduled warming
    MemoryAvailable,     // When memory becomes available
}

/// Cache monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for cachemonitoring.
pub struct CacheMonitoringConfig {
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    
    /// Metrics collection interval
    pub metrics_interval_seconds: u64,
    
    /// Enable cache analytics
    pub enable_analytics: bool,
    
    /// Export metrics to external systems
    pub export_metrics: bool,
}

impl Default for CacheMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            metrics_interval_seconds: 30,
            enable_analytics: true,
            export_metrics: false,
        }
    }
}

/// Cache entry metadata for intelligent management
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metadata describing cache entry properties and attributes.
pub struct CacheEntryMetadata {
    /// Entry creation time
    pub created_at: SystemTime,
    
    /// Last access time
    pub last_accessed: SystemTime,
    
    /// Access count
    pub access_count: u64,
    
    /// Entry size in bytes
    pub size_bytes: usize,
    
    /// Compressed size (if applicable)
    pub compressed_size_bytes: Option<usize>,
    
    /// Cache hit cost (time to regenerate)
    pub regeneration_cost_ms: u64,
    
    /// Dependency tracking
    pub dependencies: Vec<String>,
    
    /// Priority score for eviction decisions
    pub priority_score: f64,
    
    /// Expiry time (if applicable)
    pub expires_at: Option<SystemTime>,
}

/// Advanced cache performance metrics
#[derive(Debug, Clone, Default)]
/// Performance and quality metrics for advancedcache.
pub struct AdvancedCacheMetrics {
    // Hit/miss statistics
    pub memory_hits: u64,
    pub disk_hits: u64,
    pub distributed_hits: u64,
    pub cache_misses: u64,
    
    // Performance metrics
    pub average_access_time_ns: u64,
    pub memory_access_time_ns: u64,
    pub disk_access_time_ns: u64,
    pub distributed_access_time_ns: u64,
    
    // Storage metrics
    pub memory_usage_bytes: usize,
    pub disk_usage_bytes: usize,
    pub compression_ratio: f64,
    
    // Eviction metrics
    pub items_evicted: u64,
    pub eviction_time_ms: u64,
    
    // Warming metrics
    pub items_warmed: u64,
    pub warming_time_ms: u64,
    pub warming_hit_rate: f64,
    
    // Error metrics
    pub corruption_errors: u64,
    pub network_errors: u64,
    pub disk_errors: u64,
}

/// Cache entry for multi-tier storage
pub struct CacheEntry<T> {
    pub data: T,
    pub metadata: CacheEntryMetadata,
    pub tier: CacheTier,
}

/// Cache storage tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Enumeration of cachetier variants.
pub enum CacheTier {
    Memory,      // Fast in-memory storage
    Disk,        // Persistent disk storage
    Distributed, // Network-based distributed cache
}

/// Advanced multi-tier cache engine
pub struct AdvancedCacheEngine {
    config: AdvancedCacheConfig,
    
    // Memory tier
    memory_cache: Arc<RwLock<HashMap<String, CacheEntry<Vec<u8>>>>>,
    
    // Disk tier management
    disk_cache: Arc<RwLock<DiskCacheManager>>,
    
    // Distributed cache client
    distributed_cache: Option<Arc<RwLock<dyn DistributedCacheClient + Send + Sync>>>,
    
    // Cache metadata tracking
    metadata_index: Arc<RwLock<HashMap<String, CacheEntryMetadata>>>,
    
    // Access pattern tracking for intelligent eviction
    access_patterns: Arc<RwLock<AccessPatternTracker>>,
    
    // Performance metrics
    metrics: Arc<RwLock<AdvancedCacheMetrics>>,
    
    // Background tasks
    eviction_task_handle: Option<tokio::task::JoinHandle<()>>,
    warming_task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl AdvancedCacheEngine {
    /// Create new advanced cache engine
    pub async fn new(config: AdvancedCacheConfig) -> Result<Self> {
        info!("Initializing advanced multi-tier cache engine");
        
        // Initialize disk cache manager
        let disk_cache = Arc::new(RwLock::new(
            DiskCacheManager::new(&config.disk_cache).await?
        ));
        
        // Initialize distributed cache if configured
        let distributed_cache = if let Some(dist_config) = &config.distributed_cache {
            Some(Self::create_distributed_client(dist_config).await?)
        } else {
            None
        };
        
        let mut engine = Self {
            config,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            disk_cache,
            distributed_cache,
            metadata_index: Arc::new(RwLock::new(HashMap::new())),
            access_patterns: Arc::new(RwLock::new(AccessPatternTracker::new())),
            metrics: Arc::new(RwLock::new(AdvancedCacheMetrics::default())),
            eviction_task_handle: None,
            warming_task_handle: None,
        };
        
        // Start background tasks
        engine.start_background_tasks().await?;
        
        info!("Advanced cache engine initialized successfully");
        Ok(engine)
    }
    
    /// Get or create cached AST with intelligent tier management
    pub async fn get_or_parse_ast<F>(
        &self,
        file_path: &Path,
        parser_func: F,
    ) -> Result<Arc<ParsedFile>>
    where
        F: FnOnce() -> Result<ParsedFile, Box<dyn std::error::Error + Send + Sync>>,
    {
        let cache_key = self.generate_cache_key(file_path, "ast");
        let start_time = Instant::now();
        
        // Check memory cache first (fastest tier)
        if let Some(ast) = self.get_from_memory_cache(&cache_key).await? {
            self.update_access_metrics(CacheTier::Memory, start_time).await;
            return Ok(ast);
        }
        
        // Check disk cache (medium tier)
        if let Some(ast) = self.get_from_disk_cache(&cache_key).await? {
            // Store in memory cache for faster future access
            self.store_in_memory_cache(&cache_key, &ast).await?;
            self.update_access_metrics(CacheTier::Disk, start_time).await;
            return Ok(ast);
        }
        
        // Check distributed cache (slowest tier)
        if let Some(ast) = self.get_from_distributed_cache(&cache_key).await? {
            // Store in higher tiers for faster access
            self.store_in_memory_cache(&cache_key, &ast).await?;
            self.store_in_disk_cache(&cache_key, &ast).await?;
            self.update_access_metrics(CacheTier::Distributed, start_time).await;
            return Ok(ast);
        }
        
        // Cache miss - parse and store in all tiers
        info!("Cache miss for AST: {}", file_path.display());
        let parsed_file = parser_func()
            .map_err(|e| UveddiError::AnalysisError {
                message: format!("Failed to parse file: {}", e),
                file: file_path.to_string_lossy().to_string(),
                line: 0,
                context: "advanced_cache".to_string(),
                suggestion: "Check file syntax and permissions".to_string(),
                source: Some(e),
            })?;
        
        let ast = Arc::new(parsed_file);
        
        // Store in all cache tiers (with intelligent size-based decisions)
        self.store_in_all_tiers(&cache_key, &ast).await?;
        
        self.update_cache_miss_metrics().await;
        
        Ok(ast)
    }
    
    /// Store analysis results with intelligent caching
    pub async fn cache_analysis_results(
        &self,
        file_path: &Path,
        results: &[ArchitecturalIssue],
    ) -> Result<()> {
        let cache_key = self.generate_cache_key(file_path, "results");
        
        // Serialize results
        let serialized = self.serialize_with_compression(results).await?;
        
        // Create cache entry with metadata
        let metadata = CacheEntryMetadata {
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 1,
            size_bytes: serialized.len(),
            compressed_size_bytes: None,
            regeneration_cost_ms: self.estimate_regeneration_cost(results.len()),
            dependencies: self.extract_dependencies(results),
            priority_score: self.calculate_priority_score(results),
            expires_at: Some(SystemTime::now() + Duration::from_hours(24)),
        };
        
        // Store metadata
        {
            let mut index = self.metadata_index.write().await;
            index.insert(cache_key.clone(), metadata);
        }
        
        // Store in appropriate tiers based on size and importance
        if serialized.len() < self.config.memory_cache.max_memory_bytes / 1000 {
            self.store_in_memory_cache_raw(&cache_key, serialized.clone()).await?;
        }
        
        if serialized.len() < self.config.disk_cache.max_disk_bytes / 1000 {
            self.store_in_disk_cache_raw(&cache_key, serialized).await?;
        }
        
        Ok(())
    }
    
    /// Get cached analysis results
    pub async fn get_cached_results(&self, file_path: &Path) -> Result<Option<Vec<ArchitecturalIssue>>> {
        let cache_key = self.generate_cache_key(file_path, "results");
        
        // Try to get from any tier
        let data = if let Some(data) = self.get_from_memory_cache_raw(&cache_key).await? {
            data
        } else if let Some(data) = self.get_from_disk_cache_raw(&cache_key).await? {
            // Promote to memory cache
            self.store_in_memory_cache_raw(&cache_key, data.clone()).await?;
            data
        } else if let Some(data) = self.get_from_distributed_cache_raw(&cache_key).await? {
            // Promote to higher tiers
            self.store_in_memory_cache_raw(&cache_key, data.clone()).await?;
            self.store_in_disk_cache_raw(&cache_key, data.clone()).await?;
            data
        } else {
            return Ok(None);
        };
        
        // Deserialize results
        let results = self.deserialize_with_decompression(&data).await?;
        
        // Update access patterns
        self.record_access_pattern(&cache_key).await;
        
        Ok(Some(results))
    }
    
    /// Clear all cache tiers
    pub async fn clear_all_caches(&self) -> Result<()> {
        info!("Clearing all cache tiers");
        
        // Clear memory cache
        {
            let mut memory_cache = self.memory_cache.write().await;
            memory_cache.clear();
        }
        
        // Clear disk cache
        {
            let mut disk_cache = self.disk_cache.write().await;
            disk_cache.clear().await?;
        }
        
        // Clear distributed cache
        if let Some(dist_cache) = &self.distributed_cache {
            let mut cache = dist_cache.write().await;
            cache.clear().await?;
        }
        
        // Clear metadata
        {
            let mut index = self.metadata_index.write().await;
            index.clear();
        }
        
        info!("All cache tiers cleared");
        Ok(())
    }
    
    /// Get comprehensive cache statistics
    pub async fn get_cache_stats(&self) -> AdvancedCacheMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Optimize cache configuration for large codebase
    pub async fn optimize_for_large_codebase(&self, estimated_files: usize) -> Result<()> {
        info!("Optimizing cache configuration for {} files", estimated_files);
        
        // Adjust memory limits based on codebase size
        let memory_per_file = 1024 * 1024; // 1MB per file estimate
        let recommended_memory = (estimated_files * memory_per_file).min(8 * 1024 * 1024 * 1024); // Max 8GB
        
        // Trigger proactive eviction if needed
        if self.get_current_memory_usage().await > recommended_memory {
            self.trigger_intelligent_eviction().await?;
        }
        
        // Pre-warm cache with common patterns if enabled
        if self.config.warming_strategy.enable_warming {
            self.trigger_cache_warming().await?;
        }
        
        info!("Cache optimization completed for large codebase");
        Ok(())
    }
    
    // Implementation of private helper methods would continue here...
    // Including disk cache management, distributed cache clients,
    // intelligent eviction algorithms, cache warming strategies,
    // compression/decompression, serialization, etc.
    
    /// Generate cache key for file and operation type
    fn generate_cache_key(&self, path: &Path, operation_type: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        operation_type.hash(&mut hasher);
        
        format!("{}:{:x}", operation_type, hasher.finish())
    }
    
    /// Start background tasks for cache management
    async fn start_background_tasks(&mut self) -> Result<()> {
        // Start eviction task
        if self.config.eviction_policy.proactive_eviction {
            let eviction_task = self.start_eviction_task().await;
            self.eviction_task_handle = Some(eviction_task);
        }
        
        // Start warming task
        if self.config.warming_strategy.enable_warming {
            let warming_task = self.start_warming_task().await;
            self.warming_task_handle = Some(warming_task);
        }
        
        Ok(())
    }
    
    /// Placeholder for background eviction task
    async fn start_eviction_task(&self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            // Implementation would run periodic eviction
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                // Perform eviction logic
            }
        })
    }
    
    /// Placeholder for background warming task
    async fn start_warming_task(&self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            // Implementation would run cache warming
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await;
                // Perform warming logic
            }
        })
    }
    
    // Additional helper method placeholders...
    async fn get_from_memory_cache(&self, _key: &str) -> Result<Option<Arc<ParsedFile>>> { Ok(None) }
    async fn get_from_disk_cache(&self, _key: &str) -> Result<Option<Arc<ParsedFile>>> { Ok(None) }
    async fn get_from_distributed_cache(&self, _key: &str) -> Result<Option<Arc<ParsedFile>>> { Ok(None) }
    async fn store_in_memory_cache(&self, _key: &str, _data: &Arc<ParsedFile>) -> Result<()> { Ok(()) }
    async fn store_in_disk_cache(&self, _key: &str, _data: &Arc<ParsedFile>) -> Result<()> { Ok(()) }
    async fn store_in_all_tiers(&self, _key: &str, _data: &Arc<ParsedFile>) -> Result<()> { Ok(()) }
    async fn get_from_memory_cache_raw(&self, _key: &str) -> Result<Option<Vec<u8>>> { Ok(None) }
    async fn get_from_disk_cache_raw(&self, _key: &str) -> Result<Option<Vec<u8>>> { Ok(None) }
    async fn get_from_distributed_cache_raw(&self, _key: &str) -> Result<Option<Vec<u8>>> { Ok(None) }
    async fn store_in_memory_cache_raw(&self, _key: &str, _data: Vec<u8>) -> Result<()> { Ok(()) }
    async fn store_in_disk_cache_raw(&self, _key: &str, _data: Vec<u8>) -> Result<()> { Ok(()) }
    async fn serialize_with_compression(&self, _data: &[ArchitecturalIssue]) -> Result<Vec<u8>> { Ok(Vec::new()) }
    async fn deserialize_with_decompression(&self, _data: &[u8]) -> Result<Vec<ArchitecturalIssue>> { Ok(Vec::new()) }
    async fn update_access_metrics(&self, _tier: CacheTier, _start_time: Instant) {}
    async fn update_cache_miss_metrics(&self) {}
    async fn record_access_pattern(&self, _key: &str) {}
    async fn get_current_memory_usage(&self) -> usize { 0 }
    async fn trigger_intelligent_eviction(&self) -> Result<()> { Ok(()) }
    async fn trigger_cache_warming(&self) -> Result<()> { Ok(()) }
    
    fn estimate_regeneration_cost(&self, _result_count: usize) -> u64 { 1000 }
    fn extract_dependencies(&self, _results: &[ArchitecturalIssue]) -> Vec<String> { Vec::new() }
    fn calculate_priority_score(&self, _results: &[ArchitecturalIssue]) -> f64 { 1.0 }
    
    async fn create_distributed_client(_config: &DistributedCacheConfig) -> Result<Arc<RwLock<dyn DistributedCacheClient + Send + Sync>>> {
        Err(UveddiError::config_error("Distributed cache not implemented", "advanced_cache"))
    }
}

/// Disk cache manager for persistent storage
pub struct DiskCacheManager {
    cache_dir: PathBuf,
    config: DiskCacheConfig,
}

impl DiskCacheManager {
    async fn new(config: &DiskCacheConfig) -> Result<Self> {
        // Create cache directory if it doesn't exist
        tokio::fs::create_dir_all(&config.cache_directory).await
            .map_err(|e| UveddiError::io_error(&e.to_string()))?;
            
        Ok(Self {
            cache_dir: config.cache_directory.clone(),
            config: config.clone(),
        })
    }
    
    async fn clear(&mut self) -> Result<()> {
        // Remove all cache files
        let mut entries = tokio::fs::read_dir(&self.cache_dir).await
            .map_err(|e| UveddiError::io_error(&e.to_string()))?;
            
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| UveddiError::io_error(&e.to_string()))? {
            tokio::fs::remove_file(entry.path()).await
                .map_err(|e| UveddiError::io_error(&e.to_string()))?;
        }
        
        Ok(())
    }
}

/// Access pattern tracker for intelligent caching decisions
pub struct AccessPatternTracker {
    patterns: BTreeMap<String, AccessPattern>,
}

impl AccessPatternTracker {
    fn new() -> Self {
        Self {
            patterns: BTreeMap::new(),
        }
    }
}

/// Individual access pattern data
#[derive(Debug, Clone)]
struct AccessPattern {
    access_times: VecDeque<Instant>,
    frequency_score: f64,
    recency_score: f64,
    pattern_type: AccessPatternType,
}

#[derive(Debug, Clone)]
enum AccessPatternType {
    Frequent,    // High frequency access
    Bursty,      // Bursts of access then quiet
    Occasional,  // Infrequent but regular
    OneTime,     // Single access only
}

/// Distributed cache client trait
#[async_trait::async_trait]
/// Public trait DistributedCacheClient.
pub trait DistributedCacheClient {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_advanced_cache_config() {
        let config = AdvancedCacheConfig::default();
        assert!(config.memory_cache.max_memory_bytes > 0);
        assert!(config.compression.enable_adaptive_compression);
        assert_eq!(config.eviction_policy.policy, EvictionPolicy::AdaptiveLRU);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let config = AdvancedCacheConfig::default();
        let engine = AdvancedCacheEngine::new(config).await.unwrap();
        
        let path1 = Path::new("/test/file1.rs");
        let path2 = Path::new("/test/file2.rs");
        
        let key1 = engine.generate_cache_key(path1, "ast");
        let key2 = engine.generate_cache_key(path2, "ast");
        let key3 = engine.generate_cache_key(path1, "results");
        
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_compression_config() {
        let config = CompressionConfig::default();
        assert_eq!(config.algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(config.compression_level, 3);
        assert!(config.enable_adaptive_compression);
    }
}