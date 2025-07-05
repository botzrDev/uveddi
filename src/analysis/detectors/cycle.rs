use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
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

        // Filter out SCCs with only one node (not a cycle)
        let cycles: Vec<_> = sccs.into_iter().filter(|scc| scc.len() > 1).collect();

        for (_cycle_id, cycle) in cycles.iter().enumerate() {
            // Get component names for the cycle description
            let component_names: Vec<String> = cycle
                .iter()
                .map(|&node_idx| {
                    let node_ref = &petgraph[node_idx];
                    match node_ref {
                        ComponentNode::Module { path } => path.clone(),
                        ComponentNode::Class { name, .. } => name.clone(),
                        ComponentNode::Function { name, .. } => name.clone(),
                    }
                })
                .collect();

            // Create the cycle description
            let description = format!(
                "Cyclic dependency detected between components: {}",
                component_names.join(" → ")
            );

            // Create an architectural issue for each component in the cycle
            for &node_idx in cycle {
                let node_ref = &petgraph[node_idx];

                // Find the file path of the component
                let file_path = match node_ref {
                    ComponentNode::Module { path } => path.clone(),
                    ComponentNode::Class { file_path, .. } => file_path.clone(),
                    ComponentNode::Function { file_path, .. } => file_path.clone(),
                };

                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id,
                    anti_pattern_type_id: 1, // Assuming 1 is cyclic dependency type
                    file_path,
                    start_line: Some(1), // TODO: Extract actual line numbers from AST
                    end_line: Some(1),
                    severity: "high".to_string(), // Cyclic dependencies are typically high severity
                    description: description.clone(),
                    code_snippet: None,   // Will be populated separately if needed
                    ai_explanation: None, // Will be populated by AI engine
                });
            }
        }

        let elapsed = start_time.elapsed();
        info!(
            "Cycle detection completed in {:.2?}. Found {} cycles with {} total issues.",
            elapsed,
            cycles.len(),
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
