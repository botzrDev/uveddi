//! SQLite implementation for DependencyRepository (stub implementation)

use async_trait::async_trait;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::models::Dependency;
use crate::database::repositories::traits::{DependencyRepository, Repository};
use crate::error::{Result, UveddiError};

pub struct SqliteDependencyRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteDependencyRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteDependencyRepository {
    type Entity = Dependency;

    async fn find_by_id(&self, _id: i64) -> Result<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Dependency functionality not implemented",
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Dependency functionality not implemented",
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
impl DependencyRepository for SqliteDependencyRepository {
    async fn find_by_source(&self, _source: &str) -> Result<Vec<Dependency>> {
        Ok(Vec::new())
    }

    async fn find_by_target(&self, _target: &str) -> Result<Vec<Dependency>> {
        Ok(Vec::new())
    }

    async fn find_by_run(&self, _run_id: i64) -> Result<Vec<Dependency>> {
        Ok(Vec::new())
    }

    async fn exists(&self, _source: &str, _target: &str) -> Result<bool> {
        Ok(false)
    }

    async fn find_circular(&self, _run_id: i64) -> Result<Vec<Vec<Dependency>>> {
        Ok(Vec::new())
    }
}
