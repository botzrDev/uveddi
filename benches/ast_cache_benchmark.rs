use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use uveddi::analysis::cache::ast::{AstCache, CacheConfig};

fn create_benchmark_cache() -> (AstCache, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheConfig {
        max_memory_entries: 10000,
        max_memory_size_mb: 100,
        enable_disk_cache: false, // Disable for pure memory benchmarks
        disk_cache_path: temp_dir.path().to_path_buf(),
        enable_memory_mapping: false,
        lru_eviction_enabled: true,
        cache_metrics_enabled: true,
        enable_zero_copy: false,
        zero_copy_cache_dir: temp_dir.path().to_path_buf(),
        zero_copy_threshold_bytes: 1024,
    };
    let cache = AstCache::new(config).unwrap();
    (cache, temp_dir)
}

fn setup_test_files(temp_dir: &TempDir, count: usize) -> Vec<std::path::PathBuf> {
    (0..count)
        .map(|i| {
            let file = temp_dir.path().join(format!("bench_test_{}.rs", i));
            fs::write(&file, format!(
                "fn benchmark_function_{}() {{\n    let x = {};\n    println!(\"Value: {{}}\", x);\n}}", 
                i, i
            )).unwrap();
            file
        })
        .collect()
}

fn bench_cache_miss_performance(c: &mut Criterion) {
    let (cache, temp_dir) = create_benchmark_cache();
    let test_files = setup_test_files(&temp_dir, 1000);

    c.bench_function("cache_miss_1000_files", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(cache.get(file));
            }
        })
    });
}

#[cfg(not(feature = "tree-sitter"))]
fn bench_cache_hit_performance(c: &mut Criterion) {
    let (cache, temp_dir) = create_benchmark_cache();
    let test_files = setup_test_files(&temp_dir, 100);

    // Populate cache with test data
    for file in &test_files {
        let ast_data = CacheableAst {
            data: format!("mock_ast_data_for_{}", file.display()).into_bytes(),
            timestamp: std::time::SystemTime::now(),
            language: "rust".to_string(),
        };
        cache.store(file, ast_data).unwrap();
    }

    c.bench_function("cache_hit_100_files", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(cache.get(file));
            }
        })
    });
}

#[cfg(feature = "tree-sitter")]
fn bench_cache_hit_performance(c: &mut Criterion) {
    use tree_sitter::{Language, Parser};

    let (cache, temp_dir) = create_benchmark_cache();
    let test_files = setup_test_files(&temp_dir, 10); // Smaller set for tree-sitter due to complexity

    // Setup tree-sitter parser
    let mut parser = Parser::new();
    extern "C" {
        fn tree_sitter_rust() -> Language;
    }
    let language = unsafe { tree_sitter_rust() };
    parser.set_language(&language).unwrap();

    // Populate cache with actual trees
    for file in &test_files {
        let source_code = fs::read_to_string(file).unwrap();
        if let Some(tree) = parser.parse(&source_code, None) {
            cache.store(file, tree).unwrap();
        }
    }

    c.bench_function("cache_hit_10_tree_sitter_files", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(cache.get(file));
            }
        })
    });
}

fn bench_lru_update_performance(c: &mut Criterion) {
    let (cache, temp_dir) = create_benchmark_cache();
    let test_files = setup_test_files(&temp_dir, 1000);

    c.bench_function("lru_update_1000_files", |b| {
        b.iter(|| {
            for file in &test_files {
                // Use public API instead of private method
                let _ = cache.get(file); // This will update LRU order internally
            }
        })
    });
}

fn bench_hash_calculation_performance(c: &mut Criterion) {
    let (cache, temp_dir) = create_benchmark_cache();
    let test_files = setup_test_files(&temp_dir, 100);

    c.bench_function("hash_calculation_100_files", |b| {
        b.iter(|| {
            for file in &test_files {
                // Use public API - get() will calculate hash internally
                black_box(cache.get(file));
            }
        })
    });
}

fn bench_concurrent_access(c: &mut Criterion) {
    let (cache, temp_dir) = create_benchmark_cache();
    let cache = Arc::new(cache);
    let test_files = setup_test_files(&temp_dir, 100);

    c.bench_function("concurrent_access_4_threads", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|thread_id| {
                    let cache_clone = Arc::clone(&cache);
                    let files_clone = test_files.clone();

                    std::thread::spawn(move || {
                        for i in 0..25 {
                            // 25 * 4 = 100 total operations
                            let file_idx = (thread_id * 25 + i) % files_clone.len();
                            let file = &files_clone[file_idx];
                            black_box(cache_clone.get(file));
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

fn bench_memory_pressure(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheConfig {
        max_memory_entries: 50, // Small limit to trigger evictions
        max_memory_size_mb: 1,
        enable_disk_cache: false,
        disk_cache_path: temp_dir.path().to_path_buf(),
        enable_memory_mapping: false,
        lru_eviction_enabled: true,
        cache_metrics_enabled: true,
        enable_zero_copy: false,
        zero_copy_cache_dir: temp_dir.path().to_path_buf(),
        zero_copy_threshold_bytes: 1024,
    };
    let cache = AstCache::new(config).unwrap();
    let test_files = setup_test_files(&temp_dir, 100);

    c.bench_function("memory_pressure_eviction", |b| {
        b.iter(|| {
            // This should trigger multiple evictions through normal cache operations
            for file in &test_files {
                let _ = cache.get(file); // This will trigger LRU updates and potential evictions
            }
        })
    });
}

fn bench_metrics_collection(c: &mut Criterion) {
    let (cache, temp_dir) = create_benchmark_cache();
    let test_files = setup_test_files(&temp_dir, 100);

    // Generate some cache activity
    for file in &test_files {
        cache.get(file); // Cache miss - this will also update LRU order internally
    }

    c.bench_function("metrics_collection_and_export", |b| {
        b.iter(|| {
            black_box(cache.get_metrics());
            black_box(cache.export_metrics_for_observability());
        })
    });
}

criterion_group!(
    cache_benchmarks,
    bench_cache_miss_performance,
    bench_cache_hit_performance,
    bench_lru_update_performance,
    bench_hash_calculation_performance,
    bench_concurrent_access,
    bench_memory_pressure,
    bench_metrics_collection
);
criterion_main!(cache_benchmarks);
