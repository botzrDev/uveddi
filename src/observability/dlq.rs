//! Dead Letter Queue implementation for UV-86 observability framework
//!
//! Provides persistent storage for failed operations that cannot be retried,
//! with enterprise-grade features for monitoring and reprocessing.

use crate::observability::metrics::UveddiMetrics;
use crate::observability::tracing_utils::TraceId;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Record stored in the Dead Letter Queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqRecord {
    pub id: Option<i64>,
    pub trace_id: TraceId,
    pub timestamp: DateTime<Utc>,
    pub operation_type: String,
    pub input_data: serde_json::Value,
    pub error_details: String,
    pub retry_count: u32,
    pub user_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl DlqRecord {
    /// Create a new DLQ record
    pub fn new(
        trace_id: TraceId,
        operation_type: String,
        input_data: serde_json::Value,
        error_details: String,
        retry_count: u32,
        user_id: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: None,
            trace_id,
            timestamp: Utc::now(),
            operation_type,
            input_data,
            error_details,
            retry_count,
            user_id,
            metadata: metadata.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
        }
    }
}

/// Dead Letter Queue implementation with SQLite persistence
pub struct DeadLetterQueue {
    connection: Arc<Mutex<Connection>>,
    metrics: Arc<UveddiMetrics>,
}

impl DeadLetterQueue {
    /// Create a new Dead Letter Queue with the given database connection
    pub fn new(connection: Connection, metrics: Arc<UveddiMetrics>) -> Result<Self> {
        let conn = Arc::new(Mutex::new(connection));
        
        // Initialize DLQ table
        {
            let db = conn.lock().unwrap();
            db.execute(
                r#"
                CREATE TABLE IF NOT EXISTS dead_letter_queue (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    trace_id TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    operation_type TEXT NOT NULL,
                    input_data TEXT NOT NULL,
                    error_details TEXT NOT NULL,
                    retry_count INTEGER NOT NULL DEFAULT 0,
                    user_id TEXT,
                    metadata TEXT NOT NULL DEFAULT '{}',
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
                "#,
                [],
            ).context("Failed to create dead_letter_queue table")?;

            // Create indexes for better query performance
            db.execute(
                "CREATE INDEX IF NOT EXISTS idx_dlq_trace_id ON dead_letter_queue(trace_id)",
                [],
            ).context("Failed to create trace_id index")?;

            db.execute(
                "CREATE INDEX IF NOT EXISTS idx_dlq_timestamp ON dead_letter_queue(timestamp)",
                [],
            ).context("Failed to create timestamp index")?;

            db.execute(
                "CREATE INDEX IF NOT EXISTS idx_dlq_operation_type ON dead_letter_queue(operation_type)",
                [],
            ).context("Failed to create operation_type index")?;
        }

        info!("Dead Letter Queue initialized with SQLite persistence");

        Ok(Self {
            connection: conn,
            metrics,
        })
    }

    /// Add a record to the Dead Letter Queue
    #[tracing::instrument(skip(self, record), fields(trace_id = %record.trace_id, operation = %record.operation_type))]
    pub fn enqueue(&self, record: DlqRecord) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        
        let id = conn.query_row(
            r#"
            INSERT INTO dead_letter_queue 
            (trace_id, timestamp, operation_type, input_data, error_details, retry_count, user_id, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            RETURNING id
            "#,
            params![
                record.trace_id.to_string(),
                record.timestamp.to_rfc3339(),
                record.operation_type,
                serde_json::to_string(&record.input_data)?,
                record.error_details,
                record.retry_count,
                record.user_id,
                serde_json::to_string(&record.metadata)?
            ],
            |row| row.get::<_, i64>(0),
        ).context("Failed to insert record into dead letter queue")?;

        // Update metrics
        self.metrics.dlq_size().inc();
        
        warn!(
            trace_id = %record.trace_id,
            operation = %record.operation_type,
            dlq_id = id,
            retry_count = record.retry_count,
            "Request added to Dead Letter Queue"
        );

        Ok(id)
    }

    /// Get the current size of the Dead Letter Queue
    pub fn get_queue_size(&self) -> Result<i64> {
        let conn = self.connection.lock().unwrap();
        
        let size = conn.query_row(
            "SELECT COUNT(*) FROM dead_letter_queue",
            [],
            |row| row.get::<_, i64>(0),
        ).context("Failed to get DLQ size")?;

        Ok(size)
    }

    /// Retrieve a record from the Dead Letter Queue by ID
    pub fn get_record(&self, id: i64) -> Result<Option<DlqRecord>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, trace_id, timestamp, operation_type, input_data, 
                   error_details, retry_count, user_id, metadata
            FROM dead_letter_queue 
            WHERE id = ?1
            "#,
        ).context("Failed to prepare DLQ select statement")?;

        let record = stmt.query_row(params![id], |row| {
            Ok(DlqRecord {
                id: Some(row.get::<_, i64>(0)?),
                trace_id: TraceId::from_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(1, "trace_id".to_string(), rusqlite::types::Type::Text))?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(2, "timestamp".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                operation_type: row.get::<_, String>(3)?,
                input_data: serde_json::from_str(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(4, "input_data".to_string(), rusqlite::types::Type::Text))?,
                error_details: row.get::<_, String>(5)?,
                retry_count: row.get::<_, u32>(6)?,
                user_id: row.get::<_, Option<String>>(7)?,
                metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(8, "metadata".to_string(), rusqlite::types::Type::Text))?,
            })
        });

        match record {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    /// List records from the Dead Letter Queue with pagination
    pub fn list_records(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<DlqRecord>> {
        let conn = self.connection.lock().unwrap();
        
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, trace_id, timestamp, operation_type, input_data, 
                   error_details, retry_count, user_id, metadata
            FROM dead_letter_queue 
            ORDER BY timestamp DESC
            LIMIT ?1 OFFSET ?2
            "#,
        ).context("Failed to prepare DLQ list statement")?;

        let records = stmt.query_map(params![limit, offset], |row| {
            Ok(DlqRecord {
                id: Some(row.get::<_, i64>(0)?),
                trace_id: TraceId::from_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(1, "trace_id".to_string(), rusqlite::types::Type::Text))?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(2, "timestamp".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                operation_type: row.get::<_, String>(3)?,
                input_data: serde_json::from_str(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(4, "input_data".to_string(), rusqlite::types::Type::Text))?,
                error_details: row.get::<_, String>(5)?,
                retry_count: row.get::<_, u32>(6)?,
                user_id: row.get::<_, Option<String>>(7)?,
                metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(8, "metadata".to_string(), rusqlite::types::Type::Text))?,
            })
        }).context("Failed to query DLQ records")?;

        let mut result = Vec::new();
        for record in records {
            result.push(record?);
        }

        Ok(result)
    }

    /// Remove a record from the Dead Letter Queue
    pub fn remove_record(&self, id: i64) -> Result<bool> {
        let conn = self.connection.lock().unwrap();
        
        let rows_affected = conn.execute(
            "DELETE FROM dead_letter_queue WHERE id = ?1",
            params![id],
        ).context("Failed to delete DLQ record")?;

        if rows_affected > 0 {
            self.metrics.dlq_size().dec();
            info!(dlq_id = id, "Record removed from Dead Letter Queue");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clear all records from the Dead Letter Queue
    pub fn clear_all(&self) -> Result<u64> {
        let conn = self.connection.lock().unwrap();
        
        // Get count before clearing
        let count = conn.query_row(
            "SELECT COUNT(*) FROM dead_letter_queue",
            [],
            |row| row.get::<_, i64>(0),
        ).context("Failed to get DLQ size before clearing")?;
        
        conn.execute("DELETE FROM dead_letter_queue", [])
            .context("Failed to clear dead letter queue")?;

        // Reset metrics
        for _ in 0..count {
            self.metrics.dlq_size().dec();
        }

        warn!(cleared_count = count, "Dead Letter Queue cleared");
        
        Ok(count as u64)
    }

    /// Get records by operation type
    pub fn get_records_by_operation(&self, operation_type: &str) -> Result<Vec<DlqRecord>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, trace_id, timestamp, operation_type, input_data, 
                   error_details, retry_count, user_id, metadata
            FROM dead_letter_queue 
            WHERE operation_type = ?1
            ORDER BY timestamp DESC
            "#,
        ).context("Failed to prepare DLQ operation query")?;

        let records = stmt.query_map(params![operation_type], |row| {
            Ok(DlqRecord {
                id: Some(row.get::<_, i64>(0)?),
                trace_id: TraceId::from_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(1, "trace_id".to_string(), rusqlite::types::Type::Text))?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(2, "timestamp".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                operation_type: row.get::<_, String>(3)?,
                input_data: serde_json::from_str(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(4, "input_data".to_string(), rusqlite::types::Type::Text))?,
                error_details: row.get::<_, String>(5)?,
                retry_count: row.get::<_, u32>(6)?,
                user_id: row.get::<_, Option<String>>(7)?,
                metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(8, "metadata".to_string(), rusqlite::types::Type::Text))?,
            })
        }).context("Failed to query DLQ records by operation")?;

        let mut result = Vec::new();
        for record in records {
            result.push(record?);
        }

        Ok(result)
    }

    /// Get DLQ statistics
    pub fn get_statistics(&self) -> Result<DlqStatistics> {
        let conn = self.connection.lock().unwrap();
        
        let total_count = conn.query_row(
            "SELECT COUNT(*) FROM dead_letter_queue",
            [],
            |row| row.get::<_, i64>(0),
        )?;

        let operations_count = conn.prepare(
            "SELECT operation_type, COUNT(*) FROM dead_letter_queue GROUP BY operation_type"
        )?.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;

        let oldest_record = conn.query_row(
            "SELECT MIN(timestamp) FROM dead_letter_queue",
            [],
            |row| row.get::<_, Option<String>>(0),
        )?;

        Ok(DlqStatistics {
            total_count,
            operations_count: operations_count.into_iter().collect(),
            oldest_record_time: oldest_record
                .and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
    }
}

/// Statistics about the Dead Letter Queue
#[derive(Debug, Serialize, Deserialize)]
pub struct DlqStatistics {
    pub total_count: i64,
    pub operations_count: std::collections::HashMap<String, i64>,
    pub oldest_record_time: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::metrics::UveddiMetrics;
    use rusqlite::Connection;
    use std::sync::Arc;

    fn create_test_dlq() -> Result<DeadLetterQueue> {
        let conn = Connection::open_in_memory()?;
        let metrics = Arc::new(UveddiMetrics::new(&crate::observability::config::MetricsConfig::default())?);
        DeadLetterQueue::new(conn, metrics)
    }

    #[test]
    fn test_dlq_creation() {
        let dlq = create_test_dlq();
        assert!(dlq.is_ok());
    }

    #[test]
    fn test_enqueue_and_get_record() -> Result<()> {
        let dlq = create_test_dlq()?;
        
        let record = DlqRecord::new(
            TraceId::new(),
            "test_operation".to_string(),
            serde_json::json!({"key": "value"}),
            "Test error".to_string(),
            3,
            Some("test_user".to_string()),
            None,
        );

        let id = dlq.enqueue(record.clone())?;
        assert!(id > 0);

        let retrieved = dlq.get_record(id)?;
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.operation_type, record.operation_type);
        assert_eq!(retrieved.error_details, record.error_details);
        assert_eq!(retrieved.retry_count, record.retry_count);

        Ok(())
    }

    #[test]
    fn test_queue_size() -> Result<()> {
        let dlq = create_test_dlq()?;
        
        assert_eq!(dlq.get_queue_size()?, 0);

        let record = DlqRecord::new(
            TraceId::new(),
            "test_operation".to_string(),
            serde_json::json!({}),
            "Test error".to_string(),
            1,
            None,
            None,
        );

        dlq.enqueue(record)?;
        assert_eq!(dlq.get_queue_size()?, 1);

        Ok(())
    }

    #[test]
    fn test_list_records() -> Result<()> {
        let dlq = create_test_dlq()?;
        
        // Add multiple records
        for i in 0..5 {
            let record = DlqRecord::new(
                TraceId::new(),
                format!("operation_{}", i),
                serde_json::json!({"index": i}),
                format!("Error {}", i),
                1,
                None,
                None,
            );
            dlq.enqueue(record)?;
        }

        let records = dlq.list_records(Some(3), Some(0))?;
        assert_eq!(records.len(), 3);

        Ok(())
    }

    #[test]
    fn test_remove_record() -> Result<()> {
        let dlq = create_test_dlq()?;
        
        let record = DlqRecord::new(
            TraceId::new(),
            "test_operation".to_string(),
            serde_json::json!({}),
            "Test error".to_string(),
            1,
            None,
            None,
        );

        let id = dlq.enqueue(record)?;
        assert_eq!(dlq.get_queue_size()?, 1);

        let removed = dlq.remove_record(id)?;
        assert!(removed);
        assert_eq!(dlq.get_queue_size()?, 0);

        Ok(())
    }

    #[test]
    fn test_clear_all() -> Result<()> {
        let dlq = create_test_dlq()?;
        
        // Add multiple records
        for i in 0..3 {
            let record = DlqRecord::new(
                TraceId::new(),
                "test_operation".to_string(),
                serde_json::json!({"index": i}),
                "Test error".to_string(),
                1,
                None,
                None,
            );
            dlq.enqueue(record)?;
        }

        assert_eq!(dlq.get_queue_size()?, 3);
        
        let cleared = dlq.clear_all()?;
        assert_eq!(cleared, 3);
        assert_eq!(dlq.get_queue_size()?, 0);

        Ok(())
    }
}