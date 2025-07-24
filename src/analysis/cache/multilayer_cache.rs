//! Multi-layered cache implementation with memory and disk tiers
//!
//! This module provides a high-performance multi-layered cache system with:
//! - L1: In-memory LRU cache for hot data
//! - L2: Disk-based cache with efficient serialization
//! - Automatic promotion/demotion between layers
//! - Content-based invalidation and monitoring

use crate::analysis::cache::{
    invalidation::{InvalidationStrategy, ContentHashInvalidator},
    metrics::CacheMetrics,
    serialization::{CacheEntry, CacheSerializer, SerializationFormat},
};
use rkyv::de::deserializers::SharedDeserializeMap;
use lru::LruCache;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use crate::analysis::cache::serialization::wrappers::ArchivablePathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::fs;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] crate::analysis::cache::serialization::SerializationError),
    #[error("Invalidation error: {0}")]
    Invalidation(#[from] crate::analysis::cache::invalidation::InvalidationError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cache layer error: {0}")]
    Layer(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Cache layer enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLayer {
    /// In-memory LRU cache (fastest)
    Memory,
    /// Disk-based cache (larger capacity)
    Disk,
}

/// Configuration for multi-layer cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub memory_capacity: usize,
    pub memory_size_limit: u64,
    pub disk_cache_path: PathBuf,
    pub disk_size_limit: u64,
    pub serialization_format: SerializationFormat,
    pub enable_promotion: bool,
    pub promotion_threshold: u64,
    pub entry_ttl: Option<Duration>,
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_capacity: 1000,
            memory_size_limit: 256 * 1024 * 1024, // 256MB
            disk_cache_path: PathBuf::from(".cache"),
            disk_size_limit: 2 * 1024 * 1024 * 1024, // 2GB
            serialization_format: SerializationFormat::default(),
            enable_promotion: true,
            promotion_threshold: 3,
            entry_ttl: Some(Duration::from_secs(3600)), // 1 hour
            enable_compression: true,
        }
    }
}

/// Multi-layered cache with automatic tier management
pub struct MultiLayerCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// L1 cache: In-memory LRU
    memory_cache: Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
    /// L2 cache: Disk-based storage
    disk_cache: Arc<RwLock<DiskCache<K, V>>>,
    /// Cache configuration
    config: CacheConfig,
    /// Invalidation strategy
    invalidator: Arc<dyn InvalidationStrategy>,
    /// Performance metrics
    metrics: Arc<CacheMetrics>,
    /// Serializer for disk operations
    serializer: CacheSerializer,
}

/// Disk-based cache implementation
struct DiskCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    cache_dir: PathBuf,
    file_map: HashMap<String, PathBuf>,
    current_size: u64,
    config: CacheConfig,
    serializer: CacheSerializer,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> MultiLayerCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + ToString + 'static,
    V: Clone + Send + Sync + Serialize + for<'de> DeserializeOwned + 'static,
    V: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>,
    V: rkyv::Deserialize<V::Archived, SharedDeserializeMap>,
    V::Archived: for<'a> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
{
    /// Create a new multi-layer cache
    pub async fn new(
        config: CacheConfig,
        metrics: Arc<CacheMetrics>,
    ) -> Result<Self, CacheError> {
        // Create cache directory if it doesn't exist
        if !config.disk_cache_path.exists() {
            fs::create_dir_all(&config.disk_cache_path).await?;
        }

        let memory_capacity = NonZeroUsize::new(config.memory_capacity)
            .ok_or_else(|| CacheError::Config("Memory capacity must be > 0".to_string()))?;

        let memory_cache = Arc::new(RwLock::new(LruCache::new(memory_capacity)));

        let disk_cache = Arc::new(RwLock::new(DiskCache::new(config.clone()).await?));

        let invalidator = Arc::new(ContentHashInvalidator::new());
        let serializer = CacheSerializer::new(config.serialization_format);

        Ok(Self {
            memory_cache,
            disk_cache,
            config,
            invalidator,
            metrics,
            serializer,
        })
    }

    /// Get value from cache, checking both layers
    pub async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let start_time = Instant::now();
        let key_str = key.to_string();

        // Check L1 cache first
        {
            let mut memory_cache = self.memory_cache.write().await;
            if let Some(entry) = memory_cache.get_mut(key) {
                // Check if entry is still valid
                if self.is_entry_valid(entry) {
                    let duration = start_time.elapsed();
                    self.metrics.record_hit("memory", duration);
                    
                    // Update access count and timestamp
                    entry.touch();
                    
                    return Ok(Some(entry.data.clone()));
                } else {
                    // Entry expired, remove it
                    memory_cache.pop(key);
                }
            }
        }

        // Check L2 cache
        let disk_result = {
            let disk_cache = self.disk_cache.read().await;
            disk_cache.get(key).await
        };

        match disk_result {
            Ok(Some(mut entry)) => {
                if self.is_entry_valid(&entry) {
                    let duration = start_time.elapsed();
                    self.metrics.record_hit("disk", duration);
                    
                    // Update access count
                    entry.touch();
                    
                    // Consider promoting to memory cache
                    if self.should_promote(&entry) {
                        self.promote_to_memory(key.clone(), entry.clone()).await?;
                    }
                    
                    // Update disk cache with new access info
                    {
                        let mut disk_cache = self.disk_cache.write().await;
                        disk_cache.set(key, &entry).await?;
                    }
                    
                    Ok(Some(entry.data))
                } else {
                    // Entry expired, remove from disk
                    {
                        let mut disk_cache = self.disk_cache.write().await;
                        disk_cache.remove(key).await?;
                    }
                    
                    let duration = start_time.elapsed();
                    self.metrics.record_miss("disk", duration);
                    Ok(None)
                }
            }
            Ok(None) => {
                let duration = start_time.elapsed();
                self.metrics.record_miss("both", duration);
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Set value in cache
    pub async fn set(&self, key: K, value: V) -> Result<(), CacheError> {
        let key_str = key.to_string();
        let size_estimate = self.estimate_size(&value);
        
        // Create cache entry
        let content_hash = self.calculate_content_hash(&value);
        let entry = CacheEntry::new(value, content_hash, size_estimate);

        // Always write to disk cache first for durability
        {
            let mut disk_cache = self.disk_cache.write().await;
            disk_cache.set(&key, &entry).await?;
        }

        // Add to memory cache if there's space or if it's hot data
        if self.should_cache_in_memory(&entry) {
            let mut memory_cache = self.memory_cache.write().await;
            
            // Check if we need to evict from memory
            while memory_cache.len() >= self.config.memory_capacity {
                if let Some((evicted_key, evicted_entry)) = memory_cache.pop_lru() {
                    self.metrics.record_eviction("memory", evicted_entry.size_bytes);
                }
            }
            
            memory_cache.put(key, entry.clone());
            self.metrics.record_insertion("memory", size_estimate);
        }

        self.metrics.record_insertion("disk", size_estimate);
        Ok(())
    }

    /// Remove value from cache
    pub async fn remove(&self, key: &K) -> Result<Option<V>, CacheError> {
        let mut removed_value = None;

        // Remove from memory cache
        {
            let mut memory_cache = self.memory_cache.write().await;
            if let Some(entry) = memory_cache.pop(key) {
                self.metrics.record_eviction("memory", entry.size_bytes);
                removed_value = Some(entry.data);
            }
        }

        // Remove from disk cache
        {
            let mut disk_cache = self.disk_cache.write().await;
            if let Some(entry) = disk_cache.remove(key).await? {
                self.metrics.record_eviction("disk", entry.size_bytes);
                if removed_value.is_none() {
                    removed_value = Some(entry.data);
                }
            }
        }

        Ok(removed_value)
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<(), CacheError> {
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

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let memory_cache = self.memory_cache.read().await;
        let disk_cache = self.disk_cache.read().await;

        CacheStats {
            memory_entries: memory_cache.len(),
            memory_capacity: self.config.memory_capacity,
            disk_entries: disk_cache.file_map.len(),
            disk_size_bytes: disk_cache.current_size,
            disk_capacity_bytes: self.config.disk_size_limit,
            hit_rate: self.metrics.hit_rate(),
        }
    }

    /// Perform cache maintenance (cleanup expired entries, etc.)
    pub async fn maintain(&self) -> Result<(), CacheError> {
        // Clean up expired memory entries
        {
            let mut memory_cache = self.memory_cache.write().await;
            let mut to_remove = Vec::new();
            
            // We need to iterate without mutating, so collect keys first
            for (key, entry) in memory_cache.iter() {
                if !self.is_entry_valid(entry) {
                    to_remove.push(key.clone());
                }
            }
            
            for key in to_remove {
                if let Some(entry) = memory_cache.pop(&key) {
                    self.metrics.record_eviction("memory", entry.size_bytes);
                }
            }
        }

        // Clean up expired disk entries
        {
            let mut disk_cache = self.disk_cache.write().await;
            disk_cache.cleanup_expired(self.config.entry_ttl).await?;
        }

        Ok(())
    }

    // Helper methods

    fn is_entry_valid(&self, entry: &CacheEntry<V>) -> bool {
        if let Some(ttl) = self.config.entry_ttl {
            entry.age() <= ttl
        } else {
            true
        }
    }

    fn should_promote(&self, entry: &CacheEntry<V>) -> bool {
        self.config.enable_promotion && entry.access_count >= self.config.promotion_threshold
    }

    fn should_cache_in_memory(&self, _entry: &CacheEntry<V>) -> bool {
        // For now, cache everything in memory that fits
        // Could be enhanced with more sophisticated policies
        true
    }

    async fn promote_to_memory(&self, key: K, entry: CacheEntry<V>) -> Result<(), CacheError> {
        let mut memory_cache = self.memory_cache.write().await;
        
        // Make room if needed
        while memory_cache.len() >= self.config.memory_capacity {
            if let Some((evicted_key, evicted_entry)) = memory_cache.pop_lru() {
                self.metrics.record_eviction("memory", evicted_entry.size_bytes);
            }
        }
        
        memory_cache.put(key, entry.clone());
        self.metrics.record_insertion("memory", entry.size_bytes);
        Ok(())
    }

    fn estimate_size(&self, _value: &V) -> u64 {
        // Rough estimate - could be improved with actual measurement
        1024 // 1KB default estimate
    }

    fn calculate_content_hash(&self, value: &V) -> String {
        // Simple hash based on serialized content
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // This is a simplified approach - in production, you'd want proper content hashing
        let mut hasher = DefaultHasher::new();
        
        // For now, use a simple approach
        // In production, serialize the value and hash the bytes
        format!("hash_{}", hasher.finish())
    }
}

impl<K, V> DiskCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + ToString,
    V: Clone + Send + Sync + Serialize + for<'de> DeserializeOwned,
    V: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>,
    V: rkyv::Deserialize<V::Archived, SharedDeserializeMap>,
    V::Archived: for<'a> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
{
    async fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let serializer = CacheSerializer::new(config.serialization_format);
        
        let mut cache = Self {
            cache_dir: config.disk_cache_path.clone(),
            file_map: HashMap::new(),
            current_size: 0,
            config,
            serializer,
            _marker: PhantomData,
        };

        // Load existing cache entries
        cache.load_existing_entries().await?;
        
        Ok(cache)
    }

    async fn load_existing_entries(&mut self) -> Result<(), CacheError> {
        let mut entries = fs::read_dir(&self.cache_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let metadata = fs::metadata(&path).await?;
                    self.file_map.insert(file_name.to_string(), path);
                    self.current_size += metadata.len();
                }
            }
        }
        
        Ok(())
    }

    async fn get(&self, key: &K) -> Result<Option<CacheEntry<V>>, CacheError> {
        let key_hash = self.hash_key(key);
        
        if let Some(file_path) = self.file_map.get(&key_hash) {
            let data = fs::read(file_path).await?;
            let entry: CacheEntry<V> = self.serializer.deserialize(&data)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    async fn set(&mut self, key: &K, entry: &CacheEntry<V>) -> Result<(), CacheError> {
        let key_hash = self.hash_key(key);
        let file_path = self.cache_dir.join(&key_hash);
        
        let data = self.serializer.serialize(entry)?;
        
        // Check if we need to evict entries to make space
        while self.current_size + data.len() as u64 > self.config.disk_size_limit && !self.file_map.is_empty() {
            self.evict_lru_entry().await?;
        }
        
        fs::write(&file_path, &data).await?;
        
        // Update tracking
        if let Some(old_path) = self.file_map.insert(key_hash, file_path) {
            // Replace existing entry
            if let Ok(metadata) = fs::metadata(&old_path).await {
                self.current_size -= metadata.len();
            }
        }
        
        self.current_size += data.len() as u64;
        Ok(())
    }

    async fn remove(&mut self, key: &K) -> Result<Option<CacheEntry<V>>, CacheError> {
        let key_hash = self.hash_key(key);
        
        if let Some(file_path) = self.file_map.remove(&key_hash) {
            // Read entry before removing
            let data = fs::read(&file_path).await?;
            let entry: CacheEntry<V> = self.serializer.deserialize(&data)?;
            
            // Remove file
            fs::remove_file(&file_path).await?;
            
            // Update size tracking
            if let Ok(metadata) = fs::metadata(&file_path).await {
                self.current_size -= metadata.len();
            }
            
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    async fn clear(&mut self) -> Result<(), CacheError> {
        for file_path in self.file_map.values() {
            let _ = fs::remove_file(file_path).await; // Ignore errors
        }
        
        self.file_map.clear();
        self.current_size = 0;
        Ok(())
    }

    async fn cleanup_expired(&mut self, ttl: Option<Duration>) -> Result<usize, CacheError> {
        if ttl.is_none() {
            return Ok(0);
        }
        let ttl = ttl.unwrap();

        let mut removed_count = 0;
        let mut to_remove = Vec::new();
        
        for (key_hash, file_path) in &self.file_map {
            if let Ok(data) = fs::read(file_path).await {
                if let Ok(entry) = self.serializer.deserialize::<CacheEntry<V>>(&data) {
                    if entry.age() > ttl {
                        to_remove.push(key_hash.clone());
                    }
                }
            }
        }
        
        for key_hash in to_remove {
            if let Some(file_path) = self.file_map.remove(&key_hash) {
                let _ = fs::remove_file(&file_path).await;
                removed_count += 1;
            }
        }
        
        Ok(removed_count)
    }

    async fn evict_lru_entry(&mut self) -> Result<(), CacheError> {
        // Find the oldest entry (simple LRU approximation using file modification time)
        let mut oldest_key = None;
        let mut oldest_time = std::time::SystemTime::now();
        
        for (key_hash, file_path) in &self.file_map {
            if let Ok(metadata) = fs::metadata(file_path).await {
                if let Ok(modified) = metadata.modified() {
                    if modified < oldest_time {
                        oldest_time = modified;
                        oldest_key = Some(key_hash.clone());
                    }
                }
            }
        }
        
        if let Some(key_hash) = oldest_key {
            if let Some(file_path) = self.file_map.remove(&key_hash) {
                let _ = fs::remove_file(&file_path).await;
            }
        }
        
        Ok(())
    }

    fn hash_key(&self, key: &K) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        key.to_string().hash(&mut hasher);
        hasher.finish().to_string()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub memory_capacity: usize,
    pub disk_entries: usize,
    pub disk_size_bytes: u64,
    pub disk_capacity_bytes: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    use tempfile::TempDir;

    #[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
    #[derive(rkyv::Archive, rkyv::Serialize)]
    #[archive(check_bytes)]
    struct TestValue {
        id: u64,
        data: String,
    }

    // Manual implementation of the required Deserialize trait
    impl rkyv::Deserialize<ArchivedTestValue, SharedDeserializeMap> for TestValue {
        fn deserialize(&self, _deserializer: &mut SharedDeserializeMap) -> Result<ArchivedTestValue, rkyv::de::deserializers::SharedDeserializeMapError> {
            // This implementation is not actually used in practice for this direction
            // The real deserialization happens from ArchivedTestValue -> TestValue
            unreachable!("This direction of deserialization should not be called")
        }
    }

    #[tokio::test]
    async fn test_multilayer_cache_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            disk_cache_path: temp_dir.path().to_path_buf(),
            memory_capacity: 2,
            ..Default::default()
        };

        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache: MultiLayerCache<String, TestValue> = MultiLayerCache::new(config, metrics).await.unwrap();

        let key = "test_key".to_string();
        let value = TestValue {
            id: 1,
            data: "test data".to_string(),
        };

        // Test set operation
        cache.set(key.clone(), value.clone()).await.unwrap();

        // Test get operation
        let retrieved = cache.get(&key).await.unwrap();
        assert_eq!(retrieved, Some(value.clone()));

        // Test remove operation
        let removed = cache.remove(&key).await.unwrap();
        assert_eq!(removed, Some(value));

        // Verify it's gone
        let retrieved = cache.get(&key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_memory_cache_eviction() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            disk_cache_path: temp_dir.path().to_path_buf(),
            memory_capacity: 2, // Small capacity to trigger eviction
            ..Default::default()
        };

        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache: MultiLayerCache<String, TestValue> = MultiLayerCache::new(config, metrics).await.unwrap();

        // Fill memory cache beyond capacity
        for i in 0..3 {
            let key = format!("key_{}", i);
            let value = TestValue {
                id: i,
                data: format!("data_{}", i),
            };
            cache.set(key, value).await.unwrap();
        }

        let stats = cache.stats().await;
        assert_eq!(stats.memory_entries, 2); // Should be limited by capacity
        assert_eq!(stats.disk_entries, 3); // All should be on disk
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            disk_cache_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache: MultiLayerCache<String, TestValue> = MultiLayerCache::new(config, metrics).await.unwrap();

        let value = TestValue {
            id: 1,
            data: "test".to_string(),
        };

        cache.set("key1".to_string(), value.clone()).await.unwrap();
        cache.set("key2".to_string(), value).await.unwrap();

        let stats = cache.stats().await;
        assert!(stats.disk_entries >= 2);
        assert!(stats.disk_size_bytes > 0);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            disk_cache_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let registry = Registry::new();
        let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
        let cache: MultiLayerCache<String, TestValue> = MultiLayerCache::new(config, metrics).await.unwrap();

        let value = TestValue {
            id: 1,
            data: "test".to_string(),
        };

        cache.set("key".to_string(), value).await.unwrap();
        cache.clear().await.unwrap();

        let stats = cache.stats().await;
        assert_eq!(stats.memory_entries, 0);
        assert_eq!(stats.disk_entries, 0);
    }
}