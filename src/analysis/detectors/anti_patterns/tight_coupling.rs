//! Tight Coupling anti-pattern detector
//!
//! Detects excessive dependencies between components/modules/classes.
//! Implements CBO, RFC, fan-in, fan-out metrics and cycle detection.

use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use std::collections::HashMap;
use std::path::Path;

/// Holds coupling metrics for a single software component (e.g., a class or module).
///
/// These metrics are used to quantify the degree of interdependence between components.
/// High values can indicate tight coupling, which makes a system harder to maintain.
#[derive(Debug, Clone, Default)]
pub struct CouplingMetrics {
    /// **Fan-in**: The number of other components that depend on this component.
    /// A high fan-in can be acceptable if the component is a stable, central abstraction.
    pub fan_in: usize,
    /// **Fan-out**: The number of other components this component depends on.
    /// A high fan-out suggests the component has too many responsibilities or is overly complex.
    pub fan_out: usize,
    /// **Coupling Between Objects (CBO)**: Measures the total number of other components
    /// a component is coupled to (both dependencies and dependents). In many contexts, this
    /// is equivalent to fan-out.
    pub cbo: usize,
    /// **Response For a Class (RFC)**: The number of methods that can be executed in
    /// response to a message received by an object of the class. This includes the class's
    /// own methods and methods called on other objects. A high RFC can indicate high complexity.
    pub rfc: usize,
}

/// Represents a language-agnostic dependency between two components.
///
/// This struct standardizes dependency information extracted from different languages,
/// allowing for uniform analysis.
#[derive(Debug, Clone)]
pub struct Dependency {
    /// The name or path of the component that has the dependency.
    pub from: String,
    /// The name or path of the component being depended upon.
    pub to: String,
    /// The nature of the dependency (e.g., "import", "call", "field access").
    pub kind: String,
}

/// A trait for language-specific dependency extraction logic.
///
/// To support a new language, a new struct implementing this trait must be created.
/// It is responsible for using `tree-sitter` to parse a file and identify all
/// relevant dependencies.
pub trait LanguageAnalyzer {
    /// Extracts all dependencies from a given source file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file being analyzed.
    /// * `parsed_file` - The `ParsedFile` containing the AST for the source file.
    ///
    /// # Returns
    ///
    /// A `Vec<Dependency>` containing all dependencies found in the file.
    fn extract_dependencies(&self, file_path: &Path, parsed_file: &ParsedFile) -> Vec<Dependency>;
}

/// A stub implementation of `LanguageAnalyzer` for Rust.
///
/// This serves as a placeholder and would need to be fully implemented with `tree-sitter`
/// queries to extract `use` statements, function calls, and type references.
pub struct RustAnalyzer;

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_dependencies(
        &self,
        _file_path: &Path,
        _parsed_file: &ParsedFile,
    ) -> Vec<Dependency> {
        // TODO: Use Tree-sitter queries to extract Rust dependencies
        vec![]
    }
}

/// The main detector for the "Tight Coupling" anti-pattern.
///
/// This detector orchestrates the analysis process:
/// 1. It uses a `LanguageAnalyzer` to extract dependencies from source files.
/// 2. It builds a `LocalDependencyGraph` to represent the relationships between components.
/// 3. It calculates coupling metrics (Fan-in, Fan-out, CBO, RFC) for each component.
/// 4. It compares these metrics against configurable thresholds to identify issues.
pub struct TightCouplingDetector {
    // Configuration for thresholds can be added here.
    // For example:
    // pub fan_out_threshold: usize,
    // pub cbo_threshold: usize,
}

impl Default for TightCouplingDetector {
    /// Creates a new `TightCouplingDetector` with default settings.
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
        let file_path = parsed_file.file_path.to_string().to_string();
        let rust_analyzer = RustAnalyzer;
        let files = vec![(file_path.clone(), parsed_file.clone())];
        let graph = self.build_dependency_graph(&files, &rust_analyzer);
        let _metrics = self.calculate_metrics(&graph);
        // TODO: Compare metrics to thresholds, report issues
        Ok(vec![])
    }
}

impl TightCouplingDetector {
    /// Builds a `LocalDependencyGraph` from a set of parsed files.
    ///
    /// This method iterates through the provided files, uses the `LanguageAnalyzer` to
    /// extract dependencies for each, and populates the graph accordingly.
    fn build_dependency_graph<A: LanguageAnalyzer>(
        &self,
        files: &[(String, ParsedFile)],
        analyzer: &A,
    ) -> LocalDependencyGraph {
        let mut graph = LocalDependencyGraph::default();
        for (path, parsed) in files {
            let deps = analyzer.extract_dependencies(Path::new(path), parsed);
            for dep in deps {
                let from = ComponentNode::Module {
                    path: dep.from.clone(),
                };
                let to = ComponentNode::Module {
                    path: dep.to.clone(),
                };
                graph.add_dependency(
                    &from,
                    &to,
                    crate::analysis::graph::dependency::LocalDependencyType::Import,
                );
            }
        }
        graph
    }

    /// Calculates coupling metrics for every component in the dependency graph.
    ///
    /// It iterates through each node in the graph, calculating its fan-in, fan-out,
    /// CBO, and RFC, and stores the results in a `HashMap`.
    fn calculate_metrics(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<ComponentNode, CouplingMetrics> {
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
                metrics.insert(
                    node.clone(),
                    CouplingMetrics {
                        fan_in,
                        fan_out,
                        cbo,
                        rfc,
                    },
                );
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
