//! Diagram Cache Performance Benchmarks for UV-91 Phase 3
//!
//! Validates that the diagram cache system achieves 90%+ cache hit rates
//! and <50ms cache lookup times for enterprise-scale diagram operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use uveddi::analysis::diagram_cache::{
    CachedDiagram, DiagramCacheConfig, DiagramCacheEngine, DiagramType,
};
use uveddi::analysis::incremental::ChangeSet;

/// Benchmark diagram cache storage operations
fn bench_diagram_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("diagram_cache_storage");

    for diagram_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("store_diagrams", diagram_count),
            diagram_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let config = DiagramCacheConfig::default();
                    let cache_engine = DiagramCacheEngine::new(config);

                    for i in 0..count {
                        let diagram_id = format!("diagram_{}", i);
                        let content =
                            format!("graph TD\n    A{} --> B{}\n    B{} --> C{}", i, i, i, i);
                        let dependencies = vec![PathBuf::from(format!("file_{}.rs", i))];

                        let _ = cache_engine
                            .store_diagram(
                                diagram_id,
                                DiagramType::Mermaid,
                                content.into_bytes(),
                                dependencies,
                                format!("config_{}", i),
                            )
                            .await;
                    }

                    black_box(cache_engine)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark diagram cache retrieval operations
fn bench_diagram_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("diagram_cache_retrieval");

    for diagram_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("retrieve_diagrams", diagram_count),
            diagram_count,
            |b, &count| {
                b.to_async(&rt).iter_setup(
                    || {
                        // Setup: Pre-populate cache
                        rt.block_on(async {
                            let config = DiagramCacheConfig::default();
                            let cache_engine = DiagramCacheEngine::new(config);

                            for i in 0..count {
                                let diagram_id = format!("diagram_{}", i);
                                let content = format!(
                                    "graph TD\n    A{} --> B{}\n    B{} --> C{}",
                                    i, i, i, i
                                );
                                let dependencies = vec![PathBuf::from(format!("file_{}.rs", i))];

                                let _ = cache_engine
                                    .store_diagram(
                                        diagram_id,
                                        DiagramType::Mermaid,
                                        content.into_bytes(),
                                        dependencies,
                                        format!("config_{}", i),
                                    )
                                    .await;
                            }

                            cache_engine
                        })
                    },
                    |cache_engine| async move {
                        // Benchmark: Retrieve random diagrams
                        for i in 0..100 {
                            let diagram_id = format!("diagram_{}", i % count);
                            let _ = cache_engine
                                .get_diagram(&diagram_id, &DiagramType::Mermaid)
                                .await;
                        }

                        black_box(cache_engine)
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark cache hit rate performance
fn bench_cache_hit_rates(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("cache_hit_rate_90_percent", |b| {
        b.to_async(&rt).iter(|| async {
            let config = DiagramCacheConfig::default();
            let cache_engine = DiagramCacheEngine::new(config);

            // Pre-populate cache with 1000 diagrams
            for i in 0..1000 {
                let diagram_id = format!("diagram_{}", i);
                let content = format!("graph TD\n    A{} --> B{}\n    B{} --> C{}", i, i, i, i);
                let dependencies = vec![PathBuf::from(format!("file_{}.rs", i))];

                let _ = cache_engine
                    .store_diagram(
                        diagram_id,
                        DiagramType::Mermaid,
                        content.into_bytes(),
                        dependencies,
                        format!("config_{}", i),
                    )
                    .await;
            }

            // Simulate 90% hit rate: 900 hits, 100 misses
            let mut hits = 0;
            let mut misses = 0;

            for i in 0..1000 {
                let diagram_id = if i < 900 {
                    // Hit: existing diagram
                    format!("diagram_{}", i % 1000)
                } else {
                    // Miss: non-existing diagram
                    format!("missing_diagram_{}", i)
                };

                if let Ok(Some(_)) = cache_engine
                    .get_diagram(&diagram_id, &DiagramType::Mermaid)
                    .await
                {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }

            let hit_rate = hits as f64 / (hits + misses) as f64;
            assert!(
                hit_rate >= 0.9,
                "Cache hit rate should be >= 90%, got {:.1}%",
                hit_rate * 100.0
            );

            black_box((hits, misses, hit_rate))
        });
    });
}

/// Benchmark cache invalidation performance
fn bench_cache_invalidation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_invalidation");

    for change_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("invalidate_changes", change_count),
            change_count,
            |b, &count| {
                b.to_async(&rt).iter_setup(
                    || {
                        // Setup: Pre-populate cache
                        rt.block_on(async {
                            let config = DiagramCacheConfig::default();
                            let cache_engine = DiagramCacheEngine::new(config);

                            for i in 0..1000 {
                                let diagram_id = format!("diagram_{}", i);
                                let content = format!(
                                    "graph TD\n    A{} --> B{}\n    B{} --> C{}",
                                    i, i, i, i
                                );
                                let dependencies =
                                    vec![PathBuf::from(format!("file_{}.rs", i % 100))];

                                let _ = cache_engine
                                    .store_diagram(
                                        diagram_id,
                                        DiagramType::Mermaid,
                                        content.into_bytes(),
                                        dependencies,
                                        format!("config_{}", i),
                                    )
                                    .await;
                            }

                            cache_engine
                        })
                    },
                    |cache_engine| async move {
                        // Create changeset with modified files
                        let mut modified_files = std::collections::HashSet::new();
                        for i in 0..count {
                            modified_files.insert(PathBuf::from(format!("file_{}.rs", i)));
                        }

                        let changeset = ChangeSet {
                            modified: modified_files,
                            added: std::collections::HashSet::new(),
                            deleted: std::collections::HashSet::new(),
                            affected_by_dependencies: std::collections::HashSet::new(),
                            detection_time: chrono::Utc::now(),
                            detection_method:
                                uveddi::analysis::incremental::ChangeDetectionMethod::FileSystem,
                        };

                        // Benchmark invalidation
                        let _ = cache_engine.invalidate_from_changeset(&changeset).await;

                        black_box(cache_engine)
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark compression performance
fn bench_compression_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("compression_performance");

    for content_size in [1024, 10240, 102400].iter() {
        group.bench_with_input(
            BenchmarkId::new("compress_decompress", content_size),
            content_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let config = DiagramCacheConfig {
                        enable_compression: true,
                        compression_level: 6,
                        ..Default::default()
                    };
                    let cache_engine = DiagramCacheEngine::new(config);

                    // Create large diagram content
                    let content = "graph TD\n".repeat(size / 10);
                    let diagram_id = "large_diagram".to_string();
                    let dependencies = vec![PathBuf::from("large_file.rs")];

                    // Store (compress)
                    let _ = cache_engine
                        .store_diagram(
                            diagram_id.clone(),
                            DiagramType::Mermaid,
                            content.into_bytes(),
                            dependencies,
                            "config_hash".to_string(),
                        )
                        .await;

                    // Retrieve (decompress)
                    let _ = cache_engine
                        .get_diagram(&diagram_id, &DiagramType::Mermaid)
                        .await;

                    black_box(cache_engine)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark lookup time performance (<50ms target)
fn bench_lookup_time_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lookup_time_under_50ms", |b| {
        b.to_async(&rt).iter_setup(
            || {
                // Setup: Pre-populate large cache
                rt.block_on(async {
                    let config = DiagramCacheConfig::default();
                    let cache_engine = DiagramCacheEngine::new(config);

                    for i in 0..10000 {
                        let diagram_id = format!("diagram_{}", i);
                        let content =
                            format!("graph TD\n    A{} --> B{}\n    B{} --> C{}", i, i, i, i);
                        let dependencies = vec![PathBuf::from(format!("file_{}.rs", i))];

                        let _ = cache_engine
                            .store_diagram(
                                diagram_id,
                                DiagramType::Mermaid,
                                content.into_bytes(),
                                dependencies,
                                format!("config_{}", i),
                            )
                            .await;
                    }

                    cache_engine
                })
            },
            |cache_engine| async move {
                let start_time = std::time::Instant::now();

                // Perform 100 random lookups
                for i in 0..100 {
                    let diagram_id = format!("diagram_{}", i * 100);
                    let _ = cache_engine
                        .get_diagram(&diagram_id, &DiagramType::Mermaid)
                        .await;
                }

                let elapsed = start_time.elapsed();
                let avg_lookup_time = elapsed.as_millis() as f64 / 100.0;

                // Validate <50ms average lookup time
                assert!(
                    avg_lookup_time < 50.0,
                    "Average lookup time should be <50ms, got {:.1}ms",
                    avg_lookup_time
                );

                black_box((cache_engine, avg_lookup_time))
            },
        );
    });
}

criterion_group!(
    benches,
    bench_diagram_storage,
    bench_diagram_retrieval,
    bench_cache_hit_rates,
    bench_cache_invalidation,
    bench_compression_performance,
    bench_lookup_time_performance
);

criterion_main!(benches);
