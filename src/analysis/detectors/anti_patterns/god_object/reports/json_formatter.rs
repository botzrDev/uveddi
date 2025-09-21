//! JSON formatting for God Object reports

use super::utils::calculate_severity_distribution;
use crate::database::models::ArchitecturalIssue;
use serde_json::{json, Value};

/// JSON formatter for God Object reports
pub struct JsonFormatter;

impl JsonFormatter {
    /// Format issues as JSON
    pub fn format(issues: &[ArchitecturalIssue]) -> Result<String, serde_json::Error> {
        let report = json!({
            "detector": "GodObjectDetector",
            "total_issues": issues.len(),
            "severity_distribution": calculate_severity_distribution(issues),
            "issues": issues
        });

        serde_json::to_string_pretty(&report)
    }

    /// Format issues as compact JSON (no pretty printing)
    pub fn format_compact(issues: &[ArchitecturalIssue]) -> Result<String, serde_json::Error> {
        let report = json!({
            "detector": "GodObjectDetector",
            "total_issues": issues.len(),
            "severity_distribution": calculate_severity_distribution(issues),
            "issues": issues
        });

        serde_json::to_string(&report)
    }

    /// Format individual issue as JSON
    pub fn format_issue(issue: &ArchitecturalIssue) -> Value {
        json!({
            "id": issue.issue_id,
            "file_path": issue.file_path,
            "line_range": {
                "start": issue.start_line,
                "end": issue.end_line
            },
            "severity": issue.severity,
            "description": issue.description,
            "detector": issue.detector_name,
            "code_snippet": issue.code_snippet
        })
    }
}
