//! AST disk and memory cache for Uveddi

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tree_sitter::Tree;

#[derive(Serialize, Deserialize)]
pub struct CacheableAst {
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
    pub language: String,
}

pub struct AstCache {
    memory_cache: HashMap<PathBuf, (Tree, SystemTime)>,
    disk_cache_path: PathBuf,
    max_memory_entries: usize,
}

impl AstCache {
    pub fn new(capacity: usize, cache_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).unwrap_or_default();
        Self {
            memory_cache: HashMap::new(),
            disk_cache_path: cache_dir,
            max_memory_entries: capacity,
        }
    }

    pub fn get(&mut self, path: &Path) -> Option<Tree> {
        if let Some((tree, cached_time)) = self.memory_cache.get(path) {
            let modified = fs::metadata(path).and_then(|m| m.modified()).ok()?;
            if modified <= *cached_time {
                return Some(tree.clone());
            }
        }
        // TODO: Implement disk cache for ASTs using a serializable representation
        // let cache_path = self.disk_cache_path.join(format!("{:x}.ast", md5::compute(path.to_string_lossy().as_bytes())));
        // let cache_meta = fs::metadata(&cache_path).ok()?;
        // let file_meta = fs::metadata(path).ok()?;
        // let cache_time = cache_meta.modified().ok()?;
        // let file_time = file_meta.modified().ok()?;
        // if file_time <= cache_time {
        //     let bytes = fs::read(&cache_path).ok()?;
        //     let tree = deserialize(&bytes).ok()?;
        //     self.add_to_memory_cache(path.to_path_buf(), tree.clone(), file_time);
        //     return Some(tree);
        // }
        None
    }

    pub fn store(&mut self, path: &Path, tree: Tree) {
        let modified = fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::now());
        // let bytes = serialize(&tree).unwrap();
        // let cache_path = self.disk_cache_path.join(format!("{:x}.ast", md5::compute(path.to_string_lossy().as_bytes())));
        // fs::write(&cache_path, bytes).ok();
        self.add_to_memory_cache(path.to_path_buf(), tree, modified);
    }

    fn add_to_memory_cache(&mut self, path: PathBuf, tree: Tree, modified: SystemTime) {
        if self.memory_cache.len() >= self.max_memory_entries {
            if let Some(oldest) = self.memory_cache.keys().next().cloned() {
                self.memory_cache.remove(&oldest);
            }
        }
        self.memory_cache.insert(path, (tree, modified));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn test_ast_cache() {
        // Placeholder: actual test would require a real Tree object
        assert!(true);
    }
}
