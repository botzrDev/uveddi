use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::analysis::dependency_graph::{Cycle, DependencyGraph, CycleSeverity};

/// Core analysis run tracking - aligns with ERD AnalysisRun entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub run_id: Option<i64>,
    pub project_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String, // "running", "completed", "failed"
    pub total_files_analyzed: Option<i32>,
    pub total_issues_found: Option<i32>,
    pub analysis_config: String, // JSON serialized config
}

/// Architectural issues with severity and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalIssue {
    pub issue_id: Option<i64>,
    pub analysis_run_id: i64,
    pub anti_pattern_type_id: i64,
    pub file_path: String,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub severity: String, // "low", "medium", "high", "critical"
    pub description: String,
    pub code_snippet: Option<String>,
    pub ai_explanation: Option<String>,
}

impl ArchitecturalIssue {
    pub fn from_cycle(cycle: Cycle, graph: &DependencyGraph) -> Self {
        let severity_str = match cycle.severity {
            CycleSeverity::Low => "low".to_string(),
            CycleSeverity::Medium => "medium".to_string(),
            CycleSeverity::High => "high".to_string(),
        };

        let description = format!(
            "Cyclic dependency detected involving modules: {}. Files: {}.",
            cycle.modules.join(", "),
            cycle.file_paths.iter().map(|p| p.display().to_string()).collect::<Vec<String>>().join(", ")
        );

        // For simplicity, we'll use the first file in the cycle as the primary file_path
        // and set start/end lines to None as it's a project-wide issue.
        let file_path = cycle.file_paths.first().map_or("unknown".to_string(), |p| p.display().to_string());

        // Attempt to extract a code snippet from the first file (if available)
        let code_snippet = cycle.file_paths.first().and_then(|path| {
            std::fs::read_to_string(path).ok().map(|content| {
                let lines: Vec<&str> = content.lines().take(10).collect();
                lines.join("\n")
            })
        });

        ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 0, // This will be set when saving to DB
            anti_pattern_type_id: 0, // This will be set when saving to DB
            file_path,
            start_line: None,
            end_line: None,
            severity: severity_str,
            description,
            code_snippet,
            ai_explanation: None,
        }
    }
}

/// Anti-pattern type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternType {
    pub type_id: Option<i64>,
    pub name: String,
    pub description: String,
    pub category: String, // "structural", "behavioral", "creational"
}
