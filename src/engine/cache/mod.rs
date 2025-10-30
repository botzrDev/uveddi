//! # Cache Module
//!
//! Centralized cache management for ASTs and analysis results.
//! Target: <300 lines total

/// Caching implementation for analysis cache.
pub mod analysis_cache;
/// Caching implementation for ast cache.
pub mod ast_cache;
/// Caching implementation for file watcher.
pub mod file_watcher;
/// Caching implementation for graph cache.
pub mod graph_cache;
/// Caching implementation for metrics.
pub mod metrics;
/// Caching implementation for service.
pub mod service;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_test;

// Re-export cache types
pub use analysis_cache::AnalysisCache;
pub use ast_cache::AstCache;
pub use file_watcher::{FileEvent, FileEventSender, FileWatcher};
pub use graph_cache::{create_shared_graph_cache, GraphCache, GraphCacheConfig, SharedGraphCache};
pub use metrics::{
    CacheErrorType, CacheMetricsCollector, CacheOperation, CacheType, PerformanceMetrics,
    RunningAverage,
};
pub use service::CacheServiceManager;
