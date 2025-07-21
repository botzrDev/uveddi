//! Benchmark baseline management system
//! 
//! This module provides:
//! - Persistent storage of benchmark baselines
//! - Baseline comparison and regression detection
//! - Integration with both Criterion.rs and iai-callgrind
//! - Statistical validation of performance changes

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tokio::fs;
use tracing::{info, warn, error, debug};

use crate::performance::{
    StatisticalAnalyzer, TrendDetector, MannKendallResult, ChangePointResult
};

/// Benchmark baseline data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkBaseline {
    pub benchmark_name: String,
    pub baseline_type: BaselineType,
    pub measurements: Vec<f64>,
    pub statistical_summary: StatisticalSummary,
    pub metadata: BenchmarkMetadata,
    pub created_at: SystemTime,
    pub last_updated: SystemTime,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaselineType {
    Criterion {
        mean_ns: f64,
        std_dev_ns: f64,
        median_ns: f64,
    },
    IaiCallgrind {
        instructions: u64,
        l1_accesses: u64,
        l2_accesses: u64,
        ram_accesses: u64,
    },
    Custom {
        metric_name: String,
        unit: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub confidence_interval_95: (f64, f64),
    pub mann_kendall_result: Option<MannKendallResult>,
    pub change_point_analysis: Option<ChangePointResult>,
    pub trend_stability: f64,  // 0.0 to 1.0, higher is more stable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub build_config: String,
    pub system_info: SystemInfo,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub architecture: String,
    pub cpu_model: String,
    pub memory_gb: u64,
    pub rust_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub benchmark_name: String,
    pub current_baseline: BenchmarkBaseline,
    pub previous_baseline: Option<BenchmarkBaseline>,
    pub comparison_result: ComparisonResult,
    pub statistical_confidence: f64,
    pub recommendation: Recommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub performance_change_percent: f64,
    pub is_regression: bool,
    pub is_improvement: bool,
    pub statistical_significance: f64,
    pub effect_size: f64,
    pub change_category: ChangeCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeCategory {
    NoSignificantChange,
    MinorImprovement,
    MajorImprovement,
    MinorRegression,
    MajorRegression,
    HighVariance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Recommendation {
    Accept,
    Investigate { reasons: Vec<String> },
    Reject { reasons: Vec<String> },
    RequireManualReview,
}

/// Benchmark baseline manager
pub struct BenchmarkBaselineManager {
    storage_path: PathBuf,
    config: BaselineConfig,
    statistical_analyzer: StatisticalAnalyzer,
    trend_detector: TrendDetector,
    current_baselines: HashMap<String, BenchmarkBaseline>,
}

#[derive(Debug, Clone)]
pub struct BaselineConfig {
    pub regression_threshold_percent: f64,
    pub improvement_threshold_percent: f64,
    pub statistical_confidence_threshold: f64,
    pub min_samples_for_baseline: usize,
    pub max_baseline_age_days: u64,
    pub enable_automatic_baseline_updates: bool,
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            regression_threshold_percent: 5.0,
            improvement_threshold_percent: 5.0,
            statistical_confidence_threshold: 0.95,
            min_samples_for_baseline: 10,
            max_baseline_age_days: 30,
            enable_automatic_baseline_updates: false,
        }
    }
}

impl BenchmarkBaselineManager {
    /// Create a new baseline manager
    pub async fn new(storage_path: impl AsRef<Path>, config: BaselineConfig) -> Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();
        
        // Ensure storage directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut manager = Self {
            storage_path,
            config,
            statistical_analyzer: StatisticalAnalyzer::new(),
            trend_detector: TrendDetector::new(),
            current_baselines: HashMap::new(),
        };

        // Load existing baselines
        manager.load_baselines().await?;

        Ok(manager)
    }

    /// Load baselines from persistent storage
    async fn load_baselines(&mut self) -> Result<()> {
        if !self.storage_path.exists() {
            info!("No existing baseline file found, starting with empty baselines");
            return Ok(());
        }

        let content = fs::read_to_string(&self.storage_path).await?;
        let baselines: HashMap<String, BenchmarkBaseline> = serde_json::from_str(&content)?;
        
        // Filter out stale baselines
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .saturating_sub(self.config.max_baseline_age_days * 24 * 3600);

        for (name, baseline) in baselines {
            let baseline_age = baseline.created_at
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            if baseline_age >= cutoff {
                self.current_baselines.insert(name, baseline);
            } else {
                debug!("Filtered out stale baseline: {}", name);
            }
        }

        info!("Loaded {} baselines from storage", self.current_baselines.len());
        Ok(())
    }

    /// Save baselines to persistent storage
    async fn save_baselines(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.current_baselines)?;
        fs::write(&self.storage_path, content).await?;
        debug!("Saved {} baselines to storage", self.current_baselines.len());
        Ok(())
    }

    /// Create or update a benchmark baseline
    pub async fn create_baseline(
        &mut self,
        benchmark_name: &str,
        measurements: Vec<f64>,
        baseline_type: BaselineType,
    ) -> Result<BenchmarkBaseline> {
        if measurements.len() < self.config.min_samples_for_baseline {
            return Err(anyhow!(
                "Insufficient measurements for baseline: {} < {}",
                measurements.len(),
                self.config.min_samples_for_baseline
            ));
        }

        let statistical_summary = self.calculate_statistical_summary(&measurements).await?;
        let metadata = self.collect_metadata().await?;

        let baseline = BenchmarkBaseline {
            benchmark_name: benchmark_name.to_string(),
            baseline_type,
            measurements: measurements.clone(),
            statistical_summary,
            metadata,
            created_at: SystemTime::now(),
            last_updated: SystemTime::now(),
            sample_count: measurements.len(),
        };

        self.current_baselines.insert(benchmark_name.to_string(), baseline.clone());
        self.save_baselines().await?;

        info!(
            "Created baseline for '{}' with {} measurements",
            benchmark_name,
            baseline.sample_count
        );

        Ok(baseline)
    }

    /// Compare current measurements against existing baseline
    pub async fn compare_against_baseline(
        &self,
        benchmark_name: &str,
        current_measurements: &[f64],
        current_baseline_type: BaselineType,
    ) -> Result<BaselineComparison> {
        let current_statistical_summary = self.calculate_statistical_summary(current_measurements).await?;
        let current_metadata = self.collect_metadata().await?;

        let current_baseline = BenchmarkBaseline {
            benchmark_name: benchmark_name.to_string(),
            baseline_type: current_baseline_type,
            measurements: current_measurements.to_vec(),
            statistical_summary: current_statistical_summary,
            metadata: current_metadata,
            created_at: SystemTime::now(),
            last_updated: SystemTime::now(),
            sample_count: current_measurements.len(),
        };

        let previous_baseline = self.current_baselines.get(benchmark_name).cloned();

        let comparison_result = if let Some(ref previous) = previous_baseline {
            self.calculate_comparison_result(&current_baseline, previous).await?
        } else {
            ComparisonResult {
                performance_change_percent: 0.0,
                is_regression: false,
                is_improvement: false,
                statistical_significance: 0.0,
                effect_size: 0.0,
                change_category: ChangeCategory::NoSignificantChange,
            }
        };

        let statistical_confidence = if let Some(ref previous) = previous_baseline {
            self.calculate_statistical_confidence(&current_baseline, previous).await?
        } else {
            1.0
        };

        let recommendation = self.generate_recommendation(&comparison_result, statistical_confidence);

        Ok(BaselineComparison {
            benchmark_name: benchmark_name.to_string(),
            current_baseline,
            previous_baseline,
            comparison_result,
            statistical_confidence,
            recommendation,
        })
    }

    /// Calculate statistical summary for measurements
    async fn calculate_statistical_summary(&self, measurements: &[f64]) -> Result<StatisticalSummary> {
        if measurements.is_empty() {
            return Err(anyhow!("Cannot calculate statistics for empty measurements"));
        }

        let mean = measurements.iter().sum::<f64>() / measurements.len() as f64;
        
        let mut sorted = measurements.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let variance = measurements.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / measurements.len() as f64;
        let std_dev = variance.sqrt();

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];

        let confidence_interval_95 = self.statistical_analyzer
            .confidence_interval(measurements, 0.95)?;

        let mann_kendall_result = if measurements.len() >= 10 {
            self.statistical_analyzer.mann_kendall_test(measurements).ok()
        } else {
            None
        };

        let change_point_analysis = if measurements.len() >= 20 {
            self.trend_detector.detect_change_points_pelt(measurements).ok()
        } else {
            None
        };

        // Calculate trend stability (lower variance indicates higher stability)
        let trend_stability = if std_dev > 0.0 {
            1.0 / (1.0 + (std_dev / mean.abs()).min(10.0))
        } else {
            1.0
        };

        Ok(StatisticalSummary {
            mean,
            median,
            std_dev,
            min,
            max,
            confidence_interval_95,
            mann_kendall_result,
            change_point_analysis,
            trend_stability,
        })
    }

    /// Calculate comparison result between current and previous baselines
    async fn calculate_comparison_result(
        &self,
        current: &BenchmarkBaseline,
        previous: &BenchmarkBaseline,
    ) -> Result<ComparisonResult> {
        let current_mean = current.statistical_summary.mean;
        let previous_mean = previous.statistical_summary.mean;

        let performance_change_percent = if previous_mean != 0.0 {
            ((current_mean - previous_mean) / previous_mean) * 100.0
        } else {
            0.0
        };

        // Calculate effect size (Cohen's d)
        let effect_size = self.statistical_analyzer
            .effect_size(&previous.measurements, &current.measurements)?;

        let statistical_significance = if effect_size.abs() > 0.2 {
            0.95
        } else if effect_size.abs() > 0.1 {
            0.80
        } else {
            0.50
        };

        let is_regression = performance_change_percent > self.config.regression_threshold_percent
            && statistical_significance >= self.config.statistical_confidence_threshold;

        let is_improvement = performance_change_percent < -self.config.improvement_threshold_percent
            && statistical_significance >= self.config.statistical_confidence_threshold;

        let change_category = self.categorize_change(
            performance_change_percent,
            effect_size,
            statistical_significance,
            current.statistical_summary.trend_stability,
        );

        Ok(ComparisonResult {
            performance_change_percent,
            is_regression,
            is_improvement,
            statistical_significance,
            effect_size,
            change_category,
        })
    }

    /// Calculate statistical confidence in the comparison
    async fn calculate_statistical_confidence(
        &self,
        current: &BenchmarkBaseline,
        previous: &BenchmarkBaseline,
    ) -> Result<f64> {
        // Combine multiple confidence indicators
        let mut confidence_factors = Vec::new();

        // Sample size confidence
        let min_samples = current.sample_count.min(previous.sample_count) as f64;
        let sample_confidence = (min_samples / self.config.min_samples_for_baseline as f64).min(1.0);
        confidence_factors.push(sample_confidence);

        // Statistical test confidence
        if let Some(ref mk_result) = current.statistical_summary.mann_kendall_result {
            confidence_factors.push(mk_result.confidence);
        }

        // Trend stability confidence
        let stability_confidence = (current.statistical_summary.trend_stability + 
                                   previous.statistical_summary.trend_stability) / 2.0;
        confidence_factors.push(stability_confidence);

        // Environment consistency confidence
        let env_confidence = if self.environments_consistent(&current.metadata, &previous.metadata) {
            1.0
        } else {
            0.7
        };
        confidence_factors.push(env_confidence);

        // Calculate weighted average confidence
        Ok(confidence_factors.iter().sum::<f64>() / confidence_factors.len() as f64)
    }

    /// Check if test environments are consistent between runs
    fn environments_consistent(&self, current: &BenchmarkMetadata, previous: &BenchmarkMetadata) -> bool {
        current.system_info.architecture == previous.system_info.architecture
            && current.build_config == previous.build_config
            && current.system_info.rust_version == previous.system_info.rust_version
    }

    /// Categorize the type of change observed
    fn categorize_change(
        &self,
        percent_change: f64,
        effect_size: f64,
        statistical_significance: f64,
        trend_stability: f64,
    ) -> ChangeCategory {
        let abs_change = percent_change.abs();
        let abs_effect = effect_size.abs();

        if statistical_significance < 0.8 || trend_stability < 0.5 {
            return ChangeCategory::HighVariance;
        }

        if abs_change < self.config.regression_threshold_percent / 2.0 && abs_effect < 0.2 {
            ChangeCategory::NoSignificantChange
        } else if percent_change < 0.0 {
            // Improvement (negative change means better performance)
            if abs_change > self.config.improvement_threshold_percent * 2.0 {
                ChangeCategory::MajorImprovement
            } else {
                ChangeCategory::MinorImprovement
            }
        } else {
            // Regression (positive change means worse performance)
            if abs_change > self.config.regression_threshold_percent * 2.0 {
                ChangeCategory::MajorRegression
            } else {
                ChangeCategory::MinorRegression
            }
        }
    }

    /// Generate recommendation based on comparison results
    fn generate_recommendation(
        &self,
        comparison: &ComparisonResult,
        statistical_confidence: f64,
    ) -> Recommendation {
        let confidence_threshold = self.config.statistical_confidence_threshold;

        if statistical_confidence < confidence_threshold * 0.8 {
            return Recommendation::RequireManualReview;
        }

        match comparison.change_category {
            ChangeCategory::NoSignificantChange | ChangeCategory::MinorImprovement => {
                Recommendation::Accept
            },
            ChangeCategory::MajorImprovement => {
                Recommendation::Accept
            },
            ChangeCategory::MinorRegression => {
                if comparison.statistical_significance >= confidence_threshold {
                    Recommendation::Investigate {
                        reasons: vec![
                            format!("Minor performance regression detected: {:.1}%", comparison.performance_change_percent),
                            format!("Effect size: {:.3}", comparison.effect_size),
                            "Review recent changes for performance impact".to_string(),
                        ]
                    }
                } else {
                    Recommendation::Accept
                }
            },
            ChangeCategory::MajorRegression => {
                Recommendation::Reject {
                    reasons: vec![
                        format!("Major performance regression detected: {:.1}%", comparison.performance_change_percent),
                        format!("High statistical significance: {:.3}", comparison.statistical_significance),
                        format!("Large effect size: {:.3}", comparison.effect_size),
                        "Performance degradation exceeds acceptable thresholds".to_string(),
                    ]
                }
            },
            ChangeCategory::HighVariance => {
                Recommendation::Investigate {
                    reasons: vec![
                        "High performance variance detected".to_string(),
                        "Results may be unreliable due to environmental factors".to_string(),
                        "Consider re-running benchmarks in stable environment".to_string(),
                    ]
                }
            }
        }
    }

    /// Collect system and environment metadata
    async fn collect_metadata(&self) -> Result<BenchmarkMetadata> {
        Ok(BenchmarkMetadata {
            git_commit: self.get_git_commit().await,
            git_branch: self.get_git_branch().await,
            build_config: std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_else(|_| "unknown".to_string()),
            system_info: self.get_system_info(),
            environment_variables: self.get_relevant_env_vars(),
        })
    }

    async fn get_git_commit(&self) -> Option<String> {
        tokio::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .await
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }

    async fn get_git_branch(&self) -> Option<String> {
        tokio::process::Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .await
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }

    fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_model: "unknown".to_string(), // Would need system detection crate
            memory_gb: 0, // Would need system detection crate
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        }
    }

    fn get_relevant_env_vars(&self) -> HashMap<String, String> {
        let relevant_vars = [
            "CARGO_CFG_TARGET_ARCH",
            "CARGO_CFG_TARGET_OS",
            "CARGO_CFG_TARGET_FEATURE",
            "RUST_LOG",
            "RUSTFLAGS",
        ];

        relevant_vars.iter()
            .filter_map(|&var| {
                std::env::var(var).ok().map(|value| (var.to_string(), value))
            })
            .collect()
    }

    /// Update an existing baseline
    pub async fn update_baseline(
        &mut self,
        benchmark_name: &str,
        new_measurements: Vec<f64>,
        baseline_type: BaselineType,
    ) -> Result<BenchmarkBaseline> {
        if !self.config.enable_automatic_baseline_updates {
            return Err(anyhow!("Automatic baseline updates are disabled"));
        }

        self.create_baseline(benchmark_name, new_measurements, baseline_type).await
    }

    /// Get current baseline for a benchmark
    pub fn get_baseline(&self, benchmark_name: &str) -> Option<&BenchmarkBaseline> {
        self.current_baselines.get(benchmark_name)
    }

    /// List all current baselines
    pub fn list_baselines(&self) -> Vec<&BenchmarkBaseline> {
        self.current_baselines.values().collect()
    }

    /// Delete a baseline
    pub async fn delete_baseline(&mut self, benchmark_name: &str) -> Result<()> {
        if self.current_baselines.remove(benchmark_name).is_some() {
            self.save_baselines().await?;
            info!("Deleted baseline for '{}'", benchmark_name);
            Ok(())
        } else {
            Err(anyhow!("Baseline '{}' not found", benchmark_name))
        }
    }

    /// Generate a comprehensive report
    pub async fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Benchmark Baseline Report ===\n\n");
        
        if self.current_baselines.is_empty() {
            report.push_str("No baselines available.\n");
            return report;
        }

        report.push_str(&format!("Total baselines: {}\n\n", self.current_baselines.len()));

        for baseline in self.current_baselines.values() {
            report.push_str(&format!("Benchmark: {}\n", baseline.benchmark_name));
            report.push_str(&format!("Created: {:?}\n", baseline.created_at));
            report.push_str(&format!("Samples: {}\n", baseline.sample_count));
            report.push_str(&format!("Mean: {:.2}\n", baseline.statistical_summary.mean));
            report.push_str(&format!("Std Dev: {:.2}\n", baseline.statistical_summary.std_dev));
            report.push_str(&format!("Trend Stability: {:.3}\n", baseline.statistical_summary.trend_stability));
            
            if let Some(ref mk) = baseline.statistical_summary.mann_kendall_result {
                report.push_str(&format!("Trend: {:?} (confidence: {:.3})\n", mk.trend, mk.confidence));
            }
            
            report.push_str("\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_baseline_creation_and_comparison() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("baselines.json");
        let config = BaselineConfig::default();
        
        let mut manager = BenchmarkBaselineManager::new(&storage_path, config).await.unwrap();

        // Create baseline
        let measurements = (0..50).map(|i| 100.0 + i as f64 * 0.1).collect::<Vec<_>>();
        let baseline_type = BaselineType::Custom {
            metric_name: "test_metric".to_string(),
            unit: "ns".to_string(),
        };

        let baseline = manager.create_baseline("test_bench", measurements, baseline_type.clone()).await.unwrap();
        assert_eq!(baseline.benchmark_name, "test_bench");
        assert_eq!(baseline.sample_count, 50);

        // Compare against similar measurements (should be no significant change)
        let similar_measurements = (0..50).map(|i| 101.0 + i as f64 * 0.1).collect::<Vec<_>>();
        let comparison = manager.compare_against_baseline("test_bench", &similar_measurements, baseline_type.clone()).await.unwrap();
        
        assert!(!comparison.comparison_result.is_regression);
        assert!(matches!(comparison.recommendation, Recommendation::Accept));

        // Compare against significantly different measurements (should detect regression)
        let worse_measurements = (0..50).map(|i| 120.0 + i as f64 * 0.1).collect::<Vec<_>>();
        let comparison = manager.compare_against_baseline("test_bench", &worse_measurements, baseline_type).await.unwrap();
        
        assert!(comparison.comparison_result.is_regression);
        assert!(matches!(comparison.recommendation, Recommendation::Reject { .. }));
    }
}