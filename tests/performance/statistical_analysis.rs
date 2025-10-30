//! Comprehensive tests for statistical analysis module

#[cfg(test)]
mod tests {
    use uveddi::performance::statistical_analysis::{StatisticalAnalyzer, TrendType, MannKendallResult};
    use std::f64::consts::PI;

    #[test]
    fn test_mann_kendall_strong_increasing_trend() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Increasing));
        assert!(result.p_value < 0.001, "P-value should be very small for strong trend");
        assert!(result.confidence > 0.99, "Confidence should be very high");
        assert!(result.tau > 0.9, "Tau should be close to 1 for perfect increasing trend");
        assert!(result.effect_size > 0.9, "Effect size should be large");
    }

    #[test]
    fn test_mann_kendall_strong_decreasing_trend() {
        let values = vec![10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Decreasing));
        assert!(result.p_value < 0.001, "P-value should be very small for strong trend");
        assert!(result.confidence > 0.99, "Confidence should be very high");
        assert!(result.tau < -0.9, "Tau should be close to -1 for perfect decreasing trend");
        assert!(result.effect_size > 0.9, "Effect size should be large");
    }

    #[test]
    fn test_mann_kendall_weak_increasing_trend() {
        let values = vec![1.0, 1.5, 2.0, 1.8, 2.5, 2.2, 3.0, 2.8, 3.5, 3.2];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        // Should detect increasing trend but with less confidence
        assert!(result.tau > 0.0, "Tau should be positive for increasing trend");
        // P-value might be higher for weaker trend
        assert!(result.confidence >= 0.0);
    }

    #[test]
    fn test_mann_kendall_no_trend_constant() {
        let values = vec![5.0; 20]; // Perfectly constant data
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::NoTrend));
        assert!(result.tau == 0.0, "Tau should be 0 for constant data");
        assert!(result.p_value > 0.05, "P-value should be high for no trend");
    }

    #[test]
    fn test_mann_kendall_no_trend_random() {
        let values = vec![5.1, 4.9, 5.0, 5.2, 4.8, 5.0, 5.1, 4.9, 5.0, 5.05];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::NoTrend));
        assert!(result.tau.abs() < 0.5, "Tau should be small for random data");
        assert!(result.p_value > 0.05, "P-value should be high for no trend");
    }

    #[test]
    fn test_mann_kendall_with_ties() {
        let values = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        // Should still detect increasing trend despite ties
        assert!(matches!(result.trend, TrendType::Increasing));
        assert!(result.tau > 0.0, "Tau should be positive despite ties");
    }

    #[test]
    fn test_mann_kendall_insufficient_data() {
        let values = vec![1.0, 2.0, 3.0]; // Less than minimum required
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Uncertain));
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.p_value, 1.0);
    }

    #[test]
    fn test_mann_kendall_empty_data() {
        let values = vec![];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Uncertain));
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.p_value, 1.0);
    }

    #[test]
    fn test_confidence_interval_calculation() {
        let values = vec![10.0, 12.0, 11.0, 13.0, 9.0, 14.0, 10.5, 11.5, 12.5, 10.8];
        let analyzer = StatisticalAnalyzer::new();
        
        // Test 95% confidence interval
        let (lower, upper) = analyzer.confidence_interval(&values, 0.95).unwrap();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        assert!(lower < mean, "Lower bound should be less than mean");
        assert!(upper > mean, "Upper bound should be greater than mean");
        assert!(upper - lower > 0.0, "Interval should have positive width");
        
        // Test 99% confidence interval (should be wider)
        let (lower_99, upper_99) = analyzer.confidence_interval(&values, 0.99).unwrap();
        assert!(lower_99 <= lower, "99% CI lower bound should be <= 95% CI lower bound");
        assert!(upper_99 >= upper, "99% CI upper bound should be >= 95% CI upper bound");
    }

    #[test]
    fn test_confidence_interval_single_value() {
        let values = vec![5.0];
        let analyzer = StatisticalAnalyzer::new();
        let (lower, upper) = analyzer.confidence_interval(&values, 0.95).unwrap();
        
        assert_eq!(lower, 5.0);
        assert_eq!(upper, 5.0);
    }

    #[test]
    fn test_confidence_interval_two_values() {
        let values = vec![4.0, 6.0];
        let analyzer = StatisticalAnalyzer::new();
        let (lower, upper) = analyzer.confidence_interval(&values, 0.95).unwrap();
        
        let mean = 5.0;
        assert!(lower < mean);
        assert!(upper > mean);
    }

    #[test]
    fn test_effect_size_large_effect() {
        let baseline = vec![10.0, 11.0, 9.0, 10.5, 9.5]; // Mean ≈ 10
        let current = vec![15.0, 16.0, 14.0, 15.5, 14.5]; // Mean ≈ 15, shift of ~5
        
        let analyzer = StatisticalAnalyzer::new();
        let effect_size = analyzer.effect_size(&baseline, &current).unwrap();
        
        assert!(effect_size > 2.0, "Should detect large effect size for 5-point shift");
    }

    #[test]
    fn test_effect_size_medium_effect() {
        let baseline = vec![10.0, 11.0, 9.0, 10.5, 9.5]; // Mean ≈ 10
        let current = vec![11.0, 12.0, 10.0, 11.5, 10.5]; // Mean ≈ 11, shift of ~1
        
        let analyzer = StatisticalAnalyzer::new();
        let effect_size = analyzer.effect_size(&baseline, &current).unwrap();
        
        assert!(effect_size > 0.0, "Should detect positive effect size");
        assert!(effect_size < 2.0, "Effect size should be moderate");
    }

    #[test]
    fn test_effect_size_small_effect() {
        let baseline = vec![10.0, 10.1, 9.9, 10.05, 9.95];
        let current = vec![10.1, 10.2, 10.0, 10.15, 10.05]; // Very small shift
        
        let analyzer = StatisticalAnalyzer::new();
        let effect_size = analyzer.effect_size(&baseline, &current).unwrap();
        
        assert!(effect_size >= 0.0, "Effect size should be non-negative");
        assert!(effect_size < 1.0, "Effect size should be small");
    }

    #[test]
    fn test_effect_size_identical_groups() {
        let baseline = vec![10.0, 11.0, 9.0, 10.5, 9.5];
        let current = baseline.clone();
        
        let analyzer = StatisticalAnalyzer::new();
        let effect_size = analyzer.effect_size(&baseline, &current).unwrap();
        
        assert!((effect_size).abs() < 0.001, "Effect size should be near zero for identical groups");
    }

    #[test]
    fn test_effect_size_empty_data() {
        let baseline = vec![];
        let current = vec![1.0, 2.0, 3.0];
        
        let analyzer = StatisticalAnalyzer::new();
        let effect_size = analyzer.effect_size(&baseline, &current).unwrap();
        
        assert_eq!(effect_size, 0.0, "Effect size should be 0 for empty baseline");
    }

    #[test]
    fn test_analyzer_configuration() {
        let analyzer = StatisticalAnalyzer::new()
            .with_significance_threshold(0.01)  // 99% confidence
            .with_min_samples(20);
        
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // Less than 20 samples
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        // Should return uncertain due to insufficient samples
        assert!(matches!(result.trend, TrendType::Uncertain));
    }

    #[test]
    fn test_performance_regression_realistic_scenario() {
        // Simulate realistic performance data: baseline around 100ms, then degradation
        let mut baseline_data = vec![];
        let mut current_data = vec![];
        
        // Generate baseline: normal distribution around 100ms
        for i in 0..30 {
            baseline_data.push(100.0 + (i as f64 % 5.0) * 2.0); // 100-110ms range
        }
        
        // Generate current: degraded performance around 120ms
        for i in 0..10 {
            current_data.push(120.0 + (i as f64 % 3.0) * 3.0); // 120-126ms range
        }
        
        let analyzer = StatisticalAnalyzer::new();
        let effect_size = analyzer.effect_size(&baseline_data, &current_data).unwrap();
        
        // Should detect significant performance regression
        assert!(effect_size > 1.0, "Should detect large effect size for 20ms degradation");
    }

    #[test]
    fn test_seasonal_data_handling() {
        // Create data with seasonal pattern but overall increasing trend
        let mut values = vec![];
        for i in 0..50 {
            let seasonal = 2.0 * (2.0 * PI * i as f64 / 10.0).sin(); // 10-period sine wave
            let trend = i as f64 * 0.1; // Linear increase
            values.push(10.0 + trend + seasonal);
        }
        
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        // Should detect increasing trend despite seasonal variation
        assert!(result.tau > 0.0, "Should detect positive trend despite seasonality");
    }

    #[test]
    fn test_outlier_resilience() {
        // Create mostly stable data with some outliers
        let mut values = vec![5.0; 15]; // Stable baseline
        values.push(50.0); // Large outlier
        values.extend(vec![5.0; 5]); // Return to baseline
        
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        // Mann-Kendall should be resilient to outliers
        assert!(matches!(result.trend, TrendType::NoTrend));
        assert!(result.tau.abs() < 0.5, "Tau should be small despite outlier");
    }

    #[test]
    fn test_statistical_power_with_varying_sample_sizes() {
        // Test how statistical power changes with sample size
        let create_increasing_data = |n: usize| -> Vec<f64> {
            (0..n).map(|i| i as f64).collect()
        };
        
        let analyzer = StatisticalAnalyzer::new();
        
        // Small sample
        let small_data = create_increasing_data(10);
        let small_result = analyzer.mann_kendall_test(&small_data).unwrap();
        
        // Large sample
        let large_data = create_increasing_data(50);
        let large_result = analyzer.mann_kendall_test(&large_data).unwrap();
        
        // Larger sample should have higher confidence (lower p-value)
        assert!(large_result.p_value <= small_result.p_value, 
               "Larger sample should have equal or lower p-value");
        assert!(large_result.confidence >= small_result.confidence, 
               "Larger sample should have equal or higher confidence");
    }
}