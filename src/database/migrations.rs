//! Database Migration System
//!
//! This module provides utilities for managing database schema changes and migrations
//! in a controlled and reversible manner.

use crate::error::Result;
use rusqlite::Connection;
use std::collections::HashMap;
use tracing::{info, warn};

/// Database migration definition
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

/// Migration manager for handling schema changes
pub struct MigrationManager {
    migrations: HashMap<u32, Migration>,
}

impl MigrationManager {
    /// Create a new migration manager with built-in migrations
    pub fn new() -> Self {
        let mut manager = Self {
            migrations: HashMap::new(),
        };
        
        manager.add_builtin_migrations();
        manager
    }

    /// Add a migration
    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.insert(migration.version, migration);
    }

    /// Get current database version
    pub fn get_current_version(&self, conn: &Connection) -> Result<u32> {
        // Create migrations table if it doesn't exist
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );
        ")?;

        let mut stmt = conn.prepare(
            "SELECT MAX(version) FROM schema_migrations"
        )?;

        let version = stmt.query_row([], |row| {
            Ok(row.get::<_, Option<u32>>(0)?.unwrap_or(0))
        })?;

        Ok(version)
    }

    /// Apply all pending migrations
    pub fn migrate_up(&self, conn: &mut Connection) -> Result<Vec<u32>> {
        let current_version = self.get_current_version(conn)?;
        let mut applied_versions = Vec::new();

        // Get all versions greater than current, sorted
        let mut pending_versions: Vec<u32> = self.migrations
            .keys()
            .filter(|&&v| v > current_version)
            .copied()
            .collect();
        pending_versions.sort();

        for version in pending_versions {
            if let Some(migration) = self.migrations.get(&version) {
                info!("Applying migration {}: {}", version, migration.name);
                
                // Check if migration contains PRAGMA statements that need to run outside transaction
                if migration.up_sql.contains("PRAGMA") {
                    // Execute PRAGMAs outside transaction
                    conn.execute_batch(&migration.up_sql)?;
                    
                    // Record the migration in a separate transaction
                    let tx = conn.transaction()?;
                    tx.execute(
                        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?, ?, ?)",
                        rusqlite::params![
                            version,
                            migration.name,
                            chrono::Utc::now().to_rfc3339(),
                        ],
                    )?;
                    tx.commit()?;
                } else {
                    // Normal migration in transaction
                    let tx = conn.transaction()?;
                    
                    // Apply the migration
                    tx.execute_batch(&migration.up_sql)?;
                    
                    // Record the migration
                    tx.execute(
                        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?, ?, ?)",
                        rusqlite::params![
                            version,
                            migration.name,
                            chrono::Utc::now().to_rfc3339(),
                        ],
                    )?;
                    
                    tx.commit()?;
                }
                applied_versions.push(version);
                info!("Successfully applied migration {}", version);
            }
        }

        Ok(applied_versions)
    }

    /// Rollback to a specific version
    pub fn migrate_down(&self, conn: &mut Connection, target_version: u32) -> Result<Vec<u32>> {
        let current_version = self.get_current_version(conn)?;
        let mut rolled_back = Vec::new();

        if target_version >= current_version {
            return Ok(rolled_back);
        }

        // Get all versions greater than target, sorted in reverse
        let mut versions_to_rollback: Vec<u32> = self.migrations
            .keys()
            .filter(|&&v| v > target_version && v <= current_version)
            .copied()
            .collect();
        versions_to_rollback.sort_by(|a, b| b.cmp(a)); // Reverse order

        for version in versions_to_rollback {
            if let Some(migration) = self.migrations.get(&version) {
                warn!("Rolling back migration {}: {}", version, migration.name);
                
                // Check if rollback contains PRAGMA statements that need to run outside transaction
                if migration.down_sql.contains("PRAGMA") {
                    // Execute PRAGMAs outside transaction
                    conn.execute_batch(&migration.down_sql)?;
                    
                    // Remove migration record in a separate transaction
                    let tx = conn.transaction()?;
                    tx.execute("DELETE FROM schema_migrations WHERE version = ?", [version])?;
                    tx.commit()?;
                } else {
                    // Normal rollback in transaction
                    let tx = conn.transaction()?;
                    
                    // Apply the rollback
                    tx.execute_batch(&migration.down_sql)?;
                    
                    // Remove the migration record
                    tx.execute("DELETE FROM schema_migrations WHERE version = ?", [version])?;
                    tx.commit()?;
                }
                rolled_back.push(version);
                warn!("Successfully rolled back migration {}", version);
            }
        }

        Ok(rolled_back)
    }

    /// Get migration status
    pub fn get_migration_status(&self, conn: &Connection) -> Result<MigrationStatus> {
        let current_version = self.get_current_version(conn)?;
        let max_available = self.migrations.keys().max().copied().unwrap_or(0);
        
        let mut applied = Vec::new();
        let mut pending = Vec::new();

        // Get applied migrations
        let mut stmt = conn.prepare(
            "SELECT version, name, applied_at FROM schema_migrations ORDER BY version"
        )?;
        
        let applied_iter = stmt.query_map([], |row| {
            Ok(AppliedMigration {
                version: row.get(0)?,
                name: row.get(1)?,
                applied_at: row.get(2)?,
            })
        })?;

        for migration in applied_iter {
            applied.push(migration?);
        }

        // Get pending migrations
        for (&version, migration) in &self.migrations {
            if version > current_version {
                pending.push(PendingMigration {
                    version,
                    name: migration.name.clone(),
                });
            }
        }

        pending.sort_by_key(|m| m.version);

        Ok(MigrationStatus {
            current_version,
            max_available_version: max_available,
            applied,
            pending,
        })
    }

    /// Add built-in migrations for Uveddi database optimizations
    fn add_builtin_migrations(&mut self) {
        // Migration 1: Add performance indexes (v0.9.1)
        self.add_migration(Migration {
            version: 1,
            name: "Add performance indexes".to_string(),
            up_sql: "
                CREATE INDEX IF NOT EXISTS idx_analysis_runs_project_time ON analysis_runs(project_id, start_time);
                CREATE INDEX IF NOT EXISTS idx_analysis_runs_status ON analysis_runs(status);
                CREATE INDEX IF NOT EXISTS idx_architectural_issues_run_id ON architectural_issues(analysis_run_id);
                CREATE INDEX IF NOT EXISTS idx_architectural_issues_file_path ON architectural_issues(file_path);
                CREATE INDEX IF NOT EXISTS idx_architectural_issues_severity ON architectural_issues(severity);
                CREATE INDEX IF NOT EXISTS idx_architectural_issues_detector ON architectural_issues(detector_name);
                CREATE INDEX IF NOT EXISTS idx_architectural_issues_type_id ON architectural_issues(anti_pattern_type_id);
                CREATE INDEX IF NOT EXISTS idx_architectural_issues_composite ON architectural_issues(analysis_run_id, severity, detector_name);
                CREATE INDEX IF NOT EXISTS idx_anti_pattern_types_name ON anti_pattern_types(name);
                CREATE INDEX IF NOT EXISTS idx_anti_pattern_types_category ON anti_pattern_types(category);
                CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
            ".to_string(),
            down_sql: "
                DROP INDEX IF EXISTS idx_analysis_runs_project_time;
                DROP INDEX IF EXISTS idx_analysis_runs_status;
                DROP INDEX IF EXISTS idx_architectural_issues_run_id;
                DROP INDEX IF EXISTS idx_architectural_issues_file_path;
                DROP INDEX IF EXISTS idx_architectural_issues_severity;
                DROP INDEX IF EXISTS idx_architectural_issues_detector;
                DROP INDEX IF EXISTS idx_architectural_issues_type_id;
                DROP INDEX IF EXISTS idx_architectural_issues_composite;
                DROP INDEX IF EXISTS idx_anti_pattern_types_name;
                DROP INDEX IF EXISTS idx_anti_pattern_types_category;
                DROP INDEX IF EXISTS idx_projects_path;
            ".to_string(),
        });

        // Migration 2: Add dependencies table (v0.9.2)
        self.add_migration(Migration {
            version: 2,
            name: "Add dependencies table".to_string(),
            up_sql: "
                CREATE TABLE IF NOT EXISTS dependencies (
                    dependency_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    analysis_run_id INTEGER NOT NULL,
                    from_file TEXT NOT NULL,
                    to_module TEXT NOT NULL,
                    dependency_type TEXT NOT NULL,
                    line_number INTEGER,
                    FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(run_id)
                );
                CREATE INDEX IF NOT EXISTS idx_dependencies_run_id ON dependencies(analysis_run_id);
                CREATE INDEX IF NOT EXISTS idx_dependencies_from_file ON dependencies(from_file);
                CREATE INDEX IF NOT EXISTS idx_dependencies_to_module ON dependencies(to_module);
            ".to_string(),
            down_sql: "
                DROP INDEX IF EXISTS idx_dependencies_run_id;
                DROP INDEX IF EXISTS idx_dependencies_from_file;
                DROP INDEX IF EXISTS idx_dependencies_to_module;
                DROP TABLE IF EXISTS dependencies;
            ".to_string(),
        });

        // Migration 3: Add performance optimization pragmas (v0.9.3)
        self.add_migration(Migration {
            version: 3,
            name: "Enable performance pragmas".to_string(),
            up_sql: "
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA cache_size = 10000;
                PRAGMA temp_store = MEMORY;
                PRAGMA mmap_size = 268435456;
                PRAGMA foreign_keys = ON;
            ".to_string(),
            down_sql: "
                PRAGMA journal_mode = DELETE;
                PRAGMA synchronous = FULL;
                PRAGMA cache_size = 2000;
                PRAGMA temp_store = DEFAULT;
                PRAGMA mmap_size = 0;
                PRAGMA foreign_keys = OFF;
            ".to_string(),
        });

        // Migration 4: Add analysis statistics views (v0.9.4)
        self.add_migration(Migration {
            version: 4,
            name: "Add analysis statistics views".to_string(),
            up_sql: "
                CREATE VIEW IF NOT EXISTS issue_summary_by_run AS
                SELECT 
                    analysis_run_id,
                    COUNT(*) as total_issues,
                    COUNT(CASE WHEN severity = 'critical' THEN 1 END) as critical_count,
                    COUNT(CASE WHEN severity = 'high' THEN 1 END) as high_count,
                    COUNT(CASE WHEN severity = 'medium' THEN 1 END) as medium_count,
                    COUNT(CASE WHEN severity = 'low' THEN 1 END) as low_count,
                    COUNT(DISTINCT file_path) as affected_files
                FROM architectural_issues 
                GROUP BY analysis_run_id;

                CREATE VIEW IF NOT EXISTS detector_performance AS
                SELECT 
                    detector_name,
                    COUNT(*) as issues_found,
                    COUNT(DISTINCT analysis_run_id) as runs_used,
                    AVG(CASE 
                        WHEN severity = 'critical' THEN 4
                        WHEN severity = 'high' THEN 3
                        WHEN severity = 'medium' THEN 2
                        WHEN severity = 'low' THEN 1
                        ELSE 0
                    END) as avg_severity_score
                FROM architectural_issues 
                GROUP BY detector_name;
            ".to_string(),
            down_sql: "
                DROP VIEW IF EXISTS detector_performance;
                DROP VIEW IF EXISTS issue_summary_by_run;
            ".to_string(),
        });
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of applied migration
#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub version: u32,
    pub name: String,
    pub applied_at: String,
}

/// Status of pending migration
#[derive(Debug, Clone)]
pub struct PendingMigration {
    pub version: u32,
    pub name: String,
}

/// Overall migration status
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub current_version: u32,
    pub max_available_version: u32,
    pub applied: Vec<AppliedMigration>,
    pub pending: Vec<PendingMigration>,
}

impl MigrationStatus {
    /// Check if database is up to date
    pub fn is_up_to_date(&self) -> bool {
        self.current_version == self.max_available_version
    }

    /// Get number of pending migrations
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn initialize_test_schema(conn: &mut Connection) {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS analysis_runs (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                status TEXT NOT NULL,
                total_files_analyzed INTEGER,
                total_issues_found INTEGER,
                analysis_config TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS anti_pattern_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                category TEXT NOT NULL,
                description TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS architectural_issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                analysis_run_id INTEGER NOT NULL,
                anti_pattern_type_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                line_number INTEGER,
                column_number INTEGER,
                severity TEXT NOT NULL,
                detector_name TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs (run_id),
                FOREIGN KEY (anti_pattern_type_id) REFERENCES anti_pattern_types (id)
            );
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT UNIQUE NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
        ").unwrap();
    }

    #[test]
    fn test_migration_manager_creation() {
        let manager = MigrationManager::new();
        assert!(manager.migrations.len() > 0);
    }

    #[test]
    fn test_migration_up() {
        let mut conn = Connection::open_in_memory().unwrap();
        
        // Initialize base schema first 
        initialize_test_schema(&mut conn);
        
        let manager = MigrationManager::new();
        
        // Apply all migrations
        let applied = manager.migrate_up(&mut conn).unwrap();
        assert!(applied.len() > 0);
        
        // Check current version
        let version = manager.get_current_version(&mut conn).unwrap();
        let max_version = manager.migrations.keys().max().copied().unwrap();
        assert_eq!(version, max_version);
    }

    #[test]
    fn test_migration_down() {
        let mut conn = Connection::open_in_memory().unwrap();
        
        // Initialize base schema first 
        initialize_test_schema(&mut conn);
        
        let manager = MigrationManager::new();
        
        // Apply all migrations
        let _applied = manager.migrate_up(&mut conn).unwrap();
        let initial_version = manager.get_current_version(&mut conn).unwrap();
        
        // Roll back to version 2
        let rolled_back = manager.migrate_down(&mut conn, 2).unwrap();
        assert!(rolled_back.len() > 0);
        
        let current_version = manager.get_current_version(&mut conn).unwrap();
        assert_eq!(current_version, 2);
        assert!(current_version < initial_version);
    }

    #[test]
    fn test_migration_status() {
        let mut conn = Connection::open_in_memory().unwrap();
        
        // Initialize base schema first 
        initialize_test_schema(&mut conn);
        
        let manager = MigrationManager::new();
        
        // Check initial status
        let status = manager.get_migration_status(&mut conn).unwrap();
        assert_eq!(status.current_version, 0);
        assert!(status.pending.len() > 0);
        assert!(!status.is_up_to_date());
        
        // Apply migrations and check again
        let _applied = manager.migrate_up(&mut conn).unwrap();
        let status = manager.get_migration_status(&mut conn).unwrap();
        assert!(status.is_up_to_date());
        assert_eq!(status.pending_count(), 0);
    }
}