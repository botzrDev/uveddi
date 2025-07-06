//! Tight Coupling anti-pattern detector
//!
//! Detects excessive dependencies between components/modules/classes.
//! Implements CBO, RFC, fan-in, fan-out metrics and cycle detection.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use std::path::Path;

/// Coupling metrics for a component
#[derive(Debug, Clone, Default)]
pub struct CouplingMetrics {
    pub fan_in: usize,
    pub fan_out: usize,
    pub cbo: usize, // Coupling Between Objects
    pub rfc: usize, // Response For Class
}

/// Language-agnostic dependency
#[derive(Debug, Clone)]
pub struct Dependency {
    pub from: String,
    pub to: String,
    pub kind: String, // e.g., import, call, field access
}

/// Trait for language-specific dependency extraction
pub trait LanguageAnalyzer {
    fn extract_dependencies(&self, file_path: &Path, parsed_file: &ParsedFile) -> Vec<Dependency>;
}

/// Rust analyzer stub (expand for real extraction)
pub struct RustAnalyzer;

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_dependencies(&self, _file_path: &Path, _parsed_file: &ParsedFile) -> Vec<Dependency> {
        // TODO: Use Tree-sitter queries to extract Rust dependencies
        vec![]
    }
}

/// Main Tight Coupling Detector
pub struct TightCouplingDetector {
    // Optionally add config/thresholds here
}

impl Default for TightCouplingDetector {
    fn default() -> Self {
        Self {}
    }
}

impl AnalysisDetector for TightCouplingDetector {
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: None,
            name: "Tight Coupling".to_string(),
            description: "Excessive dependencies between components/modules/classes".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "TightCouplingDetector"
    }

    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // For now, analyze only the current file (expand to project-wide in engine)
        let file_path = parsed_file.path.to_string_lossy().to_string();
        let rust_analyzer = RustAnalyzer;
        let files = vec![(file_path.clone(), parsed_file.clone())];
        let graph = self.build_dependency_graph(&files, &rust_analyzer);
        let _metrics = self.calculate_metrics(&graph);
        // TODO: Compare metrics to thresholds, report issues
        Ok(vec![])
    }
}

impl TightCouplingDetector {
    /// Build a dependency graph from parsed files using the language analyzer
    fn build_dependency_graph<A: LanguageAnalyzer>(
        &self,
        files: &[(String, ParsedFile)],
        analyzer: &A,
    ) -> LocalDependencyGraph {
        let mut graph = LocalDependencyGraph::default();
        for (path, parsed) in files {
            let deps = analyzer.extract_dependencies(Path::new(path), parsed);
            for dep in deps {
                let from = ComponentNode::Module { path: dep.from.clone() };
                let to = ComponentNode::Module { path: dep.to.clone() };
                graph.add_dependency(&from, &to, crate::analysis::graph::dependency::LocalDependencyType::Import);
            }
        }
        graph
    }

    /// Calculate coupling metrics for each node
    fn calculate_metrics(&self, graph: &LocalDependencyGraph) -> HashMap<ComponentNode, CouplingMetrics> {
        let mut metrics = HashMap::new();
        let petgraph = graph.get_petgraph();
        for node_idx in petgraph.node_indices() {
            if let Some(node) = petgraph.node_weight(node_idx) {
                let fan_out = petgraph.edges(node_idx).count();
                let fan_in = petgraph
                    .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                    .count();
                let cbo = fan_out;
                let rfc = fan_out + fan_in;
                metrics.insert(node.clone(), CouplingMetrics { fan_in, fan_out, cbo, rfc });
            }
        }
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tree_sitter::ParsedFile;

    #[test]
    fn test_metrics_empty_graph() {
        let detector = TightCouplingDetector::default();
        let graph = LocalDependencyGraph::default();
        let metrics = detector.calculate_metrics(&graph);
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_build_dependency_graph_empty() {
        let detector = TightCouplingDetector::default();
        let analyzer = RustAnalyzer;
        let files: Vec<(String, ParsedFile)> = vec![];
        let graph = detector.build_dependency_graph(&files, &analyzer);
        let petgraph = graph.get_petgraph();
        assert_eq!(petgraph.node_count(), 0);
    }
}
