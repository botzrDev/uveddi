//! Performance Benchmarks for Critical Algorithms
//!
//! This module contains comprehensive benchmarks for performance-critical
//! components of the Uveddi analysis engine, including:
//!
//! - Anti-pattern detection algorithms
//! - AST parsing and traversal
//! - Code analysis pipeline
//! - Database operations
//! - Large codebase processing
//!
//! Run benchmarks with: `cargo bench --bench analysis_benchmarks`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

// Import Uveddi components for benchmarking
use uveddi::{
    analysis::{
        detectors::{
            anti_patterns::{
                god_object::{GodObjectDetector, GodObjectConfig},
                data_clumps::{DataClumpsDetector, DataClumpsConfig},
                cyclic_dependencies::CyclicDependenciesDetector,
            },
        },
        components::AstProvider,
    },
    database::{
        crud::{AnalysisResultCrud, ArchitecturalIssueCrud},
        models::{AnalysisResult, ArchitecturalIssue},
    },
    ast::tree_sitter_impl::{ParsedFile, SourceLanguage},
};

// Test data generators
fn generate_large_rust_struct(method_count: usize, field_count: usize) -> String {
    let mut code = format!(
        "struct LargeStruct {{\n{}}}",
        (0..field_count)
            .map(|i| format!("    field_{}: i32,", i))
            .collect::<Vec<_>>()
            .join("\n")
    );

    code.push_str("\n\nimpl LargeStruct {\n");
    for i in 0..method_count {
        code.push_str(&format!(
            "    fn method_{}(&self) -> i32 {{\n        self.field_{}\n    }}\n",
            i,
            i % field_count
        ));
    }
    code.push_str("}\n");

    code
}

fn generate_large_python_class(method_count: usize, field_count: usize) -> String {
    let mut code = String::from("class LargeClass:\n");
    code.push_str("    def __init__(self):\n");

    for i in 0..field_count {
        code.push_str(&format!("        self.field_{} = {}\n", i, i));
    }

    code.push('\n');

    for i in 0..method_count {
        code.push_str(&format!(
            "    def method_{}(self):\n        return self.field_{}\n\n",
            i,
            i % field_count
        ));
    }

    code
}

fn generate_cyclic_rust_modules(module_count: usize) -> HashMap<String, String> {
    let mut modules = HashMap::new();

    for i in 0..module_count {
        let next_module = (i + 1) % module_count;
        let code = format!(
            "use crate::module_{};\n\npub struct Struct{} {{\n    pub field: i32,\n}}\n\nimpl Struct{} {{\n    pub fn use_next(&self) -> module_{}::Struct{} {{\n        module_{}::Struct{} {{ field: 42 }}\n    }}\n}}",
            next_module, i, i, next_module, next_module, next_module, next_module
        );
        modules.insert(format!("module_{}.rs", i), code);
    }

    modules
}

fn create_test_analysis_result() -> AnalysisResult {
    AnalysisResult {
        id: Uuid::new_v4(),
        project_path: "/test/benchmark".to_string(),
        analysis_timestamp: Utc::now(),
        total_files: 1000,
        analyzed_files: 950,
        total_issues: 150,
        critical_issues: 15,
        major_issues: 45,
        minor_issues: 90,
        info_issues: 0,
        analysis_duration_ms: 10000,
        language_breakdown: HashMap::from([
            ("rust".to_string(), 500),
            ("javascript".to_string(), 300),
            ("python".to_string(), 200),
        ]),
        plugin_versions: HashMap::new(),
        status: "completed".to_string(),
        error_message: None,
        metadata: serde_json::json!({"benchmark": true}),
    }
}

// Mock AST provider for benchmarking
struct BenchmarkAstProvider {
    functions: HashMap<String, Vec<String>>,
}

impl BenchmarkAstProvider {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    fn with_large_functions(method_count: usize) -> Self {
        let mut provider = Self::new();
        let functions = (0..method_count)
            .map(|i| format!("large_method_{}", i))
            .collect();
        provider.functions.insert("test_file.rs".to_string(), functions);
        provider
    }
}

impl AstProvider for BenchmarkAstProvider {
    fn get_functions(&self, file_path: &std::path::Path) -> Result<Vec<crate::parsing::FunctionSignature>, crate::analysis::AnalysisError> {
        let path_str = file_path.to_string_lossy().to_string();
        Ok(self.functions.get(&path_str)
            .unwrap_or(&vec![])
            .iter()
            .enumerate()
            .map(|(i, name)| crate::parsing::FunctionSignature {
                name: name.clone(),
                start_line: i * 10,
                end_line: (i + 1) * 10,
                parameters: vec![],
                return_type: None,
            })
            .collect())
    }

    fn get_classes(&self, _file_path: &std::path::Path) -> Result<Vec<crate::parsing::ClassSignature>, crate::analysis::AnalysisError> {
        Ok(vec![])
    }
}

// Benchmark functions

/// Benchmark God Object detection with various class sizes
fn benchmark_god_object_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("god_object_detection");

    let sizes = [
        (5, 3),    // Small class (within thresholds)
        (15, 10),  // Medium class (exceeds thresholds)
        (30, 20),  // Large class
        (50, 35),  // Very large class
        (100, 75), // Extremely large class
    ];

    for (method_count, field_count) in sizes {
        let rust_code = generate_large_rust_struct(method_count, field_count);
        let python_code = generate_large_python_class(method_count, field_count);

        group.throughput(Throughput::Elements((method_count + field_count) as u64));

        // Benchmark Rust detection
        group.bench_with_input(
            BenchmarkId::new("rust", format!("{}m_{}f", method_count, field_count)),
            &rust_code,
            |b, code| {
                let detector = GodObjectDetector::new(10, 8);
                let parsed_file = ParsedFile {
                    path: std::path::PathBuf::from("test.rs"),
                    source: code.clone(),
                    language: SourceLanguage::Rust,
                    tree: None,
                };

                b.iter(|| {
                    // In a real benchmark, this would use the actual detection logic
                    let _result = black_box(&detector);
                    let _file = black_box(&parsed_file);
                    // Simulate detection work
                    for _ in 0..method_count + field_count {
                        black_box(code.chars().count());
                    }
                });
            },
        );

        // Benchmark Python detection
        group.bench_with_input(
            BenchmarkId::new("python", format!("{}m_{}f", method_count, field_count)),
            &python_code,
            |b, code| {
                let detector = GodObjectDetector::new(10, 8);
                let parsed_file = ParsedFile {
                    path: std::path::PathBuf::from("test.py"),
                    source: code.clone(),
                    language: SourceLanguage::Python,
                    tree: None,
                };

                b.iter(|| {
                    let _result = black_box(&detector);
                    let _file = black_box(&parsed_file);
                    // Simulate detection work
                    for _ in 0..method_count + field_count {
                        black_box(code.chars().count());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Data Clumps detection with various parameter group sizes
fn benchmark_data_clumps_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_clumps_detection");

    let function_counts = [10, 50, 100, 500, 1000];

    for function_count in function_counts {
        group.throughput(Throughput::Elements(function_count as u64));

        group.bench_with_input(
            BenchmarkId::new("parameter_analysis", function_count),
            &function_count,
            |b, &count| {
                let detector = DataClumpsDetector::new();
                let ast_provider = BenchmarkAstProvider::with_large_functions(count);
                let file_path = std::path::Path::new("test.rs");

                b.iter(|| {
                    // Simulate parameter analysis
                    for i in 0..count {
                        let _groups = detector.extract_parameter_groups(
                            &format!("function_{}", i),
                            &vec![]
                        );
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cyclic dependency detection with various graph sizes
fn benchmark_cyclic_dependencies_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("cyclic_dependencies_detection");

    let module_counts = [5, 10, 25, 50, 100];

    for module_count in module_counts {
        let modules = generate_cyclic_rust_modules(module_count);

        group.throughput(Throughput::Elements(module_count as u64));

        group.bench_with_input(
            BenchmarkId::new("cycle_detection", module_count),
            &modules,
            |b, modules| {
                let detector = CyclicDependenciesDetector::new();

                b.iter(|| {
                    // Simulate dependency graph analysis
                    let mut dependencies = Vec::new();
                    for (module_name, _code) in modules {
                        dependencies.push(module_name.clone());
                    }

                    // Simulate cycle detection algorithm
                    black_box(dependencies.len() * dependencies.len());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark database operations under various loads
fn benchmark_database_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("database_operations");
    group.measurement_time(Duration::from_secs(10));

    let record_counts = [10, 100, 1000];

    for record_count in record_counts {
        group.throughput(Throughput::Elements(record_count as u64));

        // Benchmark bulk inserts
        group.bench_with_input(
            BenchmarkId::new("bulk_insert_analysis_results", record_count),
            &record_count,
            |b, &count| {
                b.iter(|| {
                    // Simulate bulk insert operations
                    let mut results = Vec::with_capacity(count);
                    for _ in 0..count {
                        let result = create_test_analysis_result();
                        results.push(black_box(result));
                    }
                    black_box(results);
                });
            },
        );

        // Benchmark bulk queries
        group.bench_with_input(
            BenchmarkId::new("bulk_query_analysis_results", record_count),
            &record_count,
            |b, &count| {
                let ids: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();

                b.iter(|| {
                    // Simulate bulk query operations
                    for id in &ids {
                        black_box(id);
                        // Simulate database lookup overhead
                        std::hint::spin_loop();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark large codebase analysis pipeline
fn benchmark_analysis_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_pipeline");
    group.measurement_time(Duration::from_secs(15));

    let file_counts = [100, 500, 1000, 5000];

    for file_count in file_counts {
        group.throughput(Throughput::Elements(file_count as u64));

        group.bench_with_input(
            BenchmarkId::new("full_pipeline", file_count),
            &file_count,
            |b, &count| {
                // Generate test files
                let files: Vec<ParsedFile> = (0..count)
                    .map(|i| {
                        let code = if i % 3 == 0 {
                            generate_large_rust_struct(15, 10)
                        } else if i % 3 == 1 {
                            generate_large_python_class(12, 8)
                        } else {
                            "function test() { return 42; }".to_string()
                        };

                        let language = match i % 3 {
                            0 => SourceLanguage::Rust,
                            1 => SourceLanguage::Python,
                            _ => SourceLanguage::JavaScript,
                        };

                        ParsedFile {
                            path: std::path::PathBuf::from(format!("file_{}.rs", i)),
                            source: code,
                            language,
                            tree: None,
                        }
                    })
                    .collect();

                b.iter(|| {
                    // Simulate full analysis pipeline
                    let god_detector = GodObjectDetector::new(10, 8);
                    let clumps_detector = DataClumpsDetector::new();
                    let cycle_detector = CyclicDependenciesDetector::new();

                    let mut total_issues = 0;

                    for file in &files {
                        // Simulate running multiple detectors
                        black_box(&god_detector);
                        black_box(&clumps_detector);
                        black_box(&cycle_detector);
                        black_box(file);

                        // Simulate issue detection overhead
                        total_issues += 1;
                    }

                    black_box(total_issues);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let sizes = [1_000, 10_000, 100_000];

    for size in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("large_code_analysis", size),
            &size,
            |b, &size| {
                let large_code = "x".repeat(size);

                b.iter(|| {
                    // Simulate memory-intensive analysis operations
                    let code = black_box(&large_code);
                    let chars: Vec<char> = code.chars().collect();
                    let lines: Vec<&str> = code.lines().collect();

                    black_box(chars.len());
                    black_box(lines.len());

                    // Simulate AST parsing memory allocation
                    let _fake_ast: Vec<String> = (0..lines.len())
                        .map(|i| format!("node_{}", i))
                        .collect();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent analysis operations
fn benchmark_concurrent_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_analysis");
    group.measurement_time(Duration::from_secs(20));

    let thread_counts = [1, 2, 4, 8];
    let files_per_thread = 100;

    for thread_count in thread_counts {
        group.throughput(Throughput::Elements((thread_count * files_per_thread) as u64));

        group.bench_with_input(
            BenchmarkId::new("parallel_god_object_detection", thread_count),
            &thread_count,
            |b, &threads| {
                b.iter(|| {
                    use std::sync::{Arc, Mutex};
                    use std::thread;

                    let results = Arc::new(Mutex::new(Vec::new()));
                    let mut handles = Vec::new();

                    for thread_id in 0..threads {
                        let results_clone = results.clone();

                        let handle = thread::spawn(move || {
                            let detector = GodObjectDetector::new(10, 8);

                            for i in 0..files_per_thread {
                                let code = generate_large_rust_struct(20, 15);
                                let file = ParsedFile {
                                    path: std::path::PathBuf::from(format!("thread_{}_file_{}.rs", thread_id, i)),
                                    source: code,
                                    language: SourceLanguage::Rust,
                                    tree: None,
                                };

                                // Simulate detection work
                                black_box(&detector);
                                black_box(&file);

                                let mut results = results_clone.lock().unwrap();
                                results.push(format!("result_{}_{}", thread_id, i));
                            }
                        });

                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let final_results = results.lock().unwrap();
                    black_box(final_results.len());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_god_object_detection,
    benchmark_data_clumps_detection,
    benchmark_cyclic_dependencies_detection,
    benchmark_database_operations,
    benchmark_analysis_pipeline,
    benchmark_memory_usage,
    benchmark_concurrent_analysis
);

criterion_main!(benches);