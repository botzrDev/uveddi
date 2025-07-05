//! # Analysis Module
//!
//! The analysis module provides a comprehensive framework for detecting anti-patterns,
//! architectural issues, and code quality problems across multiple programming languages.
//!
//! ## Architecture Overview
//!
//! The analysis system is built around a trait-based architecture that allows for
//! pluggable detectors. Each detector implements the `AnalysisDetector` trait and
//! can operate on either individual files (via AST analysis) or the entire
//! dependency graph.
//!
//! ### Key Components:
//!
//! - **AnalysisEngine**: Core orchestrator that runs detectors and collects results
//! - **AnalysisDetector**: Trait that all detectors must implement
//! - **Anti-pattern Detectors**: Specialized detectors for various code quality issues
//! - **Dependency Graph**: Graph-based analysis for architectural patterns
//! - **AST Cache**: Caching layer for parsed syntax trees
//!
//! ### Pluggable Detector System
//!
//! The detector system is designed to be modular and extensible:
//!
//! 1. **File-level Analysis**: Detectors operate on individual `ParsedFile` instances
//! 2. **Graph-level Analysis**: Detectors can analyze the entire dependency graph
//! 3. **Language-specific Support**: Detectors can be specialized for specific languages
//! 4. **Multi-language Support**: Some detectors work across multiple languages
//!
//! ### Integration with Analysis Engine
//!
//! Detectors integrate with the engine through:
//!
//! - **Registration**: Detectors are registered with the engine
//! - **Execution**: Engine calls `detect_issues()` for each file
//! - **Result Collection**: Engine aggregates results from all detectors
//! - **Reporting**: Results are stored in the database for analysis
//!
/// ## Usage Example
///
/// ```rust
/// use uveddi::analysis::{AnalysisEngine, AnalysisDetector};
/// use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
/// use std::path::Path;
///
/// let engine = AnalysisEngine::new().unwrap();
/// let detector = GodObjectDetector::new(15, 20);
/// // Detectors are built into the engine
/// // let (issues, graph) = engine.analyze(Path::new("src/")).await.unwrap();
/// // println!("Found {} issues", issues.len());
/// ```
pub mod cache;
pub mod detectors;
pub mod engine;
pub mod graph;
pub mod types;

#[cfg(test)]
pub mod tests;

// Re-exports for convenience
pub use cache::AstCache;
pub use detectors::anti_patterns::GodObjectDetector;
pub use detectors::{CycleDetector, Dependency, DependencyExtractor};
pub use engine::AnalysisEngine;
pub use graph::{ComponentNode, LocalDependencyGraph, LocalDependencyType};

use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};

pub type AnalysisError = crate::error::UveddiError;

/// Core analysis trait for all detectors
///
/// The `AnalysisDetector` trait defines the interface that all analysis detectors
/// must implement. This trait enables a pluggable architecture where different
/// detectors can be easily added, removed, or replaced without affecting the
/// core analysis engine.
///
/// ## Required Methods
///
/// - `detect_issues()`: Analyze a single parsed file for issues
/// - `get_anti_pattern_types()`: Return the types of anti-patterns this detector finds
/// - `get_detector_name()`: Return a unique identifier for this detector
///
/// ## Optional Methods
///
/// - `detect_graph_issues()`: Analyze the entire dependency graph (default: no-op)
/// - `detect()`: Unified detection method for backward compatibility
///
/// ## Implementation Guidelines
///
/// 1. **Performance**: Detectors should be efficient as they run on every file
/// 2. **Language Support**: Use `ParsedFile.language` to provide language-specific logic
/// 3. **Error Handling**: Return `AnalysisError` for serious issues, empty Vec for no findings
/// 4. **Caching**: Leverage the AST cache for expensive parsing operations
///
/// # Example
/// ```rust
/// use uveddi::analysis::{AnalysisEngine, AnalysisDetector};
/// use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
/// use std::path::Path;
///
/// let engine = AnalysisEngine::new().unwrap();
/// let detector = GodObjectDetector::new(15, 20);
/// // Detectors are built into the engine
/// // let (issues, graph) = engine.analyze(Path::new("src/")).await.unwrap();
/// // println!("Found {} issues", issues.len());
/// # fn main() {}
/// ```
pub trait AnalysisDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn detect_graph_issues(
        &self,
        _graph: &LocalDependencyGraph,
        _analysis_run_id: i64,
    ) -> Vec<ArchitecturalIssue> {
        // Default implementation for detectors that don't analyze the graph
        vec![]
    }
    fn detect(&self, graph: &LocalDependencyGraph) -> Vec<ArchitecturalIssue> {
        // Unified detection method for pluggable system
        self.detect_graph_issues(graph, 0i64)
    }
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    fn get_detector_name(&self) -> &'static str;
}
