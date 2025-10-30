//! Integration tests for the migration system
//!
//! These tests verify that the migration system works correctly end-to-end

use std::sync::Arc;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio;

use uveddi::database::{
    connection::{ConnectionPool, DatabaseConfig, DatabaseType, PoolConfig},
    migrations::{create_standard_registry, runner::MigrationRunner},
};

async fn create_test_setup() -> (Arc<ConnectionPool>, NamedTempFile) {
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
    (pool, temp_file)
}

#[tokio::test]
async fn test_migration_system_initialization() {
    let (pool, _temp_file) = create_test_setup().await;
    let registry = create_standard_registry();
    let runner = MigrationRunner::new(pool, registry);

    // Initialize the migration system
    let result = runner.initialize().await;
    assert!(result.is_ok(), "Migration system should initialize successfully");
}

#[tokio::test]
async fn test_migration_system_current_version() {
    let (pool, _temp_file) = create_test_setup().await;
    let registry = create_standard_registry();
    let runner = MigrationRunner::new(pool, registry);

    // Initialize and check current version
    runner.initialize().await.unwrap();
    let current_version = runner.current_version().await.unwrap();
    assert_eq!(current_version, 0, "Initial version should be 0");
}

#[tokio::test]
async fn test_migration_system_run_migrations() {
    let (pool, _temp_file) = create_test_setup().await;
    let registry = create_standard_registry();
    let runner = MigrationRunner::new(pool, registry);

    // Initialize the migration system
    runner.initialize().await.unwrap();

    // Run all migrations
    let results = runner.run_pending_migrations().await.unwrap();

    // Should have 7 migrations (cache, metrics, events, issues, dependencies, security, debt)
    assert_eq!(results.len(), 7, "Should run 7 migrations");

    // Check that all migrations were successful
    for result in &results {
        match result {
            uveddi::database::migrations::MigrationResult::Applied { version, name } => {
                println!("Migration {} ({}) applied successfully", version, name);
            }
            uveddi::database::migrations::MigrationResult::Failed { version, error } => {
                panic!("Migration {} failed: {}", version, error);
            }
            uveddi::database::migrations::MigrationResult::Skipped { version, reason } => {
                println!("Migration {} skipped: {}", version, reason);
            }
        }
    }

    // Check final version
    let final_version = runner.current_version().await.unwrap();
    assert_eq!(final_version, 7, "Final version should be 7 after all migrations");
}

#[tokio::test]
async fn test_migration_system_applied_migrations() {
    let (pool, _temp_file) = create_test_setup().await;
    let registry = create_standard_registry();
    let runner = MigrationRunner::new(pool, registry);

    // Initialize and run migrations
    runner.initialize().await.unwrap();
    runner.run_pending_migrations().await.unwrap();

    // Get applied migrations
    let applied = runner.applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 7, "Should have 7 applied migrations");

    // Check that they are in order
    for (i, migration) in applied.iter().enumerate() {
        assert_eq!(migration.version, (i + 1) as u32);
    }
}

#[tokio::test]
async fn test_migration_system_idempotency() {
    let (pool, _temp_file) = create_test_setup().await;
    let registry = create_standard_registry();
    let runner = MigrationRunner::new(pool, registry);

    // Initialize and run migrations
    runner.initialize().await.unwrap();
    let results1 = runner.run_pending_migrations().await.unwrap();
    assert_eq!(results1.len(), 7);

    // Run again - should be idempotent (no new migrations)
    let results2 = runner.run_pending_migrations().await.unwrap();
    assert_eq!(results2.len(), 0, "Second run should have no pending migrations");

    let final_version = runner.current_version().await.unwrap();
    assert_eq!(final_version, 7);
}

#[tokio::test]
async fn test_migration_registry() {
    let registry = create_standard_registry();
    let all_migrations = registry.get_all_sorted();

    // Verify we have the expected migrations
    assert_eq!(all_migrations.len(), 7);

    // Check specific migrations exist
    let cache_migration = registry.get(1).unwrap();
    assert_eq!(cache_migration.name, "create_cache_table");
    assert!(cache_migration.up_sql.contains("CREATE TABLE cache"));

    let metrics_migration = registry.get(2).unwrap();
    assert_eq!(metrics_migration.name, "create_metrics_table");
    assert!(metrics_migration.up_sql.contains("CREATE TABLE metrics"));

    // Test pending migrations
    let pending = registry.get_pending(3);
    assert_eq!(pending.len(), 4); // Migrations 4, 5, 6, 7
}