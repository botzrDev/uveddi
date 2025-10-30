//! # AST Cache
//!
//! Caching system for parsed Abstract Syntax Trees with file modification
//! time-based invalidation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// Tree-sitter imports with feature gate
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::Tree;
#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;

/// Cache entry for an AST
#[derive(Debug, Clone)]
/// Represents ast cache entry in the system.
pub struct AstCacheEntry {
    /// The cached syntax tree
    pub tree: Tree,

    /// Source code that was parsed
    pub source: String,

    /// When the file was last modified
    pub modified_time: SystemTime,

    /// When this entry was cached
    pub cached_time: SystemTime,

    /// Size of the cached entry in bytes (for memory management)
    pub size_bytes: usize,
}

/// AST cache with file modification time tracking
#[derive(Debug)]
/// Represents ast cache in the system.
pub struct AstCache {
    /// Cache entries indexed by file path
    entries: HashMap<PathBuf, AstCacheEntry>,

    /// Maximum cache size in entries
    max_entries: usize,

    /// Cache statistics
    stats: CacheStats,
}

/// Cache statistics
#[derive(Debug, Default, Copy, Clone)]
/// Represents cache stats in the system.
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub invalidations: u64,
    pub memory_bytes: usize,
}

impl AstCache {
    /// Create a new AST cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            stats: CacheStats::default(),
        }
    }

    /// Get a cached AST if valid
    pub fn get(&mut self, path: &Path) -> Option<AstCacheEntry> {
        // Check if file exists and get modification time
        let file_modified = std::fs::metadata(path).ok()?.modified().ok()?;

        let mut is_stale = false;
        let mut result = None;

        if let Some(entry) = self.entries.get(path) {
            if entry.modified_time >= file_modified {
                self.stats.hits += 1;
                result = Some(entry.clone());
            } else {
                is_stale = true;
            }
        }

        if is_stale {
            if let Some(removed) = self.entries.remove(path) {
                self.stats.evictions += 1;
                self.stats.memory_bytes =
                    self.stats.memory_bytes.saturating_sub(removed.size_bytes);
            }
        }

        if result.is_none() {
            self.stats.misses += 1;
        }

        result
    }

    /// Store an AST in the cache
    pub fn put(&mut self, path: PathBuf, tree: Tree, source: String) -> Result<(), CacheError> {
        // Get file modification time
        let modified_time = std::fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .map_err(|_| CacheError::InvalidFile)?;

        let entry = AstCacheEntry {
            size_bytes: source.len() + std::mem::size_of::<Tree>(),
            tree,
            source,
            modified_time,
            cached_time: SystemTime::now(),
        };

        // Evict entries if necessary
        if self.entries.len() >= self.max_entries {
            self.evict_oldest();
        }

        let size_bytes = entry.size_bytes;
        self.entries.insert(path, entry);
        self.stats.memory_bytes += size_bytes;

        Ok(())
    }

    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.memory_bytes = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Evict the oldest cache entry
    fn evict_oldest(&mut self) {
        if let Some((oldest_path, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.cached_time)
            .map(|(path, entry)| (path.clone(), entry.size_bytes))
        {
            if let Some(removed) = self.entries.remove(&oldest_path) {
                self.stats.evictions += 1;
                self.stats.memory_bytes =
                    self.stats.memory_bytes.saturating_sub(removed.size_bytes);
            }
        }
    }

    /// Invalidate cache entry for a specific file
    pub fn invalidate(&mut self, path: &Path) -> Result<(), CacheError> {
        if let Some(entry) = self.entries.remove(path) {
            self.stats.memory_bytes = self.stats.memory_bytes.saturating_sub(entry.size_bytes);
            self.stats.invalidations += 1;
            Ok(())
        } else {
            Err(CacheError::InvalidFile)
        }
    }

    /// Remove entries for files that no longer exist
    pub fn cleanup_deleted_files(&mut self) {
        let mut to_remove = Vec::new();

        for (path, entry) in &self.entries {
            if !path.exists() {
                to_remove.push((path.clone(), entry.size_bytes));
            }
        }

        for (path, size) in to_remove {
            self.entries.remove(&path);
            self.stats.memory_bytes = self.stats.memory_bytes.saturating_sub(size);
        }
    }
}

/// Errors that can occur during caching operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Invalid file or unable to read metadata")]
    InvalidFile,

    #[error("Cache is full")]
    CacheFull,
}

impl CacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Format memory usage in human-readable form
    pub fn memory_usage_display(&self) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if self.memory_bytes >= GB {
            format!("{:.1} GB", self.memory_bytes as f64 / GB as f64)
        } else if self.memory_bytes >= MB {
            format!("{:.1} MB", self.memory_bytes as f64 / MB as f64)
        } else if self.memory_bytes >= KB {
            format!("{:.1} KB", self.memory_bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", self.memory_bytes)
        }
    }
}
