//! Multi-layered cache system for Uveddi analysis engine
//! 
//! This module implements a comprehensive caching strategy with:
//! - L1: In-memory cache for hot data (LRU eviction)
//! - L2: Disk-based cache with efficient binary serialization
//! - Content-based cache invalidation using SHA-256 hashing
//! - Real-time cache metrics and monitoring

pub mod ast;
pub mod invalidation;
pub mod metrics;
pub mod engine_cache;

#[cfg(feature = "memory-optimization")]
pub mod multilayer_cache;
#[cfg(feature = "memory-optimization")]
pub mod serialization;

#[cfg(not(feature = "memory-optimization"))]
pub mod compat;

pub use ast::AstCache;
pub use invalidation::{InvalidationStrategy, ContentHashInvalidator};
pub use metrics::{CacheMetrics, CacheMonitor};

#[cfg(feature = "memory-optimization")]
pub use multilayer_cache::{MultiLayerCache, CacheLayer, CacheConfig as MultiLayerCacheConfig};
#[cfg(feature = "memory-optimization")]
pub use serialization::{SerializationFormat, CacheSerializer};

// Compatibility wrappers for when memory optimization is disabled
#[cfg(not(feature = "memory-optimization"))]
pub mod wrappers {
    pub use super::compat::{ArchivableSystemTime, ArchivablePathBuf};
}

#[cfg(feature = "memory-optimization")]
pub mod wrappers {
    pub use super::serialization::wrappers::{ArchivableSystemTime, ArchivablePathBuf};
}

pub use engine_cache::{EngineCache, EngineCacheConfig, EngineCacheStats};
