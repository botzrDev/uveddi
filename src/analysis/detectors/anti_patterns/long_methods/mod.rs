//! Long Methods anti-pattern detector
//!
//! This module provides a comprehensive long methods detection system that analyzes
//! methods and functions for excessive length and complexity using multiple metrics:
//!
//! - **Size Metrics**: Logical Lines of Code (LLOC), Statement Count
//! - **Complexity Metrics**: Cyclomatic Complexity, Cognitive Complexity
//! - **Structural Metrics**: Nesting Depth, Parameter Count
//!
//! ## Detection Strategy
//!
//! The detector uses a multi-metric approach to identify methods that are:
//! - Too long (high line count)
//! - Too complex (high cyclomatic/cognitive complexity)
//! - Too deeply nested (high nesting depth)
//! - Taking too many parameters
//!
//! ## Language Support
//!
//! - **Rust**: Conservative thresholds for systems programming
//! - **Python**: Standard thresholds based on PEP guidelines
//! - **JavaScript/TypeScript**: Framework-aware thresholds
//!
//! ## Severity Scoring
//!
//! - **Info (0-25)**: Slightly above thresholds, minor concern
//! - **Low (26-50)**: Moderate size, should be monitored
//! - **Medium (51-75)**: Clear anti-pattern, refactoring recommended
//! - **High (76-90)**: Significant design issues, refactoring needed
//! - **Critical (91-100)**: Extremely long method, immediate attention required

pub mod analyzers;
pub mod config;
pub mod detector;
pub mod extractors;
pub mod language_support;
pub mod legacy_detector;
pub mod thresholds;
pub mod types;

// Re-export main components
pub use config::LongMethodsConfig;
pub use detector::LongMethodsDetector;
pub use types::{LanguageThresholds, LongMethodsResult, MethodMetrics, MethodType};