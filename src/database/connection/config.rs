//! Database Configuration
//!
//! Provides configuration abstractions for database connections and pooling.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Database type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    Sqlite,
    #[cfg(feature = "postgresql")]
    Postgresql,
}

/// Database configuration abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Type of database
    pub database_type: DatabaseType,
    /// Connection string or path
    pub connection_string: String,
    /// Pool configuration
    pub pool: PoolConfig,
    /// Enable connection metrics
    pub enable_metrics: bool,
}

impl DatabaseConfig {
    /// Create SQLite configuration
    pub fn sqlite(path: impl Into<PathBuf>) -> Self {
        Self {
            database_type: DatabaseType::Sqlite,
            connection_string: path.into().to_string_lossy().to_string(),
            pool: PoolConfig::default(),
            enable_metrics: false,
        }
    }

    /// Create PostgreSQL configuration
    #[cfg(feature = "postgresql")]
    pub fn postgresql(connection_string: impl Into<String>) -> Self {
        Self {
            database_type: DatabaseType::Postgresql,
            connection_string: connection_string.into(),
            pool: PoolConfig::default(),
            enable_metrics: false,
        }
    }

    /// Set pool configuration
    pub fn with_pool(mut self, pool: PoolConfig) -> Self {
        self.pool = pool;
        self
    }

    /// Enable metrics collection
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::sqlite("uveddi.db")
    }
}

/// Configuration for database connection pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub test_on_checkout: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(1800), // 30 minutes
            test_on_checkout: true,
        }
    }
}

impl PoolConfig {
    /// Builder pattern for pool configuration
    pub fn builder() -> PoolConfigBuilder {
        PoolConfigBuilder::default()
    }
}

/// Builder for PoolConfig
#[derive(Default)]
pub struct PoolConfigBuilder {
    max_connections: Option<usize>,
    min_connections: Option<usize>,
    connection_timeout: Option<Duration>,
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
    test_on_checkout: Option<bool>,
}

impl PoolConfigBuilder {
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn min_connections(mut self, min: usize) -> Self {
        self.min_connections = Some(min);
        self
    }

    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = Some(timeout);
        self
    }

    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = Some(lifetime);
        self
    }

    pub fn test_on_checkout(mut self, test: bool) -> Self {
        self.test_on_checkout = Some(test);
        self
    }

    pub fn build(self) -> PoolConfig {
        let default = PoolConfig::default();
        PoolConfig {
            max_connections: self.max_connections.unwrap_or(default.max_connections),
            min_connections: self.min_connections.unwrap_or(default.min_connections),
            connection_timeout: self.connection_timeout.unwrap_or(default.connection_timeout),
            idle_timeout: self.idle_timeout.unwrap_or(default.idle_timeout),
            max_lifetime: self.max_lifetime.unwrap_or(default.max_lifetime),
            test_on_checkout: self.test_on_checkout.unwrap_or(default.test_on_checkout),
        }
    }
}