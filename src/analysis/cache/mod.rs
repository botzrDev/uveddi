//! Multi-layered cache system for Uveddi analysis engine
//!
//! This module implements a comprehensive caching strategy with:
//! - L1: In-memory cache for hot data (LRU eviction)
//! - L2: Disk-based cache with efficient binary serialization
//! - Content-based cache invalidation using SHA-256 hashing
//! - Real-time cache metrics and monitoring

pub mod ast;
pub mod engine_cache;
pub mod enhanced_engine_cache;
pub mod eviction;
pub mod file_watcher;
pub mod invalidation;
pub mod telemetry_integration;

#[cfg(feature = "prometheus")]
pub mod metrics;

#[cfg(feature = "memory-optimization")]
pub mod multilayer_cache;
#[cfg(feature = "memory-optimization")]
pub mod serialization;

#[cfg(not(feature = "memory-optimization"))]
pub mod compat;

pub use ast::AstCache;
pub use invalidation::{ContentHashInvalidator, InvalidationStrategy};

#[cfg(feature = "prometheus")]
pub use metrics::{CacheMetrics, CacheMonitor};

#[cfg(feature = "memory-optimization")]
pub use multilayer_cache::{CacheConfig as MultiLayerCacheConfig, CacheLayer, MultiLayerCache};
#[cfg(feature = "memory-optimization")]
pub use serialization::{CacheSerializer, SerializationFormat};

// Compatibility wrappers for when memory optimization is disabled
#[cfg(not(feature = "memory-optimization"))]
pub mod wrappers {
    pub use super::compat::{ArchivablePathBuf, ArchivableSystemTime};
}

#[cfg(feature = "memory-optimization")]
pub mod wrappers {
    pub use super::serialization::wrappers::{ArchivablePathBuf, ArchivableSystemTime};
}

pub use engine_cache::{EngineCache, EngineCacheConfig, EngineCacheStats};
pub use enhanced_engine_cache::{EnhancedEngineCache, EnhancedCacheConfig, CacheLayerConfig, EnhancedCacheStats, CacheLayerStats};
pub use eviction::{EvictionPolicy, EvictionManager, EvictionStats};
pub use file_watcher::{ConfigurableFileWatcher, FileWatcherConfig, FileWatcherStats, CacheInvalidationCallback, FileChangeEvent, FileEventType};
pub use telemetry_integration::{CacheTelemetryCollector, CacheTelemetryConfig, CacheTelemetryReport, CacheAlert, CacheAlertType, AlertSeverity};
