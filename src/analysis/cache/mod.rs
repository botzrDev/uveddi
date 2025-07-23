//! Multi-layered cache system for Uveddi analysis engine
//! 
//! This module implements a comprehensive caching strategy with:
//! - L1: In-memory cache for hot data (LRU eviction)
//! - L2: Disk-based cache with efficient binary serialization
//! - Content-based cache invalidation using SHA-256 hashing
//! - Real-time cache metrics and monitoring

pub mod ast;
pub mod multilayer_cache;
pub mod serialization;
pub mod invalidation;
pub mod metrics;
pub mod engine_cache;

pub use ast::AstCache;
pub use multilayer_cache::{MultiLayerCache, CacheLayer, CacheConfig as MultiLayerCacheConfig};
pub use serialization::{SerializationFormat, CacheSerializer};
pub use invalidation::{InvalidationStrategy, ContentHashInvalidator};
pub use metrics::{CacheMetrics, CacheMonitor};
pub use engine_cache::{EngineCache, EngineCacheConfig, EngineCacheStats};
