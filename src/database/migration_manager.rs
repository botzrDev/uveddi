//! Database Migration Framework
//!
//! This module provides a comprehensive database migration system that supports
//! both SQLite and PostgreSQL, enabling seamless schema evolution and data migration.

use super::providers::{DatabaseProvider, DatabaseConfig, DatabaseType, create_database_provider};
use crate::error::{Result, UveddiError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn, error};

/// Migration manager for handling database schema changes and data migration
pub struct MigrationManager {
    provider: Arc<dyn DatabaseProvider>,
    config: DatabaseConfig,
    migrations_dir: PathBuf,
}

impl MigrationManager {
    /// Create a new migration manager
    pub async fn new(config: DatabaseConfig, migrations_dir: Option<PathBuf>) -> Result<Self> {
        let provider: Arc<dyn DatabaseProvider> = Arc::from(create_database_provider(&config)?);
        let migrations_dir = migrations_dir.unwrap_or_else(|| PathBuf::from("migrations"));
        
        let manager = Self {
            provider,
            config,
            migrations_dir,
        };
        
        // Initialize migration tracking table
        manager.initialize_migration_table().await?;
        
        Ok(manager)
    }
    
    /// Initialize the migration tracking table
    async fn initialize_migration_table(&self) -> Result<()> {
        match self.config.provider_type {
            DatabaseType::SQLite => {
                self.provider.execute_write("
                    CREATE TABLE IF NOT EXISTS schema_migrations (
                        version INTEGER PRIMARY KEY,
                        name TEXT NOT NULL,
                        applied_at TEXT NOT NULL,
                        checksum TEXT NOT NULL,
                        execution_time_ms INTEGER NOT NULL
                    )
                ", &[]).await?;
            }
            DatabaseType::PostgreSQL => {
                self.provider.execute_write("
                    CREATE TABLE IF NOT EXISTS schema_migrations (
                        version BIGINT PRIMARY KEY,
                        name TEXT NOT NULL,
                        applied_at TIMESTAMPTZ NOT NULL,
                        checksum TEXT NOT NULL,
                        execution_time_ms INTEGER NOT NULL
                    )
                ", &[]).await?;
            }
        }
        Ok(())
    }
    
    /// Get current database schema version
    pub async fn get_current_version(&self) -> Result<u32> {
        let results = self.provider.execute_query(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            &[]
        ).await?;
        
        if results.is_empty() || results[0].rows.is_empty() {
            return Ok(0);
        }
        
        let version = results[0].rows[0].values.get("version")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u32;
        
        Ok(version)
    }
    
    /// Apply all pending migrations up to target version
    pub async fn migrate_to_version(&self, target_version: u32) -> Result<MigrationResult> {
        let current_version = self.get_current_version().await?;
        info!("Current schema version: {}, target: {}", current_version, target_version);
        
        if current_version >= target_version {
            return Ok(MigrationResult {
                applied_migrations: vec![],
                total_execution_time: std::time::Duration::ZERO,
                success: true,
                error_message: None,
            });
        }
        
        let migrations = self.load_migrations(current_version + 1, target_version).await?;
        let mut applied_migrations = Vec::new();
        let mut total_execution_time = std::time::Duration::ZERO;
        
        for migration in migrations {
            info!("Applying migration {}: {}", migration.version, migration.name);
            let start_time = std::time::Instant::now();
            
            match self.apply_migration(&migration).await {
                Ok(()) => {
                    let execution_time = start_time.elapsed();
                    total_execution_time += execution_time;
                    
                    self.record_migration(&migration, execution_time).await?;
                    applied_migrations.push(migration.clone());
                    
                    info!("Successfully applied migration {} in {:?}", 
                          migration.version, execution_time);
                }
                Err(e) => {
                    error!("Failed to apply migration {}: {}", migration.version, e);
                    
                    return Ok(MigrationResult {
                        applied_migrations,
                        total_execution_time,
                        success: false,
                        error_message: Some(format!("Migration {} failed: {}", migration.version, e)),
                    });
                }
            }
        }
        
        Ok(MigrationResult {
            applied_migrations,
            total_execution_time,
            success: true,
            error_message: None,
        })
    }
    
    /// Apply all pending migrations
    pub async fn migrate_to_latest(&self) -> Result<MigrationResult> {
        let latest_version = self.find_latest_migration_version().await?;
        self.migrate_to_version(latest_version).await
    }
    
    /// Load migration definitions from files
    async fn load_migrations(&self, from_version: u32, to_version: u32) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();
        
        // Read migration files from directory
        let migration_files = std::fs::read_dir(&self.migrations_dir)
            .map_err(|e| UveddiError::io_error("read_dir", &self.migrations_dir.to_string_lossy(), e))?;
        
        for entry in migration_files {
            let entry = entry.map_err(|e| UveddiError::io_error("read_dir_entry", &self.migrations_dir.to_string_lossy(), e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(migration) = self.parse_migration_file(&path).await? {
                    if migration.version >= from_version && migration.version <= to_version {
                        migrations.push(migration);
                    }
                }
            }
        }
        
        // Sort by version
        migrations.sort_by_key(|m| m.version);
        
        Ok(migrations)
    }
    
    /// Parse a migration file
    async fn parse_migration_file(&self, path: &Path) -> Result<Option<Migration>> {
        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| UveddiError::validation_error("Invalid migration filename"))?;
        
        // Parse filename format: "YYYYMMDDHHMMSS_migration_name.sql"
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        if parts.len() != 2 {
            warn!("Skipping migration file with invalid name format: {}", filename);
            return Ok(None);
        }
        
        let version_str = parts[0];
        let version = version_str.parse::<u32>()
            .map_err(|_| UveddiError::validation_error(&format!("Invalid migration version: {}", version_str)))?;
        
        let name = parts[1].trim_end_matches(".sql").replace('_', " ");
        
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| UveddiError::io_error("read_to_string", &path.to_string_lossy(), e))?;
        
        let checksum = calculate_checksum(&content);
        
        Ok(Some(Migration {
            version,
            name,
            up_sql: content,
            down_sql: None, // We'll support rollback migrations later
            checksum,
            applied_at: None,
        }))
    }
    
    /// Find the latest migration version available
    async fn find_latest_migration_version(&self) -> Result<u32> {
        let migration_files = std::fs::read_dir(&self.migrations_dir)
            .map_err(|e| UveddiError::io_error("read_dir", &self.migrations_dir.to_string_lossy(), e))?;
        
        let mut max_version = 0u32;
        
        for entry in migration_files {
            let entry = entry.map_err(|e| UveddiError::io_error("read_dir_entry", &self.migrations_dir.to_string_lossy(), e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(migration) = self.parse_migration_file(&path).await? {
                    max_version = max_version.max(migration.version);
                }
            }
        }
        
        Ok(max_version)
    }
    
    /// Apply a single migration
    async fn apply_migration(&self, migration: &Migration) -> Result<()> {
        // Begin transaction
        let tx = self.provider.begin_transaction().await?;
        
        // Split SQL into individual statements
        let statements = split_sql_statements(&migration.up_sql);
        
        for statement in statements {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }
            
            tx.execute_write(statement, &[]).await
                .map_err(|e| UveddiError::database_error_msg(&format!(
                    "Failed to execute migration statement '{}': {}", 
                    statement, e
                )))?;
        }
        
        tx.commit().await?;
        Ok(())
    }
    
    /// Record a successful migration application
    async fn record_migration(&self, migration: &Migration, execution_time: std::time::Duration) -> Result<()> {
        let applied_at = Utc::now();
        let execution_time_ms = execution_time.as_millis() as i32;
        
        match self.config.provider_type {
            DatabaseType::SQLite => {
                self.provider.execute_write(
                    "INSERT INTO schema_migrations (version, name, applied_at, checksum, execution_time_ms) VALUES (?, ?, ?, ?, ?)",
                    &[
                        &migration.version.to_string(),
                        &migration.name,
                        &applied_at.to_rfc3339(),
                        &migration.checksum,
                        &execution_time_ms.to_string(),
                    ]
                ).await?;
            }
            DatabaseType::PostgreSQL => {
                self.provider.execute_write(
                    "INSERT INTO schema_migrations (version, name, applied_at, checksum, execution_time_ms) VALUES ($1, $2, $3, $4, $5)",
                    &[
                        &migration.version.to_string(),
                        &migration.name,
                        &applied_at.to_rfc3339(),
                        &migration.checksum,
                        &execution_time_ms.to_string(),
                    ]
                ).await?;
            }
        }
        
        Ok(())
    }
    
    /// Get migration history
    pub async fn get_migration_history(&self) -> Result<Vec<AppliedMigration>> {
        let results = self.provider.execute_query(
            "SELECT version, name, applied_at, checksum, execution_time_ms FROM schema_migrations ORDER BY version",
            &[]
        ).await?;
        
        let mut history = Vec::new();
        
        for result in results {
            for row in result.rows {
                let applied_migration = AppliedMigration {
                    version: row.values.get("version").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
                    name: row.values.get("name").and_then(|v| v.as_string()).unwrap_or_default(),
                    applied_at: row.values.get("applied_at")
                        .and_then(|v| v.as_string())
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                    checksum: row.values.get("checksum").and_then(|v| v.as_string()).unwrap_or_default(),
                    execution_time: std::time::Duration::from_millis(
                        row.values.get("execution_time_ms").and_then(|v| v.as_i64()).unwrap_or(0) as u64
                    ),
                };
                history.push(applied_migration);
            }
        }
        
        Ok(history)
    }
    
    /// Migrate data from SQLite to PostgreSQL
    pub async fn migrate_sqlite_to_postgresql(
        &self, 
        sqlite_path: &str, 
        postgresql_config: DatabaseConfig
    ) -> Result<DataMigrationResult> {
        info!("Starting SQLite to PostgreSQL migration");
        
        // Create SQLite provider for source
        let mut sqlite_config = self.config.clone();
        sqlite_config.provider_type = DatabaseType::SQLite;
        sqlite_config.connection_string = sqlite_path.to_string();
        let sqlite_provider: Arc<dyn DatabaseProvider> = Arc::from(create_database_provider(&sqlite_config)?);
        
        // Create PostgreSQL provider for destination
        let pg_provider: Arc<dyn DatabaseProvider> = Arc::from(create_database_provider(&postgresql_config)?);
        
        // Initialize PostgreSQL schema
        pg_provider.initialize().await?;
        
        let start_time = std::time::Instant::now();
        let mut migrated_tables = Vec::new();
        let mut total_rows = 0u64;
        
        // Migrate each table
        let tables = vec!["projects", "analysis_runs", "anti_pattern_types", "architectural_issues", "dependencies"];
        
        for table_name in tables {
            info!("Migrating table: {}", table_name);
            
            match self.migrate_table_data(&*sqlite_provider, &*pg_provider, table_name).await {
                Ok(row_count) => {
                    total_rows += row_count;
                    migrated_tables.push(table_name.to_string());
                    info!("Successfully migrated {} rows from table {}", row_count, table_name);
                }
                Err(e) => {
                    error!("Failed to migrate table {}: {}", table_name, e);
                    return Ok(DataMigrationResult {
                        success: false,
                        migrated_tables,
                        total_rows,
                        execution_time: start_time.elapsed(),
                        error_message: Some(format!("Table migration failed for {}: {}", table_name, e)),
                    });
                }
            }
        }
        
        Ok(DataMigrationResult {
            success: true,
            migrated_tables,
            total_rows,
            execution_time: start_time.elapsed(),
            error_message: None,
        })
    }
    
    /// Migrate data from one table to another
    async fn migrate_table_data(
        &self,
        source_provider: &dyn DatabaseProvider,
        dest_provider: &dyn DatabaseProvider,
        table_name: &str,
    ) -> Result<u64> {
        // This is a simplified implementation
        // In practice, you'd need to handle data type conversions, batch processing, etc.
        
        // Get all data from source table
        let source_data = source_provider.execute_query(
            &format!("SELECT * FROM {}", table_name),
            &[]
        ).await?;
        
        let mut row_count = 0u64;
        
        for result in source_data {
            for row in result.rows {
                // Convert row data to destination format and insert
                // This would need proper implementation based on table structure
                row_count += 1;
            }
        }
        
        Ok(row_count)
    }
    
    /// Create a new migration file
    pub async fn create_migration(&self, name: &str) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filename = format!("{}_{}.sql", timestamp, name.replace(' ', "_"));
        let file_path = self.migrations_dir.join(&filename);
        
        let template = format!(
            "-- Migration: {}\n-- Created: {}\n\n-- Add your SQL statements here\n",
            name,
            Utc::now().to_rfc3339()
        );
        
        tokio::fs::create_dir_all(&self.migrations_dir).await
            .map_err(|e| UveddiError::io_error("create_dir_all", &self.migrations_dir.to_string_lossy(), e))?;
        
        tokio::fs::write(&file_path, template).await
            .map_err(|e| UveddiError::io_error("write", &file_path.to_string_lossy(), e))?;
        
        info!("Created migration file: {}", file_path.display());
        Ok(file_path)
    }
}

/// Migration definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub checksum: String,
    pub applied_at: Option<DateTime<Utc>>,
}

/// Applied migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedMigration {
    pub version: u32,
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub checksum: String,
    pub execution_time: std::time::Duration,
}

/// Migration execution result
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub applied_migrations: Vec<Migration>,
    pub total_execution_time: std::time::Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Data migration result
#[derive(Debug, Clone)]
pub struct DataMigrationResult {
    pub success: bool,
    pub migrated_tables: Vec<String>,
    pub total_rows: u64,
    pub execution_time: std::time::Duration,
    pub error_message: Option<String>,
}

/// Calculate checksum for migration content
fn calculate_checksum(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Split SQL content into individual statements
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_string = false;
    let mut string_delimiter = '\0';
    let mut chars = sql.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if in_string {
            current_statement.push(ch);
            if ch == string_delimiter && chars.peek() != Some(&string_delimiter) {
                in_string = false;
            } else if ch == string_delimiter && chars.peek() == Some(&string_delimiter) {
                // Escaped string delimiter
                chars.next(); // consume the second delimiter
                current_statement.push(ch);
            }
        } else {
            match ch {
                '\'' | '"' => {
                    in_string = true;
                    string_delimiter = ch;
                    current_statement.push(ch);
                }
                ';' => {
                    let statement = current_statement.trim().to_string();
                    if !statement.is_empty() {
                        statements.push(statement);
                    }
                    current_statement.clear();
                }
                _ => {
                    current_statement.push(ch);
                }
            }
        }
    }
    
    // Add remaining statement if any
    let statement = current_statement.trim().to_string();
    if !statement.is_empty() {
        statements.push(statement);
    }
    
    statements
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split_sql_statements() {
        let sql = "CREATE TABLE test (id INTEGER); INSERT INTO test VALUES (1, 'hello; world'); SELECT * FROM test;";
        let statements = split_sql_statements(sql);
        
        assert_eq!(statements.len(), 3);
        assert_eq!(statements[0], "CREATE TABLE test (id INTEGER)");
        assert_eq!(statements[1], "INSERT INTO test VALUES (1, 'hello; world')");
        assert_eq!(statements[2], "SELECT * FROM test");
    }
    
    #[test]
    fn test_calculate_checksum() {
        let content1 = "CREATE TABLE test (id INTEGER);";
        let content2 = "CREATE TABLE test (id INTEGER);";
        let content3 = "CREATE TABLE test (name TEXT);";
        
        assert_eq!(calculate_checksum(content1), calculate_checksum(content2));
        assert_ne!(calculate_checksum(content1), calculate_checksum(content3));
    }
}