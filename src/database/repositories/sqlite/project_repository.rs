//! SQLite implementation for ProjectRepository
//!
//! This implementation uses tokio::task::spawn_blocking to handle all rusqlite
//! operations, preventing Send/Sync violations with rusqlite's Statement and Rows.

use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::models::{Project, ProjectConfig};
use crate::database::repositories::errors::{RepositoryError, RepositoryResult};
use crate::database::repositories::traits::{ProjectRepository, Repository};

pub struct SqliteProjectRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteProjectRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for SqliteProjectRepository {
    type Entity = Project;

    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<Self::Entity>> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;

        let result =
            tokio::task::spawn_blocking(move || -> Result<Option<Project>, RepositoryError> {
                let mut stmt =
                    conn.prepare("SELECT project_id, path FROM projects WHERE project_id = ?")?;
                let mut rows = stmt.query([id])?;

                if let Some(row) = rows.next()? {
                    let project = Project {
                        id: Some(row.get(0)?),
                        path: PathBuf::from(row.get::<_, String>(1)?),
                        config: ProjectConfig::default(),
                        created_at: Utc::now(),
                    };
                    Ok(Some(project))
                } else {
                    Ok(None)
                }
            })
            .await??;

        Ok(result)
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Self::Entity>> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;

        let result =
            tokio::task::spawn_blocking(move || -> Result<Vec<Project>, RepositoryError> {
                let mut stmt =
                    conn.prepare("SELECT project_id, path FROM projects ORDER BY project_id DESC")?;
                let rows = stmt.query_map([], |row| {
                    Ok(Project {
                        id: Some(row.get(0)?),
                        path: PathBuf::from(row.get::<_, String>(1)?),
                        config: ProjectConfig::default(),
                        created_at: Utc::now(),
                    })
                })?;

                let mut projects = Vec::new();
                for project in rows {
                    projects.push(project?);
                }

                Ok(projects)
            })
            .await??;

        Ok(result)
    }

    async fn save(&self, entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;
        let path_str = entity.path.to_string_lossy().to_string();
        let entity_clone = entity.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<Project, RepositoryError> {
            conn.execute("INSERT INTO projects (path) VALUES (?)", [&path_str])?;

            let id = conn.last_insert_rowid();

            let saved_project = Project {
                id: Some(id),
                path: entity_clone.path,
                config: entity_clone.config,
                created_at: entity_clone.created_at,
            };

            Ok(saved_project)
        })
        .await??;

        Ok(result)
    }

    async fn update(&self, entity: &Self::Entity) -> RepositoryResult<Self::Entity> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;
        let entity_clone = entity.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<Project, RepositoryError> {
            let id = entity_clone.id.ok_or_else(|| {
                RepositoryError::validation("id", "Cannot update project without ID".to_string())
            })?;

            let path_str = entity_clone.path.to_string_lossy().to_string();

            let rows_affected = conn.execute(
                "UPDATE projects SET path = ? WHERE project_id = ?",
                [&path_str, &id.to_string()],
            )?;

            if rows_affected == 0 {
                return Err(RepositoryError::not_found("Project", id.to_string()).into());
            }

            let updated_project = Project {
                id: Some(id),
                path: entity_clone.path,
                config: entity_clone.config,
                created_at: entity_clone.created_at,
            };

            Ok(updated_project)
        })
        .await??;

        Ok(result)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;

        let result = tokio::task::spawn_blocking(move || -> Result<bool, RepositoryError> {
            let rows_affected = conn.execute("DELETE FROM projects WHERE project_id = ?", [id])?;
            Ok(rows_affected > 0)
        })
        .await??;

        Ok(result)
    }

    async fn count(&self) -> RepositoryResult<usize> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;

        let result = tokio::task::spawn_blocking(move || -> Result<usize, RepositoryError> {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM projects")?;
            let count: i64 = stmt.query_row([], |row| row.get(0))?;
            Ok(count as usize)
        })
        .await??;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::repositories::errors::RepositoryError;

    #[test]
    fn test_repository_error_conversion() {
        let rusqlite_err = rusqlite::Error::InvalidColumnName("test_column".to_string());
        let repo_err = RepositoryError::from(rusqlite_err);

        match repo_err {
            RepositoryError::Database { .. } => {
                // Expected conversion from rusqlite error
                assert!(true);
            }
            _ => panic!("Expected Database error variant"),
        }
    }

    #[test]
    fn test_option_conversion_helpers() {
        use crate::database::repositories::sqlite::helpers::*;

        // Test Option<i32> conversion
        assert_eq!(option_i32_to_default(Some(42)), 42);
        assert_eq!(option_i32_to_default(None), 0);

        assert_eq!(i32_to_option(42), Some(42));
        assert_eq!(i32_to_option(0), None);
    }

    #[test]
    fn test_datetime_parsing_helpers() {
        use crate::database::repositories::sqlite::helpers::*;

        // Test valid RFC3339 datetime
        let valid_datetime = "2023-01-01T12:00:00Z";
        let result = parse_rfc3339_datetime(valid_datetime);
        assert!(result.is_ok());

        // Test invalid RFC3339 datetime
        let invalid_datetime = "not-a-datetime";
        let result = parse_rfc3339_datetime(invalid_datetime);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_validation());

        // Test optional parsing
        let result = parse_optional_rfc3339_datetime(Some(valid_datetime.to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = parse_optional_rfc3339_datetime(None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn find_by_path(&self, path: &str) -> RepositoryResult<Option<Project>> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;
        let path = path.to_string();

        let result =
            tokio::task::spawn_blocking(move || -> Result<Option<Project>, RepositoryError> {
                let mut stmt =
                    conn.prepare("SELECT project_id, path FROM projects WHERE path = ?")?;
                let mut rows = stmt.query([&path])?;

                if let Some(row) = rows.next()? {
                    let project = Project {
                        id: Some(row.get(0)?),
                        path: PathBuf::from(row.get::<_, String>(1)?),
                        config: ProjectConfig::default(),
                        created_at: Utc::now(),
                    };
                    Ok(Some(project))
                } else {
                    Ok(None)
                }
            })
            .await??;

        Ok(result)
    }

    async fn find_active(&self) -> RepositoryResult<Vec<Project>> {
        // For SQLite, all projects are considered "active" for now
        // This could be enhanced with an "active" field in the future
        self.find_all().await
    }

    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<Project>> {
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;

        let result =
            tokio::task::spawn_blocking(move || -> Result<Vec<Project>, RepositoryError> {
                let mut stmt = conn.prepare(
                    "SELECT project_id, path FROM projects ORDER BY project_id DESC LIMIT ?",
                )?;
                let rows = stmt.query_map([limit], |row| {
                    Ok(Project {
                        id: Some(row.get(0)?),
                        path: PathBuf::from(row.get::<_, String>(1)?),
                        config: ProjectConfig::default(),
                        created_at: Utc::now(),
                    })
                })?;

                let mut projects = Vec::new();
                for project in rows {
                    projects.push(project?);
                }

                Ok(projects)
            })
            .await??;

        Ok(result)
    }

    async fn update_last_accessed(&self, id: i64) -> RepositoryResult<()> {
        // For now, this is a no-op since we don't track last_accessed in the current schema
        // This would require a schema migration to add a last_accessed column
        let _ = id; // Suppress unused parameter warning
        Ok(())
    }

    async fn get_or_create_project_id(
        &self,
        project_path: &std::path::Path,
    ) -> RepositoryResult<i64> {
        // Delegate to the existing implementation
        self.get_or_create_project_id_impl(project_path).await
    }
}

impl SqliteProjectRepository {
    /// Get or create project ID (backward compatibility method)
    /// This preserves the exact behavior from crud.rs
    pub async fn get_or_create_project_id_impl(
        &self,
        project_path: &std::path::Path,
    ) -> RepositoryResult<i64> {
        let path_str = project_path.to_string_lossy().to_string();

        // First try to find existing project
        if let Some(project) = self.find_by_path(&path_str).await? {
            if let Some(id) = project.id {
                return Ok(id);
            }
        }

        // Create new project if not found
        let conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| RepositoryError::Pool(e.to_string()))?;
        let path_str_clone = path_str.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<i64, RepositoryError> {
            conn.execute("INSERT INTO projects (path) VALUES (?)", [&path_str_clone])?;
            let id = conn.last_insert_rowid();
            Ok(id)
        })
        .await??;

        Ok(result)
    }
}
