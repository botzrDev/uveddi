//! Performance monitoring and optimization module
//! 
//! This module provides comprehensive performance monitoring, regression detection,
//! and optimization capabilities for the Uveddi system.

pub mod regression_detection;
pub mod statistical_analysis;
pub mod trend_detection;
pub mod benchmark_baseline;
pub mod performance_reports;
pub mod criterion_integration;
pub mod genetic_bottleneck;

pub use regression_detection::{
    PerformanceRegressionDetector,
    RegressionDetectionConfig,
    PerformanceBaseline,
    RegressionResult,
    RegressionSeverity,
    MetricDataPoint,
    EnhancedRegressionResult,
    ValidationResult,
};

pub use statistical_analysis::{
    StatisticalAnalyzer,
    MannKendallResult,
    TrendType,
};

pub use trend_detection::{
    TrendDetector,
    ChangePointResult,
    Segment,
};

pub use benchmark_baseline::{
    BenchmarkBaselineManager,
    BenchmarkBaseline,
    BaselineType,
    BaselineComparison,
    ComparisonResult,
    ChangeCategory,
    Recommendation as BaselineRecommendation,
    BaselineConfig,
};

pub use performance_reports::{
    PerformanceReportGenerator,
    PerformanceReport,
    ExecutiveSummary,
    BenchmarkReport,
    PerformanceStatus,
};

pub use criterion_integration::{
    CriterionIntegrationManager,
    CriterionIntegrationConfig,
    CriterionBenchmarkResult,
    IntegratedBenchmarkResult,
    RegressionVerdict,
};

pub use genetic_bottleneck::{
    GeneticBottleneckDetector,
    BottleneckChromosome,
    OptimizationTarget,
    OptimizationObjective,
    TargetDirection,
    Bottleneck,
    ResourceType,
    BottleneckSeverity,
    BottleneckLocation,
    BottleneckMetrics,
    Recommendation as GeneticRecommendation,
    RecommendationPriority,
    ImplementationEffort,
    RecommendationCategory,
    BottleneckAnalysis,
    AnalysisMetadata,
    FitnessEvaluator,
    PerformanceDataPoint,
};

#[cfg(test)]
mod test_validation;

#[cfg(test)]
mod overhead_test;