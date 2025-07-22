//! Advanced Diagram Caching System for UV-91 Phase 3
//!
//! Implements intelligent diagram caching leveraging Phase 2's incremental analysis
//! to achieve 90%+ cache hit rates and dramatically reduce diagram generation overhead.
//!
//! Key Features:
//! - Multi-layer diagram cache architecture
//! - Change-aware cache invalidation using incremental analysis
//! - Selective diagram regeneration based on dependency tracking
//! - Intelligent compression with 60%+ storage reduction
//! - Performance-optimized cache operations (<50ms lookup)

pub mod cache_engine;
pub mod compression;
pub mod diagram_tracker;
pub mod invalidation_manager;

pub use cache_engine::DiagramCacheEngine;
pub use compression::{CompressionEngine, CompressionLevel};
pub use diagram_tracker::{DiagramDependency, DiagramDependencyTracker};
pub use invalidation_manager::{InvalidationManager, InvalidationStrategy};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Types of diagrams supported by the cache system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramType {
    /// Mermaid diagrams (flowcharts, sequence, etc.)
    Mermaid,
    /// PlantUML diagrams
    PlantUML,
    /// Graphviz/DOT diagrams
    Graphviz,
    /// Custom diagram types
    Custom(String),
}

/// Represents a cached diagram entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDiagram {
    /// Unique identifier for the diagram
    pub id: String,

    /// Type of diagram
    pub diagram_type: DiagramType,

    /// Diagram content (compressed)
    pub content: Vec<u8>,

    /// Original content size before compression
    pub original_size: usize,

    /// Compressed content size
    pub compressed_size: usize,

    /// Content hash for validation
    pub content_hash: String,

    /// Files this diagram depends on
    pub dependencies: HashSet<PathBuf>,

    /// When this diagram was generated
    pub generated_at: DateTime<Utc>,

    /// When this diagram was last accessed
    pub last_accessed: DateTime<Utc>,

    /// Number of times this diagram has been accessed
    pub access_count: u64,

    /// Diagram generation parameters/config hash
    pub config_hash: String,
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramCacheStats {
    /// Total number of cache requests
    pub total_requests: u64,

    /// Number of cache hits
    pub cache_hits: u64,

    /// Number of cache misses
    pub cache_misses: u64,

    /// Cache hit rate percentage
    pub hit_rate: f64,

    /// Total diagrams in cache
    pub cached_diagrams: usize,

    /// Total memory usage in bytes
    pub memory_usage_bytes: usize,

    /// Total storage saved through compression
    pub compression_savings_bytes: usize,

    /// Average cache lookup time in milliseconds
    pub avg_lookup_time_ms: f64,

    /// Number of invalidations performed
    pub invalidations: u64,

    /// Number of selective regenerations
    pub selective_regenerations: u64,
}

impl Default for DiagramCacheStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            hit_rate: 0.0,
            cached_diagrams: 0,
            memory_usage_bytes: 0,
            compression_savings_bytes: 0,
            avg_lookup_time_ms: 0.0,
            invalidations: 0,
            selective_regenerations: 0,
        }
    }
}

impl DiagramCacheStats {
    /// Updates hit rate based on current hits and misses
    pub fn update_hit_rate(&mut self) {
        if self.total_requests > 0 {
            self.hit_rate = (self.cache_hits as f64 / self.total_requests as f64) * 100.0;
        }
    }

    /// Records a cache hit
    pub fn record_hit(&mut self, lookup_time_ms: f64) {
        self.total_requests += 1;
        self.cache_hits += 1;
        self.update_hit_rate();
        self.update_avg_lookup_time(lookup_time_ms);
    }

    /// Records a cache miss
    pub fn record_miss(&mut self, lookup_time_ms: f64) {
        self.total_requests += 1;
        self.cache_misses += 1;
        self.update_hit_rate();
        self.update_avg_lookup_time(lookup_time_ms);
    }

    /// Updates average lookup time
    fn update_avg_lookup_time(&mut self, lookup_time_ms: f64) {
        let total_time = self.avg_lookup_time_ms * (self.total_requests - 1) as f64;
        self.avg_lookup_time_ms = (total_time + lookup_time_ms) / self.total_requests as f64;
    }
}

/// Configuration for diagram caching behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramCacheConfig {
    /// Maximum number of diagrams to cache
    pub max_cached_diagrams: usize,

    /// Maximum memory usage for cache in MB
    pub max_memory_mb: usize,

    /// Cache entry TTL in hours
    pub ttl_hours: u32,

    /// Enable compression for cached diagrams
    pub enable_compression: bool,

    /// Compression level (1-9, higher = better compression)
    pub compression_level: u8,

    /// Enable cache statistics collection
    pub enable_statistics: bool,

    /// Enable selective regeneration
    pub enable_selective_regeneration: bool,

    /// Cache warming strategy
    pub warming_strategy: CacheWarmingStrategy,

    /// Invalidation strategy
    pub invalidation_strategy: InvalidationStrategy,
}

impl Default for DiagramCacheConfig {
    fn default() -> Self {
        Self {
            max_cached_diagrams: 10000,
            max_memory_mb: 512,
            ttl_hours: 24,
            enable_compression: true,
            compression_level: 6,
            enable_statistics: true,
            enable_selective_regeneration: true,
            warming_strategy: CacheWarmingStrategy::Adaptive,
            invalidation_strategy: InvalidationStrategy::DependencyBased,
        }
    }
}

/// Cache warming strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheWarmingStrategy {
    /// No cache warming
    None,
    /// Warm cache based on recent access patterns
    AccessBased,
    /// Warm cache based on dependency analysis
    DependencyBased,
    /// Adaptive warming based on usage patterns
    Adaptive,
}

/// Result type for diagram cache operations
pub type Result<T> = std::result::Result<T, DiagramCacheError>;

/// Error types for diagram cache operations
#[derive(thiserror::Error, Debug)]
pub enum DiagramCacheError {
    #[error("Cache miss: diagram not found in cache")]
    CacheMiss,

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Decompression error: {0}")]
    DecompressionError(String),

    #[error("Invalid diagram type: {0}")]
    InvalidDiagramType(String),

    #[error("Cache full: unable to store new diagram")]
    CacheFull,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Incremental analysis error: {0}")]
    IncrementalAnalysisError(String),

    #[error("Dependency tracking error: {0}")]
    DependencyTrackingError(String),
}
