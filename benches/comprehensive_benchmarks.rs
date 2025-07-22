use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use uveddi::analysis::buffer::{FixedString, LargeBuffer, MediumBuffer, SmallBuffer};
use uveddi::analysis::cache::ast::{AstCache, CacheConfig};
use uveddi::analysis::{
    detectors::anti_patterns::GodObjectDetector, AnalysisDetector, AnalysisEngine,
};

/// Creates a temporary directory with test Rust files of varying complexity
fn create_test_project(temp_dir: &TempDir, file_count: usize) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    for i in 0..file_count {
        let complexity = match i % 4 {
            0 => "simple",
            1 => "medium",
            2 => "complex",
            _ => "very_complex",
        };

        let file_path = temp_dir.path().join(format!("test_file_{}.rs", i));
        let content = generate_rust_code(complexity, i);
        fs::write(&file_path, content).unwrap();
        files.push(file_path);
    }

    files
}

/// Generates Rust code of varying complexity for benchmarking
fn generate_rust_code(complexity: &str, seed: usize) -> String {
    match complexity {
        "simple" => format!(
            "pub fn simple_function_{}() -> i32 {{\n    {} + 1\n}}\n",
            seed, seed
        ),
        "medium" => {
            let mut code = String::new();
            code.push_str(&format!("pub struct MediumStruct{} {{\n", seed));
            for i in 0..5 {
                code.push_str(&format!("    field_{}: i32,\n", i));
            }
            code.push_str("}\n\n");

            code.push_str(&format!("impl MediumStruct{} {{\n", seed));
            for i in 0..8 {
                code.push_str(&format!(
                    "    pub fn method_{}(&self) -> i32 {{\n        self.field_0 + {}\n    }}\n",
                    i, i
                ));
            }
            code.push_str("}\n");
            code
        }
        "complex" => {
            let mut code = String::new();
            code.push_str(&format!("pub struct ComplexStruct{} {{\n", seed));
            for i in 0..12 {
                code.push_str(&format!("    field_{}: i32,\n", i));
            }
            code.push_str("}\n\n");

            code.push_str(&format!("impl ComplexStruct{} {{\n", seed));
            for i in 0..15 {
                code.push_str(&format!(
                    "    pub fn method_{}(&self) -> i32 {{\n        if self.field_0 > {} {{\n            self.field_1 + {}\n        }} else {{\n            self.field_2 * {}\n        }}\n    }}\n",
                    i, i, i, i
                ));
            }
            code.push_str("}\n");
            code
        }
        "very_complex" => {
            let mut code = String::new();
            code.push_str(&format!("pub struct VeryComplexStruct{} {{\n", seed));
            for i in 0..25 {
                code.push_str(&format!("    field_{}: i32,\n", i));
            }
            code.push_str("}\n\n");

            code.push_str(&format!("impl VeryComplexStruct{} {{\n", seed));
            for i in 0..30 {
                code.push_str(&format!(
                    "    pub fn method_{}(&self) -> i32 {{\n        match self.field_0 % 3 {{\n            0 => self.field_{} + {},\n            1 => self.field_{} * {},\n            _ => self.field_{} - {},\n        }}\n    }}\n",
                    i, i % 10, i, (i + 1) % 10, i, (i + 2) % 10, i
                ));
            }
            code.push_str("}\n");
            code
        }
        _ => "".to_string(),
    }
}

/// Benchmark God Object detection across different file sizes
fn bench_god_object_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("god_object_detection");

    for &file_count in &[10, 50, 100, 500] {
        let temp_dir = TempDir::new().unwrap();
        let files = create_test_project(&temp_dir, file_count);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::new("files", file_count),
            &file_count,
            |b, _| {
                let detector = GodObjectDetector::default();
                b.iter(|| {
                    for file_path in &files {
                        if let Ok(source) = fs::read_to_string(file_path) {
                            // Simulate ParsedFile creation for benchmarking
                            let mock_parsed_file = create_mock_parsed_file(file_path, &source);
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            black_box(rt.block_on(detector.detect_issues(&mock_parsed_file)));
                        }
                    }
                });
            },
        );
    }
    group.finish();
}

/// Creates a mock ParsedFile for benchmarking purposes
fn create_mock_parsed_file(file_path: &Path, source: &str) -> uveddi::ast::ParsedFile {
    #[cfg(feature = "tree-sitter")]
    {
        use tree_sitter::{Language, Parser};

        let mut parser = Parser::new();
        extern "C" {
            fn tree_sitter_rust() -> Language;
        }
        let language = unsafe { tree_sitter_rust() };
        parser.set_language(&language).unwrap();

        let tree = parser.parse(source, None);

        uveddi::ast::ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            source: Arc::new(source.to_string()),
            tree,
            language: uveddi::ast::tree_sitter_impl::SourceLanguage::Rust,
            custom_ast: Arc::new(None),
            modified_at: std::time::SystemTime::now(),
        }
    }
    #[cfg(not(feature = "tree-sitter"))]
    {
        uveddi::ast::ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            source: Arc::new(source.to_string()),
            tree: None,
            language: uveddi::ast::tree_sitter_impl::SourceLanguage::Rust,
            custom_ast: Arc::new(None),
            modified_at: std::time::SystemTime::now(),
        }
    }
}

/// Benchmark full analysis pipeline with different configurations
fn bench_analysis_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_pipeline");

    for &file_count in &[10, 50, 100] {
        let temp_dir = TempDir::new().unwrap();
        let _files = create_test_project(&temp_dir, file_count);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::new("full_analysis", file_count),
            &file_count,
            |b, _| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut engine = AnalysisEngine::new().unwrap();
                        let result = engine.analyze(temp_dir.path()).await;
                        black_box(result);
                    });
                });
            },
        );
    }
    group.finish();
}

/// Benchmark memory usage patterns during analysis
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("memory_allocation_pattern", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();

            for _ in 0..iters {
                // Simulate memory allocation patterns during analysis
                let _cache = AstCache::new(CacheConfig::default()).unwrap();
                let _engine = AnalysisEngine::new().unwrap();
                // Simulate some analysis work
                let _buffer = vec![0u8; 1024 * 1024]; // 1MB allocation
            }

            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark const generic buffer performance
fn bench_const_generic_buffers(c: &mut Criterion) {
    let mut group = c.benchmark_group("const_generic_buffers");

    // Test different buffer sizes
    for &size in &[256, 1024, 8192] {
        group.bench_with_input(
            BenchmarkId::new("fixed_buffer_operations", size),
            &size,
            |b, &size| {
                b.iter(|| match size {
                    256 => {
                        let mut buffer = SmallBuffer::new();
                        for i in 0..200 {
                            buffer.push(i as u8).unwrap();
                        }
                        black_box(buffer.as_slice());
                    }
                    1024 => {
                        let mut buffer = MediumBuffer::new();
                        for i in 0..800 {
                            buffer.push(i as u8).unwrap();
                        }
                        black_box(buffer.as_slice());
                    }
                    8192 => {
                        let mut buffer = LargeBuffer::new();
                        for i in 0..6000 {
                            buffer.push(i as u8).unwrap();
                        }
                        black_box(buffer.as_slice());
                    }
                    _ => {}
                });
            },
        );
    }

    // Compare with Vec<u8>
    group.bench_function("vec_u8_operations", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(1024);
            for i in 0..800 {
                buffer.push(i as u8);
            }
            black_box(buffer.as_slice());
        });
    });

    group.finish();
}

/// Benchmark string operations with const generics
fn bench_const_generic_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("const_generic_strings");

    group.bench_function("fixed_string_operations", |b| {
        b.iter(|| {
            let mut string = FixedString::<1024>::new();
            for i in 0..100 {
                string.push_str(&format!("Item {}", i)).unwrap();
                if i < 99 {
                    string.push_str(", ").unwrap();
                }
            }
            black_box(string.as_str());
        });
    });

    group.bench_function("string_operations", |b| {
        b.iter(|| {
            let mut string = String::with_capacity(1024);
            for i in 0..100 {
                string.push_str(&format!("Item {}", i));
                if i < 99 {
                    string.push_str(", ");
                }
            }
            black_box(string.as_str());
        });
    });

    group.finish();
}

/// Benchmark AST cache performance with different configurations
fn bench_ast_cache_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_cache_configurations");

    let temp_dir = TempDir::new().unwrap();
    let files = create_test_project(&temp_dir, 100);

    // Test different cache configurations
    let configs = vec![
        (
            "small_cache",
            CacheConfig {
                max_memory_entries: 100,
                max_memory_size_mb: 10,
                ..Default::default()
            },
        ),
        (
            "medium_cache",
            CacheConfig {
                max_memory_entries: 1000,
                max_memory_size_mb: 50,
                ..Default::default()
            },
        ),
        (
            "large_cache",
            CacheConfig {
                max_memory_entries: 10000,
                max_memory_size_mb: 500,
                ..Default::default()
            },
        ),
    ];

    for (name, config) in configs {
        group.bench_function(name, |b| {
            let cache = AstCache::new(config.clone()).unwrap();
            b.iter(|| {
                for file in &files {
                    black_box(cache.get(file));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("error_creation_and_propagation", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let result: Result<i32, uveddi::error::UveddiError> = if i % 10 == 0 {
                    Err(uveddi::error::UveddiError::ConfigError {
                        message: format!("Error {}", i),
                        location: "benchmark".to_string(),
                        suggestion: "Check configuration".to_string(),
                    })
                } else {
                    Ok(i)
                };

                match result {
                    Ok(value) => {
                        black_box(value);
                    }
                    Err(e) => {
                        black_box(e.category());
                        black_box(e.severity());
                    }
                }
            }
        });
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    group.bench_function("concurrent_analysis", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let _files = create_test_project(&temp_dir, 50);

                let handles = (0..4).map(|_| {
                    let path = temp_dir.path().to_path_buf();
                    tokio::spawn(async move {
                        let mut engine = AnalysisEngine::new().unwrap();
                        let result = engine.analyze(&path).await;
                        black_box(result);
                    })
                });

                futures::future::join_all(handles).await;
            });
        });
    });

    group.finish();
}

/// Benchmark regression tests for performance
fn bench_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_regression");

    // This benchmark serves as a regression test for overall performance
    group.bench_function("baseline_performance", |b| {
        let temp_dir = TempDir::new().unwrap();
        let _files = create_test_project(&temp_dir, 25);

        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut engine = AnalysisEngine::new().unwrap();
                let result = engine.analyze(temp_dir.path()).await;
                black_box(result);
            });
        });
    });

    group.finish();
}

criterion_group!(
    comprehensive_benchmarks,
    bench_god_object_detection,
    bench_analysis_pipeline,
    bench_memory_usage,
    bench_const_generic_buffers,
    bench_const_generic_strings,
    bench_ast_cache_configurations,
    bench_error_handling,
    bench_concurrent_operations,
    bench_performance_regression
);

criterion_main!(comprehensive_benchmarks);
