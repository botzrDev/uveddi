use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use crate::database::models::ArchitecturalIssue;
use crate::core::logging::info;
use petgraph::algo::tarjan_scc;

/// A detector for identifying cyclic dependencies between software components.
///
/// This detector analyzes a `LocalDependencyGraph` to find strongly connected components (SCCs),
/// which represent cycles in the dependency structure. Cycles can indicate design problems
/// such as tight coupling and poor modularization, making the codebase harder to understand,
/// maintain, and test.
///
/// # Usage
///
/// The `CycleDetector` is typically invoked by the analysis engine, which passes a
/// dependency graph for analysis.
///
/// ```rust,ignore
/// use uveddi::analysis::detectors::cycle::CycleDetector;
/// use uveddi::analysis::graph::dependency::LocalDependencyGraph;
///
/// // Assuming `graph` is a fully constructed LocalDependencyGraph
/// let detector = CycleDetector::new();
/// let cycle_issues = detector.detect_cycles(&graph, 1);
///
/// for issue in cycle_issues {
///     println!("Found cycle: {}", issue.description);
/// }
/// ```
pub struct CycleDetector;

impl CycleDetector {
    /// Creates a new instance of the `CycleDetector`.
    ///
    /// # Returns
    ///
    /// A `CycleDetector` instance ready to detect cycles.
    pub fn new() -> Self {
        Self
    }

    /// Detects all cyclic dependencies in the given `LocalDependencyGraph`.
    ///
    /// This method implements Tarjan's algorithm for finding strongly connected components (SCCs)
    /// to efficiently identify all cycles. An SCC with more than one node is considered a
    /// dependency cycle. For each component in a detected cycle, an `ArchitecturalIssue` is created.
    ///
    /// # Arguments
    ///
    /// * `graph` - A reference to the `LocalDependencyGraph` to be analyzed.
    /// * `analysis_run_id` - The ID of the current analysis run, used to associate the findings.
    ///
    /// # Returns
    ///
    /// A `Vec<ArchitecturalIssue>` containing all the cyclic dependency issues found. Each issue
    /// corresponds to a component involved in a cycle.
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

                issues.push(ArchitecturalIssue::new(
                    analysis_run_id,
                    1, // anti_pattern_type_id: Assuming 1 is cyclic dependency type
                    file_path,
                    Some(1), // line_number: TODO: Extract actual line numbers from AST
                    description.clone(),
                    "CycleDetector".to_string(),
                    "high".to_string(), // Cyclic dependencies are typically high severity
                    description.clone(),
                ));
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

/// Provides a default constructor for `CycleDetector`.
impl Default for CycleDetector {
    /// Creates a new `CycleDetector` with default settings.
    fn default() -> Self {
        Self::new()
    }
}
