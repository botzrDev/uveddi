//! SQLite implementation for IssueRepository (stub implementation)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::models::ArchitecturalIssue;
use crate::database::repositories::traits::{IssueRepository, Repository};
use crate::error::{Result, UveddiError};

pub struct SqliteIssueRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteIssueRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteIssueRepository {
    type Entity = ArchitecturalIssue;

    async fn find_by_id(&self, _id: i64) -> Result<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Issue functionality not implemented",
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Issue functionality not implemented",
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
impl IssueRepository for SqliteIssueRepository {
    async fn find_by_run(&self, _run_id: i64) -> Result<Vec<ArchitecturalIssue>> {
        Ok(Vec::new())
    }

    async fn find_by_severity(&self, _severity: &str) -> Result<Vec<ArchitecturalIssue>> {
        Ok(Vec::new())
    }

    async fn find_by_type(&self, _issue_type: &str) -> Result<Vec<ArchitecturalIssue>> {
        Ok(Vec::new())
    }

    async fn count_by_severity(&self, _run_id: i64) -> Result<HashMap<String, usize>> {
        Ok(HashMap::new())
    }
}
