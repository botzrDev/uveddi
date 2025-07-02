//! Detection algorithms and analyzers
//!
//! This module contains all the detection logic for anti-patterns, architectural issues,
//! and other code quality problems. It includes both pattern-specific detectors and
//! infrastructure for dependency analysis.

pub mod cycle;
pub mod dependency;
pub mod anti_patterns;

pub use cycle::CycleDetector;
pub use dependency::{DependencyExtractor, Dependency};
