use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::analysis::dependency_graph::{ComponentNode, DependencyGraph};
use std::collections::{HashMap, HashSet};

/// Detector for the Modularity Violation anti-pattern
pub struct ModularityViolationDetector {
    pub cross_module_threshold: usize,
}

impl Default for ModularityViolationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ModularityViolationDetector {
    pub fn new() -> Self {
        Self { cross_module_threshold: 3 } // Default threshold, can be made configurable
    }
}

impl AnalysisDetector for ModularityViolationDetector {
    fn detect_issues(&self, _file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // File-level detection is not used; detection is graph-based
        Ok(vec![])
    }

    fn detect_graph_issues(&self, graph: &DependencyGraph, analysis_run_id: i64) -> Vec<ArchitecturalIssue> {
        let petgraph = graph.get_petgraph();
        let mut communities: HashMap<String, HashSet<String>> = HashMap::new();

        // Simple community detection: group modules by top-level directory
        for node_index in petgraph.node_indices() {
            if let Some(ComponentNode::Module { path }) = graph.get_node_from_index(node_index) {
                let community_name = path.split('/').next().unwrap_or(path).to_string();
                communities.entry(community_name).or_default().insert(path.clone());
            }
        }

        let mut cross_community_edges: HashMap<(String, String), usize> = HashMap::new();
        for edge_index in petgraph.edge_indices() {
            if let Some(edge) = petgraph.edge_endpoints(edge_index) {
                if let (Some(ComponentNode::Module { path: from_path }), Some(ComponentNode::Module { path: to_path })) = 
                    (graph.get_node_from_index(edge.0), graph.get_node_from_index(edge.1)) {
                    
                    let from_community = from_path.split('/').next().unwrap_or(from_path).to_string();
                    let to_community = to_path.split('/').next().unwrap_or(to_path).to_string();

                    if from_community != to_community {
                        let key = (from_community, to_community);
                        *cross_community_edges.entry(key).or_default() += 1;
                    }
                }
            }
        }

        let mut issues = Vec::new();
        for ((from, to), count) in cross_community_edges {
            if count >= self.cross_module_threshold {
                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id,
                    anti_pattern_type_id: 4, // Standard ID for Modularity Violation
                    file_path: format!("Cross-module dependency: {} -> {}", from, to),
                    start_line: Some(1),
                    end_line: Some(1),
                    severity: if count > self.cross_module_threshold * 2 { "High".to_string() } else { "Medium".to_string() },
                    description: format!("Modularity Violation: Found {} dependencies from module group '{}' to '{}'. This suggests they are too tightly coupled.", count, from, to),
                    code_snippet: None,
                    ai_explanation: None,
                });
            }
        }
        issues
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: None,
            name: "Modularity Violation".to_string(),
            description: "Strong dependencies between modules that should be independent, violating modular boundaries.".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "modularity_violation"
    }
}
