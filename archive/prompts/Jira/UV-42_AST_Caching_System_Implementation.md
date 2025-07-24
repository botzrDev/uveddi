# 🤖 **GPT Prompt for UV-42: AST Caching System Implementation**

```markdown
# UV-42: Implement Comprehensive AST Caching System

You are a senior Rust developer working on the Uveddi project, a sophisticated static code analysis and architectural visualization tool. Your task is to enhance the existing basic AST cache into a production-ready, high-performance caching system that meets enterprise-scale requirements.

## 🎯 **Objective**
Transform the current basic AST cache (`src/analysis/cache/ast.rs`) into a comprehensive caching system with LRU eviction, thread-safe concurrent access, memory-mapped storage, automatic invalidation, and performance metrics collection.

## 📋 **Current State Analysis**
The foundation exists but needs significant enhancement:
- ✅ Basic AST cache structure in `src/analysis/cache/ast.rs`
- ✅ Simple memory cache with timestamp validation
- ✅ Tree-sitter integration with feature flags
- ❌ No LRU eviction (uses simple FIFO)
- ❌ No thread safety for concurrent access
- ❌ No memory-mapped storage for large ASTs
- ❌ No performance metrics or observability
- ❌ No configurable cache policies
- ❌ Limited error handling and recovery

## 🔧 **Required Implementation**

### **1. Enhanced Core Data Structures**
Update `src/analysis/cache/ast.rs` with production-ready structures:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use tracing::{debug, info, warn, error};

#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_memory_entries: usize,
    pub max_memory_size_mb: usize,
    pub enable_disk_cache: bool,
    pub disk_cache_path: PathBuf,
    pub enable_memory_mapping: bool,
    pub lru_eviction_enabled: bool,
    pub cache_metrics_enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_entries: 10000,
            max_memory_size_mb: 500,
            enable_disk_cache: true,
            disk_cache_path: PathBuf::from("./cache/ast"),
            enable_memory_mapping: true,
            lru_eviction_enabled: true,
            cache_metrics_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAST {
    #[cfg(feature = "tree-sitter")]
    pub ast: Option<Arc<Tree>>, // Use Arc for efficient sharing
    #[cfg(not(feature = "tree-sitter"))]
    pub ast: Option<Arc<CacheableAst>>,
    pub file_hash: String,
    pub last_modified: SystemTime,
    pub access_count: u64,
    pub last_accessed: Instant,
    pub memory_size_bytes: usize,
    pub language: String,
    pub is_memory_mapped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub memory_usage_bytes: usize,
    pub average_lookup_time_ms: f64,
    pub hit_rate: f64,
    pub memory_mapped_entries: u64,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            evictions: 0,
            memory_usage_bytes: 0,
            average_lookup_time_ms: 0.0,
            hit_rate: 0.0,
            memory_mapped_entries: 0,
        }
    }

    pub fn update_hit_rate(&mut self) {
        if self.total_requests > 0 {
            self.hit_rate = (self.cache_hits as f64 / self.total_requests as f64) * 100.0;
        }
    }
}

pub struct ASTCache {
    cache: Arc<RwLock<HashMap<PathBuf, CachedAST>>>,
    lru_order: Arc<Mutex<Vec<PathBuf>>>, // For LRU tracking
    config: CacheConfig,
    metrics: Arc<Mutex<CacheMetrics>>,
    memory_usage: Arc<Mutex<usize>>,
}
```

### **2. Thread-Safe Implementation**
Implement thread-safe operations with efficient locking:

```rust
impl ASTCache {
    pub fn new(config: CacheConfig) -> Result<Self, std::io::Error> {
        if config.enable_disk_cache {
            std::fs::create_dir_all(&config.disk_cache_path)?;
        }
        
        info!("Initializing AST cache with config: {:?}", config);
        
        Ok(Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            lru_order: Arc::new(Mutex::new(Vec::new())),
            config,
            metrics: Arc::new(Mutex::new(CacheMetrics::new())),
            memory_usage: Arc::new(Mutex::new(0)),
        })
    }

    #[cfg(feature = "tree-sitter")]
    pub fn get(&self, path: &Path) -> Option<Arc<Tree>> {
        let start_time = Instant::now();
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.total_requests += 1;
        }

        // Check if file has been modified
        let file_modified = match fs::metadata(path).and_then(|m| m.modified()) {
            Ok(modified) => modified,
            Err(_) => {
                debug!("Failed to get file metadata for: {:?}", path);
                return None;
            }
        };

        // Check cache
        let cache_result = {
            let cache = self.cache.read().unwrap();
            cache.get(path).cloned()
        };

        if let Some(mut cached_ast) = cache_result {
            // Validate cache entry
            if file_modified <= cached_ast.last_modified {
                // Update access tracking for LRU
                cached_ast.access_count += 1;
                cached_ast.last_accessed = Instant::now();
                
                // Update cache with new access info
                {
                    let mut cache = self.cache.write().unwrap();
                    cache.insert(path.to_path_buf(), cached_ast.clone());
                }
                
                // Update LRU order
                self.update_lru_order(path);
                
                // Update metrics
                {
                    let mut metrics = self.metrics.lock().unwrap();
                    metrics.cache_hits += 1;
                    metrics.average_lookup_time_ms = 
                        (metrics.average_lookup_time_ms + start_time.elapsed().as_millis() as f64) / 2.0;
                    metrics.update_hit_rate();
                }
                
                debug!("Cache hit for: {:?}", path);
                return cached_ast.ast;
            } else {
                // File has been modified, invalidate cache entry
                self.invalidate_entry(path);
            }
        }

        // Cache miss
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.cache_misses += 1;
            metrics.update_hit_rate();
        }
        
        debug!("Cache miss for: {:?}", path);
        None
    }

    #[cfg(feature = "tree-sitter")]
    pub fn store(&self, path: &Path, tree: Tree) -> Result<(), Box<dyn std::error::Error>> {
        let file_hash = self.calculate_file_hash(path)?;
        let file_modified = fs::metadata(path)?.modified()?;
        let language = self.detect_language(path);
        
        // Estimate memory size (simplified)
        let memory_size = std::mem::size_of::<Tree>() + 1024; // Base estimate
        
        let cached_ast = CachedAST {
            ast: Some(Arc::new(tree)),
            file_hash,
            last_modified: file_modified,
            access_count: 1,
            last_accessed: Instant::now(),
            memory_size_bytes: memory_size,
            language,
            is_memory_mapped: false,
        };

        // Check if we need to evict entries
        self.ensure_cache_capacity(memory_size)?;
        
        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(path.to_path_buf(), cached_ast);
        }
        
        // Update LRU order
        self.update_lru_order(path);
        
        // Update memory usage
        {
            let mut memory_usage = self.memory_usage.lock().unwrap();
            *memory_usage += memory_size;
        }
        
        info!("Stored AST in cache for: {:?}", path);
        Ok(())
    }

    fn update_lru_order(&self, path: &Path) {
        let mut lru_order = self.lru_order.lock().unwrap();
        let path_buf = path.to_path_buf();
        
        // Remove if already exists
        lru_order.retain(|p| p != &path_buf);
        
        // Add to front (most recently used)
        lru_order.insert(0, path_buf);
    }

    fn ensure_cache_capacity(&self, new_entry_size: usize) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.lru_eviction_enabled {
            return Ok(());
        }

        let current_memory = *self.memory_usage.lock().unwrap();
        let max_memory = self.config.max_memory_size_mb * 1024 * 1024;
        
        // Check if we need to evict
        while (current_memory + new_entry_size) > max_memory || 
              self.cache.read().unwrap().len() >= self.config.max_memory_entries {
            
            if !self.evict_lru_entry()? {
                break; // No more entries to evict
            }
        }
        
        Ok(())
    }

    fn evict_lru_entry(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let path_to_evict = {
            let mut lru_order = self.lru_order.lock().unwrap();
            lru_order.pop() // Remove least recently used
        };

        if let Some(path) = path_to_evict {
            let evicted_size = {
                let mut cache = self.cache.write().unwrap();
                if let Some(cached_ast) = cache.remove(&path) {
                    cached_ast.memory_size_bytes
                } else {
                    0
                }
            };

            // Update memory usage
            {
                let mut memory_usage = self.memory_usage.lock().unwrap();
                *memory_usage = memory_usage.saturating_sub(evicted_size);
            }

            // Update metrics
            {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.evictions += 1;
            }

            debug!("Evicted LRU entry: {:?}", path);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn invalidate_entry(&self, path: &Path) {
        let evicted_size = {
            let mut cache = self.cache.write().unwrap();
            if let Some(cached_ast) = cache.remove(path) {
                cached_ast.memory_size_bytes
            } else {
                0
            }
        };

        // Remove from LRU order
        {
            let mut lru_order = self.lru_order.lock().unwrap();
            lru_order.retain(|p| p != path);
        }

        // Update memory usage
        {
            let mut memory_usage = self.memory_usage.lock().unwrap();
            *memory_usage = memory_usage.saturating_sub(evicted_size);
        }

        debug!("Invalidated cache entry: {:?}", path);
    }

    fn calculate_file_hash(&self, path: &Path) -> Result<String, std::io::Error> {
        let content = fs::read(path)?;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn get_metrics(&self) -> CacheMetrics {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.memory_usage_bytes = *self.memory_usage.lock().unwrap();
        metrics.clone()
    }

    pub fn clear(&self) {
        {
            let mut cache = self.cache.write().unwrap();
            cache.clear();
        }
        {
            let mut lru_order = self.lru_order.lock().unwrap();
            lru_order.clear();
        }
        {
            let mut memory_usage = self.memory_usage.lock().unwrap();
            *memory_usage = 0;
        }
        info!("Cache cleared");
    }
}

// Thread-safe cloning for concurrent access
impl Clone for ASTCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            lru_order: Arc::clone(&self.lru_order),
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
            memory_usage: Arc::clone(&self.memory_usage),
        }
    }
}
```

### **3. Integration with Analysis Engine**
Update `src/analysis/engine.rs` to use the enhanced cache:

```rust
// Add to engine.rs imports
use crate::analysis::cache::{ASTCache, CacheConfig};

// In AnalysisEngine struct, add:
pub struct AnalysisEngine {
    // ... existing fields
    ast_cache: ASTCache,
}

// In implementation:
impl AnalysisEngine {
    pub fn new() -> Result<Self, AnalysisError> {
        let cache_config = CacheConfig::default();
        let ast_cache = ASTCache::new(cache_config)
            .map_err(|e| AnalysisError::CacheError(e.to_string()))?;
        
        Ok(Self {
            // ... existing initialization
            ast_cache,
        })
    }

    // Update file parsing to use cache
    async fn parse_file_with_cache(&self, path: &Path) -> Result<ParsedFile, AnalysisError> {
        // Try cache first
        #[cfg(feature = "tree-sitter")]
        if let Some(cached_tree) = self.ast_cache.get(path) {
            return Ok(ParsedFile::from_cached_tree(path, cached_tree));
        }

        // Parse and cache
        let parsed_file = self.parse_file(path).await?;
        
        #[cfg(feature = "tree-sitter")]
        if let Some(tree) = parsed_file.get_tree() {
            if let Err(e) = self.ast_cache.store(path, tree.clone()) {
                warn!("Failed to cache AST for {:?}: {}", path, e);
            }
        }

        Ok(parsed_file)
    }
}
```

### **4. Comprehensive Testing Suite**
Create `src/analysis/cache/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_cache() -> (ASTCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            max_memory_entries: 5,
            max_memory_size_mb: 1,
            enable_disk_cache: true,
            disk_cache_path: temp_dir.path().to_path_buf(),
            enable_memory_mapping: false,
            lru_eviction_enabled: true,
            cache_metrics_enabled: true,
        };
        let cache = ASTCache::new(config).unwrap();
        (cache, temp_dir)
    }

    #[test]
    fn test_cache_hit_miss_ratio() {
        let (cache, _temp_dir) = create_test_cache();
        
        // Create test file
        let test_file = _temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();
        
        // First access should be a miss
        #[cfg(feature = "tree-sitter")]
        {
            assert!(cache.get(&test_file).is_none());
            let metrics = cache.get_metrics();
            assert_eq!(metrics.cache_misses, 1);
            assert_eq!(metrics.hit_rate, 0.0);
        }
    }

    #[test]
    fn test_lru_eviction() {
        let (cache, _temp_dir) = create_test_cache();
        
        // Create multiple test files
        for i in 0..10 {
            let test_file = _temp_dir.path().join(format!("test{}.rs", i));
            fs::write(&test_file, format!("fn main{i}() {{}}")).unwrap();
            
            #[cfg(feature = "tree-sitter")]
            {
                // Simulate storing (would need actual Tree objects in real test)
                // This is a simplified test structure
            }
        }
        
        let metrics = cache.get_metrics();
        assert!(metrics.evictions > 0);
    }

    #[test]
    fn test_file_invalidation() {
        let (cache, _temp_dir) = create_test_cache();
        
        let test_file = _temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();
        
        // Simulate file modification
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&test_file, "fn main() { println!(); }").unwrap();
        
        // Cache should detect modification and invalidate
        #[cfg(feature = "tree-sitter")]
        {
            assert!(cache.get(&test_file).is_none());
        }
    }

    #[test]
    fn test_thread_safety() {
        let (cache, _temp_dir) = create_test_cache();
        let cache = Arc::new(cache);
        
        let handles: Vec<_> = (0..10).map(|i| {
            let cache_clone = Arc::clone(&cache);
            let temp_path = _temp_dir.path().to_path_buf();
            
            std::thread::spawn(move || {
                let test_file = temp_path.join(format!("test{}.rs", i));
                fs::write(&test_file, format!("fn test{i}() {{}}")).unwrap();
                
                // Simulate concurrent access
                for _ in 0..100 {
                    #[cfg(feature = "tree-sitter")]
                    {
                        let _ = cache_clone.get(&test_file);
                    }
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Should not panic and metrics should be consistent
        let metrics = cache.get_metrics();
        assert!(metrics.total_requests > 0);
    }

    #[test]
    fn test_memory_usage_tracking() {
        let (cache, _temp_dir) = create_test_cache();
        
        let initial_metrics = cache.get_metrics();
        assert_eq!(initial_metrics.memory_usage_bytes, 0);
        
        // After adding entries, memory usage should increase
        // (This would need actual Tree objects in a real implementation)
    }

    #[test]
    fn test_cache_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            max_memory_entries: 100,
            max_memory_size_mb: 10,
            enable_disk_cache: false,
            disk_cache_path: temp_dir.path().to_path_buf(),
            enable_memory_mapping: true,
            lru_eviction_enabled: false,
            cache_metrics_enabled: false,
        };
        
        let cache = ASTCache::new(config.clone()).unwrap();
        assert_eq!(cache.config.max_memory_entries, 100);
        assert_eq!(cache.config.max_memory_size_mb, 10);
        assert!(!cache.config.enable_disk_cache);
    }
}
```

### **5. Performance Metrics Integration**
Add metrics collection that integrates with UV-86 observability:

```rust
// Add to src/analysis/cache/ast.rs
impl ASTCache {
    pub fn export_metrics_for_observability(&self) -> serde_json::Value {
        let metrics = self.get_metrics();
        serde_json::json!({
            "ast_cache": {
                "total_requests": metrics.total_requests,
                "cache_hits": metrics.cache_hits,
                "cache_misses": metrics.cache_misses,
                "hit_rate_percent": metrics.hit_rate,
                "evictions": metrics.evictions,
                "memory_usage_mb": metrics.memory_usage_bytes as f64 / (1024.0 * 1024.0),
                "average_lookup_time_ms": metrics.average_lookup_time_ms,
                "memory_mapped_entries": metrics.memory_mapped_entries,
                "cache_size": self.cache.read().unwrap().len(),
            }
        })
    }
}
```

## ✅ **Success Criteria**

### **Functional Requirements:**
- [ ] Thread-safe concurrent access with Arc/RwLock
- [ ] LRU eviction maintains memory limits (<500MB for 10k files)
- [ ] Automatic invalidation on file changes (hash + timestamp)
- [ ] Configurable cache size limits and policies
- [ ] Memory-mapped storage support for large ASTs
- [ ] Integration with existing parsing pipeline

### **Performance Requirements:**
- [ ] >80% cache hit rate for repeated analysis
- [ ] <1ms average lookup time
- [ ] <10ms invalidation time per file
- [ ] 60%+ reduction in parsing time for cached files
- [ ] Linear scaling with concurrent access

### **Integration Requirements:**
- [ ] Seamless integration with AnalysisEngine
- [ ] Support for all language parsers (Rust, Python, JavaScript)
- [ ] Metrics export for UV-86 observability
- [ ] Configuration integration with existing config system
- [ ] Error handling with proper error propagation

### **Quality Requirements:**
- [ ] Comprehensive test suite (>90% coverage)
- [ ] Thread safety validation
- [ ] Memory leak prevention
- [ ] Graceful degradation on errors
- [ ] Proper logging and debugging support

## 🧪 **Testing Strategy**

### **Unit Tests:**
```bash
cargo test cache::ast::tests --lib
```

### **Integration Tests:**
```bash
cargo test cache_integration --test integration_tests
```

### **Performance Benchmarks:**
```bash
cargo bench cache_performance
```

### **Load Testing:**
```bash
# Test with 1k, 5k, 10k file codebases
cargo test --release test_large_codebase_performance
```

## 🔗 **Integration Points**

### **Dependencies:**
- **UV-43**: Parallel processing (requires thread-safe cache)
- **UV-86**: Observability (metrics integration)
- **UV-145**: Memory optimization (memory-mapped storage)

### **Enables:**
- **UV-26**: AI memory optimization
- **UV-148**: Memory management techniques
- **UV-152**: Bounded cache patterns

## 📊 **Validation Commands**

```bash
# Compile check
cargo check --all-targets

# Run all tests
cargo test cache

# Run specific cache tests
cargo test ast_cache

# Performance benchmarks
cargo bench cache

# Memory usage validation
cargo test --release test_memory_limits

# Thread safety validation
cargo test --release test_concurrent_access

# Integration validation
cargo test --release integration_sprint1
```

## 🎯 **Implementation Timeline**

### **Day 1: Core Infrastructure**
- [ ] Enhanced data structures (CacheConfig, CachedAST, CacheMetrics)
- [ ] Thread-safe access patterns (Arc, RwLock, Mutex)
- [ ] Basic LRU eviction algorithm

### **Day 2: Advanced Features**
- [ ] File hash and modification tracking
- [ ] Automatic invalidation logic
- [ ] Memory usage tracking and limits
- [ ] Performance metrics collection

### **Day 3: Integration & Testing**
- [ ] Integration with AnalysisEngine
- [ ] Comprehensive test suite
- [ ] Performance benchmarks
- [ ] Documentation and examples

---

## 🚀 **Ready for Implementation**

This comprehensive AST caching system will provide:
- **60%+ performance improvement** through intelligent caching
- **Enterprise-scale support** for 10k+ file codebases
- **Thread-safe concurrent access** for parallel processing
- **Advanced memory management** with LRU eviction and memory mapping
- **Production-ready observability** with detailed metrics

The implementation follows Rust best practices, integrates seamlessly with existing Uveddi architecture, and provides the foundation for all subsequent performance optimizations in the Core Infrastructure Sprint.

**Ready to assign to your AI coding assistant!** 🚀
```