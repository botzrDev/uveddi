use anyhow::{Context, Result};
use refinery::embed_migrations;
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// Embed migrations at compile time
embed_migrations!("migrations");

/// Database error types
#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] refinery::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Main database manager for CodeAtlas
pub struct DatabaseManager {
    connection: Connection,
}

impl DatabaseManager {
    /// Establish database connection and run migrations
    pub async fn new(db_path: Option<&Path>) -> Result<Self, DatabaseError> {
        let db_path = db_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Self::default_db_path());

        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        // Open connection
        let mut connection = Connection::open(&db_path)
            .context("Failed to open SQLite database")?;

        // Enable foreign key constraints
        connection
            .execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign key constraints")?;

        // Run migrations
        Self::run_migrations(&mut connection)?;

        Ok(Self { connection })
    }

    /// Get the default database path (.codeatlas/data.db in current directory)
    fn default_db_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push(".codeatlas");
        path.push("data.db");
        path
    }

    /// Run all pending migrations
    fn run_migrations(connection: &mut Connection) -> Result<(), DatabaseError> {
        migrations::runner().run(connection)?;
        Ok(())
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Get a mutable reference to the database connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

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

pub mod crud;
