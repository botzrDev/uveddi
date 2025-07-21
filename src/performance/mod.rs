//! Performance monitoring and optimization module
//! 
//! This module provides comprehensive performance monitoring, regression detection,
//! and optimization capabilities for the Uveddi system.

pub mod regression_detection;

pub use regression_detection::{
    PerformanceRegressionDetector,
    RegressionDetectionConfig,
    PerformanceBaseline,
    RegressionResult,
    RegressionSeverity,
    MetricDataPoint,
};