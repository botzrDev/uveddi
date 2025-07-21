//! Comprehensive benchmarking framework integrating Criterion.rs with statistical regression detection
//! 
//! This benchmark suite provides:
//! - Performance characterization using Criterion.rs
//! - Statistical correlation with regression detection
//! - Baseline management and comparison
//! - Performance report generation with insights

use criterion::{
    black_box, criterion_group, criterion_main, Criterion, BenchmarkId,
    measurement::WallTime, BatchSize, Throughput
};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uveddi::performance::{
    StatisticalAnalyzer, TrendDetector, PerformanceRegressionDetector,
    RegressionDetectionConfig, MetricDataPoint
};

/// Performance benchmark result with statistical correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedBenchmarkResult {
    pub benchmark_name: String,
    pub criterion_stats: CriterionStats,
    pub statistical_analysis: StatisticalAnalysis,
    pub regression_analysis: Option<RegressionAnalysis>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionStats {
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub median_ns: f64,
    pub mad_ns: f64,
    pub throughput_ops_per_sec: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    pub trend_test_result: MannKendallResult,
    pub change_point_analysis: ChangePointResult,
    pub confidence_interval_95: (f64, f64),
    pub effect_size: f64,
    pub statistical_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub is_regression: bool,
    pub regression_percentage: f64,
    pub statistical_significance: f64,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MannKendallResult {
    pub tau: f64,
    pub p_value: f64,
    pub trend: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePointResult {
    pub change_points: Vec<usize>,
    pub confidence: f64,
    pub algorithm_used: String,
}

/// Benchmarking framework with statistical correlation
pub struct CorrelatedBenchmarker {
    analyzer: StatisticalAnalyzer,
    detector: TrendDetector,
    regression_detector: Option<PerformanceRegressionDetector>,
    historical_results: Vec<CorrelatedBenchmarkResult>,
}

impl CorrelatedBenchmarker {
    pub fn new() -> Self {
        Self {
            analyzer: StatisticalAnalyzer::new(),
            detector: TrendDetector::new(),
            regression_detector: None,
            historical_results: Vec::new(),
        }
    }

    pub fn with_regression_detection(mut self, config: RegressionDetectionConfig) -> Self {
        self.regression_detector = PerformanceRegressionDetector::new(config).ok();
        self
    }

    /// Run a benchmark and correlate with statistical analysis
    pub fn run_correlated_benchmark<F>(
        &mut self,
        c: &mut Criterion,
        benchmark_name: &str,
        benchmark_fn: F,
    ) -> CorrelatedBenchmarkResult 
    where
        F: Fn(&mut criterion::Bencher) + 'static
    {
        // Collect timing samples manually for statistical analysis
        let mut samples = Vec::new();
        let sample_count = 100;
        
        // Run benchmark multiple times to collect samples
        for _ in 0..sample_count {
            let start = std::time::Instant::now();
            let mut bencher = criterion::Bencher::new(start, 1);
            benchmark_fn(&mut bencher);
            let duration = start.elapsed();
            samples.push(duration.as_nanos() as f64);
        }

        // Run official Criterion benchmark
        c.bench_function(benchmark_name, |b| benchmark_fn(b));

        // Perform statistical analysis on collected samples
        let statistical_analysis = self.analyze_samples(&samples);

        // Check for regression if detector is available
        let regression_analysis = if let Some(_detector) = &self.regression_detector {
            self.check_regression(benchmark_name, &samples)
        } else {
            None
        };

        // Create criterion stats (simplified - in real use would extract from Criterion)
        let criterion_stats = CriterionStats {
            mean_ns: samples.iter().sum::<f64>() / samples.len() as f64,
            std_dev_ns: self.calculate_std_dev(&samples),
            median_ns: self.calculate_median(&samples),
            mad_ns: self.calculate_mad(&samples),
            throughput_ops_per_sec: None,
        };

        let result = CorrelatedBenchmarkResult {
            benchmark_name: benchmark_name.to_string(),
            criterion_stats,
            statistical_analysis,
            regression_analysis,
            timestamp: SystemTime::now(),
        };

        self.historical_results.push(result.clone());
        result
    }

    /// Analyze benchmark samples using statistical methods
    fn analyze_samples(&self, samples: &[f64]) -> StatisticalAnalysis {
        // Perform Mann-Kendall trend test
        let mk_result = self.analyzer.mann_kendall_test(samples).unwrap_or_else(|_| {
            uveddi::performance::MannKendallResult {
                tau: 0.0,
                p_value: 1.0,
                trend: uveddi::performance::TrendType::NoTrend,
                confidence: 0.0,
                effect_size: 0.0,
            }
        });

        // Detect change points
        let cp_result = self.detector.detect_change_points_pelt(samples).unwrap_or_else(|_| {
            uveddi::performance::ChangePointResult {
                change_points: Vec::new(),
                segments: Vec::new(),
                confidence: 1.0,
                algorithm_used: "Failed".to_string(),
            }
        });

        // Calculate confidence interval
        let confidence_interval = self.analyzer
            .confidence_interval(samples, 0.95)
            .unwrap_or((0.0, 0.0));

        // Calculate effect size (compare against historical mean if available)
        let effect_size = if let Some(historical) = self.get_historical_baseline(samples) {
            self.analyzer.effect_size(&historical, samples).unwrap_or(0.0)
        } else {
            0.0
        };

        let statistical_confidence = mk_result.confidence.max(cp_result.confidence);

        StatisticalAnalysis {
            trend_test_result: MannKendallResult {
                tau: mk_result.tau,
                p_value: mk_result.p_value,
                trend: format!("{:?}", mk_result.trend),
                confidence: mk_result.confidence,
            },
            change_point_analysis: ChangePointResult {
                change_points: cp_result.change_points,
                confidence: cp_result.confidence,
                algorithm_used: cp_result.algorithm_used,
            },
            confidence_interval_95: confidence_interval,
            effect_size,
            statistical_confidence,
        }
    }

    /// Check for performance regression
    fn check_regression(&self, benchmark_name: &str, samples: &[f64]) -> Option<RegressionAnalysis> {
        let historical_baseline = self.get_historical_baseline(samples)?;
        
        let current_mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let baseline_mean = historical_baseline.iter().sum::<f64>() / historical_baseline.len() as f64;
        
        let regression_percentage = ((current_mean - baseline_mean) / baseline_mean) * 100.0;
        
        // Consider regression if performance degrades by more than 5%
        let is_regression = regression_percentage > 5.0;
        
        // Calculate statistical significance using effect size
        let effect_size = self.analyzer.effect_size(&historical_baseline, samples).unwrap_or(0.0);
        let statistical_significance = if effect_size.abs() > 0.2 { 0.95 } else { 0.5 };
        
        let recommended_actions = if is_regression {
            vec![
                "Review recent code changes for performance impact".to_string(),
                "Check system resource utilization during benchmark".to_string(),
                "Compare with historical performance baselines".to_string(),
                format!("Investigate {:.1}% performance degradation", regression_percentage),
            ]
        } else {
            vec!["Performance within acceptable range".to_string()]
        };

        Some(RegressionAnalysis {
            is_regression,
            regression_percentage,
            statistical_significance,
            recommended_actions,
        })
    }

    /// Get historical baseline for comparison
    fn get_historical_baseline(&self, _current_samples: &[f64]) -> Option<Vec<f64>> {
        // In a real implementation, this would load from persistent storage
        // For now, return None to indicate no historical data
        None
    }

    fn calculate_std_dev(&self, samples: &[f64]) -> f64 {
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance = samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / samples.len() as f64;
        variance.sqrt()
    }

    fn calculate_median(&self, samples: &[f64]) -> f64 {
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
        } else {
            sorted[len / 2]
        }
    }

    fn calculate_mad(&self, samples: &[f64]) -> f64 {
        let median = self.calculate_median(samples);
        let deviations: Vec<f64> = samples.iter().map(|x| (x - median).abs()).collect();
        self.calculate_median(&deviations)
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        if self.historical_results.is_empty() {
            return "No benchmark results available".to_string();
        }

        let mut report = String::new();
        report.push_str("=== Performance Analysis Report ===\n\n");

        for result in &self.historical_results {
            report.push_str(&format!("Benchmark: {}\n", result.benchmark_name));
            report.push_str(&format!("Timestamp: {:?}\n", result.timestamp));
            report.push_str(&format!("Mean: {:.2} ns\n", result.criterion_stats.mean_ns));
            report.push_str(&format!("Std Dev: {:.2} ns\n", result.criterion_stats.std_dev_ns));
            report.push_str(&format!("Statistical Confidence: {:.3}\n", 
                result.statistical_analysis.statistical_confidence));
            
            if let Some(regression) = &result.regression_analysis {
                if regression.is_regression {
                    report.push_str(&format!("⚠️  REGRESSION DETECTED: {:.1}%\n", 
                        regression.regression_percentage));
                    for action in &regression.recommended_actions {
                        report.push_str(&format!("  • {}\n", action));
                    }
                } else {
                    report.push_str("✅ No regression detected\n");
                }
            }
            
            report.push_str("\n");
        }

        report
    }
}

// Benchmark implementations using the correlated framework

static mut BENCHMARKER: Option<CorrelatedBenchmarker> = None;

fn get_benchmarker() -> &'static mut CorrelatedBenchmarker {
    unsafe {
        if BENCHMARKER.is_none() {
            BENCHMARKER = Some(CorrelatedBenchmarker::new());
        }
        BENCHMARKER.as_mut().unwrap()
    }
}

fn bench_mann_kendall_with_correlation(c: &mut Criterion) {
    let benchmarker = get_benchmarker();
    
    // Generate test data
    let test_data = (0..1000).map(|i| i as f64 + (i as f64 * 0.1).sin()).collect::<Vec<_>>();
    let analyzer = StatisticalAnalyzer::new();
    
    let _result = benchmarker.run_correlated_benchmark(c, "mann_kendall_trend_test", |b| {
        b.iter(|| {
            black_box(analyzer.mann_kendall_test(black_box(&test_data)).unwrap())
        })
    });
}

fn bench_change_point_detection_with_correlation(c: &mut Criterion) {
    let benchmarker = get_benchmarker();
    
    // Generate test data with change point
    let mut test_data = vec![100.0; 500];
    test_data.extend(vec![150.0; 500]);
    let detector = TrendDetector::new();
    
    let _result = benchmarker.run_correlated_benchmark(c, "change_point_detection", |b| {
        b.iter(|| {
            black_box(detector.detect_change_points_pelt(black_box(&test_data)).unwrap())
        })
    });
}

fn bench_complete_regression_analysis(c: &mut Criterion) {
    let benchmarker = get_benchmarker();
    
    // Simulate complete regression detection workflow
    let baseline_data = (0..100).map(|i| 100.0 + i as f64 * 0.1).collect::<Vec<_>>();
    let current_data = (0..10).map(|i| 110.0 + i as f64 * 0.15).collect::<Vec<_>>();
    
    let analyzer = StatisticalAnalyzer::new();
    let detector = TrendDetector::new();
    
    let _result = benchmarker.run_correlated_benchmark(c, "complete_regression_analysis", |b| {
        b.iter(|| {
            // Complete statistical analysis pipeline
            let mann_kendall = analyzer.mann_kendall_test(black_box(&baseline_data)).unwrap();
            let change_points = detector.detect_change_points_pelt(black_box(&baseline_data)).unwrap();
            let confidence_interval = analyzer.confidence_interval(black_box(&baseline_data), 0.95).unwrap();
            let effect_size = analyzer.effect_size(black_box(&baseline_data), black_box(&current_data)).unwrap();
            
            black_box((mann_kendall, change_points, confidence_interval, effect_size))
        })
    });
}

fn bench_throughput_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_analysis");
    
    // Set throughput measurement for ops/sec calculation
    group.throughput(Throughput::Elements(1000));
    
    let data = (0..1000).map(|i| i as f64).collect::<Vec<_>>();
    let analyzer = StatisticalAnalyzer::new();
    
    group.bench_function("mann_kendall_throughput", |b| {
        b.iter(|| {
            black_box(analyzer.mann_kendall_test(black_box(&data)).unwrap())
        })
    });
    
    group.finish();
}

fn bench_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    // Test different allocation patterns
    group.bench_function("vector_allocation", |b| {
        b.iter_batched(
            || (0..1000).map(|i| i as f64).collect::<Vec<_>>(),
            |data| {
                let analyzer = StatisticalAnalyzer::new();
                black_box(analyzer.mann_kendall_test(&data).unwrap())
            },
            BatchSize::SmallInput
        )
    });
    
    group.bench_function("slice_reuse", |b| {
        let data = (0..1000).map(|i| i as f64).collect::<Vec<_>>();
        let analyzer = StatisticalAnalyzer::new();
        
        b.iter(|| {
            black_box(analyzer.mann_kendall_test(black_box(&data)).unwrap())
        })
    });
    
    group.finish();
}

fn bench_statistical_confidence_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("confidence_levels");
    
    let data = (0..500).map(|i| 100.0 + (i as f64 * 0.01).sin() * 10.0).collect::<Vec<_>>();
    let analyzer = StatisticalAnalyzer::new();
    
    for confidence in [0.90, 0.95, 0.99].iter() {
        group.bench_with_input(
            BenchmarkId::new("confidence_interval", format!("{:.0}%", confidence * 100.0)),
            confidence,
            |b, &conf| {
                b.iter(|| {
                    black_box(analyzer.confidence_interval(black_box(&data), conf).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    criterion_integration_benches,
    bench_mann_kendall_with_correlation,
    bench_change_point_detection_with_correlation,
    bench_complete_regression_analysis,
    bench_throughput_analysis,
    bench_memory_allocation_patterns,
    bench_statistical_confidence_levels
);

criterion_main!(criterion_integration_benches);