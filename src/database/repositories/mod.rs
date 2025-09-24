//! Repository pattern implementation for database operations
//!
//! This module provides the repository abstraction layer that separates
//! business logic from data access logic, following clean architecture principles.

pub mod traits;
pub mod sqlite;
pub mod factory;

pub use traits::{
    Repository, ProjectRepository, AnalysisRepository, CacheRepository,
    MetricsRepository, EventRepository, IssueRepository, DependencyRepository,
    SecurityRepository, DebtRepository, UnitOfWork, RepositoryFactory,
};

// Re-export SQLite implementations
pub use sqlite::{
    SqliteProjectRepository, SqliteAnalysisRepository, SqliteCacheRepository,
    SqliteMetricsRepository, SqliteEventRepository, SqliteIssueRepository,
    SqliteDependencyRepository, SqliteSecurityRepository, SqliteDebtRepository,
};

// Re-export factory implementations
pub use factory::{SqliteRepositoryFactory, RepositoryManager};

use crate::error::Result;
use std::sync::Arc;

/// Create a repository factory based on the database configuration
pub fn create_repository_factory(
    pool: Arc<crate::database::connection::pool::ConnectionPool>,
) -> Result<Box<dyn RepositoryFactory>> {
    Ok(Box::new(SqliteRepositoryFactory::new(pool)))
}