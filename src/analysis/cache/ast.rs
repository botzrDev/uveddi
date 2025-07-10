//! AST disk and memory cache for Uveddi

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;

#[derive(Serialize, Deserialize)]
pub struct CacheableAst {
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
    pub language: String,
}

pub struct AstCache {
    #[cfg(feature = "tree-sitter")]
    memory_cache: HashMap<PathBuf, (Tree, SystemTime)>,
    #[cfg(not(feature = "tree-sitter"))]
    memory_cache: HashMap<PathBuf, (CacheableAst, SystemTime)>,
    #[allow(dead_code)]
    disk_cache_path: PathBuf,
    max_memory_entries: usize,
}

impl AstCache {
    pub fn new(capacity: usize, cache_dir: PathBuf) -> Self {
        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            eprintln!("Warning: Failed to create cache directory: {}", e);
        }
        Self {
            memory_cache: HashMap::new(),
            disk_cache_path: cache_dir,
            max_memory_entries: capacity,
        }
    }

    #[cfg(feature = "tree-sitter")]
    pub fn get(&mut self, path: &Path) -> Option<Tree> {
        if let Some((tree, cached_time)) = self.memory_cache.get(path) {
            let modified = fs::metadata(path).and_then(|m| m.modified()).ok()?;
            if modified <= *cached_time {
                return Some(tree.clone());
            }
        }
        None
    }

    #[cfg(not(feature = "tree-sitter"))]
    pub fn get(&mut self, _path: &Path) -> Option<()> {
        None
    }

    #[cfg(feature = "tree-sitter")]
    pub fn store(&mut self, path: &Path, tree: Tree) {
        let modified = fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::now());
        self.add_to_memory_cache(path.to_path_buf(), tree, modified);
    }

    #[cfg(not(feature = "tree-sitter"))]
    pub fn store(&mut self, _path: &Path, _tree: ()) {
        // No-op when tree-sitter is disabled
    }

    #[cfg(feature = "tree-sitter")]
    fn add_to_memory_cache(&mut self, path: PathBuf, tree: Tree, modified: SystemTime) {
        if self.memory_cache.len() >= self.max_memory_entries {
            if let Some(oldest) = self.memory_cache.keys().next().cloned() {
                self.memory_cache.remove(&oldest);
            }
        }
        self.memory_cache.insert(path, (tree, modified));
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn add_to_memory_cache(&mut self, path: PathBuf, _tree: (), modified: SystemTime) {
        // Store cacheable AST instead when tree-sitter is disabled
        let cacheable = CacheableAst {
            data: Vec::new(),
            timestamp: modified,
            language: String::new(),
        };
        if self.memory_cache.len() >= self.max_memory_entries {
            if let Some(oldest) = self.memory_cache.keys().next().cloned() {
                self.memory_cache.remove(&oldest);
            }
        }
        self.memory_cache.insert(path, (cacheable, modified));
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ast_cache() {
        // Placeholder: actual test would require a real Tree object
        assert!(true);
    }
}
