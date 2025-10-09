//! Database migration command
//!
//! Handles database schema migrations with support for applying, rolling back,
//! and planning migrations.

use crate::database::connection::{config::DatabaseConfig, ConnectionPool};
use crate::database::migrations::{create_standard_registry, runner::MigrationRunner};
use crate::error::{Result, UveddiError};
use clap::{Args, Subcommand};
use std::sync::Arc;
use tracing::{error, info};

/// Database migration management
#[derive(Args, Debug)]
pub struct MigrateCommand {
    #[command(subcommand)]
    subcommand: MigrateSubcommand,
}

/// Migration subcommands
#[derive(Subcommand, Debug)]
pub enum MigrateSubcommand {
    /// Apply all pending migrations
    Up {
        /// Database file path (defaults to uveddi_cache.db)
        #[arg(short, long, default_value = "uveddi_cache.db")]
        database: String,
    },
    /// Rollback to a specific version
    Down {
        /// Target version to rollback to
        #[arg(short, long)]
        version: u32,

        /// Database file path (defaults to uveddi_cache.db)
        #[arg(short, long, default_value = "uveddi_cache.db")]
        database: String,
    },
    /// Show migration status and plan without applying (dry-run)
    Plan {
        /// Database file path (defaults to uveddi_cache.db)
        #[arg(short, long, default_value = "uveddi_cache.db")]
        database: String,
    },
    /// Show current migration version
    Status {
        /// Database file path (defaults to uveddi_cache.db)
        #[arg(short, long, default_value = "uveddi_cache.db")]
        database: String,
    },
}

impl MigrateCommand {
    /// Execute the migrate command
    pub async fn execute(&self) -> Result<()> {
        match &self.subcommand {
            MigrateSubcommand::Up { database } => self.run_migrations(database).await,
            MigrateSubcommand::Down { version, database } => {
                self.rollback_migrations(*version, database).await
            }
            MigrateSubcommand::Plan { database } => self.plan_migrations(database).await,
            MigrateSubcommand::Status { database } => self.show_status(database).await,
        }
    }

    /// Apply all pending migrations
    async fn run_migrations(&self, database_path: &str) -> Result<()> {
        info!("Running database migrations for: {}", database_path);

        let runner = self.create_runner(database_path).await?;
        let results = runner
            .run_pending_migrations()
            .await
            .map_err(|e| UveddiError::database_error_msg(&format!("Migration failed: {}", e)))?;

        if results.is_empty() {
            println!("✓ Database is up to date - no migrations to apply.");
        } else {
            println!("Applied {} migration(s):", results.len());
            for result in results {
                match result {
                    crate::database::migrations::MigrationResult::Applied { version, name } => {
                        println!("  ✓ v{}: {}", version, name);
                    }
                    crate::database::migrations::MigrationResult::Failed { version, error } => {
                        error!("  ✗ v{}: {}", version, error);
                        return Err(UveddiError::database_error_msg(&format!(
                            "Migration {} failed: {}",
                            version, error
                        )));
                    }
                    crate::database::migrations::MigrationResult::Skipped { version, reason } => {
                        println!("  ⊙ v{}: skipped ({})", version, reason);
                    }
                }
            }
            println!("\n✓ All migrations completed successfully.");
        }

        Ok(())
    }

    /// Rollback migrations to a specific version
    async fn rollback_migrations(&self, target_version: u32, database_path: &str) -> Result<()> {
        info!("Rolling back migrations to version {}", target_version);

        let runner = self.create_runner(database_path).await?;
        let current_version = runner.current_version().await.map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to get current version: {}", e))
        })?;

        if current_version <= target_version {
            println!(
                "✓ Database is already at or below version {}.",
                target_version
            );
            return Ok(());
        }

        let results = runner
            .rollback_to_version(target_version)
            .await
            .map_err(|e| UveddiError::database_error_msg(&format!("Rollback failed: {}", e)))?;

        println!("Rolled back {} migration(s):", results.len());
        for result in results {
            match result {
                crate::database::migrations::MigrationResult::Applied { version, name } => {
                    println!("  ✓ {}", name);
                }
                crate::database::migrations::MigrationResult::Failed { version, error } => {
                    error!("  ✗ v{}: {}", version, error);
                    return Err(UveddiError::database_error_msg(&format!(
                        "Rollback {} failed: {}",
                        version, error
                    )));
                }
                crate::database::migrations::MigrationResult::Skipped { version, reason } => {
                    println!("  ⊙ v{}: skipped ({})", version, reason);
                }
            }
        }

        println!("\n✓ Rollback completed successfully.");
        Ok(())
    }

    /// Plan migrations without applying them (dry-run)
    async fn plan_migrations(&self, database_path: &str) -> Result<()> {
        info!("Planning migrations (dry-run) for: {}", database_path);

        let runner = self.create_runner(database_path).await?;
        let plan = runner.plan_migrations().await.map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to plan migrations: {}", e))
        })?;

        println!("{}", plan.display());

        Ok(())
    }

    /// Show current migration status
    async fn show_status(&self, database_path: &str) -> Result<()> {
        info!("Checking migration status for: {}", database_path);

        let runner = self.create_runner(database_path).await?;
        let current_version = runner.current_version().await.map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to get current version: {}", e))
        })?;

        let applied_migrations = runner.applied_migrations().await.map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to get applied migrations: {}", e))
        })?;

        println!("=== Migration Status ===\n");
        println!("Database: {}", database_path);
        println!("Current Version: {}\n", current_version);

        if applied_migrations.is_empty() {
            println!("No migrations have been applied yet.");
        } else {
            println!("Applied Migrations ({}):", applied_migrations.len());
            for migration in applied_migrations {
                println!(
                    "  ✓ v{}: {} (applied {})",
                    migration.version,
                    migration.name,
                    migration.applied_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }

        Ok(())
    }

    /// Create a migration runner for the specified database
    async fn create_runner(&self, database_path: &str) -> Result<MigrationRunner> {
        let config = DatabaseConfig::sqlite(database_path);
        let provider = Arc::new(
            crate::database::SqliteProvider::new(config.clone())
                .map_err(|e| UveddiError::database_error_msg(&e.to_string()))?,
        );

        let pool = ConnectionPool::new(config.into(), provider)
            .await
            .map_err(|e| UveddiError::database_error_msg(&e.to_string()))?;

        let registry = create_standard_registry();
        Ok(MigrationRunner::new(pool, registry))
    }
}
