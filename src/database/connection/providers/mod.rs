//! Database Provider Abstraction Layer
//!
//! This module provides database-agnostic interfaces and implementations
//! for different database backends, enabling seamless migration between
//! SQLite and PostgreSQL based on deployment requirements.

pub mod postgresql_provider;
pub mod sqlite_provider;
pub mod traits;

pub use postgresql_provider::PostgreSqlProvider;
pub use sqlite_provider::SqliteProvider;
pub use traits::{ConnectionProvider, DatabaseProvider, TransactionProvider};

pub use super::config::{DatabaseConfig, DatabaseType};
use crate::error::{Result, UveddiError};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Database health status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseHealthStatus {
    pub is_healthy: bool,
    pub active_connections: u32,
    pub pool_utilization: f32,
    pub last_successful_query: u64,
    pub error_count: u64,
    pub average_query_time: Duration,
}

/// Database metrics for monitoring and alerting
#[derive(Debug, Default)]
pub struct DatabaseMetrics {
    pub active_connections: AtomicU32,
    pub total_queries: AtomicU64,
    pub failed_queries: AtomicU64,
    pub average_query_time: AtomicU64,
    pub last_successful_query: AtomicU64,
    pub connection_errors: AtomicU64,
}

impl DatabaseMetrics {
    pub fn record_query(&self, duration: Duration, success: bool) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);

        if success {
            self.last_successful_query.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                Ordering::Relaxed,
            );

            // Update running average (simplified)
            let current_avg = self.average_query_time.load(Ordering::Relaxed);
            let new_avg = (current_avg + duration.as_millis() as u64) / 2;
            self.average_query_time.store(new_avg, Ordering::Relaxed);
        } else {
            self.failed_queries.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_health_status(&self) -> DatabaseHealthStatus {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let last_query = self.last_successful_query.load(Ordering::Relaxed);
        let total_queries = self.total_queries.load(Ordering::Relaxed);
        let failed_queries = self.failed_queries.load(Ordering::Relaxed);
        let active_conn = self.active_connections.load(Ordering::Relaxed);

        // Calculate health status with proper handling for newly initialized databases
        let is_healthy = if total_queries == 0 {
            // For newly initialized databases with no queries, consider them healthy
            true
        } else {
            // For databases with queries, check recency and failure rate
            let query_recency_ok = last_query > 0 && (now - last_query) < 300; // 5min
                                                                               // Avoid integer division issues: for small query counts, just check for any failures
                                                                               // For larger counts, allow up to 10% failure rate
            let failure_rate_ok = if total_queries < 10 {
                failed_queries == 0
            } else {
                failed_queries < total_queries / 10
            };
            query_recency_ok && failure_rate_ok
        };

        DatabaseHealthStatus {
            is_healthy,
            active_connections: active_conn,
            pool_utilization: 0.0, // Will be set by pool implementation
            last_successful_query: last_query,
            error_count: failed_queries,
            average_query_time: Duration::from_millis(
                self.average_query_time.load(Ordering::Relaxed),
            ),
        }
    }
}

/// Factory function to create appropriate database provider
pub fn create_database_provider(config: &DatabaseConfig) -> Result<Box<dyn DatabaseProvider>> {
    match config.database_type {
        DatabaseType::SQLite => {
            let provider = SqliteProvider::new(config.clone())?;
            Ok(Box::new(provider))
        }
        DatabaseType::PostgreSQL => {
            let provider = PostgreSqlProvider::new(config.clone())?;
            Ok(Box::new(provider))
        }
    }
}
