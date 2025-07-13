//! Tight Coupling Anti-Pattern Detector
//!
//! This module detects tight coupling between components, which makes code
//! difficult to maintain, test, and modify. Tight coupling occurs when
//! components are overly dependent on each other's internal implementation details.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::ast::tree_sitter_impl::ParsedFile;
use serde::{Deserialize, Serialize};

/// Detector for tight coupling anti-patterns
#[derive(Debug, Clone)]
pub struct TightCouplingDetector;

impl Default for TightCouplingDetector {
    fn default() -> Self {
        Self
    }
}

impl TightCouplingDetector {
    /// Creates a new tight coupling detector
    pub fn new() -> Self {
        Self
    }
}

impl AnalysisDetector for TightCouplingDetector {
    fn detect_issues(&self, _file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Tight coupling detection requires graph-level analysis
        // Individual file analysis is not meaningful for this detector
        Ok(vec![])
    }

    fn detect_graph_issues(
        &self,
        _graph: &LocalDependencyGraph,
        _analysis_run_id: i64,
    ) -> Vec<ArchitecturalIssue> {
        // TODO: Implement graph-level tight coupling detection
        // This would analyze the dependency graph to find tightly coupled components
        vec![]
    }

    fn get_detector_name(&self) -> &'static str {
        "TightCouplingDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: None,
            name: "Tight Coupling".to_string(),
            description: "Components that are overly dependent on each other's internal implementation details".to_string(),
            category: "structural".to_string(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_name() {
        let detector = TightCouplingDetector::default();
        assert_eq!(detector.get_detector_name(), "TightCouplingDetector");
    }

    #[test]
    fn test_anti_pattern_types() {
        let detector = TightCouplingDetector::default();
        let types = detector.get_anti_pattern_types();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "Tight Coupling");
    }
}