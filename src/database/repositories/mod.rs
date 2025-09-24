//! Repository pattern implementation for database operations
//!
//! This module provides the repository abstraction layer that separates
//! business logic from data access logic, following clean architecture principles.

pub mod errors;
pub mod factory;
pub mod sqlite;
pub mod traits;

pub use traits::{
    AnalysisRepository, CacheRepository, DebtRepository, DependencyRepository, EventRepository,
    IssueRepository, MetricsRepository, ProjectRepository, Repository, RepositoryFactory,
    SecurityRepository, UnitOfWork,
};

// Re-export SQLite implementations
pub use sqlite::{
    SqliteAnalysisRepository, SqliteCacheRepository, SqliteDebtRepository,
    SqliteDependencyRepository, SqliteEventRepository, SqliteIssueRepository,
    SqliteMetricsRepository, SqliteProjectRepository, SqliteSecurityRepository,
};

// Re-export factory implementations
pub use factory::{RepositoryManager, SqliteRepositoryFactory};

// Re-export error types
pub use errors::{RepositoryError, RepositoryResult};
use std::sync::Arc;

/// Create a repository factory based on the database configuration
pub fn create_repository_factory(
    pool: Arc<crate::database::connection::pool::ConnectionPool>,
) -> RepositoryResult<Box<dyn RepositoryFactory>> {
    Ok(Box::new(SqliteRepositoryFactory::new(pool)))
}
