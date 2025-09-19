//! Detector module for multi-agent security analysis
//!
//! This module coordinates the SecurityOrchestrator and individual agent implementations
//! for comprehensive security analysis through a multi-agent architecture.

pub mod orchestrator;
pub mod agents;

// Re-export main types for convenience
pub use orchestrator::SecurityOrchestrator;
pub use agents::{TaintAnalysisAgent, ConfigAnalysisAgent, DependencyAgent, ValidationAgent};