//! SQLite implementation for DebtRepository (stub implementation)

use async_trait::async_trait;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::repositories::errors::{RepositoryError, RepositoryResult};
use crate::database::repositories::traits::TechnicalDebt;
use crate::database::repositories::traits::{DebtRepository, Repository};

pub struct SqliteDebtRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteDebtRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteDebtRepository {
    type Entity = TechnicalDebt;

    async fn find_by_id(&self, _id: i64) -> RepositoryResult<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        Err(RepositoryError::Runtime(
            "Debt functionality not implemented".to_string(),
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        Err(RepositoryError::Runtime(
            "Debt functionality not implemented".to_string(),
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
impl DebtRepository for SqliteDebtRepository {
    async fn find_by_run(&self, _run_id: i64) -> RepositoryResult<Vec<TechnicalDebt>> {
        Ok(Vec::new())
    }

    async fn find_by_category(&self, _category: &str) -> RepositoryResult<Vec<TechnicalDebt>> {
        Ok(Vec::new())
    }

    async fn calculate_total_cost(&self, _run_id: i64) -> RepositoryResult<f64> {
        Ok(0.0)
    }

    async fn find_high_priority(&self, _run_id: i64) -> RepositoryResult<Vec<TechnicalDebt>> {
        Ok(Vec::new())
    }
}
