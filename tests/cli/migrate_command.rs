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

/// Expected migration metadata in version order for the standard registry.
const EXPECTED_MIGRATIONS: &[(u32, &str)] = &[
    (1, "20251001_create_cache_table"),
    (2, "20251002_create_metrics_table"),
    (3, "20251003_create_events_table"),
    (4, "20251004_create_issues_table"),
    (5, "20251005_create_dependencies_table"),
    (6, "20251006_create_security_findings_table"),
    (7, "20251007_create_technical_debt_table"),
];

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

    assert_eq!(
        all_migrations.len(),
        EXPECTED_MIGRATIONS.len(),
        "Registry should contain {} migrations",
        EXPECTED_MIGRATIONS.len()
    );

    for ((expected_version, expected_name), migration) in
        EXPECTED_MIGRATIONS.iter().zip(all_migrations.iter())
    {
        assert_eq!(
            migration.version, *expected_version,
            "Migration should have sequential version {}",
            expected_version
        );
        assert_eq!(
            migration.name, *expected_name,
            "Migration {} should be named {}",
            expected_version, expected_name
        );
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
    assert_eq!(
        plan.target_version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Target version should be the final sequential migration"
    );
    assert_eq!(
        plan.planned_migrations.len(),
        EXPECTED_MIGRATIONS.len(),
        "Should have {} pending migrations (versions 1-7)",
        EXPECTED_MIGRATIONS.len()
    );

    // Verify migrations are in order with matching names
    for ((expected_version, expected_name), migration) in EXPECTED_MIGRATIONS
        .iter()
        .zip(plan.planned_migrations.iter())
    {
        assert_eq!(
            migration.version, *expected_version,
            "Migration should have sequential version {}",
            expected_version
        );
        assert_eq!(
            migration.name, *expected_name,
            "Migration {} should be named {}",
            expected_version, expected_name
        );
    }

    // Verify plan display includes key information
    let display = plan.display();
    assert!(display.contains("Current Version: 0"));
    assert!(display.contains("Pending Migrations"));
    assert!(display.contains("20251001_create_cache_table"));
    assert!(display.contains("20251002_create_metrics_table"));
    assert!(display.contains("20251003_create_events_table"));
    assert!(display.contains("20251004_create_issues_table"));
    assert!(display.contains("20251005_create_dependencies_table"));
    assert!(display.contains("20251006_create_security_findings_table"));
    assert!(display.contains("20251007_create_technical_debt_table"));

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
    assert_eq!(
        results.len(),
        EXPECTED_MIGRATIONS.len(),
        "Should have applied {} migrations",
        EXPECTED_MIGRATIONS.len()
    );

    // Check that all results are successful applications
    for ((expected_version, expected_name), result) in
        EXPECTED_MIGRATIONS.iter().zip(results.iter())
    {
        match result {
            uveddi::database::migrations::MigrationResult::Applied { version, name } => {
                assert_eq!(
                    *version, *expected_version,
                    "Applied migration version should be sequential"
                );
                assert_eq!(
                    name, expected_name,
                    "Applied migration {} should be named {}",
                    expected_version, expected_name
                );
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
        current_version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Current version should match the final sequential migration"
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
    assert_eq!(
        current_version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Current version should match the final sequential migration"
    );

    let applied = runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        EXPECTED_MIGRATIONS.len(),
        "Should have {} applied migrations",
        EXPECTED_MIGRATIONS.len()
    );

    // Verify all expected migrations are in the applied list
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "Applied migration should have sequential version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "Applied migration {} should be named {}",
            expected_version, expected_name
        );
        assert!(
            !record.checksum.is_empty(),
            "Migration checksum should not be empty"
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
    assert_eq!(
        version_before,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Should be at the final version before rollback"
    );

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
    let expected_applied_count = target_version as usize;
    assert_eq!(
        applied.len(),
        expected_applied_count,
        "Only {} migrations should remain applied",
        expected_applied_count
    );
    for ((expected_version, expected_name), record) in EXPECTED_MIGRATIONS
        .iter()
        .take(expected_applied_count)
        .zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "After rollback, applied migration should have version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "After rollback, migration {} should be named {}",
            expected_version, expected_name
        );
    }

    // Verify that plan shows the rolled-back migrations as pending
    let plan = runner.plan_migrations().await?;
    assert_eq!(plan.current_version, target_version);
    let expected_pending: Vec<_> = EXPECTED_MIGRATIONS
        .iter()
        .skip(expected_applied_count)
        .collect();
    assert_eq!(
        plan.planned_migrations.len(),
        expected_pending.len(),
        "{} migrations should be pending after rollback to version {}",
        expected_pending.len(),
        target_version
    );

    // Verify the pending migrations are the ones we rolled back
    for ((expected_version, expected_name), migration) in expected_pending
        .into_iter()
        .zip(plan.planned_migrations.iter())
    {
        assert_eq!(
            migration.version, *expected_version,
            "Pending migration should have version {}",
            expected_version
        );
        assert_eq!(
            migration.name, *expected_name,
            "Pending migration {} should be named {}",
            expected_version, expected_name
        );
    }

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
    assert_eq!(
        plan.target_version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Dry-run target version should match final migration"
    );
    assert_eq!(
        plan.planned_migrations.len(),
        EXPECTED_MIGRATIONS.len(),
        "Dry-run should list all pending migrations"
    );
    for ((expected_version, expected_name), migration) in EXPECTED_MIGRATIONS
        .iter()
        .zip(plan.planned_migrations.iter())
    {
        assert_eq!(
            migration.version, *expected_version,
            "Dry-run should list migration version {}",
            expected_version
        );
        assert_eq!(
            migration.name, *expected_name,
            "Dry-run migration {} should be named {}",
            expected_version, expected_name
        );
    }

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
    assert_eq!(
        results.len(),
        EXPECTED_MIGRATIONS.len(),
        "Should apply all {} migrations",
        EXPECTED_MIGRATIONS.len()
    );

    // Verify all migrations succeeded
    for ((expected_version, expected_name), result) in
        EXPECTED_MIGRATIONS.iter().zip(results.iter())
    {
        match result {
            uveddi::database::migrations::MigrationResult::Applied { version, name } => {
                assert_eq!(
                    *version, *expected_version,
                    "Applied migration should have sequential version {}",
                    expected_version
                );
                assert_eq!(
                    name, expected_name,
                    "Applied migration {} should be named {}",
                    expected_version, expected_name
                );
            }
            _ => panic!("Unexpected migration result: {:?}", result),
        }
    }

    // Verify database state
    let current_version = runner.current_version().await?;
    assert_eq!(
        current_version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "All migrations should be applied"
    );

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
    assert_eq!(
        current_version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Current version should match the final migration"
    );

    let applied = runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        EXPECTED_MIGRATIONS.len(),
        "Should report {} applied migrations",
        EXPECTED_MIGRATIONS.len()
    );

    // Verify each migration record has required fields
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "Status should report migration version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "Status should report migration {} as {}",
            expected_version, expected_name
        );
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
    let expected_rolled_back: Vec<_> = EXPECTED_MIGRATIONS
        .iter()
        .rev()
        .take(EXPECTED_MIGRATIONS.len() - 3)
        .collect();
    assert_eq!(
        rollback_results.len(),
        expected_rolled_back.len(),
        "Should roll back {} migrations",
        expected_rolled_back.len()
    );
    for ((expected_version, expected_name), result) in expected_rolled_back
        .into_iter()
        .zip(rollback_results.iter())
    {
        match result {
            uveddi::database::migrations::MigrationResult::Applied { version, name } => {
                assert_eq!(
                    *version, *expected_version,
                    "Rolled back migration should have version {}",
                    expected_version
                );
                let expected_rollback_name = format!("Rollback: {}", expected_name);
                assert_eq!(
                    name, &expected_rollback_name,
                    "Rolled back migration {} should report name {}",
                    expected_version, expected_rollback_name
                );
            }
            other => panic!(
                "Expected rollback to report Applied result, got {:?} for migration {} ({})",
                other, expected_version, expected_name
            ),
        }
    }

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
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().take(3).zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "After rollback, applied migration should have version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "After rollback, migration {} should be named {}",
            expected_version, expected_name
        );
    }

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
    assert_eq!(
        results1.len(),
        EXPECTED_MIGRATIONS.len(),
        "First run should apply {} migrations",
        EXPECTED_MIGRATIONS.len()
    );
    for ((expected_version, expected_name), result) in
        EXPECTED_MIGRATIONS.iter().zip(results1.iter())
    {
        match result {
            uveddi::database::migrations::MigrationResult::Applied { version, name } => {
                assert_eq!(
                    *version, *expected_version,
                    "First run should apply migration version {}",
                    expected_version
                );
                assert_eq!(
                    name, expected_name,
                    "First run should apply migration {} named {}",
                    expected_version, expected_name
                );
            }
            other => panic!(
                "Expected Applied result for migration {} ({}), got {:?}",
                expected_version, expected_name, other
            ),
        }
    }

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
    let plan = runner.plan_migrations().await?;
    assert_eq!(
        plan.planned_migrations.len(),
        EXPECTED_MIGRATIONS.len(),
        "Plan command should report all pending migrations"
    );
    for ((expected_version, expected_name), migration) in EXPECTED_MIGRATIONS
        .iter()
        .zip(plan.planned_migrations.iter())
    {
        assert_eq!(
            migration.version, *expected_version,
            "Plan command should list migration version {}",
            expected_version
        );
        assert_eq!(
            migration.name, *expected_name,
            "Plan command should list migration {} named {}",
            expected_version, expected_name
        );
    }

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
    assert_eq!(
        version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Up command should apply all migrations"
    );

    let applied = runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        EXPECTED_MIGRATIONS.len(),
        "Up command should apply {} migrations",
        EXPECTED_MIGRATIONS.len()
    );
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "Up command should apply migration version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "Up command should apply migration {} named {}",
            expected_version, expected_name
        );
    }

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

    // Verify status reflects applied migrations
    let status_runner = harness.create_runner().await?;
    let applied = status_runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        EXPECTED_MIGRATIONS.len(),
        "Status command should report {} applied migrations",
        EXPECTED_MIGRATIONS.len()
    );
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "Status command should report migration version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "Status command should report migration {} named {}",
            expected_version, expected_name
        );
    }

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
    assert_eq!(
        version_before,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Should start at the final migration version"
    );

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
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().take(3).zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "After CLI rollback, applied migration should have version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "After CLI rollback, migration {} should be named {}",
            expected_version, expected_name
        );
    }

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
    assert_eq!(
        version,
        EXPECTED_MIGRATIONS.last().unwrap().0,
        "Multiple up commands should leave database at final version"
    );
    let applied = runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        EXPECTED_MIGRATIONS.len(),
        "Idempotent runs should still report {} applied migrations",
        EXPECTED_MIGRATIONS.len()
    );
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "Idempotent run should retain migration version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "Idempotent run should retain migration {} named {}",
            expected_version, expected_name
        );
    }

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
    let applied = runner.applied_migrations().await?;
    assert_eq!(
        applied.len(),
        3,
        "No-op down command should retain previously applied migrations"
    );
    for ((expected_version, expected_name), record) in
        EXPECTED_MIGRATIONS.iter().take(3).zip(applied.iter())
    {
        assert_eq!(
            record.version, *expected_version,
            "No-op down command should retain migration version {}",
            expected_version
        );
        assert_eq!(
            record.name, *expected_name,
            "No-op down command should retain migration {} named {}",
            expected_version, expected_name
        );
    }

    Ok(())
}
