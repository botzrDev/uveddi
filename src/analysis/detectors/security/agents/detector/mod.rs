//! Detector module for multi-agent security analysis
//!
//! This module coordinates the SecurityOrchestrator and individual agent implementations
//! for comprehensive security analysis through a multi-agent architecture.

pub mod agents;
pub mod orchestrator;

// Re-export main types for convenience
pub use agents::{ConfigAnalysisAgent, DependencyAgent, TaintAnalysisAgent, ValidationAgent};
pub use orchestrator::SecurityOrchestrator;
