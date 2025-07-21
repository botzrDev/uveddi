//! Performance benchmarks for statistical regression detection

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uveddi::performance::{StatisticalAnalyzer, TrendDetector};
use std::time::Duration;

fn generate_test_data(size: usize, pattern: &str) -> Vec<f64> {
    match pattern {
        "increasing" => (0..size).map(|i| i as f64).collect(),
        "decreasing" => (0..size).rev().map(|i| i as f64).collect(),
        "stable" => vec![100.0; size],
        "noisy" => (0..size).map(|i| {
            let base = 100.0;
            let noise = ((i as f64 * 0.1).sin()) * 10.0;
            base + noise
        }).collect(),
        "step_change" => {
            let mut data = vec![50.0; size / 2];
            data.extend(vec![100.0; size / 2]);
            data
        },
        _ => vec![0.0; size],
    }
}

fn bench_mann_kendall_test(c: &mut Criterion) {
    let analyzer = StatisticalAnalyzer::new();
    
    let mut group = c.benchmark_group("mann_kendall_test");
    
    for size in [50, 100, 500, 1000].iter() {
        for pattern in ["increasing", "stable", "noisy"].iter() {
            let data = generate_test_data(*size, pattern);
            
            group.bench_with_input(
                BenchmarkId::new(format!("{}_{}", pattern, size), size),
                &data,
                |b, data| {
                    b.iter(|| {
                        black_box(analyzer.mann_kendall_test(black_box(data)).unwrap())
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn bench_change_point_detection(c: &mut Criterion) {
    let detector = TrendDetector::new();
    
    let mut group = c.benchmark_group("change_point_detection");
    
    for size in [100, 500, 1000].iter() {
        for pattern in ["step_change", "noisy", "stable"].iter() {
            let data = generate_test_data(*size, pattern);
            
            group.bench_with_input(
                BenchmarkId::new(format!("pelt_{}_{}", pattern, size), size),
                &data,
                |b, data| {
                    b.iter(|| {
                        black_box(detector.detect_change_points_pelt(black_box(data)).unwrap())
                    })
                },
            );
            
            group.bench_with_input(
                BenchmarkId::new(format!("binary_{}_{}", pattern, size), size),
                &data,
                |b, data| {
                    b.iter(|| {
                        black_box(detector.detect_change_points_binary(black_box(data)).unwrap())
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn bench_confidence_intervals(c: &mut Criterion) {
    let analyzer = StatisticalAnalyzer::new();
    
    let mut group = c.benchmark_group("confidence_intervals");
    
    for size in [50, 100, 500, 1000].iter() {
        let data = generate_test_data(*size, "noisy");
        
        group.bench_with_input(
            BenchmarkId::new("95_percent", size),
            &data,
            |b, data| {
                b.iter(|| {
                    black_box(analyzer.confidence_interval(black_box(data), 0.95).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_effect_size_calculation(c: &mut Criterion) {
    let analyzer = StatisticalAnalyzer::new();
    
    let mut group = c.benchmark_group("effect_size");
    
    for size in [50, 100, 500].iter() {
        let baseline = generate_test_data(*size, "stable");
        let current = generate_test_data(*size / 4, "increasing");
        
        group.bench_with_input(
            BenchmarkId::new("cohens_d", size),
            &(baseline, current),
            |b, (baseline, current)| {
                b.iter(|| {
                    black_box(analyzer.effect_size(black_box(baseline), black_box(current)).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_complete_statistical_analysis(c: &mut Criterion) {
    let analyzer = StatisticalAnalyzer::new();
    let detector = TrendDetector::new();
    
    let mut group = c.benchmark_group("complete_analysis");
    
    for size in [100, 500, 1000].iter() {
        let data = generate_test_data(*size, "step_change");
        
        group.bench_with_input(
            BenchmarkId::new("full_analysis", size),
            &data,
            |b, data| {
                b.iter(|| {
                    // Complete statistical analysis pipeline
                    let mann_kendall = analyzer.mann_kendall_test(black_box(data)).unwrap();
                    let change_points = detector.detect_change_points_pelt(black_box(data)).unwrap();
                    let confidence_interval = analyzer.confidence_interval(black_box(data), 0.95).unwrap();
                    
                    black_box((mann_kendall, change_points, confidence_interval))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_regression_detection_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_overhead");
    group.measurement_time(Duration::from_secs(10));
    
    // Simulate overhead compared to simple threshold-based detection
    let simple_threshold = 10.0;
    let baseline_mean = 100.0;
    let baseline_std = 5.0;
    
    let test_values = vec![95.0, 105.0, 115.0, 125.0, 135.0];
    
    group.bench_function("simple_threshold", |b| {
        b.iter(|| {
            for &value in &test_values {
                let is_regression = (value - baseline_mean) / baseline_mean * 100.0 > simple_threshold;
                black_box(is_regression);
            }
        })
    });
    
    group.bench_function("statistical_analysis", |b| {
        let analyzer = StatisticalAnalyzer::new();
        let baseline_data = generate_test_data(100, "stable");
        
        b.iter(|| {
            for &value in &test_values {
                let current_data = vec![value];
                let effect_size = analyzer.effect_size(&baseline_data, &current_data).unwrap();
                let mann_kendall = analyzer.mann_kendall_test(&baseline_data).unwrap();
                black_box((effect_size, mann_kendall));
            }
        })
    });
    
    group.finish();
}

fn bench_memory_usage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    // Test different data access patterns that might affect cache performance
    let large_data = generate_test_data(10000, "noisy");
    let analyzer = StatisticalAnalyzer::new();
    
    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            for chunk_size in [100, 500, 1000].iter() {
                let chunk = &large_data[0..*chunk_size];
                let result = analyzer.mann_kendall_test(black_box(chunk)).unwrap();
                black_box(result);
            }
        })
    });
    
    group.bench_function("random_access", |b| {
        b.iter(|| {
            let indices = [0, 1000, 2000, 5000, 7000, 9000];
            for &start in indices.iter() {
                let end = (start + 100).min(large_data.len());
                if start < large_data.len() && end > start {
                    let chunk = &large_data[start..end];
                    let result = analyzer.mann_kendall_test(black_box(chunk)).unwrap();
                    black_box(result);
                }
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_mann_kendall_test,
    bench_change_point_detection,
    bench_confidence_intervals,
    bench_effect_size_calculation,
    bench_complete_statistical_analysis,
    bench_regression_detection_overhead,
    bench_memory_usage_patterns
);

criterion_main!(benches);