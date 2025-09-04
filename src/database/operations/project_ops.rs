//! Project-related database operations
//!
//! This module handles all database operations related to project management,
//! including project creation, retrieval, and path management.

use crate::error::{Result, UveddiError};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Project operations handler
pub struct ProjectOperations {
    conn: Arc<Mutex<Connection>>,
}

impl ProjectOperations {
    /// Create a new ProjectOperations instance
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Gets an existing project ID or creates a new project entry.
    ///
    /// This method first attempts to find a project with the given path.
    /// If no project is found, it creates a new project entry and returns
    /// the newly generated ID.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project directory
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` - The project ID (existing or newly created)
    /// * `Err(UveddiError)` - If the database operation fails
    pub fn get_or_create_project_id(&self, project_path: &Path) -> Result<i64> {
        let path_str = project_path.to_string_lossy().to_string();
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare("SELECT project_id FROM projects WHERE path = ?")?;
        let mut rows = stmt.query([&path_str])?;

        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            conn.execute("INSERT INTO projects (path) VALUES (?)", [&path_str])?;
            Ok(conn.last_insert_rowid())
        }
    }

    /// Gets the project path for a given project ID.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The ID of the project to look up
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The project path
    /// * `Err(UveddiError)` - If the query fails or project not found
    pub fn get_project_path(&self, project_id: i64) -> Result<String> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare("SELECT path FROM projects WHERE project_id = ?")?;
        let path = stmt.query_row([project_id], |row| row.get::<_, String>(0))?;
        Ok(path)
    }

    /// Gets all projects in the database
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(i64, String)>)` - Vector of (project_id, path) tuples
    /// * `Err(UveddiError)` - If the query fails
    pub fn get_all_projects(&self) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare("SELECT project_id, path FROM projects ORDER BY project_id")?;
        let project_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }
        Ok(projects)
    }
}