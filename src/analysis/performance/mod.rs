pub mod baseline;
pub mod optimizations;
pub mod testing;

#[cfg(not(feature = "image-rendering"))]
pub mod image_stubs;

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