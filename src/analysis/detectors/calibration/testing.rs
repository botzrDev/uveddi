//! Testing infrastructure for threshold calibration and validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Metrics for evaluating detector performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationMetrics {
    /// True Positives
    pub true_positives: usize,
    /// False Positives
    pub false_positives: usize,
    /// True Negatives
    pub true_negatives: usize,
    /// False Negatives
    pub false_negatives: usize,
    /// Total issues analyzed
    pub total_issues: usize,
    /// Analysis duration
    pub duration: Duration,
}

impl CalibrationMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            true_positives: 0,
            false_positives: 0,
            true_negatives: 0,
            false_negatives: 0,
            total_issues: 0,
            duration: Duration::from_secs(0),
        }
    }

    /// Calculate precision: TP / (TP + FP)
    pub fn precision(&self) -> f64 {
        if self.true_positives + self.false_positives == 0 {
            return 0.0;
        }
        self.true_positives as f64 / (self.true_positives + self.false_positives) as f64
    }

    /// Calculate recall: TP / (TP + FN)
    pub fn recall(&self) -> f64 {
        if self.true_positives + self.false_negatives == 0 {
            return 0.0;
        }
        self.true_positives as f64 / (self.true_positives + self.false_negatives) as f64
    }

    /// Calculate F1 score: 2 * (Precision * Recall) / (Precision + Recall)
    pub fn f1_score(&self) -> f64 {
        let precision = self.precision();
        let recall = self.recall();
        if precision + recall == 0.0 {
            return 0.0;
        }
        2.0 * (precision * recall) / (precision + recall)
    }

    /// Calculate false positive rate: FP / (FP + TN)
    pub fn false_positive_rate(&self) -> f64 {
        if self.false_positives + self.true_negatives == 0 {
            return 0.0;
        }
        self.false_positives as f64 / (self.false_positives + self.true_negatives) as f64
    }

    /// Calculate false negative rate: FN / (FN + TP)
    pub fn false_negative_rate(&self) -> f64 {
        if self.false_negatives + self.true_positives == 0 {
            return 0.0;
        }
        self.false_negatives as f64 / (self.false_negatives + self.true_positives) as f64
    }

    /// Calculate accuracy: (TP + TN) / (TP + TN + FP + FN)
    pub fn accuracy(&self) -> f64 {
        let total = self.true_positives
            + self.true_negatives
            + self.false_positives
            + self.false_negatives;
        if total == 0 {
            return 0.0;
        }
        (self.true_positives + self.true_negatives) as f64 / total as f64
    }

    /// Check if metrics meet target thresholds
    pub fn meets_targets(&self, target_precision: f64, target_recall: f64) -> bool {
        self.precision() >= target_precision && self.recall() >= target_recall
    }

    /// Generate summary report
    pub fn summary(&self) -> String {
        format!(
            "Metrics Summary:\n\
             - Precision: {:.2}%\n\
             - Recall: {:.2}%\n\
             - F1 Score: {:.2}\n\
             - False Positive Rate: {:.2}%\n\
             - Accuracy: {:.2}%\n\
             - True Positives: {}\n\
             - False Positives: {}\n\
             - True Negatives: {}\n\
             - False Negatives: {}\n\
             - Duration: {:?}",
            self.precision() * 100.0,
            self.recall() * 100.0,
            self.f1_score(),
            self.false_positive_rate() * 100.0,
            self.accuracy() * 100.0,
            self.true_positives,
            self.false_positives,
            self.true_negatives,
            self.false_negatives,
            self.duration
        )
    }
}

impl Default for CalibrationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Ground truth annotation for a code issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthAnnotation {
    /// File path
    pub file_path: String,
    /// Issue type
    pub issue_type: String,
    /// Line number
    pub line: u32,
    /// Severity (0-100)
    pub severity: u32,
    /// Whether this is confirmed as a true issue
    pub confirmed: bool,
    /// Annotator notes
    pub notes: String,
}

/// Ground truth dataset for calibration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthDataset {
    /// Dataset name
    pub name: String,
    /// Language
    pub language: String,
    /// Annotations
    pub annotations: Vec<GroundTruthAnnotation>,
}

impl GroundTruthDataset {
    /// Create a new dataset
    pub fn new(name: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            language: language.into(),
            annotations: Vec::new(),
        }
    }

    /// Add an annotation
    pub fn add_annotation(&mut self, annotation: GroundTruthAnnotation) {
        self.annotations.push(annotation);
    }

    /// Get annotations for a specific file
    pub fn get_file_annotations(&self, file_path: &str) -> Vec<&GroundTruthAnnotation> {
        self.annotations
            .iter()
            .filter(|a| a.file_path == file_path)
            .collect()
    }

    /// Count true positives in the dataset
    pub fn count_true_issues(&self) -> usize {
        self.annotations.iter().filter(|a| a.confirmed).count()
    }

    /// Count false positives in the dataset
    pub fn count_false_issues(&self) -> usize {
        self.annotations.iter().filter(|a| !a.confirmed).count()
    }
}

/// Configuration for threshold optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Minimum precision target
    pub min_precision: f64,
    /// Minimum recall target
    pub min_recall: f64,
    /// Weight for false positives (0.0-1.0)
    pub fp_weight: f64,
    /// Weight for false negatives (0.0-1.0)
    pub fn_weight: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Convergence threshold
    pub convergence_threshold: f64,
}

impl OptimizationConfig {
    /// Create configuration balanced between precision and recall
    pub fn balanced() -> Self {
        Self {
            min_precision: 0.80,
            min_recall: 0.85,
            fp_weight: 0.5,
            fn_weight: 0.5,
            max_iterations: 100,
            convergence_threshold: 0.01,
        }
    }

    /// Create configuration optimizing for precision (fewer false positives)
    pub fn precision_focused() -> Self {
        Self {
            min_precision: 0.90,
            min_recall: 0.70,
            fp_weight: 0.7,
            fn_weight: 0.3,
            max_iterations: 100,
            convergence_threshold: 0.01,
        }
    }

    /// Create configuration optimizing for recall (catch more issues)
    pub fn recall_focused() -> Self {
        Self {
            min_precision: 0.70,
            min_recall: 0.90,
            fp_weight: 0.3,
            fn_weight: 0.7,
            max_iterations: 100,
            convergence_threshold: 0.01,
        }
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Threshold optimizer using hill climbing
#[derive(Debug, Clone)]
pub struct ThresholdOptimizer {
    /// Optimization configuration
    pub config: OptimizationConfig,
}

impl ThresholdOptimizer {
    /// Create a new optimizer
    pub fn new(config: OptimizationConfig) -> Self {
        Self { config }
    }

    /// Calculate objective function value (lower is better)
    pub fn objective_function(&self, metrics: &CalibrationMetrics) -> f64 {
        let fp_rate = metrics.false_positive_rate();
        let fn_rate = metrics.false_negative_rate();

        self.config.fp_weight * fp_rate + self.config.fn_weight * fn_rate
    }

    /// Suggest threshold adjustment based on metrics
    pub fn suggest_adjustment(&self, metrics: &CalibrationMetrics) -> ThresholdAdjustment {
        let precision = metrics.precision();
        let recall = metrics.recall();

        // If precision is too low, increase threshold (stricter)
        if precision < self.config.min_precision {
            return ThresholdAdjustment {
                direction: AdjustmentDirection::Increase,
                magnitude: self.calculate_magnitude(
                    self.config.min_precision - precision,
                ),
                reason: format!(
                    "Precision {:.2}% below target {:.2}%",
                    precision * 100.0,
                    self.config.min_precision * 100.0
                ),
            };
        }

        // If recall is too low, decrease threshold (more lenient)
        if recall < self.config.min_recall {
            return ThresholdAdjustment {
                direction: AdjustmentDirection::Decrease,
                magnitude: self.calculate_magnitude(self.config.min_recall - recall),
                reason: format!(
                    "Recall {:.2}% below target {:.2}%",
                    recall * 100.0,
                    self.config.min_recall * 100.0
                ),
            };
        }

        // Metrics meet targets, no adjustment needed
        ThresholdAdjustment {
            direction: AdjustmentDirection::None,
            magnitude: 0.0,
            reason: "Metrics meet targets".to_string(),
        }
    }

    /// Calculate adjustment magnitude based on gap
    fn calculate_magnitude(&self, gap: f64) -> f64 {
        // Conservative adjustment: 10-20% change
        if gap > 0.2 {
            0.2 // Large gap: 20% adjustment
        } else if gap > 0.1 {
            0.15 // Medium gap: 15% adjustment
        } else {
            0.1 // Small gap: 10% adjustment
        }
    }

    /// Run optimization loop (placeholder for actual implementation)
    pub fn optimize(
        &self,
        _initial_threshold: f64,
        _ground_truth: &GroundTruthDataset,
    ) -> OptimizationResult {
        // This would run the actual optimization algorithm
        // For now, return a placeholder result
        OptimizationResult {
            optimal_threshold: 0.7,
            metrics: CalibrationMetrics::default(),
            iterations: 0,
            converged: true,
        }
    }
}

/// Suggested threshold adjustment
#[derive(Debug, Clone)]
pub struct ThresholdAdjustment {
    /// Direction of adjustment
    pub direction: AdjustmentDirection,
    /// Magnitude of adjustment (0.0-1.0)
    pub magnitude: f64,
    /// Reason for adjustment
    pub reason: String,
}

impl ThresholdAdjustment {
    /// Apply adjustment to a threshold
    pub fn apply(&self, current_threshold: f64) -> f64 {
        match self.direction {
            AdjustmentDirection::Increase => {
                (current_threshold * (1.0 + self.magnitude)).min(1.0)
            }
            AdjustmentDirection::Decrease => {
                (current_threshold * (1.0 - self.magnitude)).max(0.0)
            }
            AdjustmentDirection::None => current_threshold,
        }
    }

    /// Apply adjustment to an integer threshold
    pub fn apply_int(&self, current_threshold: u32) -> u32 {
        let adjusted = self.apply(current_threshold as f64);
        adjusted as u32
    }
}

/// Direction of threshold adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentDirection {
    Increase,
    Decrease,
    None,
}

/// Result of threshold optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Optimal threshold found
    pub optimal_threshold: f64,
    /// Metrics at optimal threshold
    pub metrics: CalibrationMetrics,
    /// Number of iterations
    pub iterations: usize,
    /// Whether optimization converged
    pub converged: bool,
}

/// A/B test configuration
#[derive(Debug, Clone)]
pub struct ABTest {
    /// Test name
    pub name: String,
    /// Control threshold
    pub control_threshold: f64,
    /// Variant threshold
    pub variant_threshold: f64,
    /// Split ratio (0.0-1.0, default 0.5 for 50/50)
    pub split_ratio: f64,
}

impl ABTest {
    /// Create a new A/B test
    pub fn new(
        name: impl Into<String>,
        control_threshold: f64,
        variant_threshold: f64,
    ) -> Self {
        Self {
            name: name.into(),
            control_threshold,
            variant_threshold,
            split_ratio: 0.5,
        }
    }

    /// Run comparison (placeholder)
    pub fn run_comparison(&self) -> ABTestResult {
        ABTestResult {
            control_metrics: CalibrationMetrics::default(),
            variant_metrics: CalibrationMetrics::default(),
            statistical_significance: false,
            p_value: 0.5,
            recommendation: "Insufficient data".to_string(),
        }
    }
}

/// A/B test result
#[derive(Debug, Clone)]
pub struct ABTestResult {
    /// Control group metrics
    pub control_metrics: CalibrationMetrics,
    /// Variant group metrics
    pub variant_metrics: CalibrationMetrics,
    /// Whether results are statistically significant
    pub statistical_significance: bool,
    /// P-value
    pub p_value: f64,
    /// Recommendation
    pub recommendation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_metrics() {
        let mut metrics = CalibrationMetrics::new();
        metrics.true_positives = 80;
        metrics.false_positives = 20;
        metrics.true_negatives = 70;
        metrics.false_negatives = 30;

        assert_eq!(metrics.precision(), 0.8); // 80 / (80 + 20)
        assert_eq!(metrics.recall(), 80.0 / 110.0); // 80 / (80 + 30)
        assert!(metrics.f1_score() > 0.0);
    }

    #[test]
    fn test_ground_truth_dataset() {
        let mut dataset = GroundTruthDataset::new("test", "rust");
        dataset.add_annotation(GroundTruthAnnotation {
            file_path: "src/main.rs".to_string(),
            issue_type: "long_method".to_string(),
            line: 10,
            severity: 75,
            confirmed: true,
            notes: "Test annotation".to_string(),
        });

        assert_eq!(dataset.count_true_issues(), 1);
        assert_eq!(dataset.count_false_issues(), 0);
    }

    #[test]
    fn test_threshold_adjustment() {
        let adj = ThresholdAdjustment {
            direction: AdjustmentDirection::Increase,
            magnitude: 0.2,
            reason: "Test".to_string(),
        };

        assert_eq!(adj.apply(0.5), 0.6); // 0.5 * 1.2
        assert_eq!(adj.apply_int(100), 120);

        let adj = ThresholdAdjustment {
            direction: AdjustmentDirection::Decrease,
            magnitude: 0.1,
            reason: "Test".to_string(),
        };

        assert_eq!(adj.apply(0.5), 0.45); // 0.5 * 0.9
    }

    #[test]
    fn test_optimizer_suggestion() {
        let optimizer = ThresholdOptimizer::new(OptimizationConfig::balanced());

        // Low precision scenario
        let mut metrics = CalibrationMetrics::new();
        metrics.true_positives = 60;
        metrics.false_positives = 40; // Low precision: 60%

        let adjustment = optimizer.suggest_adjustment(&metrics);
        assert_eq!(adjustment.direction, AdjustmentDirection::Increase);

        // Low recall scenario
        let mut metrics = CalibrationMetrics::new();
        metrics.true_positives = 60;
        metrics.false_negatives = 40; // Low recall: 60%

        let adjustment = optimizer.suggest_adjustment(&metrics);
        assert_eq!(adjustment.direction, AdjustmentDirection::Decrease);
    }

    #[test]
    fn test_ab_test_creation() {
        let test = ABTest::new("test_thresholds", 0.7, 0.75);
        assert_eq!(test.name, "test_thresholds");
        assert_eq!(test.control_threshold, 0.7);
        assert_eq!(test.variant_threshold, 0.75);
        assert_eq!(test.split_ratio, 0.5);
    }
}
