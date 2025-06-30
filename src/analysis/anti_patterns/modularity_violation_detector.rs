use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::analysis::dependency_graph::DependencyGraph;
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

    fn detect_graph_issues(&self, graph: &DependencyGraph, analysis_run_id: i32) -> Vec<ArchitecturalIssue> {
        // Simple community detection: group modules by top-level directory
        let mut communities: HashMap<String, HashSet<String>> = HashMap::new();
        for module in graph.get_modules() {
            let group = module.split("::").next().unwrap_or(module);
            communities.entry(group.to_string()).or_default().insert(module.clone());
        }
        // Count cross-community dependencies
        let mut cross_edges: HashMap<(String, String), usize> = HashMap::new();
        for module in graph.get_modules() {
            let from_group = module.split("::").next().unwrap_or(module);
            if let Some(deps) = graph.get_dependencies(module) {
                for dep in deps {
                    let to_group = dep.split("::").next().unwrap_or(dep);
                    if from_group != to_group {
                        *cross_edges.entry((from_group.to_string(), to_group.to_string())).or_insert(0) += 1;
                    }
                }
            }
        }
        // Flag strong cross-community dependencies
        let mut issues = Vec::new();
        for ((from_group, to_group), count) in cross_edges {
            if count >= self.cross_module_threshold {
                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: analysis_run_id as i64,
                    anti_pattern_type_id: 0, // To be set by DB
                    file_path: format!("{} -> {}", from_group, to_group),
                    start_line: None,
                    end_line: None,
                    severity: if count > self.cross_module_threshold * 2 { "high".to_string() } else { "medium".to_string() },
                    description: format!("Strong dependency ({} edges) from module group '{}' to '{}', indicating a modularity violation.", count, from_group, to_group),
                    code_snippet: None,
                    ai_explanation: None,
                });
            }
        }
        issues
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            type_id: None,
            name: "Modularity Violation".to_string(),
            description: "Strong dependencies between modules that should be independent, violating modular boundaries.".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "modularity_violation"
    }
}
