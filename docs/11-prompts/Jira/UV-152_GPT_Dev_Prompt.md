# 🔧 UV-152 GPT Developer Implementation Prompt

## Task Assignment: UV-152 - Implement Bounded Caches to Prevent Memory Leaks

**Task ID**: UV-152  
**Priority**: HIGH - Critical Memory Issue  
**Estimated Time**: 4 days  
**Assignee**: GPT Development Assistant  
**Jira Link**: https://zenalto.atlassian.net/browse/UV-152

## 🎯 Task Overview

You are tasked with implementing a production-ready bounded cache system to replace the current unbounded HashMap cache in the AST parser. This is a critical memory management fix that prevents OOM crashes when analyzing large codebases.

## 🚨 Problem Statement

### Current Critical Issues
1. **Unbounded Memory Growth**: `src/ast/tree_sitter_impl.rs:25` uses `Mutex<HashMap<String, ParsedFile>>` with no size limits
2. **Memory Exhaustion**: Large codebases (10,000+ files) consume >1GB RAM and cause OOM crashes
3. **Inefficient Cloning**: Lines 384-408 recreate parsers unnecessarily in Clone implementation
4. **No Observability**: Zero metrics for cache performance monitoring

### Impact Assessment
- **Small Projects**: Acceptable (<50MB)
- **Medium Projects**: Concerning (100-500MB)
- **Large Projects**: **CRITICAL** (>1GB, OOM crashes)
- **Production Risk**: **HIGH** - System instability and crashes

## 📋 Detailed Requirements

### 1. Core Cache Implementation

Replace the unbounded cache with a bounded LRU cache:

```rust
// BEFORE (Problematic)
pub struct AstParser {
    parsers: HashMap<SourceLanguage, Parser>,
    cache: Mutex<HashMap<String, ParsedFile>>, // UNBOUNDED!
}

// AFTER (Required Implementation)
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

pub struct AstParser {
    parsers: Arc<HashMap<SourceLanguage, Parser>>, // Shared parsers
    cache: Mutex<LruCache<String, ParsedFile>>,
    max_cache_size: NonZeroUsize,
    metrics: Arc<CacheMetrics>,
}
```

### 2. Configuration System

Implement environment-based configuration:

```rust
impl AstParser {
    pub fn new() -> Result<Self, AstError> {
        let cache_size = std::env::var("UVEDDI_AST_CACHE_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000);
            
        let max_size = NonZeroUsize::new(cache_size)
            .ok_or(AstError::Other("Cache size must be > 0".to_string()))?;
            
        Ok(AstParser {
            parsers: Arc::new(Self::init_parsers()?),
            cache: Mutex::new(LruCache::new(max_size)),
            max_cache_size: max_size,
            metrics: Arc::new(CacheMetrics::new()),
        })
    }
    
    pub fn with_cache_size(cache_size: usize) -> Result<Self, AstError> {
        // Implementation for custom cache size
    }
}
```

### 3. Optimized Data Structures

Implement memory-efficient shared data:

```rust
#[derive(Clone, Debug)]
pub struct ParsedFile {
    path: PathBuf,
    language: SourceLanguage,
    source: Arc<String>,           // Shared source content
    custom_ast: Arc<Option<CustomAst>>, // Shared AST data
    modified_at: SystemTime,
}
```

### 4. Cache Metrics System

Add comprehensive observability:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    memory_usage_estimate: AtomicU64,
}

impl CacheMetrics {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }
    
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            hit_rate: self.hit_rate(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f64,
}
```

### 5. Efficient Clone Implementation

Fix the inefficient Clone implementation:

```rust
impl Clone for AstParser {
    fn clone(&self) -> Self {
        AstParser {
            parsers: Arc::clone(&self.parsers), // Share parsers instead of recreating
            cache: Mutex::new(LruCache::new(self.max_cache_size)),
            max_cache_size: self.max_cache_size,
            metrics: Arc::clone(&self.metrics),
        }
    }
}
```

## 🔧 Implementation Steps

### Step 1: Add Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
lru = "0.12.0"
```

### Step 2: Update AstParser Structure
- Replace HashMap with LruCache
- Add metrics tracking
- Implement shared parsers with Arc

### Step 3: Implement Cache Operations
```rust
impl AstParser {
    pub fn get_cached_file(&self, path: &str) -> Option<ParsedFile> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(file) = cache.get(path) {
            self.metrics.record_hit();
            Some(file.clone())
        } else {
            self.metrics.record_miss();
            None
        }
    }
    
    pub fn cache_file(&self, path: String, file: ParsedFile) {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= cache.cap().get() {
            self.metrics.record_eviction();
        }
        cache.put(path, file);
    }
    
    pub fn get_cache_stats(&self) -> CacheStats {
        self.metrics.get_stats()
    }
}
```

### Step 4: Environment Configuration
Support these environment variables:
- `UVEDDI_AST_CACHE_SIZE`: Maximum cache entries (default: 1000)
- `UVEDDI_AST_CACHE_TTL`: Cache TTL in seconds (future enhancement)
- `UVEDDI_MAX_FILE_SIZE`: Maximum file size to cache (default: 10MB)

### Step 5: Update All Cache Usage
Find and update all locations where the cache is accessed to use the new LRU cache methods.

## 🧪 Testing Requirements

### Unit Tests
Create comprehensive tests in `src/ast/tree_sitter_impl.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_size_limit() {
        let parser = AstParser::with_cache_size(3).unwrap();
        // Test that cache respects size limit
    }
    
    #[test]
    fn test_lru_eviction() {
        let parser = AstParser::with_cache_size(2).unwrap();
        // Test LRU eviction behavior
    }
    
    #[test]
    fn test_cache_metrics() {
        let parser = AstParser::new().unwrap();
        // Test metrics tracking
    }
    
    #[test]
    fn test_efficient_clone() {
        let parser1 = AstParser::new().unwrap();
        let parser2 = parser1.clone();
        // Verify parsers are shared, not recreated
    }
    
    #[test]
    fn test_environment_configuration() {
        std::env::set_var("UVEDDI_AST_CACHE_SIZE", "500");
        let parser = AstParser::new().unwrap();
        // Verify configuration is respected
    }
}
```

### Integration Tests
Create `tests/cache_memory_stability.rs`:

```rust
#[tokio::test]
async fn test_memory_bounded_large_codebase() {
    // Simulate parsing many files
    // Verify memory usage stays bounded
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    // Test thread safety under concurrent load
}
```

## 📊 Success Criteria

### Functional Requirements
- [ ] LRU cache with configurable size limits implemented
- [ ] Memory usage bounded to reasonable limits (<500MB for 10,000 files)
- [ ] Cache hit/miss metrics available and accurate
- [ ] Environment variable configuration working
- [ ] Graceful degradation when cache is full
- [ ] Efficient Clone implementation (shared parsers)

### Performance Requirements
- [ ] <10% performance regression for cache hits
- [ ] Memory usage demonstrably bounded in tests
- [ ] Cache operations complete in <1ms
- [ ] No memory leaks under continuous operation

### Quality Requirements
- [ ] All unit tests pass
- [ ] Integration tests demonstrate memory stability
- [ ] Code compiles without warnings
- [ ] Proper error handling for all edge cases
- [ ] Comprehensive documentation

## 🔍 Validation Commands

Run these commands to validate your implementation:

```bash
# Compile check
cargo check --all-targets

# Run all tests
cargo test

# Run specific cache tests
cargo test cache

# Run with cache size environment variable
UVEDDI_AST_CACHE_SIZE=100 cargo test

# Memory usage test (if available)
cargo test test_memory_bounded_large_codebase --release

# Performance benchmark (if implemented)
cargo bench cache_performance

# Check for warnings
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

## 🚨 Critical Implementation Notes

### Error Handling
- Handle cache size of 0 gracefully
- Validate environment variable parsing
- Provide meaningful error messages

### Thread Safety
- Ensure all cache operations are thread-safe
- Use appropriate atomic operations for metrics
- Test concurrent access patterns

### Memory Management
- Use Arc for shared data to minimize cloning
- Implement proper Drop if needed for cleanup
- Monitor actual memory usage in tests

### Backward Compatibility
- Maintain existing public API where possible
- Ensure existing tests continue to pass
- Document any breaking changes

## 📈 Monitoring & Observability

After implementation, the cache should provide:

```rust
// Example usage for monitoring
let stats = parser.get_cache_stats();
println!("Cache hit rate: {:.2}%", stats.hit_rate * 100.0);
println!("Total evictions: {}", stats.evictions);
```

## 🎯 Definition of Done

- [ ] All code compiles without warnings
- [ ] All existing tests pass
- [ ] New comprehensive test suite passes
- [ ] Memory usage is demonstrably bounded
- [ ] Cache metrics are accurate and accessible
- [ ] Environment configuration works correctly
- [ ] Documentation is updated
- [ ] Performance regression is <10%
- [ ] Code review checklist completed

## 🚀 Next Steps After Completion

1. **Performance Monitoring**: Set up alerts for cache hit rates
2. **Memory Profiling**: Establish baseline memory usage metrics
3. **Production Deployment**: Gradual rollout with monitoring
4. **Documentation**: Update user guides with new configuration options
5. **Future Enhancements**: Consider sharded cache for higher concurrency

---

## 💬 Communication Protocol

### Progress Updates
Provide updates in this format:
```
## UV-152 Progress Update
**Status**: [In Progress/Testing/Complete]
**Completion**: X% complete
**Current Focus**: [What you're working on]
**Blockers**: [Any issues encountered]
**Next Steps**: [Immediate next actions]
**Test Results**: [Pass/Fail status]
```

### Questions/Blockers
When you need guidance:
```
## UV-152 Technical Question
**Context**: [Brief description]
**Question**: [Specific question]
**Options Considered**: [Your analysis]
**Recommendation Needed**: [What decision you need help with]
```

---

**Ready to implement the bounded cache solution for UV-152! 🚀**

Remember: This is a critical memory management fix that will prevent OOM crashes and enable analysis of large codebases. Focus on correctness, performance, and comprehensive testing.