//! Database Connection Pool Implementation
//!
//! This module provides efficient connection pooling for SQLite databases to improve
//! performance and reduce connection overhead in multi-threaded environments.

use crate::error::{Result, UveddiError};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Configuration for database connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(1800), // 30 minutes
        }
    }
}

/// Pooled connection wrapper
pub struct PooledConnection {
    connection: Connection,
    created_at: Instant,
    last_used: Arc<Mutex<Instant>>,
}

impl PooledConnection {
    fn new(connection: Connection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: Arc::new(Mutex::new(now)),
        }
    }

    pub fn execute_batch(&self, sql: &str) -> rusqlite::Result<()> {
        self.update_last_used();
        self.connection.execute_batch(sql)
    }

    pub fn execute(&self, sql: &str, params: impl rusqlite::Params) -> rusqlite::Result<usize> {
        self.update_last_used();
        self.connection.execute(sql, params)
    }

    pub fn prepare(&self, sql: &str) -> rusqlite::Result<rusqlite::Statement> {
        self.update_last_used();
        self.connection.prepare(sql)
    }

    pub fn transaction(&mut self) -> rusqlite::Result<rusqlite::Transaction> {
        self.update_last_used();
        self.connection.transaction()
    }

    pub fn last_insert_rowid(&self) -> i64 {
        self.update_last_used();
        self.connection.last_insert_rowid()
    }

    fn update_last_used(&self) {
        if let Ok(mut last_used) = self.last_used.lock() {
            *last_used = Instant::now();
        }
    }

    fn is_expired(&self, config: &PoolConfig) -> bool {
        let now = Instant::now();
        
        // Check max lifetime
        if now.duration_since(self.created_at) > config.max_lifetime {
            return true;
        }

        // Check idle timeout
        if let Ok(last_used) = self.last_used.lock() {
            if now.duration_since(*last_used) > config.idle_timeout {
                return true;
            }
        }

        false
    }
}

/// Database connection pool
pub struct DatabasePool {
    db_path: Option<std::path::PathBuf>,
    connections: Arc<Mutex<Vec<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    config: PoolConfig,
}

impl DatabasePool {
    /// Create a new database connection pool
    pub fn new(db_path: Option<&Path>, config: PoolConfig) -> Result<Self> {
        let db_path = db_path.map(|p| p.to_path_buf());
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        
        Ok(Self {
            db_path,
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore,
            config,
        })
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<PooledConnection> {
        // Acquire permit from semaphore (blocks if pool is full)
        let _permit = self.semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| UveddiError::database_error_msg(&format!("Failed to acquire connection permit: {}", e)))?;

        // Try to get existing connection
        if let Ok(mut connections) = self.connections.lock() {
            // Remove expired connections
            connections.retain(|conn| !conn.is_expired(&self.config));
            
            // Return available connection
            if let Some(conn) = connections.pop() {
                return Ok(conn);
            }
        }

        // Create new connection if none available
        self.create_connection()
    }

    /// Create a new database connection
    fn create_connection(&self) -> Result<PooledConnection> {
        let conn = match &self.db_path {
            Some(path) => Connection::open(path).map_err(UveddiError::from)?,
            None => Connection::open_in_memory().map_err(UveddiError::from)?,
        };

        // Configure connection for performance
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
            PRAGMA foreign_keys = ON;
        ")?;

        Ok(PooledConnection::new(conn))
    }

    /// Return connection to pool
    pub async fn return_connection(&self, connection: PooledConnection) -> Result<()> {
        if !connection.is_expired(&self.config) {
            if let Ok(mut connections) = self.connections.lock() {
                connections.push(connection);
            }
        }
        // Connection is automatically dropped if expired or if pool is locked
        Ok(())
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let available_connections = self.connections
            .lock()
            .map(|conns| conns.len())
            .unwrap_or(0);

        let available_permits = self.semaphore.available_permits();

        PoolStats {
            max_connections: self.config.max_connections,
            available_connections,
            active_connections: self.config.max_connections - available_permits,
        }
    }

    /// Clean up expired connections
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut removed = 0;
        if let Ok(mut connections) = self.connections.lock() {
            let initial_len = connections.len();
            connections.retain(|conn| !conn.is_expired(&self.config));
            removed = initial_len - connections.len();
        }
        Ok(removed)
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub max_connections: usize,
    pub available_connections: usize,
    pub active_connections: usize,
}

/// Pool-aware database wrapper
pub struct PooledDatabase {
    pool: Arc<DatabasePool>,
}

impl PooledDatabase {
    /// Create new pooled database
    pub fn new(db_path: Option<&Path>, config: Option<PoolConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        let pool = Arc::new(DatabasePool::new(db_path, config)?);
        
        Ok(Self { pool })
    }

    /// Execute function with pooled connection
    pub async fn with_connection<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&PooledConnection) -> Result<R>,
    {
        let conn = self.pool.get_connection().await?;
        let result = f(&conn)?;
        self.pool.return_connection(conn).await?;
        Ok(result)
    }

    /// Execute function with mutable connection
    pub async fn with_connection_mut<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut PooledConnection) -> Result<R>,
    {
        let mut conn = self.pool.get_connection().await?;
        let result = f(&mut conn)?;
        self.pool.return_connection(conn).await?;
        Ok(result)
    }

    /// Get pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        self.pool.stats()
    }

    /// Cleanup expired connections
    pub async fn cleanup(&self) -> Result<usize> {
        self.pool.cleanup_expired().await
    }
}

impl Clone for PooledDatabase {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_pool_creation() {
        let config = PoolConfig {
            max_connections: 5,
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
        };

        let db = PooledDatabase::new(None, Some(config)).unwrap();
        let stats = db.pool_stats();
        
        assert_eq!(stats.max_connections, 5);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_execution() {
        let db = PooledDatabase::new(None, None).unwrap();
        
        let result = db.with_connection(|conn| {
            conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY)")?;
            conn.execute("INSERT INTO test (id) VALUES (?)", [1])?;
            Ok(())
        }).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_connections() {
        let db = PooledDatabase::new(None, None).unwrap();
        let db_clone = db.clone();

        // Create table first
        db.with_connection(|conn| {
            conn.execute_batch("CREATE TABLE concurrent_test (id INTEGER PRIMARY KEY)")?;
            Ok(())
        }).await.unwrap();

        let handle1 = tokio::spawn(async move {
            for i in 0..5 {
                let _ = db.with_connection(|conn| {
                    conn.execute("INSERT INTO concurrent_test (id) VALUES (?)", [i])?;
                    Ok(())
                }).await;
            }
        });

        let handle2 = tokio::spawn(async move {
            for i in 5..10 {
                let _ = db_clone.with_connection(|conn| {
                    conn.execute("INSERT INTO concurrent_test (id) VALUES (?)", [i])?;
                    Ok(())
                }).await;
            }
        });

        let _ = tokio::join!(handle1, handle2);
    }
}