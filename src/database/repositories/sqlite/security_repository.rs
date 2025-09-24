//! SQLite implementation for SecurityRepository (stub implementation)

use async_trait::async_trait;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::repositories::traits::SecurityIssue;
use crate::database::repositories::traits::{Repository, SecurityRepository};
use crate::error::{Result, UveddiError};

pub struct SqliteSecurityRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteSecurityRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteSecurityRepository {
    type Entity = SecurityIssue;

    async fn find_by_id(&self, _id: i64) -> Result<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Security functionality not implemented",
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Security functionality not implemented",
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
impl SecurityRepository for SqliteSecurityRepository {
    async fn find_by_run(&self, _run_id: i64) -> Result<Vec<SecurityIssue>> {
        Ok(Vec::new())
    }

    async fn find_by_severity(&self, _severity: &str) -> Result<Vec<SecurityIssue>> {
        Ok(Vec::new())
    }

    async fn find_by_category(&self, _category: &str) -> Result<Vec<SecurityIssue>> {
        Ok(Vec::new())
    }

    async fn count_critical(&self, _run_id: i64) -> Result<usize> {
        Ok(0)
    }
}
