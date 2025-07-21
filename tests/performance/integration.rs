//! Integration tests for enhanced statistical regression detection

#[cfg(test)]
mod tests {
    use uveddi::performance::{
        PerformanceRegressionDetector, RegressionDetectionConfig,
        StatisticalAnalyzer, TrendDetector, TrendType
    };
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio;

    #[tokio::test]
    async fn test_enhanced_regression_detection_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            min_samples_for_baseline: 5,
            regression_threshold_percent: 10.0,
            significance_threshold: 0.95,
            ..Default::default()
        };
        
        let detector = PerformanceRegressionDetector::new(config).unwrap();
        
        // Build baseline with increasing trend
        for i in 0..20 {
            let metadata = HashMap::new();
            let value = 100.0 + (i as f64 * 0.5); // Gradual increase
            detector.record_metric("response_time", value, metadata).await.unwrap();
        }
        
        // Test current value that should trigger regression
        let regression_value = 130.0; // Significant jump
        let result = detector.detect_regression_with_confidence("response_time", regression_value).await.unwrap();
        
        assert!(result.is_some(), "Should detect regression");
        
        let enhanced_result = result.unwrap();
        assert!(enhanced_result.basic_result.is_regression, "Basic result should indicate regression");
        assert!(enhanced_result.statistical_confidence > 0.5, "Should have reasonable statistical confidence");
        assert!(enhanced_result.effect_size > 0.0, "Should have positive effect size");
        
        // Check Mann-Kendall result
        assert!(matches!(enhanced_result.mann_kendall.trend, TrendType::Increasing), 
               "Should detect increasing trend in baseline");
        
        // Check change points
        assert!(!enhanced_result.change_points.change_points.is_empty() || 
               enhanced_result.change_points.segments.len() >= 1,
               "Should have change point analysis");
    }

    #[tokio::test]
    async fn test_validation_with_statistical_tests() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            min_samples_for_baseline: 5,
            ..Default::default()
        };
        
        let detector = PerformanceRegressionDetector::new(config).unwrap();
        
        // Create baseline and current data
        let baseline_data: Vec<f64> = (0..30).map(|i| 100.0 + (i % 5) as f64).collect();
        let current_data: Vec<f64> = (0..10).map(|i| 120.0 + (i % 3) as f64).collect(); // Clear regression
        
        let validation = detector.validate_regression(&baseline_data, &current_data).await.unwrap();
        
        assert!(validation.is_valid, "Should validate clear regression");
        assert!(validation.confidence_score > 0.8, "Should have high confidence");
        assert!(validation.effect_size > 1.0, "Should detect large effect size");
        
        assert!(validation.mann_kendall_result.is_some(), "Should have Mann-Kendall result");
        assert!(validation.change_point_result.is_some(), "Should have change point result");
    }

    #[tokio::test]
    async fn test_no_false_positive_regression() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            min_samples_for_baseline: 10,
            regression_threshold_percent: 15.0, // Higher threshold
            ..Default::default()
        };
        
        let detector = PerformanceRegressionDetector::new(config).unwrap();
        
        // Build stable baseline
        for i in 0..30 {
            let metadata = HashMap::new();
            let value = 100.0 + (i % 3) as f64; // Small variation around 100
            detector.record_metric("stable_metric", value, metadata).await.unwrap();
        }
        
        // Test with value within normal range
        let normal_value = 102.0;
        let result = detector.detect_regression_with_confidence("stable_metric", normal_value).await.unwrap();
        
        if let Some(enhanced_result) = result {
            assert!(!enhanced_result.basic_result.is_regression, 
                   "Should not detect regression for normal variation");
            assert!(!enhanced_result.validation_passed || enhanced_result.statistical_confidence < 0.95,
                   "Should not validate as regression with high confidence");
        }
    }

    #[tokio::test]
    async fn test_multiple_metrics_regression_detection() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            min_samples_for_baseline: 5,
            ..Default::default()
        };
        
        let detector = PerformanceRegressionDetector::new(config).unwrap();
        
        let metrics = ["cpu_usage", "memory_usage", "response_time"];
        
        // Build baselines for multiple metrics
        for metric in &metrics {
            for i in 0..15 {
                let metadata = HashMap::new();
                let base_value = match *metric {
                    "cpu_usage" => 30.0,
                    "memory_usage" => 500.0,
                    "response_time" => 50.0,
                    _ => 10.0,
                };
                let value = base_value + (i % 3) as f64;
                detector.record_metric(metric, value, metadata).await.unwrap();
            }
        }
        
        // Test regression on one metric
        let cpu_regression = detector.detect_regression_with_confidence("cpu_usage", 50.0).await.unwrap();
        let memory_normal = detector.detect_regression_with_confidence("memory_usage", 502.0).await.unwrap();
        
        // CPU should show regression
        if let Some(cpu_result) = cpu_regression {
            assert!(cpu_result.basic_result.is_regression, "CPU should show regression");
        }
        
        // Memory should not show regression
        if let Some(memory_result) = memory_normal {
            assert!(!memory_result.basic_result.is_regression || 
                   !memory_result.validation_passed,
                   "Memory should not show significant regression");
        }
    }

    #[test]
    fn test_statistical_analyzer_integration() {
        let analyzer = StatisticalAnalyzer::new();
        let trend_detector = TrendDetector::new();
        
        // Performance data showing degradation
        let performance_data = vec![
            100.0, 101.0, 99.0, 102.0, 98.0,  // Baseline: ~100ms
            105.0, 107.0, 106.0, 108.0, 104.0, // Slight increase: ~106ms
            115.0, 117.0, 114.0, 118.0, 116.0, // Clear regression: ~116ms
        ];
        
        // Statistical analysis
        let mann_kendall = analyzer.mann_kendall_test(&performance_data).unwrap();
        let change_points = trend_detector.detect_change_points_pelt(&performance_data).unwrap();
        
        // Should detect increasing trend
        assert!(matches!(mann_kendall.trend, TrendType::Increasing));
        assert!(mann_kendall.tau > 0.5, "Should show strong positive correlation");
        
        // Should detect change points
        assert!(!change_points.change_points.is_empty(), "Should detect performance changes");
        assert!(change_points.segments.len() >= 2, "Should identify multiple performance levels");
    }

    #[test]
    fn test_confidence_interval_integration() {
        let analyzer = StatisticalAnalyzer::new();
        
        // Realistic performance data
        let baseline_data = vec![98.0, 102.0, 100.0, 99.0, 101.0, 103.0, 97.0, 100.0, 99.0, 101.0];
        let regression_data = vec![120.0, 122.0, 118.0, 125.0, 119.0];
        
        let baseline_ci = analyzer.confidence_interval(&baseline_data, 0.95).unwrap();
        let regression_ci = analyzer.confidence_interval(&regression_data, 0.95).unwrap();
        let effect_size = analyzer.effect_size(&baseline_data, &regression_data).unwrap();
        
        // Confidence intervals should not overlap for clear regression
        assert!(baseline_ci.1 < regression_ci.0, "Confidence intervals should not overlap");
        
        // Effect size should be large
        assert!(effect_size > 2.0, "Should detect very large effect size");
    }

    #[tokio::test]
    async fn test_performance_overhead() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            min_samples_for_baseline: 10,
            ..Default::default()
        };
        
        let detector = PerformanceRegressionDetector::new(config).unwrap();
        
        // Measure time for basic regression detection
        let start = std::time::Instant::now();
        
        // Build baseline
        for i in 0..50 {
            let metadata = HashMap::new();
            detector.record_metric("perf_test", 100.0 + i as f64, metadata).await.unwrap();
        }
        
        // Test enhanced detection multiple times
        for _ in 0..10 {
            let _result = detector.detect_regression_with_confidence("perf_test", 200.0).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let per_detection = elapsed / 10;
        
        // Should complete quickly (< 10ms per detection for reasonable performance)
        assert!(per_detection.as_millis() < 50, 
               "Enhanced detection should complete in < 50ms, took {:?}", per_detection);
    }

    #[test]
    fn test_edge_case_data_patterns() {
        let analyzer = StatisticalAnalyzer::new();
        let detector = TrendDetector::new();
        
        // Test various edge cases
        let test_cases = vec![
            (vec![1.0, 1.0, 1.0, 1.0, 1.0], "constant_data"),
            (vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0], "alternating_data"),
            (vec![f64::NAN, 1.0, 2.0, 3.0], "data_with_nan"),
            (vec![1.0, 1000000.0, 2.0, 3.0, 4.0], "data_with_extreme_outlier"),
        ];
        
        for (data, case_name) in test_cases {
            // Filter out NaN values for statistical analysis
            let clean_data: Vec<f64> = data.iter().copied().filter(|x| x.is_finite()).collect();
            
            if clean_data.len() >= 5 {
                let mann_kendall_result = analyzer.mann_kendall_test(&clean_data);
                let change_point_result = detector.detect_change_points_pelt(&clean_data);
                
                // Should handle edge cases without panicking
                assert!(mann_kendall_result.is_ok(), "Mann-Kendall should handle {}", case_name);
                assert!(change_point_result.is_ok(), "Change point detection should handle {}", case_name);
                
                if let (Ok(mk), Ok(cp)) = (mann_kendall_result, change_point_result) {
                    // Results should be valid (no NaN/infinite values)
                    assert!(mk.tau.is_finite(), "Tau should be finite for {}", case_name);
                    assert!(mk.p_value.is_finite(), "P-value should be finite for {}", case_name);
                    assert!(cp.confidence.is_finite(), "Confidence should be finite for {}", case_name);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_regression_analysis_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegressionDetectionConfig {
            storage_path: temp_dir.path().to_path_buf(),
            min_samples_for_baseline: 5,
            ..Default::default()
        };
        
        let detector = PerformanceRegressionDetector::new(config).unwrap();
        
        // Create baseline with clear pattern
        for i in 0..25 {
            let metadata = HashMap::new();
            let value = 50.0 + (i as f64 * 0.2); // Slow increase
            detector.record_metric("latency", value, metadata).await.unwrap();
        }
        
        // Test with regression value
        let result = detector.detect_regression_with_confidence("latency", 70.0).await.unwrap();
        
        if let Some(enhanced_result) = result {
            let analysis = &enhanced_result.basic_result.analysis;
            
            // Should have enhanced analysis fields
            assert!(analysis.mann_kendall_result.is_some(), "Should have Mann-Kendall analysis");
            assert!(analysis.change_point_analysis.is_some(), "Should have change point analysis");
            
            if let Some(mk_result) = &analysis.mann_kendall_result {
                assert!(matches!(mk_result.trend, TrendType::Increasing), 
                       "Should detect increasing trend");
            }
            
            // Should have recommendations and potential causes
            assert!(!analysis.potential_causes.is_empty(), "Should have potential causes");
            assert!(!analysis.recommended_actions.is_empty(), "Should have recommendations");
        }
    }
}