//! Comprehensive tests for trend detection module

#[cfg(test)]
mod tests {
    use uveddi::performance::trend_detection::{TrendDetector, ChangePointResult, Segment};
    use uveddi::performance::statistical_analysis::TrendType;

    #[test]
    fn test_single_change_point_detection() {
        // Create data with clear change point at index 50
        let mut values = vec![5.0; 50];
        values.extend(vec![15.0; 50]); // Step change from 5 to 15
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        assert!(!result.change_points.is_empty(), "Should detect at least one change point");
        
        // Check that detected change point is near the actual change (index 50)
        let has_change_near_50 = result.change_points.iter()
            .any(|&cp| (cp as i32 - 50).abs() < 10);
        assert!(has_change_near_50, "Should detect change point near index 50");
        
        // Should have 2 segments (before and after change point)
        assert_eq!(result.segments.len(), result.change_points.len() + 1);
        
        // Check segment statistics
        for segment in &result.segments {
            assert!(segment.start_index < segment.end_index, "Segment indices should be valid");
            assert!(segment.variance >= 0.0, "Variance should be non-negative");
        }
    }

    #[test]
    fn test_multiple_change_points_detection() {
        // Create data with multiple distinct levels
        let mut values = vec![10.0; 25]; // Level 1
        values.extend(vec![20.0; 25]);   // Level 2
        values.extend(vec![5.0; 25]);    // Level 3
        values.extend(vec![30.0; 25]);   // Level 4
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should detect 2-3 change points (allowing for algorithm differences)
        assert!(result.change_points.len() >= 2, "Should detect multiple change points");
        assert!(result.change_points.len() <= 4, "Should not detect too many change points");
        
        // Change points should be in order
        for i in 1..result.change_points.len() {
            assert!(result.change_points[i] > result.change_points[i-1], 
                   "Change points should be in ascending order");
        }
        
        // Should have segments equal to change points + 1
        assert_eq!(result.segments.len(), result.change_points.len() + 1);
    }

    #[test]
    fn test_no_change_points_stable_data() {
        let values = vec![10.0; 100]; // Perfectly stable data
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should detect very few or no change points
        assert!(result.change_points.len() <= 1, "Stable data should have minimal change points");
        
        // Should have high confidence for stable data
        assert!(result.confidence > 0.5, "Confidence should be reasonable for stable data");
        
        // Should have at least one segment
        assert!(!result.segments.is_empty(), "Should have at least one segment");
        
        // Single segment should cover entire range
        if result.segments.len() == 1 {
            assert_eq!(result.segments[0].start_index, 0);
            assert_eq!(result.segments[0].end_index, values.len());
        }
    }

    #[test]
    fn test_gradual_trend_vs_abrupt_change() {
        // Gradual increasing trend
        let gradual_data: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
        
        // Abrupt step change
        let mut abrupt_data = vec![0.0; 50];
        abrupt_data.extend(vec![10.0; 50]);
        
        let detector = TrendDetector::new();
        
        let gradual_result = detector.detect_change_points_pelt(&gradual_data).unwrap();
        let abrupt_result = detector.detect_change_points_pelt(&abrupt_data).unwrap();
        
        // Abrupt change should be detected more clearly
        assert!(abrupt_result.confidence >= gradual_result.confidence,
               "Abrupt changes should have higher or equal confidence");
        
        // Abrupt change should have fewer change points (cleaner signal)
        assert!(abrupt_result.change_points.len() <= gradual_result.change_points.len() + 2,
               "Abrupt change should not have many more change points than gradual");
    }

    #[test]
    fn test_binary_segmentation_algorithm() {
        let mut values = vec![5.0; 40];
        values.extend(vec![15.0; 40]);
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_binary(&values).unwrap();
        
        assert!(result.algorithm_used.contains("Binary Segmentation"));
        assert!(!result.change_points.is_empty(), "Should detect change point");
        assert!(result.confidence > 0.0, "Should have positive confidence");
    }

    #[test]
    fn test_insufficient_data_handling() {
        // Very small dataset
        let values = vec![1.0, 2.0, 3.0, 4.0];
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should handle gracefully without crashing
        assert_eq!(result.change_points.len(), 0, "Small dataset should have no change points");
        assert_eq!(result.segments.len(), 1, "Should have exactly one segment");
        assert_eq!(result.segments[0].start_index, 0);
        assert_eq!(result.segments[0].end_index, values.len());
    }

    #[test]
    fn test_segment_trend_analysis() {
        // Create data with different trend patterns
        let mut values = Vec::new();
        
        // Increasing segment
        values.extend((0..20).map(|i| i as f64));
        
        // Decreasing segment  
        values.extend((0..20).rev().map(|i| i as f64));
        
        // Stable segment
        values.extend(vec![10.0; 20]);
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should detect multiple segments with different trends
        assert!(result.segments.len() >= 2, "Should detect multiple segments");
        
        // Check that we have different trend types
        let trend_types: std::collections::HashSet<_> = result.segments
            .iter()
            .map(|s| &s.trend)
            .collect();
        
        // Should have at least 2 different trend types
        assert!(trend_types.len() >= 2, "Should detect different trend types in segments");
    }

    #[test]
    fn test_confidence_calculation() {
        // High confidence scenario: clear step change
        let mut clear_change = vec![0.0; 50];
        clear_change.extend(vec![100.0; 50]);
        
        // Low confidence scenario: noisy data
        let noisy_data: Vec<f64> = (0..100).map(|i| {
            let base = if i < 50 { 10.0 } else { 11.0 }; // Small change
            let noise = (i as f64 * 0.1).sin() * 5.0;   // High noise
            base + noise
        }).collect();
        
        let detector = TrendDetector::new();
        
        let clear_result = detector.detect_change_points_pelt(&clear_change).unwrap();
        let noisy_result = detector.detect_change_points_pelt(&noisy_data).unwrap();
        
        // Clear change should have higher confidence
        assert!(clear_result.confidence >= noisy_result.confidence,
               "Clear changes should have higher confidence than noisy data");
    }

    #[test]
    fn test_custom_parameters() {
        let values = vec![1.0; 100];
        
        // Test with custom parameters
        let detector = TrendDetector::new()
            .with_min_segment_length(15)
            .with_penalty_factor(5.0)
            .with_max_change_points(3);
        
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should respect max_change_points parameter
        assert!(result.change_points.len() <= 3, "Should respect max change points limit");
        
        // Should handle custom parameters without error
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0, 
               "Confidence should be valid range");
    }

    #[test]
    fn test_algorithm_comparison() {
        // Data with clear change point
        let mut values = vec![10.0; 30];
        values.extend(vec![20.0; 30]);
        
        let detector = TrendDetector::new();
        
        let pelt_result = detector.detect_change_points_pelt(&values).unwrap();
        let binary_result = detector.detect_change_points_binary(&values).unwrap();
        
        // Both algorithms should detect the change point
        assert!(!pelt_result.change_points.is_empty(), "PELT should detect change point");
        assert!(!binary_result.change_points.is_empty(), "Binary segmentation should detect change point");
        
        // Results should be reasonably similar (within 10 indices)
        if !pelt_result.change_points.is_empty() && !binary_result.change_points.is_empty() {
            let pelt_first = pelt_result.change_points[0];
            let binary_first = binary_result.change_points[0];
            let difference = (pelt_first as i32 - binary_first as i32).abs();
            
            assert!(difference < 15, "Algorithms should give similar results");
        }
    }

    #[test]
    fn test_variance_calculation_in_segments() {
        // Create data with different variance levels
        let mut values = Vec::new();
        
        // Low variance segment
        for _ in 0..30 {
            values.push(10.0); // Constant values = zero variance
        }
        
        // High variance segment  
        for i in 0..30 {
            values.push(20.0 + (i as f64 % 5.0) * 4.0); // High variability
        }
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should have multiple segments
        assert!(result.segments.len() >= 1, "Should have at least one segment");
        
        // Check variance calculations are reasonable
        for segment in &result.segments {
            assert!(segment.variance >= 0.0, "Variance should be non-negative");
            assert!(!segment.variance.is_nan(), "Variance should not be NaN");
            assert!(segment.variance.is_finite(), "Variance should be finite");
        }
    }

    #[test]
    fn test_empty_data_handling() {
        let values = vec![];
        let detector = TrendDetector::new();
        
        let result = detector.detect_change_points_pelt(&values);
        
        // Should handle empty data gracefully
        assert!(result.is_ok(), "Should handle empty data without error");
        
        if let Ok(result) = result {
            assert!(result.change_points.is_empty(), "Empty data should have no change points");
            assert!(result.segments.is_empty() || result.segments.len() == 1, 
                   "Empty data should have no segments or one empty segment");
        }
    }

    #[test]
    fn test_performance_regression_scenario() {
        // Simulate performance regression: stable baseline, then degradation
        let mut values = Vec::new();
        
        // Stable performance phase (50 samples around 100ms)
        for i in 0..50 {
            values.push(100.0 + (i % 3) as f64); // 100-102ms
        }
        
        // Performance degradation phase (30 samples around 150ms)
        for i in 0..30 {
            values.push(150.0 + (i % 4) as f64); // 150-153ms
        }
        
        // Recovered performance phase (20 samples around 105ms)
        for i in 0..20 {
            values.push(105.0 + (i % 2) as f64); // 105-106ms
        }
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should detect 2 change points (degradation and recovery)
        assert!(result.change_points.len() >= 1, "Should detect performance regression");
        assert!(result.change_points.len() <= 3, "Should not over-segment");
        
        // Segments should have different mean values corresponding to performance levels
        let means: Vec<f64> = result.segments.iter().map(|s| s.mean_value).collect();
        
        // Should have variation in means indicating the performance changes
        let max_mean = means.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_mean = means.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        assert!(max_mean - min_mean > 20.0, "Should detect significant performance difference");
    }

    #[test]
    fn test_seasonal_pattern_handling() {
        // Create data with seasonal pattern plus trend
        let mut values = Vec::new();
        for i in 0..100 {
            let seasonal = 5.0 * (2.0 * std::f64::consts::PI * i as f64 / 20.0).sin();
            let trend = if i < 50 { 10.0 } else { 15.0 }; // Step change at midpoint
            values.push(trend + seasonal);
        }
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should detect the step change despite seasonal variation
        assert!(!result.change_points.is_empty(), "Should detect step change despite seasonality");
        
        // Change point should be around index 50
        let has_midpoint_change = result.change_points.iter()
            .any(|&cp| (cp as i32 - 50).abs() < 15);
        assert!(has_midpoint_change, "Should detect change point near midpoint");
    }

    #[test]
    fn test_outlier_resilience() {
        // Create data with outliers
        let mut values = vec![10.0; 40];
        values[20] = 100.0; // Single outlier
        values.extend(vec![20.0; 40]); // Level change
        values[60] = -50.0; // Another outlier
        
        let detector = TrendDetector::new();
        let result = detector.detect_change_points_pelt(&values).unwrap();
        
        // Should detect the level change but not be overly influenced by outliers
        assert!(!result.change_points.is_empty(), "Should detect level change");
        
        // Should not detect too many spurious change points due to outliers
        assert!(result.change_points.len() <= 4, "Should not over-segment due to outliers");
    }
}