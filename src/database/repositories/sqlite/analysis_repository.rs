//! SQLite implementation for AnalysisRepository with proper async/Send handling
//!
//! This implementation uses tokio::task::spawn_blocking to handle the async/Send
//! issues with rusqlite connections.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::Path;
use std::sync::Arc;

use crate::database::connection::ConnectionPool;
use crate::database::models::AnalysisRun;
use crate::database::repositories::errors::{RepositoryError, RepositoryResult};
use crate::database::repositories::traits::{AnalysisRepository, Repository};

pub struct SqliteAnalysisRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteAnalysisRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteAnalysisRepository {
    type Entity = AnalysisRun;

    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<Self::Entity>> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<Option<AnalysisRun>, RepositoryError> {

            let mut stmt = conn.prepare(
                "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                 FROM analysis_runs WHERE run_id = ?"
            )?;

            let mut rows = stmt.query([id])?;

            if let Some(row) = rows.next()? {
                let analysis_run = AnalysisRun {
                    run_id: Some(row.get(0)?),
                    project_id: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| RepositoryError::validation("start_time", format!("Invalid datetime: {}", e)))?
                        .with_timezone(&Utc),
                    end_time: {
                        let end_time_str: Option<String> = row.get(3)?;
                        match end_time_str {
                            Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                .map_err(|e| RepositoryError::validation("end_time", format!("Invalid datetime: {}", e)))?
                                .with_timezone(&Utc)),
                            None => None,
                        }
                    },
                    status: row.get(4)?,
                    total_files_analyzed: row.get(5)?,
                    total_issues_found: row.get(6)?,
                    analysis_config: row.get(7)?,
                };
                Ok(Some(analysis_run))
            } else {
                Ok(None)
            }
        })
        .await?
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Self::Entity>> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<Vec<AnalysisRun>, RepositoryError> {

            let mut stmt = conn.prepare(
                "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                 FROM analysis_runs ORDER BY start_time DESC"
            )?;

            let analysis_iter = stmt.query_map([], |row| {
                Ok(AnalysisRun {
                    run_id: Some(row.get(0)?),
                    project_id: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(2, "start_time".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    end_time: {
                        let end_time_str: Option<String> = row.get(3)?;
                        match end_time_str {
                            Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                .map_err(|e| rusqlite::Error::InvalidColumnType(3, "end_time".to_string(), rusqlite::types::Type::Text))?
                                .with_timezone(&Utc)),
                            None => None,
                        }
                    },
                    status: row.get(4)?,
                    total_files_analyzed: row.get(5)?,
                    total_issues_found: row.get(6)?,
                    analysis_config: row.get(7)?,
                })
            })?;

            let mut results = Vec::new();
            for item in analysis_iter {
                results.push(item?);
            }

            Ok(results)
        })
        .await?
    }

    async fn save(&self, entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        let conn = self.pool.get_connection().await?;
        let entity_clone = entity.clone();

        tokio::task::spawn_blocking(move || -> Result<AnalysisRun, RepositoryError> {

            let mut stmt = conn.prepare(
                "INSERT INTO analysis_runs (project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 RETURNING run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config"
            )?;

            let end_time_str = entity_clone.end_time.map(|dt| dt.to_rfc3339());

            let row = stmt.query_row((
                entity_clone.project_id,
                entity_clone.start_time.to_rfc3339(),
                end_time_str,
                &entity_clone.status,
                entity_clone.total_files_analyzed,
                entity_clone.total_issues_found,
                &entity_clone.analysis_config,
            ), |row| {
                Ok(AnalysisRun {
                    run_id: Some(row.get(0)?),
                    project_id: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(2, "start_time".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    end_time: {
                        let end_time_str: Option<String> = row.get(3)?;
                        match end_time_str {
                            Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                .map_err(|e| rusqlite::Error::InvalidColumnType(3, "end_time".to_string(), rusqlite::types::Type::Text))?
                                .with_timezone(&Utc)),
                            None => None,
                        }
                    },
                    status: row.get(4)?,
                    total_files_analyzed: row.get(5)?,
                    total_issues_found: row.get(6)?,
                    analysis_config: row.get(7)?,
                })
            })?;

            Ok(row)
        })
        .await?
    }

    async fn update(&self, entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        let conn = self.pool.get_connection().await?;
        let entity_clone = entity.clone();

        tokio::task::spawn_blocking(move || -> Result<AnalysisRun, RepositoryError> {
            let run_id = entity_clone.run_id.ok_or_else(||
                RepositoryError::validation("run_id", "Cannot update analysis run without ID")
            )?;

            let end_time_str = entity_clone.end_time.map(|dt| dt.to_rfc3339());

            conn.execute(
                "UPDATE analysis_runs SET project_id = ?, start_time = ?, end_time = ?, status = ?, total_files_analyzed = ?, total_issues_found = ?, analysis_config = ?
                 WHERE run_id = ?",
                (
                    entity_clone.project_id,
                    entity_clone.start_time.to_rfc3339(),
                    end_time_str,
                    &entity_clone.status,
                    entity_clone.total_files_analyzed,
                    entity_clone.total_issues_found,
                    &entity_clone.analysis_config,
                    run_id,
                ),
            )?;

            Ok(entity_clone)
        })
        .await?
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<bool, RepositoryError> {
            let affected = conn.execute("DELETE FROM analysis_runs WHERE run_id = ?", [id])?;
            Ok(affected > 0)
        })
        .await?
    }

    async fn count(&self) -> RepositoryResult<usize> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<usize, RepositoryError> {
            let count: i64 =
                conn.query_row("SELECT COUNT(*) FROM analysis_runs", [], |row| row.get(0))?;
            Ok(count as usize)
        })
        .await?
    }
}

#[async_trait]
impl AnalysisRepository for SqliteAnalysisRepository {
    async fn find_by_project(&self, project_id: i64) -> RepositoryResult<Vec<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<Vec<AnalysisRun>, RepositoryError> {

            let mut stmt = conn.prepare(
                "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                 FROM analysis_runs WHERE project_id = ? ORDER BY start_time DESC"
            )?;

            let analysis_iter = stmt.query_map([project_id], |row| {
                Ok(AnalysisRun {
                    run_id: Some(row.get(0)?),
                    project_id: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(2, "start_time".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    end_time: {
                        let end_time_str: Option<String> = row.get(3)?;
                        match end_time_str {
                            Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                .map_err(|e| rusqlite::Error::InvalidColumnType(3, "end_time".to_string(), rusqlite::types::Type::Text))?
                                .with_timezone(&Utc)),
                            None => None,
                        }
                    },
                    status: row.get(4)?,
                    total_files_analyzed: row.get(5)?,
                    total_issues_found: row.get(6)?,
                    analysis_config: row.get(7)?,
                })
            })?;

            let mut results = Vec::new();
            for item in analysis_iter {
                results.push(item?);
            }

            Ok(results)
        })
        .await?
    }

    async fn find_latest(&self, project_id: i64) -> RepositoryResult<Option<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<Option<AnalysisRun>, RepositoryError> {

            let mut stmt = conn.prepare(
                "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                 FROM analysis_runs WHERE project_id = ? ORDER BY start_time DESC LIMIT 1"
            )?;

            let mut rows = stmt.query([project_id])?;

            if let Some(row) = rows.next()? {
                let analysis_run = AnalysisRun {
                    run_id: Some(row.get(0)?),
                    project_id: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| RepositoryError::validation("start_time", format!("Invalid datetime: {}", e)))?
                        .with_timezone(&Utc),
                    end_time: {
                        let end_time_str: Option<String> = row.get(3)?;
                        match end_time_str {
                            Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                .map_err(|e| RepositoryError::validation("end_time", format!("Invalid datetime: {}", e)))?
                                .with_timezone(&Utc)),
                            None => None,
                        }
                    },
                    status: row.get(4)?,
                    total_files_analyzed: row.get(5)?,
                    total_issues_found: row.get(6)?,
                    analysis_config: row.get(7)?,
                };
                Ok(Some(analysis_run))
            } else {
                Ok(None)
            }
        })
        .await?
    }

    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<Vec<AnalysisRun>, RepositoryError> {

            let mut stmt = conn.prepare(
                "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                 FROM analysis_runs ORDER BY start_time DESC LIMIT ?"
            )?;

            let analysis_iter = stmt.query_map([limit], |row| {
                Ok(AnalysisRun {
                    run_id: Some(row.get(0)?),
                    project_id: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(2, "start_time".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    end_time: {
                        let end_time_str: Option<String> = row.get(3)?;
                        match end_time_str {
                            Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                .map_err(|e| rusqlite::Error::InvalidColumnType(3, "end_time".to_string(), rusqlite::types::Type::Text))?
                                .with_timezone(&Utc)),
                            None => None,
                        }
                    },
                    status: row.get(4)?,
                    total_files_analyzed: row.get(5)?,
                    total_issues_found: row.get(6)?,
                    analysis_config: row.get(7)?,
                })
            })?;

            let mut results = Vec::new();
            for item in analysis_iter {
                results.push(item?);
            }

            Ok(results)
        })
        .await?
    }

    async fn find_by_date_range(
        &self,
        project_id: Option<i64>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<Vec<AnalysisRun>, RepositoryError> {

            let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(pid) = project_id {
                (
                    "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                     FROM analysis_runs WHERE project_id = ? AND start_time >= ? AND start_time <= ? ORDER BY start_time DESC".to_string(),
                    vec![
                        Box::new(pid),
                        Box::new(start.to_rfc3339()),
                        Box::new(end.to_rfc3339()),
                    ]
                )
            } else {
                (
                    "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                     FROM analysis_runs WHERE start_time >= ? AND start_time <= ? ORDER BY start_time DESC".to_string(),
                    vec![
                        Box::new(start.to_rfc3339()),
                        Box::new(end.to_rfc3339()),
                    ]
                )
            };

            let mut stmt = conn.prepare(&sql)?;

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let analysis_iter = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(AnalysisRun {
                    run_id: Some(row.get(0)?),
                    project_id: row.get(1)?,
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(2, "start_time".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    end_time: {
                        let end_time_str: Option<String> = row.get(3)?;
                        match end_time_str {
                            Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                .map_err(|e| rusqlite::Error::InvalidColumnType(3, "end_time".to_string(), rusqlite::types::Type::Text))?
                                .with_timezone(&Utc)),
                            None => None,
                        }
                    },
                    status: row.get(4)?,
                    total_files_analyzed: row.get(5)?,
                    total_issues_found: row.get(6)?,
                    analysis_config: row.get(7)?,
                })
            })?;

            let mut results = Vec::new();
            for item in analysis_iter {
                results.push(item?);
            }

            Ok(results)
        })
        .await?
    }

    async fn update_status(&self, id: i64, status: String) -> RepositoryResult<()> {
        let conn = self.pool.get_connection().await?;

        tokio::task::spawn_blocking(move || -> Result<(), RepositoryError> {
            let affected = conn.execute(
                "UPDATE analysis_runs SET status = ? WHERE run_id = ?",
                (&status, id),
            )?;

            if affected == 0 {
                return Err(RepositoryError::not_found("AnalysisRun", id.to_string()));
            }

            Ok(())
        })
        .await?
    }

    async fn create_for_path(&self, project_path: &Path) -> RepositoryResult<AnalysisRun> {
        // This is a simplified implementation - in real usage, you'd need to look up the project
        // For now, just create a new analysis run with default values
        let new_run = AnalysisRun {
            run_id: None,
            project_id: 1, // Default project ID
            start_time: Utc::now(),
            end_time: None,
            status: "pending".to_string(),
            total_files_analyzed: Some(0),
            total_issues_found: Some(0),
            analysis_config: "{}".to_string(),
        };

        self.save(&new_run).await
    }

    async fn create_analysis_run(&self, project_id: i64) -> RepositoryResult<AnalysisRun> {
        let new_run = AnalysisRun {
            run_id: None,
            project_id,
            start_time: Utc::now(),
            end_time: None,
            status: "pending".to_string(),
            total_files_analyzed: Some(0),
            total_issues_found: Some(0),
            analysis_config: "{}".to_string(),
        };

        self.save(&new_run).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::{DatabaseConfig, DatabaseType, PoolConfig};
    use std::time::Duration;
    use tempfile::NamedTempFile;

    async fn create_test_repository() -> (SqliteAnalysisRepository, NamedTempFile) {
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
                test_on_checkout: true,
                pool_timeout: Duration::from_secs(30),
            },
            enable_metrics: false,
            enable_logging: false,
            enable_prepared_statements: false,
        };

        let db_config = crate::database::connection::config::DatabaseConfig::sqlite(":memory:");
        let provider = Arc::new(crate::database::SqliteProvider::new(db_config)?);
        let pool = ConnectionPool::new(config, provider).await.unwrap();

        // Create the table
        {
            let conn = pool.get_connection().await.unwrap();
            conn.execute(
                "CREATE TABLE analysis_runs (
                    run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    start_time TEXT NOT NULL,
                    end_time TEXT,
                    status TEXT NOT NULL,
                    total_files_analyzed INTEGER DEFAULT 0,
                    total_issues_found INTEGER DEFAULT 0,
                    analysis_config TEXT DEFAULT '{}'
                )",
                [],
            )
            .unwrap();
        }

        (SqliteAnalysisRepository::new(pool), temp_file)
    }

    #[tokio::test]
    async fn test_save_and_find_analysis_run() {
        let (repo, _temp_file) = create_test_repository().await;

        let analysis_run = AnalysisRun {
            run_id: None,
            project_id: 1,
            start_time: Utc::now(),
            end_time: None,
            status: "pending".to_string(),
            total_files_analyzed: Some(0),
            total_issues_found: Some(0),
            analysis_config: "{}".to_string(),
        };

        let saved = repo.save(&analysis_run).await.unwrap();
        assert!(saved.run_id.is_some());

        let found = repo.find_by_id(saved.run_id.unwrap()).await.unwrap();
        assert!(found.is_some());

        let found_run = found.unwrap();
        assert_eq!(found_run.project_id, analysis_run.project_id);
        assert_eq!(found_run.status, analysis_run.status);
    }

    #[tokio::test]
    async fn test_find_by_project() {
        let (repo, _temp_file) = create_test_repository().await;

        let analysis_run = AnalysisRun {
            run_id: None,
            project_id: 42,
            start_time: Utc::now(),
            end_time: None,
            status: "completed".to_string(),
            total_files_analyzed: Some(100),
            total_issues_found: Some(5),
            analysis_config: "{}".to_string(),
        };

        repo.save(&analysis_run).await.unwrap();

        let runs = repo.find_by_project(42).await.unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].project_id, 42);
    }
}
