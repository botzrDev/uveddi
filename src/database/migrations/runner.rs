//! Migration runner implementation
//!
//! Handles executing migrations against SQLite and PostgreSQL databases

use super::{Migration, MigrationRecord, MigrationRegistry, MigrationResult};
use crate::database::connection::{ConnectionPool, DatabaseType};
use chrono::Utc;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Migration runner that executes migrations against a database
pub struct MigrationRunner {
    pool: Arc<ConnectionPool>,
    registry: MigrationRegistry,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new(pool: Arc<ConnectionPool>, registry: MigrationRegistry) -> Self {
        Self { pool, registry }
    }

    /// Initialize the migration tracking table
    pub async fn initialize(&self) -> Result<(), MigrationError> {
        info!("Initializing migration tracking table");

        let create_table_sql = match self.pool.database_type() {
            DatabaseType::SQLite => {
                r#"
                CREATE TABLE IF NOT EXISTS migration_history (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at TEXT NOT NULL,
                    checksum TEXT NOT NULL
                );
                "#
            }
            DatabaseType::PostgreSQL => {
                r#"
                CREATE TABLE IF NOT EXISTS migration_history (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at TIMESTAMP WITH TIME ZONE NOT NULL,
                    checksum TEXT NOT NULL
                );
                "#
            }
        };

        // Execute the table creation using spawn_blocking for SQLite compatibility
        let pool = Arc::clone(&self.pool);
        let sql = create_table_sql.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()?;
            conn.execute(&sql, [])?;
            Ok::<(), MigrationError>(())
        })
        .await
        .map_err(|e| MigrationError::Runtime(format!("Task join error: {}", e)))??;

        info!("Migration tracking table initialized");
        Ok(())
    }

    /// Get the current migration version
    pub async fn current_version(&self) -> Result<u32, MigrationError> {
        let pool = Arc::clone(&self.pool);

        let version = tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()?;

            let mut stmt = conn.prepare("SELECT MAX(version) FROM migration_history")?;
            let rows: Result<Option<u32>, rusqlite::Error> = stmt.query_row([], |row| {
                Ok(row.get::<_, Option<u32>>(0)?)
            });

            match rows {
                Ok(Some(version)) => Ok(version),
                Ok(None) => Ok(0), // No migrations applied yet
                Err(rusqlite::Error::SqliteFailure(err, _))
                    if err.code == rusqlite::ErrorCode::DatabaseCorrupt => {
                    // Table doesn't exist yet
                    Ok(0)
                }
                Err(e) => Err(MigrationError::Database(e)),
            }
        })
        .await
        .map_err(|e| MigrationError::Runtime(format!("Task join error: {}", e)))??;

        Ok(version)
    }

    /// Get all applied migration records
    pub async fn applied_migrations(&self) -> Result<Vec<MigrationRecord>, MigrationError> {
        let pool = Arc::clone(&self.pool);

        let records = tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()?;

            let mut stmt = conn.prepare(
                "SELECT version, name, applied_at, checksum FROM migration_history ORDER BY version"
            )?;

            let rows = stmt.query_map([], |row| {
                let applied_at_str: String = row.get(2)?;
                let applied_at = applied_at_str.parse::<chrono::DateTime<Utc>>()
                    .map_err(|_| rusqlite::Error::InvalidColumnType(2, "applied_at".to_string(), rusqlite::types::Type::Text))?;

                Ok(MigrationRecord {
                    version: row.get(0)?,
                    name: row.get(1)?,
                    applied_at,
                    checksum: row.get(3)?,
                })
            })?;

            let mut records = Vec::new();
            for row_result in rows {
                records.push(row_result?);
            }

            Ok::<Vec<MigrationRecord>, MigrationError>(records)
        })
        .await
        .map_err(|e| MigrationError::Runtime(format!("Task join error: {}", e)))??;

        Ok(records)
    }

    /// Run all pending migrations
    pub async fn run_pending_migrations(&self) -> Result<Vec<MigrationResult>, MigrationError> {
        info!("Starting migration run");

        // Ensure migration table exists
        self.initialize().await?;

        let current_version = self.current_version().await?;
        info!("Current database version: {}", current_version);

        let pending_migrations = self.registry.get_pending(current_version);
        if pending_migrations.is_empty() {
            info!("No pending migrations");
            return Ok(vec![]);
        }

        info!("Found {} pending migrations", pending_migrations.len());

        let mut results = Vec::new();
        for migration in pending_migrations {
            match self.apply_migration(migration).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    error!("Migration {} failed: {}", migration.version, e);
                    results.push(MigrationResult::Failed {
                        version: migration.version,
                        error: e.to_string(),
                    });
                    // Stop on first failure
                    break;
                }
            }
        }

        info!("Migration run completed with {} results", results.len());
        Ok(results)
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &Migration) -> Result<MigrationResult, MigrationError> {
        info!("Applying migration {}: {}", migration.version, migration.name);

        let pool = Arc::clone(&self.pool);
        let migration_clone = migration.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()?;

            // Start transaction
            let tx = conn.unchecked_transaction()?;

            // Execute the migration SQL
            tx.execute_batch(&migration_clone.up_sql)?;

            // Record the migration
            let applied_at = Utc::now().to_rfc3339();
            let checksum = migration_clone.checksum();

            tx.execute(
                "INSERT INTO migration_history (version, name, applied_at, checksum) VALUES (?, ?, ?, ?)",
                (migration_clone.version, &migration_clone.name, &applied_at, &checksum),
            )?;

            // Commit transaction
            tx.commit()?;

            Ok::<MigrationResult, MigrationError>(MigrationResult::Applied {
                version: migration_clone.version,
                name: migration_clone.name.clone(),
            })
        })
        .await
        .map_err(|e| MigrationError::Runtime(format!("Task join error: {}", e)))??;

        info!("Successfully applied migration {}", migration.version);
        Ok(MigrationResult::Applied {
            version: migration.version,
            name: migration.name.clone(),
        })
    }

    /// Rollback to a specific migration version
    pub async fn rollback_to_version(&self, target_version: u32) -> Result<Vec<MigrationResult>, MigrationError> {
        info!("Rolling back to version {}", target_version);

        let current_version = self.current_version().await?;
        if current_version <= target_version {
            warn!("Current version {} is already at or below target {}", current_version, target_version);
            return Ok(vec![]);
        }

        let applied = self.applied_migrations().await?;
        let mut results = Vec::new();

        // Find migrations to rollback (in reverse order)
        let mut to_rollback: Vec<_> = applied
            .iter()
            .filter(|r| r.version > target_version)
            .collect();
        to_rollback.sort_by(|a, b| b.version.cmp(&a.version)); // Descending order

        for record in to_rollback {
            if let Some(migration) = self.registry.get(record.version) {
                match self.rollback_migration(migration, record).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        error!("Rollback of migration {} failed: {}", record.version, e);
                        results.push(MigrationResult::Failed {
                            version: record.version,
                            error: e.to_string(),
                        });
                        break;
                    }
                }
            } else {
                warn!("Migration {} not found in registry for rollback", record.version);
                results.push(MigrationResult::Failed {
                    version: record.version,
                    error: "Migration not found in registry".to_string(),
                });
            }
        }

        info!("Rollback completed");
        Ok(results)
    }

    /// Rollback a single migration
    async fn rollback_migration(
        &self,
        migration: &Migration,
        record: &MigrationRecord,
    ) -> Result<MigrationResult, MigrationError> {
        info!("Rolling back migration {}: {}", migration.version, migration.name);

        let pool = Arc::clone(&self.pool);
        let migration_clone = migration.clone();
        let record_version = record.version;

        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()?;

            // Start transaction
            let tx = conn.unchecked_transaction()?;

            // Execute rollback SQL
            tx.execute_batch(&migration_clone.down_sql)?;

            // Remove migration record
            tx.execute(
                "DELETE FROM migration_history WHERE version = ?",
                [record_version],
            )?;

            // Commit transaction
            tx.commit()?;

            Ok::<MigrationResult, MigrationError>(MigrationResult::Applied {
                version: migration_clone.version,
                name: format!("Rollback: {}", migration_clone.name),
            })
        })
        .await
        .map_err(|e| MigrationError::Runtime(format!("Task join error: {}", e)))??;

        info!("Successfully rolled back migration {}", migration.version);
        Ok(MigrationResult::Applied {
            version: migration.version,
            name: format!("Rollback: {}", migration.name),
        })
    }
}

/// Migration-specific error type
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Connection pool error: {0}")]
    Pool(String),

    #[error("Migration validation error: {0}")]
    Validation(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Migration not found: version {0}")]
    NotFound(u32),
}

// TODO: Implement ConnectionError type in connection module
// impl From<crate::database::connection::ConnectionError> for MigrationError {
//     fn from(err: crate::database::connection::ConnectionError) -> Self {
//         MigrationError::Pool(err.to_string())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::{ConnectionPool, DatabaseConfig, DatabaseType, PoolConfig};
    use std::time::Duration;
    use tempfile::NamedTempFile;

    async fn create_test_runner() -> (MigrationRunner, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let config = DatabaseConfig {
            database_type: DatabaseType::SQLite,
            connection_string: format!("file:{}", temp_file.path().display()),
            read_connection_strings: vec![],
            pool: PoolConfig {
                max_connections: 5,
                min_connections: 1,
                connection_timeout: Duration::from_secs(5),
                idle_timeout: Duration::from_secs(300),
                max_lifetime: Duration::from_secs(3600),
            },
            enable_metrics: false,
            enable_logging: false,
            enable_prepared_statements: false,
        };

        let pool = Arc::new(ConnectionPool::new(config).await.unwrap());
        let registry = MigrationRegistry::new();

        (MigrationRunner::new(pool, registry), temp_file)
    }

    #[tokio::test]
    async fn test_initialize() {
        let (runner, _temp_file) = create_test_runner().await;

        runner.initialize().await.unwrap();

        // Should be able to get current version
        let version = runner.current_version().await.unwrap();
        assert_eq!(version, 0);
    }
}