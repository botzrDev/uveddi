//! Detects cyclic dependencies in the `DependencyGraph`.

use crate::analysis::dependency_graph::{ComponentNode, DependencyGraph};
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use petgraph::algo::tarjan_scc;
use log::info;

/// A detector for identifying cyclic dependencies between components.
pub struct CycleDetector;

impl CycleDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detects all cycles in the given `DependencyGraph`.
    ///
    /// This method uses Tarjan's algorithm for finding strongly connected components (SCCs)
    /// to efficiently identify all cycles. Any SCC with more than one node represents
    /// a cycle.
    ///
    /// # Arguments
    ///
    /// * `graph` - A reference to the `DependencyGraph` to be analyzed.
    /// * `analysis_run_id` - The ID of the current analysis run for associating the issues.
    ///
    /// # Returns
    ///
    /// A `Vec<ArchitecturalIssue>` containing all the cyclic dependency issues found.
    pub fn detect_cycles(&self, graph: &DependencyGraph, analysis_run_id: i32) -> Vec<ArchitecturalIssue> {
        let start_time = std::time::Instant::now();
        
        // `tarjan_scc` returns a list of strongly connected components.
        // Each component is a Vec of NodeIndices.
        let sccs = tarjan_scc(&graph.graph);

        let mut issues = Vec::new();

        for scc in sccs {
            // A strongly connected component with more than one node is a cycle.
            if scc.len() > 1 {
                let cycle_nodes: Vec<String> = scc.iter()
                    .map(|&node_index| {
                        // Safely access the node data from the graph.
                        let component = &graph.graph[node_index];
                        match component {
                            ComponentNode::Module { path } => path.clone(),
                            ComponentNode::Class { name, file_path } => format!("Class({}@{})", name, file_path),
                            ComponentNode::Function { name, file_path } => format!("Function({}@{})", name, file_path),
                        }
                    })
                    .collect();

                let description = format!("A cyclic dependency was detected involving the following components: {}. This creates tight coupling and hinders maintainability.", cycle_nodes.join(", "));
                
                // For simplicity, we'll associate the issue with the first component in the cycle.
                let representative_node = &graph.graph[scc[0]];
                let (file_path, start_line) = match representative_node {
                     ComponentNode::Module { path } => (path.clone(), 0),
                     ComponentNode::Class { file_path, .. } => (file_path.clone(), 0), // Line number could be improved
                     ComponentNode::Function { file_path, .. } => (file_path.clone(), 0), // Line number could be improved
                };

                issues.push(ArchitecturalIssue {
                    id: 0, // Will be set by the database
                    analysis_run_id,
                    detector_name: "CycleDetector".to_string(),
                    issue_type: AntiPatternType::CyclicDependency,
                    file_path,
                    line_number: Some(start_line as i32),
                    description,
                    suggestion: Some("Break the cycle by inverting dependencies, using interfaces, or extracting a new component.".to_string()),
                    severity: "High".to_string(),
                    remediation_cost: 5, // Example cost
                    created_at: chrono::Utc::now().naive_utc(),
                });
            }
        }

        info!("Cycle detection completed in {:?}, found {} cycles.", start_time.elapsed(), issues.len());
        issues
    }
}

impl Default for CycleDetector {
    fn default() -> Self {
        Self::new()
    }
}
