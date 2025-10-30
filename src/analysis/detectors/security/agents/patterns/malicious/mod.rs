//! Malicious pattern detection module
//!
//! This module provides comprehensive malicious pattern detection capabilities
//! including signature definitions, threat assessment, and behavior analysis.

pub mod signatures;
pub mod behaviors;

// Re-export key types and functions
pub use signatures::{MaliciousPattern, MaliciousPatternType, ThreatLevel, load_default_patterns};
pub use behaviors::MaliciousPatternDatabase;