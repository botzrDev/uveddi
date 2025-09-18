//! REST API endpoints for interactive reporting
//!
//! This module provides REST API endpoints specifically designed for the interactive
//! reporting SPA frontend. It serves versioned JSON contracts and static assets
//! for the React application while maintaining compatibility with the existing
//! GraphQL infrastructure.
//!
//! This module has been refactored to use a modular endpoint structure located
//! in the `endpoints` submodule for better maintainability and separation of concerns.
//!
//! # API Design
//!
//! The REST API follows these principles:
//! - Versioned endpoints for API contract stability (/api/v1/...)
//! - JSON-first with consistent error handling
//! - Offline-first with local file system storage
//! - Security-first with CSP headers and CORS restrictions
//! - Performance-optimized with caching and compression
//! - Modular endpoint organization by domain (health, reports, security)
//!
//! # Endpoints
//!
//! - `GET /api/v1/reports/{id}` - Get interactive report by ID
//! - `GET /api/v1/reports/{id}/graphs/dependency` - Get dependency graph data
//! - `GET /api/v1/reports` - List available reports
//! - `GET /health` - Health check (shared with GraphQL)
//! - `GET /metrics` - Basic metrics (shared with GraphQL)
//! - `GET /app/*` - Serve SPA static assets
//! - `GET /*` - SPA fallback for client-side routing

use crate::api::types::{ApiServer, RestApiConfig};
use crate::database::Database;
use crate::report::interactive_models::REPORT_SCHEMA_VERSION;
use crate::security::{self, validate_api_request};

// Import modular endpoints
#[path = "rest_endpoints/mod.rs"]
mod endpoints;
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json},
    routing::{get, get_service},
    Router,
};
use chrono::Utc;
use endpoints::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{error, info, warn};

/// REST API Service implementation
#[derive(Clone)]
pub struct RestApiService {
    config: RestApiConfig,
    database: Arc<Database>,
}

impl RestApiService {
    pub fn new(config: RestApiConfig, database: Arc<Database>) -> Self {
        Self { config, database }
    }

    /// Create all REST API routes with proper SPA fallback
    pub fn create_app_with_state(&self) -> Router {
        // Create shared state
        let state = Arc::new(AppState {
            config: self.config.clone(),
            database: self.database.clone(),
        });

        // API v1 routes - using modular endpoints
        let api_routes = Router::new()
            // Report endpoints
            .route("/reports", get(list_reports))
            .route("/reports/{id}", get(get_report))
            .route("/reports/{id}/graphs/dependency", get(get_dependency_graph))
            .route("/reports/demo", get(demo_report_handler))
            // Security endpoints
            .route("/security/issues", get(get_security_issues))
            .route("/security/issues/{id}", get(get_security_issue))
            .route("/security/summary", get(get_security_summary))
            .route("/security/owasp-coverage", get(get_owasp_coverage))
            .route("/security/taint-flows", get(get_taint_flows))
            .route("/security/sarif", get(export_sarif));

        // Build the main app router
        let mut app = Router::new()
            .nest("/api/v1", api_routes)
            .route("/health", get(local_health_check))
            .with_state(state.clone());

        // Add static file serving and SPA fallback if assets path is configured
        if let Some(assets_path) = &self.config.spa_assets_path {
            if assets_path.exists() {
                use tower_http::services::ServeFile;

                // Use the canonical SPA pattern: ServeDir with ServeFile fallback
                let serve_dir = ServeDir::new(assets_path.clone())
                    .fallback(ServeFile::new(assets_path.join("index.html")));

                // Add static file serving at /app and SPA fallback for everything else
                app = app.nest_service("/app", get_service(serve_dir.clone()));
                app = app.fallback_service(get_service(serve_dir));
            }
        }

        // Add CORS if enabled
        if self.config.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            app = app.layer(cors);
        }

        // Add middleware with validation
        app.layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(validate_request_middleware))
                .layer(TraceLayer::new_for_http()),
        )
    }
}

/// Combined API server that can run both GraphQL and REST endpoints
pub struct CombinedApiServer {
    config: RestApiConfig,
    port: u16,
}

impl CombinedApiServer {
    pub fn new(config: RestApiConfig, port: u16) -> Self {
        Self { config, port }
    }

    /// Start the combined server
    pub async fn start(
        self,
        database: Arc<Database>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let service = RestApiService::new(self.config, database);
        let app = service.create_app_with_state();

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        info!("🌐 Server listening on http://0.0.0.0:{}", self.port);

        axum::serve(listener, app).await?;
        Ok(())
    }

    /// Start the combined server with readiness notification
    pub async fn start_with_readiness(
        self,
        database: Arc<Database>,
        ready_tx: tokio::sync::oneshot::Sender<
            Result<(), Box<dyn std::error::Error + Send + Sync>>,
        >,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let service = RestApiService::new(self.config, database);
        let app = service.create_app_with_state();

        // Bind to the port first
        let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await {
            Ok(listener) => {
                info!("🌐 Server listening on http://0.0.0.0:{}", self.port);
                // Signal that we're ready to accept connections
                let _ = ready_tx.send(Ok(()));
                listener
            }
            Err(e) => {
                let error = Box::new(e) as Box<dyn std::error::Error + Send + Sync>;
                let _ = ready_tx.send(Err(error));
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::AddrInUse,
                    "Failed to bind to port",
                )));
            }
        };

        axum::serve(listener, app).await?;
        Ok(())
    }
}

impl ApiServer for CombinedApiServer {
    async fn start(
        self,
        database: Arc<Database>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.start(database).await
    }

    async fn start_with_readiness(
        self,
        database: Arc<Database>,
        ready_tx: tokio::sync::oneshot::Sender<
            Result<(), Box<dyn std::error::Error + Send + Sync>>,
        >,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.start_with_readiness(database, ready_tx).await
    }
}

/// Shared application state - moved to public visibility for endpoint modules
#[derive(Clone)]
pub struct AppState {
    pub config: RestApiConfig,
    pub database: Arc<Database>,
}

/// Request validation middleware that validates HTTP headers and parameters
async fn validate_request_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    // Extract headers for validation
    let headers = request.headers();

    // Get content type
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok());

    // Get content length
    let content_length = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|cl| cl.to_str().ok())
        .and_then(|cl| cl.parse::<u64>().ok());

    // Get user agent
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|ua| ua.to_str().ok());

    // Validate request headers and parameters
    match validate_api_request(content_type, content_length, user_agent) {
        Ok(_) => {
            // Continue to the next middleware/handler
            Ok(next.run(request).await)
        }
        Err(e) => {
            error!("API request validation failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Query parameters for paginated endpoints
#[derive(Debug, Deserialize)]
struct PaginationQuery {
    offset: Option<u32>,
    limit: Option<u32>,
    severity: Option<String>,
    detector: Option<String>,
}

// Health check endpoint is now in endpoints::health module
// but we provide a local wrapper for backward compatibility
async fn local_health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "api_version": "v1",
        "schema_version": REPORT_SCHEMA_VERSION
    }))
}

// All endpoint implementations have been moved to the endpoints module
// for better maintainability and separation of concerns.
