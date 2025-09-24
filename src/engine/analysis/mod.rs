//! # Analysis Module
//!
//! Contains analysis pipeline logic, visitor patterns, and analysis context.
//! Separated from parsing to maintain clean architectural boundaries.

pub mod context;
pub mod pipeline;
pub mod visitor;

// Re-export main types
pub use context::AnalysisContext;
pub use pipeline::AnalysisPipeline;
pub use visitor::{AstVisitor, VisitResult};