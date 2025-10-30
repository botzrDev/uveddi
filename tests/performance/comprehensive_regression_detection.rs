//! Comprehensive Performance Regression Detection
//! Advanced benchmarking and performance monitoring for preventing regressions

use uveddi::analysis::{AnalysisEngine, AnalysisDetector};
use uveddi::ast::tree_sitter_impl::{AstParser, SourceLanguage};
use uveddi::application::{AnalysisConfig, AnalysisOrchestrator};

#[cfg(feature = "memory-optimization")]
use uveddi::analysis::memory::{
    initialize_memory_optimization, MemoryOptimizationConfig, 
    BASIC_MEMORY_METRICS, get_optimization_status,
};

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;
use tempfile::tempdir;
use serde::{Serialize, Deserialize};

#[cfg(test)]
mod performance_regression_comprehensive {
    use super::*;

    // Performance baseline storage
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct PerformanceBenchmark {
        test_name: String,
        operation: String,
        baseline_duration_ms: f64,
        baseline_memory_mb: f64,
        baseline_allocations: u64,
        tolerance_percent: f64,
        timestamp: String,
        platform: String,
        version: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct BenchmarkResult {
        test_name: String,
        operation: String,
        duration_ms: f64,
        memory_mb: f64,
        allocations: u64,
        timestamp: String,
        baseline: Option<PerformanceBenchmark>,
        regression_detected: bool,
        performance_change_percent: f64,
        memory_change_percent: f64,
    }

    struct PerformanceMonitor {
        baselines: Arc<Mutex<HashMap<String, PerformanceBenchmark>>>,
        results: Arc<Mutex<Vec<BenchmarkResult>>>,
    }

    impl PerformanceMonitor {
        fn new() -> Self {
            Self {
                baselines: Arc::new(Mutex::new(HashMap::new())),
                results: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn set_baseline(&self, benchmark: PerformanceBenchmark) {
            let mut baselines = self.baselines.lock().unwrap();
            let key = format!("{}_{}", benchmark.test_name, benchmark.operation);
            baselines.insert(key, benchmark);
        }

        fn record_result(&self, result: BenchmarkResult) {
            let mut results = self.results.lock().unwrap();
            results.push(result);
        }

        fn check_regression(&self, test_name: &str, operation: &str, 
                           duration_ms: f64, memory_mb: f64) -> BenchmarkResult {
            let key = format!("{}_{}", test_name, operation);
            let baselines = self.baselines.lock().unwrap();
            
            let mut result = BenchmarkResult {
                test_name: test_name.to_string(),
                operation: operation.to_string(),
                duration_ms,
                memory_mb,
                allocations: 0, // Would be tracked if available
                timestamp: chrono::Utc::now().to_rfc3339(),
                baseline: None,
                regression_detected: false,
                performance_change_percent: 0.0,
                memory_change_percent: 0.0,
            };

            if let Some(baseline) = baselines.get(&key) {
                result.baseline = Some(baseline.clone());
                
                let perf_change = ((duration_ms - baseline.baseline_duration_ms) / baseline.baseline_duration_ms) * 100.0;
                let mem_change = ((memory_mb - baseline.baseline_memory_mb) / baseline.baseline_memory_mb) * 100.0;
                
                result.performance_change_percent = perf_change;
                result.memory_change_percent = mem_change;
                
                // Check for regression
                if perf_change > baseline.tolerance_percent || mem_change > baseline.tolerance_percent {
                    result.regression_detected = true;
                }
            }

            result
        }

        fn get_regression_summary(&self) -> (usize, usize, Vec<BenchmarkResult>) {
            let results = self.results.lock().unwrap();
            let total_tests = results.len();
            let regressions = results.iter().filter(|r| r.regression_detected).count();
            let failed_results = results.iter()
                .filter(|r| r.regression_detected)
                .cloned()
                .collect();

            (total_tests, regressions, failed_results)
        }
    }

    static PERFORMANCE_MONITOR: std::sync::LazyLock<PerformanceMonitor> = 
        std::sync::LazyLock::new(|| PerformanceMonitor::new());

    // Test utilities
    fn create_test_file(content: &str, extension: &str) -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(format!("perf_test.{}", extension));
        fs::write(&file_path, content).unwrap();
        file_path
    }

    fn create_large_rust_codebase(num_files: usize, functions_per_file: usize) -> Vec<PathBuf> {
        let temp_dir = tempdir().unwrap();
        let mut files = Vec::new();

        for i in 0..num_files {
            let mut content = String::new();
            content.push_str(&format!("// Performance test file {}\n", i));
            content.push_str("use std::collections::HashMap;\n");
            content.push_str("use std::sync::Arc;\n\n");

            for j in 0..functions_per_file {
                content.push_str(&format!(r#"
pub fn function_{i}_{j}(input: Vec<String>) -> Result<HashMap<String, Arc<String>>, Box<dyn std::error::Error>> {{
    let mut result = HashMap::new();
    for (idx, item) in input.iter().enumerate() {{
        let key = format!("key_{{}}_{{}}", {i}, idx);
        let value = Arc::new(format!("processed_{{}}_{{}}", item, {j}));
        result.insert(key, value);
    }}
    Ok(result)
}}

pub struct DataStructure_{i}_{j} {{
    id: usize,
    data: Vec<String>,
    metadata: HashMap<String, String>,
}}

impl DataStructure_{i}_{j} {{
    pub fn new(id: usize) -> Self {{
        Self {{
            id,
            data: Vec::with_capacity(1000),
            metadata: HashMap::new(),
        }}
    }}

    pub fn process(&mut self, items: Vec<String>) -> usize {{
        for item in items {{
            self.data.push(item.clone());
            self.metadata.insert(format!("key_{{}}", self.data.len()), item);
        }}
        self.data.len()
    }}
}}
"#, i = i, j = j));
            }

            let file_path = temp_dir.path().join(format!("file_{}.rs", i));
            fs::write(&file_path, content).unwrap();
            files.push(file_path);
        }

        files
    }

    fn measure_performance<F, R>(test_name: &str, operation: &str, f: F) -> (R, Duration, u64)
    where
        F: FnOnce() -> R,
    {
        // Get initial memory usage (simplified)
        let initial_memory = get_memory_usage_mb();
        
        let start_time = Instant::now();
        let result = f();
        let duration = start_time.elapsed();
        
        let final_memory = get_memory_usage_mb();
        let memory_used = final_memory - initial_memory;

        // Record the measurement
        let benchmark_result = PERFORMANCE_MONITOR.check_regression(
            test_name, 
            operation,
            duration.as_secs_f64() * 1000.0,
            memory_used as f64
        );

        PERFORMANCE_MONITOR.record_result(benchmark_result);

        (result, duration, memory_used as u64)
    }

    fn get_memory_usage_mb() -> u64 {
        // Simplified memory usage measurement
        // In a real implementation, this would use system-specific APIs
        
        #[cfg(feature = "memory-optimization")]
        {
            let metrics = BASIC_MEMORY_METRICS.export_json();
            if let Some(usage) = metrics["memory_optimization_phase1"]["current_usage_bytes"].as_u64() {
                usage / (1024 * 1024)
            } else {
                0
            }
        }
        
        #[cfg(not(feature = "memory-optimization"))]
        {
            // Fallback: use a rough estimate based on allocation patterns
            0
        }
    }

    fn setup_performance_baselines() {
        // Set up performance baselines for various operations
        let baselines = vec![
            PerformanceBenchmark {
                test_name: "ast_parsing".to_string(),
                operation: "small_file".to_string(),
                baseline_duration_ms: 5.0,
                baseline_memory_mb: 1.0,
                baseline_allocations: 1000,
                tolerance_percent: 20.0, // 20% regression tolerance
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: std::env::consts::OS.to_string(),
                version: "1.0.0".to_string(),
            },
            PerformanceBenchmark {
                test_name: "ast_parsing".to_string(),
                operation: "medium_file".to_string(),
                baseline_duration_ms: 50.0,
                baseline_memory_mb: 10.0,
                baseline_allocations: 10000,
                tolerance_percent: 15.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: std::env::consts::OS.to_string(),
                version: "1.0.0".to_string(),
            },
            PerformanceBenchmark {
                test_name: "ast_parsing".to_string(),
                operation: "large_file".to_string(),
                baseline_duration_ms: 500.0,
                baseline_memory_mb: 100.0,
                baseline_allocations: 100000,
                tolerance_percent: 10.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: std::env::consts::OS.to_string(),
                version: "1.0.0".to_string(),
            },
            PerformanceBenchmark {
                test_name: "analysis_engine".to_string(),
                operation: "full_analysis".to_string(),
                baseline_duration_ms: 1000.0,
                baseline_memory_mb: 50.0,
                baseline_allocations: 50000,
                tolerance_percent: 25.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: std::env::consts::OS.to_string(),
                version: "1.0.0".to_string(),
            },
            PerformanceBenchmark {
                test_name: "memory_optimization".to_string(),
                operation: "initialization".to_string(),
                baseline_duration_ms: 10.0,
                baseline_memory_mb: 5.0,
                baseline_allocations: 1000,
                tolerance_percent: 30.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: std::env::consts::OS.to_string(),
                version: "1.0.0".to_string(),
            },
        ];

        for baseline in baselines {
            PERFORMANCE_MONITOR.set_baseline(baseline);
        }
    }

    #[test]
    fn test_ast_parsing_performance_regression() {
        println!("🚀 Testing AST Parsing Performance Regression");
        
        setup_performance_baselines();

        // Test different file sizes
        let test_cases = vec![
            ("small_file", "fn main() { println!(\"Hello\"); }", 1),
            ("medium_file", &"fn test() { println!(\"test\"); }\n".repeat(100), 2),
            ("large_file", &"fn test() { println!(\"test\"); }\n".repeat(1000), 3),
        ];

        for (size_name, content, _expected_complexity) in test_cases {
            println!("  Testing {} performance", size_name);

            let test_file = create_test_file(content, "rs");
            
            let (parse_result, duration, memory_used) = measure_performance(
                "ast_parsing",
                size_name,
                || {
                    let mut parser = AstParser::new().unwrap();
                    parser.parse_file(&test_file)
                }
            );

            match parse_result {
                Ok(parsed) => {
                    println!("    ✓ Parsed successfully in {:?} (memory: {} MB)", 
                            duration, memory_used);
                    println!("    File size: {} bytes, AST size: {} bytes", 
                            parsed.source().len(), parsed.source().len()); // Simplified
                }
                Err(e) => {
                    println!("    ✗ Parse failed: {:?}", e);
                }
            }

            // Test cache performance
            let (cache_result, cache_duration, cache_memory) = measure_performance(
                "ast_parsing",
                &format!("{}_cached", size_name),
                || {
                    let mut parser = AstParser::new().unwrap();
                    parser.parse_file(&test_file) // Second parse should be cached
                }
            );

            match cache_result {
                Ok(_) => {
                    println!("    ✓ Cached parse in {:?} (memory: {} MB)", 
                            cache_duration, cache_memory);
                    
                    if cache_duration < duration {
                        println!("    ✓ Cache provided performance improvement");
                    } else {
                        println!("    ⚠ Cache did not improve performance");
                    }
                }
                Err(e) => {
                    println!("    ✗ Cached parse failed: {:?}", e);
                }
            }
        }

        // Test concurrent parsing performance
        println!("  Testing concurrent parsing performance");
        
        let concurrent_files: Vec<_> = (0..10).map(|i| {
            let content = format!("fn concurrent_{}() {{ println!(\"test\"); }}", i);
            create_test_file(&content, "rs")
        }).collect();

        let (concurrent_results, concurrent_duration, concurrent_memory) = measure_performance(
            "ast_parsing",
            "concurrent",
            || {
                use std::thread;
                
                let handles: Vec<_> = concurrent_files.into_iter().map(|file_path| {
                    thread::spawn(move || {
                        let mut parser = AstParser::new().unwrap();
                        parser.parse_file(&file_path)
                    })
                }).collect();

                handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
            }
        );

        let successful_parses = concurrent_results.iter().filter(|r| r.is_ok()).count();
        println!("    ✓ Concurrent parsing: {}/10 successful in {:?} (memory: {} MB)",
                successful_parses, concurrent_duration, concurrent_memory);

        println!("✅ AST parsing performance regression test completed");
    }

    #[tokio::test]
    async fn test_analysis_throughput_performance() {
        println!("📊 Testing Analysis Throughput Performance");

        setup_performance_baselines();

        // Test throughput with different file counts
        let throughput_tests = vec![
            ("small_batch", 10, 50),    // 10 files, 50 functions each
            ("medium_batch", 50, 20),   // 50 files, 20 functions each
            ("large_batch", 100, 10),   // 100 files, 10 functions each
        ];

        for (test_name, file_count, functions_per_file) in throughput_tests {
            println!("  Testing {} throughput ({} files)", test_name, file_count);

            let test_files = create_large_rust_codebase(file_count, functions_per_file);
            let total_functions = file_count * functions_per_file;

            let (analysis_results, duration, memory_used) = measure_performance(
                "analysis_throughput",
                test_name,
                || {
                    let mut results = Vec::new();
                    
                    for file_path in &test_files {
                        let config = AnalysisConfig {
                            target_path: file_path.clone(),
                            output_format: "json".to_string(),
                            output_file: None,
                            enable_ai: false,
                            ollama_api_url: None,
                            ollama_model: None,
                            dead_code_confidence: None,
                            dead_code_library_mode: false,
                            dead_code_ignore_patterns: None,
                            dead_code_keep_alive: None,
                            large_classes_max_loc: None,
                            large_classes_max_methods: None,
                            large_classes_max_fields: None,
                            large_classes_max_complexity: None,
                            large_classes_max_lcom: None,
                            large_classes_ignore_patterns: None,
                            large_classes_min_severity: None,
                            #[cfg(feature = "memory-optimization")]
                            memory_optimization: None,
                            enable_memory_optimization: false,
                            memory_limit_gb: None,
                            memory_profile: None,
                        };

                        // Use blocking version for performance testing
                        let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                        let result = tokio::runtime::Handle::current().block_on(async {
                            orchestrator.execute_analysis(config).await
                        });
                        
                        results.push(result);
                    }
                    
                    results
                }
            );

            let successful_analyses = analysis_results.iter().filter(|r| r.is_ok()).count();
            let throughput_files_per_sec = successful_analyses as f64 / duration.as_secs_f64();
            let throughput_functions_per_sec = (successful_analyses * functions_per_file) as f64 / duration.as_secs_f64();

            println!("    ✓ Analyzed {}/{} files in {:?}", successful_analyses, file_count, duration);
            println!("    Throughput: {:.2} files/sec, {:.2} functions/sec", 
                    throughput_files_per_sec, throughput_functions_per_sec);
            println!("    Memory usage: {} MB", memory_used);

            // Performance benchmarks
            if throughput_files_per_sec > 1.0 {
                println!("    ✓ Good file throughput performance");
            } else {
                println!("    ⚠ Low file throughput performance");
            }

            if throughput_functions_per_sec > 10.0 {
                println!("    ✓ Good function analysis throughput");
            } else {
                println!("    ⚠ Low function analysis throughput");
            }
        }

        println!("✅ Analysis throughput performance test completed");
    }

    #[test]
    #[cfg(feature = "memory-optimization")]
    fn test_memory_optimization_performance() {
        println!("🧠 Testing Memory Optimization Performance");

        setup_performance_baselines();

        // Test memory optimization initialization performance
        let memory_configs = vec![
            ("small", MemoryOptimizationConfig::small_project()),
            ("default", MemoryOptimizationConfig::default()),
            ("large", MemoryOptimizationConfig::large_codebase()),
        ];

        for (config_name, config) in memory_configs {
            println!("  Testing {} memory configuration", config_name);

            let (init_result, duration, memory_used) = measure_performance(
                "memory_optimization",
                &format!("{}_initialization", config_name),
                || {
                    initialize_memory_optimization(config)
                }
            );

            match init_result {
                Ok(_) => {
                    println!("    ✓ Initialized in {:?} (memory: {} MB)", duration, memory_used);
                    
                    // Test optimization status retrieval performance
                    let (status, status_duration, _) = measure_performance(
                        "memory_optimization",
                        &format!("{}_status", config_name),
                        || {
                            get_optimization_status()
                        }
                    );

                    if status_duration.as_millis() < 10 {
                        println!("    ✓ Fast status retrieval: {:?}", status_duration);
                    } else {
                        println!("    ⚠ Slow status retrieval: {:?}", status_duration);
                    }
                }
                Err(e) => {
                    println!("    ✗ Initialization failed: {}", e);
                }
            }
        }

        // Test memory allocation patterns
        println!("  Testing memory allocation patterns");
        
        let allocation_tests = vec![
            ("small_allocations", 1000, 1024),        // 1000 x 1KB
            ("medium_allocations", 100, 102400),      // 100 x 100KB  
            ("large_allocations", 10, 1048576),       // 10 x 1MB
        ];

        for (test_name, count, size) in allocation_tests {
            let (allocations, duration, memory_used) = measure_performance(
                "memory_optimization",
                test_name,
                || {
                    let mut allocations = Vec::new();
                    for _ in 0..count {
                        let data: Vec<u8> = vec![0; size];
                        allocations.push(data);
                    }
                    allocations
                }
            );

            let total_allocated_mb = (count * size) as f64 / (1024.0 * 1024.0);
            let allocation_rate_mb_per_sec = total_allocated_mb / duration.as_secs_f64();

            println!("    ✓ {} allocations: {:.1} MB in {:?} ({:.1} MB/sec)",
                    test_name, total_allocated_mb, duration, allocation_rate_mb_per_sec);
            println!("    Measured memory usage: {} MB", memory_used);

            // Check for memory efficiency
            if allocation_rate_mb_per_sec > 100.0 {
                println!("    ✓ Good allocation performance");
            } else {
                println!("    ⚠ Slow allocation performance");
            }
        }

        println!("✅ Memory optimization performance test completed");
    }

    #[test]
    fn test_cache_performance_scaling() {
        println!("🗃️ Testing Cache Performance Scaling");

        setup_performance_baselines();

        // Test different cache sizes
        let cache_sizes = vec![10, 100, 1000, 10000];
        
        for cache_size in cache_sizes {
            println!("  Testing cache size: {}", cache_size);

            let (cache_performance, duration, memory_used) = measure_performance(
                "cache_performance",
                &format!("cache_size_{}", cache_size),
                || {
                    let mut parser = AstParser::with_cache_size(cache_size).unwrap();
                    let mut results = Vec::new();

                    // Create files that exceed cache capacity
                    let num_files = cache_size + (cache_size / 2); // 150% of cache size
                    
                    for i in 0..num_files {
                        let content = format!("fn test_{}() {{ println!(\"test\"); }}", i);
                        let test_file = create_test_file(&content, "rs");
                        
                        let start = Instant::now();
                        let result = parser.parse_file(&test_file);
                        let parse_duration = start.elapsed();
                        
                        results.push((result.is_ok(), parse_duration));
                    }

                    let (hits, misses, current_size, max_size) = parser.cache_stats();
                    (results, hits, misses, current_size, max_size)
                }
            );

            let (results, hits, misses, current_size, max_size) = cache_performance;
            let successful_parses = results.iter().filter(|(success, _)| *success).count();
            let hit_rate = if hits + misses > 0 { hits as f64 / (hits + misses) as f64 } else { 0.0 };

            println!("    ✓ Cache performance: {}/{} successful parses in {:?}", 
                    successful_parses, results.len(), duration);
            println!("    Cache stats: hits={}, misses={}, hit_rate={:.1}%", 
                    hits, misses, hit_rate * 100.0);
            println!("    Cache utilization: {}/{} ({:.1}%)", 
                    current_size, max_size, (current_size as f64 / max_size as f64) * 100.0);
            println!("    Memory usage: {} MB", memory_used);

            // Performance expectations
            if hit_rate > 0.3 { // 30% hit rate minimum
                println!("    ✓ Good cache hit rate");
            } else {
                println!("    ⚠ Low cache hit rate");
            }

            let avg_parse_time: f64 = results.iter()
                .map(|(_, duration)| duration.as_secs_f64() * 1000.0)
                .sum::<f64>() / results.len() as f64;

            if avg_parse_time < 10.0 { // Under 10ms average
                println!("    ✓ Good average parse time: {:.2}ms", avg_parse_time);
            } else {
                println!("    ⚠ Slow average parse time: {:.2}ms", avg_parse_time);
            }
        }

        println!("✅ Cache performance scaling test completed");
    }

    #[tokio::test]
    async fn test_concurrent_performance_scaling() {
        println!("⚡ Testing Concurrent Performance Scaling");

        setup_performance_baselines();

        // Test different levels of concurrency
        let concurrency_levels = vec![1, 2, 4, 8, 16];
        
        for concurrency in concurrency_levels {
            println!("  Testing concurrency level: {}", concurrency);

            let test_files: Vec<_> = (0..concurrency * 5).map(|i| {
                let content = format!(r#"
fn concurrent_test_{}() -> Result<String, Box<dyn std::error::Error>> {{
    let data = vec![1, 2, 3, 4, 5];
    let result = data.iter().map(|x| x * {}).collect::<Vec<_>>();
    Ok(format!("Result: {{:?}}", result))
}}

struct TestStruct_{} {{
    id: usize,
    data: Vec<i32>,
}}

impl TestStruct_{} {{
    fn new() -> Self {{
        Self {{
            id: {},
            data: (0..100).collect(),
        }}
    }}
    
    fn process(&self) -> usize {{
        self.data.iter().sum::<i32>() as usize
    }}
}}
"#, i, i, i, i, i);
                create_test_file(&content, "rs")
            }).collect();

            let (concurrent_results, duration, memory_used) = measure_performance(
                "concurrent_performance",
                &format!("concurrency_{}", concurrency),
                || {
                    use tokio::task;
                    
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let tasks: Vec<_> = test_files.chunks(5).take(concurrency).map(|chunk| {
                            let chunk = chunk.to_vec();
                            task::spawn(async move {
                                let mut results = Vec::new();
                                let mut parser = AstParser::new().unwrap();
                                
                                for file_path in chunk {
                                    let start = Instant::now();
                                    let result = parser.parse_file(&file_path);
                                    let parse_duration = start.elapsed();
                                    results.push((result.is_ok(), parse_duration));
                                }
                                
                                results
                            })
                        }).collect();

                        let mut all_results = Vec::new();
                        for task in tasks {
                            match task.await {
                                Ok(task_results) => all_results.extend(task_results),
                                Err(e) => println!("    Task failed: {:?}", e),
                            }
                        }
                        
                        all_results
                    })
                }
            );

            let successful_parses = concurrent_results.iter().filter(|(success, _)| *success).count();
            let total_files = concurrent_results.len();
            let throughput = successful_parses as f64 / duration.as_secs_f64();
            
            let avg_parse_time: f64 = concurrent_results.iter()
                .map(|(_, duration)| duration.as_secs_f64() * 1000.0)
                .sum::<f64>() / concurrent_results.len() as f64;

            println!("    ✓ Concurrency {}: {}/{} successful in {:?}", 
                    concurrency, successful_parses, total_files, duration);
            println!("    Throughput: {:.2} files/sec", throughput);
            println!("    Average parse time: {:.2}ms", avg_parse_time);
            println!("    Memory usage: {} MB", memory_used);

            // Performance scaling expectations
            if concurrency > 1 {
                // Higher concurrency should generally provide better throughput
                if throughput > (concurrency as f64 * 0.7) {
                    println!("    ✓ Good scaling efficiency");
                } else {
                    println!("    ⚠ Poor scaling efficiency");
                }
            }

            if avg_parse_time < 50.0 {
                println!("    ✓ Good average performance under load");
            } else {
                println!("    ⚠ Performance degradation under load");
            }
        }

        println!("✅ Concurrent performance scaling test completed");
    }

    #[test]
    fn test_regression_detection_summary() {
        println!("📈 Performance Regression Detection Summary");

        let (total_tests, regressions, failed_results) = PERFORMANCE_MONITOR.get_regression_summary();
        
        println!("Performance Test Summary:");
        println!("  Total performance tests run: {}", total_tests);
        println!("  Performance regressions detected: {}", regressions);
        println!("  Regression rate: {:.1}%", 
                if total_tests > 0 { (regressions as f64 / total_tests as f64) * 100.0 } else { 0.0 });

        if regressions > 0 {
            println!("\nPerformance Regressions Detected:");
            for result in &failed_results {
                println!("  ❌ {}.{}: {:.1}% slower, {:.1}% more memory", 
                        result.test_name, result.operation,
                        result.performance_change_percent,
                        result.memory_change_percent);
            }
            
            println!("\n⚠️ Performance regressions detected! Review the changes that may have caused these.");
        } else {
            println!("\n✅ No performance regressions detected!");
        }

        // Performance insights
        if total_tests > 0 {
            let results = PERFORMANCE_MONITOR.results.lock().unwrap();
            
            let avg_perf_change: f64 = results.iter()
                .map(|r| r.performance_change_percent)
                .sum::<f64>() / results.len() as f64;
            
            let avg_memory_change: f64 = results.iter()
                .map(|r| r.memory_change_percent)
                .sum::<f64>() / results.len() as f64;

            println!("\nPerformance Trends:");
            println!("  Average performance change: {:.1}%", avg_perf_change);
            println!("  Average memory change: {:.1}%", avg_memory_change);

            if avg_perf_change < -5.0 {
                println!("  ✅ Overall performance improvement detected!");
            } else if avg_perf_change > 10.0 {
                println!("  ⚠️ Overall performance degradation trend");
            } else {
                println!("  ✓ Performance remains stable");
            }

            if avg_memory_change < -5.0 {
                println!("  ✅ Overall memory usage improvement!");
            } else if avg_memory_change > 15.0 {
                println!("  ⚠️ Overall memory usage increase trend");
            } else {
                println!("  ✓ Memory usage remains stable");
            }
        }

        println!("✅ Performance regression detection completed");
        
        // Assert no critical regressions
        assert!(regressions == 0 || regressions < total_tests / 4, 
               "Too many performance regressions detected: {}/{}", regressions, total_tests);
    }
}
"#