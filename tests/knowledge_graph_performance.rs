//! # Knowledge Graph Performance Tests
//!
//! Performance validation tests for cached knowledge graph operations.
//! Validates the 2-30x speedup target from caching.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use uveddi::engine::analysis::context::{AnalysisContext, FileInfo, ProjectContext};
use uveddi::engine::analysis::graph_pipeline::{GraphAwarePipeline, GraphPipelineConfig};
use uveddi::engine::analysis::pipeline::AnalysisPipeline;
use uveddi::engine::cache::{GraphCache, GraphCacheConfig};
use uveddi::engine::knowledge_graph::{GraphBuilder, KnowledgeGraph};
use uveddi::engine::knowledge_graph::relations::{GraphRelation, RelationType};
use uveddi::engine::parsing::{Relation, Symbol};
use uveddi::ast::SourceLanguage;

/// Performance benchmarks for graph operations
struct GraphPerformanceBenchmark {
    /// Sample contexts for testing
    contexts: Vec<AnalysisContext>,
    /// Expected cache hit ratios
    target_hit_ratios: HashMap<String, f64>,
}

impl GraphPerformanceBenchmark {
    /// Create benchmark with sample data
    fn new() -> Self {
        let contexts = Self::generate_test_contexts(100); // Generate 100 test files

        let mut target_hit_ratios = HashMap::new();
        target_hit_ratios.insert("graph_relations".to_string(), 0.85); // 85% hit ratio target
        target_hit_ratios.insert("graph_dependencies".to_string(), 0.75); // 75% hit ratio target
        target_hit_ratios.insert("graph_queries".to_string(), 0.90); // 90% hit ratio target

        Self {
            contexts,
            target_hit_ratios,
        }
    }

    /// Generate test contexts with varying dependency patterns
    fn generate_test_contexts(count: usize) -> Vec<AnalysisContext> {
        let mut contexts = Vec::new();

        for i in 0..count {
            let file_path = PathBuf::from(format!("test_file_{}.rs", i));

            // Create symbols with dependencies
            let symbols = vec![
                Symbol {
                    name: format!("function_{}", i),
                    kind: "function".to_string(),
                    line: 10,
                    column: 5,
                },
                Symbol {
                    name: format!("struct_{}", i),
                    kind: "struct".to_string(),
                    line: 20,
                    column: 1,
                },
            ];

            // Create relations (dependencies between modules)
            let relations = vec![
                Relation {
                    from: format!("function_{}", i),
                    to: format!("struct_{}", (i + 1) % count), // Circular dependencies
                    kind: crate::engine::parsing::RelationKind::Uses,
                },
                Relation {
                    from: format!("struct_{}", i),
                    to: "std::vec::Vec".to_string(),
                    kind: crate::engine::parsing::RelationKind::Imports,
                },
            ];

            let context = AnalysisContext::new(
                FileInfo {
                    path: file_path,
                    language: SourceLanguage::Rust,
                    lines_of_code: 50 + (i % 200), // Varying file sizes
                    size_bytes: 1024 + (i * 100),
                    modified_at: std::time::SystemTime::now(),
                },
                None, // No syntax tree for performance tests
                format!("// Test file {}\nfn function_{}() {{}}\nstruct struct_{} {{}}", i, i, i),
                symbols,
                relations,
                ProjectContext {
                    project_root: PathBuf::from("/test_project"),
                    project_files: vec![],
                    dependencies: vec![],
                    global_symbols: vec![],
                },
            );

            contexts.push(context);
        }

        contexts
    }

    /// Benchmark cold graph building (no cache)
    async fn benchmark_cold_build(&self) -> BenchmarkResult {
        let start_time = Instant::now();
        let mut total_nodes = 0;
        let mut total_edges = 0;

        for context in &self.contexts {
            let mut builder = GraphBuilder::new();
            let _ = builder.build_from_context(context);
            let graph = builder.build();
            let stats = graph.stats();

            total_nodes += stats.node_count;
            total_edges += stats.edge_count;
        }

        let duration = start_time.elapsed();

        BenchmarkResult {
            duration,
            throughput: self.contexts.len() as f64 / duration.as_secs_f64(),
            nodes_processed: total_nodes,
            edges_processed: total_edges,
            cache_hit_ratio: 0.0, // No cache in cold build
        }
    }

    /// Benchmark warm graph building (with cache)
    async fn benchmark_warm_build(&self) -> BenchmarkResult {
        // Create graph-aware pipeline with caching
        let cache_config = GraphCacheConfig {
            max_entries: 5000,
            ttl: Duration::from_secs(3600),
            enable_cleanup: false,
            cleanup_interval: Duration::from_secs(300),
        };

        let pipeline_config = GraphPipelineConfig {
            incremental_build: true,
            graph_optimizations: true,
            max_graph_size: 50_000,
            dependency_ordering: true,
            cache_config,
        };

        let base_pipeline = AnalysisPipeline::new();
        let mut graph_pipeline = GraphAwarePipeline::new(base_pipeline, pipeline_config);

        let start_time = Instant::now();

        // First pass - populate cache
        let _ = graph_pipeline.analyze_with_graph(self.contexts.clone()).await;

        // Second pass - should hit cache
        let result = graph_pipeline.analyze_with_graph(self.contexts.clone()).await
            .expect("Graph analysis should succeed");

        let duration = start_time.elapsed();

        BenchmarkResult {
            duration,
            throughput: (self.contexts.len() * 2) as f64 / duration.as_secs_f64(), // Two passes
            nodes_processed: result.graph_stats.node_count,
            edges_processed: result.graph_stats.edge_count,
            cache_hit_ratio: result.cache_efficiency.hit_rate,
        }
    }

    /// Benchmark incremental graph updates
    async fn benchmark_incremental_updates(&self) -> BenchmarkResult {
        let cache_config = GraphCacheConfig::default();
        let mut cache = GraphCache::new(cache_config);

        let start_time = Instant::now();
        let mut updates_processed = 0;

        // Simulate incremental updates
        for (i, context) in self.contexts.iter().enumerate() {
            let file_path = context.file_info.path.to_string_lossy();
            let file_hash = i as u64; // Simple hash for testing

            // Create some relations
            let relations = vec![
                GraphRelation {
                    from: format!("{}::func", file_path),
                    to: "external::dependency".to_string(),
                    relation_type: RelationType::Uses,
                    file_path: file_path.to_string(),
                },
            ];

            // Cache the relations
            cache.cache_relations(format!("incremental_{}", i), relations, file_hash);
            updates_processed += 1;

            // Retrieve to test cache performance
            let _cached = cache.get_relations(&format!("incremental_{}", i), file_hash);
        }

        let duration = start_time.elapsed();

        BenchmarkResult {
            duration,
            throughput: updates_processed as f64 / duration.as_secs_f64(),
            nodes_processed: updates_processed,
            edges_processed: updates_processed,
            cache_hit_ratio: 0.5, // Approximation for incremental scenario
        }
    }

    /// Run comprehensive performance validation
    async fn run_performance_validation(&self) -> PerformanceValidationResult {
        println!("🚀 Running Knowledge Graph Performance Validation...");

        // Benchmark cold build
        println!("📊 Benchmarking cold graph building...");
        let cold_result = self.benchmark_cold_build().await;
        println!("   Cold build: {:.2} files/sec, {} nodes, {} edges",
                 cold_result.throughput, cold_result.nodes_processed, cold_result.edges_processed);

        // Benchmark warm build with cache
        println!("📊 Benchmarking warm graph building with cache...");
        let warm_result = self.benchmark_warm_build().await;
        println!("   Warm build: {:.2} files/sec, cache hit ratio: {:.2}%",
                 warm_result.throughput, warm_result.cache_hit_ratio * 100.0);

        // Benchmark incremental updates
        println!("📊 Benchmarking incremental updates...");
        let incremental_result = self.benchmark_incremental_updates().await;
        println!("   Incremental: {:.2} updates/sec", incremental_result.throughput);

        // Calculate speedup ratios
        let cache_speedup = warm_result.throughput / cold_result.throughput;
        let incremental_speedup = incremental_result.throughput / cold_result.throughput;

        println!("🎯 Performance Results:");
        println!("   Cache speedup: {:.1}x", cache_speedup);
        println!("   Incremental speedup: {:.1}x", incremental_speedup);
        println!("   Cache hit ratio: {:.1}%", warm_result.cache_hit_ratio * 100.0);

        PerformanceValidationResult {
            cold_performance: cold_result,
            warm_performance: warm_result,
            incremental_performance: incremental_result,
            cache_speedup,
            incremental_speedup,
            validation_passed: self.validate_performance_targets(cache_speedup, warm_result.cache_hit_ratio),
        }
    }

    /// Validate performance against targets
    fn validate_performance_targets(&self, cache_speedup: f64, hit_ratio: f64) -> bool {
        const MIN_SPEEDUP: f64 = 2.0; // Minimum 2x speedup target
        const MIN_HIT_RATIO: f64 = 0.75; // Minimum 75% cache hit ratio

        let speedup_ok = cache_speedup >= MIN_SPEEDUP;
        let hit_ratio_ok = hit_ratio >= MIN_HIT_RATIO;

        if !speedup_ok {
            println!("❌ Cache speedup {:.1}x below target {:.1}x", cache_speedup, MIN_SPEEDUP);
        }
        if !hit_ratio_ok {
            println!("❌ Cache hit ratio {:.1}% below target {:.1}%", hit_ratio * 100.0, MIN_HIT_RATIO * 100.0);
        }

        speedup_ok && hit_ratio_ok
    }
}

/// Result of a single benchmark
#[derive(Debug, Clone)]
struct BenchmarkResult {
    duration: Duration,
    throughput: f64, // operations per second
    nodes_processed: usize,
    edges_processed: usize,
    cache_hit_ratio: f64,
}

/// Result of complete performance validation
#[derive(Debug)]
struct PerformanceValidationResult {
    cold_performance: BenchmarkResult,
    warm_performance: BenchmarkResult,
    incremental_performance: BenchmarkResult,
    cache_speedup: f64,
    incremental_speedup: f64,
    validation_passed: bool,
}

#[tokio::test]
async fn test_knowledge_graph_cache_performance() {
    let benchmark = GraphPerformanceBenchmark::new();
    let results = benchmark.run_performance_validation().await;

    // Assert performance targets are met
    assert!(results.validation_passed,
            "Performance validation failed. Cache speedup: {:.1}x, Hit ratio: {:.1}%",
            results.cache_speedup, results.warm_performance.cache_hit_ratio * 100.0);

    // Assert minimum speedup of 2x
    assert!(results.cache_speedup >= 2.0,
            "Cache speedup {:.1}x is below minimum target of 2.0x", results.cache_speedup);

    // Assert decent cache hit ratio
    assert!(results.warm_performance.cache_hit_ratio >= 0.75,
            "Cache hit ratio {:.1}% is below target of 75%",
            results.warm_performance.cache_hit_ratio * 100.0);

    println!("✅ Knowledge graph cache performance validation passed!");
    println!("   🚀 Cache provides {:.1}x speedup with {:.1}% hit ratio",
             results.cache_speedup, results.warm_performance.cache_hit_ratio * 100.0);
}

#[tokio::test]
async fn test_graph_cache_memory_efficiency() {
    let cache_config = GraphCacheConfig {
        max_entries: 100, // Small limit to test eviction
        ttl: Duration::from_secs(1), // Short TTL to test expiration
        enable_cleanup: true,
        cleanup_interval: Duration::from_millis(100),
    };

    let mut cache = GraphCache::new(cache_config);

    // Fill cache beyond limit
    for i in 0..150 {
        let relations = vec![
            GraphRelation {
                from: format!("test_func_{}", i),
                to: "std::vec::Vec".to_string(),
                relation_type: RelationType::Uses,
                file_path: format!("test_{}.rs", i),
            }
        ];
        cache.cache_relations(format!("test_key_{}", i), relations, i as u64);
    }

    let stats = cache.get_cache_stats();

    // Should enforce size limits through eviction
    assert!(stats.relations_count <= 100,
            "Cache relations count {} exceeds limit of 100", stats.relations_count);

    println!("✅ Graph cache memory efficiency test passed!");
    println!("   📊 Cache entries: {} (within limit of 100)", stats.relations_count);
}

#[tokio::test]
async fn test_graph_cache_invalidation() {
    let mut cache = GraphCache::with_default_config();

    // Cache some relations for multiple files
    for i in 0..10 {
        let relations = vec![
            GraphRelation {
                from: format!("file:test_{}.rs::func", i),
                to: "std::vec::Vec".to_string(),
                relation_type: RelationType::Uses,
                file_path: format!("test_{}.rs", i),
            }
        ];
        cache.cache_relations(format!("file:test_{}.rs::key", i), relations, i as u64);
    }

    // Verify all entries are cached
    let initial_stats = cache.get_cache_stats();
    assert_eq!(initial_stats.relations_count, 10);

    // Invalidate one file
    cache.invalidate_file("test_5.rs");

    // Verify only that file's entries were removed
    let after_invalidation = cache.get_cache_stats();
    assert_eq!(after_invalidation.relations_count, 9,
               "Should have 9 entries after invalidating 1 file");

    // Verify specific entry is gone
    let invalidated_result = cache.get_relations("file:test_5.rs::key", 5);
    assert!(invalidated_result.is_none(), "Invalidated entry should not be found");

    // Verify other entries still exist
    let preserved_result = cache.get_relations("file:test_3.rs::key", 3);
    assert!(preserved_result.is_some(), "Other entries should be preserved");

    println!("✅ Graph cache invalidation test passed!");
    println!("   🗑️  Selective invalidation working correctly");
}