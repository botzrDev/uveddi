//! Repository factory implementation for creating repository instances

use std::sync::Arc;
use crate::database::connection::pool::ConnectionPool;
use crate::database::repositories::traits::{
    RepositoryFactory, ProjectRepository, AnalysisRepository, CacheRepository,
    MetricsRepository, EventRepository, IssueRepository, DependencyRepository,
    SecurityRepository, DebtRepository,
};
use crate::database::repositories::sqlite::{
    SqliteProjectRepository, SqliteAnalysisRepository, SqliteCacheRepository,
    SqliteMetricsRepository, SqliteEventRepository, SqliteIssueRepository,
    SqliteDependencyRepository, SqliteSecurityRepository, SqliteDebtRepository,
};

/// SQLite-based repository factory
pub struct SqliteRepositoryFactory {
    pool: Arc<ConnectionPool>,
}

impl SqliteRepositoryFactory {
    /// Create a new SQLite repository factory with the given connection pool
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

impl RepositoryFactory for SqliteRepositoryFactory {
    fn create_project_repository(&self) -> Box<dyn ProjectRepository> {
        Box::new(SqliteProjectRepository::new(Arc::clone(&self.pool)))
    }

    fn create_analysis_repository(&self) -> Box<dyn AnalysisRepository> {
        Box::new(SqliteAnalysisRepository::new(Arc::clone(&self.pool)))
    }

    fn create_cache_repository(&self) -> Box<dyn CacheRepository> {
        Box::new(SqliteCacheRepository::new(Arc::clone(&self.pool)))
    }

    fn create_metrics_repository(&self) -> Box<dyn MetricsRepository> {
        Box::new(SqliteMetricsRepository::new(Arc::clone(&self.pool)))
    }

    fn create_event_repository(&self) -> Box<dyn EventRepository> {
        Box::new(SqliteEventRepository::new(Arc::clone(&self.pool)))
    }

    fn create_issue_repository(&self) -> Box<dyn IssueRepository> {
        Box::new(SqliteIssueRepository::new(Arc::clone(&self.pool)))
    }

    fn create_dependency_repository(&self) -> Box<dyn DependencyRepository> {
        Box::new(SqliteDependencyRepository::new(Arc::clone(&self.pool)))
    }

    fn create_security_repository(&self) -> Box<dyn SecurityRepository> {
        Box::new(SqliteSecurityRepository::new(Arc::clone(&self.pool)))
    }

    fn create_debt_repository(&self) -> Box<dyn DebtRepository> {
        Box::new(SqliteDebtRepository::new(Arc::clone(&self.pool)))
    }
}

/// Application-wide repository manager that provides shared repository instances
pub struct RepositoryManager {
    factory: Arc<dyn RepositoryFactory>,
    // Cached repository instances for performance
    project_repo: Arc<dyn ProjectRepository>,
    analysis_repo: Arc<dyn AnalysisRepository>,
    cache_repo: Arc<dyn CacheRepository>,
}

impl RepositoryManager {
    /// Create a new repository manager with the given factory
    pub fn new(factory: Arc<dyn RepositoryFactory>) -> Self {
        let project_repo = Arc::from(factory.create_project_repository());
        let analysis_repo = Arc::from(factory.create_analysis_repository());
        let cache_repo = Arc::from(factory.create_cache_repository());

        Self {
            factory,
            project_repo,
            analysis_repo,
            cache_repo,
        }
    }

    /// Get the project repository
    pub fn project(&self) -> Arc<dyn ProjectRepository> {
        Arc::clone(&self.project_repo)
    }

    /// Get the analysis repository
    pub fn analysis(&self) -> Arc<dyn AnalysisRepository> {
        Arc::clone(&self.analysis_repo)
    }

    /// Get the cache repository
    pub fn cache(&self) -> Arc<dyn CacheRepository> {
        Arc::clone(&self.cache_repo)
    }

    /// Get the factory for creating additional repositories
    pub fn factory(&self) -> Arc<dyn RepositoryFactory> {
        Arc::clone(&self.factory)
    }

    /// Create a metrics repository on demand
    pub fn create_metrics_repository(&self) -> Box<dyn MetricsRepository> {
        self.factory.create_metrics_repository()
    }

    /// Create an event repository on demand
    pub fn create_event_repository(&self) -> Box<dyn EventRepository> {
        self.factory.create_event_repository()
    }

    /// Create an issue repository on demand
    pub fn create_issue_repository(&self) -> Box<dyn IssueRepository> {
        self.factory.create_issue_repository()
    }

    /// Create a dependency repository on demand
    pub fn create_dependency_repository(&self) -> Box<dyn DependencyRepository> {
        self.factory.create_dependency_repository()
    }

    /// Create a security repository on demand
    pub fn create_security_repository(&self) -> Box<dyn SecurityRepository> {
        self.factory.create_security_repository()
    }

    /// Create a debt repository on demand
    pub fn create_debt_repository(&self) -> Box<dyn DebtRepository> {
        self.factory.create_debt_repository()
    }
}