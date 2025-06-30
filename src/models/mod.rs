use rusqlite::Row;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Convert SystemTime to Unix timestamp
pub fn system_time_to_unix_timestamp(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Convert Unix timestamp to SystemTime
pub fn unix_timestamp_to_system_time(timestamp: i64) -> SystemTime {
    UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64)
}

/// Helper trait for converting database rows to structs
pub trait FromRow: Sized {
    fn from_row(row: &Row) -> rusqlite::Result<Self>;
}

/// Project model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub name: String,
    pub repository_url: String,
    pub last_analyzed_commit: Option<String>,
    pub config_data: Option<serde_json::Value>,
    pub created_at: SystemTime,
}

impl FromRow for Project {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let config_data: Option<String> = row.get("config_data")?;
        let config_data = config_data
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))?;

        Ok(Project {
            project_id: Some(row.get("project_id")?),
            organization_id: row.get("organization_id")?,
            name: row.get("name")?,
            repository_url: row.get("repository_url")?,
            last_analyzed_commit: row.get("last_analyzed_commit")?,
            config_data,
            created_at: unix_timestamp_to_system_time(row.get("created_at")?),
        })
    }
}

/// AnalysisRun model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub run_id: Option<i64>,
    pub project_id: i64,
    pub user_id: Option<i64>,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub status: String,
    pub ai_model_used: Option<String>,
    pub total_files_scanned: Option<i64>,
    pub total_issues_found: Option<i64>,
    pub exit_code: Option<i64>,
    pub raw_analysis_output: Option<serde_json::Value>,
}

impl FromRow for AnalysisRun {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let raw_analysis_output: Option<String> = row.get("raw_analysis_output")?;
        let raw_analysis_output = raw_analysis_output
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))?;

        let end_time: Option<i64> = row.get("end_time")?;
        let end_time = end_time.map(unix_timestamp_to_system_time);

        Ok(AnalysisRun {
            run_id: Some(row.get("run_id")?),
            project_id: row.get("project_id")?,
            user_id: row.get("user_id")?,
            start_time: unix_timestamp_to_system_time(row.get("start_time")?),
            end_time,
            status: row.get("status")?,
            ai_model_used: row.get("ai_model_used")?,
            total_files_scanned: row.get("total_files_scanned")?,
            total_issues_found: row.get("total_issues_found")?,
            exit_code: row.get("exit_code")?,
            raw_analysis_output,
        })
    }
}

/// ArchitecturalIssue model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalIssue {
    pub issue_id: Option<i64>,
    pub run_id: i64,
    pub anti_pattern_type_id: i64,
    pub file_path: String,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub ai_refactoring_suggestion: Option<String>,
    pub is_ignored: bool,
    pub ignored_by_user_id: Option<i64>,
    pub ignored_at: Option<SystemTime>,
}

impl FromRow for ArchitecturalIssue {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let ignored_at: Option<i64> = row.get("ignored_at")?;
        let ignored_at = ignored_at.map(unix_timestamp_to_system_time);
        
        // SQLite stores booleans as integers
        let is_ignored: i64 = row.get("is_ignored")?;

        Ok(ArchitecturalIssue {
            issue_id: Some(row.get("issue_id")?),
            run_id: row.get("run_id")?,
            anti_pattern_type_id: row.get("anti_pattern_type_id")?,
            file_path: row.get("file_path")?,
            line_start: row.get("line_start")?,
            line_end: row.get("line_end")?,
            severity: row.get("severity")?,
            title: row.get("title")?,
            description: row.get("description")?,
            ai_refactoring_suggestion: row.get("ai_refactoring_suggestion")?,
            is_ignored: is_ignored != 0,
            ignored_by_user_id: row.get("ignored_by_user_id")?,
            ignored_at,
        })
    }
}

/// CodeSnippet model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub snippet_id: Option<i64>,
    pub issue_id: i64,
    pub content: String,
    pub language: Option<String>,
    pub context_lines_before: Option<i64>,
    pub context_lines_after: Option<i64>,
}

impl FromRow for CodeSnippet {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(CodeSnippet {
            snippet_id: Some(row.get("snippet_id")?),
            issue_id: row.get("issue_id")?,
            content: row.get("content")?,
            language: row.get("language")?,
            context_lines_before: row.get("context_lines_before")?,
            context_lines_after: row.get("context_lines_after")?,
        })
    }
}

/// Organization model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub organization_id: Option<i64>,
    pub name: String,
    pub created_at: SystemTime,
}

impl FromRow for Organization {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Organization {
            organization_id: Some(row.get("organization_id")?),
            name: row.get("name")?,
            created_at: unix_timestamp_to_system_time(row.get("created_at")?),
        })
    }
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub username: String,
    pub email: Option<String>,
    pub created_at: SystemTime,
}

impl FromRow for User {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(User {
            user_id: Some(row.get("user_id")?),
            organization_id: row.get("organization_id")?,
            username: row.get("username")?,
            email: row.get("email")?,
            created_at: unix_timestamp_to_system_time(row.get("created_at")?),
        })
    }
}

pub mod antipattern_type;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_organization_serialization() {
        let org = Organization {
            organization_id: Some(1),
            name: "TestOrg".to_string(),
            created_at: UNIX_EPOCH,
        };
        let json = serde_json::to_string(&org).unwrap();
        let deserialized: Organization = serde_json::from_str(&json).unwrap();
        assert_eq!(org.organization_id, deserialized.organization_id);
        assert_eq!(org.name, deserialized.name);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            user_id: Some(1),
            organization_id: Some(2),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            created_at: UNIX_EPOCH,
        };
        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();
        assert_eq!(user.user_id, deserialized.user_id);
        assert_eq!(user.organization_id, deserialized.organization_id);
        assert_eq!(user.username, deserialized.username);
        assert_eq!(user.email, deserialized.email);
    }
}
