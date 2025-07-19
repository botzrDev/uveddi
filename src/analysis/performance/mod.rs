pub mod baseline;
pub mod optimizations;
pub mod testing;

pub use baseline::{
    PerformanceAnalyzer, 
    PerformanceBaseline, 
    PerformanceBottleneck, 
    BottleneckSeverity,
    AnalysisError
};

pub use optimizations::{
    RenderingOptimizer,
    OptimizationRequest,
    OptimizationResult,
    RenderQuality,
    PerformanceStats,
    OptimizationError
};

pub use testing::{
    PerformanceValidator,
    PerformanceTestSuite,
    TestResult,
    TestSummary,
    ValidationError
};