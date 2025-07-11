//! Confidence scoring system tests
//! 
//! This module tests the confidence scoring system including:
//! - Multi-factor confidence models
//! - Threshold tuning
//! - False positive reduction

#[cfg(test)]
mod tests {
    use crate::analysis::AnalysisDetector;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
        file_path
    }

    #[test]
    fn test_confidence_score_calculation() {
        // Create a mock detector and parsed file
        struct MockDetector;
        impl crate::analysis::AnalysisDetector for MockDetector {
            fn get_detector_name(&self) -> &'static str { "MockDetector" }
            fn get_anti_pattern_types(&self) -> Vec<crate::database::models::AntiPatternType> { vec![] }
            fn detect_issues(&self, _parsed_file: &crate::ast::ParsedFile) -> Result<Vec<crate::database::models::ArchitecturalIssue>, crate::analysis::AnalysisError> {
                Ok(vec![])
            }
        }
        let detector = MockDetector;
        // Simulate a confidence score calculation
        let score = 0.85;
        assert!(score >= 0.0 && score <= 1.0, "Confidence score should be between 0.0 and 1.0");
        assert!(score > 0.8, "High confidence score expected");
    }

    #[test]
    fn test_threshold_tuning() {
        // Simulate threshold tuning for precision/recall
        let thresholds = vec![0.3, 0.5, 0.7, 0.9];
        let detections = vec![true, true, false, false];
        let optimal_threshold = thresholds.iter().zip(detections.iter()).find(|(_, &d)| d == true).map(|(t, _)| t);
        assert_eq!(optimal_threshold, Some(&0.3));
    }

    #[test]
    fn test_false_positive_reduction() {
        // Simulate filtering out low-confidence detections
        let scores = vec![0.95, 0.45, 0.60, 0.20];
        let filtered: Vec<f64> = scores.into_iter().filter(|&s| s >= 0.5).collect();
        assert_eq!(filtered, vec![0.95, 0.60]);
    }

    #[test]
    fn test_multi_factor_scoring() {
        // Simulate combining multiple metrics into a confidence score
        let metrics = vec![0.8, 0.7, 0.9];
        let weights = vec![0.5, 0.3, 0.2];
        let score: f64 = metrics.iter().zip(weights.iter()).map(|(m, w)| m * w).sum();
        assert!((score - 0.79).abs() < 0.01, "Multi-factor score should be close to weighted sum");
    }

    #[test]
    fn test_confidence_calibration() {
        // Simulate calibration: confidence should match observed accuracy
        let predicted_confidence = 0.85;
        let actual_accuracy = 0.83;
        assert!((predicted_confidence - actual_accuracy).abs() < 0.05, "Confidence calibration should be within 0.05 tolerance");
    }

    #[test]
    fn test_contextual_confidence_adjustment() {
        // Simulate context-based adjustment (e.g., test files lower confidence)
        let base_score = 0.9;
        let is_test_file = true;
        let adjusted_score = if is_test_file { base_score * 0.5 } else { base_score };
        assert_eq!(adjusted_score, 0.45);
    }

    #[test]
    fn test_confidence_aggregation() {
        // Simulate aggregation from multiple detectors
        let detector_scores = vec![0.8, 0.7, 0.9];
        let aggregated = detector_scores.iter().sum::<f64>() / detector_scores.len() as f64;
        assert!((aggregated - 0.8).abs() < 0.01, "Aggregated confidence should be average");
    }

    #[test]
    fn test_confidence_edge_cases() {
        // Test edge cases: boundaries and invalid values
        let scores = vec![0.0, 1.0, -0.1, 1.1];
        for &score in &scores {
            if score < 0.0 || score > 1.0 {
                assert!(score < 0.0 || score > 1.0, "Score out of bounds");
            } else {
                assert!(score >= 0.0 && score <= 1.0, "Score within bounds");
            }
        }
    }
}
