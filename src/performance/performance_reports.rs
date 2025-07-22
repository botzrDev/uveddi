//! Performance report generation with statistical insights
//!
//! This module provides:
//! - Comprehensive performance reports combining Criterion.rs and statistical analysis
//! - HTML and JSON report formats
//! - Historical trend analysis
//! - Executive summary generation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use tera::{Context, Tera};
use tokio::fs;

use crate::performance::{
    BaselineComparison, BaselineRecommendation, BenchmarkBaseline, ChangeCategory,
    ChangePointResult, MannKendallResult, StatisticalAnalyzer, TrendDetector,
};

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub metadata: ReportMetadata,
    pub executive_summary: ExecutiveSummary,
    pub benchmark_results: Vec<BenchmarkReport>,
    pub statistical_analysis: ReportStatisticalAnalysis,
    pub trends: TrendAnalysis,
    pub recommendations: Vec<ReportRecommendation>,
    pub historical_context: Option<HistoricalContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub generated_at: SystemTime,
    pub report_version: String,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub build_config: String,
    pub environment: String,
    pub total_benchmarks: usize,
    pub analysis_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub overall_performance_status: PerformanceStatus,
    pub total_regressions: usize,
    pub total_improvements: usize,
    pub critical_issues: usize,
    pub performance_score: f64, // 0.0 to 100.0
    pub key_findings: Vec<String>,
    pub action_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceStatus {
    Excellent,
    Good,
    Concerning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub name: String,
    pub status: BenchmarkStatus,
    pub current_performance: BenchmarkPerformance,
    pub baseline_comparison: Option<BaselineComparisonReport>,
    pub statistical_confidence: f64,
    pub insights: Vec<String>,
    pub visualization_data: VisualizationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkStatus {
    Pass,
    Warning,
    Fail,
    NoBaseline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkPerformance {
    pub metric_value: f64,
    pub metric_unit: String,
    pub percentile_95: f64,
    pub standard_deviation: f64,
    pub coefficient_of_variation: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparisonReport {
    pub performance_change_percent: f64,
    pub change_category: ChangeCategory,
    pub statistical_significance: f64,
    pub effect_size: f64,
    pub recommendation: BaselineRecommendation,
    pub confidence_interval: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    pub histogram_buckets: Vec<f64>,
    pub histogram_counts: Vec<u32>,
    pub time_series: Vec<(SystemTime, f64)>,
    pub trend_line: Option<Vec<(SystemTime, f64)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStatisticalAnalysis {
    pub overall_trend: TrendType,
    pub stability_score: f64,
    pub variability_analysis: VariabilityAnalysis,
    pub correlation_analysis: CorrelationAnalysis,
    pub outlier_analysis: OutlierAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendType {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariabilityAnalysis {
    pub high_variability_benchmarks: Vec<String>,
    pub low_variability_benchmarks: Vec<String>,
    pub average_cv: f64, // Coefficient of variation
    pub variability_trend: TrendType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    pub correlated_benchmarks: Vec<(String, String, f64)>,
    pub independent_benchmarks: Vec<String>,
    pub correlation_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysis {
    pub outlier_benchmarks: Vec<OutlierBenchmark>,
    pub outlier_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierBenchmark {
    pub name: String,
    pub outlier_type: OutlierType,
    pub severity: f64,
    pub likely_cause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierType {
    PerformanceOutlier,
    VariabilityOutlier,
    TrendOutlier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub overall_performance_trend: TrendDirection,
    pub trend_strength: f64,
    pub trend_confidence: f64,
    pub projected_performance: Option<PerformanceProjection>,
    pub trend_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    StronglyImproving,
    Improving,
    Stable,
    Degrading,
    StronglyDegrading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProjection {
    pub projected_change_30_days: f64,
    pub projected_change_90_days: f64,
    pub confidence_bands: (f64, f64),
    pub projection_quality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecommendation {
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub affected_benchmarks: Vec<String>,
    pub implementation_effort: EffortLevel,
    pub expected_impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Performance,
    Stability,
    Infrastructure,
    Methodology,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,
    Low,
    Medium,
    High,
    Extensive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalContext {
    pub historical_baselines: Vec<HistoricalBaseline>,
    pub performance_evolution: Vec<PerformanceSnapshot>,
    pub milestone_comparisons: Vec<MilestoneComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalBaseline {
    pub timestamp: SystemTime,
    pub git_commit: Option<String>,
    pub performance_summary: f64,
    pub notable_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: SystemTime,
    pub overall_score: f64,
    pub regression_count: usize,
    pub improvement_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneComparison {
    pub milestone_name: String,
    pub milestone_commit: String,
    pub performance_change: f64,
    pub significant_changes: Vec<String>,
}

/// Performance report generator
pub struct PerformanceReportGenerator {
    templates: Tera,
    statistical_analyzer: StatisticalAnalyzer,
    trend_detector: TrendDetector,
}

impl PerformanceReportGenerator {
    pub fn new() -> Result<Self> {
        let mut templates = Tera::new("templates/performance/*.html")?;

        // Add built-in templates if external templates not found
        if templates.get_template_names().count() == 0 {
            templates.add_raw_template("report.html", PERFORMANCE_REPORT_HTML)?;
            templates.add_raw_template("summary.html", PERFORMANCE_SUMMARY_HTML)?;
        }

        Ok(Self {
            templates,
            statistical_analyzer: StatisticalAnalyzer::new(),
            trend_detector: TrendDetector::new(),
        })
    }

    /// Generate comprehensive performance report
    pub async fn generate_report(
        &self,
        benchmark_comparisons: Vec<BaselineComparison>,
        historical_data: Option<Vec<BenchmarkBaseline>>,
    ) -> Result<PerformanceReport> {
        let start_time = SystemTime::now();

        let metadata = self.generate_metadata(&benchmark_comparisons).await?;
        let benchmark_reports = self
            .generate_benchmark_reports(&benchmark_comparisons)
            .await?;
        let executive_summary = self.generate_executive_summary(&benchmark_reports);
        let statistical_analysis = self
            .generate_statistical_analysis(&benchmark_reports)
            .await?;
        let trends = self
            .generate_trend_analysis(&benchmark_reports, &historical_data)
            .await?;
        let recommendations =
            self.generate_recommendations(&benchmark_reports, &statistical_analysis, &trends);
        let historical_context = self.generate_historical_context(historical_data).await?;

        let analysis_duration = start_time
            .elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut metadata = metadata;
        metadata.analysis_duration_ms = analysis_duration;

        Ok(PerformanceReport {
            metadata,
            executive_summary,
            benchmark_results: benchmark_reports,
            statistical_analysis,
            trends,
            recommendations,
            historical_context,
        })
    }

    /// Generate report metadata
    async fn generate_metadata(
        &self,
        comparisons: &[BaselineComparison],
    ) -> Result<ReportMetadata> {
        Ok(ReportMetadata {
            generated_at: SystemTime::now(),
            report_version: "2.0".to_string(),
            git_commit: self.get_git_commit().await,
            git_branch: self.get_git_branch().await,
            build_config: std::env::var("CARGO_CFG_TARGET_FEATURE")
                .unwrap_or_else(|_| "unknown".to_string()),
            environment: self.detect_environment(),
            total_benchmarks: comparisons.len(),
            analysis_duration_ms: 0, // Will be filled later
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

    fn detect_environment(&self) -> String {
        if std::env::var("CI").is_ok() {
            "CI".to_string()
        } else if std::env::var("GITHUB_ACTIONS").is_ok() {
            "GitHub Actions".to_string()
        } else {
            "Local Development".to_string()
        }
    }

    /// Generate individual benchmark reports
    async fn generate_benchmark_reports(
        &self,
        comparisons: &[BaselineComparison],
    ) -> Result<Vec<BenchmarkReport>> {
        let mut reports = Vec::new();

        for comparison in comparisons {
            let current_perf = &comparison.current_baseline.statistical_summary;

            let status = match (
                &comparison.comparison_result.change_category,
                &comparison.recommendation,
            ) {
                (ChangeCategory::MajorRegression, _) => BenchmarkStatus::Fail,
                (ChangeCategory::MinorRegression, BaselineRecommendation::Reject { .. }) => {
                    BenchmarkStatus::Fail
                }
                (ChangeCategory::MinorRegression, _) => BenchmarkStatus::Warning,
                (ChangeCategory::HighVariance, _) => BenchmarkStatus::Warning,
                (_, _) if comparison.previous_baseline.is_none() => BenchmarkStatus::NoBaseline,
                _ => BenchmarkStatus::Pass,
            };

            let current_performance = BenchmarkPerformance {
                metric_value: current_perf.mean,
                metric_unit: "ns".to_string(), // Would be determined from baseline type
                percentile_95: current_perf.confidence_interval_95.1,
                standard_deviation: current_perf.std_dev,
                coefficient_of_variation: if current_perf.mean != 0.0 {
                    current_perf.std_dev / current_perf.mean.abs()
                } else {
                    0.0
                },
                sample_count: comparison.current_baseline.sample_count,
            };

            let baseline_comparison = if comparison.previous_baseline.is_some() {
                Some(BaselineComparisonReport {
                    performance_change_percent: comparison
                        .comparison_result
                        .performance_change_percent,
                    change_category: comparison.comparison_result.change_category.clone(),
                    statistical_significance: comparison.comparison_result.statistical_significance,
                    effect_size: comparison.comparison_result.effect_size,
                    recommendation: comparison.recommendation.clone(),
                    confidence_interval: current_perf.confidence_interval_95,
                })
            } else {
                None
            };

            let insights = self.generate_benchmark_insights(comparison);
            let visualization_data = self.generate_visualization_data(&comparison.current_baseline);

            reports.push(BenchmarkReport {
                name: comparison.benchmark_name.clone(),
                status,
                current_performance,
                baseline_comparison,
                statistical_confidence: comparison.statistical_confidence,
                insights,
                visualization_data,
            });
        }

        Ok(reports)
    }

    fn generate_benchmark_insights(&self, comparison: &BaselineComparison) -> Vec<String> {
        let mut insights = Vec::new();

        let current_stats = &comparison.current_baseline.statistical_summary;

        // Stability insight
        if current_stats.trend_stability > 0.9 {
            insights.push("Highly stable performance with low variance".to_string());
        } else if current_stats.trend_stability < 0.5 {
            insights.push("High performance variability detected".to_string());
        }

        // Statistical significance insight
        if let Some(mk) = &current_stats.mann_kendall_result {
            if mk.p_value < 0.01 {
                insights.push(format!(
                    "Strong statistical evidence of {} trend",
                    format!("{:?}", mk.trend).to_lowercase()
                ));
            }
        }

        // Change point insight
        if let Some(cp) = &current_stats.change_point_analysis {
            if !cp.change_points.is_empty() {
                insights.push(format!(
                    "Detected {} change points in performance data",
                    cp.change_points.len()
                ));
            }
        }

        // Performance comparison insight
        if let Some(ref previous) = comparison.previous_baseline {
            let change = comparison.comparison_result.performance_change_percent;
            if change.abs() > 10.0 {
                let direction = if change > 0.0 { "degraded" } else { "improved" };
                insights.push(format!(
                    "Performance {} by {:.1}% compared to baseline",
                    direction,
                    change.abs()
                ));
            }
        }

        insights
    }

    fn generate_visualization_data(&self, baseline: &BenchmarkBaseline) -> VisualizationData {
        // Generate histogram
        let mut measurements = baseline.measurements.clone();
        measurements.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = measurements[0];
        let max = measurements[measurements.len() - 1];
        let bucket_count = 20.min(measurements.len());
        let bucket_width = (max - min) / bucket_count as f64;

        let mut histogram_buckets = Vec::new();
        let mut histogram_counts = vec![0u32; bucket_count];

        for i in 0..bucket_count {
            histogram_buckets.push(min + i as f64 * bucket_width);
        }

        for &measurement in &measurements {
            let bucket_index = if bucket_width > 0.0 {
                ((measurement - min) / bucket_width).floor() as usize
            } else {
                0
            }
            .min(bucket_count - 1);
            histogram_counts[bucket_index] += 1;
        }

        // Generate time series (simplified - would use actual timestamps in real implementation)
        let time_series: Vec<(SystemTime, f64)> = measurements
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let time = baseline.created_at + std::time::Duration::from_secs(i as u64);
                (time, value)
            })
            .collect();

        VisualizationData {
            histogram_buckets,
            histogram_counts,
            time_series,
            trend_line: None, // Would be calculated using trend analysis
        }
    }

    /// Generate executive summary
    fn generate_executive_summary(&self, reports: &[BenchmarkReport]) -> ExecutiveSummary {
        let total_regressions = reports
            .iter()
            .filter(|r| matches!(r.status, BenchmarkStatus::Fail))
            .count();

        let total_improvements = reports
            .iter()
            .filter(|r| {
                r.baseline_comparison
                    .as_ref()
                    .map(|c| {
                        matches!(
                            c.change_category,
                            ChangeCategory::MinorImprovement | ChangeCategory::MajorImprovement
                        )
                    })
                    .unwrap_or(false)
            })
            .count();

        let critical_issues = reports
            .iter()
            .filter(|r| {
                r.baseline_comparison
                    .as_ref()
                    .map(|c| matches!(c.change_category, ChangeCategory::MajorRegression))
                    .unwrap_or(false)
            })
            .count();

        let performance_score = self.calculate_performance_score(reports);

        let overall_status = match performance_score {
            s if s >= 90.0 => PerformanceStatus::Excellent,
            s if s >= 75.0 => PerformanceStatus::Good,
            s if s >= 50.0 => PerformanceStatus::Concerning,
            _ => PerformanceStatus::Critical,
        };

        let key_findings = self.generate_key_findings(reports);
        let action_required = critical_issues > 0 || total_regressions > reports.len() / 4;

        ExecutiveSummary {
            overall_performance_status: overall_status,
            total_regressions,
            total_improvements,
            critical_issues,
            performance_score,
            key_findings,
            action_required,
        }
    }

    fn calculate_performance_score(&self, reports: &[BenchmarkReport]) -> f64 {
        if reports.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        for report in reports {
            let weight = report.statistical_confidence;
            let score = match report.status {
                BenchmarkStatus::Pass => 100.0,
                BenchmarkStatus::Warning => 60.0,
                BenchmarkStatus::Fail => 20.0,
                BenchmarkStatus::NoBaseline => 80.0, // Neutral score for no baseline
            };

            total_score += score * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            total_score / weight_sum
        } else {
            0.0
        }
    }

    fn generate_key_findings(&self, reports: &[BenchmarkReport]) -> Vec<String> {
        let mut findings = Vec::new();

        // Major regressions
        let major_regressions: Vec<_> = reports
            .iter()
            .filter(|r| {
                r.baseline_comparison
                    .as_ref()
                    .map(|c| matches!(c.change_category, ChangeCategory::MajorRegression))
                    .unwrap_or(false)
            })
            .collect();

        if !major_regressions.is_empty() {
            findings.push(format!(
                "{} benchmark(s) show major performance regressions",
                major_regressions.len()
            ));
        }

        // High variability
        let high_variability: Vec<_> = reports
            .iter()
            .filter(|r| r.current_performance.coefficient_of_variation > 0.2)
            .collect();

        if high_variability.len() > reports.len() / 4 {
            findings.push("Multiple benchmarks show high performance variability".to_string());
        }

        // Improvements
        let improvements: Vec<_> = reports
            .iter()
            .filter(|r| {
                r.baseline_comparison
                    .as_ref()
                    .map(|c| matches!(c.change_category, ChangeCategory::MajorImprovement))
                    .unwrap_or(false)
            })
            .collect();

        if !improvements.is_empty() {
            findings.push(format!(
                "{} benchmark(s) show significant performance improvements",
                improvements.len()
            ));
        }

        findings
    }

    /// Generate statistical analysis section
    async fn generate_statistical_analysis(
        &self,
        reports: &[BenchmarkReport],
    ) -> Result<ReportStatisticalAnalysis> {
        let overall_trend = self.determine_overall_trend(reports);
        let stability_score = self.calculate_stability_score(reports);
        let variability_analysis = self.analyze_variability(reports);
        let correlation_analysis = self.analyze_correlations(reports).await?;
        let outlier_analysis = self.analyze_outliers(reports);

        Ok(ReportStatisticalAnalysis {
            overall_trend,
            stability_score,
            variability_analysis,
            correlation_analysis,
            outlier_analysis,
        })
    }

    fn determine_overall_trend(&self, reports: &[BenchmarkReport]) -> TrendType {
        let regression_count = reports
            .iter()
            .filter(|r| matches!(r.status, BenchmarkStatus::Fail | BenchmarkStatus::Warning))
            .count();

        let improvement_count = reports
            .iter()
            .filter(|r| {
                r.baseline_comparison
                    .as_ref()
                    .map(|c| {
                        matches!(
                            c.change_category,
                            ChangeCategory::MinorImprovement | ChangeCategory::MajorImprovement
                        )
                    })
                    .unwrap_or(false)
            })
            .count();

        let total_with_baselines = reports
            .iter()
            .filter(|r| !matches!(r.status, BenchmarkStatus::NoBaseline))
            .count();

        if total_with_baselines == 0 {
            return TrendType::Stable;
        }

        let regression_ratio = regression_count as f64 / total_with_baselines as f64;
        let improvement_ratio = improvement_count as f64 / total_with_baselines as f64;

        if improvement_ratio > regression_ratio * 2.0 && improvement_ratio > 0.25 {
            TrendType::Improving
        } else if regression_ratio > improvement_ratio * 2.0 && regression_ratio > 0.25 {
            TrendType::Degrading
        } else if regression_ratio > 0.5 || improvement_ratio > 0.5 {
            TrendType::Volatile
        } else {
            TrendType::Stable
        }
    }

    fn calculate_stability_score(&self, reports: &[BenchmarkReport]) -> f64 {
        if reports.is_empty() {
            return 0.0;
        }

        let avg_cv = reports
            .iter()
            .map(|r| r.current_performance.coefficient_of_variation)
            .sum::<f64>()
            / reports.len() as f64;

        // Convert CV to stability score (lower CV = higher stability)
        1.0 / (1.0 + avg_cv)
    }

    fn analyze_variability(&self, reports: &[BenchmarkReport]) -> VariabilityAnalysis {
        let threshold = 0.1; // 10% CV threshold

        let high_variability: Vec<String> = reports
            .iter()
            .filter(|r| r.current_performance.coefficient_of_variation > threshold)
            .map(|r| r.name.clone())
            .collect();

        let low_variability: Vec<String> = reports
            .iter()
            .filter(|r| r.current_performance.coefficient_of_variation <= threshold / 2.0)
            .map(|r| r.name.clone())
            .collect();

        let average_cv = if !reports.is_empty() {
            reports
                .iter()
                .map(|r| r.current_performance.coefficient_of_variation)
                .sum::<f64>()
                / reports.len() as f64
        } else {
            0.0
        };

        let variability_trend = if average_cv > 0.15 {
            TrendType::Volatile
        } else if average_cv < 0.05 {
            TrendType::Stable
        } else {
            TrendType::Stable
        };

        VariabilityAnalysis {
            high_variability_benchmarks: high_variability,
            low_variability_benchmarks: low_variability,
            average_cv,
            variability_trend,
        }
    }

    async fn analyze_correlations(
        &self,
        _reports: &[BenchmarkReport],
    ) -> Result<CorrelationAnalysis> {
        // Simplified correlation analysis - would implement proper correlation calculation
        Ok(CorrelationAnalysis {
            correlated_benchmarks: Vec::new(),
            independent_benchmarks: Vec::new(),
            correlation_insights: vec!["Correlation analysis not yet implemented".to_string()],
        })
    }

    fn analyze_outliers(&self, reports: &[BenchmarkReport]) -> OutlierAnalysis {
        let mut outliers = Vec::new();
        let mut insights = Vec::new();

        // Detect performance outliers
        let performances: Vec<f64> = reports
            .iter()
            .map(|r| r.current_performance.metric_value)
            .collect();

        if let (Some(q1), Some(q3)) = (
            self.percentile(&performances, 0.25),
            self.percentile(&performances, 0.75),
        ) {
            let iqr = q3 - q1;
            let lower_bound = q1 - 1.5 * iqr;
            let upper_bound = q3 + 1.5 * iqr;

            for report in reports {
                let value = report.current_performance.metric_value;
                if value < lower_bound || value > upper_bound {
                    outliers.push(OutlierBenchmark {
                        name: report.name.clone(),
                        outlier_type: OutlierType::PerformanceOutlier,
                        severity: if value < lower_bound {
                            (lower_bound - value) / iqr
                        } else {
                            (value - upper_bound) / iqr
                        },
                        likely_cause: "Unusual performance compared to other benchmarks"
                            .to_string(),
                    });
                }
            }
        }

        if !outliers.is_empty() {
            insights.push(format!("Detected {} performance outliers", outliers.len()));
        }

        OutlierAnalysis {
            outlier_benchmarks: outliers,
            outlier_insights: insights,
        }
    }

    fn percentile(&self, data: &[f64], p: f64) -> Option<f64> {
        if data.is_empty() {
            return None;
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = p * (sorted.len() - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            Some(sorted[lower])
        } else {
            let weight = index - lower as f64;
            Some(sorted[lower] * (1.0 - weight) + sorted[upper] * weight)
        }
    }

    /// Generate trend analysis
    async fn generate_trend_analysis(
        &self,
        reports: &[BenchmarkReport],
        historical_data: &Option<Vec<BenchmarkBaseline>>,
    ) -> Result<TrendAnalysis> {
        let overall_trend = self.determine_overall_performance_trend(reports);
        let (trend_strength, trend_confidence) = self.calculate_trend_metrics(reports);
        let projected_performance = self.project_future_performance(historical_data).await?;
        let trend_insights = self.generate_trend_insights(reports, &overall_trend);

        Ok(TrendAnalysis {
            overall_performance_trend: overall_trend,
            trend_strength,
            trend_confidence,
            projected_performance,
            trend_insights,
        })
    }

    fn determine_overall_performance_trend(&self, reports: &[BenchmarkReport]) -> TrendDirection {
        let changes: Vec<f64> = reports
            .iter()
            .filter_map(|r| r.baseline_comparison.as_ref())
            .map(|c| c.performance_change_percent)
            .collect();

        if changes.is_empty() {
            return TrendDirection::Stable;
        }

        let avg_change = changes.iter().sum::<f64>() / changes.len() as f64;
        let significant_changes = changes.iter().filter(|&&change| change.abs() > 5.0).count();

        let significant_ratio = significant_changes as f64 / changes.len() as f64;

        match (avg_change, significant_ratio) {
            (change, ratio) if change < -10.0 && ratio > 0.5 => TrendDirection::StronglyImproving,
            (change, _) if change < -5.0 => TrendDirection::Improving,
            (change, ratio) if change > 10.0 && ratio > 0.5 => TrendDirection::StronglyDegrading,
            (change, _) if change > 5.0 => TrendDirection::Degrading,
            _ => TrendDirection::Stable,
        }
    }

    fn calculate_trend_metrics(&self, reports: &[BenchmarkReport]) -> (f64, f64) {
        let changes: Vec<f64> = reports
            .iter()
            .filter_map(|r| r.baseline_comparison.as_ref())
            .map(|c| c.performance_change_percent)
            .collect();

        if changes.is_empty() {
            return (0.0, 0.0);
        }

        let avg_change = changes.iter().sum::<f64>() / changes.len() as f64;
        let trend_strength = avg_change.abs() / 100.0; // Normalize to 0-1 range

        let confidences: Vec<f64> = reports.iter().map(|r| r.statistical_confidence).collect();

        let trend_confidence = if !confidences.is_empty() {
            confidences.iter().sum::<f64>() / confidences.len() as f64
        } else {
            0.0
        };

        (trend_strength, trend_confidence)
    }

    async fn project_future_performance(
        &self,
        _historical_data: &Option<Vec<BenchmarkBaseline>>,
    ) -> Result<Option<PerformanceProjection>> {
        // Simplified projection - would implement proper forecasting
        Ok(None)
    }

    fn generate_trend_insights(
        &self,
        reports: &[BenchmarkReport],
        trend: &TrendDirection,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        match trend {
            TrendDirection::StronglyImproving => {
                insights.push(
                    "Performance is showing consistent improvement across benchmarks".to_string(),
                );
            }
            TrendDirection::Improving => {
                insights.push("Overall performance trend is positive".to_string());
            }
            TrendDirection::StronglyDegrading => {
                insights.push(
                    "⚠️ Performance is degrading significantly across multiple benchmarks"
                        .to_string(),
                );
            }
            TrendDirection::Degrading => {
                insights.push("Performance trend shows some degradation".to_string());
            }
            TrendDirection::Stable => {
                insights.push("Performance remains stable with no significant trends".to_string());
            }
        }

        let high_confidence_count = reports
            .iter()
            .filter(|r| r.statistical_confidence > 0.9)
            .count();

        if high_confidence_count > reports.len() * 3 / 4 {
            insights.push("High statistical confidence in trend analysis".to_string());
        }

        insights
    }

    /// Generate recommendations
    fn generate_recommendations(
        &self,
        reports: &[BenchmarkReport],
        statistical_analysis: &ReportStatisticalAnalysis,
        trends: &TrendAnalysis,
    ) -> Vec<ReportRecommendation> {
        let mut recommendations = Vec::new();

        // Critical performance regressions
        let critical_regressions: Vec<&BenchmarkReport> = reports
            .iter()
            .filter(|r| matches!(r.status, BenchmarkStatus::Fail))
            .collect();

        if !critical_regressions.is_empty() {
            recommendations.push(ReportRecommendation {
                priority: RecommendationPriority::Critical,
                category: RecommendationCategory::Performance,
                title: "Address Critical Performance Regressions".to_string(),
                description: format!(
                    "Immediate action required for {} benchmark(s) showing significant performance degradation.",
                    critical_regressions.len()
                ),
                affected_benchmarks: critical_regressions.iter().map(|r| r.name.clone()).collect(),
                implementation_effort: EffortLevel::High,
                expected_impact: ImpactLevel::Critical,
            });
        }

        // High variability issues
        if !statistical_analysis
            .variability_analysis
            .high_variability_benchmarks
            .is_empty()
        {
            recommendations.push(ReportRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Stability,
                title: "Improve Benchmark Stability".to_string(),
                description: "Several benchmarks show high variability that may indicate environmental issues or unstable performance.".to_string(),
                affected_benchmarks: statistical_analysis.variability_analysis.high_variability_benchmarks.clone(),
                implementation_effort: EffortLevel::Medium,
                expected_impact: ImpactLevel::Medium,
            });
        }

        // Trend-based recommendations
        match trends.overall_performance_trend {
            TrendDirection::StronglyDegrading | TrendDirection::Degrading => {
                recommendations.push(ReportRecommendation {
                    priority: RecommendationPriority::High,
                    category: RecommendationCategory::Performance,
                    title: "Investigate Performance Degradation Trend".to_string(),
                    description: "Overall performance trend is negative, suggesting systematic issues that need investigation.".to_string(),
                    affected_benchmarks: vec!["All benchmarks".to_string()],
                    implementation_effort: EffortLevel::High,
                    expected_impact: ImpactLevel::High,
                });
            }
            _ => {}
        }

        recommendations
    }

    /// Generate historical context
    async fn generate_historical_context(
        &self,
        historical_data: Option<Vec<BenchmarkBaseline>>,
    ) -> Result<Option<HistoricalContext>> {
        if let Some(_data) = historical_data {
            // Would implement historical analysis
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// Export report as HTML
    pub async fn export_html(&self, report: &PerformanceReport, output_path: &Path) -> Result<()> {
        let mut context = Context::new();
        context.insert("report", report);

        let html = self.templates.render("report.html", &context)?;
        fs::write(output_path, html).await?;
        Ok(())
    }

    /// Export report as JSON
    pub async fn export_json(&self, report: &PerformanceReport, output_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        fs::write(output_path, json).await?;
        Ok(())
    }
}

// Built-in HTML templates (simplified versions)
const PERFORMANCE_REPORT_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Performance Analysis Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .summary { background: #f0f8ff; padding: 20px; border-radius: 8px; }
        .benchmark { border: 1px solid #ddd; margin: 10px 0; padding: 15px; }
        .fail { border-left: 4px solid #e74c3c; }
        .warning { border-left: 4px solid #f39c12; }
        .pass { border-left: 4px solid #27ae60; }
        .metric { display: inline-block; margin: 5px 10px; }
    </style>
</head>
<body>
    <h1>Performance Analysis Report</h1>
    <div class="summary">
        <h2>Executive Summary</h2>
        <p>Status: {{ report.executive_summary.overall_performance_status }}</p>
        <p>Performance Score: {{ report.executive_summary.performance_score }}%</p>
        <p>Regressions: {{ report.executive_summary.total_regressions }}</p>
        <p>Improvements: {{ report.executive_summary.total_improvements }}</p>
    </div>
    
    {% for benchmark in report.benchmark_results %}
    <div class="benchmark {{ benchmark.status | lower }}">
        <h3>{{ benchmark.name }}</h3>
        <div class="metric">Value: {{ benchmark.current_performance.metric_value }}</div>
        <div class="metric">CV: {{ benchmark.current_performance.coefficient_of_variation }}</div>
        <div class="metric">Confidence: {{ benchmark.statistical_confidence }}</div>
    </div>
    {% endfor %}
</body>
</html>
"#;

const PERFORMANCE_SUMMARY_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Performance Summary</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .metric { margin: 10px 0; padding: 10px; background: #f9f9f9; }
    </style>
</head>
<body>
    <h1>Performance Summary</h1>
    <div class="metric">Total Benchmarks: {{ total_benchmarks }}</div>
    <div class="metric">Performance Score: {{ performance_score }}%</div>
</body>
</html>
"#;
