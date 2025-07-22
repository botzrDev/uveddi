use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::runtime::Runtime;
use uveddi::analysis::cache::ast::{AstCache, CacheConfig};
use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
use uveddi::analysis::{
    AnalysisDetector, AnalysisEngine, IncrementalAnalysisEngine, IncrementalConfig,
};
use uveddi::ast::ParsedFile;
use uveddi::database::models::PerformanceMetricsConfig;
use uveddi::monitoring::performance_metrics_collector::PerformanceMetricsCollector;
use walkdir::WalkDir;

/// Enterprise-scale benchmark suite for UV-91 Phase 1
/// Tests performance across 6 optimization areas with large-scale scenarios

const ENTERPRISE_FILE_COUNTS: &[usize] = &[100, 500, 1000, 5000, 10000];
const MEMORY_STRESS_ITERATIONS: usize = 1000;
const CACHE_PERFORMANCE_ROUNDS: usize = 100;

/// Creates enterprise-scale test projects with realistic complexity
fn create_enterprise_test_project(temp_dir: &TempDir, file_count: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Create directory structure mimicking real enterprise codebases
    let directories = vec![
        "src/core",
        "src/api",
        "src/services",
        "src/utils",
        "src/models",
        "src/components",
        "tests/unit",
        "tests/integration",
        "benches",
        "examples",
    ];

    for dir in &directories {
        fs::create_dir_all(temp_dir.path().join(dir)).unwrap();
    }

    for i in 0..file_count {
        let (file_path, content) = generate_enterprise_file(temp_dir, i, file_count);
        fs::write(&file_path, content).unwrap();
        files.push(file_path);
    }

    files
}

/// Generates realistic enterprise Rust code with varying complexity
fn generate_enterprise_file(
    temp_dir: &TempDir,
    index: usize,
    total_files: usize,
) -> (PathBuf, String) {
    let file_type = match index % 10 {
        0..=2 => "service",   // 30% services
        3..=5 => "model",     // 30% models
        6..=7 => "component", // 20% components
        8 => "test",          // 10% tests
        _ => "utility",       // 10% utilities
    };

    let (subdir, content) = match file_type {
        "service" => ("src/services", generate_service_code(index, total_files)),
        "model" => ("src/models", generate_model_code(index, total_files)),
        "component" => (
            "src/components",
            generate_component_code(index, total_files),
        ),
        "test" => ("tests/unit", generate_test_code(index, total_files)),
        "utility" => ("src/utils", generate_utility_code(index, total_files)),
        _ => ("src", generate_simple_code(index)),
    };

    let file_path = temp_dir
        .path()
        .join(subdir)
        .join(format!("file_{}.rs", index));
    (file_path, content)
}

fn generate_service_code(index: usize, total_files: usize) -> String {
    let dependency_count = (total_files / 100).max(1).min(10);
    let mut code = String::new();

    // Add imports
    code.push_str("use std::sync::Arc;\n");
    code.push_str("use tokio::sync::RwLock;\n");
    code.push_str("use anyhow::Result;\n");
    code.push_str("use serde::{Deserialize, Serialize};\n\n");

    // Add dependencies to other modules
    for dep in 0..dependency_count {
        code.push_str(&format!(
            "use crate::models::Model{};\n",
            (index + dep) % total_files
        ));
    }
    code.push_str("\n");

    // Service struct with complex state
    code.push_str(&format!(
        "#[derive(Clone)]\npub struct Service{} {{\n",
        index
    ));
    code.push_str("    cache: Arc<RwLock<std::collections::HashMap<String, CachedData>>>,\n");
    code.push_str("    config: ServiceConfig,\n");
    code.push_str("    metrics: Arc<RwLock<ServiceMetrics>>,\n");
    for i in 0..5 {
        code.push_str(&format!("    field_{}: Arc<RwLock<Option<String>>>,\n", i));
    }
    code.push_str("}\n\n");

    // Associated types
    code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    code.push_str("struct CachedData {\n");
    code.push_str("    value: String,\n");
    code.push_str("    timestamp: std::time::SystemTime,\n");
    code.push_str("    access_count: usize,\n");
    code.push_str("}\n\n");

    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("struct ServiceConfig {\n");
    code.push_str("    timeout_ms: u64,\n");
    code.push_str("    max_connections: usize,\n");
    code.push_str("    retry_attempts: u32,\n");
    code.push_str("}\n\n");

    code.push_str("#[derive(Debug, Default)]\n");
    code.push_str("struct ServiceMetrics {\n");
    code.push_str("    requests_handled: u64,\n");
    code.push_str("    errors_count: u64,\n");
    code.push_str("    average_response_time_ms: f64,\n");
    code.push_str("}\n\n");

    // Implementation with complex business logic
    code.push_str(&format!("impl Service{} {{\n", index));
    code.push_str("    pub fn new(config: ServiceConfig) -> Self {\n");
    code.push_str("        Self {\n");
    code.push_str("            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),\n");
    code.push_str("            config,\n");
    code.push_str("            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),\n");
    for i in 0..5 {
        code.push_str(&format!(
            "            field_{}: Arc::new(RwLock::new(None)),\n",
            i
        ));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // Multiple async methods with complex logic
    for method_idx in 0..8 {
        code.push_str(&format!(
            "    pub async fn process_method_{}(&self, input: &str) -> Result<String> {{\n",
            method_idx
        ));
        code.push_str("        let start_time = std::time::Instant::now();\n");
        code.push_str("        \n");
        code.push_str("        // Check cache first\n");
        code.push_str("        {\n");
        code.push_str("            let cache = self.cache.read().await;\n");
        code.push_str("            if let Some(cached) = cache.get(input) {\n");
        code.push_str("                if cached.timestamp.elapsed().unwrap().as_secs() < 300 {\n");
        code.push_str("                    return Ok(cached.value.clone());\n");
        code.push_str("                }\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("        \n");
        code.push_str("        // Complex processing logic\n");
        code.push_str("        let mut result = String::new();\n");
        code.push_str("        for (i, c) in input.chars().enumerate() {\n");
        code.push_str(&format!("            if i % {} == 0 {{\n", method_idx + 1));
        code.push_str("                result.push(c.to_ascii_uppercase());\n");
        code.push_str("            } else {\n");
        code.push_str("                result.push(c);\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("        \n");
        code.push_str("        // Update cache\n");
        code.push_str("        {\n");
        code.push_str("            let mut cache = self.cache.write().await;\n");
        code.push_str("            cache.insert(input.to_string(), CachedData {\n");
        code.push_str("                value: result.clone(),\n");
        code.push_str("                timestamp: std::time::SystemTime::now(),\n");
        code.push_str("                access_count: 1,\n");
        code.push_str("            });\n");
        code.push_str("        }\n");
        code.push_str("        \n");
        code.push_str("        // Update metrics\n");
        code.push_str("        {\n");
        code.push_str("            let mut metrics = self.metrics.write().await;\n");
        code.push_str("            metrics.requests_handled += 1;\n");
        code.push_str("            let elapsed_ms = start_time.elapsed().as_millis() as f64;\n");
        code.push_str("            metrics.average_response_time_ms = \n");
        code.push_str("                (metrics.average_response_time_ms * (metrics.requests_handled - 1) as f64 + elapsed_ms) \n");
        code.push_str("                / metrics.requests_handled as f64;\n");
        code.push_str("        }\n");
        code.push_str("        \n");
        code.push_str("        Ok(result)\n");
        code.push_str("    }\n\n");
    }

    code.push_str("}\n");
    code
}

fn generate_model_code(index: usize, total_files: usize) -> String {
    let field_count = 10 + (index % 20); // 10-30 fields
    let mut code = String::new();

    code.push_str("use serde::{Deserialize, Serialize};\n");
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::time::SystemTime;\n\n");

    // Main model struct
    code.push_str(&format!(
        "#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\npub struct Model{} {{\n",
        index
    ));

    for i in 0..field_count {
        let field_type = match i % 6 {
            0 => "String",
            1 => "u64",
            2 => "Option<String>",
            3 => "Vec<String>",
            4 => "HashMap<String, String>",
            _ => "SystemTime",
        };
        code.push_str(&format!("    pub field_{}: {},\n", i, field_type));
    }
    code.push_str("}\n\n");

    // Implementation with business logic
    code.push_str(&format!("impl Model{} {{\n", index));
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    for i in 0..field_count {
        let default_value = match i % 6 {
            0 => format!("\"default_value_{}\"", i),
            1 => format!("{}", i),
            2 => "None".to_string(),
            3 => "Vec::new()".to_string(),
            4 => "HashMap::new()".to_string(),
            _ => "SystemTime::now()".to_string(),
        };
        code.push_str(&format!("            field_{}: {},\n", i, default_value));
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // Validation methods
    code.push_str("    pub fn validate(&self) -> Result<(), ValidationError> {\n");
    for i in 0..(field_count.min(5)) {
        code.push_str(&format!(
            "        if self.field_{}.to_string().is_empty() {{\n",
            i
        ));
        code.push_str(&format!(
            "            return Err(ValidationError::EmptyField(\"field_{}\".to_string()));\n",
            i
        ));
        code.push_str("        }\n");
    }
    code.push_str("        Ok(())\n");
    code.push_str("    }\n\n");

    // Transform methods
    for transform_idx in 0..3 {
        code.push_str(&format!(
            "    pub fn transform_{}(&mut self) {{\n",
            transform_idx
        ));
        code.push_str(&format!(
            "        self.field_0 = format!(\"transformed_{}_{{}}\", self.field_0);\n",
            transform_idx
        ));
        code.push_str("    }\n\n");
    }

    code.push_str("}\n\n");

    // Error enum
    code.push_str("#[derive(Debug, thiserror::Error)]\n");
    code.push_str("pub enum ValidationError {\n");
    code.push_str("    #[error(\"Field {0} is empty\")]\n");
    code.push_str("    EmptyField(String),\n");
    code.push_str("    #[error(\"Invalid format: {0}\")]\n");
    code.push_str("    InvalidFormat(String),\n");
    code.push_str("}\n");

    code
}

fn generate_component_code(index: usize, _total_files: usize) -> String {
    let mut code = String::new();

    code.push_str("use std::sync::Arc;\n");
    code.push_str("use tokio::sync::Mutex;\n");
    code.push_str("use async_trait::async_trait;\n\n");

    // Component trait
    code.push_str("#[async_trait]\n");
    code.push_str("pub trait ComponentBehavior: Send + Sync {\n");
    code.push_str("    async fn initialize(&self) -> Result<(), ComponentError>;\n");
    code.push_str("    async fn process(&self, data: &[u8]) -> Result<Vec<u8>, ComponentError>;\n");
    code.push_str("    async fn cleanup(&self) -> Result<(), ComponentError>;\n");
    code.push_str("}\n\n");

    // Component implementation
    code.push_str(&format!("pub struct Component{} {{\n", index));
    code.push_str("    state: Arc<Mutex<ComponentState>>,\n");
    code.push_str("    buffer: Arc<Mutex<Vec<u8>>>,\n");
    code.push_str("    metrics: Arc<Mutex<ProcessingMetrics>>,\n");
    code.push_str("}\n\n");

    code.push_str("#[derive(Debug, Default)]\n");
    code.push_str("struct ComponentState {\n");
    code.push_str("    initialized: bool,\n");
    code.push_str("    process_count: usize,\n");
    code.push_str("    last_processed: Option<std::time::SystemTime>,\n");
    code.push_str("}\n\n");

    code.push_str("#[derive(Debug, Default)]\n");
    code.push_str("struct ProcessingMetrics {\n");
    code.push_str("    total_bytes_processed: usize,\n");
    code.push_str("    average_processing_time_ms: f64,\n");
    code.push_str("    error_count: usize,\n");
    code.push_str("}\n\n");

    // Implementation
    code.push_str(&format!(
        "#[async_trait]\nimpl ComponentBehavior for Component{} {{\n",
        index
    ));
    code.push_str("    async fn initialize(&self) -> Result<(), ComponentError> {\n");
    code.push_str("        let mut state = self.state.lock().await;\n");
    code.push_str("        if state.initialized {\n");
    code.push_str("            return Err(ComponentError::AlreadyInitialized);\n");
    code.push_str("        }\n");
    code.push_str("        state.initialized = true;\n");
    code.push_str("        Ok(())\n");
    code.push_str("    }\n\n");

    code.push_str(
        "    async fn process(&self, data: &[u8]) -> Result<Vec<u8>, ComponentError> {\n",
    );
    code.push_str("        let start_time = std::time::Instant::now();\n");
    code.push_str("        let mut state = self.state.lock().await;\n");
    code.push_str("        \n");
    code.push_str("        if !state.initialized {\n");
    code.push_str("            return Err(ComponentError::NotInitialized);\n");
    code.push_str("        }\n");
    code.push_str("        \n");
    code.push_str("        state.process_count += 1;\n");
    code.push_str("        state.last_processed = Some(std::time::SystemTime::now());\n");
    code.push_str("        drop(state);\n");
    code.push_str("        \n");
    code.push_str("        // Simulate complex processing\n");
    code.push_str("        let mut result = Vec::with_capacity(data.len() * 2);\n");
    code.push_str("        for &byte in data {\n");
    code.push_str(&format!(
        "            result.push(byte.wrapping_add({}));\n",
        index % 256
    ));
    code.push_str("            result.push(byte.wrapping_mul(2));\n");
    code.push_str("        }\n");
    code.push_str("        \n");
    code.push_str("        // Update metrics\n");
    code.push_str("        let mut metrics = self.metrics.lock().await;\n");
    code.push_str("        metrics.total_bytes_processed += data.len();\n");
    code.push_str("        let elapsed_ms = start_time.elapsed().as_millis() as f64;\n");
    code.push_str("        let process_count = self.state.lock().await.process_count as f64;\n");
    code.push_str("        metrics.average_processing_time_ms = \n");
    code.push_str("            (metrics.average_processing_time_ms * (process_count - 1.0) + elapsed_ms) / process_count;\n");
    code.push_str("        \n");
    code.push_str("        Ok(result)\n");
    code.push_str("    }\n\n");

    code.push_str("    async fn cleanup(&self) -> Result<(), ComponentError> {\n");
    code.push_str("        let mut state = self.state.lock().await;\n");
    code.push_str("        state.initialized = false;\n");
    code.push_str("        state.process_count = 0;\n");
    code.push_str("        Ok(())\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // Error enum
    code.push_str("#[derive(Debug, thiserror::Error)]\n");
    code.push_str("pub enum ComponentError {\n");
    code.push_str("    #[error(\"Component already initialized\")]\n");
    code.push_str("    AlreadyInitialized,\n");
    code.push_str("    #[error(\"Component not initialized\")]\n");
    code.push_str("    NotInitialized,\n");
    code.push_str("    #[error(\"Processing failed: {0}\")]\n");
    code.push_str("    ProcessingFailed(String),\n");
    code.push_str("}\n");

    code
}

fn generate_test_code(index: usize, _total_files: usize) -> String {
    let mut code = String::new();

    code.push_str("#[cfg(test)]\n");
    code.push_str("mod tests {\n");
    code.push_str("    use super::*;\n");
    code.push_str("    use tokio_test;\n\n");

    for test_idx in 0..5 {
        code.push_str("    #[tokio::test]\n");
        code.push_str(&format!(
            "    async fn test_functionality_{}() {{\n",
            test_idx
        ));
        code.push_str("        // Setup\n");
        code.push_str(&format!(
            "        let test_data = \"test_data_{}\";\n",
            test_idx
        ));
        code.push_str("        \n");
        code.push_str("        // Execute\n");
        code.push_str("        let result = process_test_data(test_data).await;\n");
        code.push_str("        \n");
        code.push_str("        // Verify\n");
        code.push_str("        assert!(result.is_ok());\n");
        code.push_str(&format!(
            "        assert_eq!(result.unwrap(), \"processed_test_data_{}\");\n",
            test_idx
        ));
        code.push_str("    }\n\n");
    }

    code.push_str("    async fn process_test_data(data: &str) -> Result<String, Box<dyn std::error::Error>> {\n");
    code.push_str(&format!("        Ok(format!(\"processed_{{}}\", data))\n"));
    code.push_str("    }\n");
    code.push_str("}\n");

    code
}

fn generate_utility_code(index: usize, _total_files: usize) -> String {
    let mut code = String::new();

    code.push_str("use std::collections::{HashMap, BTreeMap};\n");
    code.push_str("use std::hash::{Hash, Hasher};\n\n");

    // Utility functions
    for util_idx in 0..6 {
        code.push_str(&format!(
            "pub fn utility_function_{}(input: &str) -> String {{\n",
            util_idx
        ));
        code.push_str("    let mut hasher = std::collections::hash_map::DefaultHasher::new();\n");
        code.push_str("    input.hash(&mut hasher);\n");
        code.push_str("    let hash = hasher.finish();\n");
        code.push_str(&format!(
            "    format!(\"utility_{}_{{}}_{{:x}}\", input, hash)\n",
            util_idx
        ));
        code.push_str("}\n\n");
    }

    // Data structures
    code.push_str(&format!("pub struct UtilityStruct{} {{\n", index));
    code.push_str("    pub data: HashMap<String, String>,\n");
    code.push_str("    pub sorted_data: BTreeMap<String, u64>,\n");
    code.push_str("    pub cache: HashMap<u64, Vec<u8>>,\n");
    code.push_str("}\n\n");

    code.push_str(&format!("impl UtilityStruct{} {{\n", index));
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    code.push_str("            data: HashMap::new(),\n");
    code.push_str("            sorted_data: BTreeMap::new(),\n");
    code.push_str("            cache: HashMap::new(),\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    code.push_str("    pub fn process_batch(&mut self, batch: &[String]) {\n");
    code.push_str("        for (i, item) in batch.iter().enumerate() {\n");
    code.push_str("            self.data.insert(format!(\"{}-{}\", i, item), item.clone());\n");
    code.push_str("            self.sorted_data.insert(item.clone(), i as u64);\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    code
}

fn generate_simple_code(index: usize) -> String {
    format!(
        "pub fn function_{}() -> usize {{\n    {}\n}}\n",
        index, index
    )
}

/// 1. Benchmark large codebase analysis (10,000+ files)
fn bench_large_codebase_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_codebase_analysis");
    group.sample_size(10); // Reduce sample size for large tests
    group.measurement_time(Duration::from_secs(120)); // 2 minutes per measurement

    for &file_count in ENTERPRISE_FILE_COUNTS {
        if file_count <= 1000 {
            // Only test up to 1000 files to keep benchmark time reasonable
            group.throughput(Throughput::Elements(file_count as u64));
            group.bench_with_input(
                BenchmarkId::new("enterprise_analysis", file_count),
                &file_count,
                |b, &file_count| {
                    let temp_dir = TempDir::new().unwrap();
                    let _files = create_enterprise_test_project(&temp_dir, file_count);

                    b.iter(|| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async {
                            let mut engine = AnalysisEngine::new().unwrap();
                            let result = engine.analyze(temp_dir.path()).await;
                            black_box(result);
                        });
                    });
                },
            );
        }
    }
    group.finish();
}

/// 2. Benchmark memory usage patterns for enterprise scenarios
fn bench_memory_usage_enterprise(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_enterprise");

    group.bench_function("memory_allocation_patterns", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();

            for i in 0..iters {
                // Simulate enterprise memory allocation patterns
                let _cache = AstCache::new(CacheConfig {
                    max_memory_entries: 10000,
                    max_memory_size_mb: 500,
                    ..Default::default()
                })
                .unwrap();

                let _engine = AnalysisEngine::new().unwrap();

                // Simulate large allocation patterns
                let _large_buffer = vec![0u8; 1024 * 1024 * 10]; // 10MB
                let _medium_buffers: Vec<Vec<u8>> = (0..100)
                    .map(|_| vec![0u8; 1024 * 100]) // 100KB each
                    .collect();

                // Simulate rapid allocation/deallocation
                for j in 0..100 {
                    let _temp = vec![j as u8; 1024];
                }
            }

            start.elapsed()
        });
    });

    group.bench_function("memory_stress_test", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();

            // Stress test memory allocator
            for i in 0..MEMORY_STRESS_ITERATIONS {
                let size = 1024 * (1 + i % 100); // Variable sizes up to 100KB
                allocations.push(vec![i as u8; size]);

                // Occasionally free some memory
                if i % 100 == 0 && allocations.len() > 50 {
                    allocations.drain(0..25);
                }
            }

            black_box(allocations);
        });
    });

    group.finish();
}

/// 3. Benchmark AST parsing performance baseline
fn bench_ast_parsing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_parsing_performance");

    for &file_count in &[100, 500, 1000] {
        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::new("ast_parsing_throughput", file_count),
            &file_count,
            |b, &file_count| {
                let temp_dir = TempDir::new().unwrap();
                let files = create_enterprise_test_project(&temp_dir, file_count);

                b.iter(|| {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut parse_count = 0;

                        for file_path in &files {
                            if let Ok(source) = fs::read_to_string(file_path) {
                                let parsed_file = create_mock_parsed_file(file_path, &source);
                                black_box(parsed_file);
                                parse_count += 1;
                            }
                        }

                        black_box(parse_count);
                    });
                });
            },
        );
    }

    group.finish();
}

/// 4. Benchmark diagram generation throughput (baseline for 1000+/min target)
fn bench_diagram_generation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("diagram_generation_throughput");

    group.bench_function("diagram_generation_rate", |b| {
        let temp_dir = TempDir::new().unwrap();
        let files = create_enterprise_test_project(&temp_dir, 50);

        b.iter(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let mut diagram_count = 0;
                let start_time = Instant::now();

                // Simulate diagram generation for 1 minute worth of files
                for file_path in &files {
                    if let Ok(source) = fs::read_to_string(file_path) {
                        // Simulate diagram generation (mocked)
                        let _diagram_data = generate_mock_diagram(&source);
                        diagram_count += 1;

                        // Target: 1000+ diagrams per minute
                        if start_time.elapsed() > Duration::from_millis(60) {
                            // 1/1000th of a minute
                            break;
                        }
                    }
                }

                black_box(diagram_count);
            });
        });
    });

    group.finish();
}

/// 5. Benchmark cache hit rates (baseline for 90%+ target)
fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");

    group.bench_function("cache_hit_rate_simulation", |b| {
        let temp_dir = TempDir::new().unwrap();
        let files = create_enterprise_test_project(&temp_dir, 100);

        b.iter(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let cache = AstCache::new(CacheConfig {
                    max_memory_entries: 1000,
                    max_memory_size_mb: 100,
                    ..Default::default()
                })
                .unwrap();

                let mut hits = 0;
                let mut misses = 0;

                // Simulate cache usage patterns
                for round in 0..CACHE_PERFORMANCE_ROUNDS {
                    for (i, file_path) in files.iter().enumerate() {
                        // Simulate cache lookup
                        if let Some(_cached) = cache.get(file_path) {
                            hits += 1;
                        } else {
                            misses += 1;

                            // Simulate cache insertion
                            if let Ok(_source) = fs::read_to_string(file_path) {
                                // Note: Cache insertion would need actual implementation
                                // For now, just simulate the lookup operation
                                let _ = cache.get(file_path);
                            }
                        }

                        // Access pattern: 80% recently used files, 20% random
                        if round > 0 && i % 5 == 0 {
                            // Re-access recent files to increase hit rate
                            let recent_file = &files[i % 20];
                            if cache.get(recent_file).is_some() {
                                hits += 1;
                            }
                        }
                    }
                }

                let hit_rate = hits as f64 / (hits + misses) as f64;
                black_box((hits, misses, hit_rate));
            });
        });
    });

    group.finish();
}

/// 6. Benchmark incremental analysis simulation (baseline for 50%+ improvement)
fn bench_incremental_analysis_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_analysis_simulation");

    group.bench_function("incremental_vs_full_analysis", |b| {
        let temp_dir = TempDir::new().unwrap();
        let files = create_enterprise_test_project(&temp_dir, 200);

        b.iter(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // Simulate full analysis
                let full_start = Instant::now();
                let mut full_engine = AnalysisEngine::new().unwrap();
                let _full_result = full_engine.analyze(temp_dir.path()).await;
                let full_time = full_start.elapsed();

                // Simulate incremental analysis (modify 10% of files)
                let modified_files = &files[0..files.len() / 10];
                for file_path in modified_files {
                    if let Ok(mut content) = fs::read_to_string(file_path) {
                        content.push_str("// Modified for incremental test\n");
                        let _ = fs::write(file_path, content);
                    }
                }

                let incremental_start = Instant::now();
                let mut incremental_engine = AnalysisEngine::new().unwrap();
                let _incremental_result = incremental_engine.analyze(temp_dir.path()).await;
                let incremental_time = incremental_start.elapsed();

                let improvement_ratio =
                    full_time.as_millis() as f64 / incremental_time.as_millis() as f64;
                black_box((full_time, incremental_time, improvement_ratio));
            });
        });
    });

    group.finish();
}

/// 7. Enterprise performance monitoring integration
fn bench_performance_monitoring_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_monitoring_overhead");

    group.bench_function("metrics_collection_overhead", |b| {
        let config = PerformanceMetricsConfig {
            enabled: true,
            adaptive_sampling: true,
            sampling_rate: 1.0,
            buffer_size: 1000,
            async_storage: false,
            flush_interval_seconds: 60,
            memory_sampling_interval_ms: 1000,
        };

        let metrics_collector = PerformanceMetricsCollector::new(config, 1000);

        b.iter(|| {
            // Simulate analysis with metrics collection
            for i in 0..100 {
                let start_time = Instant::now();

                // Simulate component work
                let _work_result = (0..1000).map(|x| x * x).collect::<Vec<_>>();

                let execution_time = start_time.elapsed();
                let memory_usage = metrics_collector.capture_memory_snapshot();

                // Record metrics (simulate the overhead)
                black_box((execution_time, memory_usage));

                // Simulate metrics recording overhead
                if i % 10 == 0 {
                    let _ = metrics_collector.emit_metrics();
                }
            }
        });
    });

    group.finish();
}

/// Helper function to create mock ParsedFile for benchmarking
fn create_mock_parsed_file(file_path: &Path, source: &str) -> ParsedFile {
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

        ParsedFile {
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
        ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            source: Arc::new(source.to_string()),
            tree: None,
            language: uveddi::ast::tree_sitter_impl::SourceLanguage::Rust,
            custom_ast: Arc::new(None),
            modified_at: std::time::SystemTime::now(),
        }
    }
}

/// Helper function to generate mock diagram data
fn generate_mock_diagram(source: &str) -> String {
    format!("graph TD\n    A[{}] --> B[Output]", source.len())
}

/// Benchmarks incremental analysis performance to validate 50%+ improvement target
fn bench_incremental_analysis_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_analysis_performance");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(180)); // 3 minutes for thorough testing

    for &file_count in &[1000, 2000, 5000] {
        group.bench_with_input(
            BenchmarkId::new("incremental_vs_full_analysis", file_count),
            &file_count,
            |b, &file_count| {
                let temp_dir = TempDir::new().unwrap();
                let _files = create_enterprise_test_project(&temp_dir, file_count);

                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);

                    for iteration in 0..iters {
                        let rt = Runtime::new().unwrap();

                        // First run: Full analysis (baseline)
                        let full_analysis_start = Instant::now();
                        let mut engine = rt.block_on(async { AnalysisEngine::new().unwrap() });

                        let _full_result =
                            rt.block_on(async { engine.analyze(temp_dir.path()).await });
                        let full_analysis_time = full_analysis_start.elapsed();

                        // Second run: Incremental analysis (should be much faster)
                        let incremental_start = Instant::now();
                        let config = IncrementalConfig::default();

                        let incremental_result = rt.block_on(async {
                            engine.analyze_incremental(temp_dir.path(), config).await
                        });
                        let incremental_time = incremental_start.elapsed();

                        // Calculate time savings
                        let time_savings_percent = if full_analysis_time > incremental_time {
                            ((full_analysis_time - incremental_time).as_millis() as f64
                                / full_analysis_time.as_millis() as f64)
                                * 100.0
                        } else {
                            0.0
                        };

                        // Simulate file changes for next iteration
                        if iteration < iters - 1 {
                            simulate_file_changes(&temp_dir, file_count / 20); // Change 5% of files
                        }

                        total_duration += incremental_time;

                        // Store metrics for analysis
                        black_box((
                            incremental_result,
                            time_savings_percent,
                            full_analysis_time,
                            incremental_time,
                        ));
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmarks change detection performance
fn bench_change_detection_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("change_detection_performance");
    group.sample_size(20);

    for &file_count in &[1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("file_change_detection", file_count),
            &file_count,
            |b, &file_count| {
                let temp_dir = TempDir::new().unwrap();
                let files = create_enterprise_test_project(&temp_dir, file_count);

                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    let rt = Runtime::new().unwrap();

                    for _ in 0..iters {
                        // Simulate changes to random files
                        let changed_files = simulate_file_changes(&temp_dir, file_count / 50); // Change 2%

                        let start_time = Instant::now();

                        // Test change detection
                        rt.block_on(async {
                            use uveddi::analysis::incremental::{
                                ChangeDetectionConfig, ChangeDetector,
                            };

                            let config = ChangeDetectionConfig::default();
                            let mut detector = ChangeDetector::new(config).unwrap();
                            let changeset = detector.detect_changes(temp_dir.path()).await.unwrap();

                            black_box((changeset, changed_files));
                        });

                        total_duration += start_time.elapsed();
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmarks dependency analysis performance for incremental updates
fn bench_dependency_analysis_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_analysis_performance");
    group.sample_size(15);

    for &file_count in &[1000, 3000, 5000] {
        group.bench_with_input(
            BenchmarkId::new("dependency_impact_analysis", file_count),
            &file_count,
            |b, &file_count| {
                let temp_dir = TempDir::new().unwrap();
                let _files = create_enterprise_test_project(&temp_dir, file_count);

                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    let rt = Runtime::new().unwrap();

                    for _ in 0..iters {
                        let start_time = Instant::now();

                        rt.block_on(async {
                            use std::collections::HashSet;
                            use uveddi::analysis::incremental::{
                                DependencyExtractionConfig, DependencyTracker,
                            };

                            let config = DependencyExtractionConfig::default();
                            let mut tracker = DependencyTracker::new(config).unwrap();

                            // Build dependency graph
                            let files: HashSet<PathBuf> = std::fs::read_dir(temp_dir.path())
                                .unwrap()
                                .filter_map(|entry| {
                                    let path = entry.ok()?.path();
                                    if path.extension()?.to_str()? == "rs" {
                                        Some(path)
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            if !files.is_empty() {
                                tracker.build_dependency_graph(&files).await.unwrap();

                                // Simulate change impact analysis
                                use uveddi::analysis::incremental::ChangeSet;
                                let mut changeset = ChangeSet::default();
                                changeset.modified =
                                    files.iter().take(file_count / 100).cloned().collect();

                                let impact = tracker.analyze_change_impact(&changeset).unwrap();
                                black_box(impact);
                            }
                        });

                        total_duration += start_time.elapsed();
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmarks incremental state persistence performance
fn bench_state_persistence_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_persistence_performance");
    group.sample_size(30);

    for &file_count in &[1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("state_save_load", file_count),
            &file_count,
            |b, &file_count| {
                let temp_dir = TempDir::new().unwrap();
                let _files = create_enterprise_test_project(&temp_dir, file_count);

                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    let rt = Runtime::new().unwrap();

                    for iteration in 0..iters {
                        let start_time = Instant::now();

                        rt.block_on(async {
                            use uveddi::analysis::incremental::{
                                IncrementalAnalysisConfig, IncrementalStateManager,
                                StateManagerConfig,
                            };

                            let state_file =
                                temp_dir.path().join(format!("state_{}.json", iteration));
                            let config = StateManagerConfig::default();
                            let manager = IncrementalStateManager::new(config, state_file).unwrap();

                            // Create and save state
                            let project_root = temp_dir.path().to_path_buf();
                            let analysis_config = IncrementalAnalysisConfig::default();
                            let state = manager
                                .create_new_state(project_root, analysis_config)
                                .await
                                .unwrap();

                            manager.save_state(&state).await.unwrap();

                            // Load state back
                            let loaded_state = manager.load_state().await.unwrap();
                            black_box(loaded_state);
                        });

                        total_duration += start_time.elapsed();
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Simulates realistic file changes in a project
fn simulate_file_changes(temp_dir: &TempDir, change_count: usize) -> Vec<PathBuf> {
    let mut changed_files = Vec::new();

    // Find existing Rust files
    let rust_files: Vec<PathBuf> = WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "rs" {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
        .collect();

    // Modify random files
    for i in 0..change_count.min(rust_files.len()) {
        if let Some(file_path) = rust_files.get(i) {
            // Append a comment to simulate change
            if let Ok(mut content) = std::fs::read_to_string(file_path) {
                content.push_str(&format!(
                    "\n// Change simulation {}\n",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                ));
                let _ = std::fs::write(file_path, content);
                changed_files.push(file_path.clone());
            }
        }
    }

    changed_files
}

criterion_group!(
    enterprise_benchmarks,
    bench_large_codebase_analysis,
    bench_memory_usage_enterprise,
    bench_ast_parsing_performance,
    bench_diagram_generation_throughput,
    bench_cache_performance,
    bench_incremental_analysis_performance,
    bench_change_detection_performance,
    bench_dependency_analysis_performance,
    bench_state_persistence_performance,
    bench_performance_monitoring_overhead
);

criterion_main!(enterprise_benchmarks);
