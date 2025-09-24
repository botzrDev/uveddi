//! SQLite repository implementations
//!
//! This module provides concrete SQLite implementations for all repository traits.

pub mod project_repository;
pub mod analysis_repository;
pub mod cache_repository;
pub mod metrics_repository;
pub mod event_repository;
pub mod issue_repository;
pub mod dependency_repository;
pub mod security_repository;
pub mod debt_repository;

pub use project_repository::SqliteProjectRepository;
pub use analysis_repository::SqliteAnalysisRepository;
pub use cache_repository::SqliteCacheRepository;
pub use metrics_repository::SqliteMetricsRepository;
pub use event_repository::SqliteEventRepository;
pub use issue_repository::SqliteIssueRepository;
pub use dependency_repository::SqliteDependencyRepository;
pub use security_repository::SqliteSecurityRepository;
pub use debt_repository::SqliteDebtRepository;

use crate::database::connection::pool::ConnectionPool;
use crate::database::repositories::traits::RepositoryFactory;
use std::sync::Arc;

/// SQLite repository factory implementation
pub struct SqliteRepositoryFactory {
    pool: Arc<ConnectionPool>,
}

impl SqliteRepositoryFactory {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

impl RepositoryFactory for SqliteRepositoryFactory {
    fn create_project_repository(&self) -> Box<dyn crate::database::repositories::ProjectRepository> {
        Box::new(SqliteProjectRepository::new(self.pool.clone()))
    }

    fn create_analysis_repository(&self) -> Box<dyn crate::database::repositories::AnalysisRepository> {
        Box::new(SqliteAnalysisRepository::new(self.pool.clone()))
    }

    fn create_cache_repository(&self) -> Box<dyn crate::database::repositories::CacheRepository> {
        Box::new(SqliteCacheRepository::new(self.pool.clone()))
    }

    fn create_metrics_repository(&self) -> Box<dyn crate::database::repositories::MetricsRepository> {
        Box::new(SqliteMetricsRepository::new(self.pool.clone()))
    }

    fn create_event_repository(&self) -> Box<dyn crate::database::repositories::EventRepository> {
        Box::new(SqliteEventRepository::new(self.pool.clone()))
    }

    fn create_issue_repository(&self) -> Box<dyn crate::database::repositories::IssueRepository> {
        Box::new(SqliteIssueRepository::new(self.pool.clone()))
    }

    fn create_dependency_repository(&self) -> Box<dyn crate::database::repositories::DependencyRepository> {
        Box::new(SqliteDependencyRepository::new(self.pool.clone()))
    }

    fn create_security_repository(&self) -> Box<dyn crate::database::repositories::SecurityRepository> {
        Box::new(SqliteSecurityRepository::new(self.pool.clone()))
    }

    fn create_debt_repository(&self) -> Box<dyn crate::database::repositories::DebtRepository> {
        Box::new(SqliteDebtRepository::new(self.pool.clone()))
    }
}