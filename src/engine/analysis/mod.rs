//! # Analysis Module
//!
//! Contains analysis pipeline logic, visitor patterns, and analysis context.
//! Separated from parsing to maintain clean architectural boundaries.

pub mod context;
pub mod detector_factory;
pub mod performance;
pub mod pipeline;
pub mod visitor;

// Re-export main types
pub use context::AnalysisContext;
pub use detector_factory::{ContextDetectorFactory, DetectorMigrationStatus};
pub use performance::{AnalysisInstrumentation, AnalysisMetrics, PerformanceTimer};
pub use pipeline::AnalysisPipeline;
pub use visitor::{AstVisitor, VisitResult};
