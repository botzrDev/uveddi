//! SQLite implementation for EventRepository (stub implementation)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::models::LifecycleEvent;
use crate::database::repositories::traits::{EventRepository, Repository};
use crate::error::{Result, UveddiError};

pub struct SqliteEventRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteEventRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteEventRepository {
    type Entity = LifecycleEvent;

    async fn find_by_id(&self, _id: i64) -> Result<Option<Self::Entity>> {
        Ok(None)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>> {
        Ok(Vec::new())
    }

    async fn save(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Event functionality not implemented",
        ))
    }

    async fn update(&self, _entity: &Self::Entity) -> Result<Self::Entity> {
        Err(UveddiError::database_error_msg(
            "Event functionality not implemented",
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
impl EventRepository for SqliteEventRepository {
    async fn find_by_entity(
        &self,
        _entity_type: &str,
        _entity_id: i64,
    ) -> Result<Vec<LifecycleEvent>> {
        Ok(Vec::new())
    }

    async fn find_by_time_range(
        &self,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<LifecycleEvent>> {
        Ok(Vec::new())
    }

    async fn create_event(
        &self,
        _entity_type: &str,
        _entity_id: i64,
        _event_type: &str,
        _description: Option<String>,
    ) -> Result<LifecycleEvent> {
        Err(UveddiError::database_error_msg(
            "Event functionality not implemented",
        ))
    }
}
