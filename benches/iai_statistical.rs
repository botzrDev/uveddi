//! iai-callgrind benchmarks for deterministic CI performance testing
//! 
//! These benchmarks provide:
//! - Deterministic instruction-level measurements 
//! - CI-safe performance regression detection
//! - Memory allocation analysis
//! - Integration with statistical regression detection

use iai_callgrind::{library_benchmark_group, main};
use uveddi::performance::{StatisticalAnalyzer, TrendDetector};

/// Generate deterministic test data for consistent benchmarking
fn generate_deterministic_data(size: usize, pattern: &str) -> Vec<f64> {
    match pattern {
        "linear_increasing" => (0..size).map(|i| i as f64).collect(),
        "stable" => vec![100.0; size],
        "sine_wave" => (0..size).map(|i| {
            let x = i as f64 * std::f64::consts::PI / 50.0;
            100.0 + 10.0 * x.sin()
        }).collect(),
        "step_function" => {
            let mut data = Vec::with_capacity(size);
            for i in 0..size {
                let step = if i < size / 2 { 50.0 } else { 100.0 };
                data.push(step);
            }
            data
        },
        _ => vec![0.0; size],
    }
}

// Individual benchmark functions
fn mann_kendall_small_stable() {
    let data = generate_deterministic_data(100, "stable");
    let analyzer = StatisticalAnalyzer::new();
    let _result = analyzer.mann_kendall_test(&data);
}

fn mann_kendall_medium_increasing() {
    let data = generate_deterministic_data(500, "linear_increasing");
    let analyzer = StatisticalAnalyzer::new();
    let _result = analyzer.mann_kendall_test(&data);
}

fn mann_kendall_large_sine() {
    let data = generate_deterministic_data(1000, "sine_wave");
    let analyzer = StatisticalAnalyzer::new();
    let _result = analyzer.mann_kendall_test(&data);
}

fn change_point_detection_small() {
    let data = generate_deterministic_data(200, "step_function");
    let detector = TrendDetector::new();
    let _result = detector.detect_change_points_pelt(&data);
}

fn change_point_detection_medium() {
    let data = generate_deterministic_data(500, "step_function");
    let detector = TrendDetector::new();
    let _result = detector.detect_change_points_pelt(&data);
}

fn confidence_interval_calculation() {
    let data = generate_deterministic_data(300, "sine_wave");
    let analyzer = StatisticalAnalyzer::new();
    let _result = analyzer.confidence_interval(&data, 0.95);
}

fn effect_size_calculation() {
    let baseline = generate_deterministic_data(250, "stable");
    let current = generate_deterministic_data(250, "linear_increasing");
    let analyzer = StatisticalAnalyzer::new();
    let _result = analyzer.effect_size(&baseline, &current);
}

fn complete_statistical_pipeline() {
    let data = generate_deterministic_data(500, "step_function");
    let analyzer = StatisticalAnalyzer::new();
    let detector = TrendDetector::new();
    
    // Full analysis pipeline
    let _mann_kendall = analyzer.mann_kendall_test(&data);
    let _change_points = detector.detect_change_points_pelt(&data);
    let _confidence_interval = analyzer.confidence_interval(&data, 0.95);
    
    // Effect size calculation (comparing first and second half)
    if data.len() >= 20 {
        let mid = data.len() / 2;
        let first_half = &data[..mid];
        let second_half = &data[mid..];
        let _effect_size = analyzer.effect_size(first_half, second_half);
    }
}

// Define the benchmark groups using correct macro syntax
library_benchmark_group!(
    name = statistical_benchmarks;
    benchmarks = mann_kendall_small_stable, mann_kendall_medium_increasing, mann_kendall_large_sine, 
                change_point_detection_small, change_point_detection_medium,
                confidence_interval_calculation, effect_size_calculation, complete_statistical_pipeline
);

// Main benchmark runner
main!(library_benchmark_groups = statistical_benchmarks);