use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core analysis run tracking entity
///
/// Represents a single execution of the analysis engine on a project.
/// This is the primary entity that groups all analysis results and
/// tracks execution metadata.
///
/// # Database Mapping
///
/// Maps to the `analysis_runs` table in the database schema.
///
/// # Status Values
///
/// - `"running"`: Analysis is currently in progress
/// - `"completed"`: Analysis finished successfully
/// - `"failed"`: Analysis encountered an error and stopped
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisRun {
    /// Unique identifier for this analysis run (auto-generated)
    pub run_id: Option<i64>,
    /// Foreign key reference to the project being analyzed
    pub project_id: i64,
    /// Timestamp when the analysis started
    pub start_time: DateTime<Utc>,
    /// Timestamp when the analysis completed (None if still running)
    pub end_time: Option<DateTime<Utc>>,
    /// Current status of the analysis run
    pub status: String,
    /// Total number of source files processed
    pub total_files_analyzed: Option<i32>,
    /// Total number of issues detected across all files
    pub total_issues_found: Option<i32>,
    /// JSON-serialized configuration used for this analysis
    pub analysis_config: String,
}

/// Analysis statistics for efficient reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStats {
    pub total_issues: u32,
    pub critical_count: u32,
    pub high_count: u32,
    pub medium_count: u32,
    pub low_count: u32,
    pub affected_files: u32,
    pub detector_breakdown: HashMap<String, u32>,
    pub category_breakdown: HashMap<String, u32>,
}

impl Default for AnalysisStats {
    fn default() -> Self {
        Self {
            total_issues: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            affected_files: 0,
            detector_breakdown: HashMap::new(),
            category_breakdown: HashMap::new(),
        }
    }
}
