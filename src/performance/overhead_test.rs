//! Simple overhead testing for statistical analysis

use super::statistical_analysis::StatisticalAnalyzer;
use super::trend_detection::TrendDetector;
use std::time::Instant;
use tracing::{debug, error, info, warn};

pub fn measure_statistical_overhead() {
    println!("=== Performance Overhead Validation ===");

    let analyzer = StatisticalAnalyzer::new();
    let detector = TrendDetector::new();

    // Generate test data
    let small_data: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.1).collect();
    let medium_data: Vec<f64> = (0..500).map(|i| 100.0 + i as f64 * 0.1).collect();
    let large_data: Vec<f64> = (0..1000).map(|i| 100.0 + i as f64 * 0.1).collect();

    // Test Mann-Kendall performance
    println!("\n--- Mann-Kendall Test Performance ---");

    let start = Instant::now();
    for _ in 0..10 {
        let _ = analyzer.mann_kendall_test(&small_data).unwrap();
    }
    let small_avg = start.elapsed() / 10;
    println!("Small data (100 points): {:?} per test", small_avg);

    let start = Instant::now();
    for _ in 0..10 {
        let _ = analyzer.mann_kendall_test(&medium_data).unwrap();
    }
    let medium_avg = start.elapsed() / 10;
    println!("Medium data (500 points): {:?} per test", medium_avg);

    let start = Instant::now();
    for _ in 0..5 {
        let _ = analyzer.mann_kendall_test(&large_data).unwrap();
    }
    let large_avg = start.elapsed() / 5;
    println!("Large data (1000 points): {:?} per test", large_avg);

    // Test change point detection performance
    println!("\n--- Change Point Detection Performance ---");

    let start = Instant::now();
    for _ in 0..10 {
        let _ = detector.detect_change_points_pelt(&small_data).unwrap();
    }
    let cp_small_avg = start.elapsed() / 10;
    println!("Small data change points: {:?} per test", cp_small_avg);

    let start = Instant::now();
    for _ in 0..5 {
        let _ = detector.detect_change_points_pelt(&medium_data).unwrap();
    }
    let cp_medium_avg = start.elapsed() / 5;
    println!("Medium data change points: {:?} per test", cp_medium_avg);

    // Test complete statistical analysis pipeline
    println!("\n--- Complete Analysis Pipeline ---");

    let start = Instant::now();
    for _ in 0..10 {
        let _mk = analyzer.mann_kendall_test(&small_data).unwrap();
        let _cp = detector.detect_change_points_pelt(&small_data).unwrap();
        let _ci = analyzer.confidence_interval(&small_data, 0.95).unwrap();
        let _es = analyzer
            .effect_size(&small_data[..50], &small_data[50..])
            .unwrap();
    }
    let pipeline_avg = start.elapsed() / 10;
    println!(
        "Complete analysis (100 points): {:?} per test",
        pipeline_avg
    );

    // Performance overhead assessment
    println!("\n--- Overhead Assessment ---");

    // Simple threshold check (baseline)
    let start = Instant::now();
    for _ in 0..1000 {
        let current_value = 120.0;
        let baseline_mean = 100.0;
        let _is_regression = (current_value - baseline_mean) / baseline_mean * 100.0 > 10.0;
    }
    let simple_avg = start.elapsed() / 1000;
    println!("Simple threshold check: {:?} per test", simple_avg);

    // Statistical analysis overhead
    let baseline_data = &small_data[..50];
    let start = Instant::now();
    for _ in 0..100 {
        let current_data = &[120.0];
        let _mk = analyzer.mann_kendall_test(baseline_data).unwrap();
        let _es = analyzer.effect_size(baseline_data, current_data).unwrap();
    }
    let stats_avg = start.elapsed() / 100;
    println!("Statistical analysis: {:?} per test", stats_avg);

    // Calculate overhead percentage
    let overhead_ratio = stats_avg.as_nanos() as f64 / simple_avg.as_nanos() as f64;
    let overhead_percent = (overhead_ratio - 1.0) * 100.0;

    println!("\n--- Results ---");
    println!(
        "Statistical analysis overhead: {:.1}x slower",
        overhead_ratio
    );
    println!("Overhead percentage: {:.1}%", overhead_percent);

    if overhead_percent < 500.0 {
        // 5x slower = 500% overhead
        info!("✅ PASS: Overhead is within acceptable limits (<500%)");
    } else {
        println!("❌ FAIL: Overhead is too high (>500%)");
    }

    // Memory usage estimation
    println!("\n--- Memory Usage ---");
    let analyzer_size = std::mem::size_of::<StatisticalAnalyzer>();
    let detector_size = std::mem::size_of::<TrendDetector>();
    println!("StatisticalAnalyzer size: {} bytes", analyzer_size);
    println!("TrendDetector size: {} bytes", detector_size);
    println!(
        "Total static overhead: {} bytes",
        analyzer_size + detector_size
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistical_performance_overhead() {
        measure_statistical_overhead();
    }
}
