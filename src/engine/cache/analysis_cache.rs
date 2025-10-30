//! # Analysis Cache
//!
//! Caching system for analysis results with incremental analysis support.

use crate::database::models::ArchitecturalIssue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache entry for analysis results
#[derive(Debug, Clone)]
/// Represents analysis cache entry in the system.
pub struct AnalysisCacheEntry {
    /// Analysis results (issues found)
    pub issues: Vec<ArchitecturalIssue>,

    /// File modification time when analyzed
    pub file_modified_time: SystemTime,

    /// Analysis execution time
    pub execution_time: std::time::Duration,

    /// When this analysis was performed
    pub analyzed_at: SystemTime,

    /// Detector versions used (for cache invalidation)
    pub detector_versions: HashMap<String, String>,
}

/// Analysis cache with incremental update support
#[derive(Debug)]
/// Represents analysis cache in the system.
pub struct AnalysisCache {
    /// Cache entries indexed by file path
    entries: HashMap<PathBuf, AnalysisCacheEntry>,

    /// Maximum cache size in entries
    max_entries: usize,

    /// Cache statistics
    stats: AnalysisCacheStats,
}

/// Analysis cache statistics
#[derive(Debug, Default, Clone)]
/// Represents analysis cache stats in the system.
pub struct AnalysisCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
    pub total_files_cached: usize,
}

impl AnalysisCache {
    /// Create a new analysis cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            stats: AnalysisCacheStats::default(),
        }
    }

    /// Get cached analysis results if valid
    pub fn get(
        &mut self,
        path: &Path,
        detector_versions: &HashMap<String, String>,
    ) -> Option<AnalysisCacheEntry> {
        // Check if file exists and get modification time
        let file_modified = std::fs::metadata(path).ok()?.modified().ok()?;

        let mut invalidate = false;
        let mut result = None;

        if let Some(entry) = self.entries.get(path) {
            let is_file_valid = entry.file_modified_time >= file_modified;
            let are_detectors_valid = entry.detector_versions == *detector_versions;

            if is_file_valid && are_detectors_valid {
                self.stats.hits += 1;
                result = Some(entry.clone());
            } else {
                invalidate = true;
            }
        }

        if invalidate {
            if self.entries.remove(path).is_some() {
                self.stats.invalidations += 1;
                self.stats.total_files_cached = self.entries.len();
            }
        }

        if result.is_none() {
            self.stats.misses += 1;
        }

        result
    }

    /// Store analysis results in the cache
    pub fn put(
        &mut self,
        path: PathBuf,
        issues: Vec<ArchitecturalIssue>,
        execution_time: std::time::Duration,
        detector_versions: HashMap<String, String>,
    ) -> Result<(), CacheError> {
        // Get file modification time
        let file_modified_time = std::fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .map_err(|_| CacheError::InvalidFile)?;

        let entry = AnalysisCacheEntry {
            issues,
            file_modified_time,
            execution_time,
            analyzed_at: SystemTime::now(),
            detector_versions,
        };

        // Evict entries if necessary
        if self.entries.len() >= self.max_entries {
            self.evict_oldest();
        }

        self.entries.insert(path, entry);
        self.stats.total_files_cached = self.entries.len();

        Ok(())
    }

    /// Clear all cached results
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.total_files_cached = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> &AnalysisCacheStats {
        &self.stats
    }

    /// Invalidate cache entry for a specific file
    pub fn invalidate(
        &mut self,
        path: &Path,
    ) -> Result<(), crate::engine::cache::ast_cache::CacheError> {
        if self.entries.remove(path).is_some() {
            self.stats.invalidations += 1;
            self.stats.total_files_cached = self.entries.len();
            Ok(())
        } else {
            Err(crate::engine::cache::ast_cache::CacheError::InvalidFile)
        }
    }

    /// Invalidate cache entries for specific detectors
    pub fn invalidate_detector(&mut self, detector_name: &str) {
        let mut to_remove = Vec::new();

        for (path, entry) in &self.entries {
            if entry.detector_versions.contains_key(detector_name) {
                to_remove.push(path.clone());
            }
        }

        for path in to_remove {
            self.entries.remove(&path);
            self.stats.invalidations += 1;
        }

        self.stats.total_files_cached = self.entries.len();
    }

    /// Get all cached file paths
    pub fn cached_files(&self) -> Vec<&PathBuf> {
        self.entries.keys().collect()
    }

    /// Evict the oldest cache entry
    fn evict_oldest(&mut self) {
        if let Some(oldest_path) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.analyzed_at)
            .map(|(path, _)| path.clone())
        {
            self.entries.remove(&oldest_path);
        }
    }

    /// Remove entries for files that no longer exist
    pub fn cleanup_deleted_files(&mut self) {
        let to_remove: Vec<PathBuf> = self
            .entries
            .keys()
            .filter(|path| !path.exists())
            .cloned()
            .collect();

        for path in to_remove {
            self.entries.remove(&path);
        }

        self.stats.total_files_cached = self.entries.len();
    }
}

/// Errors that can occur during analysis caching operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Invalid file or unable to read metadata")]
    InvalidFile,

    #[error("Cache is full")]
    CacheFull,
}

impl AnalysisCacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}
