//! SQLite implementation for DebtRepository (stub implementation)

use async_trait::async_trait;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::repositories::traits::TechnicalDebt;
use crate::database::repositories::traits::{DebtRepository, Repository};
use crate::error::{Result, UveddiError};

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

    async fn find_by_id(&self, _id: i64) -> Result<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Debt functionality not implemented",
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Debt functionality not implemented",
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
impl DebtRepository for SqliteDebtRepository {
    async fn find_by_run(&self, _run_id: i64) -> Result<Vec<TechnicalDebt>> {
        Ok(Vec::new())
    }

    async fn find_by_category(&self, _category: &str) -> Result<Vec<TechnicalDebt>> {
        Ok(Vec::new())
    }

    async fn calculate_total_cost(&self, _run_id: i64) -> Result<f64> {
        Ok(0.0)
    }

    async fn find_high_priority(&self, _run_id: i64) -> Result<Vec<TechnicalDebt>> {
        Ok(Vec::new())
    }
}
