pub mod engine;
pub mod types;
pub mod cache;
pub mod detectors;
pub mod graph;

#[cfg(test)]
pub mod tests;

// Re-exports for convenience
pub use engine::AnalysisEngine;
pub use cache::AstCache;
pub use detectors::{CycleDetector, DependencyExtractor, Dependency};
pub use detectors::anti_patterns::GodObjectDetector;
pub use graph::{LocalDependencyGraph, ComponentNode, LocalDependencyType};

use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::ast::tree_sitter::ParsedFile;

pub type AnalysisError = crate::error::UveddiError;

/// Core analysis trait for all detectors
pub trait AnalysisDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn detect_graph_issues(&self, _graph: &LocalDependencyGraph, _analysis_run_id: i64) -> Vec<ArchitecturalIssue> {
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
