//! Database Persistence Provider Implementation
//!
//! This module implements the PersistenceProvider interface using the existing
//! database infrastructure. It converts between domain types and database models
//! to break circular dependencies (UV-105, Phase 1.1).

use crate::core::interfaces::persistence::{
    AnalysisRunDomain, DomainIssue, IssueFilter, IssueSeverity, IssueStats, PersistenceError,
    PersistenceProvider, PersistenceResult,
};
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::database::{Database, RepositoryManager};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Database implementation of the persistence provider
///
/// This implementation converts between domain types (used by analysis components)
/// and database models (used by storage layer) to eliminate circular dependencies.
pub struct DatabasePersistenceProvider {
    database: Arc<Database>,
    repository_manager: Arc<RepositoryManager>,
}

impl DatabasePersistenceProvider {
    /// Create a new database persistence provider
    pub fn new(database: Arc<Database>, repository_manager: Arc<RepositoryManager>) -> Self {
        Self {
            database,
            repository_manager,
        }
    }
}

#[async_trait]
impl PersistenceProvider for DatabasePersistenceProvider {
    type Error = PersistenceError;

    async fn save_issues(&self, issues: Vec<DomainIssue>) -> Result<(), Self::Error> {
        // Convert domain issues to database models
        let db_issues: Vec<ArchitecturalIssue> = issues
            .into_iter()
            .map(|issue| self.convert_to_db_model(issue))
            .collect::<Result<Vec<_>, _>>()?;

        // Save using database operations
        // Note: The current Database struct doesn't have async methods,
        // so we'll use tokio::task::spawn_blocking for now
        let _database = self.database.clone();
        tokio::task::spawn_blocking(move || {
            // For now, we'll use the store_issues method which expects a mutable reference
            // This is a temporary implementation until we can properly refactor the database layer
            for _issue in &db_issues {
                // Note: This is a simplified implementation
                // In a full implementation, we'd need to extend the Database struct
                // to have proper methods for individual issue storage
            }
            Ok::<(), PersistenceError>(())
        })
        .await
        .map_err(|e| PersistenceError::Internal(e.to_string()))?
    }

    async fn load_issues(&self, _filter: IssueFilter) -> Result<Vec<DomainIssue>, Self::Error> {
        // For now, return empty vector as this requires implementing additional database methods
        // This is a placeholder implementation that needs to be extended
        Ok(Vec::new())
    }

    async fn get_issue_stats(&self) -> Result<IssueStats, Self::Error> {
        // Placeholder implementation - returns empty stats
        Ok(IssueStats::default())
    }

    async fn save_analysis_run(&self, run: AnalysisRunDomain) -> Result<i64, Self::Error> {
        // Use repository to save analysis run
        let analysis_repo = self.repository_manager.analysis_repository();

        // Create a new analysis run - assuming project_id is available in the domain run
        let project_id = 1; // TODO: Extract from run domain object properly
        let analysis_run = analysis_repo
            .create_analysis_run(project_id)
            .await
            .map_err(|e| PersistenceError::QueryError(e.to_string()))?;

        Ok(analysis_run.run_id.unwrap_or(0))
    }

    async fn update_analysis_run(
        &self,
        _run_id: i64,
        _status: String,
        _end_time: Option<DateTime<Utc>>,
    ) -> Result<(), Self::Error> {
        // Placeholder implementation
        Ok(())
    }
}

impl DatabasePersistenceProvider {
    /// Convert domain issue to database model
    fn convert_to_db_model(
        &self,
        domain_issue: DomainIssue,
    ) -> PersistenceResult<ArchitecturalIssue> {
        {
            let mut _issue = ArchitecturalIssue::new(
                domain_issue.run_id.unwrap_or(0),
                1, // TODO: Map issue_type to anti_pattern_type_id
                domain_issue.file_path,
                Some(domain_issue.line_number as i32),
                domain_issue.message,
                domain_issue.detector_name,
                domain_issue.severity.to_string(),
                domain_issue.description,
            );
            _issue.issue_id = domain_issue.id.map(|id| id.parse().unwrap_or(0));
            _issue.column_number = domain_issue.column_number.map(|c| c as i32);
            _issue.created_at = domain_issue.created_at;
            // Note: metadata field is not available in the new ArchitecturalIssue structure
            Ok(_issue)
        }
    }

    /// Convert database model to domain issue
    fn convert_to_domain(&self, _db_issue: ArchitecturalIssue) -> PersistenceResult<DomainIssue> {
        let severity = match _db_issue.severity.to_lowercase().as_str() {
            "low" => IssueSeverity::Low,
            "medium" => IssueSeverity::Medium,
            "high" => IssueSeverity::High,
            "critical" => IssueSeverity::Critical,
            _ => IssueSeverity::Medium, // Default
        };

        let metadata = serde_json::from_str(&_db_issue.metadata).unwrap_or(serde_json::Value::Null);

        Ok(DomainIssue {
            id: _db_issue.issue_id.map(|id| id.to_string()),
            run_id: Some(_db_issue.analysis_run_id),
            detector_name: _db_issue.detector_name,
            issue_type: "architectural".to_string(), // TODO: Map from anti_pattern_type_id
            severity,
            message: _db_issue.message,
            description: _db_issue.description,
            file_path: _db_issue.file_path,
            line_number: _db_issue.line_number.map(|l| l as u32).unwrap_or(0),
            column_number: _db_issue.column_number.map(|c| c as u32),
            metadata,
            created_at: _db_issue.created_at,
        })
    }

    /// Convert domain analysis run to database model
    fn convert_run_to_db_model(
        &self,
        domain_run: AnalysisRunDomain,
    ) -> PersistenceResult<AnalysisRun> {
        let analysis_config = serde_json::to_string(&domain_run.analysis_config)
            .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;

        Ok(AnalysisRun {
            run_id: domain_run.run_id,
            project_id: domain_run.project_id.unwrap_or(1), // Default project
            start_time: domain_run.start_time,
            end_time: domain_run.end_time,
            status: domain_run.status,
            total_files_analyzed: domain_run.total_files_analyzed,
            total_issues_found: domain_run.total_issues_found,
            analysis_config,
        })
    }
}

/// Mock implementation for testing
pub struct MockPersistenceProvider {
    issues: std::sync::Mutex<Vec<DomainIssue>>,
    runs: std::sync::Mutex<Vec<AnalysisRunDomain>>,
    next_run_id: std::sync::atomic::AtomicI64,
}

impl MockPersistenceProvider {
    pub fn new() -> Self {
        Self {
            issues: std::sync::Mutex::new(Vec::new()),
            runs: std::sync::Mutex::new(Vec::new()),
            next_run_id: std::sync::atomic::AtomicI64::new(1),
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
        let mut filtered = stored_issues.clone();

        if let Some(run_id) = filter.run_id {
            filtered.retain(|issue| issue.run_id == Some(run_id));
        }

        if let Some(detector_name) = filter.detector_name {
            filtered.retain(|issue| issue.detector_name == detector_name);
        }

        if let Some(severity) = filter.severity {
            filtered.retain(|issue| issue.severity == severity);
        }

        Ok(filtered)
    }

    async fn get_issue_stats(&self) -> Result<IssueStats, Self::Error> {
        let stored_issues = self.issues.lock().unwrap();

        let mut stats = IssueStats::default();
        stats.total_issues = stored_issues.len();

        for issue in stored_issues.iter() {
            *stats
                .issues_by_severity
                .entry(issue.severity.clone())
                .or_insert(0) += 1;
            *stats
                .issues_by_type
                .entry(issue.issue_type.clone())
                .or_insert(0) += 1;
            *stats
                .issues_by_detector
                .entry(issue.detector_name.clone())
                .or_insert(0) += 1;
        }

        Ok(stats)
    }

    async fn save_analysis_run(&self, mut run: AnalysisRunDomain) -> Result<i64, Self::Error> {
        let run_id = self
            .next_run_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        run.run_id = Some(run_id);

        let mut stored_runs = self.runs.lock().unwrap();
        stored_runs.push(run);

        Ok(run_id)
    }

    async fn update_analysis_run(
        &self,
        run_id: i64,
        status: String,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<(), Self::Error> {
        let mut stored_runs = self.runs.lock().unwrap();

        if let Some(run) = stored_runs.iter_mut().find(|r| r.run_id == Some(run_id)) {
            run.status = status;
            run.end_time = end_time;
        }

        Ok(())
    }
}

impl Default for MockPersistenceProvider {
    fn default() -> Self {
        Self::new()
    }
}
