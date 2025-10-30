use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Architectural issue detected during analysis
///
/// Represents a specific code quality issue, anti-pattern, or architectural
/// problem identified by the analysis engine. Each issue is associated with
/// a particular analysis run and anti-pattern type.
///
/// # Database Mapping
///
/// Maps to the `architectural_issues` table in the database schema.
///
/// # Severity Levels
///
/// - `"low"`: Minor issues that don't significantly impact maintainability
/// - `"medium"`: Moderate issues that should be addressed over time
/// - `"high"`: Important issues that impact code quality
/// - `"critical"`: Severe issues that require immediate attention
#[derive(Debug, Clone, Serialize, Deserialize)]
// Note: rkyv derives temporarily disabled due to DateTime<Utc> incompatibility
// TODO: Implement custom wrapper or use different timestamp format for rkyv
// #[cfg_attr(
//     feature = "memory-optimization",
//     derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
// )]
pub struct ArchitecturalIssue {
    /// Unique identifier for this issue (auto-generated)
    pub issue_id: Option<i64>,
    /// Foreign key reference to the analysis run that found this issue
    pub analysis_run_id: i64,
    /// Foreign key reference to the type of anti-pattern detected
    pub anti_pattern_type_id: i64,
    /// Path to the file where this issue was found
    pub file_path: String,
    /// Starting line number of the problematic code (optional)
    pub start_line: Option<i32>,
    /// Ending line number of the problematic code (optional)
    pub end_line: Option<i32>,
    /// Line number where issue was found (for compatibility with domain model)
    pub line_number: Option<i32>,
    /// Column number where issue was found (for compatibility with domain model)
    pub column_number: Option<i32>,
    /// Issue message (for compatibility with domain model)
    pub message: String,
    /// JSON metadata for additional issue information
    pub metadata: String,
    /// Name of the detector that found this issue
    pub detector_name: String,
    /// Timestamp when this issue was created
    pub created_at: DateTime<Utc>,
    /// Severity level of this issue
    pub severity: String,
    /// Human-readable description of the issue
    pub description: String,
    /// Optional code snippet showing the problematic code
    pub code_snippet: Option<String>,
    /// Optional AI-generated explanation and remediation advice
    pub ai_explanation: Option<String>,
}

impl ArchitecturalIssue {
    /// Creates a new ArchitecturalIssue with required fields
    pub fn new(
        analysis_run_id: i64,
        anti_pattern_type_id: i64,
        file_path: String,
        line_number: Option<i32>,
        message: String,
        detector_name: String,
        severity: String,
        description: String,
    ) -> Self {
        Self {
            issue_id: None,
            analysis_run_id,
            anti_pattern_type_id,
            file_path,
            start_line: line_number,
            end_line: line_number,
            line_number,
            column_number: None,
            message,
            metadata: "{}".to_string(),
            detector_name,
            created_at: chrono::Utc::now(),
            severity,
            description,
            code_snippet: None,
            ai_explanation: None,
        }
    }
}

impl Default for ArchitecturalIssue {
    fn default() -> Self {
        Self {
            issue_id: None,
            analysis_run_id: 0,
            anti_pattern_type_id: 0,
            file_path: String::new(),
            start_line: None,
            end_line: None,
            line_number: None,
            column_number: None,
            message: String::new(),
            metadata: String::new(),
            detector_name: String::new(),
            created_at: Utc::now(),
            severity: String::new(),
            description: String::new(),
            code_snippet: None,
            ai_explanation: None,
        }
    }
}
