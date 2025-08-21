//! Detection algorithms and analyzers
//!
//! This module contains all the detection logic for anti-patterns, architectural issues,
//! and other code quality problems. It includes both pattern-specific detectors and
//! infrastructure for dependency analysis.

pub mod anti_patterns;
pub mod cycle;
pub mod dependency;
pub mod security;

pub use cycle::CycleDetector;
pub use dependency::{Dependency, DependencyExtractor};
pub use security::{SecurityDetector, SecurityConfig};
