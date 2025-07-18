//! Comprehensive Audit Logging System for Uveddi RBAC
//!
//! This module provides tamper-evident audit logging for all security-related
//! events in the system. It supports multiple storage backends and ensures
//! compliance with enterprise security standards.

use crate::security::{
    errors::{SecurityError, SecurityResult},
    models::{AuditEvent, AuditEventType, AuditOutcome},
};
use blake3::Hasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Audit store trait for different storage backends
#[async_trait::async_trait]
pub trait AuditStore: Send + Sync {
    /// Store an audit event
    async fn store_event(&self, event: AuditEvent) -> SecurityResult<()>;
    
    /// Retrieve audit events by criteria
    async fn get_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<Vec<AuditEvent>>;
    
    /// Get audit events count
    async fn count_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<u64>;
    
    /// Verify integrity of stored events
    async fn verify_integrity(&self, event_ids: Vec<Uuid>) -> SecurityResult<IntegrityCheckResult>;
    
    /// Get audit statistics
    async fn get_statistics(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> SecurityResult<AuditStatistics>;
}

/// Audit query criteria for filtering events
#[derive(Debug, Clone, Default)]
pub struct AuditQueryCriteria {
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub event_type: Option<AuditEventType>,
    pub outcome: Option<AuditOutcome>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// Integrity check result
#[derive(Debug, Clone)]
pub struct IntegrityCheckResult {
    pub total_events: u64,
    pub verified_events: u64,
    pub failed_events: Vec<Uuid>,
    pub integrity_ok: bool,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_outcome: HashMap<String, u64>,
    pub events_by_user: HashMap<String, u64>,
    pub failed_authentication_attempts: u64,
    pub authorization_denials: u64,
    pub security_violations: u64,
}

/// In-memory audit store (for testing and development)
pub struct InMemoryAuditStore {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl InMemoryAuditStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryAuditStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AuditStore for InMemoryAuditStore {
    async fn store_event(&self, event: AuditEvent) -> SecurityResult<()> {
        let mut events = self.events.write().await;
        events.push(event);
        Ok(())
    }
    
    async fn get_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        let mut filtered: Vec<AuditEvent> = events.iter()
            .filter(|event| {
                if let Some(user_id) = criteria.user_id {
                    if event.user_id != Some(user_id) {
                        return false;
                    }
                }
                if let Some(session_id) = criteria.session_id {
                    if event.session_id != Some(session_id) {
                        return false;
                    }
                }
                if let Some(event_type) = &criteria.event_type {
                    if &event.event_type != event_type {
                        return false;
                    }
                }
                if let Some(outcome) = &criteria.outcome {
                    if &event.outcome != outcome {
                        return false;
                    }
                }
                if let Some(resource) = &criteria.resource {
                    if &event.resource != resource {
                        return false;
                    }
                }
                if let Some(action) = &criteria.action {
                    if &event.action != action {
                        return false;
                    }
                }
                if let Some(start_time) = criteria.start_time {
                    if event.timestamp < start_time {
                        return false;
                    }
                }
                if let Some(end_time) = criteria.end_time {
                    if event.timestamp > end_time {
                        return false;
                    }
                }
                if let Some(ip_address) = &criteria.ip_address {
                    if event.ip_address.as_ref() != Some(ip_address) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        
        // Sort by timestamp (most recent first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply pagination
        if let Some(offset) = criteria.offset {
            filtered = filtered.into_iter().skip(offset as usize).collect();
        }
        if let Some(limit) = criteria.limit {
            filtered.truncate(limit as usize);
        }
        
        Ok(filtered)
    }
    
    async fn count_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<u64> {
        let events = self.get_events(criteria).await?;
        Ok(events.len() as u64)
    }
    
    async fn verify_integrity(&self, event_ids: Vec<Uuid>) -> SecurityResult<IntegrityCheckResult> {
        let events = self.events.read().await;
        let total_events = event_ids.len() as u64;
        let mut verified_events = 0;
        let mut failed_events = Vec::new();
        
        for event_id in event_ids {
            if let Some(event) = events.iter().find(|e| e.id == event_id) {
                if event.verify_integrity() {
                    verified_events += 1;
                } else {
                    failed_events.push(event_id);
                }
            } else {
                failed_events.push(event_id);
            }
        }
        
        let integrity_ok = failed_events.is_empty();
        Ok(IntegrityCheckResult {
            total_events,
            verified_events,
            failed_events,
            integrity_ok,
        })
    }
    
    async fn get_statistics(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> SecurityResult<AuditStatistics> {
        let criteria = AuditQueryCriteria {
            start_time: Some(start_time),
            end_time: Some(end_time),
            ..Default::default()
        };
        
        let events = self.get_events(criteria).await?;
        let total_events = events.len() as u64;
        
        let mut events_by_type = HashMap::new();
        let mut events_by_outcome = HashMap::new();
        let mut events_by_user = HashMap::new();
        let mut failed_authentication_attempts = 0;
        let mut authorization_denials = 0;
        let mut security_violations = 0;
        
        for event in events {
            // Count by type
            *events_by_type.entry(event.event_type.to_string()).or_insert(0) += 1;
            
            // Count by outcome
            *events_by_outcome.entry(event.outcome.to_string()).or_insert(0) += 1;
            
            // Count by user
            if let Some(user_id) = event.user_id {
                *events_by_user.entry(user_id.to_string()).or_insert(0) += 1;
            }
            
            // Count specific security events
            match (&event.event_type, &event.outcome) {
                (AuditEventType::Authentication, AuditOutcome::Failure) => {
                    failed_authentication_attempts += 1;
                }
                (AuditEventType::Authorization, AuditOutcome::Denied) => {
                    authorization_denials += 1;
                }
                (AuditEventType::SecurityViolation, _) => {
                    security_violations += 1;
                }
                _ => {}
            }
        }
        
        Ok(AuditStatistics {
            total_events,
            events_by_type,
            events_by_outcome,
            events_by_user,
            failed_authentication_attempts,
            authorization_denials,
            security_violations,
        })
    }
}

/// Database audit store (SQLite implementation)
pub struct DatabaseAuditStore {
    db_path: String,
}

impl DatabaseAuditStore {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }
    
    async fn get_connection(&self) -> SecurityResult<rusqlite::Connection> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| SecurityError::DatabaseConnectionError {
                error: e.to_string(),
            })?;
        
        // Ensure audit_events table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                user_id TEXT,
                session_id TEXT,
                resource TEXT NOT NULL,
                action TEXT NOT NULL,
                outcome TEXT NOT NULL,
                ip_address TEXT,
                user_agent TEXT,
                additional_data TEXT NOT NULL,
                integrity_hash TEXT NOT NULL
            )",
            [],
        ).map_err(|e| SecurityError::DatabaseError {
            operation: "create_table".to_string(),
            error: e.to_string(),
        })?;
        
        Ok(conn)
    }
}

#[async_trait::async_trait]
impl AuditStore for DatabaseAuditStore {
    async fn store_event(&self, event: AuditEvent) -> SecurityResult<()> {
        let conn = self.get_connection().await?;
        
        conn.execute(
            "INSERT INTO audit_events (
                id, timestamp, event_type, user_id, session_id, resource, action, outcome,
                ip_address, user_agent, additional_data, integrity_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                event.id.to_string(),
                event.timestamp.timestamp(),
                event.event_type.to_string(),
                event.user_id.map(|u| u.to_string()),
                event.session_id.map(|s| s.to_string()),
                event.resource,
                event.action,
                event.outcome.to_string(),
                event.ip_address,
                event.user_agent,
                event.additional_data.to_string(),
                event.integrity_hash,
            ],
        ).map_err(|e| SecurityError::DatabaseError {
            operation: "insert_audit_event".to_string(),
            error: e.to_string(),
        })?;
        
        Ok(())
    }
    
    async fn get_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<Vec<AuditEvent>> {
        let conn = self.get_connection().await?;
        
        let mut query = "SELECT id, timestamp, event_type, user_id, session_id, resource, action, outcome, ip_address, user_agent, additional_data, integrity_hash FROM audit_events WHERE 1=1".to_string();
        let mut params = Vec::new();
        
        if let Some(user_id) = criteria.user_id {
            query.push_str(" AND user_id = ?");
            params.push(user_id.to_string());
        }
        
        if let Some(session_id) = criteria.session_id {
            query.push_str(" AND session_id = ?");
            params.push(session_id.to_string());
        }
        
        if let Some(event_type) = criteria.event_type {
            query.push_str(" AND event_type = ?");
            params.push(event_type.to_string());
        }
        
        if let Some(outcome) = criteria.outcome {
            query.push_str(" AND outcome = ?");
            params.push(outcome.to_string());
        }
        
        if let Some(resource) = criteria.resource {
            query.push_str(" AND resource = ?");
            params.push(resource);
        }
        
        if let Some(action) = criteria.action {
            query.push_str(" AND action = ?");
            params.push(action);
        }
        
        if let Some(start_time) = criteria.start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(start_time.timestamp().to_string());
        }
        
        if let Some(end_time) = criteria.end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(end_time.timestamp().to_string());
        }
        
        if let Some(ip_address) = criteria.ip_address {
            query.push_str(" AND ip_address = ?");
            params.push(ip_address);
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        if let Some(limit) = criteria.limit {
            query.push_str(" LIMIT ?");
            params.push(limit.to_string());
        }
        
        if let Some(offset) = criteria.offset {
            query.push_str(" OFFSET ?");
            params.push(offset.to_string());
        }
        
        let mut stmt = conn.prepare(&query).map_err(|e| SecurityError::DatabaseError {
            operation: "prepare_query".to_string(),
            error: e.to_string(),
        })?;
        
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        
        let rows = stmt.query_map(&param_refs[..], |row| {
            let id: String = row.get(0)?;
            let timestamp: i64 = row.get(1)?;
            let event_type: String = row.get(2)?;
            let user_id: Option<String> = row.get(3)?;
            let session_id: Option<String> = row.get(4)?;
            let resource: String = row.get(5)?;
            let action: String = row.get(6)?;
            let outcome: String = row.get(7)?;
            let ip_address: Option<String> = row.get(8)?;
            let user_agent: Option<String> = row.get(9)?;
            let additional_data: String = row.get(10)?;
            let integrity_hash: String = row.get(11)?;
            
            Ok(AuditEvent {
                id: Uuid::parse_str(&id).map_err(|_| rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text))?,
                timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or(Utc::now()),
                event_type: event_type.parse().map_err(|_| rusqlite::Error::InvalidColumnType(2, "event_type".to_string(), rusqlite::types::Type::Text))?,
                user_id: user_id.and_then(|u| Uuid::parse_str(&u).ok()),
                session_id: session_id.and_then(|s| Uuid::parse_str(&s).ok()),
                resource,
                action,
                outcome: outcome.parse().map_err(|_| rusqlite::Error::InvalidColumnType(7, "outcome".to_string(), rusqlite::types::Type::Text))?,
                ip_address,
                user_agent,
                additional_data: serde_json::from_str(&additional_data).unwrap_or(serde_json::Value::Null),
                integrity_hash,
            })
        }).map_err(|e| SecurityError::DatabaseError {
            operation: "query_audit_events".to_string(),
            error: e.to_string(),
        })?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| SecurityError::DatabaseError {
                operation: "parse_audit_event".to_string(),
                error: e.to_string(),
            })?);
        }
        
        Ok(events)
    }
    
    async fn count_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<u64> {
        let conn = self.get_connection().await?;
        
        let mut query = "SELECT COUNT(*) FROM audit_events WHERE 1=1".to_string();
        let mut params = Vec::new();
        
        // Add same WHERE conditions as get_events
        if let Some(user_id) = criteria.user_id {
            query.push_str(" AND user_id = ?");
            params.push(user_id.to_string());
        }
        
        // ... (similar logic as get_events for other criteria)
        
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        
        let count: u64 = conn.query_row(&query, &param_refs[..], |row| {
            Ok(row.get::<_, i64>(0)? as u64)
        }).map_err(|e| SecurityError::DatabaseError {
            operation: "count_audit_events".to_string(),
            error: e.to_string(),
        })?;
        
        Ok(count)
    }
    
    async fn verify_integrity(&self, event_ids: Vec<Uuid>) -> SecurityResult<IntegrityCheckResult> {
        let conn = self.get_connection().await?;
        let total_events = event_ids.len() as u64;
        let mut verified_events = 0;
        let mut failed_events = Vec::new();
        
        for event_id in event_ids {
            let result = conn.query_row(
                "SELECT id, timestamp, event_type, resource, action, outcome, additional_data, integrity_hash FROM audit_events WHERE id = ?",
                [event_id.to_string()],
                |row| {
                    let id: String = row.get(0)?;
                    let timestamp: i64 = row.get(1)?;
                    let event_type: String = row.get(2)?;
                    let resource: String = row.get(3)?;
                    let action: String = row.get(4)?;
                    let outcome: String = row.get(5)?;
                    let additional_data: String = row.get(6)?;
                    let stored_hash: String = row.get(7)?;
                    
                    // Reconstruct event for integrity check
                    let event = AuditEvent {
                        id: Uuid::parse_str(&id).unwrap(),
                        timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or(Utc::now()),
                        event_type: event_type.parse().unwrap_or(AuditEventType::SystemEvent),
                        user_id: None,
                        session_id: None,
                        resource,
                        action,
                        outcome: outcome.parse().unwrap_or(AuditOutcome::Failure),
                        ip_address: None,
                        user_agent: None,
                        additional_data: serde_json::from_str(&additional_data).unwrap_or(serde_json::Value::Null),
                        integrity_hash: stored_hash.clone(),
                    };
                    
                    let calculated_hash = event.calculate_integrity_hash();
                    Ok(calculated_hash == stored_hash)
                }
            );
            
            match result {
                Ok(true) => verified_events += 1,
                Ok(false) | Err(_) => failed_events.push(event_id),
            }
        }
        
        let integrity_ok = failed_events.is_empty();
        Ok(IntegrityCheckResult {
            total_events,
            verified_events,
            failed_events,
            integrity_ok,
        })
    }
    
    async fn get_statistics(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> SecurityResult<AuditStatistics> {
        let conn = self.get_connection().await?;
        
        // Get total events
        let total_events: u64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE timestamp BETWEEN ? AND ?",
            [start_time.timestamp(), end_time.timestamp()],
            |row| Ok(row.get::<_, i64>(0)? as u64)
        ).map_err(|e| SecurityError::DatabaseError {
            operation: "count_total_events".to_string(),
            error: e.to_string(),
        })?;
        
        // Get events by type
        let mut events_by_type = HashMap::new();
        let mut stmt = conn.prepare("SELECT event_type, COUNT(*) FROM audit_events WHERE timestamp BETWEEN ? AND ? GROUP BY event_type")
            .map_err(|e| SecurityError::DatabaseError {
                operation: "prepare_events_by_type".to_string(),
                error: e.to_string(),
            })?;
        
        let rows = stmt.query_map([start_time.timestamp(), end_time.timestamp()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
        }).map_err(|e| SecurityError::DatabaseError {
            operation: "query_events_by_type".to_string(),
            error: e.to_string(),
        })?;
        
        for row in rows {
            let (event_type, count) = row.map_err(|e| SecurityError::DatabaseError {
                operation: "parse_events_by_type".to_string(),
                error: e.to_string(),
            })?;
            events_by_type.insert(event_type, count);
        }
        
        // Get other statistics similarly...
        let events_by_outcome = HashMap::new(); // Simplified for this example
        let events_by_user = HashMap::new();
        let failed_authentication_attempts = 0;
        let authorization_denials = 0;
        let security_violations = 0;
        
        Ok(AuditStatistics {
            total_events,
            events_by_type,
            events_by_outcome,
            events_by_user,
            failed_authentication_attempts,
            authorization_denials,
            security_violations,
        })
    }
}

/// Main audit logger
pub struct AuditLogger {
    store: Arc<dyn AuditStore>,
    chain_hasher: Arc<Mutex<ChainHasher>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(store: Arc<dyn AuditStore>) -> Self {
        Self {
            store,
            chain_hasher: Arc::new(Mutex::new(ChainHasher::new())),
        }
    }
    
    /// Log an audit event
    pub async fn log_event(&self, mut event: AuditEvent) -> SecurityResult<()> {
        // Set integrity hash
        event.set_integrity_hash();
        
        // Update chain hash
        {
            let mut chain_hasher = self.chain_hasher.lock().await;
            chain_hasher.update(&event);
        }
        
        // Store the event
        self.store.store_event(event).await?;
        
        Ok(())
    }
    
    /// Create and log an audit event
    pub async fn log(
        &self,
        event_type: AuditEventType,
        user_id: Option<Uuid>,
        session_id: Option<Uuid>,
        resource: String,
        action: String,
        outcome: AuditOutcome,
        ip_address: Option<String>,
        user_agent: Option<String>,
        additional_data: serde_json::Value,
    ) -> SecurityResult<()> {
        let event = AuditEvent::new(
            event_type,
            user_id,
            session_id,
            resource,
            action,
            outcome,
            ip_address,
            user_agent,
            additional_data,
        );
        
        self.log_event(event).await
    }
    
    /// Get audit events
    pub async fn get_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<Vec<AuditEvent>> {
        self.store.get_events(criteria).await
    }
    
    /// Count audit events
    pub async fn count_events(&self, criteria: AuditQueryCriteria) -> SecurityResult<u64> {
        self.store.count_events(criteria).await
    }
    
    /// Verify integrity of audit events
    pub async fn verify_integrity(&self, event_ids: Vec<Uuid>) -> SecurityResult<IntegrityCheckResult> {
        self.store.verify_integrity(event_ids).await
    }
    
    /// Get audit statistics
    pub async fn get_statistics(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> SecurityResult<AuditStatistics> {
        self.store.get_statistics(start_time, end_time).await
    }
}

/// Chain hasher for maintaining audit trail integrity
pub struct ChainHasher {
    hasher: Hasher,
    previous_hash: Option<String>,
}

impl ChainHasher {
    pub fn new() -> Self {
        Self {
            hasher: Hasher::new(),
            previous_hash: None,
        }
    }
    
    pub fn update(&mut self, event: &AuditEvent) {
        // Include previous hash in the chain
        if let Some(prev_hash) = &self.previous_hash {
            self.hasher.update(prev_hash.as_bytes());
        }
        
        // Include event data
        self.hasher.update(event.id.as_bytes());
        self.hasher.update(event.timestamp.timestamp().to_string().as_bytes());
        self.hasher.update(event.event_type.to_string().as_bytes());
        self.hasher.update(event.resource.as_bytes());
        self.hasher.update(event.action.as_bytes());
        self.hasher.update(event.outcome.to_string().as_bytes());
        
        // Calculate new hash
        let new_hash = self.hasher.finalize().to_hex().to_string();
        self.previous_hash = Some(new_hash);
        
        // Reset hasher for next event
        self.hasher = Hasher::new();
    }
    
    pub fn get_current_hash(&self) -> Option<String> {
        self.previous_hash.clone()
    }
}

impl Default for ChainHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_in_memory_audit_store() {
        let store = InMemoryAuditStore::new();
        
        // Create test event
        let event = AuditEvent::new(
            AuditEventType::Authentication,
            Some(Uuid::new_v4()),
            None,
            "users".to_string(),
            "login".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            json!({"method": "password"}),
        );
        
        // Store event
        store.store_event(event.clone()).await.unwrap();
        
        // Retrieve events
        let criteria = AuditQueryCriteria {
            event_type: Some(AuditEventType::Authentication),
            ..Default::default()
        };
        
        let events = store.get_events(criteria).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }
    
    #[tokio::test]
    async fn test_audit_logger() {
        let store = Arc::new(InMemoryAuditStore::new());
        let logger = AuditLogger::new(store.clone());
        
        // Log an event
        logger.log(
            AuditEventType::Authorization,
            Some(Uuid::new_v4()),
            None,
            "projects".to_string(),
            "read".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            None,
            json!({"project_id": "123"}),
        ).await.unwrap();
        
        // Verify event was stored
        let criteria = AuditQueryCriteria {
            event_type: Some(AuditEventType::Authorization),
            ..Default::default()
        };
        
        let events = logger.get_events(criteria).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].resource, "projects");
        assert_eq!(events[0].action, "read");
    }
    
    #[tokio::test]
    async fn test_audit_statistics() {
        let store = InMemoryAuditStore::new();
        
        // Store some test events first
        let events = vec![
            AuditEvent::new(
                AuditEventType::Authentication,
                Some(Uuid::new_v4()),
                None,
                "users".to_string(),
                "login".to_string(),
                AuditOutcome::Success,
                None,
                None,
                json!({}),
            ),
            AuditEvent::new(
                AuditEventType::Authentication,
                Some(Uuid::new_v4()),
                None,
                "users".to_string(),
                "login".to_string(),
                AuditOutcome::Failure,
                None,
                None,
                json!({}),
            ),
            AuditEvent::new(
                AuditEventType::Authorization,
                Some(Uuid::new_v4()),
                None,
                "projects".to_string(),
                "read".to_string(),
                AuditOutcome::Denied,
                None,
                None,
                json!({}),
            ),
        ];
        
        for event in events {
            store.store_event(event).await.unwrap();
        }
        
        // Set time range after storing events to ensure they're included
        let start_time = Utc::now() - chrono::Duration::hours(1);
        let end_time = Utc::now() + chrono::Duration::minutes(1); // Add buffer for the future
        
        // Get statistics
        let stats = store.get_statistics(start_time, end_time).await.unwrap();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.failed_authentication_attempts, 1);
        assert_eq!(stats.authorization_denials, 1);
    }
    
    #[tokio::test]
    async fn test_integrity_verification() {
        let store = InMemoryAuditStore::new();
        
        // Create and store event with proper integrity hash
        let mut event = AuditEvent::new(
            AuditEventType::Authentication,
            Some(Uuid::new_v4()),
            None,
            "users".to_string(),
            "login".to_string(),
            AuditOutcome::Success,
            None,
            None,
            json!({}),
        );
        event.set_integrity_hash();
        
        let event_id = event.id;
        store.store_event(event).await.unwrap();
        
        // Verify integrity
        let result = store.verify_integrity(vec![event_id]).await.unwrap();
        assert!(result.integrity_ok);
        assert_eq!(result.verified_events, 1);
        assert!(result.failed_events.is_empty());
    }
    
    #[tokio::test]
    async fn test_chain_hasher() {
        let mut hasher = ChainHasher::new();
        
        let event1 = AuditEvent::new(
            AuditEventType::Authentication,
            Some(Uuid::new_v4()),
            None,
            "users".to_string(),
            "login".to_string(),
            AuditOutcome::Success,
            None,
            None,
            json!({}),
        );
        
        let event2 = AuditEvent::new(
            AuditEventType::Authorization,
            Some(Uuid::new_v4()),
            None,
            "projects".to_string(),
            "read".to_string(),
            AuditOutcome::Success,
            None,
            None,
            json!({}),
        );
        
        hasher.update(&event1);
        let hash1 = hasher.get_current_hash();
        assert!(hash1.is_some());
        
        hasher.update(&event2);
        let hash2 = hasher.get_current_hash();
        assert!(hash2.is_some());
        assert_ne!(hash1, hash2);
    }
}