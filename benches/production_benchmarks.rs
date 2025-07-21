//! Production-grade benchmark suite for performance validation
//! 
//! This benchmark suite validates the system meets the production requirements:
//! - 4.3M+ metrics/sec sustained throughput
//! - P99 latency < 100ms for analysis requests
//! - Memory usage < 2GB for large codebases
//! - 1000+ concurrent analyses supported

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;
use futures::future::join_all;

use uveddi::analysis::{AnalysisEngine, config::AnalysisConfig};
use uveddi::models::metrics::MetricValue;
use uveddi::observability::monitoring::PerformanceTracker;
use uveddi::cache::AstCache;

/// Performance targets for production validation
const TARGET_METRICS_PER_SECOND: u64 = 4_300_000;
const TARGET_P99_LATENCY_MS: u64 = 100;
const TARGET_MAX_MEMORY_GB: f64 = 2.0;
const TARGET_CONCURRENT_ANALYSES: usize = 1000;

/// Metric processing benchmark - validates 4.3M+ metrics/sec target
fn benchmark_metric_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("metric_processing");
    group.throughput(Throughput::Elements(TARGET_METRICS_PER_SECOND));
    
    // Test different batch sizes to find optimal throughput
    for batch_size in [1000, 10000, 100000, 1000000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_processing", batch_size),
            batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async {
                    let tracker = PerformanceTracker::new();
                    let start = Instant::now();
                    
                    // Generate test metrics
                    let metrics = generate_test_metrics(batch_size);
                    
                    // Process metrics
                    for metric in metrics {
                        tracker.record_metric("test_metric", metric).await;
                    }
                    
                    let duration = start.elapsed();
                    let throughput = batch_size as f64 / duration.as_secs_f64();
                    
                    // Validate throughput meets target
                    assert!(
                        throughput >= TARGET_METRICS_PER_SECOND as f64,
                        "Throughput {} metrics/sec below target {} metrics/sec",
                        throughput,
                        TARGET_METRICS_PER_SECOND
                    );
                    
                    throughput
                });
            },
        );
    }
    
    group.finish();
}

/// Analysis latency benchmark - validates P99 latency < 100ms target
fn benchmark_analysis_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("analysis_latency");
    
    // Test different codebase sizes
    for file_count in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("analysis_latency", file_count),
            file_count,
            |b, &file_count| {
                b.to_async(&rt).iter(|| async {
                    let engine = create_optimized_analysis_engine().await;
                    let test_files = generate_test_codebase(file_count);
                    
                    let mut latencies = Vec::new();
                    
                    // Perform multiple analysis runs to get latency distribution
                    for _ in 0..100 {
                        let start = Instant::now();
                        let _result = engine.analyze_files(&test_files).await;
                        let latency = start.elapsed();
                        latencies.push(latency.as_millis() as u64);
                    }
                    
                    // Calculate P99 latency
                    latencies.sort();
                    let p99_index = (latencies.len() as f64 * 0.99) as usize;
                    let p99_latency = latencies[p99_index.min(latencies.len() - 1)];
                    
                    // Validate P99 latency meets target
                    assert!(
                        p99_latency <= TARGET_P99_LATENCY_MS,
                        "P99 latency {}ms exceeds target {}ms for {} files",
                        p99_latency,
                        TARGET_P99_LATENCY_MS,
                        file_count
                    );
                    
                    p99_latency
                });
            },
        );
    }
    
    group.finish();
}

/// Memory efficiency benchmark - validates < 2GB memory usage
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_efficiency");
    
    // Test different codebase sizes
    for codebase_size_mb in [10, 100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", codebase_size_mb),
            codebase_size_mb,
            |b, &codebase_size_mb| {
                b.to_async(&rt).iter(|| async {
                    let initial_memory = get_memory_usage_mb();
                    
                    let engine = create_memory_optimized_engine().await;
                    let large_codebase = generate_large_codebase(codebase_size_mb);
                    
                    // Perform analysis
                    let _result = engine.analyze_codebase(&large_codebase).await;
                    
                    let final_memory = get_memory_usage_mb();
                    let memory_delta = final_memory - initial_memory;
                    
                    // Validate memory usage meets target
                    let memory_gb = memory_delta / 1024.0;
                    assert!(
                        memory_gb <= TARGET_MAX_MEMORY_GB,
                        "Memory usage {:.2}GB exceeds target {:.1}GB for {}MB codebase",
                        memory_gb,
                        TARGET_MAX_MEMORY_GB,
                        codebase_size_mb
                    );
                    
                    memory_delta
                });
            },
        );
    }
    
    group.finish();
}

/// Concurrent analysis benchmark - validates 1000+ concurrent analyses
fn benchmark_concurrent_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_analysis");
    
    // Test different concurrency levels
    for concurrency in [10, 100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let engine = Arc::new(create_concurrent_analysis_engine().await);
                    let semaphore = Arc::new(Semaphore::new(*concurrency));
                    
                    let start = Instant::now();
                    
                    // Launch concurrent analysis tasks
                    let tasks: Vec<_> = (0..*concurrency)
                        .map(|i| {
                            let engine = Arc::clone(&engine);
                            let semaphore = Arc::clone(&semaphore);
                            
                            tokio::spawn(async move {
                                let _permit = semaphore.acquire().await.unwrap();
                                let test_code = generate_test_code(i);
                                let start = Instant::now();
                                let _result = engine.analyze_code(&test_code).await;
                                start.elapsed()
                            })
                        })
                        .collect();
                    
                    // Wait for all tasks to complete
                    let results = join_all(tasks).await;
                    let total_duration = start.elapsed();
                    
                    // Validate all analyses completed successfully
                    let mut latencies = Vec::new();
                    for result in results {
                        match result {
                            Ok(latency) => latencies.push(latency.as_millis() as u64),
                            Err(e) => panic!("Analysis task failed: {}", e),
                        }
                    }
                    
                    // Calculate statistics
                    let avg_latency = latencies.iter().sum::<u64>() / latencies.len() as u64;
                    let throughput = *concurrency as f64 / total_duration.as_secs_f64();
                    
                    // Validate performance meets targets
                    assert!(
                        *concurrency >= TARGET_CONCURRENT_ANALYSES,
                        "Concurrent analysis count {} below target {}",
                        concurrency,
                        TARGET_CONCURRENT_ANALYSES
                    );
                    
                    assert!(
                        avg_latency <= TARGET_P99_LATENCY_MS * 2, // Allow 2x latency for concurrent load
                        "Average latency {}ms too high under {} concurrent load",
                        avg_latency,
                        concurrency
                    );
                    
                    throughput
                });
            },
        );
    }
    
    group.finish();
}

/// Sustained load benchmark - validates performance under sustained load
fn benchmark_sustained_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("sustained_load");
    group.sample_size(10); // Fewer samples for long-running test
    group.measurement_time(Duration::from_secs(60)); // 1-minute sustained test
    
    group.bench_function("sustained_metrics_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let tracker = PerformanceTracker::new();
            let start = Instant::now();
            let test_duration = Duration::from_secs(30); // 30-second test
            
            let mut total_metrics = 0u64;
            let mut iteration = 0;
            
            while start.elapsed() < test_duration {
                // Process batch of metrics
                let batch_size = 100000;
                let metrics = generate_test_metrics(batch_size);
                
                for metric in metrics {
                    tracker.record_metric("sustained_test", metric).await;
                    total_metrics += 1;
                }
                
                iteration += 1;
                
                // Brief pause to simulate realistic load patterns
                if iteration % 10 == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            
            let duration = start.elapsed();
            let sustained_throughput = total_metrics as f64 / duration.as_secs_f64();
            
            // Validate sustained throughput meets target
            assert!(
                sustained_throughput >= TARGET_METRICS_PER_SECOND as f64 * 0.9, // Allow 10% degradation
                "Sustained throughput {:.0} metrics/sec below 90% of target {} metrics/sec",
                sustained_throughput,
                TARGET_METRICS_PER_SECOND
            );
            
            sustained_throughput
        });
    });
    
    group.finish();
}

/// Cache performance benchmark - validates AST cache efficiency
fn benchmark_cache_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("cache_performance");
    
    // Test different cache sizes
    for cache_size in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("ast_cache_hit_ratio", cache_size),
            cache_size,
            |b, &cache_size| {
                b.to_async(&rt).iter(|| async {
                    let cache = Arc::new(AstCache::with_capacity(cache_size)?);
                    let test_files = generate_test_files_for_cache(cache_size * 2); // 2x cache size for eviction testing
                    
                    let mut hits = 0;
                    let mut misses = 0;
                    
                    // First pass - populate cache
                    for file in &test_files[..cache_size] {
                        let result = cache.get_or_parse(file).await?;
                        if result.from_cache {
                            hits += 1;
                        } else {
                            misses += 1;
                        }
                    }
                    
                    // Second pass - test hit ratio
                    for file in &test_files[..cache_size] {
                        let result = cache.get_or_parse(file).await?;
                        if result.from_cache {
                            hits += 1;
                        } else {
                            misses += 1;
                        }
                    }
                    
                    let hit_ratio = hits as f64 / (hits + misses) as f64;
                    
                    // Validate cache hit ratio meets target (>80% for repeated access)
                    assert!(
                        hit_ratio >= 0.8,
                        "Cache hit ratio {:.2}% below target 80% for cache size {}",
                        hit_ratio * 100.0,
                        cache_size
                    );
                    
                    Ok::<f64, anyhow::Error>(hit_ratio)
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions

async fn create_optimized_analysis_engine() -> AnalysisEngine {
    let config = AnalysisConfig {
        max_file_size: 1024 * 1024, // 1MB
        parallel_analysis: true,
        cache_enabled: true,
        max_depth: 10,
        ..Default::default()
    };
    
    AnalysisEngine::builder()
        .with_config(config)
        .build()
        .expect("Failed to create analysis engine")
}

async fn create_memory_optimized_engine() -> AnalysisEngine {
    let config = AnalysisConfig {
        max_file_size: 2 * 1024 * 1024, // 2MB
        parallel_analysis: true,
        cache_enabled: true,
        memory_limit_mb: Some(1024), // 1GB limit
        ..Default::default()
    };
    
    AnalysisEngine::builder()
        .with_config(config)
        .with_memory_optimization(true)
        .build()
        .expect("Failed to create memory-optimized engine")
}

async fn create_concurrent_analysis_engine() -> AnalysisEngine {
    let config = AnalysisConfig {
        max_file_size: 512 * 1024, // 512KB for faster processing
        parallel_analysis: true,
        cache_enabled: true,
        max_concurrent_tasks: Some(1000),
        ..Default::default()
    };
    
    AnalysisEngine::builder()
        .with_config(config)
        .with_concurrency_optimization(true)
        .build()
        .expect("Failed to create concurrent analysis engine")
}

fn generate_test_metrics(count: usize) -> Vec<MetricValue> {
    (0..count)
        .map(|i| MetricValue {
            name: format!("test_metric_{}", i % 100),
            value: (i as f64 * 1.5) % 1000.0,
            timestamp: std::time::SystemTime::now(),
            tags: HashMap::from([
                ("component".to_string(), format!("comp_{}", i % 10)),
                ("environment".to_string(), "benchmark".to_string()),
            ]),
        })
        .collect()
}

fn generate_test_codebase(file_count: usize) -> Vec<String> {
    (0..file_count)
        .map(|i| format!(
            "// Test file {}\n\
             fn test_function_{}() {{\n\
                 let mut data = Vec::new();\n\
                 for i in 0..{} {{\n\
                     data.push(i * {});\n\
                 }}\n\
                 data.iter().sum::<i32>()\n\
             }}",
            i, i, i % 100, i % 7
        ))
        .collect()
}

fn generate_large_codebase(size_mb: usize) -> String {
    let target_size = size_mb * 1024 * 1024; // Convert to bytes
    let base_content = "fn large_function() {\n    let data = vec![0; 1000];\n    data.iter().sum::<i32>()\n}\n";
    let content_size = base_content.len();
    let repetitions = target_size / content_size;
    
    base_content.repeat(repetitions)
}

fn generate_test_code(id: usize) -> String {
    format!(
        "// Analysis test code {}\n\
         struct TestStruct_{} {{\n\
             field1: i32,\n\
             field2: String,\n\
         }}\n\
         \n\
         impl TestStruct_{} {{\n\
             fn new(value: i32) -> Self {{\n\
                 Self {{\n\
                     field1: value,\n\
                     field2: format!(\"test_{{}}\", value),\n\
                 }}\n\
             }}\n\
             \n\
             fn process(&self) -> i32 {{\n\
                 self.field1 * {}\n\
             }}\n\
         }}",
        id, id, id, id % 10
    )
}

fn generate_test_files_for_cache(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("test_file_{}.rs", i))
        .collect()
}

fn get_memory_usage_mb() -> f64 {
    use sysinfo::{System, SystemExt};
    let mut system = System::new_all();
    system.refresh_all();
    
    if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
        process.memory() as f64 / 1024.0 // Convert to MB
    } else {
        0.0
    }
}

criterion_group!(
    production_benches,
    benchmark_metric_processing,
    benchmark_analysis_latency,
    benchmark_memory_efficiency,
    benchmark_concurrent_analysis,
    benchmark_sustained_load,
    benchmark_cache_performance
);

criterion_main!(production_benches);