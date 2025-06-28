use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// Anti-pattern type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternType {
    pub type_id: Option<i64>,
    pub name: String,
    pub description: String,
    pub category: String, // "structural", "behavioral", "creational"
}
