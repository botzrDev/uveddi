use anyhow::{Context, Result};
use refinery::embed_migrations;
use rusqlite::Connection;
use std::path::{Path, PathBuf};



// Embed migrations at compile time
embed_migrations!("migrations");

/// Database error types
#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] refinery::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Main database manager for CodeAtlas
pub struct DatabaseManager {
    connection: Connection,
}

impl DatabaseManager {
    /// Establish database connection and run migrations
    pub async fn new(db_path: Option<&Path>) -> Result<Self, DatabaseError> {
        let db_path = db_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Self::default_db_path());

        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        // Open connection
        let mut connection = Connection::open(&db_path)
            .context("Failed to open SQLite database")?;

        // Enable foreign key constraints
        connection
            .execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign key constraints")?;

        // Run migrations
        Self::run_migrations(&mut connection)?;

        Ok(Self { connection })
    }

    /// Get the default database path (.codeatlas/data.db in current directory)
    fn default_db_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push(".codeatlas");
        path.push("data.db");
        path
    }

    /// Run all pending migrations
    fn run_migrations(connection: &mut Connection) -> Result<(), DatabaseError> {
        migrations::runner().run(connection)?;
        Ok(())
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Get a mutable reference to the database connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

pub mod crud;
