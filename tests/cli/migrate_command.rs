//! Integration tests for the migrate CLI command
//!
//! Tests the full lifecycle of database migrations including plan, up, down, and status commands.
//! Uses temporary databases to avoid polluting the repository.

use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use uveddi::cli::commands::migrate::{MigrateCommand, MigrateSubcommand};
use uveddi::database::connection::{config::DatabaseConfig, ConnectionPool};
use uveddi::database::migrations::{create_standard_registry, runner::MigrationRunner};
use std::sync::Arc;

/// Test harness that provides a temporary database for migration testing
struct MigrationTestHarness {
    _temp_dir: TempDir,  // Kept alive to ensure cleanup on drop
    db_path: PathBuf,
}

impl MigrationTestHarness {
    /// Create a new test harness with a temporary database
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_migration.db");
        
        Ok(Self { _temp_dir: temp_dir, db_path })
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
async fn test_migrate_plan_shows_pending_migrations() -> Result<(), Box<dyn std::error::Error>> {
    let harness = MigrationTestHarness::new()?;
    let runner = harness.create_runner().await?;

    // Get the migration plan
    let plan = runner.plan_migrations().await?;

    // Verify that we have pending migrations from 20251001 to 20251007
    assert_eq!(plan.current_version, 0, "Initial version should be 0");
    assert!(plan.target_version >= 20251007, "Target version should include all migrations");
    assert_eq!(plan.planned_migrations.len(), 7, "Should have 7 pending migrations (20251001-20251007)");

    // Verify migrations are in order
    let expected_versions = vec![20251001, 20251002, 20251003, 20251004, 20251005, 20251006, 20251007];
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
    assert!(harness.db_exists().await, "Database should exist after initialization");

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
    assert_eq!(current_version, 20251007, "Current version should be 20251007 after all migrations");

    // Verify no pending migrations remain
    let plan = runner.plan_migrations().await?;
    assert_eq!(plan.planned_migrations.len(), 0, "No pending migrations should remain");

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
    assert_eq!(initial_applied.len(), 0, "No migrations should be applied initially");

    // Apply migrations
    runner.run_pending_migrations().await?;

    // Check status after migrations
    let current_version = runner.current_version().await?;
    assert_eq!(current_version, 20251007, "Current version should be 20251007");

    let applied = runner.applied_migrations().await?;
    assert_eq!(applied.len(), 7, "7 migrations should be applied");

    // Verify all expected migrations are in the applied list
    let expected_versions = vec![20251001, 20251002, 20251003, 20251004, 20251005, 20251006, 20251007];
    for (i, record) in applied.iter().enumerate() {
        assert_eq!(record.version, expected_versions[i]);
        assert!(!record.name.is_empty(), "Migration name should not be empty");
        assert!(!record.checksum.is_empty(), "Migration checksum should not be empty");
        println!("✓ Migration v{}: {} applied at {}", record.version, record.name, record.applied_at);
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
    assert_eq!(version_before, 20251007, "Should be at version 20251007 before rollback");

    // Rollback to version 20251003
    let target_version = 20251003;
    let rollback_results = runner.rollback_to_version(target_version).await?;

    // Verify rollback occurred
    assert!(rollback_results.len() > 0, "Some migrations should have been rolled back");

    // Check current version after rollback
    let current_version = runner.current_version().await?;
    assert_eq!(current_version, target_version, "Should be at version {} after rollback", target_version);

    // Verify applied migrations list is correct
    let applied = runner.applied_migrations().await?;
    assert_eq!(applied.len(), 3, "Only 3 migrations should remain applied");

    // Verify that plan shows the rolled-back migrations as pending
    let plan = runner.plan_migrations().await?;
    assert_eq!(plan.current_version, target_version);
    assert_eq!(plan.planned_migrations.len(), 4, "4 migrations should be pending after rollback to 20251003");

    // Verify the pending migrations are the ones we rolled back
    let pending_versions: Vec<u32> = plan.planned_migrations.iter().map(|m| m.version).collect();
    assert_eq!(pending_versions, vec![20251004, 20251005, 20251006, 20251007]);

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
    assert!(plan.target_version >= 20251007);
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
    assert_eq!(current_version, 20251007, "All migrations should be applied");
    
    // Verify database file size
    let size = harness.db_size().await?;
    assert!(size > 8192, "Database should have substantial content after migrations");

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
    assert_eq!(current_version, 20251007);
    
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
    let rollback_results = runner.rollback_to_version(20251003).await?;
    assert!(rollback_results.len() > 0, "Should have rolled back some migrations");

    // Verify rollback occurred
    let current_version = runner.current_version().await?;
    assert_eq!(current_version, 20251003, "Should be rolled back to version 20251003");
    
    // Verify only 3 migrations remain applied
    let applied = runner.applied_migrations().await?;
    assert_eq!(applied.len(), 3, "Only 3 migrations should remain after rollback");

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
    assert_eq!(results2.len(), 0, "Second run should apply 0 migrations (idempotent)");

    Ok(())
}
