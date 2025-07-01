use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::analysis::dependency_graph::{ComponentNode, DependencyGraph};
use std::collections::HashMap;

/// Detector for the Unstable Interface anti-pattern
pub struct UnstableInterfaceDetector {
    pub fan_in_threshold: usize,
}

impl Default for UnstableInterfaceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl UnstableInterfaceDetector {
    pub fn new() -> Self {
        Self { fan_in_threshold: 5 } // Default threshold, can be made configurable
    }
}

impl AnalysisDetector for UnstableInterfaceDetector {
    fn detect_issues(&self, _file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // File-level detection is not used; detection is graph-based
        Ok(vec![])
    }

    fn detect_graph_issues(&self, graph: &DependencyGraph, analysis_run_id: i64) -> Vec<ArchitecturalIssue> {
        let petgraph = graph.get_petgraph();
        let mut fan_in_count: HashMap<String, usize> = HashMap::new();

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                if let ComponentNode::Module { path } = component {
                    fan_in_count.entry(path.clone()).or_insert(0);
                }
            }
        }

        for edge_index in petgraph.edge_indices() {
            if let Some(edge) = petgraph.edge_endpoints(edge_index) {
                if let Some(target_component) = graph.get_node_from_index(edge.1) {
                     if let ComponentNode::Module { path } = target_component {
                        *fan_in_count.entry(path.clone()).or_default() += 1;
                    }
                }
            }
        }

        let mut issues = Vec::new();
        for (module, &count) in &fan_in_count {
            if count >= self.fan_in_threshold {
                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id,
                    anti_pattern_type_id: 3, // Standard ID for Unstable Interface
                    file_path: module.clone(),
                    start_line: Some(1),
                    end_line: Some(1),
                    severity: if count > self.fan_in_threshold * 2 { "High".to_string() } else { "Medium".to_string() },
                    description: format!("Unstable Interface: Module '{}' has a high fan-in of {}. Changes to this module could have a widespread impact.", module, count),
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
            name: "Unstable Interface".to_string(),
            description: "A module or interface with high fan-in that changes frequently, causing ripple effects.".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "unstable_interface"
    }
}
