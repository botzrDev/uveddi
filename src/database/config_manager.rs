//! Database Configuration Management
//!
//! This module provides comprehensive configuration management for database
//! providers, connection settings, and environment-specific configurations.

use super::connection::config::{DatabaseConfig, DatabaseType, PoolConfig};
use crate::error::{Result, UveddiError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{info, warn};

/// Database configuration manager
pub struct DatabaseConfigManager {
    config: DatabaseEnvironmentConfig,
    environment: Environment,
}

impl DatabaseConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let environment = Self::detect_environment();
        let config = Self::load_config(&environment)?;

        Ok(Self {
            config,
            environment,
        })
    }

    /// Create from explicit configuration file
    pub fn from_file<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_content = std::fs::read_to_string(config_path.as_ref()).map_err(|e| {
            UveddiError::io_error("read_to_string", &config_path.as_ref().to_string_lossy(), e)
        })?;

        let config: DatabaseEnvironmentConfig =
            match config_path.as_ref().extension().and_then(|s| s.to_str()) {
                Some("toml") => toml::from_str(&config_content).map_err(|e| {
                    UveddiError::configuration_error(&format!("Failed to parse TOML config: {}", e))
                })?,
                Some("json") => serde_json::from_str(&config_content).map_err(|e| {
                    UveddiError::configuration_error(&format!("Failed to parse JSON config: {}", e))
                })?,
                _ => {
                    return Err(UveddiError::configuration_error(
                        "Unsupported config file format (only .toml and .json supported)",
                    ))
                }
            };

        let environment = Self::detect_environment();

        Ok(Self {
            config,
            environment,
        })
    }

    /// Detect current environment
    fn detect_environment() -> Environment {
        match env::var("UVEDDI_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            "test" | "testing" => Environment::Test,
            _ => Environment::Development,
        }
    }

    /// Load configuration for the detected environment
    fn load_config(environment: &Environment) -> Result<DatabaseEnvironmentConfig> {
        // First try to load from environment-specific files
        let config_paths = vec![
            format!("database-{}.toml", environment.as_str()),
            format!("config/database-{}.toml", environment.as_str()),
            "database.toml".to_string(),
            "config/database.toml".to_string(),
        ];

        for config_path in config_paths {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                info!("Loading database config from: {}", config_path);
                return toml::from_str(&content).map_err(|e| {
                    UveddiError::configuration_error(&format!("Failed to parse config: {}", e))
                });
            }
        }

        // Fall back to environment variables or defaults
        info!("No config file found, using environment variables and defaults");
        Ok(Self::load_from_environment(environment))
    }

    /// Load configuration from environment variables
    fn load_from_environment(environment: &Environment) -> DatabaseEnvironmentConfig {
        let default_database_type = match environment {
            Environment::Production | Environment::Staging => DatabaseType::PostgreSQL,
            _ => DatabaseType::SQLite,
        };

        let database_type = env::var("DATABASE_TYPE")
            .unwrap_or_else(|_| format!("{:?}", default_database_type))
            .parse::<DatabaseType>()
            .unwrap_or(default_database_type);

        let connection_string = env::var("DATABASE_URL").unwrap_or_else(|_| match database_type {
            DatabaseType::SQLite => "./uveddi.db".to_string(),
            DatabaseType::PostgreSQL => "postgresql://localhost:5432/uveddi".to_string(),
        });

        let read_connection_strings: Vec<String> = env::var("DATABASE_READ_URLS")
            .map(|urls| urls.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        DatabaseEnvironmentConfig {
            development: Self::create_config_for_env(
                Environment::Development,
                &database_type,
                &connection_string,
                &read_connection_strings,
            ),
            staging: Self::create_config_for_env(
                Environment::Staging,
                &database_type,
                &connection_string,
                &read_connection_strings,
            ),
            production: Self::create_config_for_env(
                Environment::Production,
                &database_type,
                &connection_string,
                &read_connection_strings,
            ),
            test: Self::create_config_for_env(
                Environment::Test,
                &database_type,
                &connection_string,
                &read_connection_strings,
            ),
        }
    }

    /// Create configuration for specific environment
    fn create_config_for_env(
        env: Environment,
        database_type: &DatabaseType,
        connection_string: &str,
        read_connection_strings: &[String],
    ) -> DatabaseConfig {
        let (max_connections, connection_timeout, pool_timeout) = match env {
            Environment::Production => (100usize, Duration::from_secs(30), Duration::from_secs(30)),
            Environment::Staging => (50usize, Duration::from_secs(20), Duration::from_secs(20)),
            Environment::Test => (5usize, Duration::from_secs(10), Duration::from_secs(5)),
            Environment::Development => (20usize, Duration::from_secs(30), Duration::from_secs(15)),
        };

        let pool = PoolConfig {
            max_connections,
            min_connections: max_connections / 4,
            connection_timeout,
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            test_on_checkout: true,
            pool_timeout,
        };

        DatabaseConfig {
            database_type: database_type.clone(),
            connection_string: connection_string.to_string(),
            read_connection_strings: read_connection_strings.to_vec(),
            pool,
            enable_metrics: false,
            enable_logging: matches!(env, Environment::Development | Environment::Test),
            enable_prepared_statements: true,
        }
    }

    /// Get configuration for current environment
    pub fn get_config(&self) -> &DatabaseConfig {
        match self.environment {
            Environment::Development => &self.config.development,
            Environment::Staging => &self.config.staging,
            Environment::Production => &self.config.production,
            Environment::Test => &self.config.test,
        }
    }

    /// Get configuration for specific environment
    pub fn get_config_for_env(&self, env: Environment) -> &DatabaseConfig {
        match env {
            Environment::Development => &self.config.development,
            Environment::Staging => &self.config.staging,
            Environment::Production => &self.config.production,
            Environment::Test => &self.config.test,
        }
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<()> {
        let config = self.get_config();

        // Validate connection string format
        match config.database_type {
            DatabaseType::SQLite => {
                if config.connection_string.is_empty() {
                    return Err(UveddiError::configuration_error(
                        "SQLite connection string cannot be empty",
                    ));
                }
            }
            DatabaseType::PostgreSQL => {
                if !config.connection_string.starts_with("postgresql://")
                    && !config.connection_string.starts_with("postgres://")
                {
                    return Err(UveddiError::configuration_error(
                        "Invalid PostgreSQL connection string format",
                    ));
                }
            }
        }

        // Validate connection pool settings
        if config.pool.max_connections == 0 {
            return Err(UveddiError::configuration_error(
                "max_connections must be greater than 0",
            ));
        }

        if config.pool.min_connections > config.pool.max_connections {
            return Err(UveddiError::configuration_error(
                "min_connections cannot be greater than max_connections",
            ));
        }

        // Validate timeouts
        if config.pool.connection_timeout.as_secs() == 0 {
            return Err(UveddiError::configuration_error(
                "connection_timeout must be greater than 0",
            ));
        }

        // Environment-specific validations
        match self.environment {
            Environment::Production => {
                if config.enable_logging {
                    warn!(
                        "Database logging is enabled in production - this may impact performance"
                    );
                }

                if config.pool.max_connections < 50 {
                    warn!("Low max_connections setting for production environment");
                }

                if config.database_type == DatabaseType::SQLite {
                    warn!(
                        "Using SQLite in production - consider PostgreSQL for better scalability"
                    );
                }
            }
            Environment::Test => {
                if config.pool.max_connections > 10 {
                    warn!("High max_connections setting for test environment");
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get current environment
    pub fn get_environment(&self) -> Environment {
        self.environment
    }

    /// Create configuration template file
    pub fn create_template<P: AsRef<Path>>(output_path: P) -> Result<()> {
        let template = DatabaseEnvironmentConfig::default();
        let toml_content = toml::to_string_pretty(&template).map_err(|e| {
            UveddiError::configuration_error(&format!("Failed to serialize template: {}", e))
        })?;

        std::fs::write(output_path.as_ref(), toml_content).map_err(|e| {
            UveddiError::io_error("write", &output_path.as_ref().to_string_lossy(), e)
        })?;

        info!(
            "Created database configuration template at: {}",
            output_path.as_ref().display()
        );
        Ok(())
    }

    /// Load secrets from external sources (environment, secrets manager, etc.)
    pub fn load_secrets(&mut self) -> Result<()> {
        self.load_from_env_vars()?;

        // Could extend to load from:
        // - AWS Secrets Manager
        // - HashiCorp Vault
        // - Kubernetes Secrets
        // - Azure Key Vault

        Ok(())
    }

    /// Load sensitive configuration from environment variables
    fn load_from_env_vars(&mut self) -> Result<()> {
        // Override database URLs if set in environment
        if let Ok(database_url) = env::var("DATABASE_URL") {
            match self.environment {
                Environment::Development => {
                    self.config.development.connection_string = database_url
                }
                Environment::Staging => self.config.staging.connection_string = database_url,
                Environment::Production => self.config.production.connection_string = database_url,
                Environment::Test => self.config.test.connection_string = database_url,
            }
        }

        // Override read URLs if set
        if let Ok(read_urls) = env::var("DATABASE_READ_URLS") {
            let urls: Vec<String> = read_urls.split(',').map(|s| s.trim().to_string()).collect();
            match self.environment {
                Environment::Development => self.config.development.read_connection_strings = urls,
                Environment::Staging => self.config.staging.read_connection_strings = urls,
                Environment::Production => self.config.production.read_connection_strings = urls,
                Environment::Test => self.config.test.read_connection_strings = urls,
            }
        }

        // Override connection pool settings if set
        if let Ok(max_conn_str) = env::var("DATABASE_MAX_CONNECTIONS") {
            if let Ok(max_conn) = max_conn_str.parse::<usize>() {
                match self.environment {
                    Environment::Development => {
                        self.config.development.pool.max_connections = max_conn
                    }
                    Environment::Staging => {
                        self.config.staging.pool.max_connections = max_conn
                    }
                    Environment::Production => {
                        self.config.production.pool.max_connections = max_conn
                    }
                    Environment::Test => self.config.test.pool.max_connections = max_conn,
                }
            }
        }

        Ok(())
    }

    /// Get connection string with credentials masked for logging
    pub fn get_masked_connection_string(&self) -> String {
        let config = self.get_config();
        mask_connection_string(&config.connection_string)
    }
}

impl Default for DatabaseConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default database config manager")
    }
}

/// Environment enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Test,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
            Environment::Test => "test",
        }
    }
}

impl std::str::FromStr for DatabaseType {
    type Err = UveddiError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sqlite" => Ok(DatabaseType::SQLite),
            "postgresql" | "postgres" => Ok(DatabaseType::PostgreSQL),
            _ => Err(UveddiError::configuration_error(&format!(
                "Unknown database type: {}",
                s
            ))),
        }
    }
}

/// Multi-environment database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseEnvironmentConfig {
    pub development: DatabaseConfig,
    pub staging: DatabaseConfig,
    pub production: DatabaseConfig,
    pub test: DatabaseConfig,
}

impl Default for DatabaseEnvironmentConfig {
    fn default() -> Self {
        Self {
            development: DatabaseConfig {
                database_type: DatabaseType::SQLite,
                connection_string: "./uveddi-dev.db".to_string(),
                read_connection_strings: Vec::new(),
                pool: PoolConfig {
                    max_connections: 20,
                    min_connections: 5,
                    connection_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                    test_on_checkout: true,
                    pool_timeout: Duration::from_secs(15),
                },
                enable_metrics: false,
                enable_logging: true,
                enable_prepared_statements: true,
            },
            staging: DatabaseConfig {
                database_type: DatabaseType::PostgreSQL,
                connection_string: "postgresql://user:password@localhost:5432/uveddi_staging"
                    .to_string(),
                read_connection_strings: vec![
                    "postgresql://user:password@read-replica1:5432/uveddi_staging".to_string(),
                ],
                pool: PoolConfig {
                    max_connections: 50,
                    min_connections: 10,
                    connection_timeout: Duration::from_secs(20),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                    test_on_checkout: true,
                    pool_timeout: Duration::from_secs(20),
                },
                enable_metrics: false,
                enable_logging: false,
                enable_prepared_statements: true,
            },
            production: DatabaseConfig {
                database_type: DatabaseType::PostgreSQL,
                connection_string: "postgresql://user:password@db-primary:5432/uveddi".to_string(),
                read_connection_strings: vec![
                    "postgresql://user:password@db-read1:5432/uveddi".to_string(),
                    "postgresql://user:password@db-read2:5432/uveddi".to_string(),
                ],
                pool: PoolConfig {
                    max_connections: 100,
                    min_connections: 25,
                    connection_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                    test_on_checkout: true,
                    pool_timeout: Duration::from_secs(30),
                },
                enable_metrics: true,
                enable_logging: false,
                enable_prepared_statements: true,
            },
            test: DatabaseConfig {
                database_type: DatabaseType::SQLite,
                connection_string: ":memory:".to_string(),
                read_connection_strings: Vec::new(),
                pool: PoolConfig {
                    max_connections: 5,
                    min_connections: 1,
                    connection_timeout: Duration::from_secs(10),
                    idle_timeout: Duration::from_secs(300),
                    max_lifetime: Duration::from_secs(600),
                    test_on_checkout: true,
                    pool_timeout: Duration::from_secs(5),
                },
                enable_metrics: false,
                enable_logging: true,
                enable_prepared_statements: true,
            },
        }
    }
}

/// Mask sensitive information in connection strings for logging
fn mask_connection_string(connection_string: &str) -> String {
    if connection_string.contains("://") {
        // Handle PostgreSQL-style URLs
        let parts: Vec<&str> = connection_string.split('@').collect();
        if parts.len() == 2 {
            let host_and_rest = parts[1];
            let scheme_and_creds: Vec<&str> = parts[0].split("://").collect();
            if scheme_and_creds.len() == 2 {
                let scheme = scheme_and_creds[0];
                return format!("{}://***@{}", scheme, host_and_rest);
            }
        }
    }

    // For SQLite or other formats, just mask if it looks like a sensitive path
    if connection_string.contains("password") || connection_string.contains("secret") {
        return "***".to_string();
    }

    connection_string.to_string()
}

/// Configuration builder for fluent API
pub struct DatabaseConfigBuilder {
    config: DatabaseConfig,
}

impl DatabaseConfigBuilder {
    pub fn new(database_type: DatabaseType) -> Self {
        Self {
            config: DatabaseConfig {
                database_type,
                ..DatabaseConfig::default()
            },
        }
    }

    pub fn connection_string<S: Into<String>>(mut self, connection_string: S) -> Self {
        self.config.connection_string = connection_string.into();
        self
    }

    pub fn read_connections(mut self, read_connections: Vec<String>) -> Self {
        self.config.read_connection_strings = read_connections;
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.pool.max_connections = max;
        self
    }

    pub fn min_connections(mut self, min: usize) -> Self {
        self.config.pool.min_connections = min;
        self
    }

    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.config.pool.connection_timeout = timeout;
        self
    }

    pub fn enable_prepared_statements(mut self, enable: bool) -> Self {
        self.config.enable_prepared_statements = enable;
        self
    }

    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.config.enable_logging = enable;
        self
    }

    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    pub fn build(self) -> DatabaseConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_connection_string() {
        let pg_url = "postgresql://user:password@localhost:5432/dbname";
        let masked = mask_connection_string(pg_url);
        assert_eq!(masked, "postgresql://***@localhost:5432/dbname");

        let sqlite_path = "./test.db";
        let masked_sqlite = mask_connection_string(sqlite_path);
        assert_eq!(masked_sqlite, "./test.db");
    }

    #[test]
    fn test_config_builder() {
        let config = DatabaseConfigBuilder::new(DatabaseType::PostgreSQL)
            .connection_string("postgresql://localhost:5432/test")
            .max_connections(50)
            .enable_logging(false)
            .build();

        assert_eq!(config.database_type, DatabaseType::PostgreSQL);
        assert_eq!(config.connection_string, "postgresql://localhost:5432/test");
        assert_eq!(config.pool.max_connections, 50);
        assert!(!config.enable_logging);
    }

    #[test]
    fn test_environment_detection() {
        // Test default environment
        let env = DatabaseConfigManager::detect_environment();
        assert_eq!(env, Environment::Development); // Default when UVEDDI_ENV is not set
    }
}
