//! Integration tests for the migrate CLI command
//!
//! Tests the full lifecycle of database migrations including plan, up, down, and status commands.
//! Uses temporary databases to avoid polluting the repository.

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use uveddi::cli::commands::migrate::{MigrateCommand, MigrateSubcommand};
use uveddi::database::connection::{config::DatabaseConfig, ConnectionPool};
use uveddi::database::migrations::{create_standard_registry, runner::MigrationRunner};

/// Test harness that provides a temporary database for migration testing
struct MigrationTestHarness {
    _temp_dir: TempDir, // Kept alive to ensure cleanup on drop
    db_path: PathBuf,
}

impl MigrationTestHarness {
    /// Create a new test harness with a temporary database
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_migration.db");

        Ok(Self {
            _temp_dir: temp_dir,
            db_path,
        })
    }

    /// Get the database path as a string
    fn db_path_string(&self) -> String {
        self.db_path.to_string_lossy().to_string()
    }

    /// Create a migration runner for testing
    async fn create_runner(&self) -> Result<MigrationRunner, Box<dyn std::error::Error>> {
        let config = DatabaseConfig::sqlite(&self.db_path_string());
        let provider = Arc::new(uveddi::database::SqliteProvider::new(config.clone())?);
        let pool = ConnectionPool::new(config.into(), provider).await?;
        let registry = create_standard_registry();
        let runner = MigrationRunner::new(pool, registry);

        // Initialize migration tracking table
        runner.initialize().await?;

        Ok(runner)
    }

    /// Verify the database file exists
    async fn db_exists(&self) -> bool {
        fs::metadata(&self.db_path).await.is_ok()
    }

    /// Get the size of the database file in bytes
    async fn db_size(&self) -> Result<u64, std::io::Error> {
        let metadata = fs::metadata(&self.db_path).await?;
        Ok(metadata.len())
    }
}

// Cleanup happens automatically when TempDir is dropped

#[tokio::test]
async fn test_registry_has_all_migrations() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry and check all migrations are present
    let registry = create_standard_registry();
    let all_migrations = registry.get_all_sorted();

    println!("Registry contains {} migrations:", all_migrations.len());
    for migration in &all_migrations {
        println!("  v{}: {}", migration.version, migration.name);
    }

    assert_eq!(
        all_migrations.len(),
        7,
        "Registry should contain 7 migrations"
    );

    // Verify each migration is present
    for i in 1..=7 {
        assert!(registry.get(i).is_some(), "Migration {} should exist", i);
    }

    Ok(())
}

#[tokio::test]
async fn test_migrate_plan_shows_pending_migrations() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Get the migration plan
    let plan = runner.plan_migrations().await?;

    // Verify that we have pending migrations from 1 to 7 (sequential versions)
    assert_eq!(plan.current_version, 0, "Initial version should be 0");
    assert!(
        plan.target_version >= 7,
        "Target version should include all migrations"
    );
    assert_eq!(
        plan.planned_migrations.len(),
        7,
        "Should have 7 pending migrations (versions 1-7)"
    );

    // Verify migrations are in order
    let expected_versions = vec![1, 2, 3, 4, 5, 6, 7];
    for (i, migration) in plan.planned_migrations.iter().enumerate() {
        assert_eq!(
            migration.version, expected_versions[i],
            "Migration {} should have version {}",
            i, expected_versions[i]
        );
    }

    // Verify plan display includes key information
    let display = plan.display();
    assert!(display.contains("Current Version: 0"));
    assert!(display.contains("Pending Migrations"));
    assert!(display.contains("create_cache_table"));
    assert!(display.contains("create_metrics_table"));
    assert!(display.contains("create_events_table"));
    assert!(display.contains("create_issues_table"));
    assert!(display.contains("create_dependencies_table"));
    assert!(display.contains("create_security_findings_table"));
    assert!(display.contains("create_technical_debt_table"));

    Ok(())
}

#[tokio::test]
async fn test_migrate_up_applies_all_pending() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Verify database exists before migrations
    assert!(
        harness.db_exists().await,
        "Database should exist after initialization"
    );

    // Run migrations
    let results = runner.run_pending_migrations().await?;

    // Verify all migrations were applied
    assert_eq!(results.len(), 7, "Should have applied 7 migrations");

    // Check that all results are successful applications
    for result in &results {
        match result {
            uveddi::database::migrations::MigrationResult::Applied { version, name } => {
                println!("✓ Applied migration v{}: {}", version, name);
            }
            uveddi::database::migrations::MigrationResult::Failed { version, error } => {
                panic!("Migration {} failed: {}", version, error);
            }
            uveddi::database::migrations::MigrationResult::Skipped { version, reason } => {
                panic!("Migration {} was unexpectedly skipped: {}", version, reason);
            }
        }
    }

    // Verify current version is updated
    let current_version = runner.current_version().await?;
    assert_eq!(
        current_version, 7,
        "Current version should be 7 after all migrations"
    );

    // Verify no pending migrations remain
    let plan = runner.plan_migrations().await?;
    assert_eq!(
        plan.planned_migrations.len(),
        0,
        "No pending migrations should remain"
    );

    // Verify database file has grown
    let size = harness.db_size().await?;
    assert!(size > 0, "Database should have content after migrations");

    Ok(())
}

#[tokio::test]
async fn test_migrate_status_shows_applied_migrations() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Initially, no migrations should be applied
    let initial_version = runner.current_version().await?;
    assert_eq!(initial_version, 0, "Initial version should be 0");

    let initial_applied = runner.applied_migrations().await?;
    assert_eq!(
        initial_applied.len(),
        0,
        "No migrations should be applied initially"
    );

    // Apply migrations
    runner.run_pending_migrations().await?;

    // Check status after migrations
    let current_version = runner.current_version().await?;
    assert_eq!(current_version, 7, "Current version should be 7");

    let applied = runner.applied_migrations().await?;
    assert_eq!(applied.len(), 7, "7 migrations should be applied");

    // Verify all expected migrations are in the applied list
    let expected_versions = vec![1, 2, 3, 4, 5, 6, 7];
    for (i, record) in applied.iter().enumerate() {
        assert_eq!(record.version, expected_versions[i]);
        assert!(
            !record.name.is_empty(),
            "Migration name should not be empty"
        );
        assert!(
            !record.checksum.is_empty(),
            "Migration checksum should not be empty"
        );
        println!(
            "✓ Migration v{}: {} applied at {}",
            record.version, record.name, record.applied_at
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_migrate_down_rolls_back_to_version() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Apply all migrations first
    runner.run_pending_migrations().await?;

    let version_before = runner.current_version().await?;
    assert_eq!(version_before, 7, "Should be at version 7 before rollback");

    // Rollback to version 3
    let target_version = 3;
    let rollback_results = runner.rollback_to_version(target_version).await?;

    // Verify rollback occurred
    assert!(
        rollback_results.len() > 0,
        "Some migrations should have been rolled back"
    );

    // Check current version after rollback
    let current_version = runner.current_version().await?;
    assert_eq!(
        current_version, target_version,
        "Should be at version {} after rollback",
        target_version
    );

    // Verify applied migrations list is correct
    let applied = runner.applied_migrations().await?;
    assert_eq!(applied.len(), 3, "Only 3 migrations should remain applied");

    // Verify that plan shows the rolled-back migrations as pending
    let plan = runner.plan_migrations().await?;
    assert_eq!(plan.current_version, target_version);
    assert_eq!(
        plan.planned_migrations.len(),
        4,
        "4 migrations should be pending after rollback to version 3"
    );

    // Verify the pending migrations are the ones we rolled back
    let pending_versions: Vec<u32> = plan.planned_migrations.iter().map(|m| m.version).collect();
    assert_eq!(pending_versions, vec![4, 5, 6, 7]);

    Ok(())
}

#[tokio::test]
async fn test_migrate_plan_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Execute a plan (dry-run) operation
    let plan = runner.plan_migrations().await?;

    // Verify plan contains expected information
    assert_eq!(plan.current_version, 0);
    assert!(plan.target_version >= 7);
    assert_eq!(plan.planned_migrations.len(), 7);

    // Verify display output is well-formed
    let display = plan.display();
    assert!(display.contains("Migration Plan"));
    assert!(display.contains("Pending Migrations"));

    Ok(())
}

#[tokio::test]
async fn test_migrate_up_with_verification() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Execute migrations
    let results = runner.run_pending_migrations().await?;
    assert_eq!(results.len(), 7, "Should apply all 7 migrations");

    // Verify all migrations succeeded
    for result in &results {
        assert!(matches!(
            result,
            uveddi::database::migrations::MigrationResult::Applied { .. }
        ));
    }

    // Verify database state
    let current_version = runner.current_version().await?;
    assert_eq!(current_version, 7, "All migrations should be applied");

    // Verify database file size
    let size = harness.db_size().await?;
    assert!(
        size > 8192,
        "Database should have substantial content after migrations"
    );

    Ok(())
}

#[tokio::test]
async fn test_migrate_status_reporting() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Apply some migrations first
    runner.run_pending_migrations().await?;

    // Get and verify migration status
    let current_version = runner.current_version().await?;
    assert_eq!(current_version, 7);

    let applied = runner.applied_migrations().await?;
    assert_eq!(applied.len(), 7);

    // Verify each migration record has required fields
    for record in &applied {
        assert!(record.version > 0, "Version should be set");
        assert!(!record.name.is_empty(), "Name should be set");
        assert!(!record.checksum.is_empty(), "Checksum should be set");
    }

    Ok(())
}

#[tokio::test]
async fn test_migrate_down_with_verification() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Apply all migrations first
    runner.run_pending_migrations().await?;

    // Execute rollback
    let rollback_results = runner.rollback_to_version(3).await?;
    assert!(
        rollback_results.len() > 0,
        "Should have rolled back some migrations"
    );

    // Verify rollback occurred
    let current_version = runner.current_version().await?;
    assert_eq!(current_version, 3, "Should be rolled back to version 3");

    // Verify only 3 migrations remain applied
    let applied = runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        3,
        "Only 3 migrations should remain after rollback"
    );

    Ok(())
}

#[tokio::test]
async fn test_migrate_idempotency() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Apply migrations twice
    let results1 = runner.run_pending_migrations().await?;
    let results2 = runner.run_pending_migrations().await?;

    // First run should apply migrations
    assert_eq!(results1.len(), 7, "First run should apply 7 migrations");

    // Second run should find nothing to apply
    assert_eq!(
        results2.len(),
        0,
        "Second run should apply 0 migrations (idempotent)"
    );

    Ok(())
}

// ============================================================================
// CLI Command Integration Tests
// ============================================================================
// These tests exercise the MigrateCommand::execute() paths directly

#[tokio::test]
async fn test_cli_migrate_plan_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;

    // Create command with Plan subcommand
    let command = MigrateCommand {
        subcommand: MigrateSubcommand::Plan {
            database: harness.db_path_string(),
        },
    };

    // Execute the command
    command.execute().await?;

    // Verify database was initialized but no migrations were applied
    let runner = harness.create_runner().await?;
    let version = runner.current_version().await?;
    assert_eq!(version, 0, "Plan command should not apply migrations");

    Ok(())
}

#[tokio::test]
async fn test_cli_migrate_up_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;

    // Create command with Up subcommand
    let command = MigrateCommand {
        subcommand: MigrateSubcommand::Up {
            database: harness.db_path_string(),
        },
    };

    // Execute the command
    command.execute().await?;

    // Verify all migrations were applied
    let runner = harness.create_runner().await?;
    let version = runner.current_version().await?;
    assert_eq!(version, 7, "Up command should apply all 7 migrations");

    let applied = runner.applied_migrations().await?;
    assert_eq!(applied.len(), 7, "7 migrations should be applied");

    Ok(())
}

#[tokio::test]
async fn test_cli_migrate_status_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;

    // Apply migrations first
    let runner = harness.create_runner().await?;
    runner.run_pending_migrations().await?;

    // Create command with Status subcommand
    let command = MigrateCommand {
        subcommand: MigrateSubcommand::Status {
            database: harness.db_path_string(),
        },
    };

    // Execute the command - should not error
    command.execute().await?;

    Ok(())
}

#[tokio::test]
async fn test_cli_migrate_down_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;

    // Apply all migrations first
    let runner = harness.create_runner().await?;
    runner.run_pending_migrations().await?;

    // Verify we're at version 7
    let version_before = runner.current_version().await?;
    assert_eq!(version_before, 7);

    // Create command with Down subcommand to rollback to version 3
    let command = MigrateCommand {
        subcommand: MigrateSubcommand::Down {
            version: 3,
            database: harness.db_path_string(),
        },
    };

    // Execute the command
    command.execute().await?;

    // Verify rollback occurred
    let version_after = runner.current_version().await?;
    assert_eq!(
        version_after, 3,
        "Down command should rollback to version 3"
    );

    let applied = runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        3,
        "Only 3 migrations should remain after rollback"
    );

    Ok(())
}

#[tokio::test]
async fn test_cli_migrate_up_idempotency() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;

    // Create command with Up subcommand
    let command = MigrateCommand {
        subcommand: MigrateSubcommand::Up {
            database: harness.db_path_string(),
        },
    };

    // Execute twice
    command.execute().await?;
    command.execute().await?;

    // Verify still at version 7 (second run should be no-op)
    let runner = harness.create_runner().await?;
    let version = runner.current_version().await?;
    assert_eq!(version, 7, "Multiple up commands should be idempotent");

    Ok(())
}

#[tokio::test]
async fn test_cli_migrate_down_already_at_target() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;

    // Apply all migrations then rollback to version 3
    let runner = harness.create_runner().await?;
    runner.run_pending_migrations().await?;
    runner.rollback_to_version(3).await?;

    // Try to rollback to version 5 (which is above current version 3)
    let command = MigrateCommand {
        subcommand: MigrateSubcommand::Down {
            version: 5,
            database: harness.db_path_string(),
        },
    };

    // Should succeed without error (no-op)
    command.execute().await?;

    // Verify we're still at version 3
    let version = runner.current_version().await?;
    assert_eq!(
        version, 3,
        "Down command should be no-op when target is higher"
    );

    Ok(())
}
