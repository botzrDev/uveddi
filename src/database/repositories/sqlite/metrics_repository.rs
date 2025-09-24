//! SQLite implementation for MetricsRepository (stub implementation)

use async_trait::async_trait;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::repositories::errors::{RepositoryError, RepositoryResult};
use crate::database::repositories::traits::PerformanceMetrics;
use crate::database::repositories::traits::{MetricsRepository, Repository};

pub struct SqliteMetricsRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteMetricsRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteMetricsRepository {
    type Entity = PerformanceMetrics;

    async fn find_by_id(&self, _id: i64) -> RepositoryResult<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        Err(RepositoryError::Runtime(
            "Metrics functionality not implemented",
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        Err(RepositoryError::Runtime(
            "Metrics functionality not implemented",
        ))
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<bool> {
        Ok(false)
    }

    async fn count(&self) -> RepositoryResult<usize> {
        Ok(0)
    }
}

#[async_trait]
impl MetricsRepository for SqliteMetricsRepository {
    async fn find_by_run(&self, _run_id: i64) -> RepositoryResult<Vec<PerformanceMetrics>> {
        Ok(Vec::new())
    }

    async fn find_by_type(&self, _metric_type: &str) -> RepositoryResult<Vec<PerformanceMetrics>> {
        Ok(Vec::new())
    }

    async fn calculate_averages(&self, _project_id: i64) -> RepositoryResult<PerformanceMetrics> {
        Err(RepositoryError::Runtime(
            "Metrics functionality not implemented",
        ))
    }

    async fn find_latest(&self, _project_id: i64) -> RepositoryResult<Option<PerformanceMetrics>> {
        Ok(None)
    }
}
