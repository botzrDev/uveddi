# UV-152 Comprehensive Analysis Report: Bounded Cache Implementation

## Executive Summary

**Issue**: UV-152 - HIGH: Implement bounded caches to prevent memory leaks  
**Status**: In Progress  
**Priority**: Medium (High Impact)  
**Assignee**: Phillip Austin Green  
**Created**: July 8, 2025  

### Critical Problem
The AST parser in `src/ast/tree_sitter_impl.rs` uses an unbounded `HashMap` cache that grows indefinitely, leading to memory exhaustion and potential OOM crashes when analyzing large codebases (10,000+ files consuming >1GB RAM).

### Root Cause Analysis
1. **Unbounded Cache**: Line 25 - `cache: Mutex<HashMap<String, ParsedFile>>` has no size limits
2. **Inefficient Cloning**: Lines 384-408 - Clone implementation recreates parsers unnecessarily
3. **No Eviction Policy**: Old/unused entries never removed from cache
4. **Missing Observability**: No metrics for cache performance monitoring

## Technical Analysis

### Current Problematic Implementation

```rust
// src/ast/tree_sitter_impl.rs:23-26
pub struct AstParser {
    parsers: HashMap<SourceLanguage, Parser>,
    cache: Mutex<HashMap<String, ParsedFile>>, // UNBOUNDED!
}
```

### Memory Impact Assessment

| Scenario | Current Behavior | Memory Impact | Risk Level |
|----------|------------------|---------------|------------|
| Small Projects (<100 files) | Acceptable performance | <50MB | Low |
| Medium Projects (1,000 files) | Noticeable memory growth | 100-500MB | Medium |
| Large Projects (10,000+ files) | Memory exhaustion | >1GB, OOM crashes | **CRITICAL** |
| Long-running Processes | Indefinite growth | Unbounded | **CRITICAL** |

### Performance Bottlenecks Identified

1. **Cache Growth**: No upper bound on memory usage
2. **Parser Recreation**: Clone method recreates parsers instead of sharing
3. **Lock Contention**: Single mutex for entire cache
4. **No Cache Metrics**: Unable to monitor hit/miss rates

## Proposed Solution Architecture

### 1. Bounded LRU Cache Implementation

```rust
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

pub struct AstParser {
    parsers: Arc<HashMap<SourceLanguage, Parser>>, // Shared parsers
    cache: Mutex<LruCache<String, ParsedFile>>,
    max_cache_size: NonZeroUsize,
    metrics: CacheMetrics,
}

impl AstParser {
    pub fn new() -> Result<Self, AstError> {
        let max_size = NonZeroUsize::new(
            std::env::var("UVEDDI_AST_CACHE_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000)
        ).unwrap();
        
        Ok(AstParser {
            parsers: Arc::new(Self::init_parsers()?),
            cache: Mutex::new(LruCache::new(max_size)),
            max_cache_size: max_size,
            metrics: CacheMetrics::new(),
        })
    }
}
```

### 2. Optimized Data Structures

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

### 3. Cache Metrics and Observability

```rust
#[derive(Debug, Default)]
pub struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    memory_usage: AtomicU64,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }
}
```

## Configuration Strategy

### Environment Variables
```bash
# Cache configuration
export UVEDDI_AST_CACHE_SIZE=1000        # Max cached files (default: 1000)
export UVEDDI_AST_CACHE_TTL=3600         # Cache TTL in seconds (default: 1 hour)
export UVEDDI_MAX_FILE_SIZE=10485760     # Max file size 10MB (default: 10MB)
export UVEDDI_CACHE_METRICS_ENABLED=true # Enable metrics collection
```

### Adaptive Sizing Strategy
```rust
fn calculate_optimal_cache_size() -> usize {
    let available_memory = get_available_memory();
    let estimated_file_size = 50_000; // 50KB average per parsed file
    let max_cache_memory = available_memory / 4; // Use 25% of available memory
    (max_cache_memory / estimated_file_size).max(100).min(10_000)
}
```

## Implementation Plan

### Phase 1: Core Cache Replacement (2 days)
- [ ] Add `lru` dependency to Cargo.toml
- [ ] Replace HashMap with LruCache in AstParser
- [ ] Implement configurable cache size
- [ ] Add basic metrics collection
- [ ] Update Clone implementation for efficiency

### Phase 2: Performance Optimization (1 day)
- [ ] Implement Arc-based shared data structures
- [ ] Add cache hit/miss tracking
- [ ] Implement memory usage monitoring
- [ ] Add graceful degradation for cache full scenarios

### Phase 3: Testing & Validation (1 day)
- [ ] Unit tests for cache behavior
- [ ] Load tests with large codebases
- [ ] Memory usage validation
- [ ] Performance regression testing

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_size_limit() {
        let parser = AstParser::with_cache_size(3).unwrap();
        // Test that cache evicts oldest entries when full
    }
    
    #[test]
    fn test_lru_eviction_policy() {
        // Verify LRU behavior
    }
    
    #[test]
    fn test_cache_metrics() {
        // Validate hit/miss tracking
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_large_codebase_memory_stability() {
    // Simulate parsing 10,000+ files
    // Verify memory usage stays bounded
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    // Test thread safety under load
}
```

### Performance Benchmarks
```rust
#[bench]
fn bench_cache_performance(b: &mut Bencher) {
    // Measure cache hit/miss performance
}
```

## Risk Assessment & Mitigation

### High Risk Areas
1. **Cache Thrashing**: If cache size too small for workload
   - **Mitigation**: Adaptive sizing based on available memory
   - **Monitoring**: Track eviction rates

2. **Lock Contention**: Single mutex under high concurrency
   - **Mitigation**: Consider sharded cache in future iterations
   - **Monitoring**: Track lock wait times

3. **Memory Overhead**: LRU tracking adds overhead
   - **Mitigation**: Benchmark memory usage vs. unbounded cache
   - **Monitoring**: Compare before/after memory profiles

### Medium Risk Areas
1. **Configuration Complexity**: Too many tuning parameters
   - **Mitigation**: Provide sensible defaults and documentation
2. **Backward Compatibility**: API changes
   - **Mitigation**: Maintain existing public interface

## Success Metrics

### Functional Requirements
- [ ] Memory usage bounded to configurable limits
- [ ] Cache hit rate >80% for typical workloads
- [ ] No OOM crashes under normal operation
- [ ] Graceful degradation when cache full

### Performance Requirements
- [ ] <10% performance regression for cache hits
- [ ] Memory usage <500MB for 10,000 file analysis
- [ ] Cache eviction time <1ms per operation

### Observability Requirements
- [ ] Real-time cache metrics available
- [ ] Memory usage monitoring
- [ ] Hit/miss rate tracking
- [ ] Eviction count monitoring

## Dependencies & Integration

### Required Dependencies
```toml
[dependencies]
lru = "0.12.0"  # LRU cache implementation
```

### Integration Points
1. **Analysis Engine**: Uses AstParser for file processing
2. **Result Cache**: Separate caching layer for analysis results
3. **Component Extractor**: May have similar unbounded cache issues
4. **Metrics System**: Integration with observability infrastructure

## Future Enhancements

### Short Term (Next Sprint)
- [ ] Extend bounded caching to other modules
- [ ] Add cache warming strategies
- [ ] Implement cache persistence across restarts

### Medium Term (Next Release)
- [ ] Sharded cache for better concurrency
- [ ] Adaptive cache sizing based on system resources
- [ ] Advanced eviction policies (LFU, TTL-based)

### Long Term (Future Releases)
- [ ] Distributed caching for multi-node deployments
- [ ] Machine learning-based cache optimization
- [ ] Integration with external cache systems (Redis, etc.)

## Acceptance Criteria

### Must Have
- [x] LRU cache with configurable size limits
- [ ] Memory usage bounded to reasonable limits (<500MB for large projects)
- [ ] Cache hit/miss metrics available
- [ ] Performance tests showing memory stability
- [ ] Configuration documentation
- [ ] Graceful degradation when cache is full

### Should Have
- [ ] Adaptive cache sizing
- [ ] Real-time metrics dashboard
- [ ] Cache warming on startup
- [ ] Comprehensive error handling

### Could Have
- [ ] Cache persistence across restarts
- [ ] Advanced eviction policies
- [ ] Integration with monitoring systems

## Conclusion

UV-152 represents a critical memory management issue that affects the stability and scalability of the Uveddi analysis engine. The proposed bounded LRU cache solution addresses the root cause while providing enhanced observability and configurability.

**Estimated Effort**: 4 days (as per original estimate of 8 hours, but comprehensive implementation requires more)  
**Risk Level**: Medium (well-understood problem with proven solutions)  
**Business Impact**: High (prevents OOM crashes, enables analysis of large codebases)

The implementation will follow a phased approach to minimize risk and ensure thorough testing at each stage. Success will be measured by memory stability, performance maintenance, and enhanced observability capabilities.

---

**Next Steps**:
1. Review and approve implementation plan
2. Add `lru` dependency to Cargo.toml
3. Begin Phase 1 implementation
4. Set up monitoring for cache metrics
5. Plan integration testing with large codebases