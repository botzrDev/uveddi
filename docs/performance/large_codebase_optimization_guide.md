# Large Codebase Performance Optimization Guide

## Overview

This guide provides comprehensive optimization strategies for analyzing large codebases (>10,000 files) with Uveddi. The optimizations are based on architectural analysis and performance profiling, designed to achieve enterprise-scale performance while maintaining analysis accuracy.

## Performance Baseline and Targets

### Current Performance Characteristics
- **Small Codebases (100-1,000 files)**: 10-30 seconds analysis time
- **Medium Codebases (1,000-5,000 files)**: 1-5 minutes analysis time  
- **Large Codebases (5,000-10,000 files)**: 5-15 minutes analysis time
- **Extra Large Codebases (>10,000 files)**: 15-45 minutes analysis time

### Optimization Targets
- **>50% reduction in analysis time** for large codebases
- **Memory usage under 8GB** for codebases up to 50,000 files
- **Linear scalability** with file count rather than exponential
- **90%+ cache hit rate** on subsequent analysis runs
- **75%+ parallel processing efficiency**

## Architecture Overview

### Multi-Layered Optimization Strategy

The performance optimizations are implemented across multiple architectural layers:

1. **Large Codebase Optimizer** (`src/analysis/performance/large_codebase_optimizer.rs`)
2. **Database Query Optimizer** (`src/database/performance/query_optimizer.rs`)
3. **Advanced Cache Engine** (`src/analysis/cache/advanced_cache_engine.rs`)
4. **Multi-Language Parallel Processor** (`src/analysis/parallel/multi_language_processor.rs`)
5. **Performance Testing Framework** (`src/testing/performance_testing_framework.rs`)

## Configuration for Large Codebases

### Recommended Configuration

```rust
use uveddi::analysis::performance::large_codebase_optimizer::{
    LargeCodebaseOptimizer, LargeCodebaseConfig
};

// Configuration for >10k file codebases
let config = LargeCodebaseConfig {
    memory_limit_bytes: 8 * 1024 * 1024 * 1024, // 8GB
    batch_size: 2000,                            // 2000 files per batch
    worker_count: num_cpus::get().max(8),        // Use available cores
    min_file_size_threshold: 1024,               // Skip files <1KB
    max_file_size_threshold: 50 * 1024 * 1024,   // Stream files >50MB
    cache_compression_threshold: 10000,          // Compress after 10k entries
    enable_adaptive_partitioning: true,         // Intelligent file grouping
    memory_pressure_threshold: 0.75,            // Trigger relief at 75%
    enable_incremental_analysis: true,          // Use change detection
    
    parallel_config: ParallelProcessingConfig {
        max_concurrent_parsers: num_cpus::get() * 2,
        max_concurrent_detectors: 8,
        enable_work_stealing: true,
        language_worker_pools: {
            let mut pools = HashMap::new();
            pools.insert("rust".to_string(), num_cpus::get() / 2);
            pools.insert("python".to_string(), num_cpus::get() / 3);
            pools.insert("javascript".to_string(), num_cpus::get() / 3);
            pools.insert("typescript".to_string(), num_cpus::get() / 4);
            pools
        },
        ..Default::default()
    },
};
```

### Database Optimization Configuration

```rust
use uveddi::database::performance::query_optimizer::{
    QueryOptimizationConfig, ConnectionPoolConfig
};

let db_config = QueryOptimizationConfig {
    max_batch_size: 10000,
    optimal_batch_sizes: {
        let mut sizes = HashMap::new();
        sizes.insert("issues".to_string(), 5000);
        sizes.insert("dependencies".to_string(), 10000);
        sizes.insert("anti_patterns".to_string(), 1000);
        sizes
    },
    connection_pool_config: ConnectionPoolConfig {
        max_connections: 20,
        min_idle_connections: 5,
        acquire_timeout_seconds: 30,
        idle_timeout_seconds: 600,
        max_lifetime_seconds: 3600,
        enable_health_checks: true,
    },
    enable_prepared_statements: true,
    enable_result_caching: true,
    ..Default::default()
};
```

### Cache Configuration for Large Codebases

```rust
use uveddi::analysis::cache::advanced_cache_engine::{
    AdvancedCacheConfig, MemoryCacheConfig, DiskCacheConfig
};

let cache_config = AdvancedCacheConfig {
    memory_cache: MemoryCacheConfig {
        max_memory_bytes: 4 * 1024 * 1024 * 1024, // 4GB memory cache
        ast_cache_size: 50000,                      // Cache 50k ASTs
        result_cache_size: 100000,                  // Cache 100k results
        enable_memory_mapping: true,                // Use memory mapping
        pressure_threshold: 0.8,                    // Relief at 80%
    },
    disk_cache: DiskCacheConfig {
        max_disk_bytes: 50 * 1024 * 1024 * 1024,   // 50GB disk cache
        enable_compression: true,                   // Compress disk storage
        chunk_size_bytes: 4 * 1024 * 1024,         // 4MB chunks
        retention_hours: 168,                       // 1 week retention
        enable_integrity_checks: true,
    },
    compression: CompressionConfig {
        algorithm: CompressionAlgorithm::Zstd,     // Fast compression
        compression_level: 3,                      // Balanced speed/size
        min_compression_size: 1024,                // Compress >1KB
        enable_adaptive_compression: true,
    },
    eviction_policy: EvictionPolicyConfig {
        policy: EvictionPolicy::AdaptiveLRU,       // Intelligent eviction
        proactive_eviction: true,                  // Prevent memory pressure
        trigger_threshold: 0.9,                    // Evict at 90% full
    },
    ..Default::default()
};
```

## Usage Examples

### Basic Large Codebase Analysis

```rust
use std::sync::Arc;
use uveddi::analysis::performance::large_codebase_optimizer::LargeCodebaseOptimizer;
use uveddi::analysis::AnalysisEngine;
use uveddi::database::scalable_manager::ScalableDatabase;

async fn analyze_large_codebase() -> Result<(), Box<dyn std::error::Error>> {
    // Create optimized configuration
    let config = LargeCodebaseConfig::default(); // Use defaults optimized for large codebases
    
    // Create base analysis engine
    let analysis_engine = AnalysisEngine::builder()
        .with_parallel_processing(true)
        .enable_plugins(true)
        .build()?;
    
    // Create scalable database
    let database = Arc::new(ScalableDatabase::new(
        DatabaseConfig::for_large_codebases()
    ).await?);
    
    // Create large codebase optimizer
    let optimizer = LargeCodebaseOptimizer::new(
        config, 
        analysis_engine, 
        database
    ).await?;
    
    // Analyze large codebase
    let codebase_path = Path::new("/path/to/large/codebase");
    let (issues, dependency_graph) = optimizer
        .analyze_large_codebase(codebase_path)
        .await?;
    
    println!("Analysis completed: {} issues found", issues.len());
    println!("Dependency graph: {} nodes", dependency_graph.node_count());
    
    // Get performance metrics
    let metrics = optimizer.get_metrics().await;
    println!("Performance metrics: {:#?}", metrics);
    
    Ok(())
}
```

### Incremental Analysis for Development Workflows

```rust
async fn incremental_analysis_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let config = LargeCodebaseConfig {
        enable_incremental_analysis: true,
        enable_adaptive_partitioning: true,
        memory_pressure_threshold: 0.7, // More aggressive for development
        ..Default::default()
    };
    
    let optimizer = LargeCodebaseOptimizer::new(config, engine, database).await?;
    
    // First analysis (full)
    let (issues1, graph1) = optimizer.analyze_large_codebase(&codebase_path).await?;
    println!("Initial analysis: {} issues", issues1.len());
    
    // Subsequent analysis (incremental - should be much faster)
    let (issues2, graph2) = optimizer.analyze_large_codebase(&codebase_path).await?;
    println!("Incremental analysis: {} issues", issues2.len());
    
    let metrics = optimizer.get_metrics().await;
    println!("Time savings: {}%", 
        (metrics.analysis_time_savings_ms as f64 / metrics.analysis_time_savings_ms as f64) * 100.0
    );
    
    Ok(())
}
```

### Parallel Multi-Language Processing

```rust
use uveddi::analysis::parallel::multi_language_processor::MultiLanguageProcessor;

async fn parallel_multi_language_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let parallel_config = ParallelProcessingConfig {
        max_workers: 12,
        language_pools: {
            let mut pools = HashMap::new();
            pools.insert("rust".to_string(), LanguagePoolConfig {
                min_workers: 4,
                max_workers: 6,
                priority: LanguagePriority::High,
                memory_per_worker_mb: 512,
                specialized_detectors: vec![
                    "GodObjectDetector".to_string(),
                    "DeadCodeDetector".to_string()
                ],
            });
            pools.insert("typescript".to_string(), LanguagePoolConfig {
                min_workers: 2,
                max_workers: 4,
                priority: LanguagePriority::Medium,
                memory_per_worker_mb: 256,
                specialized_detectors: vec![
                    "ComplexityDetector".to_string()
                ],
            });
            pools
        },
        work_stealing: WorkStealingConfig {
            enable_stealing: true,
            steal_threshold: 10,
            strategy: WorkStealingStrategy::Adaptive,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let processor = MultiLanguageProcessor::new(parallel_config).await?;
    
    // Get list of files to process
    let file_paths = discover_source_files(&codebase_path)?;
    
    // Process with optimized parallel execution
    let analysis_engine = Arc::new(AnalysisEngine::new()?);
    let (issues, graph) = processor
        .process_files_parallel(file_paths, analysis_engine)
        .await?;
    
    // Get parallel processing metrics
    let metrics = processor.get_performance_metrics().await;
    println!("Parallelization efficiency: {:.2}%", 
        metrics.average_parallelization_factor * 100.0
    );
    
    Ok(())
}
```

### Performance Testing and Validation

```rust
use uveddi::testing::performance_testing_framework::PerformanceTestingFramework;

async fn validate_large_codebase_performance() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = PerformanceTestConfig {
        test_scenarios: vec![
            TestScenario::LargeCodebase,      // 5,000-10,000 files
            TestScenario::ExtraLargeCodebase, // 10,000+ files
        ],
        benchmarks: PerformanceBenchmarks {
            analysis_time_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert(TestScenario::LargeCodebase, Duration::from_secs(300));     // 5 minutes
                thresholds.insert(TestScenario::ExtraLargeCodebase, Duration::from_secs(600)); // 10 minutes
                thresholds
            },
            memory_usage_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert(TestScenario::LargeCodebase, 4096);      // 4GB
                thresholds.insert(TestScenario::ExtraLargeCodebase, 8192); // 8GB
                thresholds
            },
            cache_hit_rate_thresholds: 0.8, // 80% cache hit rate
            parallelization_efficiency_threshold: 0.7, // 70% efficiency
            ..Default::default()
        },
        ..Default::default()
    };
    
    let framework = PerformanceTestingFramework::new(test_config);
    let results = framework.execute_test_suite().await?;
    
    // Analyze results
    if results.summary.overall_performance_score > 80.0 {
        println!("Performance validation passed: {:.1}%", 
            results.summary.overall_performance_score);
    } else {
        println!("Performance issues detected:");
        for regression in &results.regressions {
            println!("- {}: {:.1}% regression in {}", 
                format!("{:?}", regression.scenario),
                regression.regression_percentage,
                regression.metric
            );
        }
    }
    
    // Print recommendations
    for recommendation in &results.recommendations {
        println!("Recommendation ({}): {}", 
            format!("{:?}", recommendation.category),
            recommendation.recommendation
        );
    }
    
    Ok(())
}
```

## Performance Optimization Strategies

### 1. Intelligent File Processing

#### File Prioritization Strategy
- **Critical Files First**: `main.rs`, `lib.rs`, `app.ts` - core architecture files
- **Size-Based Processing**: Large files (likely to have issues) processed with higher priority
- **Dependency-Aware Ordering**: Process dependency roots before dependents
- **Language-Specific Batching**: Group files by language for optimized parsing

#### File Filtering and Skipping
```rust
// Skip small files that are unlikely to have significant issues
min_file_size_threshold: 1024, // Skip files < 1KB

// Skip common non-source directories
skip_patterns: [
    "/target/", "/build/", "/dist/", "/node_modules/", 
    "/.git/", "/vendor/", "/__pycache__/"
]

// Skip binary and generated files
skip_extensions: [".so", ".dylib", ".exe", ".dll", ".jar", ".class"]
```

### 2. Memory Management Optimizations

#### Streaming Analysis
For codebases >10k files, use streaming analysis to prevent memory exhaustion:

```rust
// Process files in batches to manage memory
batch_size: 2000, // Process 2000 files at a time

// Trigger memory relief strategies
memory_pressure_threshold: 0.75, // At 75% memory usage

// Memory relief strategies:
// 1. Clear AST caches for processed files
// 2. Compress intermediate results
// 3. Force garbage collection
// 4. Reduce batch size dynamically
```

#### Memory-Mapped Files
For very large files, use memory mapping instead of loading entire files:

```rust
enable_memory_mapping: true,
max_file_size_threshold: 50 * 1024 * 1024, // 50MB threshold
```

### 3. Parallel Processing Optimizations

#### Language-Specific Worker Pools
Different languages have different parsing characteristics:

```rust
language_worker_pools: {
    "rust" => 6 workers,      // Rust parsing is CPU intensive
    "python" => 4 workers,    // Python is more I/O bound
    "javascript" => 4 workers, // Fast parsing, more workers
    "typescript" => 3 workers, // Type checking overhead
}
```

#### Work Stealing Implementation
When one language pool is overloaded, steal work from less busy pools:

```rust
work_stealing: WorkStealingConfig {
    enable_stealing: true,
    steal_threshold: 10,        // Steal when queue >10 items
    max_steal_count: 5,         // Steal up to 5 items
    strategy: WorkStealingStrategy::Adaptive, // ML-based stealing
}
```

### 4. Database Performance Optimizations

#### Bulk Operations
Instead of individual inserts, use bulk operations:

```rust
// Batch database operations
optimal_batch_sizes: {
    "issues" => 5000,           // Insert 5000 issues at once
    "dependencies" => 10000,    // Insert 10k dependencies at once
    "anti_patterns" => 1000,    // Insert 1000 patterns at once
}
```

#### Connection Pool Optimization
```rust
connection_pool_config: ConnectionPoolConfig {
    max_connections: 20,        // Enough for parallel workers
    min_idle_connections: 5,    // Keep connections warm
    acquire_timeout_seconds: 30, // Don't wait too long
}
```

#### Query Result Caching
Cache expensive query results:

```rust
enable_result_caching: true,    // Cache query results
cache_ttl_seconds: 300,         // 5-minute TTL
```

### 5. Advanced Caching Strategies

#### Multi-Tier Caching
1. **Memory Cache**: Hot data (recently accessed ASTs and results)
2. **Disk Cache**: Warm data (compressed ASTs and analysis results)  
3. **Distributed Cache**: Cold data (shared across multiple analysis runs)

#### Intelligent Cache Eviction
```rust
eviction_policy: EvictionPolicyConfig {
    policy: EvictionPolicy::AdaptiveLRU,  // Learn access patterns
    proactive_eviction: true,             // Prevent memory pressure
    trigger_threshold: 0.9,               // Start evicting at 90%
}
```

#### Cache Warming
Pre-populate caches with likely-to-be-accessed data:

```rust
warming_strategy: CacheWarmingConfig {
    enable_warming: true,
    strategy: WarmingStrategy::PredictiveWarming, // ML-based prediction
    trigger_conditions: [
        WarmingTrigger::ColdStart,
        WarmingTrigger::LowHitRate,
    ],
}
```

### 6. Incremental Analysis

#### Change Detection
Only re-analyze files that have changed:

```rust
enable_incremental_analysis: true,

change_detection: ChangeDetectionConfig {
    track_file_modifications: true,     // Monitor file mtime
    track_dependency_changes: true,     // Track transitive changes
    change_detection_algorithm: ChangeDetectionAlgorithm::HashBased,
}
```

#### Dependency-Aware Incremental Updates
When a file changes, only re-analyze its dependents:

```rust
dependency_tracking: DependencyTrackingConfig {
    build_dependency_graph: true,       // Build dependency graph
    track_transitive_dependencies: true, // Track indirect deps
    max_dependency_depth: 10,           // Limit analysis depth
}
```

## Monitoring and Observability

### Performance Metrics to Monitor

#### Analysis Performance
- **Throughput**: Files analyzed per second
- **Latency**: Time per file analysis
- **Queue Depth**: Files waiting to be processed
- **Worker Utilization**: Percentage of workers busy

#### Resource Utilization
- **Memory Usage**: Peak and average memory consumption
- **CPU Usage**: Per-core utilization during analysis
- **Disk I/O**: Read/write operations for caching
- **Network I/O**: Distributed cache operations

#### Cache Performance
- **Hit Rate**: Percentage of cache hits vs misses
- **Eviction Rate**: How often cache items are evicted
- **Cache Size**: Current cache memory/disk usage
- **Compression Ratio**: Space savings from compression

### Alerting Thresholds

```rust
monitoring_thresholds: MonitoringThresholds {
    memory_usage_alert: 0.85,          // Alert at 85% memory
    cpu_usage_alert: 0.90,             // Alert at 90% CPU
    cache_hit_rate_warning: 0.70,      // Warn if hit rate <70%
    analysis_time_regression: 0.20,    // Alert on 20% slowdown
    error_rate_threshold: 0.05,        // Alert on 5% error rate
}
```

### Performance Debugging

#### Profiling Tools
```bash
# Enable detailed profiling
UVEDDI_PROFILE=1 UVEDDI_PROFILE_MEMORY=1 uveddi analyze /large/codebase

# Generate flame graph
cargo run --release --features=profiling -- analyze /codebase --profile-output profile.svg

# Memory profiling with jemalloc
MALLOC_CONF=prof:true,lg_prof_interval:30 uveddi analyze /codebase
```

#### Performance Logs
```rust
// Enable detailed performance logging
RUST_LOG=uveddi::performance=debug,uveddi::analysis::performance=trace
```

## Troubleshooting Common Issues

### Memory Pressure Issues

**Symptoms**: Out of memory errors, slow performance, system swapping

**Solutions**:
1. Reduce batch size: `batch_size: 1000` instead of `2000`
2. Enable more aggressive caching: `memory_pressure_threshold: 0.60`
3. Use disk cache more: `max_memory_bytes: 2GB` instead of `4GB`
4. Enable incremental analysis: `enable_incremental_analysis: true`

### Poor Cache Hit Rates

**Symptoms**: Cache hit rate <50%, repeated file parsing

**Solutions**:
1. Increase cache sizes: `ast_cache_size: 100000`
2. Improve cache retention: `retention_hours: 336` (2 weeks)
3. Enable cache warming: `enable_warming: true`
4. Check cache compression: `enable_compression: true`

### Low Parallelization Efficiency

**Symptoms**: Parallelization efficiency <50%, high CPU idle time

**Solutions**:
1. Increase worker counts: `max_workers: num_cpus::get() * 2`
2. Enable work stealing: `enable_work_stealing: true`
3. Balance language pools: Adjust worker ratios based on file distribution
4. Reduce synchronization overhead: Increase batch sizes

### Database Performance Issues

**Symptoms**: Slow database operations, high database CPU usage

**Solutions**:
1. Increase batch sizes: `max_batch_size: 20000`
2. Optimize connection pool: `max_connections: 30`
3. Enable query caching: `enable_result_caching: true`
4. Use prepared statements: `enable_prepared_statements: true`

## Best Practices Summary

### Configuration Best Practices
1. **Start with defaults** and tune based on monitoring data
2. **Scale worker counts** with available CPU cores
3. **Set memory limits** to 75% of available system memory
4. **Enable all optimizations** for production use
5. **Use incremental analysis** for development workflows

### Development Workflow Best Practices
1. **Profile first analysis runs** to establish baselines
2. **Monitor memory usage** during large codebase analysis
3. **Use cache warming** for repeated analysis workflows
4. **Enable detailed logging** for performance troubleshooting
5. **Validate optimizations** with performance tests

### Production Deployment Best Practices
1. **Allocate sufficient memory**: Minimum 8GB for >10k file codebases
2. **Use SSD storage** for cache directories
3. **Monitor resource usage** continuously
4. **Set up alerting** for performance regressions
5. **Regular cache cleanup** to prevent disk space issues

## Conclusion

The large codebase optimizations in Uveddi provide comprehensive performance improvements through:

- **Multi-layered optimization**: From file processing to database queries
- **Intelligent resource management**: Memory pressure relief and adaptive scaling
- **Advanced caching strategies**: Multi-tier caching with compression and warming
- **Parallel processing**: Language-specific worker pools with work stealing
- **Incremental analysis**: Change detection with dependency tracking

These optimizations enable Uveddi to efficiently analyze enterprise-scale codebases while maintaining analysis accuracy and providing detailed performance insights.

For additional support or optimization consulting, please refer to the [Performance Troubleshooting Guide](./performance_troubleshooting.md) or contact the development team.