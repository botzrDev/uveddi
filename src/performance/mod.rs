//! Performance monitoring and optimization module
//!
//! This module provides comprehensive performance monitoring, regression detection,
//! and optimization capabilities for the Uveddi system.

pub mod benchmark_baseline;
pub mod criterion_integration;
pub mod genetic_bottleneck;
pub mod performance_reports;
pub mod regression_detection;
pub mod statistical_analysis;
pub mod trend_detection;

pub use regression_detection::{
    EnhancedRegressionResult, MetricDataPoint, PerformanceBaseline, PerformanceRegressionDetector,
    RegressionDetectionConfig, RegressionResult, RegressionSeverity, ValidationResult,
};

pub use statistical_analysis::{MannKendallResult, StatisticalAnalyzer, TrendType};

pub use trend_detection::{ChangePointResult, Segment, TrendDetector};

pub use benchmark_baseline::{
    BaselineComparison, BaselineConfig, BaselineType, BenchmarkBaseline, BenchmarkBaselineManager,
    ChangeCategory, ComparisonResult, Recommendation as BaselineRecommendation,
};

pub use performance_reports::{
    BenchmarkReport, ExecutiveSummary, PerformanceReport, PerformanceReportGenerator,
    PerformanceStatus,
};

pub use criterion_integration::{
    CriterionBenchmarkResult, CriterionIntegrationConfig, CriterionIntegrationManager,
    IntegratedBenchmarkResult, RegressionVerdict,
};

pub use genetic_bottleneck::{
    AnalysisMetadata, Bottleneck, BottleneckAnalysis, BottleneckChromosome, BottleneckLocation,
    BottleneckMetrics, BottleneckSeverity, FitnessEvaluator, GeneticBottleneckDetector,
    ImplementationEffort, OptimizationObjective, OptimizationTarget, PerformanceDataPoint,
    Recommendation as GeneticRecommendation, RecommendationCategory, RecommendationPriority,
    ResourceType, TargetDirection,
};

#[cfg(test)]
mod test_validation;

#[cfg(test)]
mod overhead_test;
