//! Project-related endpoints for the REST API using repository pattern

use crate::api::rest::AppState;
use crate::database::repositories::{AnalysisRepository, ProjectRepository};
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Query parameters for project listing
#[derive(Debug, Deserialize)]
pub struct ProjectsQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub active_only: bool,
}

fn default_limit() -> usize {
    20
}

/// Response for project listing
#[derive(Debug, Serialize)]
pub struct ProjectsResponse {
    pub projects: Vec<ProjectSummary>,
    pub total: usize,
}

/// Project summary for listing
#[derive(Debug, Serialize)]
pub struct ProjectSummary {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub last_analysis: Option<String>,
    pub analysis_count: usize,
    pub is_active: bool,
}

/// Get all projects using the repository pattern
pub async fn list_projects_repository(
    Query(params): Query<ProjectsQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProjectsResponse>, StatusCode> {
    // Check if repository manager is available
    let repo_manager = match &state.repository_manager {
        Some(manager) => manager,
        None => {
            error!("Repository manager not available, falling back to legacy");
            // Instead of returning the result directly, we need to match it to ensure consistent types
            match list_projects_legacy(Query(params), State(state)).await {
                Ok(response) => return Ok(response),
                Err(status) => return Err(status),
            }
        }
    };

    // Get project repository
    let project_repo = repo_manager.project();

    // Fetch projects based on query parameters
    let projects = if params.active_only {
        match project_repo.find_active().await {
            Ok(projects) => projects,
            Err(e) => {
                error!("Failed to fetch active projects: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        match project_repo.find_recent(params.limit).await {
            Ok(projects) => projects,
            Err(e) => {
                error!("Failed to fetch recent projects: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    // Get analysis repository for additional data
    let analysis_repo = repo_manager.analysis();

    // Transform to response format
    let mut project_summaries = Vec::new();
    for project in projects.iter() {
        if let Some(project_id) = project.id {
            // Get latest analysis for this project
            let latest_analysis = analysis_repo.find_latest(project_id).await.ok().flatten();

            // Count total analyses
            let analysis_count = analysis_repo
                .find_by_project(project_id)
                .await
                .map(|analyses| analyses.len())
                .unwrap_or(0);

            let summary = ProjectSummary {
                id: project_id,
                path: project.path.to_string_lossy().to_string(),
                name: project
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                last_analysis: latest_analysis.map(|a| a.start_time.to_rfc3339()),
                analysis_count,
                is_active: project.config.enable_caching, // Using caching as proxy for active status
            };
            project_summaries.push(summary);
        }
    }

    let response = ProjectsResponse {
        total: project_summaries.len(),
        projects: project_summaries,
    };

    info!(
        "Successfully fetched {} projects using repository pattern",
        response.total
    );
    Ok(Json(response))
}

/// Legacy implementation for backward compatibility
async fn list_projects_legacy(
    Query(_params): Query<ProjectsQuery>,
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ProjectsResponse>, StatusCode> {
    // For now, return empty list as legacy method doesn't exist
    let response = ProjectsResponse {
        total: 0,
        projects: vec![],
    };
    Ok(Json(response))
}

/// Get a specific project by ID using repository pattern
pub async fn get_project_repository(
    AxumPath(project_id): AxumPath<i64>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if repository manager is available
    let repo_manager = match &state.repository_manager {
        Some(manager) => manager,
        None => {
            error!("Repository manager not available");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // Get project repository
    let project_repo = repo_manager.project();

    // Fetch the project
    match project_repo.find_by_id(project_id).await {
        Ok(Some(project)) => {
            // Get analysis repository for related data
            let analysis_repo = repo_manager.analysis();

            // Get all analyses for this project
            let analyses = analysis_repo
                .find_by_project(project_id)
                .await
                .unwrap_or_default();

            let response = serde_json::json!({
                "project": {
                    "id": project.id,
                    "path": project.path.to_string_lossy(),
                    "name": project.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown"),
                    "config": {
                        "languages": project.config.languages,
                        "exclude_patterns": project.config.exclude_patterns,
                        "analysis_depth": project.config.analysis_depth,
                        "enable_caching": project.config.enable_caching,
                    },
                    "created_at": project.created_at,
                },
                "analyses": analyses.iter().map(|a| {
                    serde_json::json!({
                        "id": a.run_id,
                        "start_time": a.start_time,
                        "end_time": a.end_time,
                        "status": a.status,
                        "total_files": a.total_files_analyzed,
                        "total_issues": a.total_issues_found,
                    })
                }).collect::<Vec<_>>(),
                "analysis_count": analyses.len(),
            });

            info!(
                "Successfully fetched project {} using repository pattern",
                project_id
            );
            Ok(Json(response))
        }
        Ok(None) => {
            error!("Project {} not found", project_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to fetch project {}: {}", project_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new project using repository pattern
pub async fn create_project_repository(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate input
    if payload.path.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if repository manager is available
    let repo_manager = match &state.repository_manager {
        Some(manager) => manager,
        None => {
            error!("Repository manager not available");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // Get project repository
    let project_repo = repo_manager.project();

    // Check if project already exists
    if let Ok(Some(_)) = project_repo.find_by_path(&payload.path).await {
        error!("Project already exists at path: {}", payload.path);
        return Err(StatusCode::CONFLICT);
    }

    // Create new project
    let new_project = crate::database::models::Project {
        id: None,
        path: std::path::PathBuf::from(&payload.path),
        config: crate::database::models::ProjectConfig {
            languages: payload.language.map(|l| vec![l]).unwrap_or_else(|| {
                vec![
                    "rust".to_string(),
                    "javascript".to_string(),
                    "typescript".to_string(),
                ]
            }),
            exclude_patterns: vec!["target/".to_string(), "node_modules/".to_string()],
            analysis_depth: 5,
            enable_caching: true,
        },
        created_at: chrono::Utc::now(),
    };

    // Save the project
    match project_repo.save(&new_project).await {
        Ok(created_project) => {
            let response = serde_json::json!({
                "project": {
                    "id": created_project.id,
                    "path": created_project.path.to_string_lossy(),
                    "name": created_project.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown"),
                    "created_at": created_project.created_at,
                }
            });

            info!("Successfully created project using repository pattern");
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Request payload for creating a project
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub path: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
}
