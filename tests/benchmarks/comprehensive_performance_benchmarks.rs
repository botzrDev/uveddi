//! Comprehensive performance benchmarks for Uveddi
//! 
//! This module contains benchmarks for all performance-critical components
//! to ensure optimal performance and detect regressions.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use uveddi::analysis::{
    AnalysisEngine,
    cache::{AstCache, MultilayerCache},
    components::ast_provider::AstProvider,
    config::AnalysisConfig,
    detectors::{DetectorType, GodObjectDetector, DeadCodeDetector},
    memory::{ArenaManager, MemoryConfig},
    parallel::ParallelEngine,
};
use uveddi::ast::tree_sitter_impl::TreeSitterProvider;

/// Benchmark AST parsing performance across different languages
fn bench_ast_parsing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let ast_provider = AstProvider::new();
    
    let test_cases = vec![
        ("rust_small", "fn hello() { println!(\"Hello\"); }", "rust"),
        ("rust_medium", create_medium_rust_code(), "rust"),
        ("rust_large", create_large_rust_code(), "rust"),
        ("python_small", "def hello(): print('Hello')", "python"),
        ("python_medium", create_medium_python_code(), "python"),
        ("python_large", create_large_python_code(), "python"),
        ("javascript_small", "function hello() { console.log('Hello'); }", "javascript"),
        ("javascript_medium", create_medium_js_code(), "javascript"),
        ("javascript_large", create_large_js_code(), "javascript"),
        ("typescript_small", "function hello(): void { console.log('Hello'); }", "typescript"),
        ("typescript_medium", create_medium_ts_code(), "typescript"),
        ("typescript_large", create_large_ts_code(), "typescript"),
    ];
    
    let mut group = c.benchmark_group("ast_parsing");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));
    
    for (name, code, language) in test_cases {
        group.throughput(Throughput::Bytes(code.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("parse", format!("{}_{}", language, name)),
            &(code, language),
            |b, (code, lang)| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = TempDir::new().unwrap();
                    let file_path = temp_dir.path().join(format!("test.{}", 
                        match *lang {
                            "rust" => "rs",
                            "python" => "py",
                            "javascript" => "js",
                            "typescript" => "ts",
                            _ => "txt",
                        }
                    ));
                    
                    tokio::fs::write(&file_path, code).await.unwrap();
                    
                    black_box(
                        ast_provider.parse_file(&file_path).await.unwrap()
                    );
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark cache performance
fn bench_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let cache_sizes = vec![100, 1000, 10000];
    let operation_counts = vec![10, 100, 1000];
    
    let mut group = c.benchmark_group("cache_operations");
    group.sample_size(50);
    
    for cache_size in cache_sizes {
        for op_count in &operation_counts {
            // AST Cache benchmarks
            group.bench_with_input(
                BenchmarkId::new("ast_cache_write", format!("size_{}_ops_{}", cache_size, op_count)),
                &(cache_size, *op_count),
                |b, (cache_size, op_count)| {
                    b.to_async(&rt).iter(|| async {
                        let temp_dir = TempDir::new().unwrap();
                        let cache = AstCache::new(temp_dir.path(), *cache_size).unwrap();
                        let ast_provider = AstProvider::new();
                        
                        for i in 0..*op_count {
                            let file_path = temp_dir.path().join(format!("test_{}.rs", i));
                            tokio::fs::write(&file_path, format!("fn test_{}() {{}}", i)).await.unwrap();
                            
                            let ast = ast_provider.parse_file(&file_path).await.unwrap();
                            black_box(cache.store_ast(&file_path, &ast).await.unwrap());
                        }
                    });
                },
            );
            
            group.bench_with_input(
                BenchmarkId::new("ast_cache_read", format!("size_{}_ops_{}", cache_size, op_count)),
                &(cache_size, *op_count),
                |b, (cache_size, op_count)| {
                    b.to_async(&rt).iter_batched(
                        || {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async {
                                let temp_dir = TempDir::new().unwrap();
                                let cache = AstCache::new(temp_dir.path(), *cache_size).unwrap();
                                let ast_provider = AstProvider::new();
                                let mut file_paths = Vec::new();
                                
                                for i in 0..*op_count {
                                    let file_path = temp_dir.path().join(format!("test_{}.rs", i));
                                    tokio::fs::write(&file_path, format!("fn test_{}() {{}}", i)).await.unwrap();
                                    
                                    let ast = ast_provider.parse_file(&file_path).await.unwrap();
                                    cache.store_ast(&file_path, &ast).await.unwrap();
                                    file_paths.push(file_path);
                                }
                                
                                (cache, file_paths, temp_dir)
                            })
                        },
                        |(cache, file_paths, _temp_dir)| async move {
                            for file_path in file_paths {
                                black_box(cache.get_ast(&file_path).await.unwrap());
                            }
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
            
            // Multilayer Cache benchmarks
            group.bench_with_input(
                BenchmarkId::new("multilayer_cache", format!("size_{}_ops_{}", cache_size, op_count)),
                &(cache_size, *op_count),
                |b, (cache_size, op_count)| {
                    b.to_async(&rt).iter(|| async {
                        let temp_dir = TempDir::new().unwrap();
                        let cache = MultilayerCache::new(temp_dir.path(), *cache_size).unwrap();
                        let ast_provider = AstProvider::new();
                        
                        // Write operations
                        for i in 0..*op_count {
                            let file_path = temp_dir.path().join(format!("test_{}.rs", i));
                            tokio::fs::write(&file_path, format!("fn test_{}() {{}}", i)).await.unwrap();
                            
                            let ast = ast_provider.parse_file(&file_path).await.unwrap();
                            black_box(cache.store_ast(&file_path, &ast).await.unwrap());
                        }
                        
                        // Read operations
                        for i in 0..*op_count {
                            let file_path = temp_dir.path().join(format!("test_{}.rs", i));
                            black_box(cache.get_ast(&file_path).await.unwrap());
                        }
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark memory allocation performance
fn bench_memory_allocation(c: &mut Criterion) {
    let allocation_sizes = vec![1024, 8192, 65536, 1024 * 1024]; // 1KB to 1MB
    let allocation_counts = vec![10, 100, 1000];
    
    let mut group = c.benchmark_group("memory_allocation");
    group.sample_size(50);
    
    for size in allocation_sizes {
        for count in &allocation_counts {
            group.throughput(Throughput::Bytes((size * count) as u64));
            
            group.bench_with_input(
                BenchmarkId::new("arena_allocation", format!("size_{}_count_{}", size, count)),
                &(size, *count),
                |b, (size, count)| {
                    b.iter(|| {
                        let config = MemoryConfig {
                            max_arena_size: 64 * 1024 * 1024, // 64MB
                            detector_pool_size: 10,
                            ast_cache_size: 1000,
                            enable_zero_copy: true,
                        };
                        
                        let arena_manager = ArenaManager::new(config).unwrap();
                        
                        for _ in 0..*count {
                            let arena = arena_manager.create_arena(*size).unwrap();
                            let data: &mut [u8] = arena.alloc_slice_fill_default(*size);
                            black_box(data.fill(0xAA));
                        }
                    });
                },
            );
            
            // Compare with standard allocation
            group.bench_with_input(
                BenchmarkId::new("standard_allocation", format!("size_{}_count_{}", size, count)),
                &(size, *count),
                |b, (size, count)| {
                    b.iter(|| {
                        for _ in 0..*count {
                            let mut data = vec![0u8; *size];
                            black_box(data.fill(0xAA));
                        }
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark detector performance
fn bench_detectors(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let code_sizes = vec!["small", "medium", "large"];
    
    let mut group = c.benchmark_group("detectors");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(15));
    
    for size in code_sizes {
        let (rust_code, expected_lines) = match size {
            "small" => (create_small_rust_code(), 50),
            "medium" => (create_medium_rust_code(), 500),
            "large" => (create_large_rust_code(), 2000),
            _ => unreachable!(),
        };
        
        group.throughput(Throughput::Elements(expected_lines));
        
        // God Object Detector
        group.bench_with_input(
            BenchmarkId::new("god_object_detector", size),
            &rust_code,
            |b, code| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = TempDir::new().unwrap();
                    let file_path = temp_dir.path().join("test.rs");
                    tokio::fs::write(&file_path, code).await.unwrap();
                    
                    let ast_provider = AstProvider::new();
                    let ast = ast_provider.parse_file(&file_path).await.unwrap();
                    
                    let detector = GodObjectDetector::new();
                    black_box(detector.analyze(&file_path, &ast).await.unwrap());
                });
            },
        );
        
        // Dead Code Detector
        group.bench_with_input(
            BenchmarkId::new("dead_code_detector", size),
            &rust_code,
            |b, code| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = TempDir::new().unwrap();
                    let file_path = temp_dir.path().join("test.rs");
                    tokio::fs::write(&file_path, code).await.unwrap();
                    
                    let ast_provider = AstProvider::new();
                    let ast = ast_provider.parse_file(&file_path).await.unwrap();
                    
                    let detector = DeadCodeDetector::new();
                    black_box(detector.analyze(&file_path, &ast).await.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parallel processing performance
fn bench_parallel_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let file_counts = vec![10, 50, 100, 500];
    let thread_counts = vec![1, 2, 4, 8];
    
    let mut group = c.benchmark_group("parallel_processing");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(20));
    
    for file_count in file_counts {
        for thread_count in &thread_counts {
            group.throughput(Throughput::Elements(file_count));
            
            group.bench_with_input(
                BenchmarkId::new("parallel_analysis", format!("files_{}_threads_{}", file_count, thread_count)),
                &(file_count, *thread_count),
                |b, (file_count, thread_count)| {
                    b.to_async(&rt).iter(|| async {
                        let temp_dir = create_test_project_with_files(*file_count).await;
                        
                        let config = AnalysisConfig {
                            max_file_size: 1024 * 1024,
                            parallel_analysis: true,
                            thread_count: Some(*thread_count),
                            enable_ai_explanations: false,
                            cache_enabled: false, // Disable for pure processing benchmark
                            detector_types: vec![DetectorType::GodObject],
                            language_filters: vec!["rust".to_string()],
                            exclude_patterns: vec![],
                        };
                        
                        let engine = ParallelEngine::new(config).unwrap();
                        black_box(engine.analyze(temp_dir.path()).await.unwrap());
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark end-to-end analysis performance
fn bench_end_to_end_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let project_sizes = vec![
        ("tiny", 5),
        ("small", 25),
        ("medium", 100),
        ("large", 500),
    ];
    
    let mut group = c.benchmark_group("end_to_end_analysis");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    
    for (size_name, file_count) in project_sizes {
        group.throughput(Throughput::Elements(file_count));
        
        group.bench_with_input(
            BenchmarkId::new("full_analysis", size_name),
            &file_count,
            |b, file_count| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = create_realistic_project_with_files(*file_count).await;
                    
                    let config = AnalysisConfig {
                        max_file_size: 1024 * 1024,
                        parallel_analysis: true,
                        enable_ai_explanations: false,
                        cache_enabled: true,
                        detector_types: vec![
                            DetectorType::GodObject,
                            DetectorType::DeadCode,
                            DetectorType::CyclicDependencies,
                            DetectorType::TightCoupling,
                        ],
                        language_filters: vec!["rust".to_string()],
                        exclude_patterns: vec!["target/**".to_string()],
                    };
                    
                    let engine = AnalysisEngine::new(config).unwrap();
                    black_box(engine.analyze(temp_dir.path()).await.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark report generation performance
fn bench_report_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let issue_counts = vec![10, 100, 1000, 5000];
    
    let mut group = c.benchmark_group("report_generation");
    group.sample_size(50);
    
    for issue_count in issue_counts {
        group.throughput(Throughput::Elements(issue_count));
        
        group.bench_with_input(
            BenchmarkId::new("json_report", format!("issues_{}", issue_count)),
            &issue_count,
            |b, issue_count| {
                b.to_async(&rt).iter(|| async {
                    let (issues, dependency_graph) = create_test_analysis_results(*issue_count).await;
                    
                    let report_config = ReportConfig {
                        format: ReportFormat::Json,
                        include_metrics: true,
                        include_code_snippets: true,
                        include_dependency_graph: true,
                        output_path: None, // Generate in memory
                    };
                    
                    let report_generator = ReportGenerator::new(report_config);
                    black_box(report_generator.generate_report_string(&issues, &dependency_graph).await.unwrap());
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("html_report", format!("issues_{}", issue_count)),
            &issue_count,
            |b, issue_count| {
                b.to_async(&rt).iter(|| async {
                    let (issues, dependency_graph) = create_test_analysis_results(*issue_count).await;
                    
                    let report_config = ReportConfig {
                        format: ReportFormat::Html,
                        include_metrics: true,
                        include_code_snippets: true,
                        include_dependency_graph: true,
                        output_path: None,
                    };
                    
                    let report_generator = ReportGenerator::new(report_config);
                    black_box(report_generator.generate_report_string(&issues, &dependency_graph).await.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark tree-sitter performance specifically
fn bench_tree_sitter_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let provider = TreeSitterProvider::new();
    
    let test_codes = vec![
        ("rust_functions", create_function_heavy_rust_code()),
        ("rust_structs", create_struct_heavy_rust_code()),
        ("rust_generics", create_generic_heavy_rust_code()),
        ("python_classes", create_class_heavy_python_code()),
        ("javascript_objects", create_object_heavy_js_code()),
    ];
    
    let mut group = c.benchmark_group("tree_sitter_operations");
    group.sample_size(100);
    
    for (name, code) in test_codes {
        let lang = if name.starts_with("rust") { "rust" }
                  else if name.starts_with("python") { "python" }
                  else { "javascript" };
        
        group.throughput(Throughput::Bytes(code.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("parse_to_ast", name),
            &(code, lang),
            |b, (code, lang)| {
                b.to_async(&rt).iter(|| async {
                    black_box(provider.parse_source_code(code, lang).await.unwrap());
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("query_execution", name),
            &(code, lang),
            |b, (code, lang)| {
                b.to_async(&rt).iter_batched(
                    || {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async {
                            provider.parse_source_code(code, lang).await.unwrap()
                        })
                    },
                    |ast| async move {
                        // Execute common queries
                        black_box(provider.execute_query(&ast, "function_definitions").await.unwrap());
                        black_box(provider.execute_query(&ast, "struct_definitions").await.unwrap());
                        black_box(provider.execute_query(&ast, "variable_declarations").await.unwrap());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

// Helper functions to create test data
async fn create_test_project_with_files(file_count: u64) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    tokio::fs::create_dir_all(&src_dir).await.unwrap();
    
    for i in 0..file_count {
        let content = format!(r#"
pub struct Data{} {{
    value: i32,
}}

impl Data{} {{
    pub fn new(value: i32) -> Self {{
        Self {{ value }}
    }}
    
    pub fn process(&self) -> i32 {{
        self.value * 2
    }}
}}

pub fn function_{}() -> i32 {{
    let data = Data{}::new({});
    data.process()
}}
"#, i, i, i, i, i);
        
        tokio::fs::write(src_dir.join(format!("module_{}.rs", i)), content).await.unwrap();
    }
    
    temp_dir
}

async fn create_realistic_project_with_files(file_count: u64) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    tokio::fs::create_dir_all(&src_dir).await.unwrap();
    
    // Create a mix of file types with realistic content
    for i in 0..file_count {
        let content = match i % 4 {
            0 => create_god_object_content(i),
            1 => create_service_layer_content(i),
            2 => create_data_model_content(i),
            3 => create_utility_content(i),
            _ => unreachable!(),
        };
        
        tokio::fs::write(src_dir.join(format!("module_{}.rs", i)), content).await.unwrap();
    }
    
    temp_dir
}

fn create_god_object_content(id: u64) -> String {
    format!(r#"
use std::collections::HashMap;

pub struct GodObject{} {{
    data: Vec<String>,
    cache: HashMap<String, i32>,
    config: HashMap<String, String>,
    state: i32,
    flags: Vec<bool>,
}}

impl GodObject{} {{
    pub fn new() -> Self {{
        Self {{
            data: Vec::new(),
            cache: HashMap::new(),
            config: HashMap::new(),
            state: 0,
            flags: Vec::new(),
        }}
    }}
    
    // Data management methods
    pub fn add_data(&mut self, item: String) {{ self.data.push(item); }}
    pub fn remove_data(&mut self, index: usize) {{ self.data.remove(index); }}
    pub fn clear_data(&mut self) {{ self.data.clear(); }}
    pub fn get_data(&self, index: usize) -> Option<&String> {{ self.data.get(index) }}
    
    // Cache management methods
    pub fn cache_put(&mut self, key: String, value: i32) {{ self.cache.insert(key, value); }}
    pub fn cache_get(&self, key: &str) -> Option<&i32> {{ self.cache.get(key) }}
    pub fn cache_clear(&mut self) {{ self.cache.clear(); }}
    
    // Config management methods
    pub fn set_config(&mut self, key: String, value: String) {{ self.config.insert(key, value); }}
    pub fn get_config(&self, key: &str) -> Option<&String> {{ self.config.get(key) }}
    
    // State management methods
    pub fn set_state(&mut self, state: i32) {{ self.state = state; }}
    pub fn get_state(&self) -> i32 {{ self.state }}
    pub fn increment_state(&mut self) {{ self.state += 1; }}
    
    // Flag management methods
    pub fn add_flag(&mut self, flag: bool) {{ self.flags.push(flag); }}
    pub fn get_flag(&self, index: usize) -> Option<bool> {{ self.flags.get(index).copied() }}
    
    pub fn do_everything(&mut self) {{
        self.add_data(format!("data_{}", {}));
        self.cache_put("key".to_string(), 42);
        self.set_config("setting".to_string(), "value".to_string());
        self.increment_state();
        self.add_flag(true);
    }}
}}
"#, id, id, id)
}

fn create_service_layer_content(id: u64) -> String {
    format!(r#"
use std::sync::Arc;
use std::collections::HashMap;

pub struct Service{} {{
    repository: Arc<Repository{}>,
    cache: HashMap<String, String>,
}}

impl Service{} {{
    pub fn new(repository: Arc<Repository{}>) -> Self {{
        Self {{
            repository,
            cache: HashMap::new(),
        }}
    }}
    
    pub async fn get_data(&self, id: &str) -> Result<Option<String>, String> {{
        if let Some(cached) = self.cache.get(id) {{
            return Ok(Some(cached.clone()));
        }}
        
        self.repository.fetch(id).await
    }}
    
    pub async fn save_data(&mut self, id: String, data: String) -> Result<(), String> {{
        self.repository.store(&id, &data).await?;
        self.cache.insert(id, data);
        Ok(())
    }}
}}

pub struct Repository{} {{
    connection: String,
}}

impl Repository{} {{
    pub fn new(connection: String) -> Self {{
        Self {{ connection }}
    }}
    
    pub async fn fetch(&self, id: &str) -> Result<Option<String>, String> {{
        // Simulate database fetch
        Ok(Some(format!("data_for_{}", id)))
    }}
    
    pub async fn store(&self, id: &str, data: &str) -> Result<(), String> {{
        // Simulate database store
        Ok(())
    }}
}}
"#, id, id, id, id, id, id)
}

fn create_data_model_content(id: u64) -> String {
    format!(r#"
use serde::{{Serialize, Deserialize}};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model{} {{
    pub id: u64,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
}}

impl Model{} {{
    pub fn new(id: u64, name: String, value: f64) -> Self {{
        Self {{
            id,
            name,
            value,
            metadata: HashMap::new(),
            tags: Vec::new(),
        }}
    }}
    
    pub fn add_metadata(&mut self, key: String, value: String) {{
        self.metadata.insert(key, value);
    }}
    
    pub fn add_tag(&mut self, tag: String) {{
        self.tags.push(tag);
    }}
    
    pub fn calculate_score(&self) -> f64 {{
        self.value * self.tags.len() as f64
    }}
    
    pub fn validate(&self) -> Result<(), String> {{
        if self.name.is_empty() {{
            return Err("Name cannot be empty".to_string());
        }}
        if self.value < 0.0 {{
            return Err("Value must be non-negative".to_string());
        }}
        Ok(())
    }}
}}

#[derive(Debug)]
pub struct ModelCollection{} {{
    models: Vec<Model{}>,
    index: HashMap<u64, usize>,
}}

impl ModelCollection{} {{
    pub fn new() -> Self {{
        Self {{
            models: Vec::new(),
            index: HashMap::new(),
        }}
    }}
    
    pub fn add_model(&mut self, model: Model{}) {{
        let index = self.models.len();
        self.index.insert(model.id, index);
        self.models.push(model);
    }}
    
    pub fn get_model(&self, id: u64) -> Option<&Model{}> {{
        self.index.get(&id).and_then(|&index| self.models.get(index))
    }}
}}
"#, id, id, id, id, id, id, id, id)
}

fn create_utility_content(id: u64) -> String {
    format!(r#"
use std::collections::HashMap;

pub fn utility_function_{}(input: &str) -> String {{
    format!("processed_{}", input)
}}

pub fn calculate_hash(data: &[u8]) -> u64 {{
    let mut hash = 0u64;
    for &byte in data {{
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }}
    hash
}}

pub fn parse_config(content: &str) -> HashMap<String, String> {{
    let mut config = HashMap::new();
    for line in content.lines() {{
        if let Some((key, value)) = line.split_once('=') {{
            config.insert(key.trim().to_string(), value.trim().to_string());
        }}
    }}
    config
}}

pub fn format_output(data: &HashMap<String, String>) -> String {{
    let mut result = String::new();
    for (key, value) in data {{
        result.push_str(&format!("{}={}\n", key, value));
    }}
    result
}}

// Some dead code
fn unused_function_{}() {{
    let _unused = "This function is never called";
}}

struct UnusedStruct{} {{
    _field: i32,
}}
"#, id, id, id)
}

fn create_small_rust_code() -> String {
    r#"
fn small_function() -> i32 {
    let x = 42;
    let y = x * 2;
    y
}

struct SmallStruct {
    value: i32,
}

impl SmallStruct {
    fn new(value: i32) -> Self {
        Self { value }
    }
}
"#.to_string()
}

fn create_medium_rust_code() -> String {
    create_god_object_content(999)
}

fn create_large_rust_code() -> String {
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&create_god_object_content(i));
        code.push('\n');
    }
    code
}

fn create_medium_python_code() -> String {
    r#"
class MediumPythonClass:
    def __init__(self):
        self.data = []
        self.cache = {}
    
    def add_data(self, item):
        self.data.append(item)
    
    def process_data(self):
        for item in self.data:
            self.cache[str(len(self.cache))] = item
    
    def get_summary(self):
        return {
            'count': len(self.data),
            'cache_size': len(self.cache)
        }

def utility_function(x, y):
    return x * y + 42

class AnotherClass:
    def method1(self): pass
    def method2(self): pass
    def method3(self): pass
"#.to_string()
}

fn create_large_python_code() -> String {
    let mut code = String::new();
    for i in 0..30 {
        code.push_str(&format!(r#"
class LargeClass{}:
    def __init__(self):
        self.value = {}
        self.data = []
    
    def method1(self): return self.value * 2
    def method2(self): return self.value * 3
    def method3(self): return self.value * 4
    def method4(self): return self.value * 5

"#, i, i));
    }
    code
}

fn create_medium_js_code() -> String {
    r#"
class MediumJavaScriptClass {
    constructor() {
        this.data = [];
        this.cache = new Map();
    }
    
    addData(item) {
        this.data.push(item);
    }
    
    processData() {
        this.data.forEach((item, index) => {
            this.cache.set(index.toString(), item);
        });
    }
    
    getSummary() {
        return {
            count: this.data.length,
            cacheSize: this.cache.size
        };
    }
}

function utilityFunction(x, y) {
    return x * y + 42;
}

const anotherObject = {
    method1() { return 1; },
    method2() { return 2; },
    method3() { return 3; }
};
"#.to_string()
}

fn create_large_js_code() -> String {
    let mut code = String::new();
    for i in 0..30 {
        code.push_str(&format!(r#"
class LargeClass{} {{
    constructor() {{
        this.value = {};
        this.data = [];
    }}
    
    method1() {{ return this.value * 2; }}
    method2() {{ return this.value * 3; }}
    method3() {{ return this.value * 4; }}
    method4() {{ return this.value * 5; }}
}}

"#, i, i));
    }
    code
}

fn create_medium_ts_code() -> String {
    r#"
interface DataItem {
    id: number;
    value: string;
}

class MediumTypeScriptClass {
    private data: DataItem[] = [];
    private cache: Map<string, DataItem> = new Map();
    
    addData(item: DataItem): void {
        this.data.push(item);
    }
    
    processData(): void {
        this.data.forEach(item => {
            this.cache.set(item.id.toString(), item);
        });
    }
    
    getSummary(): { count: number, cacheSize: number } {
        return {
            count: this.data.length,
            cacheSize: this.cache.size
        };
    }
}

function utilityFunction(x: number, y: number): number {
    return x * y + 42;
}
"#.to_string()
}

fn create_large_ts_code() -> String {
    let mut code = String::new();
    for i in 0..30 {
        code.push_str(&format!(r#"
interface Interface{} {{
    value{}: number;
    data{}: string[];
}}

class LargeClass{} implements Interface{} {{
    value{}: number = {};
    data{}: string[] = [];
    
    method1(): number {{ return this.value{} * 2; }}
    method2(): number {{ return this.value{} * 3; }}
    method3(): number {{ return this.value{} * 4; }}
    method4(): number {{ return this.value{} * 5; }}
}}

"#, i, i, i, i, i, i, i, i, i, i, i, i, i));
    }
    code
}

fn create_function_heavy_rust_code() -> String {
    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!(r#"
fn function_{}(param: i32) -> i32 {{
    param * {} + 1
}}
"#, i, i));
    }
    code
}

fn create_struct_heavy_rust_code() -> String {
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&format!(r#"
struct Struct{} {{
    field1: i32,
    field2: String,
    field3: Vec<i32>,
}}

impl Struct{} {{
    fn new() -> Self {{
        Self {{
            field1: {},
            field2: String::from("test"),
            field3: Vec::new(),
        }}
    }}
}}
"#, i, i, i));
    }
    code
}

fn create_generic_heavy_rust_code() -> String {
    let mut code = String::new();
    for i in 0..30 {
        code.push_str(&format!(r#"
struct Generic{}<T, U> {{
    field1: T,
    field2: U,
    field3: Vec<T>,
}}

impl<T: Clone, U: Clone> Generic{}<T, U> {{
    fn new(val1: T, val2: U) -> Self {{
        Self {{
            field1: val1,
            field2: val2,
            field3: Vec::new(),
        }}
    }}
    
    fn process<V>(&self, input: V) -> V {{
        input
    }}
}}
"#, i, i));
    }
    code
}

fn create_class_heavy_python_code() -> String {
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&format!(r#"
class Class{}:
    def __init__(self, value):
        self.value = value
        self.data = []
    
    def method1(self):
        return self.value * 2
    
    def method2(self):
        return len(self.data)
    
    def method3(self):
        self.data.append(self.value)

"#, i));
    }
    code
}

fn create_object_heavy_js_code() -> String {
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&format!(r#"
const object{} = {{
    value: {},
    data: [],
    
    method1() {{
        return this.value * 2;
    }},
    
    method2() {{
        return this.data.length;
    }},
    
    method3() {{
        this.data.push(this.value);
    }}
}};

"#, i, i));
    }
    code
}

async fn create_test_analysis_results(issue_count: u64) -> (Vec<ArchitecturalIssue>, DependencyGraph) {
    use uveddi::analysis::types::{IssueType, Severity, ArchitecturalIssue, CodeMetrics};
    use uveddi::analysis::graph::dependency::DependencyGraph;
    
    let mut issues = Vec::new();
    for i in 0..issue_count {
        let issue = ArchitecturalIssue {
            id: format!("issue_{}", i),
            issue_type: match i % 4 {
                0 => IssueType::GodObject,
                1 => IssueType::DeadCode,
                2 => IssueType::CyclicDependencies,
                3 => IssueType::TightCoupling,
                _ => unreachable!(),
            },
            severity: match i % 3 {
                0 => Severity::Low,
                1 => Severity::Medium,
                2 => Severity::High,
                _ => unreachable!(),
            },
            file_path: PathBuf::from(format!("src/file_{}.rs", i)),
            line_start: (i * 10) as usize,
            line_end: (i * 10 + 5) as usize,
            description: format!("Test issue {}", i),
            code_snippet: Some(format!("fn test_{}() {{}}", i)),
            metrics: CodeMetrics {
                lines_of_code: (50 + i * 2) as usize,
                cyclomatic_complexity: (5 + i % 10) as usize,
                coupling_score: (i % 100) as f64 / 100.0,
            },
            ai_explanation: None,
        };
        issues.push(issue);
    }
    
    let mut dependency_graph = DependencyGraph::new();
    for i in 0..std::cmp::min(issue_count, 100) {
        let file_path = PathBuf::from(format!("src/file_{}.rs", i));
        dependency_graph.add_file(file_path);
    }
    
    (issues, dependency_graph)
}

criterion_group!(
    benches,
    bench_ast_parsing,
    bench_cache_operations,
    bench_memory_allocation,
    bench_detectors,
    bench_parallel_processing,
    bench_end_to_end_analysis,
    bench_report_generation,
    bench_tree_sitter_operations
);

criterion_main!(benches)