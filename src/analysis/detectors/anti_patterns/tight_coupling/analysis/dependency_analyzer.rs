use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

use super::super::{
    types::{Dependency, DependencyStrength},
    language_support::LanguageAnalyzer,
};

/// Analyzes dependencies between components for coupling detection
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Extract dependencies from a single file using language-specific analyzer
    pub fn extract_file_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
        analyzer: &dyn LanguageAnalyzer,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        debug!("Extracting dependencies from {}", file_path.display());
        analyzer.extract_dependencies(file_path, parsed_file)
    }

    /// Build dependency graph from multiple files with parallel processing
    pub fn build_project_dependency_graph(
        &self,
        files: &[(String, ParsedFile)],
        analyzers: &HashMap<crate::ast::tree_sitter_impl::SourceLanguage, Box<dyn LanguageAnalyzer>>,
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        info!("Building dependency graph for {} files", files.len());

        // Extract dependencies in parallel
        let all_dependencies: Result<Vec<Vec<Dependency>>, AnalysisError> = files
            .iter()
            .map(|(file_path, parsed_file)| {
                if let Some(analyzer) = analyzers.get(&parsed_file.language) {
                    self.extract_file_dependencies(Path::new(file_path), parsed_file, analyzer.as_ref())
                } else {
                    Ok(Vec::new())
                }
            })
            .collect();

        let dependencies_list = all_dependencies?;

        // Build graph sequentially (graph modifications need to be sequential)
        let mut graph = LocalDependencyGraph::new();
        for dependencies in dependencies_list {
            for dependency in dependencies {
                graph.add_dependency(
                    &dependency.from_component,
                    &dependency.to_component,
                    dependency.dependency_type,
                );
            }
        }

        info!(
            "Built dependency graph with {} nodes",
            graph.get_petgraph().node_count()
        );
        Ok(graph)
    }

    /// Build dependency graph incrementally for changed files only
    pub fn build_incremental_dependency_graph(
        &self,
        changed_files: &[(String, ParsedFile)],
        _existing_graph: &LocalDependencyGraph,
        analyzers: &HashMap<crate::ast::tree_sitter_impl::SourceLanguage, Box<dyn LanguageAnalyzer>>,
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        info!(
            "Building incremental dependency graph for {} changed files",
            changed_files.len()
        );

        // For now, rebuild completely - in future could optimize to only update affected nodes
        // This would require tracking file->component mappings and dependency provenance
        self.build_project_dependency_graph(changed_files, analyzers)
    }

    /// Detect circular dependencies in the graph
    pub fn detect_circular_dependencies(&self, graph: &LocalDependencyGraph) -> Vec<Vec<ComponentNode>> {
        let mut cycles = Vec::new();
        let petgraph = graph.get_petgraph();

        // Use simple DFS-based cycle detection
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                if !visited.contains(&node_index) {
                    let mut path = Vec::new();
                    if self.dfs_cycle_detection(graph, node_index, &mut visited, &mut rec_stack, &mut path) {
                        cycles.push(path);
                    }
                }
            }
        }

        cycles
    }

    fn dfs_cycle_detection(
        &self,
        graph: &LocalDependencyGraph,
        node_index: petgraph::graph::NodeIndex,
        visited: &mut std::collections::HashSet<petgraph::graph::NodeIndex>,
        rec_stack: &mut std::collections::HashSet<petgraph::graph::NodeIndex>,
        path: &mut Vec<ComponentNode>,
    ) -> bool {
        visited.insert(node_index);
        rec_stack.insert(node_index);

        if let Some(component) = graph.get_node_from_index(node_index) {
            path.push(component.clone());
        }

        let petgraph = graph.get_petgraph();
        for edge in petgraph.edges(node_index) {
            let target_index = edge.target();

            if !visited.contains(&target_index) {
                if self.dfs_cycle_detection(graph, target_index, visited, rec_stack, path) {
                    return true;
                }
            } else if rec_stack.contains(&target_index) {
                // Found a cycle
                return true;
            }
        }

        rec_stack.remove(&node_index);
        path.pop();
        false
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}