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

pub use uveddi_plugin_api::models::ArchitecturalIssue;

/// Anti-pattern type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternType {
    pub anti_pattern_type_id: Option<i64>,
    pub name: String,
    pub description: String,
    pub category: String, // "structural", "behavioral", "creational"
}
