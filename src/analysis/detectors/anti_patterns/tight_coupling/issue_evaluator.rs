use crate::analysis::graph::dependency::ComponentNode;
use crate::ast::tree_sitter_impl::SourceLanguage;
use crate::database::models::ArchitecturalIssue;
use std::collections::HashMap;
use tracing::debug;

use super::types::{CouplingMetrics, CouplingThresholds};

/// Evaluates coupling metrics and creates architectural issues
pub struct IssueEvaluator;

impl IssueEvaluator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate coupling issues based on thresholds
    pub fn evaluate_coupling_issues(
        &self,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        language: SourceLanguage,
        thresholds: &CouplingThresholds,
    ) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();

        for (component, metric) in metrics {
            // Evaluate fan-out
            if metric.fan_out >= thresholds.fan_out_critical {
                issues.push(self.create_coupling_issue(
                    component,
                    "Critical",
                    format!(
                        "Fan-out {} exceeds critical threshold {}",
                        metric.fan_out, thresholds.fan_out_critical
                    ),
                    metric,
                ));
            } else if metric.fan_out >= thresholds.fan_out_warning {
                issues.push(self.create_coupling_issue(
                    component,
                    "Warning",
                    format!(
                        "Fan-out {} exceeds warning threshold {}",
                        metric.fan_out, thresholds.fan_out_warning
                    ),
                    metric,
                ));
            }

            // Evaluate CBO
            if metric.cbo >= thresholds.cbo_critical {
                issues.push(self.create_coupling_issue(
                    component,
                    "Critical",
                    format!(
                        "CBO {} exceeds critical threshold {}",
                        metric.cbo, thresholds.cbo_critical
                    ),
                    metric,
                ));
            } else if metric.cbo >= thresholds.cbo_warning {
                issues.push(self.create_coupling_issue(
                    component,
                    "Warning",
                    format!(
                        "CBO {} exceeds warning threshold {}",
                        metric.cbo, thresholds.cbo_warning
                    ),
                    metric,
                ));
            }

            // Evaluate RFC
            if metric.rfc >= thresholds.rfc_critical {
                issues.push(self.create_coupling_issue(
                    component,
                    "Critical",
                    format!(
                        "RFC {} exceeds critical threshold {}",
                        metric.rfc, thresholds.rfc_critical
                    ),
                    metric,
                ));
            } else if metric.rfc >= thresholds.rfc_warning {
                issues.push(self.create_coupling_issue(
                    component,
                    "Warning",
                    format!(
                        "RFC {} exceeds warning threshold {}",
                        metric.rfc, thresholds.rfc_warning
                    ),
                    metric,
                ));
            }
        }

        debug!("Evaluated {} coupling issues for {:?}", issues.len(), language);
        issues
    }

    /// Create a coupling issue
    fn create_coupling_issue(
        &self,
        component: &ComponentNode,
        severity: &str,
        description: String,
        metrics: &CouplingMetrics,
    ) -> ArchitecturalIssue {
        let (file_path, component_name) = match component {
            ComponentNode::Class { name, file_path } => (file_path.clone(), name.clone()),
            ComponentNode::Function { name, file_path } => (file_path.clone(), name.clone()),
            ComponentNode::Module { path } => (path.clone(), "module".to_string()),
        };

        let mut issue = ArchitecturalIssue::new(
            0, // analysis_run_id will be set by caller
            3, // anti_pattern_type_id for tight coupling
            file_path,
            None, // line_number
            description.clone(),
            "TightCouplingDetector".to_string(),
            severity.to_string(),
            description,
        );
        issue.start_line = None;
        issue.end_line = None;
        issue.code_snippet = Some(component_name);
        issue.ai_explanation = Some(format!(
            "Reduce coupling by: 1) Using dependency injection, 2) Applying interfaces/traits, 3) Reducing direct dependencies. Current metrics: Fan-out={}, Fan-in={}, CBO={}, RFC={}",
            metrics.fan_out, metrics.fan_in, metrics.cbo, metrics.rfc
        ));
        issue
    }
}

impl Default for IssueEvaluator {
    fn default() -> Self {
        Self::new()
    }
}