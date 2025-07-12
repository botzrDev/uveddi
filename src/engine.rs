//! Engine module for Uveddi
//!
//! This module provides the main orchestration logic for codebase analysis.
//! Currently delegated to AnalysisEngine in the analysis module.

// Re-export the main analysis engine
pub use crate::analysis::AnalysisEngine as Engine;