pub mod baseline;
pub mod optimizations;
pub mod testing;

#[cfg(not(feature = "image-rendering"))]
pub mod image_stubs;

pub use baseline::{
    AnalysisError, BottleneckSeverity, PerformanceAnalyzer, PerformanceBaseline,
    PerformanceBottleneck,
};

pub use optimizations::{
    OptimizationError, OptimizationRequest, OptimizationResult, PerformanceStats, RenderQuality,
    RenderingOptimizer,
};

pub use testing::{
    PerformanceTestSuite, PerformanceValidator, TestResult, TestSummary, ValidationError,
};
