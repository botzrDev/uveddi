//! REST API endpoints for interactive reporting
//!
//! This module provides REST API endpoints specifically designed for the interactive
//! reporting SPA frontend. It serves versioned JSON contracts and static assets
//! for the React application while maintaining compatibility with the existing
//! GraphQL infrastructure.
//!
//! # API Design
//!
//! The REST API follows these principles:
//! - Versioned endpoints for API contract stability (/api/v1/...)
//! - JSON-first with consistent error handling
//! - Offline-first with local file system storage
//! - Security-first with CSP headers and CORS restrictions
//! - Performance-optimized with caching and compression
//!
//! # Endpoints
//!
//! - `GET /api/v1/reports/:id` - Get interactive report by ID
//! - `GET /api/v1/reports/:id/graphs/dependency` - Get dependency graph data
//! - `GET /api/v1/reports` - List available reports
//! - `GET /health` - Health check (shared with GraphQL)
//! - `GET /metrics` - Basic metrics (shared with GraphQL)
//! - `GET /app/*` - Serve SPA static assets
//! - `GET /*` - SPA fallback for client-side routing

use crate::database::Database;
use crate::report::interactive_models::{InteractiveReport, DependencyGraph, REPORT_SCHEMA_VERSION};
use crate::database::models::{AnalysisRun, ArchitecturalIssue, AntiPatternType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use warp::{Filter, Rejection, Reply};

/// Configuration for the REST API server
#[derive(Debug, Clone)]
pub struct RestApiConfig {
    /// Enable CORS for browser clients
    pub enable_cors: bool,
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
    /// Path to SPA static assets
    pub spa_assets_path: Option<PathBuf>,
    /// Path to report storage directory
    pub reports_storage_path: PathBuf,
    /// Enable serving of static assets
    pub serve_spa: bool,
    /// Enable Content Security Policy headers
    pub enable_csp: bool,
    /// Maximum report cache age in seconds
    pub cache_max_age: u32,
}

impl Default for RestApiConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            cors_origins: vec!["http://localhost:3000".to_string(), "http://127.0.0.1:3000".to_string()],
            spa_assets_path: None, // Will be set based on build artifacts
            reports_storage_path: PathBuf::from("./.uveddi/reports"),
            serve_spa: true,
            enable_csp: true,
            cache_max_age: 3600, // 1 hour
        }
    }
}

/// Report metadata for listing reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportListItem {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(rename = "generatedAt")]
    pub generated_at: DateTime<Utc>,
    #[serde(rename = "issuesTotal")]
    pub issues_total: u32,
    pub status: String,
    #[serde(rename = "filesAnalyzed")]
    pub files_analyzed: u32,
}

/// Pagination parameters for report listing
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

/// Error response structure
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Success response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
}

/// REST API service managing report endpoints
pub struct RestApiService {
    config: RestApiConfig,
    database: Arc<Database>,
}

impl RestApiService {
    pub fn new(config: RestApiConfig, database: Arc<Database>) -> Self {
        Self { config, database }
    }

    /// Create all REST API routes
    pub fn routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let api_routes = self.api_v1_routes();
        let spa_routes = self.spa_routes();
        let health_routes = self.health_routes();

        // Combine all routes with middleware
        let routes = api_routes
            .or(spa_routes)
            .or(health_routes)
            .with(self.cors_filter())
            .with(self.security_headers())
            .with(warp::log("uveddi_rest_api"))
            .recover(handle_api_rejection);

        routes
    }

    /// API v1 routes for interactive reports
    fn api_v1_routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db = self.database.clone();
        let config = self.config.clone();

        // GET /api/v1/reports
        let list_reports = warp::path!("api" / "v1" / "reports")
            .and(warp::get())
            .and(warp::query::<PaginationParams>())
            .and(with_db(db.clone()))
            .and(with_config(config.clone()))
            .and_then(list_reports_handler);

        // GET /api/v1/reports/:id
        let get_report = warp::path!("api" / "v1" / "reports" / String)
            .and(warp::get())
            .and(with_db(db.clone()))
            .and(with_config(config.clone()))
            .and_then(get_report_handler);

        // GET /api/v1/reports/:id/graphs/dependency
        let get_dependency_graph = warp::path!("api" / "v1" / "reports" / String / "graphs" / "dependency")
            .and(warp::get())
            .and(with_db(db.clone()))
            .and(with_config(config.clone()))
            .and_then(get_dependency_graph_handler);

        // GET /api/v1/reports/demo - Demo report for development
        let demo_report = warp::path!("api" / "v1" / "reports" / "demo")
            .and(warp::get())
            .and_then(demo_report_handler);

        list_reports.or(get_report).or(get_dependency_graph).or(demo_report)
    }

    /// SPA static asset routes
    fn spa_routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        if !self.config.serve_spa {
            return warp::any().and_then(|| async { Err(warp::reject::not_found()) }).boxed();
        }

        if let Some(assets_path) = &self.config.spa_assets_path {
            // Serve static assets from /app/*
            let static_files = warp::path("app")
                .and(warp::fs::dir(assets_path.clone()))
                .with(warp::reply::with::header(
                    "cache-control",
                    format!("public, max-age={}", self.config.cache_max_age),
                ));

            // SPA fallback for client-side routing
            let spa_fallback = warp::get()
                .and(warp::path::full())
                .and(warp::fs::file(assets_path.join("index.html")))
                .with(warp::reply::with::header("cache-control", "no-cache"));

            static_files.or(spa_fallback).boxed()
        } else {
            // Return simple HTML if no SPA assets are configured
            let fallback_html = r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Uveddi Interactive Reports</title>
                    <style>
                        body { font-family: Arial, sans-serif; margin: 40px; }
                        .container { max-width: 800px; margin: 0 auto; }
                        .api-link { display: block; margin: 10px 0; padding: 10px; 
                                   background: #f5f5f5; text-decoration: none; color: #333; }
                        .api-link:hover { background: #e0e0e0; }
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>Uveddi Interactive Reports API</h1>
                        <p>The React SPA is not yet available. API endpoints:</p>
                        <a href="/api/v1/reports/demo" class="api-link">
                            <strong>GET /api/v1/reports/demo</strong><br>
                            Demo interactive report data
                        </a>
                        <a href="/api/v1/reports" class="api-link">
                            <strong>GET /api/v1/reports</strong><br>
                            List available reports
                        </a>
                        <a href="/health" class="api-link">
                            <strong>GET /health</strong><br>
                            Service health status
                        </a>
                    </div>
                </body>
                </html>
            "#;

            warp::get()
                .map(move || warp::reply::html(fallback_html))
                .boxed()
        }
    }

    /// Health and metrics routes
    fn health_routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // Health check endpoint
        let health = warp::path("health")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&serde_json::json!({
                    "status": "healthy",
                    "service": "uveddi-interactive-reports",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "api_version": REPORT_SCHEMA_VERSION
                }))
            });

        // Basic metrics endpoint
        let metrics = warp::path("metrics")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&serde_json::json!({
                    "reports_served": 0, // TODO: Implement metrics collection
                    "cache_hit_rate": 0.0,
                    "average_response_time_ms": 0.0,
                    "active_connections": 0
                }))
            });

        health.or(metrics)
    }

    /// CORS filter configuration
    fn cors_filter(&self) -> warp::cors::Builder {
        if self.config.enable_cors {
            let origins: Vec<&str> = self.config.cors_origins.iter().map(|s| s.as_str()).collect();
            warp::cors()
                .allow_origins(origins)
                .allow_headers(vec!["content-type", "authorization", "accept"])
                .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        } else {
            warp::cors().allow_any_origin()
        }
    }

    /// Security headers middleware
    fn security_headers(&self) -> warp::reply::with::HeaderValue<&'static str> {
        if self.config.enable_csp {
            // Strict CSP for local-only operation
            let csp = "default-src 'self'; \
                       img-src 'self' data:; \
                       style-src 'self' 'unsafe-inline'; \
                       script-src 'self'; \
                       connect-src 'self'; \
                       font-src 'self' data:; \
                       frame-ancestors 'none'";
            warp::reply::with::header("content-security-policy", csp)
        } else {
            warp::reply::with::header("x-content-type-options", "nosniff")
        }
    }
}

// Handler functions

async fn list_reports_handler(
    params: PaginationParams,
    db: Arc<Database>,
    config: RestApiConfig,
) -> Result<impl warp::Reply, warp::Rejection> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100); // Max 100 per page
    let offset = (page - 1) * limit;

    // Query recent analysis runs from database
    // Note: This is a placeholder implementation - you'd implement the actual database query
    let reports = vec![
        ReportListItem {
            id: "demo".to_string(),
            name: "Demo Analysis".to_string(),
            path: "/demo".to_string(),
            generated_at: Utc::now(),
            issues_total: 12,
            status: "completed".to_string(),
            files_analyzed: 42,
        }
    ];

    let response = ApiResponse {
        data: serde_json::json!({
            "reports": reports,
            "pagination": {
                "page": page,
                "limit": limit,
                "total": 1,
                "pages": 1
            }
        }),
        timestamp: Utc::now(),
        schema_version: REPORT_SCHEMA_VERSION.to_string(),
    };

    Ok(warp::reply::json(&response))
}

async fn get_report_handler(
    report_id: String,
    db: Arc<Database>,
    config: RestApiConfig,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Try to load report from storage first
    let report_path = config.reports_storage_path.join(format!("{}.json", report_id));
    
    if let Ok(report_data) = fs::read_to_string(&report_path).await {
        if let Ok(report) = serde_json::from_str::<InteractiveReport>(&report_data) {
            let response = ApiResponse {
                data: report,
                timestamp: Utc::now(),
                schema_version: REPORT_SCHEMA_VERSION.to_string(),
            };
            return Ok(warp::reply::json(&response));
        }
    }

    // If not found in storage, try to generate from database
    // Note: This would be implemented based on the specific requirements
    // For now, return a 404
    Err(warp::reject::not_found())
}

async fn get_dependency_graph_handler(
    report_id: String,
    db: Arc<Database>,
    config: RestApiConfig,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Load just the dependency graph portion of the report
    let report_path = config.reports_storage_path.join(format!("{}.json", report_id));
    
    if let Ok(report_data) = fs::read_to_string(&report_path).await {
        if let Ok(report) = serde_json::from_str::<InteractiveReport>(&report_data) {
            let response = ApiResponse {
                data: report.dependency_graph,
                timestamp: Utc::now(),
                schema_version: REPORT_SCHEMA_VERSION.to_string(),
            };
            return Ok(warp::reply::json(&response));
        }
    }

    Err(warp::reject::not_found())
}

async fn demo_report_handler() -> Result<impl warp::Reply, warp::Rejection> {
    // Return a demo report for development and testing
    let demo_report = InteractiveReport::default();
    
    let response = ApiResponse {
        data: demo_report,
        timestamp: Utc::now(),
        schema_version: REPORT_SCHEMA_VERSION.to_string(),
    };

    Ok(warp::reply::json(&response))
}

// Error handling

async fn handle_api_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message, details) = if err.is_not_found() {
        (
            warp::http::StatusCode::NOT_FOUND,
            "Resource not found".to_string(),
            None,
        )
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        (
            warp::http::StatusCode::BAD_REQUEST,
            "Invalid request body".to_string(),
            None,
        )
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        (
            warp::http::StatusCode::METHOD_NOT_ALLOWED,
            "Method not allowed".to_string(),
            None,
        )
    } else {
        eprintln!("Unhandled API rejection: {:?}", err);
        (
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
            None,
        )
    };

    let error_response = ApiError {
        error: code.canonical_reason().unwrap_or("Unknown Error").to_string(),
        message,
        timestamp: Utc::now(),
        details,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&error_response),
        code,
    ))
}

// Helper filters

fn with_db(
    db: Arc<Database>,
) -> impl Filter<Extract = (Arc<Database>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_config(
    config: RestApiConfig,
) -> impl Filter<Extract = (RestApiConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

/// Combined server for both GraphQL and REST APIs
pub struct CombinedApiServer {
    pub rest_config: RestApiConfig,
    pub port: u16,
}

impl CombinedApiServer {
    pub fn new(rest_config: RestApiConfig, port: u16) -> Self {
        Self { rest_config, port }
    }

    pub async fn start(
        self,
        database: Arc<Database>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting Uveddi Interactive Reports API on http://localhost:{}", self.port);
        println!("  - REST API: http://localhost:{}/api/v1/reports", self.port);
        println!("  - Health: http://localhost:{}/health", self.port);
        
        if self.rest_config.serve_spa {
            println!("  - SPA: http://localhost:{}/", self.port);
        }

        let api_service = RestApiService::new(self.rest_config, database);
        let routes = api_service.routes();

        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([127, 0, 0, 1], self.port), async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for ctrl-c signal");
                println!("Received shutdown signal, gracefully shutting down server...");
            });

        server.await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    async fn create_test_config() -> RestApiConfig {
        RestApiConfig {
            serve_spa: false,
            enable_csp: false,
            reports_storage_path: PathBuf::from("/tmp/test_reports"),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_demo_report_endpoint() {
        let config = create_test_config().await;
        let db = Arc::new(Database::new(":memory:").unwrap());
        let api_service = RestApiService::new(config, db);
        
        let routes = api_service.routes();
        
        let response = warp::test::request()
            .method("GET")
            .path("/api/v1/reports/demo")
            .reply(&routes)
            .await;

        assert_eq!(response.status(), 200);
        
        let body: ApiResponse<InteractiveReport> = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(body.data.project.name, "Demo Project");
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let config = create_test_config().await;
        let db = Arc::new(Database::new(":memory:").unwrap());
        let api_service = RestApiService::new(config, db);
        
        let routes = api_service.routes();
        
        let response = warp::test::request()
            .method("GET")
            .path("/health")
            .reply(&routes)
            .await;

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "uveddi-interactive-reports");
    }

    #[tokio::test]
    async fn test_not_found_handling() {
        let config = create_test_config().await;
        let db = Arc::new(Database::new(":memory:").unwrap());
        let api_service = RestApiService::new(config, db);
        
        let routes = api_service.routes();
        
        let response = warp::test::request()
            .method("GET")
            .path("/api/v1/reports/nonexistent")
            .reply(&routes)
            .await;

        assert_eq!(response.status(), 404);
        
        let body: ApiError = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.error, "Not Found");
    }
}