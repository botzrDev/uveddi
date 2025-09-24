//! Database migration system
//!
//! This module provides a robust migration system for database schema evolution.
//! It supports both SQLite and PostgreSQL, with version tracking and rollback capability.

pub mod runner;
pub mod schema;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single database migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Migration version number (sequential)
    pub version: u32,
    /// Human-readable name for the migration
    pub name: String,
    /// SQL to apply the migration
    pub up_sql: String,
    /// SQL to rollback the migration
    pub down_sql: String,
    /// Optional dependencies on other migrations
    pub dependencies: Vec<u32>,
}

/// Migration execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub version: u32,
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub checksum: String,
}

/// Migration execution result
#[derive(Debug)]
pub enum MigrationResult {
    Applied { version: u32, name: String },
    Skipped { version: u32, reason: String },
    Failed { version: u32, error: String },
}

/// Migration registry containing all available migrations
#[derive(Debug, Default)]
pub struct MigrationRegistry {
    migrations: HashMap<u32, Migration>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }

    /// Register a migration
    pub fn register(&mut self, migration: Migration) {
        self.migrations.insert(migration.version, migration);
    }

    /// Get all migrations sorted by version
    pub fn get_all_sorted(&self) -> Vec<&Migration> {
        let mut migrations: Vec<_> = self.migrations.values().collect();
        migrations.sort_by_key(|m| m.version);
        migrations
    }

    /// Get migration by version
    pub fn get(&self, version: u32) -> Option<&Migration> {
        self.migrations.get(&version)
    }

    /// Get migrations after a specific version
    pub fn get_pending(&self, after_version: u32) -> Vec<&Migration> {
        let mut migrations: Vec<_> = self.migrations
            .values()
            .filter(|m| m.version > after_version)
            .collect();
        migrations.sort_by_key(|m| m.version);
        migrations
    }
}

/// Create the standard migration registry with all built-in migrations
pub fn create_standard_registry() -> MigrationRegistry {
    let mut registry = MigrationRegistry::new();

    // Migration 1: Create cache table
    registry.register(Migration {
        version: 1,
        name: "create_cache_table".to_string(),
        up_sql: include_str!("001_create_cache_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS cache;".to_string(),
        dependencies: vec![],
    });

    // Migration 2: Create metrics table
    registry.register(Migration {
        version: 2,
        name: "create_metrics_table".to_string(),
        up_sql: include_str!("002_create_metrics_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS metrics;".to_string(),
        dependencies: vec![],
    });

    // Migration 3: Create events table
    registry.register(Migration {
        version: 3,
        name: "create_events_table".to_string(),
        up_sql: include_str!("003_create_events_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS events;".to_string(),
        dependencies: vec![],
    });

    // Migration 4: Create issues table
    registry.register(Migration {
        version: 4,
        name: "create_issues_table".to_string(),
        up_sql: include_str!("004_create_issues_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS issues;".to_string(),
        dependencies: vec![],
    });

    // Migration 5: Create dependencies table
    registry.register(Migration {
        version: 5,
        name: "create_dependencies_table".to_string(),
        up_sql: include_str!("005_create_dependencies_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS dependencies;".to_string(),
        dependencies: vec![],
    });

    // Migration 6: Create security findings table
    registry.register(Migration {
        version: 6,
        name: "create_security_findings_table".to_string(),
        up_sql: include_str!("006_create_security_findings_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS security_findings;".to_string(),
        dependencies: vec![],
    });

    // Migration 7: Create technical debt table
    registry.register(Migration {
        version: 7,
        name: "create_technical_debt_table".to_string(),
        up_sql: include_str!("007_create_technical_debt_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS technical_debt;".to_string(),
        dependencies: vec![],
    });

    registry
}

impl Migration {
    /// Calculate checksum for migration content
    pub fn checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.version.hash(&mut hasher);
        self.name.hash(&mut hasher);
        self.up_sql.hash(&mut hasher);
        self.down_sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_registry() {
        let mut registry = MigrationRegistry::new();

        let migration = Migration {
            version: 1,
            name: "test_migration".to_string(),
            up_sql: "CREATE TABLE test ();".to_string(),
            down_sql: "DROP TABLE test;".to_string(),
            dependencies: vec![],
        };

        registry.register(migration);

        assert!(registry.get(1).is_some());
        assert!(registry.get(2).is_none());
        assert_eq!(registry.get_all_sorted().len(), 1);
    }

    #[test]
    fn test_migration_checksum() {
        let migration = Migration {
            version: 1,
            name: "test".to_string(),
            up_sql: "CREATE TABLE test ();".to_string(),
            down_sql: "DROP TABLE test;".to_string(),
            dependencies: vec![],
        };

        let checksum1 = migration.checksum();
        let checksum2 = migration.checksum();
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_standard_registry() {
        let registry = create_standard_registry();
        let migrations = registry.get_all_sorted();

        // Should have 7 migrations
        assert_eq!(migrations.len(), 7);

        // Check they are in order
        for (i, migration) in migrations.iter().enumerate() {
            assert_eq!(migration.version, (i + 1) as u32);
        }
    }
}