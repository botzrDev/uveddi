//! SQLite Database Provider Implementation
//!
//! This module provides the SQLite implementation of the database provider trait,
//! with connection pooling, WAL mode optimization, and performance enhancements.

use super::traits::{DatabaseProvider, TransactionProvider, QueryResult, QueryRow, QueryValue, PoolStats};
use super::{DatabaseConfig, DatabaseMetrics, DatabaseHealthStatus};
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue, AnalysisStats, Dependency, DependencyType};
use crate::error::{Result, UveddiError};
use crate::security;
use async_trait::async_trait;
use chrono::Utc;
use rusqlite::{Connection, Transaction, Row, params};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// SQLite provider with advanced connection pooling and performance optimizations
pub struct SqliteProvider {
    config: DatabaseConfig,
    write_pool: Arc<SqliteConnectionPool>,
    read_pool: Arc<SqliteConnectionPool>,
    metrics: Arc<DatabaseMetrics>,
}

impl SqliteProvider {
    /// Create a new SQLite provider with the given configuration
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        let write_pool = Arc::new(SqliteConnectionPool::new(&config.connection_string, &config)?);
        
        // For SQLite, read pool can be the same as write pool or separate read-only instances
        let read_pool = if config.read_connection_strings.is_empty() {
            write_pool.clone()
        } else {
            // Use first read connection string for SQLite
            let read_config = config.connection_string.clone();
            Arc::new(SqliteConnectionPool::new(&read_config, &config)?)
        };
        
        let metrics = Arc::new(DatabaseMetrics::default());
        
        Ok(Self {
            config,
            write_pool,
            read_pool,
            metrics,
        })
    }
    
    /// Internal helper to execute query with timing and error tracking
    async fn execute_with_metrics<T, F>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        match &result {
            Ok(_) => self.metrics.record_query(duration, true),
            Err(_) => self.metrics.record_query(duration, false),
        }
        
        result
    }
}

#[async_trait]
impl DatabaseProvider for SqliteProvider {
    async fn initialize(&self) -> Result<()> {
        let pool = self.write_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        self.execute_with_metrics(|| {
            // Enable SQLite optimizations
            conn.execute_batch("
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA cache_size = -64000;  -- 64MB cache
                PRAGMA temp_store = MEMORY;
                PRAGMA mmap_size = 268435456;  -- 256MB mmap
                PRAGMA foreign_keys = ON;
                PRAGMA optimize;
            ")?;
            
            // Create tables
            conn.execute_batch("
                CREATE TABLE IF NOT EXISTS projects (
                    project_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL UNIQUE,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS analysis_runs (
                    run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    start_time TEXT NOT NULL,
                    end_time TEXT,
                    status TEXT NOT NULL,
                    total_files_analyzed INTEGER,
                    total_issues_found INTEGER,
                    analysis_config TEXT NOT NULL DEFAULT '{}',
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (project_id) REFERENCES projects(project_id)
                );
                
                CREATE TABLE IF NOT EXISTS anti_pattern_types (
                    anti_pattern_type_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT NOT NULL,
                    category TEXT NOT NULL,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP
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
                
                CREATE TABLE IF NOT EXISTS dependencies (
                    dependency_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    analysis_run_id INTEGER NOT NULL,
                    from_file TEXT NOT NULL,
                    to_module TEXT NOT NULL,
                    dependency_type TEXT NOT NULL,
                    line_number INTEGER,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
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
                CREATE INDEX IF NOT EXISTS idx_dependencies_composite ON dependencies(analysis_run_id, from_file, to_module);
            ")?;
            
            Ok(())
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(())
    }
    
    async fn test_connection(&self) -> Result<()> {
        let pool = self.read_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        self.execute_with_metrics(|| {
            conn.execute("SELECT 1", params![])?;
            Ok(())
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(())
    }
    
    async fn execute_query(&self, query: &str, params: &[&str]) -> Result<Vec<QueryResult>> {
        // Not implemented for this specific use case, but would convert rusqlite::Rows to QueryResult
        todo!("Generic query execution not implemented - use specific methods instead")
    }
    
    async fn execute_write(&self, query: &str, params: &[&str]) -> Result<u64> {
        // Not implemented for this specific use case, but would execute write operations
        todo!("Generic write execution not implemented - use specific methods instead")
    }
    
    async fn begin_transaction(&self) -> Result<Box<dyn TransactionProvider>> {
        todo!("Transaction implementation")
    }
    
    async fn get_or_create_project_id(&self, project_path: &Path) -> Result<i64> {
        let path_str = project_path.to_string_lossy().to_string();
        let pool = self.write_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        let result = self.execute_with_metrics(|| {
            // First try to find existing project
            let project_id: Option<i64> = {
                let mut stmt = conn.prepare_cached("SELECT project_id FROM projects WHERE path = ?")?;
                let mut rows = stmt.query([&path_str])?;
                if let Some(row) = rows.next()? {
                    Some(row.get(0)?)
                } else {
                    None
                }
            };
            
            // If not found, insert new project
            if let Some(id) = project_id {
                Ok(id)
            } else {
                conn.execute("INSERT INTO projects (path) VALUES (?)", [&path_str])?;
                Ok(conn.last_insert_rowid())
            }
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(result)
    }
    
    async fn create_analysis_run(&self, project_path: &Path) -> Result<AnalysisRun> {
        let project_id = self.get_or_create_project_id(project_path).await?;
        let pool = self.write_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        let analysis_run = AnalysisRun {
            run_id: None,
            project_id,
            start_time: Utc::now(),
            end_time: None,
            status: "running".to_string(),
            total_files_analyzed: None,
            total_issues_found: None,
            analysis_config: "{}".to_string(),
        };
        
        let run_id = self.execute_with_metrics(|| {
            conn.execute(
                "INSERT INTO analysis_runs (project_id, start_time, status, analysis_config) VALUES (?, ?, ?, ?)",
                params![
                    analysis_run.project_id,
                    analysis_run.start_time.to_rfc3339(),
                    analysis_run.status,
                    analysis_run.analysis_config,
                ],
            )?;
            Ok(conn.last_insert_rowid())
        }).await?;
        
        pool.return_connection(conn).await?;
        
        Ok(AnalysisRun {
            run_id: Some(run_id),
            ..analysis_run
        })
    }
    
    async fn update_analysis_run(&self, run: &AnalysisRun) -> Result<()> {
        let pool = self.write_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        self.execute_with_metrics(|| {
            conn.execute(
                "UPDATE analysis_runs SET end_time = ?, status = ?, total_files_analyzed = ?, total_issues_found = ? WHERE run_id = ?",
                params![
                    run.end_time.map(|dt| dt.to_rfc3339()),
                    run.status,
                    run.total_files_analyzed,
                    run.total_issues_found,
                    run.run_id,
                ],
            )?;
            Ok(())
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(())
    }
    
    async fn store_anti_pattern_types_batch(&self, anti_pattern_types: &mut [AntiPatternType]) -> Result<()> {
        let pool = self.write_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        self.execute_with_metrics(|| {
            let tx = conn.transaction()?;
            {
                let mut stmt = tx.prepare_cached(
                    "INSERT OR IGNORE INTO anti_pattern_types (name, description, category) VALUES (?, ?, ?)"
                )?;
                
                for anti_pattern_type in anti_pattern_types.iter_mut() {
                    anti_pattern_type.description = security::sanitize_description(&anti_pattern_type.description);
                    
                    stmt.execute(params![
                        anti_pattern_type.name,
                        anti_pattern_type.description,
                        anti_pattern_type.category,
                    ])?;
                    
                    if anti_pattern_type.anti_pattern_type_id.is_none() {
                        let mut id_stmt = tx.prepare_cached(
                            "SELECT anti_pattern_type_id FROM anti_pattern_types WHERE name = ?",
                        )?;
                        anti_pattern_type.anti_pattern_type_id =
                            Some(id_stmt.query_row([&anti_pattern_type.name], |row| row.get(0))?);
                    }
                }
            }
            tx.commit()?;
            Ok(())
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(())
    }
    
    async fn store_issues_batch(&self, issues: &[ArchitecturalIssue]) -> Result<()> {
        let pool = self.write_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        self.execute_with_metrics(|| {
            let tx = conn.transaction()?;
            {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO architectural_issues (analysis_run_id, anti_pattern_type_id, file_path, start_line, end_line, line_number, column_number, message, metadata, detector_name, created_at, severity, description, code_snippet, ai_explanation) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )?;
                
                for issue in issues {
                    // Validate inputs
                    security::validate_code_analysis_data(&issue.description, "description", None)
                        .map_err(UveddiError::from)?;
                    security::validate_file_path_for_storage(&issue.file_path, "file_path")
                        .map_err(UveddiError::from)?;
                    security::validate_input(&issue.severity, "severity")
                        .map_err(UveddiError::from)?;
                    
                    let sanitized_description = security::sanitize_description(&issue.description);
                    let sanitized_ai_explanation = issue.ai_explanation.as_ref()
                        .map(|exp| security::sanitize_description(exp));
                    
                    stmt.execute(params![
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
                    ])?;
                }
            }
            tx.commit()?;
            Ok(())
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(())
    }
    
    async fn store_dependencies_batch(&self, run_id: i64, dependencies: &[Dependency]) -> Result<()> {
        let pool = self.write_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        self.execute_with_metrics(|| {
            let tx = conn.transaction()?;
            {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO dependencies (analysis_run_id, from_file, to_module, dependency_type, line_number) VALUES (?, ?, ?, ?, ?)"
                )?;
                
                for dep in dependencies {
                    stmt.execute(params![
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
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(())
    }
    
    // Additional implementation methods continue here...
    // For brevity, I'll implement key methods and indicate where others would follow
    
    async fn get_analysis_run(&self, run_id: i64) -> Result<Option<AnalysisRun>> {
        let pool = self.read_pool.clone();
        let mut conn = pool.get_connection().await?;
        
        let result = self.execute_with_metrics(|| {
            let mut stmt = conn.prepare_cached(
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
                        .map_err(|_| rusqlite::Error::InvalidColumnType(2, "start_time".to_string(), rusqlite::types::Type::Text))?
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
                Err(e) => Err(UveddiError::from(e)),
            }
        }).await?;
        
        pool.return_connection(conn).await?;
        Ok(result)
    }
    
    async fn get_latest_analysis_run(&self) -> Result<Option<AnalysisRun>> {
        // Implementation similar to get_analysis_run but with ORDER BY start_time DESC LIMIT 1
        todo!("Implement get_latest_analysis_run")
    }
    
    async fn get_recent_analysis_runs(&self, limit: u32) -> Result<Vec<AnalysisRun>> {
        todo!("Implement get_recent_analysis_runs")
    }
    
    async fn get_issues_for_run(&self, run_id: i64) -> Result<Vec<ArchitecturalIssue>> {
        todo!("Implement get_issues_for_run")
    }
    
    async fn get_dependencies_for_run(&self, run_id: i64) -> Result<Vec<Dependency>> {
        todo!("Implement get_dependencies_for_run")
    }
    
    async fn get_issues_with_types_for_run(&self, run_id: i64) -> Result<Vec<(ArchitecturalIssue, AntiPatternType)>> {
        todo!("Implement get_issues_with_types_for_run")
    }
    
    async fn get_analysis_stats(&self, run_id: i64) -> Result<AnalysisStats> {
        todo!("Implement get_analysis_stats")
    }
    
    async fn get_issues_paginated(&self, run_id: i64, offset: u32, limit: u32, severity_filter: Option<&str>, detector_filter: Option<&str>) -> Result<Vec<ArchitecturalIssue>> {
        todo!("Implement get_issues_paginated")
    }
    
    async fn get_all_anti_pattern_types(&self) -> Result<Vec<AntiPatternType>> {
        todo!("Implement get_all_anti_pattern_types")
    }
    
    async fn get_project_path(&self, project_id: i64) -> Result<String> {
        todo!("Implement get_project_path")
    }
    
    async fn cleanup(&self) -> Result<u64> {
        let write_cleanup = self.write_pool.cleanup_expired().await?;
        let read_cleanup = if !Arc::ptr_eq(&self.write_pool, &self.read_pool) {
            self.read_pool.cleanup_expired().await?
        } else {
            0
        };
        Ok(write_cleanup + read_cleanup)
    }
    
    async fn get_health_status(&self) -> Result<DatabaseHealthStatus> {
        let mut status = self.metrics.get_health_status();
        
        // Update pool utilization
        let write_stats = self.write_pool.get_stats().await?;
        status.pool_utilization = write_stats.active_connections as f32 / write_stats.max_connections as f32;
        
        Ok(status)
    }
}

/// SQLite connection pool implementation with advanced features
struct SqliteConnectionPool {
    db_path: String,
    config: DatabaseConfig,
    connections: Arc<Mutex<Vec<PooledSqliteConnection>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<PoolStats>>,
}

impl SqliteConnectionPool {
    fn new(db_path: &str, config: &DatabaseConfig) -> Result<Self> {
        let semaphore = Arc::new(Semaphore::new(config.max_connections as usize));
        let stats = Arc::new(Mutex::new(PoolStats {
            max_connections: config.max_connections as usize,
            available_connections: 0,
            active_connections: 0,
            pending_requests: 0,
            total_connections_created: 0,
            total_connections_closed: 0,
        }));
        
        Ok(Self {
            db_path: db_path.to_string(),
            config: config.clone(),
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore,
            stats,
        })
    }
    
    async fn get_connection(&self) -> Result<PooledSqliteConnection> {
        let _permit = self.semaphore.clone().acquire_owned().await
            .map_err(|e| UveddiError::database_error_msg(&format!("Failed to acquire connection permit: {}", e)))?;
        
        // Try to get existing connection
        if let Ok(mut connections) = self.connections.lock() {
            connections.retain(|conn| !conn.is_expired(&self.config));
            
            if let Some(conn) = connections.pop() {
                return Ok(conn);
            }
        }
        
        // Create new connection
        self.create_connection()
    }
    
    fn create_connection(&self) -> Result<PooledSqliteConnection> {
        let conn = if self.db_path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            Connection::open(&self.db_path)?
        };
        
        // Configure connection for performance
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -16000;  -- 16MB per connection
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 67108864;  -- 64MB mmap per connection
            PRAGMA foreign_keys = ON;
        ")?;
        
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_connections_created += 1;
        }
        
        Ok(PooledSqliteConnection::new(conn))
    }
    
    async fn return_connection(&self, connection: PooledSqliteConnection) -> Result<()> {
        if !connection.is_expired(&self.config) {
            if let Ok(mut connections) = self.connections.lock() {
                connections.push(connection);
            }
        } else if let Ok(mut stats) = self.stats.lock() {
            stats.total_connections_closed += 1;
        }
        Ok(())
    }
    
    async fn cleanup_expired(&self) -> Result<u64> {
        let mut removed = 0;
        if let Ok(mut connections) = self.connections.lock() {
            let initial_len = connections.len();
            connections.retain(|conn| !conn.is_expired(&self.config));
            removed = (initial_len - connections.len()) as u64;
            
            if let Ok(mut stats) = self.stats.lock() {
                stats.total_connections_closed += removed;
            }
        }
        Ok(removed)
    }
    
    async fn get_stats(&self) -> Result<PoolStats> {
        let stats = self.stats.lock()
            .map_err(|e| UveddiError::database_error_msg(&format!("Failed to get pool stats: {}", e)))?
            .clone();
        Ok(stats)
    }
}

/// Pooled SQLite connection wrapper
struct PooledSqliteConnection {
    connection: Connection,
    created_at: Instant,
    last_used: Instant,
}

impl PooledSqliteConnection {
    fn new(connection: Connection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
        }
    }
    
    fn is_expired(&self, config: &DatabaseConfig) -> bool {
        let now = Instant::now();
        
        // Check max lifetime
        if now.duration_since(self.created_at) > config.max_lifetime {
            return true;
        }
        
        // Check idle timeout
        if now.duration_since(self.last_used) > config.idle_timeout {
            return true;
        }
        
        false
    }
    
    fn update_last_used(&mut self) {
        self.last_used = Instant::now();
    }
    
    // Delegate connection methods
    fn execute(&mut self, sql: &str, params: impl rusqlite::Params) -> rusqlite::Result<usize> {
        self.update_last_used();
        self.connection.execute(sql, params)
    }
    
    fn prepare_cached(&mut self, sql: &str) -> rusqlite::Result<rusqlite::CachedStatement> {
        self.update_last_used();
        self.connection.prepare_cached(sql)
    }
    
    fn prepare(&mut self, sql: &str) -> rusqlite::Result<rusqlite::Statement> {
        self.update_last_used();
        self.connection.prepare(sql)
    }
    
    fn transaction(&mut self) -> rusqlite::Result<Transaction> {
        self.update_last_used();
        self.connection.transaction()
    }
    
    fn last_insert_rowid(&self) -> i64 {
        self.connection.last_insert_rowid()
    }
    
    fn execute_batch(&mut self, sql: &str) -> rusqlite::Result<()> {
        self.update_last_used();
        self.connection.execute_batch(sql)
    }
}