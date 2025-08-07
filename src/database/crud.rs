use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::error::Result;
use crate::security;
use chrono::Utc;
use crate::core::logging::error;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Creates a new database connection and initializes tables if needed.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Optional path to the SQLite database file. If `None`, uses an in-memory database.
    ///
    /// # Returns
    ///
    /// * `Ok(Database)` - The initialized database instance.
    /// * `Err(UveddiError)` - If the database cannot be opened or initialized.
    pub fn new(db_path: Option<&Path>) -> Result<Self> {
        let conn = match db_path {
            Some(path) => Connection::open(path).map_err(crate::error::UveddiError::from)?,
            None => Connection::open_in_memory().map_err(crate::error::UveddiError::from)?,
        };
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS analysis_runs (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                status TEXT NOT NULL,
                total_files_analyzed INTEGER,
                total_issues_found INTEGER,
                analysis_config TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS anti_pattern_types (
                anti_pattern_type_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                category TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS architectural_issues (
                issue_id INTEGER PRIMARY KEY AUTOINCREMENT,
                analysis_run_id INTEGER NOT NULL,
                anti_pattern_type_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                line_number INTEGER,
                column_number INTEGER,
                message TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                detector_name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                severity TEXT NOT NULL,
                description TEXT NOT NULL,
                code_snippet TEXT,
                ai_explanation TEXT,
                FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(run_id),
                FOREIGN KEY (anti_pattern_type_id) REFERENCES anti_pattern_types(anti_pattern_type_id)
            );
            CREATE TABLE IF NOT EXISTS projects (
                project_id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE
            );
        ").map_err(crate::error::UveddiError::from)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    /// Gets the project ID for the given path, creating a new project entry if needed.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project directory.
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` - The project ID.
    /// * `Err(UveddiError)` - If the query or insert fails.
    pub fn get_or_create_project_id(&self, project_path: &Path) -> Result<i64> {
        let path_str = project_path.to_string_lossy().to_string();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT project_id FROM projects WHERE path = ?")?;
        let mut rows = stmt.query([&path_str])?;

        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            conn
                .execute("INSERT INTO projects (path) VALUES (?)", [&path_str])?;
            Ok(conn.last_insert_rowid())
        }
    }

    /// Creates a new analysis run entry for the given project path.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project directory.
    ///
    /// # Returns
    ///
    /// * `Ok(AnalysisRun)` - The created analysis run record.
    /// * `Err(UveddiError)` - If the insert fails.
    pub fn create_analysis_run(&self, project_path: &Path) -> Result<AnalysisRun> {
        let project_id = self.get_or_create_project_id(project_path)?;
        let analysis_run = AnalysisRun {
            run_id: None,
            project_id,
            start_time: Utc::now(),
            end_time: None,
            status: "running".to_string(),
            total_files_analyzed: None,
            total_issues_found: None,
            analysis_config: "{}".to_string(), // Default empty JSON config
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO analysis_runs (project_id, start_time, status, analysis_config) VALUES (?, ?, ?, ?)",
            rusqlite::params![
                analysis_run.project_id,
                analysis_run.start_time.to_rfc3339(),
                analysis_run.status,
                analysis_run.analysis_config,
            ],
        )?;

        let last_id = conn.last_insert_rowid();
        Ok(AnalysisRun {
            run_id: Some(last_id),
            ..analysis_run
        })
    }

    /// Updates an existing analysis run with new status, end time, and metrics.
    ///
    /// # Arguments
    ///
    /// * `run` - The analysis run to update (must have a valid `run_id`).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update succeeds.
    /// * `Err(UveddiError)` - If the update fails.
    pub fn update_analysis_run(&self, run: &AnalysisRun) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE analysis_runs SET end_time = ?, status = ?, total_files_analyzed = ?, total_issues_found = ? WHERE run_id = ?",
            rusqlite::params![
                run.end_time.map(|dt| dt.to_rfc3339()),
                run.status,
                run.total_files_analyzed,
                run.total_issues_found,
                run.run_id,
            ],
        )?;
        Ok(())
    }

    /// Stores a single anti-pattern type in the database, updating its ID if newly inserted.
    ///
    /// # Arguments
    ///
    /// * `anti_pattern_type` - The anti-pattern type to store (ID will be set if inserted).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the operation succeeds.
    /// * `Err(UveddiError)` - If the insert or query fails.
    pub fn store_anti_pattern_type(&self, anti_pattern_type: &mut AntiPatternType) -> Result<()> {
        // Note: Skip strict validation for internal anti-pattern types as they contain
        // legitimate technical terms that may trigger false positives in SQL injection detection
        // Only sanitize to ensure safe storage
        anti_pattern_type.description =
            security::sanitize_description(&anti_pattern_type.description);

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO anti_pattern_types (name, description, category) VALUES (?, ?, ?)",
            rusqlite::params![
                anti_pattern_type.name,
                anti_pattern_type.description,
                anti_pattern_type.category,
            ],
        )?;
        if anti_pattern_type.anti_pattern_type_id.is_none() {
            let mut stmt = conn
                .prepare("SELECT anti_pattern_type_id FROM anti_pattern_types WHERE name = ?")?;
            anti_pattern_type.anti_pattern_type_id =
                Some(stmt.query_row([&anti_pattern_type.name], |row| row.get(0))?);
        }
        Ok(())
    }

    /// Stores architectural issues in a transaction to ensure data consistency.
    ///
    /// # Arguments
    ///
    /// * `issues` - Slice of architectural issues to store.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all issues are stored successfully.
    /// * `Err(UveddiError)` - If any insert fails.
    pub fn store_issues(&mut self, issues: &[ArchitecturalIssue]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        for issue in issues {
            // Use code analysis validation for analysis results (more permissive than user input)
            security::validate_code_analysis_data(&issue.description, "description", None)
                .map_err(crate::error::UveddiError::from)?;
            security::validate_file_path_for_storage(&issue.file_path, "file_path")
                .map_err(crate::error::UveddiError::from)?;
            security::validate_input(&issue.severity, "severity")
                .map_err(crate::error::UveddiError::from)?;

            // Validate line numbers
            if let Some(start_line) = issue.start_line {
                security::validate_numeric_range(start_line as i64, 1, 1_000_000, "start_line")
                    .map_err(crate::error::UveddiError::from)?;
            }
            if let Some(end_line) = issue.end_line {
                security::validate_numeric_range(end_line as i64, 1, 1_000_000, "end_line")
                    .map_err(crate::error::UveddiError::from)?;
            }

            // Validate code snippet if present - use code analysis validation
            if let Some(ref snippet) = issue.code_snippet {
                security::validate_code_analysis_data(snippet, "code_snippet", None)
                    .map_err(crate::error::UveddiError::from)?;
            }

            // Validate AI explanation if present - use code analysis validation
            if let Some(ref explanation) = issue.ai_explanation {
                security::validate_code_analysis_data(explanation, "ai_explanation", None)
                    .map_err(crate::error::UveddiError::from)?;
            }

            // Sanitize description and AI explanation after validation
            let sanitized_description = security::sanitize_description(&issue.description);
            let sanitized_ai_explanation = if let Some(ref explanation) = issue.ai_explanation {
                Some(security::sanitize_description(explanation))
            } else {
                None
            };

            tx.execute(
                "INSERT INTO architectural_issues (analysis_run_id, anti_pattern_type_id, file_path, start_line, end_line, line_number, column_number, message, metadata, detector_name, created_at, severity, description, code_snippet, ai_explanation) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    issue.analysis_run_id,
                    issue.anti_pattern_type_id,
                    issue.file_path,
                    issue.start_line,
                    issue.end_line,
                    issue.line_number,
                    issue.column_number,
                    issue.message,
                    issue.metadata,
                    issue.detector_name,
                    issue.created_at.to_rfc3339(),
                    issue.severity,
                    sanitized_description,
                    issue.code_snippet,
                    sanitized_ai_explanation,
                ],
            )?;
        }
        match tx.commit() {
            Ok(_) => {
                tracing::debug!("Successfully stored {} architectural issues", issues.len());
                Ok(())
            },
            Err(e) => {
                // Enhanced error logging to expose specific SQLite error codes
                error!("Transaction commit failed while storing {} issues: {:?}", issues.len(), e);
                if let rusqlite::Error::SqliteFailure(sqlite_err, Some(msg)) = &e {
                    error!("Underlying SQLite error message: {}", msg);
                    error!("SQLite extended error code: {}", sqlite_err.extended_code);
                }
                if let Some(error_code) = e.sqlite_error_code() {
                    error!("SQLite primary error code: {:?}", error_code);
                }
                
                // Provide more specific error context based on the error type
                let context_msg = match &e {
                    rusqlite::Error::SqliteFailure(sqlite_err, _) => {
                        match sqlite_err.code {
                            rusqlite::ErrorCode::ConstraintViolation => "Foreign key constraint violation - ensure analysis_run_id and anti_pattern_type_id are valid",
                            rusqlite::ErrorCode::SchemaChanged => "Database schema mismatch - database may need to be recreated",
                            rusqlite::ErrorCode::DatabaseCorrupt => "Database corruption detected - consider recreating the database",
                            _ => "Database operation failed during issue storage"
                        }
                    },
                    _ => "Unexpected database error during transaction commit"
                };
                
                Err(crate::error::UveddiError::database_error_msg(
                    &format!("Failed to store architectural issues: {}", context_msg)
                ))
            }
        }
    }

    /// Stores multiple anti-pattern types in a batch operation.
    ///
    /// # Arguments
    ///
    /// * `anti_pattern_types` - Mutable slice of anti-pattern types to store (IDs will be set if inserted).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all types are stored successfully.
    /// * `Err(UveddiError)` - If any insert or query fails.
    pub fn store_anti_pattern_types_batch(
        &mut self,
        anti_pattern_types: &mut [AntiPatternType],
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO anti_pattern_types (name, description, category) VALUES (?, ?, ?)"
            )?;
            for anti_pattern_type in anti_pattern_types.iter_mut() {
                // Validate and sanitize description
                anti_pattern_type.description =
                    security::sanitize_description(&anti_pattern_type.description);

                stmt.execute(rusqlite::params![
                    anti_pattern_type.name,
                    anti_pattern_type.description,
                    anti_pattern_type.category,
                ])?;
                if anti_pattern_type.anti_pattern_type_id.is_none() {
                    let mut id_stmt = tx.prepare(
                        "SELECT anti_pattern_type_id FROM anti_pattern_types WHERE name = ?",
                    )?;
                    anti_pattern_type.anti_pattern_type_id =
                        Some(id_stmt.query_row([&anti_pattern_type.name], |row| row.get(0))?);
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Get the project path by project ID
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project ID to look up
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The project path
    /// * `Err(UveddiError)` - If the query fails or project not found
    pub fn get_project_path(&self, project_id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT path FROM projects WHERE project_id = ?")?;
        let path = stmt.query_row([project_id], |row| row.get::<_, String>(0))?;
        Ok(path)
    }

    /// Retrieves all anti-pattern types from the database.
    ///
    /// This method fetches all anti-pattern type definitions that have been
    /// stored in the database, which are needed for proper report generation
    /// and issue categorization.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<AntiPatternType>)` - Vector of all anti-pattern types in the database
    /// * `Err(UveddiError)` - If the query fails
    pub fn get_all_anti_pattern_types(&self) -> Result<Vec<AntiPatternType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT anti_pattern_type_id, name, description, category FROM anti_pattern_types ORDER BY name"
        )?;
        
        let anti_pattern_iter = stmt.query_map([], |row| {
            Ok(AntiPatternType {
                anti_pattern_type_id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
            })
        })?;

        let mut anti_patterns = Vec::new();
        for anti_pattern in anti_pattern_iter {
            anti_patterns.push(anti_pattern?);
        }
        
        Ok(anti_patterns)
    }
}
