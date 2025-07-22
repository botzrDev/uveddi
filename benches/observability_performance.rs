//! Performance benchmarks for UV-86 Enterprise Observability Framework
//!
//! These benchmarks verify the "sub-millisecond overhead" claims and ensure
//! the observability framework meets enterprise performance requirements.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;
use uveddi::observability::config::LogFormat;
use uveddi::observability::telemetry::TelemetryConfig;
use uveddi::observability::{
    InstrumentedCircuitBreaker, InstrumentedFallback, InstrumentedRetry, ObservabilityConfig,
    ObservabilityService, TelemetryCollector, TraceId,
};

/// Create a test observability service with minimal configuration for benchmarks
async fn create_benchmark_service() -> ObservabilityService {
    let mut config = ObservabilityConfig::default();
    // Use human format for faster logging in benchmarks
    config.logging.format = LogFormat::Human;
    // Disable metrics server to avoid port conflicts
    config.metrics.enabled = false;

    ObservabilityService::new(config).await.unwrap()
}

/// Benchmark structured logging overhead
fn benchmark_logging_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Initialize the observability service once
    let _service = rt.block_on(create_benchmark_service());

    let mut group = c.benchmark_group("logging_overhead");

    // Test different message sizes
    let message_sizes = vec![10, 50, 100, 500];

    for size in message_sizes {
        let message = "x".repeat(size);

        group.bench_with_input(
            BenchmarkId::new("structured_logging", size),
            &message,
            |b, msg| {
                b.iter(|| {
                    tracing::info!(
                        trace_id = %TraceId::new(),
                        operation = "benchmark_operation",
                        message_size = size,
                        message = black_box(msg),
                        "Benchmark logging message"
                    );
                })
            },
        );
    }

    group.finish();
}

/// Benchmark metrics collection overhead
fn benchmark_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(create_benchmark_service());
    let metrics = service.metrics();

    let mut group = c.benchmark_group("metrics_collection");

    // Benchmark different types of metrics
    group.bench_function("counter_increment", |b| {
        b.iter(|| {
            metrics.record_error(
                black_box("test_error"),
                black_box("low"),
                black_box("benchmark"),
            );
        })
    });

    group.bench_function("histogram_observation", |b| {
        b.iter(|| {
            metrics.record_request(
                black_box("GET"),
                black_box("/api/test"),
                black_box("200"),
                black_box(Duration::from_millis(10)),
            );
        })
    });

    group.bench_function("gauge_update", |b| {
        b.iter(|| {
            metrics.update_system_metrics(
                black_box(50.0),                    // CPU
                black_box(1024.0 * 1024.0 * 100.0), // Memory (100MB)
                black_box(42.0),                    // Connections
            );
        })
    });

    group.finish();
}

/// Benchmark trace ID generation and manipulation
fn benchmark_trace_id_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("trace_id_operations");

    group.bench_function("trace_id_creation", |b| {
        b.iter(|| {
            black_box(TraceId::new());
        })
    });

    group.bench_function("trace_id_to_string", |b| {
        let trace_id = TraceId::new();
        b.iter(|| {
            black_box(trace_id.to_string());
        })
    });

    group.bench_function("trace_id_from_string", |b| {
        let trace_id = TraceId::new();
        let trace_str = trace_id.to_string();
        b.iter(|| {
            black_box(TraceId::from_str(&trace_str).unwrap());
        })
    });

    group.finish();
}

/// Benchmark telemetry event creation and processing
fn benchmark_telemetry_events(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("telemetry_events");

    group.bench_function("telemetry_collector_creation", |b| {
        b.iter(|| {
            let collector = TelemetryCollector::new(TelemetryConfig::default());
            black_box(collector);
        })
    });

    group.finish();
}

/// Benchmark resilience pattern overhead (simplified - focuses on core instrumentation)
fn benchmark_resilience_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(create_benchmark_service());
    let metrics = service.metrics();

    let mut group = c.benchmark_group("resilience_patterns");

    // Benchmark instrumented retry creation overhead
    group.bench_function("instrumented_retry_creation", |b| {
        b.iter(|| {
            let retry = InstrumentedRetry::new(
                3,                         // max_attempts
                Duration::from_millis(10), // initial_delay
                Duration::from_secs(1),    // max_delay
                2.0,                       // backoff_multiplier
                true,                      // jitter
                metrics.clone(),
                None, // telemetry
            );
            black_box(retry);
        })
    });

    // Benchmark instrumented circuit breaker creation overhead
    group.bench_function("circuit_breaker_creation", |b| {
        b.iter(|| {
            let cb = InstrumentedCircuitBreaker::new(
                "test_service".to_string(),
                5,                       // failure_threshold
                Duration::from_secs(10), // timeout
                metrics.clone(),
                None, // telemetry
            );
            black_box(cb);
        })
    });

    // Benchmark instrumented fallback creation overhead
    group.bench_function("instrumented_fallback_creation", |b| {
        b.iter(|| {
            let fallback = InstrumentedFallback::new(
                "test_service".to_string(),
                metrics.clone(),
                None, // telemetry
            );
            black_box(fallback);
        })
    });

    group.finish();
}

/// Benchmark full observability pipeline
fn benchmark_full_pipeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(create_benchmark_service());

    c.bench_function("full_observability_pipeline", |b| {
        b.iter(|| {
            rt.block_on(async {
                let trace_id = service.new_trace_id();
                let metrics = service.metrics();

                // Start request timing
                let start = std::time::Instant::now();

                // Log operation start
                tracing::info!(
                    trace_id = %trace_id,
                    operation = "benchmark_pipeline",
                    "Starting operation"
                );

                // Simulate some work
                let work_duration = Duration::from_micros(100);
                tokio::time::sleep(work_duration).await;

                // Record metrics
                let duration = start.elapsed();
                metrics.record_request("POST", "/api/benchmark", "200", duration);

                // Log operation completion
                tracing::info!(
                    trace_id = %trace_id,
                    operation = "benchmark_pipeline",
                    duration_ms = duration.as_millis(),
                    "Operation completed"
                );

                black_box(trace_id);
            })
        })
    });
}

/// Benchmark concurrent observability operations
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(create_benchmark_service());

    let mut group = c.benchmark_group("concurrent_operations");

    // Test different concurrency levels
    let concurrency_levels = vec![1, 10, 50, 100];

    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_logging", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    rt.block_on(async {
                        let tasks: Vec<_> = (0..concurrency)
                            .map(|i| {
                                let trace_id = TraceId::new();
                                tokio::spawn(async move {
                                    tracing::info!(
                                        trace_id = %trace_id,
                                        operation = "concurrent_benchmark",
                                        task_id = i,
                                        "Concurrent operation"
                                    );
                                })
                            })
                            .collect();

                        for task in tasks {
                            task.await.unwrap();
                        }

                        black_box(concurrency);
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark observability overhead under different loads
fn benchmark_load_testing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(create_benchmark_service());
    let metrics = service.metrics();

    let mut group = c.benchmark_group("load_testing");

    // Test different operation counts per batch
    let operation_counts = vec![100, 500, 1000, 5000];

    for count in operation_counts {
        group.bench_with_input(
            BenchmarkId::new("batch_operations", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    for i in 0..count {
                        let trace_id = TraceId::new();

                        // Log operation
                        tracing::debug!(
                            trace_id = %trace_id,
                            operation_id = i,
                            "Batch operation"
                        );

                        // Record metric
                        metrics.record_error("batch_error", "low", "load_test");

                        black_box((trace_id, i));
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    observability_benches,
    benchmark_logging_overhead,
    benchmark_metrics_collection,
    benchmark_trace_id_operations,
    benchmark_telemetry_events,
    benchmark_resilience_patterns,
    benchmark_full_pipeline,
    benchmark_concurrent_operations,
    benchmark_load_testing
);

criterion_main!(observability_benches);
