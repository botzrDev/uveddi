//! SQLite implementation for ProjectRepository

use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;

use crate::database::connection::pool::ConnectionPool;
use crate::database::models::{Project, ProjectConfig};
use crate::database::repositories::traits::{ProjectRepository, Repository};
use crate::error::{Result, UveddiError};

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

    async fn find_by_id(&self, id: i64) -> Result<Option<Self::Entity>> {
        let conn = self.pool.get_connection().await?;

        let mut stmt =
            conn.prepare("SELECT project_id, path FROM projects WHERE project_id = ?")?;
        let mut rows = stmt.query([id])?;

        if let Some(row) = rows.next()? {
            let project = Project {
                id: Some(row.get(0)?),
                path: PathBuf::from(row.get::<_, String>(1)?),
                config: ProjectConfig::default(),
                created_at: Utc::now(), // Default to current time since not stored in DB
            };

            self.pool.return_connection(conn).await?;
            Ok(Some(project))
        } else {
            self.pool.return_connection(conn).await?;
            Ok(None)
        }
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>> {
        let conn = self.pool.get_connection().await?;

        let mut stmt =
            conn.prepare("SELECT project_id, path FROM projects ORDER BY project_id DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: Some(row.get(0)?),
                path: PathBuf::from(row.get::<_, String>(1)?),
                config: ProjectConfig::default(),
                created_at: Utc::now(), // Default to current time since not stored in DB
            })
        })?;

        let mut projects = Vec::new();
        for project in rows {
            projects.push(project?);
        }

        self.pool.return_connection(conn).await?;
        Ok(projects)
    }

    async fn save(&self, entity: &Self::Entity) -> Result<Self::Entity> {
        let path_str = entity.path.to_string_lossy().to_string();
        let conn = self.pool.get_connection().await?;

        conn.execute("INSERT INTO projects (path) VALUES (?)", [&path_str])?;

        let id = conn.last_insert_rowid();

        let saved_project = Project {
            id: Some(id),
            path: entity.path.clone(),
            config: entity.config.clone(),
            created_at: entity.created_at,
        };

        self.pool.return_connection(conn).await?;
        Ok(saved_project)
    }

    async fn update(&self, entity: &Self::Entity) -> Result<Self::Entity> {
        let id = entity
            .id
            .ok_or_else(|| UveddiError::database_error_msg("Cannot update project without ID"))?;

        let path_str = entity.path.to_string_lossy().to_string();
        let conn = self.pool.get_connection().await?;

        let rows_affected = conn.execute(
            "UPDATE projects SET path = ? WHERE project_id = ?",
            [&path_str, &id.to_string()],
        )?;

        if rows_affected == 0 {
            self.pool.return_connection(conn).await?;
            return Err(UveddiError::database_error_msg(
                "Project not found for update",
            ));
        }

        let updated_project = Project {
            id: Some(id),
            path: entity.path.clone(),
            config: entity.config.clone(),
            created_at: entity.created_at,
        };

        self.pool.return_connection(conn).await?;
        Ok(updated_project)
    }

    async fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.pool.get_connection().await?;

        let rows_affected = conn.execute("DELETE FROM projects WHERE project_id = ?", [id])?;

        let deleted = rows_affected > 0;
        self.pool.return_connection(conn).await?;
        Ok(deleted)
    }

    async fn count(&self) -> Result<usize> {
        let conn = self.pool.get_connection().await?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM projects")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;

        self.pool.return_connection(conn).await?;
        Ok(count as usize)
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn find_by_path(&self, path: &str) -> Result<Option<Project>> {
        let conn = self.pool.get_connection().await?;

        let mut stmt = conn.prepare("SELECT project_id, path FROM projects WHERE path = ?")?;
        let mut rows = stmt.query([path])?;

        if let Some(row) = rows.next()? {
            let project = Project {
                id: Some(row.get(0)?),
                path: PathBuf::from(row.get::<_, String>(1)?),
                config: ProjectConfig::default(),
                created_at: Utc::now(), // Default to current time since not stored in DB
            };

            self.pool.return_connection(conn).await?;
            Ok(Some(project))
        } else {
            self.pool.return_connection(conn).await?;
            Ok(None)
        }
    }

    async fn find_active(&self) -> Result<Vec<Project>> {
        // For SQLite, all projects are considered "active" for now
        // This could be enhanced with an "active" field in the future
        self.find_all().await
    }

    async fn find_recent(&self, limit: usize) -> Result<Vec<Project>> {
        let conn = self.pool.get_connection().await?;

        let mut stmt =
            conn.prepare("SELECT project_id, path FROM projects ORDER BY project_id DESC LIMIT ?")?;
        let rows = stmt.query_map([limit], |row| {
            Ok(Project {
                id: Some(row.get(0)?),
                path: PathBuf::from(row.get::<_, String>(1)?),
                config: ProjectConfig::default(),
                created_at: Utc::now(), // Default to current time since not stored in DB
            })
        })?;

        let mut projects = Vec::new();
        for project in rows {
            projects.push(project?);
        }

        self.pool.return_connection(conn).await?;
        Ok(projects)
    }

    async fn update_last_accessed(&self, id: i64) -> Result<()> {
        // For now, this is a no-op since we don't track last_accessed in the current schema
        // This would require a schema migration to add a last_accessed column
        let _ = id; // Suppress unused parameter warning
        Ok(())
    }
}

impl SqliteProjectRepository {
    /// Get or create project ID (backward compatibility method)
    /// This preserves the exact behavior from crud.rs
    pub async fn get_or_create_project_id(&self, project_path: &std::path::Path) -> Result<i64> {
        let path_str = project_path.to_string_lossy().to_string();

        // First try to find existing project
        if let Some(project) = self.find_by_path(&path_str).await? {
            if let Some(id) = project.id {
                return Ok(id);
            }
        }

        // Create new project if not found
        let conn = self.pool.get_connection().await?;
        conn.execute("INSERT INTO projects (path) VALUES (?)", [&path_str])?;
        let id = conn.last_insert_rowid();
        self.pool.return_connection(conn).await?;

        Ok(id)
    }
}
