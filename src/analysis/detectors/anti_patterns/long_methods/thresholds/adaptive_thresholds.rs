//! Adaptive threshold calculation based on codebase characteristics

use crate::analysis::detectors::anti_patterns::long_methods::types::{
    LanguageThresholds, MethodMetrics,
};

/// Calculator for adaptive thresholds based on codebase analysis
pub struct AdaptiveThresholdCalculator {
    /// Percentile to use for threshold calculation (e.g., 90 = 90th percentile)
    percentile: f64,
    /// Minimum sample size required for adaptation
    min_sample_size: usize,
}

impl AdaptiveThresholdCalculator {
    /// Create a new adaptive threshold calculator
    pub fn new(percentile: f64, min_sample_size: usize) -> Self {
        Self {
            percentile: percentile.clamp(50.0, 99.0),
            min_sample_size,
        }
    }

    /// Calculate adaptive thresholds based on analyzed methods
    pub fn calculate_thresholds(
        &self,
        methods: &[MethodMetrics],
        base_thresholds: &LanguageThresholds,
    ) -> LanguageThresholds {
        if methods.len() < self.min_sample_size {
            return base_thresholds.clone();
        }

        // Collect all metrics
        let mut loc_values: Vec<u32> = methods.iter().map(|m| m.logical_loc).collect();
        let mut statement_counts: Vec<u32> = methods.iter().map(|m| m.statement_count).collect();
        let mut param_counts: Vec<u32> = methods.iter().map(|m| m.parameter_count).collect();
        let mut nesting_depths: Vec<u32> = methods.iter().map(|m| m.max_nesting_depth).collect();
        let mut cyclo_complexities: Vec<u32> =
            methods.iter().map(|m| m.cyclomatic_complexity).collect();
        let mut cognitive_complexities: Vec<u32> =
            methods.iter().map(|m| m.cognitive_complexity).collect();

        // Sort all vectors
        loc_values.sort_unstable();
        statement_counts.sort_unstable();
        param_counts.sort_unstable();
        nesting_depths.sort_unstable();
        cyclo_complexities.sort_unstable();
        cognitive_complexities.sort_unstable();

        // Calculate percentile values
        LanguageThresholds {
            max_logical_loc: self
                .calculate_percentile(&loc_values)
                .max(base_thresholds.max_logical_loc),
            max_statements: self
                .calculate_percentile(&statement_counts)
                .max(base_thresholds.max_statements),
            max_parameters: self
                .calculate_percentile(&param_counts)
                .max(base_thresholds.max_parameters),
            max_nesting_depth: self
                .calculate_percentile(&nesting_depths)
                .max(base_thresholds.max_nesting_depth),
            max_cyclomatic_complexity: self
                .calculate_percentile(&cyclo_complexities)
                .max(base_thresholds.max_cyclomatic_complexity),
            max_cognitive_complexity: self
                .calculate_percentile(&cognitive_complexities)
                .max(base_thresholds.max_cognitive_complexity),
        }
    }

    /// Calculate percentile value for a sorted vector
    fn calculate_percentile(&self, sorted_values: &[u32]) -> u32 {
        if sorted_values.is_empty() {
            return 0;
        }

        let index = ((self.percentile / 100.0) * sorted_values.len() as f64) as usize;
        let index = index.min(sorted_values.len() - 1);
        sorted_values[index]
    }

    /// Suggest threshold adjustments based on violation patterns
    pub fn suggest_adjustments(
        &self,
        violations: &[MethodMetrics],
        current_thresholds: &LanguageThresholds,
    ) -> LanguageThresholds {
        if violations.is_empty() {
            return current_thresholds.clone();
        }

        // Calculate average violation amounts
        let avg_loc_violation = violations
            .iter()
            .filter(|m| m.logical_loc > current_thresholds.max_logical_loc)
            .map(|m| m.logical_loc - current_thresholds.max_logical_loc)
            .sum::<u32>() as f64
            / violations.len().max(1) as f64;

        let avg_complexity_violation = violations
            .iter()
            .filter(|m| m.cyclomatic_complexity > current_thresholds.max_cyclomatic_complexity)
            .map(|m| m.cyclomatic_complexity - current_thresholds.max_cyclomatic_complexity)
            .sum::<u32>() as f64
            / violations.len().max(1) as f64;

        // Suggest minor adjustments if violations are consistently small
        LanguageThresholds {
            max_logical_loc: if avg_loc_violation < 10.0 && avg_loc_violation > 0.0 {
                current_thresholds.max_logical_loc + 5
            } else {
                current_thresholds.max_logical_loc
            },
            max_cyclomatic_complexity: if avg_complexity_violation < 3.0
                && avg_complexity_violation > 0.0
            {
                current_thresholds.max_cyclomatic_complexity + 2
            } else {
                current_thresholds.max_cyclomatic_complexity
            },
            ..current_thresholds.clone()
        }
    }
}

impl Default for AdaptiveThresholdCalculator {
    fn default() -> Self {
        Self::new(90.0, 50)
    }
}
