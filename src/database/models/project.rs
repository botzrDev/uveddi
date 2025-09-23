//! Project-related database models
//!
//! This module defines the database models for project management,
//! including project entities and their configurations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Database record for a project
///
/// This represents the database persistence layer for projects,
/// focused on storage and retrieval from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    /// Primary key identifier
    pub project_id: Option<i64>,
    /// Path to the project directory  
    pub path: String,
    /// When the project was first created
    pub created_at: DateTime<Utc>,
}

/// Domain model for a project  
///
/// This represents the business logic layer for projects,
/// with richer types and validation logic.
#[derive(Debug, Clone)]
pub struct Project {
    /// Project identifier
    pub id: Option<i64>,
    /// Path to project directory
    pub path: PathBuf,
    /// Project configuration settings
    pub config: ProjectConfig,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Configuration settings for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Languages to analyze in this project
    pub languages: Vec<String>,
    /// File patterns to exclude from analysis
    pub exclude_patterns: Vec<String>,
    /// Analysis depth (how deep to traverse dependencies)
    pub analysis_depth: u32,
    /// Whether to enable caching for this project
    pub enable_caching: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            languages: vec!["rust".to_string(), "javascript".to_string(), "typescript".to_string()],
            exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string(), ".git/".to_string()],
            analysis_depth: 5,
            enable_caching: true,
        }
    }
}

/// Conversion from domain model to persistence model
impl From<Project> for ProjectRecord {
    fn from(project: Project) -> Self {
        Self {
            project_id: project.id,
            path: project.path.to_string_lossy().to_string(),
            created_at: project.created_at,
        }
    }
}

/// Conversion from persistence model to domain model
impl TryFrom<ProjectRecord> for Project {
    type Error = std::io::Error;
    
    fn try_from(record: ProjectRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: record.project_id,
            path: PathBuf::from(record.path),
            config: ProjectConfig::default(),
            created_at: record.created_at,
        })
    }
}