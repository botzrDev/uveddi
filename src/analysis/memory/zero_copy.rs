//! Zero-copy AST serialization using rkyv and memory mapping
//! Based on UV-210 research: instant cache access with no deserialization overhead

use crate::analysis::memory::metrics::BASIC_MEMORY_METRICS;
use crate::ast::tree_sitter::ParsedFile;
use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Serializable AST representation optimized for zero-copy access
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SerializableAst {
    /// File metadata
    pub file_path: String,
    pub source_hash: u64,
    pub language: String,
    pub file_size_bytes: usize,
    pub parse_timestamp: u64,

    /// AST structure
    pub root_node: SerializableNode,
    pub total_nodes: usize,
    pub max_depth: usize,

    /// Source text (optional, for small files)
    pub source_text: Option<String>,
}

/// Serializable AST node optimized for cache efficiency
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SerializableNode {
    /// Node identification
    pub node_type: String,
    pub kind_id: u16,

    /// Position information
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,

    /// Tree structure
    pub children: Vec<SerializableNode>,
    pub named_children_count: usize,

    /// Node properties
    pub is_named: bool,
    pub is_missing: bool,
    pub is_extra: bool,

    /// Text content (for leaf nodes)
    pub text: Option<String>,
}

impl SerializableAst {
    /// Create from ParsedFile
    pub fn from_parsed_file(parsed_file: &ParsedFile) -> Result<Self, ZeroCopyError> {
        let source_hash = Self::calculate_source_hash(&parsed_file.source);
        let root_node = Self::convert_node(
            &parsed_file
                .tree
                .as_ref()
                .ok_or_else(|| ZeroCopyError::NodeConversion("No tree available".to_string()))?
                .root_node(),
            &parsed_file.source,
        )?;
        let (total_nodes, max_depth) = Self::calculate_tree_stats(&root_node);

        Ok(Self {
            file_path: parsed_file.file_path.to_string_lossy().to_string(),
            source_hash,
            language: format!("{:?}", parsed_file.language),
            file_size_bytes: parsed_file.source.len(),
            parse_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            root_node,
            total_nodes,
            max_depth,
            source_text: if parsed_file.source.len() < 64 * 1024 {
                Some(parsed_file.source.to_string())
            } else {
                None
            },
        })
    }

    /// Convert tree-sitter node to serializable format
    fn convert_node(
        node: &tree_sitter::Node,
        source: &str,
    ) -> Result<SerializableNode, ZeroCopyError> {
        let text = if node.child_count() == 0 {
            // Leaf node - capture text
            Some(
                node.utf8_text(source.as_bytes())
                    .map_err(|e| ZeroCopyError::NodeConversion(e.to_string()))?
                    .to_string(),
            )
        } else {
            None
        };

        let children = (0..node.child_count())
            .map(|i| node.child(i).unwrap())
            .map(|child| Self::convert_node(&child, source))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SerializableNode {
            node_type: node.kind().to_string(),
            kind_id: node.kind_id(),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_line: node.start_position().row as u32,
            start_column: node.start_position().column as u32,
            end_line: node.end_position().row as u32,
            end_column: node.end_position().column as u32,
            children,
            named_children_count: node.named_child_count(),
            is_named: node.is_named(),
            is_missing: node.is_missing(),
            is_extra: node.is_extra(),
            text,
        })
    }

    /// Calculate source hash for cache validation
    pub fn calculate_source_hash(source: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate tree statistics
    fn calculate_tree_stats(node: &SerializableNode) -> (usize, usize) {
        fn count_nodes_and_depth(node: &SerializableNode, current_depth: usize) -> (usize, usize) {
            let mut total_nodes = 1;
            let mut max_depth = current_depth;

            for child in &node.children {
                let (child_nodes, child_depth) = count_nodes_and_depth(child, current_depth + 1);
                total_nodes += child_nodes;
                max_depth = max_depth.max(child_depth);
            }

            (total_nodes, max_depth)
        }

        count_nodes_and_depth(node, 1)
    }

    /// Check if AST is still valid for given source
    pub fn is_valid_for_source(&self, source: &str) -> bool {
        self.source_hash == Self::calculate_source_hash(source)
    }

    /// Get cache efficiency score
    pub fn cache_efficiency_score(&self) -> f64 {
        // Factors: file size, node count, depth
        let size_factor = (self.file_size_bytes as f64 / (1024.0 * 1024.0)).min(1.0); // Normalize to 1MB
        let node_factor = (self.total_nodes as f64 / 10000.0).min(1.0); // Normalize to 10k nodes
        let depth_factor = (self.max_depth as f64 / 50.0).min(1.0); // Normalize to depth 50

        (size_factor + node_factor + depth_factor) / 3.0
    }
}

/// Zero-copy AST cache using memory-mapped files
#[derive(Clone)]
pub struct ZeroCopyAstCache {
    cache_dir: PathBuf,
    memory_maps: Arc<RwLock<HashMap<String, Arc<Mmap>>>>,
    cache_stats: Arc<RwLock<ZeroCopyCacheStats>>,
}

impl ZeroCopyAstCache {
    /// Create new zero-copy AST cache
    pub fn new(cache_dir: PathBuf) -> Result<Self, ZeroCopyError> {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| ZeroCopyError::CacheInitialization(e.to_string()))?;

        tracing::info!(
            "Initialized zero-copy AST cache at: {}",
            cache_dir.display()
        );

        Ok(Self {
            cache_dir,
            memory_maps: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(ZeroCopyCacheStats::new())),
        })
    }

    /// Store AST with zero-copy serialization
    pub fn store(&self, file_path: &Path, ast: &SerializableAst) -> Result<(), ZeroCopyError> {
        let cache_file = self.get_cache_path(file_path);

        // Serialize using bincode for now (will switch to rkyv once recursive issue is resolved)
        let serialized =
            bincode::serialize(ast).map_err(|e| ZeroCopyError::Serialization(e.to_string()))?;

        // Write to file
        std::fs::write(&cache_file, &serialized)
            .map_err(|e| ZeroCopyError::FileWrite(e.to_string()))?;

        // Update stats
        if let Ok(mut stats) = self.cache_stats.write() {
            stats.record_store(serialized.len());
        }

        tracing::debug!(
            "Stored zero-copy AST cache for: {} ({} bytes)",
            file_path.display(),
            serialized.len()
        );

        Ok(())
    }

    /// Load AST with memory-mapped access
    pub fn load(&self, file_path: &Path) -> Result<Option<SerializableAst>, ZeroCopyError> {
        let cache_file = self.get_cache_path(file_path);
        let cache_key = self.get_cache_key(file_path);

        if !cache_file.exists() {
            if let Ok(mut stats) = self.cache_stats.write() {
                stats.record_miss();
            }
            return Ok(None);
        }

        // Check if already memory-mapped
        if let Ok(maps) = self.memory_maps.read() {
            if let Some(existing_map) = maps.get(&cache_key) {
                if let Ok(mut stats) = self.cache_stats.write() {
                    stats.record_hit();
                }

                // Deserialize from memory-mapped data
                let ast: SerializableAst = bincode::deserialize(&existing_map[..])
                    .map_err(|e| ZeroCopyError::Serialization(e.to_string()))?;

                return Ok(Some(ast));
            }
        }

        // Create new memory mapping
        let file = File::open(&cache_file).map_err(|e| ZeroCopyError::FileRead(e.to_string()))?;

        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|e| ZeroCopyError::MemoryMapping(e.to_string()))?
        };

        // Deserialize from memory-mapped data
        let ast: SerializableAst = bincode::deserialize(&mmap[..])
            .map_err(|e| ZeroCopyError::Serialization(e.to_string()))?;

        // Store memory mapping for reuse
        if let Ok(mut maps) = self.memory_maps.write() {
            maps.insert(cache_key, Arc::new(mmap));
        }

        // Update stats
        if let Ok(mut stats) = self.cache_stats.write() {
            stats.record_hit();
        }

        Ok(Some(ast))
    }

    /// Check if cache entry exists and is valid
    pub fn is_valid(&self, file_path: &Path, source_hash: u64) -> bool {
        match self.load(file_path) {
            Ok(Some(ast)) => ast.source_hash == source_hash,
            _ => false,
        }
    }

    /// Remove cache entry
    pub fn remove(&self, file_path: &Path) -> Result<(), ZeroCopyError> {
        let cache_file = self.get_cache_path(file_path);
        let cache_key = self.get_cache_key(file_path);

        // Remove from memory maps
        if let Ok(mut maps) = self.memory_maps.write() {
            maps.remove(&cache_key);
        }

        // Remove file
        if cache_file.exists() {
            std::fs::remove_file(&cache_file)
                .map_err(|e| ZeroCopyError::FileRemoval(e.to_string()))?;
        }

        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&self) -> Result<(), ZeroCopyError> {
        // Clear memory maps
        if let Ok(mut maps) = self.memory_maps.write() {
            maps.clear();
        }

        // Remove cache directory contents
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| ZeroCopyError::CacheClear(e.to_string()))?;
            std::fs::create_dir_all(&self.cache_dir)
                .map_err(|e| ZeroCopyError::CacheInitialization(e.to_string()))?;
        }

        // Reset stats
        if let Ok(mut stats) = self.cache_stats.write() {
            *stats = ZeroCopyCacheStats::new();
        }

        tracing::info!("Cleared zero-copy AST cache");
        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> ZeroCopyCacheStats {
        self.cache_stats.read().unwrap().clone()
    }

    /// Export metrics for observability
    pub fn export_metrics(&self) -> serde_json::Value {
        let stats = self.get_stats();
        let memory_maps_count = self.memory_maps.read().unwrap().len();

        serde_json::json!({
            "zero_copy_ast_cache": {
                "cache_hits": stats.cache_hits,
                "cache_misses": stats.cache_misses,
                "hit_rate_percent": stats.hit_rate_percentage(),
                "total_stores": stats.total_stores,
                "total_bytes_stored": stats.total_bytes_stored,
                "average_entry_size_kb": stats.average_entry_size_kb(),
                "active_memory_maps": memory_maps_count,
                "cache_directory": self.cache_dir.display().to_string()
            }
        })
    }

    fn get_cache_path(&self, file_path: &Path) -> PathBuf {
        let hash = self.hash_path(file_path);
        self.cache_dir.join(format!("{}.ast.rkyv", hash))
    }

    fn get_cache_key(&self, file_path: &Path) -> String {
        self.hash_path(file_path)
    }

    fn hash_path(&self, file_path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        hasher.finish().to_string()
    }
}

/// Statistics for zero-copy cache operations
#[derive(Debug, Clone)]
pub struct ZeroCopyCacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_stores: u64,
    pub total_bytes_stored: u64,
}

impl ZeroCopyCacheStats {
    pub fn new() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            total_stores: 0,
            total_bytes_stored: 0,
        }
    }

    pub fn record_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn record_store(&mut self, bytes: usize) {
        self.total_stores += 1;
        self.total_bytes_stored += bytes as u64;
    }

    pub fn hit_rate_percentage(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total_requests as f64) * 100.0
        }
    }

    pub fn average_entry_size_kb(&self) -> f64 {
        if self.total_stores == 0 {
            0.0
        } else {
            (self.total_bytes_stored as f64 / self.total_stores as f64) / 1024.0
        }
    }
}

/// Errors that can occur during zero-copy operations
#[derive(Debug, thiserror::Error)]
pub enum ZeroCopyError {
    #[error("Cache initialization failed: {0}")]
    CacheInitialization(String),

    #[error("Serialization failed: {0}")]
    Serialization(String),

    #[error("File write failed: {0}")]
    FileWrite(String),

    #[error("File read failed: {0}")]
    FileRead(String),

    #[error("Memory mapping failed: {0}")]
    MemoryMapping(String),

    #[error("Node conversion failed: {0}")]
    NodeConversion(String),

    #[error("File removal failed: {0}")]
    FileRemoval(String),

    #[error("Cache clear failed: {0}")]
    CacheClear(String),
}

// Type alias for loaded AST (will be true zero-copy once rkyv recursive issue is resolved)
pub type LoadedAst = SerializableAst;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_serializable_ast() -> SerializableAst {
        SerializableAst {
            file_path: "test.rs".to_string(),
            source_hash: 12345,
            language: "rust".to_string(),
            file_size_bytes: 100,
            parse_timestamp: 1000000,
            root_node: SerializableNode {
                node_type: "source_file".to_string(),
                kind_id: 1,
                start_byte: 0,
                end_byte: 100,
                start_line: 0,
                start_column: 0,
                end_line: 1,
                end_column: 0,
                children: vec![],
                named_children_count: 0,
                is_named: true,
                is_missing: false,
                is_extra: false,
                text: None,
            },
            total_nodes: 1,
            max_depth: 1,
            source_text: Some("test".to_string()),
        }
    }

    #[test]
    fn test_serializable_ast_creation() {
        let ast = create_test_serializable_ast();

        assert_eq!(ast.file_path, "test.rs");
        assert_eq!(ast.total_nodes, 1);
        assert!(ast.cache_efficiency_score() >= 0.0);
    }

    #[test]
    fn test_zero_copy_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf());

        assert!(cache.is_ok());
        let cache = cache.unwrap();

        let stats = cache.get_stats();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }

    #[test]
    fn test_cache_store_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();
        let ast = create_test_serializable_ast();
        let file_path = Path::new("test.rs");

        // Store AST
        let store_result = cache.store(file_path, &ast);
        assert!(store_result.is_ok());

        // Load AST
        let load_result = cache.load(file_path);
        assert!(load_result.is_ok());
        let loaded_ast = load_result.unwrap();
        assert!(loaded_ast.is_some());

        let loaded_ast = loaded_ast.unwrap();
        assert_eq!(loaded_ast.file_path, "test.rs");
        assert_eq!(loaded_ast.source_hash, 12345);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = ZeroCopyCacheStats::new();

        assert_eq!(stats.hit_rate_percentage(), 0.0);

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert!((stats.hit_rate_percentage() - 66.67).abs() < 0.1);

        stats.record_store(1024);
        stats.record_store(2048);

        assert_eq!(stats.total_stores, 2);
        assert_eq!(stats.average_entry_size_kb(), 1.5);
    }

    #[test]
    fn test_source_hash_calculation() {
        let source1 = "fn main() {}";
        let source2 = "fn main() {}";
        let source3 = "fn test() {}";

        let hash1 = SerializableAst::calculate_source_hash(source1);
        let hash2 = SerializableAst::calculate_source_hash(source2);
        let hash3 = SerializableAst::calculate_source_hash(source3);

        assert_eq!(hash1, hash2); // Same source should have same hash
        assert_ne!(hash1, hash3); // Different source should have different hash
    }

    #[test]
    fn test_cache_validity() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();
        let ast = create_test_serializable_ast();
        let file_path = Path::new("test.rs");

        // Store AST
        cache.store(file_path, &ast).unwrap();

        // Check validity with correct hash
        assert!(cache.is_valid(file_path, 12345));

        // Check validity with wrong hash
        assert!(!cache.is_valid(file_path, 54321));
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();
        let ast = create_test_serializable_ast();
        let file_path = Path::new("test.rs");

        // Store AST
        cache.store(file_path, &ast).unwrap();

        // Verify it exists
        assert!(cache.load(file_path).unwrap().is_some());

        // Clear cache
        cache.clear().unwrap();

        // Verify it's gone
        assert!(cache.load(file_path).unwrap().is_none());

        // Verify stats are reset
        let stats = cache.get_stats();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 1); // The load after clear counts as miss
    }

    #[test]
    fn test_metrics_export() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();
        let ast = create_test_serializable_ast();
        let file_path = Path::new("test.rs");

        // Store and load to generate metrics
        cache.store(file_path, &ast).unwrap();
        cache.load(file_path).unwrap();

        let metrics = cache.export_metrics();
        assert!(metrics["zero_copy_ast_cache"].is_object());
        assert!(metrics["zero_copy_ast_cache"]["cache_hits"].is_number());
        assert!(metrics["zero_copy_ast_cache"]["cache_misses"].is_number());
        assert!(metrics["zero_copy_ast_cache"]["hit_rate_percent"].is_number());
    }
}
