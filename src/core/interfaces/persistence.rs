//! Persistence Interface
//!
//! This module defines abstract persistence interfaces that break the circular
//! dependency between analysis and database modules (UV-105, Phase 1.1).
//!
//! The interfaces use domain types instead of database-specific types to
//! eliminate direct dependencies on database models.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

/// Abstract persistence provider interface
///
/// This interface breaks the circular dependency between analysis and database
/// by providing an abstraction layer. Analysis components depend on this
/// interface rather than concrete database implementations.
#[async_trait]
pub trait PersistenceProvider: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Save analysis issues to persistent storage
    async fn save_issues(&self, issues: Vec<DomainIssue>) -> Result<(), Self::Error>;
    
    /// Load issues with optional filtering
    async fn load_issues(&self, filter: IssueFilter) -> Result<Vec<DomainIssue>, Self::Error>;
    
    /// Get statistical information about issues
    async fn get_issue_stats(&self) -> Result<IssueStats, Self::Error>;
    
    /// Save analysis run metadata
    async fn save_analysis_run(&self, run: AnalysisRunDomain) -> Result<i64, Self::Error>;
    
    /// Update analysis run status
    async fn update_analysis_run(&self, run_id: i64, status: String, end_time: Option<DateTime<Utc>>) -> Result<(), Self::Error>;
}

/// Domain representation of an architectural issue
///
/// This is a domain-specific type that doesn't depend on database implementation.
/// It contains all necessary information for analysis while being independent
/// of storage specifics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainIssue {
    pub id: Option<String>,
    pub run_id: Option<i64>,
    pub detector_name: String,
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub description: String,
    pub file_path: String,
    pub line_number: u32,
    pub column_number: Option<u32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Domain representation of analysis run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRunDomain {
    pub run_id: Option<i64>,
    pub project_id: Option<i64>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub total_files_analyzed: Option<i32>,
    pub total_issues_found: Option<i32>,
    pub analysis_config: HashMap<String, serde_json::Value>,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Low => write!(f, "Low"),
            IssueSeverity::Medium => write!(f, "Medium"),
            IssueSeverity::High => write!(f, "High"),
            IssueSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Filter criteria for loading issues
#[derive(Debug, Clone, Default)]
pub struct IssueFilter {
    pub run_id: Option<i64>,
    pub detector_name: Option<String>,
    pub severity: Option<IssueSeverity>,
    pub file_path: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Statistical information about issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStats {
    pub total_issues: usize,
    pub issues_by_severity: HashMap<IssueSeverity, usize>,
    pub issues_by_type: HashMap<String, usize>,
    pub issues_by_detector: HashMap<String, usize>,
    pub most_affected_files: Vec<(String, usize)>,
}

impl Default for IssueStats {
    fn default() -> Self {
        Self {
            total_issues: 0,
            issues_by_severity: HashMap::new(),
            issues_by_type: HashMap::new(),
            issues_by_detector: HashMap::new(),
            most_affected_files: Vec::new(),
        }
    }
}

/// Error types for persistence operations
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience type alias for persistence results
pub type PersistenceResult<T> = Result<T, PersistenceError>;

impl DomainIssue {
    /// Create a new domain issue
    pub fn new(
        detector_name: String,
        issue_type: String,
        severity: IssueSeverity,
        message: String,
        description: String,
        file_path: String,
        line_number: u32,
    ) -> Self {
        Self {
            id: None,
            run_id: None,
            detector_name,
            issue_type,
            severity,
            message,
            description,
            file_path,
            line_number,
            column_number: None,
            metadata: serde_json::Value::Null,
            created_at: Utc::now(),
        }
    }
    
    /// Set metadata for the issue
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Set column number for the issue
    pub fn with_column(mut self, column: u32) -> Self {
        self.column_number = Some(column);
        self
    }
    
    /// Set run ID for the issue
    pub fn with_run_id(mut self, run_id: i64) -> Self {
        self.run_id = Some(run_id);
        self
    }
}

impl AnalysisRunDomain {
    /// Create a new analysis run
    pub fn new(project_id: Option<i64>) -> Self {
        Self {
            run_id: None,
            project_id,
            start_time: Utc::now(),
            end_time: None,
            status: "running".to_string(),
            total_files_analyzed: None,
            total_issues_found: None,
            analysis_config: HashMap::new(),
        }
    }
    
    /// Mark the analysis run as completed
    pub fn complete(mut self, files_analyzed: i32, issues_found: i32) -> Self {
        self.end_time = Some(Utc::now());
        self.status = "completed".to_string();
        self.total_files_analyzed = Some(files_analyzed);
        self.total_issues_found = Some(issues_found);
        self
    }
    
    /// Mark the analysis run as failed
    pub fn fail(mut self, error_message: String) -> Self {
        self.end_time = Some(Utc::now());
        self.status = format!("failed: {}", error_message);
        self
    }
}

/// Mock implementation for testing and demonstrations
pub struct MockPersistenceProvider {
    issues: Arc<Mutex<Vec<DomainIssue>>>,
    runs: Arc<Mutex<Vec<AnalysisRunDomain>>>,
    next_run_id: Arc<Mutex<i64>>,
}

impl MockPersistenceProvider {
    pub fn new() -> Self {
        Self {
            issues: Arc::new(Mutex::new(Vec::new())),
            runs: Arc::new(Mutex::new(Vec::new())),
            next_run_id: Arc::new(Mutex::new(1)),
        }
    }
}

#[async_trait]
impl PersistenceProvider for MockPersistenceProvider {
    type Error = PersistenceError;

    async fn save_issues(&self, issues: Vec<DomainIssue>) -> Result<(), Self::Error> {
        let mut stored_issues = self.issues.lock().unwrap();
        stored_issues.extend(issues);
        Ok(())
    }

    async fn load_issues(&self, filter: IssueFilter) -> Result<Vec<DomainIssue>, Self::Error> {
        let stored_issues = self.issues.lock().unwrap();
        // In a real implementation, this would filter by the provided criteria
        Ok(stored_issues.clone())
    }

    async fn save_analysis_run(&self, mut run: AnalysisRunDomain) -> Result<i64, Self::Error> {
        let mut next_id = self.next_run_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        
        run.run_id = Some(id);
        
        let mut runs = self.runs.lock().unwrap();
        runs.push(run);
        
        Ok(id)
    }

    async fn update_analysis_run(&self, run_id: i64, status: String, end_time: Option<DateTime<Utc>>) -> Result<(), Self::Error> {
        let mut runs = self.runs.lock().unwrap();
        if let Some(run) = runs.iter_mut().find(|r| r.run_id == Some(run_id)) {
            run.status = status;
            run.end_time = end_time;
        }
        Ok(())
    }

    async fn get_issue_stats(&self) -> Result<IssueStats, Self::Error> {
        let stored_issues = self.issues.lock().unwrap();
        let total_issues = stored_issues.len();
        
        let mut by_severity = HashMap::new();
        let mut by_detector = HashMap::new();
        
        for issue in stored_issues.iter() {
            *by_severity.entry(issue.severity.clone()).or_insert(0) += 1;
            *by_detector.entry(issue.detector_name.clone()).or_insert(0) += 1;
        }
        
        Ok(IssueStats {
            total_issues,
            issues_by_severity: by_severity,
            issues_by_type: HashMap::new(), // Mock implementation
            issues_by_detector: by_detector,
            most_affected_files: Vec::new(), // Mock implementation
        })
    }
}
