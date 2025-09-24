//! Repository trait definitions for database operations
//!
//! This module defines the core repository traits that abstract database operations
//! following the Repository pattern. These traits provide a clean interface between
//! the domain layer and the persistence layer.

use async_trait::async_trait;
use crate::error::Result;
use crate::database::models::{
    Project, AnalysisRun, CacheEntry, ArchitecturalIssue, Dependency, LifecycleEvent,
};

// For now, use placeholder types for models that don't exist yet
pub type PerformanceMetrics = String; // Placeholder
pub type SecurityIssue = String; // Placeholder
pub type TechnicalDebt = String; // Placeholder
use std::path::Path;

/// Core repository trait that all repositories must implement
#[async_trait]
pub trait Repository: Send + Sync {
    /// The entity type this repository manages
    type Entity;

    /// Find an entity by its primary key
    async fn find_by_id(&self, id: i64) -> Result<Option<Self::Entity>>;

    /// Find all entities
    async fn find_all(&self) -> Result<Vec<Self::Entity>>;

    /// Save a new entity
    async fn save(&self, entity: &Self::Entity) -> Result<Self::Entity>;

    /// Update an existing entity
    async fn update(&self, entity: &Self::Entity) -> Result<Self::Entity>;

    /// Delete an entity by its primary key
    async fn delete(&self, id: i64) -> Result<bool>;

    /// Count total entities
    async fn count(&self) -> Result<usize>;
}

/// Repository for Project entities
#[async_trait]
pub trait ProjectRepository: Repository<Entity = Project> {
    /// Find a project by its file path
    async fn find_by_path(&self, path: &str) -> Result<Option<Project>>;

    /// Find all active projects
    async fn find_active(&self) -> Result<Vec<Project>>;

    /// Find projects with recent activity
    async fn find_recent(&self, limit: usize) -> Result<Vec<Project>>;

    /// Update project last accessed time
    async fn update_last_accessed(&self, id: i64) -> Result<()>;
}

/// Repository for AnalysisRun entities
#[async_trait]
pub trait AnalysisRepository: Repository<Entity = AnalysisRun> {
    /// Find analysis runs by project ID
    async fn find_by_project(&self, project_id: i64) -> Result<Vec<AnalysisRun>>;

    /// Find the latest analysis run for a project
    async fn find_latest(&self, project_id: i64) -> Result<Option<AnalysisRun>>;

    /// Find recent analysis runs across all projects
    async fn find_recent(&self, limit: usize) -> Result<Vec<AnalysisRun>>;

    /// Find analysis runs in a date range
    async fn find_by_date_range(
        &self,
        project_id: Option<i64>,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<AnalysisRun>>;

    /// Update analysis run status
    async fn update_status(&self, id: i64, status: String) -> Result<()>;

    /// Create a new analysis run for a project path
    async fn create_for_path(&self, project_path: &Path) -> Result<AnalysisRun>;
}

/// Repository for CacheEntry entities
#[async_trait]
pub trait CacheRepository: Repository<Entity = CacheEntry> {
    /// Find a cache entry by its key
    async fn find_by_key(&self, key: &str) -> Result<Option<CacheEntry>>;

    /// Find cache entries by category
    async fn find_by_category(&self, category: &str) -> Result<Vec<CacheEntry>>;

    /// Clean up expired cache entries
    async fn cleanup_expired(&self) -> Result<usize>;

    /// Get or create a cache entry
    async fn get_or_create(&self, key: &str, category: &str) -> Result<CacheEntry>;

    /// Update cache value
    async fn update_value(&self, key: &str, value: Vec<u8>) -> Result<()>;

    /// Check if cache key exists and is valid
    async fn is_valid(&self, key: &str) -> Result<bool>;
}

/// Repository for PerformanceMetrics entities
#[async_trait]
pub trait MetricsRepository: Repository<Entity = PerformanceMetrics> {
    /// Find metrics by run ID
    async fn find_by_run(&self, run_id: i64) -> Result<Vec<PerformanceMetrics>>;

    /// Find metrics by metric type
    async fn find_by_type(&self, metric_type: &str) -> Result<Vec<PerformanceMetrics>>;

    /// Calculate average metrics for a project
    async fn calculate_averages(&self, project_id: i64) -> Result<PerformanceMetrics>;

    /// Get latest metrics for a project
    async fn find_latest(&self, project_id: i64) -> Result<Option<PerformanceMetrics>>;
}

/// Repository for LifecycleEvent entities
#[async_trait]
pub trait EventRepository: Repository<Entity = LifecycleEvent> {
    /// Find events by entity type and ID
    async fn find_by_entity(&self, entity_type: &str, entity_id: i64) -> Result<Vec<LifecycleEvent>>;

    /// Find events in a time range
    async fn find_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<LifecycleEvent>>;

    /// Create an event
    async fn create_event(
        &self,
        entity_type: &str,
        entity_id: i64,
        event_type: &str,
        description: Option<String>,
    ) -> Result<LifecycleEvent>;
}

/// Repository for ArchitecturalIssue entities
#[async_trait]
pub trait IssueRepository: Repository<Entity = ArchitecturalIssue> {
    /// Find issues by analysis run
    async fn find_by_run(&self, run_id: i64) -> Result<Vec<ArchitecturalIssue>>;

    /// Find issues by severity
    async fn find_by_severity(&self, severity: &str) -> Result<Vec<ArchitecturalIssue>>;

    /// Find issues by type
    async fn find_by_type(&self, issue_type: &str) -> Result<Vec<ArchitecturalIssue>>;

    /// Count issues by severity for a run
    async fn count_by_severity(&self, run_id: i64) -> Result<std::collections::HashMap<String, usize>>;
}

/// Repository for Dependency entities
#[async_trait]
pub trait DependencyRepository: Repository<Entity = Dependency> {
    /// Find dependencies by source
    async fn find_by_source(&self, source: &str) -> Result<Vec<Dependency>>;

    /// Find dependencies by target
    async fn find_by_target(&self, target: &str) -> Result<Vec<Dependency>>;

    /// Find all dependencies for a run
    async fn find_by_run(&self, run_id: i64) -> Result<Vec<Dependency>>;

    /// Check if dependency exists
    async fn exists(&self, source: &str, target: &str) -> Result<bool>;

    /// Find circular dependencies
    async fn find_circular(&self, run_id: i64) -> Result<Vec<Vec<Dependency>>>;
}

/// Repository for SecurityIssue entities
#[async_trait]
pub trait SecurityRepository: Repository<Entity = SecurityIssue> {
    /// Find security issues by run
    async fn find_by_run(&self, run_id: i64) -> Result<Vec<SecurityIssue>>;

    /// Find security issues by severity
    async fn find_by_severity(&self, severity: &str) -> Result<Vec<SecurityIssue>>;

    /// Find security issues by category
    async fn find_by_category(&self, category: &str) -> Result<Vec<SecurityIssue>>;

    /// Count critical issues for a run
    async fn count_critical(&self, run_id: i64) -> Result<usize>;
}

/// Repository for TechnicalDebt entities
#[async_trait]
pub trait DebtRepository: Repository<Entity = TechnicalDebt> {
    /// Find technical debt items by run
    async fn find_by_run(&self, run_id: i64) -> Result<Vec<TechnicalDebt>>;

    /// Find technical debt by category
    async fn find_by_category(&self, category: &str) -> Result<Vec<TechnicalDebt>>;

    /// Calculate total debt cost for a run
    async fn calculate_total_cost(&self, run_id: i64) -> Result<f64>;

    /// Find high-priority debt items
    async fn find_high_priority(&self, run_id: i64) -> Result<Vec<TechnicalDebt>>;
}

/// Unit of Work pattern for coordinating multiple repositories
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin a new transaction
    async fn begin(&self) -> Result<()>;

    /// Commit the current transaction
    async fn commit(&self) -> Result<()>;

    /// Rollback the current transaction
    async fn rollback(&self) -> Result<()>;

    /// Execute a function within a transaction
    async fn execute<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send,
        R: Send;
}

/// Factory trait for creating repository instances
pub trait RepositoryFactory: Send + Sync {
    /// Create a project repository
    fn create_project_repository(&self) -> Box<dyn ProjectRepository>;

    /// Create an analysis repository
    fn create_analysis_repository(&self) -> Box<dyn AnalysisRepository>;

    /// Create a cache repository
    fn create_cache_repository(&self) -> Box<dyn CacheRepository>;

    /// Create a metrics repository
    fn create_metrics_repository(&self) -> Box<dyn MetricsRepository>;

    /// Create an event repository
    fn create_event_repository(&self) -> Box<dyn EventRepository>;

    /// Create an issue repository
    fn create_issue_repository(&self) -> Box<dyn IssueRepository>;

    /// Create a dependency repository
    fn create_dependency_repository(&self) -> Box<dyn DependencyRepository>;

    /// Create a security repository
    fn create_security_repository(&self) -> Box<dyn SecurityRepository>;

    /// Create a debt repository
    fn create_debt_repository(&self) -> Box<dyn DebtRepository>;
}