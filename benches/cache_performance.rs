//! # Cache Performance Validation Benchmarks
//!
//! This module provides comprehensive benchmarks for validating cache system
//! performance in various scenarios, comparing cached vs non-cached analysis
//! runs to measure performance improvements and identify optimization opportunities.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tempfile::{tempdir, TempDir};

use uveddi::analysis::components::cache_manager::{CacheManager, CacheManagerImpl};
use uveddi::analysis::detector_factory::DetectorFactory;
use uveddi::analysis::engine::{AnalysisEngine, AnalysisEngineBuilder};
use uveddi::ast::tree_sitter_impl::{AstParser, ParsedFile};
use uveddi::ast::SourceLanguage;
use uveddi::database::models::ArchitecturalIssue;
use uveddi::engine::analysis::context::{AnalysisContext, FileInfo, ProjectContext};

/// Test code samples for benchmarking different complexity levels
struct TestCodeSamples;

impl TestCodeSamples {
    fn small_rust_file() -> &'static str {
        r#"
fn simple_function() -> i32 {
    42
}

struct SimpleStruct {
    value: i32,
}

impl SimpleStruct {
    fn new(value: i32) -> Self {
        Self { value }
    }
}
"#
    }

    fn medium_rust_file() -> &'static str {
        r#"
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct DataProcessor {
    cache: HashMap<String, i32>,
    processing_time: Duration,
    state: Arc<Mutex<ProcessorState>>,
}

enum ProcessorState {
    Idle,
    Processing,
    Error(String),
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            processing_time: Duration::from_secs(0),
            state: Arc::new(Mutex::new(ProcessorState::Idle)),
        }
    }

    pub fn process_data(&mut self, input: &str) -> Result<i32, String> {
        let start = Instant::now();

        if let Some(&cached) = self.cache.get(input) {
            return Ok(cached);
        }

        let result = self.complex_processing(input)?;
        self.cache.insert(input.to_string(), result);
        self.processing_time = start.elapsed();

        Ok(result)
    }

    fn complex_processing(&self, input: &str) -> Result<i32, String> {
        if input.is_empty() {
            return Err("Empty input".to_string());
        }

        let mut result = 0;
        for (i, char) in input.chars().enumerate() {
            result += (char as u32) * (i + 1) as u32;
        }

        Ok(result as i32)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

fn duplicate_function_1(x: i32) -> i32 {
    let mut sum = 0;
    for i in 0..10 {
        sum += x * i;
    }
    sum
}

fn duplicate_function_2(x: i32) -> i32 {
    let mut sum = 0;
    for i in 0..10 {
        sum += x * i;
    }
    sum
}
"#
    }

    fn large_rust_file() -> &'static str {
        r#"
use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

pub struct ComplexDataProcessor {
    primary_cache: HashMap<String, ComplexData>,
    secondary_cache: BTreeMap<i32, Vec<String>>,
    metadata_cache: HashSet<String>,
    state: Arc<RwLock<ProcessorState>>,
    metrics: Arc<Mutex<ProcessingMetrics>>,
    worker_pool: Vec<thread::JoinHandle<()>>,
}

#[derive(Clone, Debug)]
pub struct ComplexData {
    id: i32,
    name: String,
    values: Vec<f64>,
    nested: Option<Box<ComplexData>>,
    references: HashMap<String, i32>,
}

#[derive(Debug)]
enum ProcessorState {
    Initializing,
    Ready,
    Processing,
    Flushing,
    Error(String),
    Shutdown,
}

#[derive(Debug, Default)]
struct ProcessingMetrics {
    total_processed: u64,
    cache_hits: u64,
    cache_misses: u64,
    average_latency: Duration,
    error_count: u64,
}

impl ComplexDataProcessor {
    pub fn new() -> Self {
        Self {
            primary_cache: HashMap::new(),
            secondary_cache: BTreeMap::new(),
            metadata_cache: HashSet::new(),
            state: Arc::new(RwLock::new(ProcessorState::Initializing)),
            metrics: Arc::new(Mutex::new(ProcessingMetrics::default())),
            worker_pool: Vec::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        *self.state.write().unwrap() = ProcessorState::Ready;
        Ok(())
    }

    pub fn process_complex_data(&mut self, key: &str, data: &[u8]) -> Result<ComplexData, String> {
        let start = Instant::now();

        if let Some(cached) = self.primary_cache.get(key) {
            self.metrics.lock().unwrap().cache_hits += 1;
            return Ok(cached.clone());
        }

        self.metrics.lock().unwrap().cache_misses += 1;

        let processed = self.intensive_processing(key, data)?;
        self.primary_cache.insert(key.to_string(), processed.clone());

        let elapsed = start.elapsed();
        self.metrics.lock().unwrap().average_latency = elapsed;
        self.metrics.lock().unwrap().total_processed += 1;

        Ok(processed)
    }

    fn intensive_processing(&self, key: &str, data: &[u8]) -> Result<ComplexData, String> {
        // Simulate intensive processing
        thread::sleep(Duration::from_nanos(100)); // Very short sleep for benchmarking

        let mut values = Vec::new();
        for chunk in data.chunks(4) {
            let mut sum = 0.0;
            for &byte in chunk {
                sum += byte as f64;
            }
            values.push(sum / chunk.len() as f64);
        }

        let mut references = HashMap::new();
        for (i, &byte) in data.iter().enumerate().take(10) {
            references.insert(format!("ref_{}", i), byte as i32);
        }

        Ok(ComplexData {
            id: key.len() as i32,
            name: key.to_string(),
            values,
            nested: None,
            references,
        })
    }

    pub fn batch_process(&mut self, inputs: Vec<(&str, &[u8])>) -> Vec<Result<ComplexData, String>> {
        inputs
            .into_iter()
            .map(|(key, data)| self.process_complex_data(key, data))
            .collect()
    }

    pub fn clear_caches(&mut self) {
        self.primary_cache.clear();
        self.secondary_cache.clear();
        self.metadata_cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize, usize) {
        (
            self.primary_cache.len(),
            self.secondary_cache.len(),
            self.metadata_cache.len(),
        )
    }

    pub fn memory_usage_estimate(&self) -> usize {
        let primary_size = self.primary_cache.len() * 1024; // Rough estimate
        let secondary_size = self.secondary_cache.len() * 512;
        let metadata_size = self.metadata_cache.len() * 64;
        primary_size + secondary_size + metadata_size
    }
}

// God Object anti-pattern for detection
pub struct GodObject {
    field1: String,
    field2: i32,
    field3: Vec<u8>,
    field4: HashMap<String, String>,
    field5: Option<bool>,
    field6: Result<i32, String>,
    field7: Box<dyn std::fmt::Display>,
    field8: Arc<Mutex<i32>>,
    field9: std::cell::RefCell<String>,
    field10: std::rc::Rc<Vec<i32>>,
}

impl GodObject {
    pub fn method1(&self) -> i32 { 1 }
    pub fn method2(&mut self, x: i32) { self.field2 = x; }
    pub fn method3(&self) -> String { self.field1.clone() }
    pub fn method4(&self, s: &str) -> bool { self.field1.contains(s) }
    pub fn method5(&mut self) { self.field1.clear(); }
    pub fn method6(&self) -> Vec<u8> { self.field3.clone() }
    pub fn method7(&self, v: Vec<u8>) { /* ... */ }
    pub fn method8(&mut self, x: i32, y: String) { /* ... */ }
    pub fn method9(&self) -> Option<i32> { Some(self.field2) }
    pub fn method10(&self) -> Result<String, String> { Ok(self.field1.clone()) }
    pub fn method11(&self) -> bool { true }
    pub fn method12(&mut self) { /* ... */ }
}

// Dead code examples
fn unused_function_1() -> i32 { 42 }
fn unused_function_2(x: String) -> String { x }
fn unused_function_3(a: i32, b: i32) -> i32 { a + b }

// Code duplication examples
fn duplicate_logic_a() -> i32 {
    let mut result = 0;
    for i in 0..100 {
        result += i * 2;
    }
    result
}

fn duplicate_logic_b() -> i32 {
    let mut result = 0;
    for i in 0..100 {
        result += i * 2;
    }
    result
}

pub fn main() {
    let mut processor = ComplexDataProcessor::new();
    processor.initialize().unwrap();

    let test_data = vec![1u8, 2, 3, 4, 5];
    let _ = processor.process_complex_data("test", &test_data);
}
"#
    }
}

/// Create a temporary file with the given content
fn create_temp_file(content: &str, suffix: &str) -> (TempDir, PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join(format!("test{}", suffix));
    std::fs::write(&file_path, content).expect("Failed to write temp file");
    (temp_dir, file_path)
}

/// Benchmark cache vs non-cache AST parsing performance
fn benchmark_ast_parsing_cache(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("ast_cache_performance");

    // Test different file sizes
    let test_cases = vec![
        ("small", TestCodeSamples::small_rust_file()),
        ("medium", TestCodeSamples::medium_rust_file()),
        ("large", TestCodeSamples::large_rust_file()),
    ];

    for (size, content) in test_cases {
        let (_temp_dir, file_path) = create_temp_file(content, ".rs");

        group.throughput(Throughput::Bytes(content.len() as u64));

        // Benchmark without cache (direct parsing)
        group.bench_with_input(BenchmarkId::new("no_cache", size), &file_path, |b, path| {
            b.iter(|| {
                let mut parser = AstParser::new().expect("Failed to create parser");
                black_box(parser.parse_file(path).expect("Failed to parse file"));
            })
        });

        // Benchmark with cache manager
        group.bench_with_input(
            BenchmarkId::new("with_cache", size),
            &file_path,
            |b, path| {
                let cache_manager = rt.block_on(async {
                    CacheManagerImpl::new()
                        .await
                        .expect("Failed to create cache manager")
                });

                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            cache_manager
                                .get_or_parse_ast(path)
                                .await
                                .expect("Failed to get or parse AST"),
                        );
                    })
                })
            },
        );

        // Benchmark cache hit scenario (second access)
        group.bench_with_input(
            BenchmarkId::new("cache_hit", size),
            &file_path,
            |b, path| {
                let cache_manager = rt.block_on(async {
                    CacheManagerImpl::new()
                        .await
                        .expect("Failed to create cache manager")
                });

                // Prime the cache
                rt.block_on(async {
                    cache_manager
                        .get_or_parse_ast(path)
                        .await
                        .expect("Failed to prime cache");
                });

                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            cache_manager
                                .get_or_parse_ast(path)
                                .await
                                .expect("Failed to get cached AST"),
                        );
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark analysis result caching performance
fn benchmark_analysis_result_cache(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("analysis_result_cache");

    let (_temp_dir, file_path) = create_temp_file(TestCodeSamples::large_rust_file(), ".rs");

    // Create mock analysis results
    let sample_results = vec![
        ArchitecturalIssue {
            id: None,
            detector_name: "god_object".to_string(),
            issue_type: "anti_pattern".to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            start_line: 10,
            end_line: 50,
            start_column: 0,
            end_column: 1,
            message: "God object detected".to_string(),
            description: Some("This class has too many responsibilities".to_string()),
            severity: "medium".to_string(),
            confidence: 0.8,
            suggestion: Some("Consider breaking this into smaller classes".to_string()),
            metadata: None,
            created_at: None,
        },
        ArchitecturalIssue {
            id: None,
            detector_name: "dead_code".to_string(),
            issue_type: "maintainability".to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            start_line: 100,
            end_line: 105,
            start_column: 0,
            end_column: 20,
            message: "Dead code detected".to_string(),
            description: Some("This function is never called".to_string()),
            severity: "low".to_string(),
            confidence: 0.9,
            suggestion: Some("Remove unused code".to_string()),
            metadata: None,
            created_at: None,
        },
    ];

    // Benchmark caching analysis results
    group.bench_with_input(
        BenchmarkId::new("cache_results", "store"),
        &sample_results,
        |b, results| {
            let cache_manager = rt.block_on(async {
                CacheManagerImpl::new()
                    .await
                    .expect("Failed to create cache manager")
            });

            b.iter(|| {
                rt.block_on(async {
                    cache_manager
                        .cache_results(&file_path, black_box(results.clone()))
                        .await;
                })
            })
        },
    );

    // Benchmark retrieving cached results
    group.bench_with_input(
        BenchmarkId::new("retrieve_results", "cached"),
        &sample_results,
        |b, results| {
            let cache_manager = rt.block_on(async {
                CacheManagerImpl::new()
                    .await
                    .expect("Failed to create cache manager")
            });

            // Prime the cache
            rt.block_on(async {
                cache_manager
                    .cache_results(&file_path, results.clone())
                    .await;
            });

            b.iter(|| {
                rt.block_on(async {
                    black_box(cache_manager.get_cached_results(&file_path).await);
                })
            })
        },
    );

    group.finish();
}

/// Benchmark full analysis pipeline with and without caching
fn benchmark_full_analysis_pipeline(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("full_analysis_cache");

    let (_temp_dir, file_path) = create_temp_file(TestCodeSamples::large_rust_file(), ".rs");

    // Benchmark analysis with in-memory cache
    group.bench_with_input(
        BenchmarkId::new("with_cache", "in_memory"),
        &file_path,
        |b, path| {
            b.iter(|| {
                rt.block_on(async {
                    // Create engine with in-memory cache
                    let mut engine = AnalysisEngineBuilder::new()
                        .with_in_memory_cache()
                        .build()
                        .expect("Failed to create engine");

                    black_box(engine.analyze(path).await.expect("Failed to analyze file"));
                })
            })
        },
    );

    // Benchmark analysis with cache (warm cache simulation)
    group.bench_with_input(
        BenchmarkId::new("with_cache", "warm_simulation"),
        &file_path,
        |b, path| {
            let mut engine = rt.block_on(async {
                AnalysisEngineBuilder::new()
                    .with_in_memory_cache()
                    .build()
                    .expect("Failed to create engine")
            });

            // Prime the cache by running analysis once
            rt.block_on(async {
                let _ = engine.analyze(path).await;
            });

            b.iter(|| {
                rt.block_on(async {
                    black_box(engine.analyze(path).await.expect("Failed to analyze file"));
                })
            })
        },
    );

    // Benchmark cache operations directly
    group.bench_with_input(
        BenchmarkId::new("cache_manager", "direct_operations"),
        &file_path,
        |b, path| {
            let cache_manager = rt.block_on(async {
                CacheManagerImpl::new()
                    .await
                    .expect("Failed to create cache manager")
            });

            b.iter(|| {
                rt.block_on(async {
                    // Test cache operations
                    let ast_result = cache_manager.get_or_parse_ast(path).await;
                    black_box(ast_result);

                    let stats = cache_manager.get_cache_stats().await;
                    black_box(stats);
                })
            })
        },
    );

    group.finish();
}

/// Benchmark cache memory usage and performance under different configurations
fn benchmark_cache_memory_scaling(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("cache_memory_scaling");

    // Test with different numbers of files
    let file_counts = vec![1, 10, 50, 100];

    for count in file_counts {
        // Create multiple temporary files
        let mut temp_files = Vec::new();
        for i in 0..count {
            let content = if i % 3 == 0 {
                TestCodeSamples::small_rust_file()
            } else if i % 3 == 1 {
                TestCodeSamples::medium_rust_file()
            } else {
                TestCodeSamples::large_rust_file()
            };

            let (_temp_dir, file_path) = create_temp_file(content, &format!("_{}.rs", i));
            temp_files.push((_temp_dir, file_path));
        }

        group.bench_with_input(
            BenchmarkId::new("cache_scaling", count),
            &temp_files,
            |b, files| {
                b.iter(|| {
                    rt.block_on(async {
                        let cache_manager = CacheManagerImpl::new()
                            .await
                            .expect("Failed to create cache manager");

                        // Parse all files to populate cache
                        for (_temp_dir, file_path) in files {
                            black_box(
                                cache_manager
                                    .get_or_parse_ast(file_path)
                                    .await
                                    .expect("Failed to parse file"),
                            );
                        }

                        // Get cache stats to measure memory usage
                        let stats = cache_manager.get_cache_stats().await;
                        black_box(stats);
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark cache invalidation performance
fn benchmark_cache_invalidation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("cache_invalidation");

    let (_temp_dir, file_path) = create_temp_file(TestCodeSamples::large_rust_file(), ".rs");

    group.bench_with_input(
        BenchmarkId::new("clear_all_caches", "full"),
        &file_path,
        |b, path| {
            b.iter(|| {
                rt.block_on(async {
                    let cache_manager = CacheManagerImpl::new()
                        .await
                        .expect("Failed to create cache manager");

                    // Populate cache first
                    let _ = cache_manager.get_or_parse_ast(path).await;
                    let results = vec![];
                    cache_manager.cache_results(path, results).await;

                    // Benchmark clearing
                    black_box(cache_manager.clear_all_caches().await);
                })
            })
        },
    );

    group.finish();
}

criterion_group!(
    cache_benches,
    benchmark_ast_parsing_cache,
    benchmark_analysis_result_cache,
    benchmark_full_analysis_pipeline,
    benchmark_cache_memory_scaling,
    benchmark_cache_invalidation
);
criterion_main!(cache_benches);
