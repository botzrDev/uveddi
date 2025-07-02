pub mod analysis_engine;
pub mod anti_patterns;
pub mod cycle_detector;
pub mod dependency_extractor;
pub mod dependency_graph;
pub mod api_types;

use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::ast::tree_sitter::ParsedFile;
use crate::analysis::dependency_graph::LocalDependencyGraph;

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
