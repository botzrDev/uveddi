//! Core Diagram Cache Engine Implementation
//!
//! Provides the main caching engine that orchestrates diagram storage, retrieval,
//! and invalidation using Phase 2's incremental analysis capabilities.

use super::{
    CacheWarmingStrategy, CachedDiagram, CompressionEngine, DiagramCacheConfig, DiagramCacheError,
    DiagramCacheStats, DiagramDependencyTracker, DiagramType, InvalidationManager, Result,
};
use crate::analysis::incremental::{ChangeSet, IncrementalAnalysisEngine};
use blake3::Hasher;
use chrono::Utc;
use crate::core::logging::{debug, error, info, warn};
use lru::LruCache;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Main diagram cache engine
pub struct DiagramCacheEngine {
    /// Cache storage by diagram type
    cache_layers: HashMap<DiagramType, Arc<RwLock<LruCache<String, CachedDiagram>>>>,

    /// Dependency tracker for change impact analysis
    dependency_tracker: DiagramDependencyTracker,

    /// Cache invalidation manager
    invalidation_manager: InvalidationManager,

    /// Compression engine for diagram storage
    compression_engine: CompressionEngine,

    /// Cache configuration
    config: DiagramCacheConfig,

    /// Cache statistics
    stats: Arc<RwLock<DiagramCacheStats>>,

    /// Integration with incremental analysis
    incremental_engine: Option<Arc<RwLock<IncrementalAnalysisEngine>>>,
}

impl DiagramCacheEngine {
    /// Creates a new diagram cache engine
    pub fn new(config: DiagramCacheConfig) -> Self {
        let mut cache_layers = HashMap::new();

        // Initialize cache layers for each diagram type
        for diagram_type in [
            DiagramType::Mermaid,
            DiagramType::PlantUML,
            DiagramType::Graphviz,
        ] {
            let cache_size = std::num::NonZero::new(config.max_cached_diagrams / 3)
                .unwrap_or(std::num::NonZero::new(1000).unwrap());
            cache_layers.insert(
                diagram_type,
                Arc::new(RwLock::new(LruCache::new(cache_size))),
            );
        }

        Self {
            cache_layers,
            dependency_tracker: DiagramDependencyTracker::new(),
            invalidation_manager: InvalidationManager::new(config.invalidation_strategy.clone()),
            compression_engine: CompressionEngine::new(config.compression_level),
            config,
            stats: Arc::new(RwLock::new(DiagramCacheStats::default())),
            incremental_engine: None,
        }
    }

    /// Sets the incremental analysis engine for integration
    pub fn set_incremental_engine(&mut self, engine: Arc<RwLock<IncrementalAnalysisEngine>>) {
        self.incremental_engine = Some(engine);
    }

    /// Retrieves a diagram from cache
    pub async fn get_diagram(
        &self,
        diagram_id: &str,
        diagram_type: &DiagramType,
    ) -> Result<Option<Vec<u8>>> {
        let start_time = Instant::now();

        if let Some(cache_layer) = self.cache_layers.get(diagram_type) {
            let mut cache = cache_layer.write().await;

            if let Some(cached_diagram) = cache.get_mut(diagram_id) {
                // Update access statistics
                cached_diagram.last_accessed = Utc::now();
                cached_diagram.access_count += 1;

                // Decompress content
                let content = self
                    .compression_engine
                    .decompress(&cached_diagram.content, diagram_type)
                    .await?;

                // Record cache hit
                let lookup_time = start_time.elapsed().as_millis() as f64;
                let mut stats = self.stats.write().await;
                stats.record_hit(lookup_time);

                debug!(
                    "Cache hit for diagram: {} (type: {:?})",
                    diagram_id, diagram_type
                );
                return Ok(Some(content));
            }
        }

        // Record cache miss
        let lookup_time = start_time.elapsed().as_millis() as f64;
        let mut stats = self.stats.write().await;
        stats.record_miss(lookup_time);

        debug!(
            "Cache miss for diagram: {} (type: {:?})",
            diagram_id, diagram_type
        );
        Ok(None)
    }

    /// Stores a diagram in cache
    pub async fn store_diagram(
        &self,
        diagram_id: String,
        diagram_type: DiagramType,
        content: Vec<u8>,
        dependencies: Vec<PathBuf>,
        config_hash: String,
    ) -> Result<()> {
        // Check cache capacity
        if !self.has_capacity(&diagram_type).await {
            return Err(DiagramCacheError::CacheFull);
        }

        // Compress content
        let compressed_content = self
            .compression_engine
            .compress(&content, &diagram_type)
            .await?;
        let original_size = content.len();
        let compressed_size = compressed_content.len();

        // Create content hash
        let content_hash = self.create_content_hash(&content);

        // Create cached diagram entry
        let cached_diagram = CachedDiagram {
            id: diagram_id.clone(),
            diagram_type: diagram_type.clone(),
            content: compressed_content,
            original_size,
            compressed_size,
            content_hash,
            dependencies: dependencies.iter().cloned().collect(),
            generated_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            config_hash,
        };

        // Store in appropriate cache layer
        if let Some(cache_layer) = self.cache_layers.get(&diagram_type) {
            let mut cache = cache_layer.write().await;
            cache.put(diagram_id.clone(), cached_diagram.clone());
        }

        // Update dependency tracking
        self.dependency_tracker
            .add_diagram_dependencies(&diagram_id, &diagram_type, &dependencies)
            .await?;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.cached_diagrams += 1;
        stats.memory_usage_bytes += compressed_size;
        stats.compression_savings_bytes += original_size.saturating_sub(compressed_size);

        info!(
            "Stored diagram in cache: {} (type: {:?}, compression: {:.1}%)",
            diagram_id,
            diagram_type,
            (1.0 - compressed_size as f64 / original_size as f64) * 100.0
        );

        Ok(())
    }

    /// Invalidates cache entries based on file changes from incremental analysis
    pub async fn invalidate_from_changeset(&self, changeset: &ChangeSet) -> Result<()> {
        let start_time = Instant::now();

        // Get affected diagrams from dependency tracker
        let affected_diagrams = self
            .dependency_tracker
            .get_affected_diagrams(changeset)
            .await?;

        let mut total_invalidated = 0;

        // Invalidate affected diagrams across all cache layers
        for (diagram_type, cache_layer) in &self.cache_layers {
            let mut cache = cache_layer.write().await;
            let mut to_remove = Vec::new();

            for (diagram_id, _) in cache.iter() {
                if affected_diagrams.contains(diagram_id) {
                    to_remove.push(diagram_id.clone());
                }
            }

            for diagram_id in to_remove {
                if cache.pop(&diagram_id).is_some() {
                    total_invalidated += 1;
                    debug!(
                        "Invalidated diagram: {} (type: {:?})",
                        diagram_id, diagram_type
                    );
                }
            }
        }

        // Update dependency tracker
        self.dependency_tracker
            .remove_affected_dependencies(&affected_diagrams)
            .await?;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.invalidations += total_invalidated;
        stats.cached_diagrams = stats
            .cached_diagrams
            .saturating_sub(total_invalidated as usize);

        let invalidation_time = start_time.elapsed().as_millis();
        info!(
            "Invalidated {} diagrams in {}ms based on changeset",
            total_invalidated, invalidation_time
        );

        Ok(())
    }

    /// Performs selective regeneration of affected diagrams
    pub async fn selective_regeneration(&self, changeset: &ChangeSet) -> Result<Vec<String>> {
        if !self.config.enable_selective_regeneration {
            return Ok(Vec::new());
        }

        // Get diagrams that need regeneration
        let diagrams_to_regenerate = self
            .dependency_tracker
            .get_diagrams_for_regeneration(changeset)
            .await?;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.selective_regenerations += diagrams_to_regenerate.len() as u64;

        info!(
            "Identified {} diagrams for selective regeneration",
            diagrams_to_regenerate.len()
        );
        Ok(diagrams_to_regenerate)
    }

    /// Warms the cache based on configured strategy
    pub async fn warm_cache(&self, root_path: &Path) -> Result<()> {
        match self.config.warming_strategy {
            CacheWarmingStrategy::None => Ok(()),
            CacheWarmingStrategy::AccessBased => self.warm_cache_access_based().await,
            CacheWarmingStrategy::DependencyBased => {
                self.warm_cache_dependency_based(root_path).await
            }
            CacheWarmingStrategy::Adaptive => self.warm_cache_adaptive(root_path).await,
        }
    }

    /// Gets current cache statistics
    pub async fn get_statistics(&self) -> DiagramCacheStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Clears all cached diagrams
    pub async fn clear_cache(&self) -> Result<()> {
        for cache_layer in self.cache_layers.values() {
            let mut cache = cache_layer.write().await;
            cache.clear();
        }

        self.dependency_tracker.clear().await?;

        let mut stats = self.stats.write().await;
        *stats = DiagramCacheStats::default();

        info!("Cleared all diagram caches");
        Ok(())
    }

    /// Checks if cache has capacity for new entries
    async fn has_capacity(&self, diagram_type: &DiagramType) -> bool {
        if let Some(cache_layer) = self.cache_layers.get(diagram_type) {
            let cache = cache_layer.read().await;
            cache.len() < cache.cap().get()
        } else {
            false
        }
    }

    /// Creates content hash for validation
    fn create_content_hash(&self, content: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(content);
        hasher.finalize().to_hex().to_string()
    }

    /// Warms cache based on access patterns
    async fn warm_cache_access_based(&self) -> Result<()> {
        // Implementation for access-based cache warming
        debug!("Performing access-based cache warming");
        Ok(())
    }

    /// Warms cache based on dependency analysis
    async fn warm_cache_dependency_based(&self, _root_path: &Path) -> Result<()> {
        // Implementation for dependency-based cache warming
        debug!("Performing dependency-based cache warming");
        Ok(())
    }

    /// Adaptive cache warming strategy
    async fn warm_cache_adaptive(&self, _root_path: &Path) -> Result<()> {
        // Implementation for adaptive cache warming
        debug!("Performing adaptive cache warming");
        Ok(())
    }
}

impl Default for DiagramCacheEngine {
    fn default() -> Self {
        Self::new(DiagramCacheConfig::default())
    }
}
