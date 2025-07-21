//! Statistical analysis module for performance regression detection
//! Implements Mann-Kendall trend tests and advanced statistical methods

#[cfg(all(feature = "regression-detection", feature = "chaos"))]
use statrs::distribution::{Normal, ContinuousCDF};

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Mann-Kendall trend test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MannKendallResult {
    pub tau: f64,           // Kendall's tau
    pub p_value: f64,       // Statistical significance
    pub trend: TrendType,   // Detected trend direction
    pub confidence: f64,    // Confidence level (0.0-1.0)
    pub effect_size: f64,   // Magnitude of trend
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendType {
    Increasing,
    Decreasing,
    NoTrend,
    Uncertain,
}

/// Statistical analyzer for performance data
#[derive(Debug)]
pub struct StatisticalAnalyzer {
    significance_threshold: f64,  // Default: 0.05 for 95% confidence
    min_samples: usize,          // Minimum samples for reliable analysis
}

impl StatisticalAnalyzer {
    pub fn new() -> Self {
        Self {
            significance_threshold: 0.05,
            min_samples: 10,
        }
    }

    /// Perform Mann-Kendall trend test on time series data
    pub fn mann_kendall_test(&self, values: &[f64]) -> Result<MannKendallResult> {
        if values.len() < self.min_samples {
            return Ok(MannKendallResult {
                tau: 0.0,
                p_value: 1.0,
                trend: TrendType::Uncertain,
                confidence: 0.0,
                effect_size: 0.0,
            });
        }

        let n = values.len();
        let mut s = 0i64;
        let mut ties = HashMap::new();

        // Calculate S statistic
        for i in 0..(n-1) {
            for j in (i+1)..n {
                let diff = values[j] - values[i];
                if diff > 0.0 {
                    s += 1;
                } else if diff < 0.0 {
                    s -= 1;
                } else {
                    // Count ties for variance adjustment
                    *ties.entry((values[i] * 1e6) as i64).or_insert(0) += 1;
                }
            }
        }

        // Calculate variance with tie correction
        let n_f64 = n as f64;
        let mut var_s = (n_f64 * (n_f64 - 1.0) * (2.0 * n_f64 + 5.0)) / 18.0;
        
        // Apply tie correction
        for &tie_count in ties.values() {
            let t = tie_count as f64;
            var_s -= (t * (t - 1.0) * (2.0 * t + 5.0)) / 18.0;
        }

        // Calculate normalized test statistic
        let z = if s > 0 {
            ((s as f64) - 1.0) / var_s.sqrt()
        } else if s < 0 {
            ((s as f64) + 1.0) / var_s.sqrt()
        } else {
            0.0
        };

        // Calculate p-value using normal distribution
        #[cfg(all(feature = "regression-detection", feature = "chaos"))]
        let p_value = {
            let normal = Normal::new(0.0, 1.0).unwrap();
            2.0 * (1.0 - normal.cdf(z.abs()))
        };
        
        #[cfg(not(all(feature = "regression-detection", feature = "chaos")))]
        let p_value = self.approximate_p_value(z);

        // Calculate Kendall's tau
        let tau = (s as f64) / ((n_f64 * (n_f64 - 1.0)) / 2.0);

        // Determine trend direction and confidence
        let trend = if p_value < self.significance_threshold {
            if s > 0 {
                TrendType::Increasing
            } else if s < 0 {
                TrendType::Decreasing
            } else {
                TrendType::NoTrend
            }
        } else {
            TrendType::NoTrend
        };

        let confidence = 1.0 - p_value;
        let effect_size = tau.abs();

        Ok(MannKendallResult {
            tau,
            p_value,
            trend,
            confidence,
            effect_size,
        })
    }

    /// Approximate p-value calculation when statrs is not available
    fn approximate_p_value(&self, z: f64) -> f64 {
        let abs_z = z.abs();
        
        // Approximation using complementary error function
        // This is a simplified approximation for demonstration
        if abs_z > 3.0 {
            0.001
        } else if abs_z > 2.58 {
            0.01
        } else if abs_z > 1.96 {
            0.05
        } else if abs_z > 1.64 {
            0.10
        } else {
            0.5 - (abs_z / 3.29) * 0.4
        }
    }

    /// Calculate confidence intervals for performance metrics
    pub fn confidence_interval(&self, values: &[f64], confidence_level: f64) -> Result<(f64, f64)> {
        if values.len() < 2 {
            let value = values.first().copied().unwrap_or(0.0);
            return Ok((value, value));
        }

        let mean = self.calculate_mean(values);
        let std_dev = self.calculate_std_dev(values, mean);
        
        // Determine critical value based on confidence level
        let alpha = 1.0 - confidence_level;
        let t_critical = self.get_t_critical(values.len(), alpha / 2.0);
        
        let standard_error = std_dev / (values.len() as f64).sqrt();
        let margin_of_error = t_critical * standard_error;
        
        Ok((mean - margin_of_error, mean + margin_of_error))
    }

    /// Calculate effect size (Cohen's d) for regression magnitude
    pub fn effect_size(&self, baseline: &[f64], current: &[f64]) -> Result<f64> {
        if baseline.is_empty() || current.is_empty() {
            return Ok(0.0);
        }

        let mean_baseline = self.calculate_mean(baseline);
        let mean_current = self.calculate_mean(current);
        
        let var_baseline = self.calculate_variance(baseline, mean_baseline);
        let var_current = self.calculate_variance(current, mean_current);
        
        // Calculate pooled standard deviation
        let n1 = baseline.len() as f64;
        let n2 = current.len() as f64;
        let pooled_variance = ((n1 - 1.0) * var_baseline + (n2 - 1.0) * var_current) / (n1 + n2 - 2.0);
        let pooled_std = pooled_variance.sqrt();
        
        if pooled_std == 0.0 {
            return Ok(0.0);
        }
        
        Ok((mean_current - mean_baseline) / pooled_std)
    }

    /// Helper function to calculate mean
    fn calculate_mean(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// Helper function to calculate standard deviation
    fn calculate_std_dev(&self, values: &[f64], mean: f64) -> f64 {
        if values.len() <= 1 {
            return 0.0;
        }
        
        let variance = self.calculate_variance(values, mean);
        variance.sqrt()
    }

    /// Helper function to calculate variance
    fn calculate_variance(&self, values: &[f64], mean: f64) -> f64 {
        if values.len() <= 1 {
            return 0.0;
        }
        
        values.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64
    }

    /// Get t-critical value (approximation)
    fn get_t_critical(&self, n: usize, alpha: f64) -> f64 {
        // Simplified t-critical values for common cases
        // In production, use proper t-distribution
        if n >= 30 {
            // Large sample, use normal distribution approximation
            if alpha <= 0.005 {
                2.576  // 99% confidence
            } else if alpha <= 0.01 {
                2.326  // 98% confidence
            } else if alpha <= 0.025 {
                1.96   // 95% confidence
            } else if alpha <= 0.05 {
                1.645  // 90% confidence
            } else {
                1.0
            }
        } else {
            // Small sample approximations (simplified)
            if alpha <= 0.025 {
                2.5    // Rough approximation for 95% confidence
            } else if alpha <= 0.05 {
                2.0    // Rough approximation for 90% confidence
            } else {
                1.5
            }
        }
    }

    /// Set significance threshold
    pub fn with_significance_threshold(mut self, threshold: f64) -> Self {
        self.significance_threshold = threshold;
        self
    }

    /// Set minimum samples requirement
    pub fn with_min_samples(mut self, min_samples: usize) -> Self {
        self.min_samples = min_samples;
        self
    }
}

impl Default for StatisticalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mann_kendall_increasing_trend() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Increasing));
        assert!(result.p_value < 0.05);
        assert!(result.confidence > 0.95);
        assert!(result.tau > 0.8); // Strong positive correlation
    }

    #[test]
    fn test_mann_kendall_decreasing_trend() {
        let values = vec![10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Decreasing));
        assert!(result.p_value < 0.05);
        assert!(result.confidence > 0.95);
        assert!(result.tau < -0.8); // Strong negative correlation
    }

    #[test]
    fn test_mann_kendall_no_trend() {
        let values = vec![5.0, 5.1, 4.9, 5.0, 5.2, 4.8, 5.0, 5.1, 4.9, 5.0];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::NoTrend));
        assert!(result.p_value > 0.05);
        assert!(result.tau.abs() < 0.3); // Weak correlation
    }

    #[test]
    fn test_confidence_intervals() {
        let values = vec![10.0, 12.0, 11.0, 13.0, 9.0, 14.0, 10.5, 11.5, 12.5, 10.8];
        let analyzer = StatisticalAnalyzer::new();
        let (lower, upper) = analyzer.confidence_interval(&values, 0.95).unwrap();
        
        let mean = analyzer.calculate_mean(&values);
        assert!(lower < mean && mean < upper);
        assert!((upper - lower) > 0.0); // Non-zero interval width
    }

    #[test]
    fn test_effect_size() {
        let baseline = vec![10.0, 11.0, 9.0, 10.5, 9.5];
        let current = vec![12.0, 13.0, 11.0, 12.5, 11.5]; // 2-point increase
        
        let analyzer = StatisticalAnalyzer::new();
        let effect_size = analyzer.effect_size(&baseline, &current).unwrap();
        
        assert!(effect_size > 1.0); // Large effect size
    }

    #[test]
    fn test_insufficient_samples() {
        let values = vec![1.0, 2.0, 3.0]; // Less than minimum
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Uncertain));
        assert!(result.confidence == 0.0);
    }

    #[test]
    fn test_empty_input() {
        let values = vec![];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        assert!(matches!(result.trend, TrendType::Uncertain));
        assert!(result.p_value == 1.0);
    }

    #[test]
    fn test_tied_values() {
        let values = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0];
        let analyzer = StatisticalAnalyzer::new();
        let result = analyzer.mann_kendall_test(&values).unwrap();
        
        // Should still detect increasing trend despite ties
        assert!(matches!(result.trend, TrendType::Increasing));
        assert!(result.tau > 0.0);
    }
}