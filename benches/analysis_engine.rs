use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
use uveddi::analysis::{AnalysisDetector, AnalysisEngine};

/// Generate test Rust code for benchmarking
fn create_test_rust_file(file_name: &str, complexity: &str) -> String {
    match complexity {
        "simple" => format!(
            r#"
pub fn simple_function() -> i32 {{
    42
}}

pub struct SimpleStruct {{
    value: i32,
}}

impl SimpleStruct {{
    pub fn new(value: i32) -> Self {{
        Self {{ value }}
    }}
}}
"#
        ),
        "medium" => format!(
            r#"
use std::collections::HashMap;

pub struct MediumStruct {{
    id: u32,
    name: String,
    data: HashMap<String, i32>,
}}

impl MediumStruct {{
    pub fn new(id: u32, name: String) -> Self {{
        let mut data = HashMap::new();
        for i in 0..10 {{
            data.insert(format!("key_{{}}", i), i);
        }}
        Self {{ id, name, data }}
    }}
    
    pub fn process_data(&self) -> i32 {{
        self.data.values().sum()
    }}
    
    pub fn filter_data(&self, threshold: i32) -> Vec<i32> {{
        self.data.values().filter(|&&v| v > threshold).cloned().collect()
    }}
    
    pub fn update_data(&mut self, key: String, value: i32) {{
        self.data.insert(key, value);
    }}
}}

pub fn complex_computation(input: &[i32]) -> i32 {{
    input.iter()
        .enumerate()
        .map(|(i, &val)| {{
            if i % 2 == 0 {{
                val * val
            }} else {{
                val + i as i32
            }}
        }})
        .sum()
}}
"#
        ),
        "complex" => format!(
            r#"
use std::collections::{{HashMap, HashSet}};

pub struct ComplexStruct {{
    primary_data: HashMap<String, Vec<i32>>,
    secondary_data: HashSet<String>,
    metadata: HashMap<String, String>,
    counters: Vec<u64>,
}}

impl ComplexStruct {{
    pub fn new() -> Self {{
        let mut primary_data = HashMap::new();
        let mut secondary_data = HashSet::new();
        let mut metadata = HashMap::new();
        
        for i in 0..50 {{
            primary_data.insert(format!("key_{{}}", i), (0..10).collect());
            secondary_data.insert(format!("secondary_{{}}", i));
            metadata.insert(format!("meta_{{}}", i), format!("value_{{}}", i));
        }}
        
        Self {{
            primary_data,
            secondary_data,
            metadata,
            counters: vec![0; 100],
        }}
    }}
    
    pub fn heavy_computation(&self) -> u64 {{
        let mut result = 0u64;
        
        for (key, values) in &self.primary_data {{
            if self.secondary_data.contains(key) {{
                for &value in values {{
                    result += value as u64 * value as u64;
                }}
            }}
        }}
        
        for (i, &counter) in self.counters.iter().enumerate() {{
            result += counter * i as u64;
        }}
        
        result
    }}
    
    pub fn complex_filter(&self, pattern: &str) -> Vec<String> {{
        self.metadata
            .iter()
            .filter(|(k, v)| k.contains(pattern) || v.contains(pattern))
            .map(|(k, _)| k.clone())
            .collect()
    }}
    
    pub fn batch_update(&mut self, updates: Vec<(String, Vec<i32>)>) {{
        for (key, values) in updates {{
            self.primary_data.insert(key.clone(), values);
            self.secondary_data.insert(key);
        }}
    }}
    
    pub fn analyze_patterns(&self) -> HashMap<String, usize> {{
        let mut patterns = HashMap::new();
        
        for key in self.primary_data.keys() {{
            let pattern = if key.len() > 5 {{
                &key[..5]
            }} else {{
                key
            }};
            *patterns.entry(pattern.to_string()).or_insert(0) += 1;
        }}
        
        patterns
    }}
}}

pub trait ComplexProcessor {{
    fn process(&self, data: &[i32]) -> Vec<i32>;
    fn validate(&self, input: &str) -> bool;
}}

impl ComplexProcessor for ComplexStruct {{
    fn process(&self, data: &[i32]) -> Vec<i32> {{
        data.iter()
            .enumerate()
            .filter_map(|(i, &val)| {{
                if self.counters.get(i % self.counters.len()).unwrap_or(&0) > &(val as u64) {{
                    Some(val * 2)
                }} else {{
                    None
                }}
            }})
            .collect()
    }}
    
    fn validate(&self, input: &str) -> bool {{
        input.len() > 3 && self.metadata.contains_key(input)
    }}
}}
"#
        ),
        _ => "".to_string(),
    }
}

/// Create a temporary test project
fn create_test_project(file_count: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Create lib.rs
    let mut lib_content = String::new();
    for i in 0..file_count {
        lib_content.push_str(&format!("pub mod file_{};\n", i));
    }
    fs::write(src_dir.join("lib.rs"), lib_content).unwrap();

    // Create individual files
    for i in 0..file_count {
        let complexity = match i % 3 {
            0 => "simple",
            1 => "medium",
            _ => "complex",
        };
        let content = create_test_rust_file(&format!("file_{}", i), complexity);
        fs::write(src_dir.join(&format!("file_{}.rs", i)), content).unwrap();
    }

    // Create Cargo.toml
    let cargo_toml = r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

    temp_dir
}

/// Benchmark the analysis engine with different project sizes
fn bench_analysis_engine_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_engine_scalability");

    for &file_count in &[10, 50, 100, 200] {
        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::new("project_analysis", file_count),
            &file_count,
            |b, &file_count| {
                let temp_project = create_test_project(file_count);

                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut engine = AnalysisEngine::new().unwrap();
                        let result = engine.analyze(temp_project.path()).await;
                        black_box(result)
                    })
                });
            },
        );
    }
    group.finish();
}

/// Benchmark God Object detection specifically
fn bench_god_object_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("god_object_detection");

    for &file_count in &[10, 25, 50, 100] {
        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::new("god_object_files", file_count),
            &file_count,
            |b, &file_count| {
                let temp_project = create_test_project(file_count);
                let detector = GodObjectDetector::default();

                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        // Simulate processing each file
                        for i in 0..file_count {
                            let file_path = temp_project
                                .path()
                                .join("src")
                                .join(&format!("file_{}.rs", i));
                            if let Ok(source) = fs::read_to_string(&file_path) {
                                let parsed_file = create_mock_parsed_file(&file_path, &source);
                                let result = detector.detect_issues(&parsed_file).await;
                                black_box(result);
                            }
                        }
                    })
                });
            },
        );
    }
    group.finish();
}

/// Create a mock ParsedFile for testing
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

/// Benchmark different analysis phases
fn bench_analysis_phases(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_phases");
    let temp_project = create_test_project(50);

    group.bench_function("initialization", |b| {
        b.iter(|| {
            let engine = AnalysisEngine::new().unwrap();
            black_box(engine)
        })
    });

    group.bench_function("file_discovery", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut engine = AnalysisEngine::new().unwrap();
                // Simulate file discovery phase
                let mut files = Vec::new();
                for entry in walkdir::WalkDir::new(temp_project.path()) {
                    if let Ok(entry) = entry {
                        if entry.path().extension().map_or(false, |ext| ext == "rs") {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                }
                black_box(files)
            })
        })
    });

    group.bench_function("parsing_phase", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Simulate parsing a single complex file
                let file_path = temp_project.path().join("src").join("file_2.rs"); // complex file
                if let Ok(source) = fs::read_to_string(&file_path) {
                    let parsed = create_mock_parsed_file(&file_path, &source);
                    black_box(parsed);
                }
                // Return unit value to maintain consistency
                black_box(())
            })
        })
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("large_project_memory", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();

            for _ in 0..iters {
                let temp_project = create_test_project(100);
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut engine = AnalysisEngine::new().unwrap();
                    let result = engine.analyze(temp_project.path()).await;
                    black_box(result);
                });
                // Project automatically cleaned up when temp_project goes out of scope
            }

            start.elapsed()
        })
    });

    group.finish();
}

/// Benchmark concurrent analysis
fn bench_concurrent_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_analysis");

    for &thread_count in &[1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_projects", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut handles = Vec::new();

                        for _ in 0..thread_count {
                            let handle = tokio::spawn(async move {
                                let temp_project = create_test_project(25);
                                let mut engine = AnalysisEngine::new().unwrap();
                                let result = engine.analyze(temp_project.path()).await;
                                black_box(result)
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            let _ = handle.await;
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    analysis_engine_benches,
    bench_analysis_engine_scalability,
    bench_god_object_detection,
    bench_analysis_phases,
    bench_memory_patterns,
    bench_concurrent_analysis
);

criterion_main!(analysis_engine_benches);
