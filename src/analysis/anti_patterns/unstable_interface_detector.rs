use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::analysis::dependency_graph::DependencyGraph;
use std::collections::HashMap;

/// Detector for the Unstable Interface anti-pattern
pub struct UnstableInterfaceDetector {
    pub fan_in_threshold: usize,
}

impl UnstableInterfaceDetector {
    pub fn new() -> Self {
        Self { fan_in_threshold: 5 } // Default threshold, can be made configurable
    }

    /// Detects modules/interfaces with high fan-in using the dependency graph
    pub fn detect_in_graph(&self, graph: &DependencyGraph, analysis_run_id: i64) -> Vec<ArchitecturalIssue> {
        // Count fan-in for each module
        let mut fan_in_count: HashMap<&String, usize> = HashMap::new();
        for module in graph.get_modules() {
            fan_in_count.insert(module, 0);
        }
        for module in graph.get_modules() {
            if let Some(deps) = graph.get_dependencies(module) {
                for dep in deps {
                    *fan_in_count.entry(dep).or_insert(0) += 1;
                }
            }
        }
        // Flag modules with high fan-in
        let mut issues = Vec::new();
        for (&module, &count) in &fan_in_count {
            if count >= self.fan_in_threshold {
                let file_path = graph.get_file_path(module).map(|p| p.display().to_string()).unwrap_or_else(|| module.clone());
                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id,
                    anti_pattern_type_id: 0, // To be set by DB
                    file_path,
                    start_line: None,
                    end_line: None,
                    severity: if count > self.fan_in_threshold * 2 { "high".to_string() } else { "medium".to_string() },
                    description: format!("Module/interface '{}' has high fan-in ({} dependents). This can cause ripple effects if the interface changes.", module, count),
                    code_snippet: None,
                    ai_explanation: None,
                });
            }
        }
        issues
    }
}

impl AnalysisDetector for UnstableInterfaceDetector {
    fn detect_issues(&self, _file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // File-level detection is not used; detection is graph-based
        Ok(vec![])
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            type_id: None,
            name: "Unstable Interface".to_string(),
            description: "A module or interface with high fan-in that changes frequently, causing ripple effects.".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "unstable_interface"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
