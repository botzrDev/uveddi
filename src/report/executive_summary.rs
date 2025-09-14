//! Executive Summary Generation Module
//!
//! This module provides business logic for generating executive summaries
//! and severity-based analysis summaries for architectural analysis reports.

use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::report::{ReportGenerator, ArchitecturalComponent};
use std::collections::HashSet;
use std::path::Path;

impl ReportGenerator {
    /// Generate the report header section
    pub fn generate_report_header(&self, analysis_run: &AnalysisRun) -> String {
        format!(
            "# Uveddi Architecture Analysis Report\n\n\
            **Analysis Date**: {}\n\
            **Configuration**: {}\n\
            **Run ID**: {}\n\n",
            analysis_run
                .start_time
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            "default", // analysis_run doesn't have config_name field
            analysis_run
                .run_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )
    }

    /// Generate the executive summary section
    pub fn generate_executive_summary(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        codebase_path: Option<&str>,
    ) -> String {
        let total_issues = issues.len();
        let high_severity = issues
            .iter()
            .filter(|i| {
                i.severity.to_lowercase() == "high" || i.severity.to_lowercase() == "critical"
            })
            .count();
        let medium_severity = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "medium")
            .count();
        let low_severity = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "low")
            .count();

        let unique_files = issues
            .iter()
            .map(|i| &i.file_path)
            .collect::<HashSet<_>>()
            .len();

        format!(
            "This report analyzes the codebase at `{}` and identified **{} architectural issues** across **{} files**.\n\n\
            - **High Severity**: {} issues\n\
            - **Medium Severity**: {} issues\n\
            - **Low Severity**: {} issues\n\n\
            The analysis took {:.2} seconds to complete.",
            codebase_path.unwrap_or("Unknown"),
            total_issues,
            unique_files,
            high_severity,
            medium_severity,
            low_severity,
            analysis_run.end_time.and_then(|end|
                Some((end - analysis_run.start_time).num_seconds() as f64)
            ).unwrap_or(0.0)
        )
    }

    /// Generate the severity summary section with tables
    pub fn generate_severity_summary(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut summary = String::new();

        // Group issues by severity
        let high_issues: Vec<&ArchitecturalIssue> = issues
            .iter()
            .filter(|i| {
                i.severity.to_lowercase() == "high" || i.severity.to_lowercase() == "critical"
            })
            .collect();
        let medium_issues: Vec<&ArchitecturalIssue> = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "medium")
            .collect();
        let low_issues: Vec<&ArchitecturalIssue> = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "low")
            .collect();

        // High severity table
        if !high_issues.is_empty() {
            summary.push_str("### 🔴 High Severity Issues\n\n");
            summary.push_str("| File | Issue |\n");
            summary.push_str("|------|-------|\n");

            for issue in high_issues {
                let file_path = Path::new(&issue.file_path);
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                let lines = if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                    format!(":{}-{}", start, end)
                } else {
                    String::new()
                };

                summary.push_str(&format!(
                    "| `{}{}`{} | {} |\n",
                    file_name,
                    lines,
                    if issue.file_path.len() > 40 {
                        " ..."
                    } else {
                        ""
                    },
                    issue.description.replace("|", "\\|").replace("\n", " ")
                ));
            }
            summary.push_str("\n");
        }

        // Medium severity table
        if !medium_issues.is_empty() {
            summary.push_str("### 🟡 Medium Severity Issues\n\n");
            summary.push_str("| File | Issue |\n");
            summary.push_str("|------|-------|\n");

            for issue in medium_issues {
                let file_path = Path::new(&issue.file_path);
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                let lines = if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                    format!(":{}-{}", start, end)
                } else {
                    String::new()
                };

                summary.push_str(&format!(
                    "| `{}{}`{} | {} |\n",
                    file_name,
                    lines,
                    if issue.file_path.len() > 40 {
                        " ..."
                    } else {
                        ""
                    },
                    issue.description.replace("|", "\\|").replace("\n", " ")
                ));
            }
            summary.push_str("\n");
        }

        // Low severity table
        if !low_issues.is_empty() {
            summary.push_str("### 🔵 Low Severity Issues\n\n");
            summary.push_str("| File | Issue |\n");
            summary.push_str("|------|-------|\n");

            for issue in low_issues {
                let file_path = Path::new(&issue.file_path);
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                let lines = if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                    format!(":{}-{}", start, end)
                } else {
                    String::new()
                };

                summary.push_str(&format!(
                    "| `{}{}`{} | {} |\n",
                    file_name,
                    lines,
                    if issue.file_path.len() > 40 {
                        " ..."
                    } else {
                        ""
                    },
                    issue.description.replace("|", "\\|").replace("\n", " ")
                ));
            }
            summary.push_str("\n");
        }

        summary
    }

    /// Create enhanced summary with component and diagram information
    pub fn create_enhanced_summary(
        &self,
        issues: &[ArchitecturalIssue],
        components: Option<&[ArchitecturalComponent]>,
        diagrams: &[crate::report::interactive_models::DiagramDefinition],
    ) -> serde_json::Value {
        let mut severity_counts = std::collections::HashMap::new();
        for issue in issues {
            *severity_counts.entry(&issue.severity).or_insert(0) += 1;
        }

        let component_summary = if let Some(comps) = components {
            let mut type_counts = std::collections::HashMap::new();
            for comp in comps {
                *type_counts.entry(&comp.component_type).or_insert(0) += 1;
            }

            serde_json::json!({
                "total_components": comps.len(),
                "by_type": type_counts,
                "avg_dependencies": comps.iter()
                    .map(|c| c.dependencies.len())
                    .sum::<usize>() as f64 / comps.len() as f64
            })
        } else {
            serde_json::json!(null)
        };

        serde_json::json!({
            "issues": {
                "total": issues.len(),
                "by_severity": severity_counts,
                "unique_files": issues.iter()
                    .map(|i| &i.file_path)
                    .collect::<HashSet<_>>()
                    .len()
            },
            "components": component_summary,
            "diagrams": {
                "total": diagrams.len(),
                "types": diagrams.iter()
                    .map(|d| &d.diagram_type)
                    .collect::<HashSet<_>>()
                    .len()
            }
        })
    }
}