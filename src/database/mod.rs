//! Database Layer for Uveddi
//!
//! This module provides comprehensive database functionality for persisting analysis
//! results, project metadata, and architectural insights. It uses SQLite for local
//! storage with support for both in-memory and file-based databases.
//!
//! # Architecture
//!
//! The database layer is organized into several key components:
//!
//! - **Models**: Data structures representing database entities
//! - **CRUD Operations**: Create, Read, Update, Delete operations
//! - **Migrations**: Database schema management
//! - **Connection Management**: Database connection pooling and lifecycle
//!
//! # Key Features
//!
//! - **Analysis Persistence**: Store analysis runs and their results
//! - **Issue Tracking**: Track architectural issues and anti-patterns
//! - **Project Management**: Manage project metadata and history
//! - **Performance Optimization**: Efficient queries and indexing
//!
//! # Usage
//!
//! ```rust,no_run
//! use uveddi::database::Database;
//! use std::path::Path;
//!
//! // Initialize database
//! let db = Database::new("analysis.db")?;
//!
//! // Create a new analysis run
//! let project_path = Path::new("./src");
//! let run = db.create_analysis_run(project_path)?;
//!
//! println!("Created analysis run with ID: {}", run.id);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Database Schema
//!
//! The database schema includes tables for:
//! - `projects`: Project metadata and paths
//! - `analysis_runs`: Individual analysis executions
//! - `anti_pattern_types`: Definitions of detectable anti-patterns
//! - `architectural_issues`: Specific issues found during analysis
//! - `dependencies`: Code dependency relationships
//!
//! See the [`models`] module for detailed entity definitions.

use tracing::{info, warn, error, debug};

pub mod crud;
pub mod models;

pub use self::crud::Database;
