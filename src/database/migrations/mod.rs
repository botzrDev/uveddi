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

/// Planned migration for dry-run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedMigration {
    pub version: u32,
    pub name: String,
    pub dependencies: Vec<u32>,
    pub checksum: String,
    pub up_sql_preview: String,
}

/// Migration plan showing current state and pending migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub current_version: u32,
    pub target_version: u32,
    pub applied_migrations: Vec<MigrationRecord>,
    pub planned_migrations: Vec<PlannedMigration>,
}

impl MigrationPlan {
    /// Pretty-print the migration plan
    pub fn display(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Migration Plan (Dry Run) ===\n\n");
        output.push_str(&format!("Current Version: {}\n", self.current_version));
        output.push_str(&format!("Target Version:  {}\n\n", self.target_version));

        if self.applied_migrations.is_empty() {
            output.push_str("No migrations applied yet.\n\n");
        } else {
            output.push_str(&format!(
                "Applied Migrations ({}):\n",
                self.applied_migrations.len()
            ));
            for migration in &self.applied_migrations {
                output.push_str(&format!(
                    "  ✓ v{}: {} (applied {})\n",
                    migration.version,
                    migration.name,
                    migration.applied_at.format("%Y-%m-%d %H:%M:%S")
                ));
            }
            output.push_str("\n");
        }

        if self.planned_migrations.is_empty() {
            output.push_str("✓ Database is up to date - no pending migrations.\n");
        } else {
            output.push_str(&format!(
                "Pending Migrations ({}):\n",
                self.planned_migrations.len()
            ));
            for migration in &self.planned_migrations {
                output.push_str(&format!("  → v{}: {}\n", migration.version, migration.name));
                if !migration.dependencies.is_empty() {
                    output.push_str(&format!(
                        "      Dependencies: {:?}\n",
                        migration.dependencies
                    ));
                }
                output.push_str(&format!("      Checksum: {}\n", migration.checksum));
                if !migration.up_sql_preview.is_empty() {
                    output.push_str("      SQL Preview:\n");
                    for line in migration.up_sql_preview.lines() {
                        output.push_str(&format!("        {}\n", line));
                    }
                }
                output.push_str("\n");
            }
        }

        output
    }
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
        let mut migrations: Vec<_> = self
            .migrations
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

    // Migration 1: Create cache table (2025-10-01)
    registry.register(Migration {
        version: 1,
        name: "20251001_create_cache_table".to_string(),
        up_sql: include_str!("20251001_create_cache_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS cache;".to_string(),
        dependencies: vec![],
    });

    // Migration 2: Create metrics table (2025-10-02)
    registry.register(Migration {
        version: 2,
        name: "20251002_create_metrics_table".to_string(),
        up_sql: include_str!("20251002_create_metrics_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS metrics;".to_string(),
        dependencies: vec![],
    });

    // Migration 3: Create events table (2025-10-03)
    registry.register(Migration {
        version: 3,
        name: "20251003_create_events_table".to_string(),
        up_sql: include_str!("20251003_create_events_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS events;".to_string(),
        dependencies: vec![],
    });

    // Migration 4: Create issues table (2025-10-04)
    registry.register(Migration {
        version: 4,
        name: "20251004_create_issues_table".to_string(),
        up_sql: include_str!("20251004_create_issues_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS issues;".to_string(),
        dependencies: vec![],
    });

    // Migration 5: Create dependencies table (2025-10-05)
    registry.register(Migration {
        version: 5,
        name: "20251005_create_dependencies_table".to_string(),
        up_sql: include_str!("20251005_create_dependencies_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS dependencies;".to_string(),
        dependencies: vec![],
    });

    // Migration 6: Create security findings table (2025-10-06)
    registry.register(Migration {
        version: 6,
        name: "20251006_create_security_findings_table".to_string(),
        up_sql: include_str!("20251006_create_security_findings_table.sql").to_string(),
        down_sql: "DROP TABLE IF EXISTS security_findings;".to_string(),
        dependencies: vec![],
    });

    // Migration 7: Create technical debt table (2025-10-07)
    registry.register(Migration {
        version: 7,
        name: "20251007_create_technical_debt_table".to_string(),
        up_sql: include_str!("20251007_create_technical_debt_table.sql").to_string(),
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
