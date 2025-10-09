//! Database Connection Infrastructure
//!
//! Provides connection management, pooling, and provider abstractions.

pub mod config;
pub mod pool;
pub mod providers;

pub use config::{DatabaseConfig, DatabaseType, PoolConfig, PoolConfigBuilder};
pub use pool::{ConnectionPool, PooledConnection};
pub use providers::{DatabaseProvider, SqliteProvider};

#[cfg(feature = "full")]
pub use providers::PostgreSqlProvider;

use crate::error::{Result, UveddiError};
use std::sync::Arc;

/// Database connection abstraction
pub trait DatabaseConnection: Send + Sync {
    /// Execute a SQL query with parameters
    fn execute(&self, query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize>;

    /// Execute a batch of SQL statements
    fn execute_batch(&self, sql: &str) -> Result<()>;

    /// Query rows from the database
    fn query<T, F>(&self, query: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<Vec<T>>
    where
        Self: Sized,
        T: Send + 'static,
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T> + Send;

    /// Query a single row from the database
    fn query_row<T, F>(
        &self,
        query: &str,
        params: &[&dyn rusqlite::ToSql],
        f: F,
    ) -> Result<Option<T>>
    where
        Self: Sized,
        T: Send + 'static,
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T> + Send;

    /// Test the connection
    fn test_connection(&self) -> Result<()>;

    /// Get connection statistics
    fn stats(&self) -> ConnectionStats;
}

/// Database transaction abstraction
pub trait DatabaseTransaction: Send {
    /// Commit the transaction
    fn commit(self: Box<Self>) -> Result<()>;

    /// Rollback the transaction
    fn rollback(self: Box<Self>) -> Result<()>;

    /// Execute within transaction
    fn execute(&mut self, query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize>;
}

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub wait_count: usize,
    pub wait_duration: std::time::Duration,
}

/// Connection manager for creating and managing database connections
pub struct ConnectionManager {
    config: DatabaseConfig,
    provider: Arc<dyn DatabaseProvider>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        let provider: Arc<dyn DatabaseProvider> = match config.database_type {
            DatabaseType::SQLite => Arc::new(SqliteProvider::new(config.clone())?),
            #[cfg(feature = "full")]
            DatabaseType::PostgreSQL => Arc::new(PostgreSqlProvider::new(&config)?),
            #[cfg(not(feature = "full"))]
            DatabaseType::PostgreSQL => {
                return Err(UveddiError::DatabaseConnection(
                    "PostgreSQL support not enabled. Enable with 'full' feature.".to_string(),
                ))
            }
        };

        Ok(Self { config, provider })
    }

    /// Create a connection pool
    pub async fn create_pool(&self) -> Result<Arc<ConnectionPool>> {
        ConnectionPool::new(self.config.clone(), self.provider.clone()).await
    }

    /// Get a single connection (non-pooled)
    pub async fn connect(&self) -> Result<Box<dyn providers::traits::DatabaseConnection>> {
        self.provider.get_connection().await
    }

    /// Get the configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get the provider
    pub fn provider(&self) -> &dyn DatabaseProvider {
        self.provider.as_ref()
    }
}

/// Builder pattern for ConnectionManager
pub struct ConnectionManagerBuilder {
    config: Option<DatabaseConfig>,
}

impl ConnectionManagerBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn with_config(mut self, config: DatabaseConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<ConnectionManager> {
        let config = self.config.ok_or_else(|| {
            UveddiError::Configuration("Database configuration not provided".to_string())
        })?;
        ConnectionManager::new(config)
    }
}

impl Default for ConnectionManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_stats_default() {
        let stats = ConnectionStats::default();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.idle_connections, 0);
    }

    #[test]
    fn test_database_config_builder() {
        let config = DatabaseConfig::sqlite("test.db")
            .with_metrics(true)
            .with_pool(
                PoolConfig::builder()
                    .max_connections(20)
                    .min_connections(2)
                    .build(),
            );

        assert!(matches!(config.database_type, DatabaseType::SQLite));
        assert_eq!(config.pool.max_connections, 20);
        assert_eq!(config.pool.min_connections, 2);
        assert!(config.enable_metrics);
    }
}
