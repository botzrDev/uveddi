# Cache System Architecture Documentation

## Overview

The Uveddi cache system provides multi-layer intelligent caching that delivers 2-30x performance improvements through sophisticated caching strategies, automatic invalidation, and comprehensive monitoring.

## Architecture Components

### 1. Cache Layers

#### AST Cache (`src/engine/cache/ast_cache.rs`)
- **Purpose**: Caches parsed Abstract Syntax Trees to avoid expensive re-parsing
- **Hit Rate**: 85-95% in typical workflows  
- **Features**:
  - Content-based invalidation using file hashes
  - LRU eviction with configurable size limits
  - Memory usage tracking and optimization
  - Automatic cleanup of deleted files

**Key Methods:**
```rust
impl AstCache {
    // Retrieve cached AST
    pub fn get(&mut self, file_path: &Path) -> Option<CachedAstEntry>
    
    // Store AST with metadata
    pub fn put(&mut self, file_path: &Path, ast: ParsedAst, content_hash: u64) -> Result<(), CacheError>
    
    // Invalidate specific entry
    pub fn invalidate(&mut self, path: &Path) -> Result<(), CacheError>
    
    // Get performance statistics
    pub fn stats(&self) -> &CacheStats
}
```

#### Analysis Cache (`src/engine/cache/analysis_cache.rs`)
- **Purpose**: Caches detector analysis results to avoid re-running expensive analysis
- **Hit Rate**: 75-90% for typical codebases
- **Features**:
  - Per-detector result caching with version tracking
  - Incremental invalidation when dependencies change
  - Configurable retention policies
  - Memory-efficient storage of analysis results

**Key Methods:**
```rust
impl AnalysisCache {
    // Get cached analysis for specific detector
    pub fn get(&mut self, file_path: &Path, detector: &str) -> Option<&AnalysisResult>
    
    // Store detector results
    pub fn put(&mut self, file_path: PathBuf, detector: String, results: Vec<AnalysisResult>) -> Result<(), CacheError>
    
    // Invalidate by detector type
    pub fn invalidate_detector(&mut self, detector_name: &str)
}
```

#### Graph Cache (`src/engine/cache/graph_cache.rs`)
- **Purpose**: Caches knowledge graph relationships and computed graph traversals
- **Hit Rate**: 80-95% for relationship queries
- **Features**:
  - Relationship caching with dependency tracking  
  - Query result caching with TTL expiration
  - Incremental graph updates
  - Graph analytics caching

**Key Methods:**
```rust
impl GraphCache {
    // Cache computed relationships
    pub fn put_relations(&mut self, key: String, relations: CachedRelations)
    
    // Cache dependency analysis
    pub fn put_dependencies(&mut self, key: String, deps: Dependencies)
    
    // Cache graph query results
    pub fn put_query_result(&mut self, query_hash: String, result: CachedQueryResult)
}
```

### 2. Cache Coordination

#### Cache Handles (`src/engine/analysis/context.rs`)
Provides thread-safe access to all cache layers:

```rust
#[derive(Debug, Clone)]
pub struct CacheHandles {
    pub ast_cache: Arc<Mutex<AstCache>>,
    pub analysis_cache: Arc<Mutex<AnalysisCache>>,
    pub graph_cache: Arc<Mutex<GraphCache>>,
}
```

#### Cache Service Manager (`src/engine/cache/service.rs`)
Coordinates background cache services:

- **File Watcher**: Monitors file system for changes and invalidates stale entries
- **Metrics Collector**: Tracks performance and provides optimization recommendations  
- **Background Maintenance**: Cleanup, eviction, and memory management

```rust
impl CacheServiceManager {
    // Start all cache services
    pub fn start_services(&mut self, watch_paths: Vec<PathBuf>) -> Result<(), String>
    
    // Stop services gracefully
    pub fn stop_services(&mut self)
    
    // Check service health
    pub fn is_running(&self) -> bool
}
```

### 3. Intelligent Invalidation

#### File Watcher (`src/engine/cache/file_watcher.rs`)
Monitors file system changes and invalidates affected cache entries:

```rust
pub struct FileWatcher {
    // Configurable polling interval
    poll_interval: Duration,
    
    // File pattern matching
    watched_patterns: Vec<String>,
    
    // Timestamp tracking for change detection
    file_timestamps: HashMap<PathBuf, SystemTime>,
}

impl FileWatcher {
    // Synchronous polling for file changes
    pub fn poll_changes(&mut self) -> Result<(), String>
    
    // Manual cache invalidation
    pub fn invalidate_caches_for_file_sync(&self, path: &Path)
}
```

#### Invalidation Strategies
1. **File-based**: Invalidate when source files change
2. **Dependency-based**: Invalidate dependent analysis when dependencies change
3. **Time-based**: TTL expiration for certain cache types
4. **Memory-based**: LRU eviction when memory limits exceeded

### 4. Performance Monitoring

#### Metrics Collection (`src/engine/cache/metrics.rs`)
Comprehensive performance tracking and analysis:

```rust
pub struct CacheMetricsCollector {
    // Performance timing metrics
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    
    // Memory usage tracking
    memory_metrics: MemoryUsageMetrics,
    
    // Error tracking
    error_counts: ErrorCounts,
}

impl CacheMetricsCollector {
    // Record cache operations
    pub fn record_cache_hit(&self, cache_type: CacheType)
    pub fn record_cache_miss(&self, cache_type: CacheType)
    pub fn record_operation_time(&self, operation: CacheOperation, duration: Duration)
    
    // Generate performance reports
    pub fn generate_report(&self) -> String
    pub fn get_simplified_metrics(&self) -> SimplifiedMetrics
}
```

#### Tracked Metrics
- **Hit Rates**: Per-cache-type effectiveness
- **Operation Latency**: Get/put/eviction timing
- **Memory Usage**: Current and peak usage by cache type
- **Throughput**: Operations per second
- **Error Rates**: Cache failures and recovery

## Cache Integration Patterns

### 1. Analysis Pipeline Integration

The cache system integrates seamlessly with the analysis pipeline:

```rust
impl AnalysisPipeline {
    // Create pipeline with cache support
    pub fn with_caches(cache_handles: CacheHandles) -> Self
    
    // Start cache services
    pub fn start_cache_services(&self, watch_paths: Vec<PathBuf>) -> Result<(), PipelineError>
    
    // Build context with cache handles
    fn build_context(&self, file_path: &Path) -> Result<AnalysisContext, PipelineError>
}
```

### 2. Detector Integration

Detectors automatically benefit from caching through the AnalysisContext:

```rust
impl SecurityDetector {
    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Issue>, DetectorError> {
        // Context automatically provides cached AST and previous analysis results
        let ast = context.get_ast()?; // May come from cache
        let previous_results = context.get_cached_results("security")?;
        
        // Perform analysis with cache acceleration
        self.analyze_with_cache(ast, previous_results)
    }
}
```

### 3. Graph-Aware Caching

Knowledge graph construction benefits from multi-layer caching:

```rust
impl GraphBuilder {
    pub fn build_from_context(&mut self, context: &AnalysisContext) -> Result<(), GraphBuildError> {
        // Use cached ASTs and analysis results
        for file in context.files() {
            let ast = context.get_cached_ast(file)?; // Cache hit
            let analysis = context.get_cached_analysis(file)?; // Cache hit
            
            // Build graph incrementally with caching
            self.process_file_cached(file, ast, analysis)?;
        }
    }
}
```

## Configuration Guide

### Basic Configuration

```toml
[cache]
enabled = true

[cache.ast]
enabled = true
max_entries = 10000        # Maximum number of cached ASTs
max_memory_mb = 1024       # Memory limit in MB
eviction_policy = "lru"    # LRU, LFU, FIFO, TTL

[cache.analysis]
enabled = true
max_entries = 5000
max_memory_mb = 2048
eviction_policy = "lru"
ttl_seconds = 3600         # Optional TTL

[cache.graph]
enabled = true
max_entries = 1000
max_memory_mb = 512
query_cache_ttl = 300      # Query result TTL

[cache.file_watcher]
enabled = true
poll_interval_secs = 2
watch_patterns = [
    "**/*.rs",
    "**/*.py", 
    "**/*.js",
    "**/*.ts"
]
debounce_ms = 500          # Debounce file events

[cache.metrics]
enabled = true
collection_interval_secs = 30
detailed_logging = false
```

### Performance Tuning

#### For Small Codebases (< 1000 files)
```toml
[cache.ast]
max_entries = 2000
max_memory_mb = 256

[cache.analysis]  
max_entries = 1000
max_memory_mb = 512
```

#### For Large Enterprise Codebases (10000+ files)
```toml
[cache.ast]
max_entries = 50000
max_memory_mb = 8192

[cache.analysis]
max_entries = 25000  
max_memory_mb = 16384

[cache.memory]
max_total_memory_mb = 32768
enable_memory_pressure_monitoring = true
aggressive_eviction_threshold = 0.85
```

#### Memory-Constrained Environments
```toml
[cache]
enabled = true

[cache.memory]
max_total_memory_mb = 1024
enable_aggressive_eviction = true
eviction_threshold = 0.75

[cache.ast]
eviction_policy = "lfu"    # More memory efficient
ttl_seconds = 1800         # Shorter TTL

[cache.analysis]
eviction_policy = "ttl"
ttl_seconds = 900
```

## Performance Optimization

### 1. Cache Size Tuning

Monitor cache effectiveness and adjust sizes:

```bash
# Get cache statistics
curl http://localhost:3000/api/v1/cache/stats

# Monitor cache efficiency over time  
uveddi cache monitor --interval 30s --metrics hit_rate,memory_usage

# Get size recommendations
uveddi cache analyze-usage --duration 24h --recommend-sizes
```

### 2. Eviction Policy Selection

Choose eviction policies based on usage patterns:

- **LRU**: Best for typical development workflows
- **LFU**: Better for codebases with hot files
- **TTL**: Good for memory-constrained environments
- **Adaptive**: Automatically adjusts based on usage patterns

### 3. File Watcher Optimization

```toml
[cache.file_watcher]
# Faster polling for active development
poll_interval_secs = 1

# More specific patterns for better performance
watch_patterns = [
    "src/**/*.rs",           # Only source files
    "!**/target/**",         # Exclude build artifacts
    "!**/node_modules/**"    # Exclude dependencies
]

# Reduced debouncing for faster invalidation
debounce_ms = 100
```

### 4. Memory Management

Enable memory pressure monitoring:

```toml
[cache.memory]
enable_pressure_monitoring = true
pressure_threshold = 0.80           # Start eviction at 80%
emergency_threshold = 0.95          # Aggressive eviction at 95%
memory_check_interval_secs = 10     # Check every 10 seconds
```

## Monitoring and Observability

### 1. Real-time Metrics

WebSocket streaming provides real-time cache metrics:

```javascript
const ws = new WebSocket('ws://localhost:3001/api/v1/stream/cache');

ws.onmessage = (event) => {
    const metrics = JSON.parse(event.data);
    
    console.log(`AST Hit Rate: ${metrics.ast_hit_rate * 100}%`);
    console.log(`Analysis Hit Rate: ${metrics.analysis_hit_rate * 100}%`);
    console.log(`Memory Usage: ${metrics.memory_usage_mb}MB`);
    console.log(`Operations/sec: ${metrics.operations_per_second}`);
};
```

### 2. Performance Dashboards

Key metrics to monitor:

#### Cache Effectiveness
- Hit rates by cache type (target: >75%)
- Miss patterns and reasons
- Invalidation frequency and causes

#### Performance Impact  
- Analysis speedup over baseline (target: >2x)
- Time saved per analysis run
- Cost reduction calculations

#### Resource Usage
- Memory usage trends
- Eviction frequency and patterns  
- I/O patterns for cache persistence

#### System Health
- Cache service uptime
- File watcher responsiveness
- Error rates and recovery patterns

### 3. Alerting Rules

Recommended Prometheus alerts:

```yaml
groups:
  - name: cache_performance
    rules:
      - alert: LowCacheHitRate
        expr: cache_hit_rate < 0.6
        for: 5m
        annotations:
          summary: "Cache hit rate below 60%"
          
      - alert: HighMemoryUsage
        expr: cache_memory_usage_ratio > 0.9
        for: 2m
        annotations:
          summary: "Cache memory usage above 90%"
          
      - alert: CacheServiceDown  
        expr: up{job="uveddi-cache"} == 0
        for: 1m
        annotations:
          summary: "Cache service is down"
```

## Troubleshooting Guide

### Common Issues

#### Low Hit Rates
**Symptoms**: Hit rates below 60%, poor performance gains
**Causes**: 
- Frequent file modifications invalidating cache
- Cache sizes too small for codebase
- Ineffective eviction policies

**Solutions**:
- Increase cache size limits
- Adjust file watcher debouncing
- Switch to more appropriate eviction policy
- Check for memory pressure causing premature eviction

#### High Memory Usage
**Symptoms**: Memory usage approaching limits, frequent evictions
**Causes**:
- Cache sizes too large for available memory
- Memory leaks in cached data
- Inefficient eviction policies

**Solutions**:
- Reduce cache size limits
- Enable memory pressure monitoring
- Switch to more aggressive eviction policies
- Check for memory leaks in analysis results

#### File Watcher Issues
**Symptoms**: Stale cache entries, incorrect analysis results
**Causes**:
- File watcher not detecting changes
- Polling interval too long
- Pattern matching issues

**Solutions**:
- Reduce polling interval
- Check file patterns match your codebase
- Verify file system permissions
- Enable debug logging for file watcher

### Debug Commands

```bash
# Check cache health
uveddi cache health-check

# Validate cache integrity  
uveddi cache validate --deep

# Clear problematic cache entries
uveddi cache clear --type analysis --reason debugging

# Test file watcher
uveddi cache test-watcher --path /path/to/test

# Generate diagnostic report
uveddi cache diagnose --output cache-debug.json
```

### Performance Analysis

```bash
# Profile cache performance
uveddi cache profile --duration 300s --output profile.json

# Compare cache vs no-cache performance
uveddi benchmark --with-cache --without-cache /path/to/codebase

# Analyze cache usage patterns
uveddi cache analyze-patterns --duration 24h --detailed
```

## Best Practices

### 1. Development Workflow

- Enable caching in development for faster iteration
- Use smaller cache sizes in development environments
- Monitor cache effectiveness during development
- Clear caches when switching branches or major changes

### 2. Production Deployment

- Size caches based on actual usage patterns
- Enable comprehensive monitoring and alerting
- Use persistent storage for cache data
- Implement cache warming strategies for faster startup

### 3. Performance Optimization

- Monitor hit rates and adjust cache sizes accordingly
- Use appropriate eviction policies for your workload
- Enable memory pressure monitoring in production
- Implement cache preloading for common analysis patterns

### 4. Maintenance

- Regularly review cache performance metrics
- Update cache configurations based on usage patterns
- Monitor for cache-related memory leaks
- Keep file watcher patterns updated as codebase evolves

---

This cache architecture provides the foundation for Uveddi's 2-30x performance improvements while maintaining data integrity and providing comprehensive observability into cache performance and behavior.