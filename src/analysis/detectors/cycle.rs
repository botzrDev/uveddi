use crate::analysis::graph::dependency::ComponentNode;
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::database::models::ArchitecturalIssue;
use log::info;
use petgraph::algo::tarjan_scc;

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
    pub fn detect_cycles(
        &self,
        graph: &LocalDependencyGraph,
        analysis_run_id: i64,
    ) -> Vec<ArchitecturalIssue> {
        let start_time = std::time::Instant::now();

        let petgraph = graph.get_petgraph();
        let sccs = tarjan_scc(&petgraph);

        let mut issues = Vec::new();

        for scc in sccs {
            if scc.len() > 1 {
                let cycle_nodes: Vec<String> = scc
                    .iter()
                    .filter_map(|&node_index| {
                        graph
                            .get_node_from_index(node_index)
                            .map(|node| match node {
                                ComponentNode::Module { path } => path.clone(),
                                ComponentNode::Class { name: _, file_path } => {
                                    file_path.clone()
                                }
                                ComponentNode::Function { name: _, file_path } => {
                                    file_path.clone()
                                }
                            })
                    })
                    .collect();

                let description = format!("Cyclic dependency detected involving: {}. This creates tight coupling and hinders maintainability.", cycle_nodes.join(", "));

                let representative_node = graph.get_node_from_index(scc[0]).unwrap(); // Safe due to scc.len() > 1
                let file_path = match representative_node {
                    ComponentNode::Module { path } => path.clone(),
                    ComponentNode::Class { file_path, .. } => file_path.clone(),
                    ComponentNode::Function { file_path, .. } => file_path.clone(),
                };
                let start_line = 1;

                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id,
                    anti_pattern_type_id: 2, // Standard ID for Cyclic Dependency
                    file_path,
                    start_line: Some(start_line),
                    end_line: Some(start_line),
                    severity: "High".to_string(),
                    description,
                    code_snippet: None,
                    ai_explanation: None,
                });
            }
        }

        info!(
            "Cycle detection completed in {:?}, found {} cycles.",
            start_time.elapsed(),
            issues.len()
        );
        issues
    }
}

impl Default for CycleDetector {
    fn default() -> Self {
        Self::new()
    }
}
