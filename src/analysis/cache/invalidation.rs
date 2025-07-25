//! Cache invalidation strategies with content-based hashing
//!
//! This module provides intelligent cache invalidation mechanisms using
//! content-based hashing to ensure cache coherence and optimal performance.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::analysis::cache::wrappers::{ArchivablePathBuf, ArchivableSystemTime};
use std::time::SystemTime;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvalidationError {
    #[error("File system error: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("Invalid hash format: {0}")]
    InvalidHash(String),
    #[error("Dependency resolution error: {0}")]
    DependencyError(String),
}

/// Strategy for cache invalidation
pub trait InvalidationStrategy: Send + Sync {
    /// Check if a cache entry should be invalidated
    fn should_invalidate(&self, key: &str, cached_hash: &str) -> Result<bool, InvalidationError>;
    
    /// Get the current content hash for a key
    fn get_current_hash(&self, key: &str) -> Result<String, InvalidationError>;
    
    /// Invalidate entries based on dependency changes
    fn invalidate_dependents(&self, changed_key: &str) -> Result<Vec<String>, InvalidationError>;
}

/// Content-based invalidation using SHA-256 hashing
pub struct ContentHashInvalidator {
    dependency_graph: HashMap<PathBuf, Vec<PathBuf>>,
    hash_cache: HashMap<PathBuf, (String, SystemTime)>,
    hash_cache_ttl: u64,
}

impl ContentHashInvalidator {
    pub fn new() -> Self {
        Self {
            dependency_graph: HashMap::new(),
            hash_cache: HashMap::new(),
            hash_cache_ttl: 300, // 5 minutes
        }
    }

    pub fn with_ttl(ttl_seconds: u64) -> Self {
        Self {
            dependency_graph: HashMap::new(),
            hash_cache: HashMap::new(),
            hash_cache_ttl: ttl_seconds,
        }
    }

    /// Add a dependency relationship (source depends on target)
    pub fn add_dependency(&mut self, source: PathBuf, target: PathBuf) {
        self.dependency_graph
            .entry(source)
            .or_insert_with(Vec::new)
            .push(target);
    }

    /// Set the entire dependency graph
    pub fn set_dependency_graph(&mut self, graph: HashMap<PathBuf, Vec<PathBuf>>) {
        self.dependency_graph = graph;
    }

    /// Calculate SHA-256 hash of file content
    fn calculate_file_hash(&self, path: &Path) -> Result<String, InvalidationError> {
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get cached hash or calculate new one
    fn get_file_hash_cached(&mut self, path: &Path) -> Result<String, InvalidationError> {
        let now = SystemTime::now();
        let path_buf = path.to_path_buf();

        // Check if we have a cached hash that's still valid
        if let Some((cached_hash, cached_time)) = self.hash_cache.get(&path_buf) {
            if let Ok(elapsed) = now.duration_since(*cached_time) {
                if elapsed.as_secs() < self.hash_cache_ttl {
                    return Ok(cached_hash.clone());
                }
            }
        }

        // Calculate new hash and cache it
        let hash = self.calculate_file_hash(path)?;
        self.hash_cache.insert(path_buf, (hash.clone(), now));
        Ok(hash)
    }

    /// Create composite hash for a file and its dependencies
    fn calculate_composite_hash(&mut self, path: &Path) -> Result<String, InvalidationError> {
        let mut hasher = Sha256::new();
        
        // Hash the main file
        let main_hash = self.get_file_hash_cached(path)?;
        hasher.update(main_hash.as_bytes());

        // Hash all dependencies
        let path_buf = path.to_path_buf();
        let dependencies = self.dependency_graph.get(&path_buf).cloned();
        if let Some(dependencies) = dependencies {
            let mut dep_hashes: Vec<String> = Vec::new();
            
            for dep_path in dependencies {
                if dep_path.exists() {
                    let dep_hash = self.get_file_hash_cached(&dep_path)?;
                    dep_hashes.push(dep_hash);
                }
            }
            
            // Sort for deterministic ordering
            dep_hashes.sort();
            for dep_hash in dep_hashes {
                hasher.update(dep_hash.as_bytes());
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Find all files that depend on the given file
    fn find_dependents(&self, changed_path: &Path) -> Vec<PathBuf> {
        let mut dependents = Vec::new();
        let changed_path_buf = changed_path.to_path_buf();

        for (source, deps) in &self.dependency_graph {
            if deps.contains(&changed_path_buf) {
                dependents.push(source.clone());
            }
        }

        dependents
    }

    /// Clear expired entries from hash cache
    pub fn cleanup_hash_cache(&mut self) {
        let now = SystemTime::now();
        self.hash_cache.retain(|_path, (_, timestamp)| {
            if let Ok(elapsed) = now.duration_since(*timestamp) {
                elapsed.as_secs() < self.hash_cache_ttl
            } else {
                false
            }
        });
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> InvalidationStats {
        InvalidationStats {
            dependency_count: self.dependency_graph.len(),
            hash_cache_size: self.hash_cache.len(),
            total_dependencies: self.dependency_graph.values().map(|v| v.len()).sum(),
        }
    }
}

impl Default for ContentHashInvalidator {
    fn default() -> Self {
        Self::new()
    }
}

impl InvalidationStrategy for ContentHashInvalidator {
    fn should_invalidate(&self, key: &str, cached_hash: &str) -> Result<bool, InvalidationError> {
        let path = Path::new(key);
        if !path.exists() {
            // File doesn't exist, invalidate the cache entry
            return Ok(true);
        }

        // Create a mutable reference to self for hash calculation
        // In practice, this would be handled with Arc<Mutex<_>> or similar
        let mut invalidator = ContentHashInvalidator {
            dependency_graph: self.dependency_graph.clone(),
            hash_cache: HashMap::new(), // Fresh cache for this check
            hash_cache_ttl: self.hash_cache_ttl,
        };

        let current_hash = invalidator.calculate_composite_hash(path)?;
        Ok(current_hash != cached_hash)
    }

    fn get_current_hash(&self, key: &str) -> Result<String, InvalidationError> {
        let path = Path::new(key);
        if !path.exists() {
            return Err(InvalidationError::InvalidHash(format!(
                "File does not exist: {}",
                key
            )));
        }

        // Create a mutable copy for hash calculation
        let mut invalidator = ContentHashInvalidator {
            dependency_graph: self.dependency_graph.clone(),
            hash_cache: HashMap::new(),
            hash_cache_ttl: self.hash_cache_ttl,
        };

        invalidator.calculate_composite_hash(path)
    }

    fn invalidate_dependents(&self, changed_key: &str) -> Result<Vec<String>, InvalidationError> {
        let changed_path = Path::new(changed_key);
        let dependents = self.find_dependents(changed_path);
        
        Ok(dependents
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect())
    }
}

/// Time-based invalidation strategy
pub struct TimeBasedInvalidator {
    ttl_seconds: u64,
}

impl TimeBasedInvalidator {
    pub fn new(ttl_seconds: u64) -> Self {
        Self { ttl_seconds }
    }

    fn get_file_modification_time(&self, path: &Path) -> Result<SystemTime, InvalidationError> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.modified()?)
    }
}

impl InvalidationStrategy for TimeBasedInvalidator {
    fn should_invalidate(&self, key: &str, cached_hash: &str) -> Result<bool, InvalidationError> {
        let path = Path::new(key);
        if !path.exists() {
            return Ok(true);
        }

        // Extract timestamp from cached hash (format: "timestamp:actualhash")
        let parts: Vec<&str> = cached_hash.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Ok(true); // Invalid format, invalidate
        }

        let cached_timestamp: u64 = parts[0]
            .parse()
            .map_err(|_| InvalidationError::InvalidHash("Invalid timestamp format".to_string()))?;

        let file_modified = self.get_file_modification_time(path)?;
        let file_timestamp = file_modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| InvalidationError::InvalidHash("Invalid file timestamp".to_string()))?
            .as_secs();

        // Check if file was modified after cache entry or TTL expired
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| InvalidationError::InvalidHash("Invalid system time".to_string()))?
            .as_secs();

        Ok(file_timestamp > cached_timestamp || (now - cached_timestamp) > self.ttl_seconds)
    }

    fn get_current_hash(&self, key: &str) -> Result<String, InvalidationError> {
        let path = Path::new(key);
        if !path.exists() {
            return Err(InvalidationError::InvalidHash(format!(
                "File does not exist: {}",
                key
            )));
        }

        let file_modified = self.get_file_modification_time(path)?;
        let timestamp = file_modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| InvalidationError::InvalidHash("Invalid file timestamp".to_string()))?
            .as_secs();

        // Create a simple hash based on timestamp and file size
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let content_hint = format!("{}:{}", timestamp, size);
        
        let mut hasher = Sha256::new();
        hasher.update(content_hint.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        Ok(format!("{}:{}", timestamp, hash))
    }

    fn invalidate_dependents(&self, _changed_key: &str) -> Result<Vec<String>, InvalidationError> {
        // Time-based invalidation doesn't track dependencies
        Ok(Vec::new())
    }
}

/// Statistics for cache invalidation
#[derive(Debug, Clone)]
pub struct InvalidationStats {
    pub dependency_count: usize,
    pub hash_cache_size: usize,
    pub total_dependencies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_content_hash_invalidator() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut invalidator = ContentHashInvalidator::new();
        let key = file_path.to_string_lossy().to_string();

        // Get initial hash
        let hash1 = invalidator.get_current_hash(&key).unwrap();
        
        // Should not invalidate with same content
        assert!(!invalidator.should_invalidate(&key, &hash1).unwrap());

        // Modify file content
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, universe!").unwrap();

        // Should invalidate with different content
        assert!(invalidator.should_invalidate(&key, &hash1).unwrap());
    }

    #[test]
    fn test_dependency_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.rs");
        let dep_path = temp_dir.path().join("dependency.rs");

        // Create test files
        std::fs::write(&source_path, "use dependency;\nfn main() {}").unwrap();
        std::fs::write(&dep_path, "pub fn helper() {}").unwrap();

        let mut invalidator = ContentHashInvalidator::new();
        invalidator.add_dependency(source_path.clone(), dep_path.clone());

        let dep_key = dep_path.to_string_lossy().to_string();
        let dependents = invalidator.invalidate_dependents(&dep_key).unwrap();

        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], source_path.to_string_lossy());
    }

    #[test]
    fn test_time_based_invalidator() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a test file
        std::fs::write(&file_path, "Hello, world!").unwrap();

        let invalidator = TimeBasedInvalidator::new(60); // 1 minute TTL
        let key = file_path.to_string_lossy().to_string();

        // Get initial hash
        let hash1 = invalidator.get_current_hash(&key).unwrap();
        
        // Should not invalidate immediately
        assert!(!invalidator.should_invalidate(&key, &hash1).unwrap());

        // Wait at least 1 second to ensure timestamp changes (file timestamps have second precision)
        std::thread::sleep(std::time::Duration::from_secs(1));
        std::fs::write(&file_path, "Hello, universe!").unwrap();

        // Should invalidate after modification
        assert!(invalidator.should_invalidate(&key, &hash1).unwrap());
    }

    #[test]
    fn test_invalidation_stats() {
        let mut invalidator = ContentHashInvalidator::new();
        
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.rs");
        let file2 = temp_dir.path().join("file2.rs");
        let file3 = temp_dir.path().join("file3.rs");

        invalidator.add_dependency(file1.clone(), file2.clone());
        invalidator.add_dependency(file1.clone(), file3.clone());

        let stats = invalidator.get_stats();
        assert_eq!(stats.dependency_count, 1);
        assert_eq!(stats.total_dependencies, 2);
    }

    #[test]
    fn test_hash_cache_cleanup() {
        let mut invalidator = ContentHashInvalidator::with_ttl(1); // 1 second TTL
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        // Add entry to hash cache
        let _hash = invalidator.get_file_hash_cached(&file_path).unwrap();
        assert_eq!(invalidator.hash_cache.len(), 1);

        // Wait for TTL to expire
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Cleanup should remove expired entries
        invalidator.cleanup_hash_cache();
        assert_eq!(invalidator.hash_cache.len(), 0);
    }
}