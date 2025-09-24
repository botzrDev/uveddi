//! SQLite implementation for AnalysisRepository

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::Path;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::models::AnalysisRun;
use crate::database::repositories::traits::{Repository, AnalysisRepository};
use crate::database::repositories::sqlite::project_repository::SqliteProjectRepository;
use crate::database::repositories::errors::{RepositoryError, RepositoryResult};

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
                    .map_err(|e| RepositoryError::Runtime(&format!("Invalid start_time: {}", e)))?
                    .with_timezone(&Utc),
                end_time: {
                    let end_time_str: Option<String> = row.get(3)?;
                    match end_time_str {
                        Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                            .map_err(|e| RepositoryError::Runtime(&format!("Invalid end_time: {}", e)))?
                            .with_timezone(&Utc)),
                        None => None,
                    }
                },
                status: row.get(4)?,
                total_files_analyzed: row.get(5)?,
                total_issues_found: row.get(6)?,
                analysis_config: row.get(7)?,
            };

            self.pool.return_connection(conn).await?;
            Ok(Some(analysis_run))
        } else {
            self.pool.return_connection(conn).await?;
            Ok(None)
        }
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Self::Entity>> {
        let conn = self.pool.get_connection().await?;

        let mut stmt = conn.prepare(
            "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
             FROM analysis_runs ORDER BY start_time DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AnalysisRun {
                run_id: Some(row.get(0)?),
                project_id: row.get(1)?,
                start_time: {
                    let start_time_str: String = row.get(2)?;
                    DateTime::parse_from_rfc3339(&start_time_str)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(
                            2,
                            "start_time".to_string(),
                            rusqlite::types::Type::Text
                        ))?
                        .with_timezone(&Utc)
                },
                end_time: {
                    let end_time_str: Option<String> = row.get(3)?;
                    match end_time_str {
                        Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                            .map_err(|e| rusqlite::Error::InvalidColumnType(
                                3,
                                "end_time".to_string(),
                                rusqlite::types::Type::Text
                            ))?
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

        let mut analysis_runs = Vec::new();
        for run in rows {
            analysis_runs.push(run?);
        }

        self.pool.return_connection(conn).await?;
        Ok(analysis_runs)
    }

    async fn save(&self, entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        let conn = self.pool.get_connection().await?;

        let end_time_str = entity.end_time.map(|dt| dt.to_rfc3339());

        conn.execute(
            "INSERT INTO analysis_runs (project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                entity.project_id,
                entity.start_time.to_rfc3339(),
                end_time_str,
                entity.status,
                entity.total_files_analyzed,
                entity.total_issues_found,
                entity.analysis_config,
            ],
        )?;

        let id = conn.last_insert_rowid();

        let saved_run = AnalysisRun {
            run_id: Some(id),
            project_id: entity.project_id,
            start_time: entity.start_time,
            end_time: entity.end_time,
            status: entity.status.clone(),
            total_files_analyzed: entity.total_files_analyzed,
            total_issues_found: entity.total_issues_found,
            analysis_config: entity.analysis_config.clone(),
        };

        self.pool.return_connection(conn).await?;
        Ok(saved_run)
    }

    async fn update(&self, entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        let id = entity.run_id.ok_or_else(|| {
            RepositoryError::Runtime("Cannot update analysis run without ID")
        })?;

        let conn = self.pool.get_connection().await?;
        let end_time_str = entity.end_time.map(|dt| dt.to_rfc3339());

        let rows_affected = conn.execute(
            "UPDATE analysis_runs SET project_id = ?, start_time = ?, end_time = ?, status = ?,
             total_files_analyzed = ?, total_issues_found = ?, analysis_config = ? WHERE run_id = ?",
            rusqlite::params![
                entity.project_id,
                entity.start_time.to_rfc3339(),
                end_time_str,
                entity.status,
                entity.total_files_analyzed,
                entity.total_issues_found,
                entity.analysis_config,
                id,
            ],
        )?;

        if rows_affected == 0 {
            self.pool.return_connection(conn).await?;
            return Err(RepositoryError::Runtime("Analysis run not found for update"));
        }

        let updated_run = entity.clone();
        self.pool.return_connection(conn).await?;
        Ok(updated_run)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let conn = self.pool.get_connection().await?;

        let rows_affected = conn.execute(
            "DELETE FROM analysis_runs WHERE run_id = ?",
            [id],
        )?;

        let deleted = rows_affected > 0;
        self.pool.return_connection(conn).await?;
        Ok(deleted)
    }

    async fn count(&self) -> RepositoryResult<usize> {
        let conn = self.pool.get_connection().await?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM analysis_runs")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;

        self.pool.return_connection(conn).await?;
        Ok(count as usize)
    }
}

#[async_trait]
impl AnalysisRepository for SqliteAnalysisRepository {
    async fn find_by_project(&self, project_id: i64) -> RepositoryResult<Vec<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

        let mut stmt = conn.prepare(
            "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
             FROM analysis_runs WHERE project_id = ? ORDER BY start_time DESC"
        )?;
        let rows = stmt.query_map([project_id], |row| {
            Ok(AnalysisRun {
                run_id: Some(row.get(0)?),
                project_id: row.get(1)?,
                start_time: {
                    let start_time_str: String = row.get(2)?;
                    DateTime::parse_from_rfc3339(&start_time_str)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(
                            2,
                            "start_time".to_string(),
                            rusqlite::types::Type::Text
                        ))?
                        .with_timezone(&Utc)
                },
                end_time: {
                    let end_time_str: Option<String> = row.get(3)?;
                    match end_time_str {
                        Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                            .map_err(|e| rusqlite::Error::InvalidColumnType(
                                3,
                                "end_time".to_string(),
                                rusqlite::types::Type::Text
                            ))?
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

        let mut analysis_runs = Vec::new();
        for run in rows {
            analysis_runs.push(run?);
        }

        self.pool.return_connection(conn).await?;
        Ok(analysis_runs)
    }

    async fn find_latest(&self, project_id: i64) -> RepositoryResult<Option<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

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
                    .map_err(|e| RepositoryError::Runtime(&format!("Invalid start_time: {}", e)))?
                    .with_timezone(&Utc),
                end_time: {
                    let end_time_str: Option<String> = row.get(3)?;
                    match end_time_str {
                        Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                            .map_err(|e| RepositoryError::Runtime(&format!("Invalid end_time: {}", e)))?
                            .with_timezone(&Utc)),
                        None => None,
                    }
                },
                status: row.get(4)?,
                total_files_analyzed: row.get(5)?,
                total_issues_found: row.get(6)?,
                analysis_config: row.get(7)?,
            };

            self.pool.return_connection(conn).await?;
            Ok(Some(analysis_run))
        } else {
            self.pool.return_connection(conn).await?;
            Ok(None)
        }
    }

    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

        let mut stmt = conn.prepare(
            "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
             FROM analysis_runs ORDER BY start_time DESC LIMIT ?"
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(AnalysisRun {
                run_id: Some(row.get(0)?),
                project_id: row.get(1)?,
                start_time: {
                    let start_time_str: String = row.get(2)?;
                    DateTime::parse_from_rfc3339(&start_time_str)
                        .map_err(|e| rusqlite::Error::InvalidColumnType(
                            2,
                            "start_time".to_string(),
                            rusqlite::types::Type::Text
                        ))?
                        .with_timezone(&Utc)
                },
                end_time: {
                    let end_time_str: Option<String> = row.get(3)?;
                    match end_time_str {
                        Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                            .map_err(|e| rusqlite::Error::InvalidColumnType(
                                3,
                                "end_time".to_string(),
                                rusqlite::types::Type::Text
                            ))?
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

        let mut analysis_runs = Vec::new();
        for run in rows {
            analysis_runs.push(run?);
        }

        self.pool.return_connection(conn).await?;
        Ok(analysis_runs)
    }

    async fn find_by_date_range(
        &self,
        project_id: Option<i64>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<AnalysisRun>> {
        let conn = self.pool.get_connection().await?;

        let (query, analysis_runs) = match project_id {
            Some(pid) => {
                let query = "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                             FROM analysis_runs WHERE project_id = ? AND start_time >= ? AND start_time <= ? ORDER BY start_time DESC";
                let mut stmt = conn.prepare(query)?;
                let rows = stmt.query_map([pid.to_string(), start.to_rfc3339(), end.to_rfc3339()], |row| {
                    Ok(AnalysisRun {
                        run_id: Some(row.get(0)?),
                        project_id: row.get(1)?,
                        start_time: {
                            let start_time_str: String = row.get(2)?;
                            DateTime::parse_from_rfc3339(&start_time_str)
                                .map_err(|e| rusqlite::Error::InvalidColumnType(
                                    2,
                                    "start_time".to_string(),
                                    rusqlite::types::Type::Text
                                ))?
                                .with_timezone(&Utc)
                        },
                        end_time: {
                            let end_time_str: Option<String> = row.get(3)?;
                            match end_time_str {
                                Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                    .map_err(|e| rusqlite::Error::InvalidColumnType(
                                        3,
                                        "end_time".to_string(),
                                        rusqlite::types::Type::Text
                                    ))?
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

                let mut analysis_runs = Vec::new();
                for run in rows {
                    analysis_runs.push(run?);
                }
                (query, analysis_runs)
            },
            None => {
                let query = "SELECT run_id, project_id, start_time, end_time, status, total_files_analyzed, total_issues_found, analysis_config
                             FROM analysis_runs WHERE start_time >= ? AND start_time <= ? ORDER BY start_time DESC";
                let mut stmt = conn.prepare(query)?;
                let rows = stmt.query_map([start.to_rfc3339(), end.to_rfc3339()], |row| {
                    Ok(AnalysisRun {
                        run_id: Some(row.get(0)?),
                        project_id: row.get(1)?,
                        start_time: {
                            let start_time_str: String = row.get(2)?;
                            DateTime::parse_from_rfc3339(&start_time_str)
                                .map_err(|e| rusqlite::Error::InvalidColumnType(
                                    2,
                                    "start_time".to_string(),
                                    rusqlite::types::Type::Text
                                ))?
                                .with_timezone(&Utc)
                        },
                        end_time: {
                            let end_time_str: Option<String> = row.get(3)?;
                            match end_time_str {
                                Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                                    .map_err(|e| rusqlite::Error::InvalidColumnType(
                                        3,
                                        "end_time".to_string(),
                                        rusqlite::types::Type::Text
                                    ))?
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

                let mut analysis_runs = Vec::new();
                for run in rows {
                    analysis_runs.push(run?);
                }
                (query, analysis_runs)
            },
        };

        self.pool.return_connection(conn).await?;
        Ok(analysis_runs)
    }

    async fn update_status(&self, id: i64, status: String) -> RepositoryResult<()> {
        let conn = self.pool.get_connection().await?;

        let rows_affected = conn.execute(
            "UPDATE analysis_runs SET status = ? WHERE run_id = ?",
            [&status, &id.to_string()],
        )?;

        if rows_affected == 0 {
            self.pool.return_connection(conn).await?;
            return Err(RepositoryError::Runtime("Analysis run not found for status update"));
        }

        self.pool.return_connection(conn).await?;
        Ok(())
    }

    async fn create_for_path(&self, project_path: &Path) -> RepositoryResult<AnalysisRun> {
        // Use the project repository to get or create project ID
        let project_repo = SqliteProjectRepository::new(self.pool.clone());
        let project_id = project_repo.get_or_create_project_id(project_path).await?;

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

        self.save(&analysis_run).await
    }
}