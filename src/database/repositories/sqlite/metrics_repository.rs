//! SQLite implementation for MetricsRepository (stub implementation)

use async_trait::async_trait;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::repositories::traits::PerformanceMetrics;
use crate::database::repositories::traits::{MetricsRepository, Repository};
use crate::error::{Result, UveddiError};

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

    async fn find_by_id(&self, _id: i64) -> Result<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Metrics functionality not implemented",
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Metrics functionality not implemented",
        ))
    }

    async fn delete(&self, _id: i64) -> Result<bool> {
        Ok(false)
    }

    async fn count(&self) -> Result<usize> {
        Ok(0)
    }
}

#[async_trait]
impl MetricsRepository for SqliteMetricsRepository {
    async fn find_by_run(&self, _run_id: i64) -> Result<Vec<PerformanceMetrics>> {
        Ok(Vec::new())
    }

    async fn find_by_type(&self, _metric_type: &str) -> Result<Vec<PerformanceMetrics>> {
        Ok(Vec::new())
    }

    async fn calculate_averages(&self, _project_id: i64) -> Result<PerformanceMetrics> {
        Err(UveddiError::database_error_msg(
            "Metrics functionality not implemented",
        ))
    }

    async fn find_latest(&self, _project_id: i64) -> Result<Option<PerformanceMetrics>> {
        Ok(None)
    }
}
