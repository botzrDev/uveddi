//! Graph analysis and dependency modeling
//!
//! This module provides graph-based analysis capabilities for understanding
//! code structure, dependencies, and architectural patterns.

pub mod dependency;

pub use dependency::{ComponentNode, LocalDependencyGraph, LocalDependencyType};
