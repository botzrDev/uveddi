//! Caching System for Uveddi
//!
//! This module provides comprehensive caching capabilities to optimize performance
//! across repeated analysis runs. The caching system stores intermediate results,
//! parsed ASTs, and analysis outcomes to avoid redundant computation.
//!
//! # Caching Strategies
//!
//! ## Result Caching
//! - **Analysis Results**: Cache complete analysis results for unchanged files
//! - **AST Parsing**: Cache parsed syntax trees to avoid re-parsing
//! - **AI Responses**: Cache AI-generated explanations for identical code patterns
//! - **Dependency Graphs**: Cache dependency relationships for static codebases
//!
//! ## Cache Invalidation
//! - **File Modification**: Automatically invalidate cache when source files change
//! - **Configuration Changes**: Clear cache when analysis configuration is modified
//! - **Manual Clearing**: Provide utilities for manual cache management
//!
//! # Performance Benefits
//!
//! - **Reduced Parse Time**: Skip AST generation for unchanged files
//! - **Faster Analysis**: Reuse previous analysis results when applicable
//! - **AI Efficiency**: Avoid redundant AI API calls for similar code patterns
//! - **Memory Optimization**: Intelligent cache eviction based on usage patterns
//!
//! # Usage Examples
//!
//! ```rust,no_run
//! use uveddi::cache::result_cache::ResultCache;
//!
//! // Initialize cache
//! let mut cache = ResultCache::new();
//!
//! // Cache analysis result
//! let file_path = "src/main.rs";
//! let analysis_result = /* ... */;
//! cache.store_result(file_path, analysis_result);
//!
//! // Retrieve cached result
//! if let Some(cached) = cache.get_result(file_path) {
//!     println!("Using cached analysis for {}", file_path);
//! }
//! ```
//!
//! # Cache Storage
//!
//! The caching system supports multiple storage backends:
//! - **In-Memory**: Fast access for session-based caching
//! - **File-Based**: Persistent cache across application restarts
//! - **Database**: Integrated with SQLite for complex cache queries
//!
//! # Thread Safety
//!
//! All cache implementations are designed to be thread-safe and can be
//! used safely across async tasks and concurrent analysis operations.

pub mod result_cache;

/// Placeholder documentation for public items
