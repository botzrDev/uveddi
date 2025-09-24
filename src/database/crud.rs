use crate::core::logging::error;
use crate::database::connection::{ConnectionManager, DatabaseConfig, DatabaseType};
use crate::database::models::{AnalysisRun, AnalysisStats, AntiPatternType, ArchitecturalIssue};
use crate::database::repositories::{
    AnalysisRepository, ProjectRepository, RepositoryManager, SqliteAnalysisRepository,
    SqliteProjectRepository, SqliteRepositoryFactory,
};
use crate::error::{Result, UveddiError};
use crate::security;
use chrono::Utc;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    // Legacy connection for backward compatibility
    conn: Arc<Mutex<Connection>>,
    // New repository-based architecture
    project_repo: Option<Arc<SqliteProjectRepository>>,
    analysis_repo: Option<Arc<SqliteAnalysisRepository>>,
    // Repository manager for advanced usage
    repository_manager: Option<Arc<RepositoryManager>>,
}

impl Database {
    /// Creates a legacy database connection (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// This method is deprecated. Use `Database::new_with_repositories()` instead.
    /// This method is maintained for backward compatibility only.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Optional path to the SQLite database file. If `None`, uses an in-memory database.
    ///
    /// # Returns
    ///
    /// * `Ok(Database)` - The initialized database instance.
    /// * `Err(UveddiError)` - If the database cannot be opened or initialized.
    #[deprecated(
        since = "0.9.0",
        note = "Use Database::new_with_repositories() instead"
    )]
    pub fn new(db_path: Option<&Path>) -> Result<Self> {
        let conn = match db_path {
            Some(path) => Connection::open(path).map_err(crate::error::UveddiError::from)?,
            None => Connection::open_in_memory().map_err(crate::error::UveddiError::from)?,
        };
        // Enable performance optimizations
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
        ",
        )?;

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
            CREATE TABLE IF NOT EXISTS dependencies (
                dependency_id INTEGER PRIMARY KEY AUTOINCREMENT,
                analysis_run_id INTEGER NOT NULL,
                from_file TEXT NOT NULL,
                to_module TEXT NOT NULL,
                dependency_type TEXT NOT NULL,
                line_number INTEGER,
                FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(run_id)
            );
        ")?;

        // Create performance indexes
        conn.execute_batch("
            CREATE INDEX IF NOT EXISTS idx_analysis_runs_project_time ON analysis_runs(project_id, start_time);
            CREATE INDEX IF NOT EXISTS idx_analysis_runs_status ON analysis_runs(status);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_run_id ON architectural_issues(analysis_run_id);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_file_path ON architectural_issues(file_path);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_severity ON architectural_issues(severity);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_detector ON architectural_issues(detector_name);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_type_id ON architectural_issues(anti_pattern_type_id);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_composite ON architectural_issues(analysis_run_id, severity, detector_name);
            CREATE INDEX IF NOT EXISTS idx_anti_pattern_types_name ON anti_pattern_types(name);
            CREATE INDEX IF NOT EXISTS idx_anti_pattern_types_category ON anti_pattern_types(category);
            CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
            CREATE INDEX IF NOT EXISTS idx_dependencies_run_id ON dependencies(analysis_run_id);
            CREATE INDEX IF NOT EXISTS idx_dependencies_from_file ON dependencies(from_file);
            CREATE INDEX IF NOT EXISTS idx_dependencies_to_module ON dependencies(to_module);
        ").map_err(crate::error::UveddiError::from)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            // For now, repositories are None to maintain compatibility
            // They will be initialized when the connection pooling is used
            project_repo: None,
            analysis_repo: None,
            repository_manager: None,
        })
    }

    /// Create a new Database instance with connection pooling and repositories
    pub async fn new_with_repositories(config: Option<DatabaseConfig>) -> Result<Self> {
        // Create a legacy instance for backward compatibility
        let legacy = match config.as_ref().map(|c| c.connection_string.as_str()) {
            Some(path) => Self::new(Some(Path::new(path)))?,
            None => Self::new(None)?,
        };

        // If config is provided, also set up repositories
        if let Some(config) = config {
            let manager = ConnectionManager::new(config)?;
            let pool = manager.create_pool().await?;

            let factory = Arc::new(SqliteRepositoryFactory::new(pool.clone()));
            let repository_manager = Arc::new(RepositoryManager::new(factory));

            let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));
            let analysis_repo = Arc::new(SqliteAnalysisRepository::new(pool));

            Ok(Self {
                conn: legacy.conn,
                project_repo: Some(project_repo),
                analysis_repo: Some(analysis_repo),
                repository_manager: Some(repository_manager),
            })
        } else {
            Ok(legacy)
        }
    }

    /// Get the repository manager if available
    pub fn repository_manager(&self) -> Option<Arc<RepositoryManager>> {
        self.repository_manager.clone()
    }

    /// Get the project repository if available
    pub fn project_repository(&self) -> Option<Arc<dyn ProjectRepository>> {
        self.project_repo
            .clone()
            .map(|r| r as Arc<dyn ProjectRepository>)
    }

    /// Get the analysis repository if available
    pub fn analysis_repository(&self) -> Option<Arc<dyn AnalysisRepository>> {
        self.analysis_repo
            .clone()
            .map(|r| r as Arc<dyn AnalysisRepository>)
    }

    /// Gets the project ID for the given path, creating a new project entry if needed (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use `ProjectRepository::get_or_create_project_id()` through the repository pattern instead.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project directory.
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` - The project ID.
    /// * `Err(UveddiError)` - If the query or insert fails.
    #[deprecated(
        since = "0.9.0",
        note = "Use ProjectRepository::get_or_create_project_id() instead"
    )]
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

    /// Creates a new analysis run entry for the given project path (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use `AnalysisRepository::create_analysis_run()` through the repository pattern instead.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project directory.
    ///
    /// # Returns
    ///
    /// * `Ok(AnalysisRun)` - The created analysis run record.
    /// * `Err(UveddiError)` - If the insert fails.
    #[deprecated(
        since = "0.9.0",
        note = "Use AnalysisRepository::create_analysis_run() instead"
    )]
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

        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
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

    /// Updates an existing analysis run with new status, end time, and metrics (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use `AnalysisRepository::update()` through the repository pattern instead.
    ///
    /// # Arguments
    ///
    /// * `run` - The analysis run to update (must have a valid `run_id`).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update succeeds.
    /// * `Err(UveddiError)` - If the update fails.
    #[deprecated(since = "0.9.0", note = "Use AnalysisRepository::update() instead")]
    pub fn update_analysis_run(&self, run: &AnalysisRun) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
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

    /// Stores a single anti-pattern type in the database, updating its ID if newly inserted (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern to store anti-pattern types instead.
    ///
    /// # Arguments
    ///
    /// * `anti_pattern_type` - The anti-pattern type to store (ID will be set if inserted).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the operation succeeds.
    /// * `Err(UveddiError)` - If the insert or query fails.
    #[deprecated(
        since = "0.9.0",
        note = "Use repository pattern for anti-pattern type storage"
    )]
    pub fn store_anti_pattern_type(&self, anti_pattern_type: &mut AntiPatternType) -> Result<()> {
        // Note: Skip strict validation for internal anti-pattern types as they contain
        // legitimate technical terms that may trigger false positives in SQL injection detection
        // Only sanitize to ensure safe storage
        anti_pattern_type.description =
            security::sanitize_description(&anti_pattern_type.description);

        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        conn.execute(
            "INSERT OR IGNORE INTO anti_pattern_types (name, description, category) VALUES (?, ?, ?)",
            rusqlite::params![
                anti_pattern_type.name,
                anti_pattern_type.description,
                anti_pattern_type.category,
            ],
        )?;
        if anti_pattern_type.anti_pattern_type_id.is_none() {
            let mut stmt =
                conn.prepare("SELECT anti_pattern_type_id FROM anti_pattern_types WHERE name = ?")?;
            anti_pattern_type.anti_pattern_type_id =
                Some(stmt.query_row([&anti_pattern_type.name], |row| row.get(0))?);
        }
        Ok(())
    }

    /// Stores architectural issues in a transaction to ensure data consistency (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern to store issues instead.
    ///
    /// # Arguments
    ///
    /// * `issues` - Slice of architectural issues to store.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all issues are stored successfully.
    /// * `Err(UveddiError)` - If any insert fails.
    #[deprecated(since = "0.9.0", note = "Use repository pattern for issue storage")]
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
                security::validate_numeric_range(
                    (start_line as i32).into(),
                    1,
                    1_000_000,
                    "start_line",
                )
                .map_err(crate::error::UveddiError::from)?;
            }
            if let Some(end_line) = issue.end_line {
                security::validate_numeric_range(
                    (end_line as i32).into(),
                    1,
                    1_000_000,
                    "end_line",
                )
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
            }
            Err(e) => {
                // Enhanced error logging to expose specific SQLite error codes
                error!(
                    "Transaction commit failed while storing {} issues: {:?}",
                    issues.len(),
                    e
                );
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

                Err(crate::error::UveddiError::database_error_msg(&format!(
                    "Failed to store architectural issues: {}",
                    context_msg
                )))
            }
        }
    }

    /// Stores multiple anti-pattern types in a batch operation (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern for batch operations instead.
    ///
    /// # Arguments
    ///
    /// * `anti_pattern_types` - Mutable slice of anti-pattern types to store (IDs will be set if inserted).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all types are stored successfully.
    /// * `Err(UveddiError)` - If any insert or query fails.
    #[deprecated(since = "0.9.0", note = "Use repository pattern for batch operations")]
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

    /// Get the project path by project ID (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use `ProjectRepository::get_project_path()` instead.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project ID to look up
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The project path
    /// * `Err(UveddiError)` - If the query fails or project not found
    #[deprecated(
        since = "0.9.0",
        note = "Use ProjectRepository::get_project_path() instead"
    )]
    pub fn get_project_path(&self, project_id: i64) -> Result<String> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare("SELECT path FROM projects WHERE project_id = ?")?;
        let path = stmt.query_row([project_id], |row| row.get::<_, String>(0))?;
        Ok(path)
    }

    /// Retrieves all anti-pattern types from the database (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern to retrieve anti-pattern types instead.
    ///
    /// This method fetches all anti-pattern type definitions that have been
    /// stored in the database, which are needed for proper report generation
    /// and issue categorization.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<AntiPatternType>)` - Vector of all anti-pattern types in the database
    /// * `Err(UveddiError)` - If the query fails
    #[deprecated(
        since = "0.9.0",
        note = "Use repository pattern for anti-pattern type retrieval"
    )]
    pub fn get_all_anti_pattern_types(&self) -> Result<Vec<AntiPatternType>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
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

    /// Get analysis run by ID (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use `AnalysisRepository::find_by_id()` instead.
    #[deprecated(since = "0.9.0", note = "Use AnalysisRepository::find_by_id() instead")]
    pub async fn get_analysis_run(&self, run_id: i64) -> Result<Option<AnalysisRun>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare(
            "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config 
             FROM analysis_runs WHERE run_id = ?"
        )?;

        let result = stmt.query_row([run_id], |row| {
            let start_time_str: String = row.get(2)?;
            let end_time_str: Option<String> = row.get(3)?;

            Ok(AnalysisRun {
                run_id: Some(row.get(0)?),
                project_id: row.get(1)?,
                start_time: chrono::DateTime::parse_from_rfc3339(&start_time_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            2,
                            "start_time".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                end_time: end_time_str.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
                status: row.get(4)?,
                total_files_analyzed: row.get(5)?,
                total_issues_found: row.get(6)?,
                analysis_config: row.get(7)?,
            })
        });

        match result {
            Ok(run) => Ok(Some(run)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(crate::error::UveddiError::from(e)),
        }
    }

    /// Get latest analysis run (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use `AnalysisRepository::find_latest()` instead.
    #[deprecated(
        since = "0.9.0",
        note = "Use AnalysisRepository::find_latest() instead"
    )]
    pub async fn get_latest_analysis_run(&self) -> Result<Option<AnalysisRun>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare(
            "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config 
             FROM analysis_runs ORDER BY start_time DESC LIMIT 1"
        )?;

        let result = stmt.query_row([], |row| {
            let start_time_str: String = row.get(2)?;
            let end_time_str: Option<String> = row.get(3)?;

            Ok(AnalysisRun {
                run_id: Some(row.get(0)?),
                project_id: row.get(1)?,
                start_time: chrono::DateTime::parse_from_rfc3339(&start_time_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            2,
                            "start_time".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                end_time: end_time_str.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
                status: row.get(4)?,
                total_files_analyzed: row.get(5)?,
                total_issues_found: row.get(6)?,
                analysis_config: row.get(7)?,
            })
        });

        match result {
            Ok(run) => Ok(Some(run)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(crate::error::UveddiError::from(e)),
        }
    }

    /// Get recent analysis runs (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use `AnalysisRepository::find_recent()` instead.
    #[deprecated(
        since = "0.9.0",
        note = "Use AnalysisRepository::find_recent() instead"
    )]
    pub async fn get_recent_analysis_runs(&self, limit: u32) -> Result<Vec<AnalysisRun>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare(
            "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config 
             FROM analysis_runs ORDER BY start_time DESC LIMIT ?"
        )?;

        let run_iter = stmt.query_map([limit], |row| {
            let start_time_str: String = row.get(2)?;
            let end_time_str: Option<String> = row.get(3)?;

            Ok(AnalysisRun {
                run_id: Some(row.get(0)?),
                project_id: row.get(1)?,
                start_time: chrono::DateTime::parse_from_rfc3339(&start_time_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            2,
                            "start_time".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                end_time: end_time_str.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
                status: row.get(4)?,
                total_files_analyzed: row.get(5)?,
                total_issues_found: row.get(6)?,
                analysis_config: row.get(7)?,
            })
        })?;

        let mut runs = Vec::new();
        for run in run_iter {
            runs.push(run?);
        }

        Ok(runs)
    }

    /// Get issues for a specific analysis run (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern to retrieve issues instead.
    #[deprecated(since = "0.9.0", note = "Use repository pattern for issue retrieval")]
    pub async fn get_issues_for_run(&self, run_id: i64) -> Result<Vec<ArchitecturalIssue>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare(
            "SELECT issue_id, analysis_run_id, anti_pattern_type_id, file_path, start_line, end_line, 
                    line_number, column_number, message, metadata, detector_name, created_at, severity, 
                    description, code_snippet, ai_explanation
             FROM architectural_issues WHERE analysis_run_id = ?"
        )?;

        let issue_iter = stmt.query_map([run_id], |row| {
            let created_at_str: String = row.get(11)?;

            Ok(ArchitecturalIssue {
                issue_id: Some(row.get(0)?),
                analysis_run_id: row.get(1)?,
                anti_pattern_type_id: row.get(2)?,
                file_path: row.get(3)?,
                start_line: row.get(4)?,
                end_line: row.get(5)?,
                line_number: row.get(6)?,
                column_number: row.get(7)?,
                message: row.get(8)?,
                metadata: row.get(9)?,
                detector_name: row.get(10)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            11,
                            "created_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                severity: row.get(12)?,
                description: row.get(13)?,
                code_snippet: row.get(14)?,
                ai_explanation: row.get(15)?,
            })
        })?;

        let mut issues = Vec::new();
        for issue in issue_iter {
            issues.push(issue?);
        }

        Ok(issues)
    }

    /// Store dependencies in batch for performance (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern for dependency storage instead.
    #[deprecated(
        since = "0.9.0",
        note = "Use repository pattern for dependency storage"
    )]
    pub fn store_dependencies_batch(
        &mut self,
        run_id: i64,
        dependencies: &[crate::database::models::Dependency],
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO dependencies (analysis_run_id, from_file, to_module, dependency_type, line_number) VALUES (?, ?, ?, ?, ?)"
            )?;

            for dep in dependencies {
                stmt.execute(rusqlite::params![
                    run_id,
                    dep.from_file.to_string_lossy(),
                    dep.to_module,
                    format!("{:?}", dep.dependency_type),
                    dep.line_number.map(|l| l as i32),
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Get dependencies for a specific analysis run (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern for dependency retrieval instead.
    #[deprecated(
        since = "0.9.0",
        note = "Use repository pattern for dependency retrieval"
    )]
    pub async fn get_dependencies_for_run(
        &self,
        run_id: i64,
    ) -> Result<Vec<crate::database::models::Dependency>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;
        let mut stmt = conn.prepare(
            "SELECT from_file, to_module, dependency_type, line_number 
             FROM dependencies WHERE analysis_run_id = ? ORDER BY from_file, to_module",
        )?;

        let dep_iter = stmt.query_map([run_id], |row| {
            let from_file: String = row.get(0)?;
            let to_module: String = row.get(1)?;
            let dep_type_str: String = row.get(2)?;
            let line_number: Option<i32> = row.get(3)?;

            // Parse dependency type
            use crate::database::models::DependencyType;
            let dependency_type = match dep_type_str.as_str() {
                "Use" => DependencyType::Use,
                "Mod" => DependencyType::Mod,
                "External" => DependencyType::External,
                "Import" => DependencyType::Import,
                "DataFlow" => DependencyType::DataFlow,
                "ControlFlow" => DependencyType::ControlFlow,
                _ => DependencyType::Use, // Default fallback
            };

            Ok(crate::database::models::Dependency {
                from_file: std::path::PathBuf::from(from_file),
                to_module,
                dependency_type,
                line_number: line_number.map(|l| l as u32),
            })
        })?;

        let mut dependencies = Vec::new();
        for dep in dep_iter {
            dependencies.push(dep?);
        }

        Ok(dependencies)
    }

    /// Get security issues for a specific analysis run (if security feature enabled) (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern for security issue retrieval instead.
    #[cfg(feature = "security")]
    #[deprecated(
        since = "0.9.0",
        note = "Use repository pattern for security issue retrieval"
    )]
    pub async fn get_security_issues_for_run(
        &self,
        _run_id: i64,
    ) -> Result<Vec<crate::analysis::detectors::security::types::SecurityIssue>> {
        // TODO: Implement security issue storage and retrieval
        // For now, return empty vec as security issues aren't stored in current schema
        Ok(vec![])
    }

    /// Get security issues for a specific analysis run (stub when security feature disabled) (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern instead.
    #[cfg(not(feature = "security"))]
    #[deprecated(since = "0.9.0", note = "Use repository pattern instead")]
    pub async fn get_security_issues_for_run(&self, _run_id: i64) -> Result<Vec<()>> {
        Ok(vec![])
    }

    /// Batch get issues with their anti-pattern types (prevents N+1 queries) (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern for issue and type retrieval instead.
    #[deprecated(
        since = "0.9.0",
        note = "Use repository pattern for issue and type retrieval"
    )]
    pub async fn get_issues_with_types_for_run(
        &self,
        run_id: i64,
    ) -> Result<Vec<(ArchitecturalIssue, AntiPatternType)>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;

        // Use a JOIN to avoid N+1 queries
        let mut stmt = conn.prepare(
            "SELECT ai.issue_id, ai.analysis_run_id, ai.anti_pattern_type_id, ai.file_path, 
                    ai.start_line, ai.end_line, ai.line_number, ai.column_number, ai.message, 
                    ai.metadata, ai.detector_name, ai.created_at, ai.severity, ai.description, 
                    ai.code_snippet, ai.ai_explanation,
                    apt.name, apt.description, apt.category
             FROM architectural_issues ai
             INNER JOIN anti_pattern_types apt ON ai.anti_pattern_type_id = apt.anti_pattern_type_id
             WHERE ai.analysis_run_id = ?
             ORDER BY ai.severity DESC, ai.file_path, ai.start_line",
        )?;

        let result_iter = stmt.query_map([run_id], |row| {
            let created_at_str: String = row.get(11)?;

            let issue = ArchitecturalIssue {
                issue_id: Some(row.get(0)?),
                analysis_run_id: row.get(1)?,
                anti_pattern_type_id: row.get(2)?,
                file_path: row.get(3)?,
                start_line: row.get(4)?,
                end_line: row.get(5)?,
                line_number: row.get(6)?,
                column_number: row.get(7)?,
                message: row.get(8)?,
                metadata: row.get(9)?,
                detector_name: row.get(10)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            11,
                            "created_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                severity: row.get(12)?,
                description: row.get(13)?,
                code_snippet: row.get(14)?,
                ai_explanation: row.get(15)?,
            };

            let anti_pattern_type = AntiPatternType {
                anti_pattern_type_id: Some(row.get(2)?),
                name: row.get(16)?,
                description: row.get(17)?,
                category: row.get(18)?,
            };

            Ok((issue, anti_pattern_type))
        })?;

        let mut results = Vec::new();
        for result in result_iter {
            results.push(result?);
        }

        Ok(results)
    }

    /// Get issue statistics efficiently using aggregation (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern for statistical queries instead.
    #[deprecated(
        since = "0.9.0",
        note = "Use repository pattern for statistical queries"
    )]
    pub async fn get_analysis_stats(&self, run_id: i64) -> Result<AnalysisStats> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;

        // Get overall stats
        let mut stmt = conn.prepare(
            "SELECT 
                COUNT(*) as total_issues,
                COUNT(CASE WHEN severity = 'critical' THEN 1 END) as critical_count,
                COUNT(CASE WHEN severity = 'high' THEN 1 END) as high_count,
                COUNT(CASE WHEN severity = 'medium' THEN 1 END) as medium_count,
                COUNT(CASE WHEN severity = 'low' THEN 1 END) as low_count,
                COUNT(DISTINCT file_path) as affected_files
             FROM architectural_issues WHERE analysis_run_id = ?",
        )?;

        let (total_issues, critical_count, high_count, medium_count, low_count, affected_files) =
            stmt.query_row([run_id], |row| {
                Ok((
                    row.get::<_, i64>(0)? as u32,
                    row.get::<_, i64>(1)? as u32,
                    row.get::<_, i64>(2)? as u32,
                    row.get::<_, i64>(3)? as u32,
                    row.get::<_, i64>(4)? as u32,
                    row.get::<_, i64>(5)? as u32,
                ))
            })?;

        // Get detector breakdown
        let mut stmt = conn.prepare(
            "SELECT detector_name, COUNT(*) as count 
             FROM architectural_issues WHERE analysis_run_id = ? 
             GROUP BY detector_name ORDER BY count DESC",
        )?;

        let detector_iter = stmt.query_map([run_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })?;

        let mut detector_breakdown = std::collections::HashMap::new();
        for result in detector_iter {
            let (detector, count) = result?;
            detector_breakdown.insert(detector, count);
        }

        // Get category breakdown
        let mut stmt = conn.prepare(
            "SELECT apt.category, COUNT(*) as count 
             FROM architectural_issues ai
             INNER JOIN anti_pattern_types apt ON ai.anti_pattern_type_id = apt.anti_pattern_type_id
             WHERE ai.analysis_run_id = ? 
             GROUP BY apt.category ORDER BY count DESC",
        )?;

        let category_iter = stmt.query_map([run_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })?;

        let mut category_breakdown = std::collections::HashMap::new();
        for result in category_iter {
            let (category, count) = result?;
            category_breakdown.insert(category, count);
        }

        Ok(AnalysisStats {
            total_issues,
            critical_count,
            high_count,
            medium_count,
            low_count,
            affected_files,
            detector_breakdown,
            category_breakdown,
        })
    }

    /// Get paginated issues with efficient query (DEPRECATED)
    ///
    /// # Deprecated
    ///
    /// Use the repository pattern for paginated queries instead.
    #[deprecated(since = "0.9.0", note = "Use repository pattern for paginated queries")]
    pub async fn get_issues_paginated(
        &self,
        run_id: i64,
        offset: u32,
        limit: u32,
        severity_filter: Option<&str>,
        detector_filter: Option<&str>,
    ) -> Result<Vec<ArchitecturalIssue>> {
        let conn = self.conn.lock().map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to acquire database lock: {}", e))
        })?;

        let mut query = "SELECT issue_id, analysis_run_id, anti_pattern_type_id, file_path, start_line, end_line, 
                                line_number, column_number, message, metadata, detector_name, created_at, severity, 
                                description, code_snippet, ai_explanation
                         FROM architectural_issues WHERE analysis_run_id = ?".to_string();
        let mut params = vec![run_id.to_string()];

        if let Some(severity) = severity_filter {
            query.push_str(" AND severity = ?");
            params.push(severity.to_string());
        }

        if let Some(detector) = detector_filter {
            query.push_str(" AND detector_name = ?");
            params.push(detector.to_string());
        }

        query.push_str(" ORDER BY severity DESC, file_path, start_line LIMIT ? OFFSET ?");
        params.push(limit.to_string());
        params.push(offset.to_string());

        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let issue_iter = stmt.query_map(param_refs.as_slice(), |row| {
            let created_at_str: String = row.get(11)?;

            Ok(ArchitecturalIssue {
                issue_id: Some(row.get(0)?),
                analysis_run_id: row.get(1)?,
                anti_pattern_type_id: row.get(2)?,
                file_path: row.get(3)?,
                start_line: row.get(4)?,
                end_line: row.get(5)?,
                line_number: row.get(6)?,
                column_number: row.get(7)?,
                message: row.get(8)?,
                metadata: row.get(9)?,
                detector_name: row.get(10)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            11,
                            "created_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                severity: row.get(12)?,
                description: row.get(13)?,
                code_snippet: row.get(14)?,
                ai_explanation: row.get(15)?,
            })
        })?;

        let mut issues = Vec::new();
        for issue in issue_iter {
            issues.push(issue?);
        }

        Ok(issues)
    }
}
