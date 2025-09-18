//! Structured data formatting for God Object reports

use crate::database::models::ArchitecturalIssue;
use serde_json::{json, Value};
use super::utils::{calculate_severity_distribution, extract_class_name, extract_metrics_from_description, generate_recommendations};

/// Structured data formatter for external consumption
pub struct StructuredFormatter;

impl StructuredFormatter {
    /// Format issues as structured data for external consumption
    pub fn format(issues: &[ArchitecturalIssue]) -> Value {
        json!({
            "report_type": "god_object_detection",
            "metadata": {
                "detector_name": "GodObjectDetector",
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "total_issues": issues.len(),
                "severity_distribution": calculate_severity_distribution(issues)
            },
            "issues": issues.iter().map(|issue| Self::format_issue(issue)).collect::<Vec<_>>()
        })
    }

    /// Format individual issue with enhanced metadata
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
            "class_name": extract_class_name(&issue.description),
            "metrics": extract_metrics_from_description(&issue.description),
            "recommendations": generate_recommendations(&issue.description),
            "detector": issue.detector_name,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Generate machine-readable analysis summary
    pub fn generate_analysis_summary(issues: &[ArchitecturalIssue]) -> Value {
        let distribution = calculate_severity_distribution(issues);
        let total_issues = issues.len();

        let risk_score = Self::calculate_risk_score(&distribution, total_issues);

        json!({
            "summary": {
                "total_issues": total_issues,
                "risk_score": risk_score,
                "severity_distribution": distribution,
                "recommendation": Self::get_overall_recommendation(risk_score)
            },
            "files_affected": Self::get_affected_files_summary(issues),
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Calculate overall risk score based on severity distribution
    fn calculate_risk_score(distribution: &std::collections::HashMap<String, usize>, total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }

        let critical = distribution.get("Critical").unwrap_or(&0);
        let high = distribution.get("High").unwrap_or(&0);
        let medium = distribution.get("Medium").unwrap_or(&0);
        let low = distribution.get("Low").unwrap_or(&0);

        let weighted_score = (*critical * 10 + *high * 5 + *medium * 2 + *low * 1) as f64;
        let max_possible = total * 10;

        (weighted_score / max_possible as f64) * 100.0
    }

    /// Get overall recommendation based on risk score
    fn get_overall_recommendation(risk_score: f64) -> &'static str {
        match risk_score {
            score if score >= 70.0 => "urgent_refactoring_required",
            score if score >= 40.0 => "schedule_refactoring_soon",
            score if score >= 20.0 => "consider_gradual_improvement",
            _ => "monitor_and_maintain"
        }
    }

    /// Get summary of affected files
    fn get_affected_files_summary(issues: &[ArchitecturalIssue]) -> Value {
        let mut files_map = std::collections::HashMap::new();

        for issue in issues {
            let entry = files_map.entry(&issue.file_path).or_insert_with(|| {
                json!({
                    "issue_count": 0,
                    "severities": std::collections::HashMap::<String, u64>::new()
                })
            });

            entry["issue_count"] = json!(entry["issue_count"].as_u64().unwrap_or(0) + 1);

            let severity = &issue.severity;
            let severities = entry["severities"].as_object_mut().unwrap();
            let count = severities.get(severity).and_then(|v| v.as_u64()).unwrap_or(0);
            severities.insert(severity.clone(), json!(count + 1));
        }

        json!(files_map)
    }
}