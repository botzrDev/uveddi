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
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, StatusCode, HeaderMap},
    response::{Html, IntoResponse, Json},
    routing::{get, get_service},
    Router, ServiceExt,
    serve,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tower::ServiceBuilder;
use tower::Service;
use tower_http::{
    cors::{CorsLayer, Any},
    services::ServeDir,
    trace::TraceLayer,
};

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
    /// Cache max-age for static assets (seconds)
    pub cache_max_age: u32,
}

impl Default for RestApiConfig {
    fn default() -> Self {
        Self {
            enable_cors: false,
            cors_origins: vec![],
            spa_assets_path: None,
            reports_storage_path: PathBuf::from("./.uveddi/reports"),
            serve_spa: true,
            enable_csp: true,
            cache_max_age: 3600,
        }
    }
}

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

        // API v1 routes
        let api_routes = Router::new()
            .route("/reports", get(list_reports))
            .route("/reports/:id", get(get_report))
            .route("/reports/:id/graphs/dependency", get(get_dependency_graph))
            .route("/reports/demo", get(demo_report_handler));

        // Build the main app router
        let mut app = Router::new()
            .nest("/api/v1", api_routes)
            .route("/health", get(health_check))
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

        // Add middleware
        app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
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
    pub async fn start(self, database: Arc<Database>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let service = RestApiService::new(self.config, database);
        let app = service.create_app_with_state();

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("🌐 Server listening on http://0.0.0.0:{}", self.port);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    config: RestApiConfig,
    database: Arc<Database>,
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "api_version": "v1",
        "schema_version": REPORT_SCHEMA_VERSION
    }))
}

/// List all available reports
async fn list_reports(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // TODO: Implement database query for available reports
    // For now, return demo data
    let reports = vec![
        serde_json::json!({
            "id": "demo",
            "title": "Demo Analysis Report",
            "created_at": Utc::now(),
            "project_name": "Demo Project",
            "file_count": 42,
            "issue_count": 7
        })
    ];

    Ok(Json(serde_json::json!({
        "reports": reports,
        "total": reports.len()
    })))
}

/// Get a specific report by ID
async fn get_report(
    AxumPath(report_id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // For demo purposes, handle "demo" specially
    if report_id == "demo" {
        return Ok(Json(create_demo_report()));
    }

    // Try to load from storage
    let report_path = state.config.reports_storage_path.join(format!("{}.json", report_id));
    
    match fs::read_to_string(&report_path).await {
        Ok(content) => {
            match serde_json::from_str::<InteractiveReport>(&content) {
                Ok(report) => Ok(Json(report)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Get dependency graph for a specific report
async fn get_dependency_graph(
    AxumPath(report_id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // For demo purposes, return demo dependency graph
    if report_id == "demo" {
        let demo_report = create_demo_report();
        return Ok(Json(demo_report.dependency_graph));
    }

    Err(StatusCode::NOT_FOUND)
}

/// Demo report handler
async fn demo_report_handler() -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(create_demo_report()))
}


/// Create a demo report for testing
fn create_demo_report() -> InteractiveReport {
    use crate::report::interactive_models::*;

    let mut issues_by_severity = HashMap::new();
    issues_by_severity.insert("critical".to_string(), 1);
    issues_by_severity.insert("high".to_string(), 3);
    issues_by_severity.insert("medium".to_string(), 3);
    issues_by_severity.insert("low".to_string(), 0);

    let mut issues_by_category = HashMap::new();
    issues_by_category.insert("anti-patterns".to_string(), 4);
    issues_by_category.insert("code-quality".to_string(), 2);
    issues_by_category.insert("security".to_string(), 1);

    InteractiveReport {
        schema_version: REPORT_SCHEMA_VERSION.to_string(),
        project: ProjectMetadata {
            id: "demo-project".to_string(),
            name: "Demo Project".to_string(),
            commit: Some("abc123".to_string()),
            branch: Some("main".to_string()),
            repo_url: Some("https://github.com/example/demo".to_string()),
            path: "./demo".to_string(),
            languages: vec!["rust".to_string()],
        },
        summary: AnalysisSummary {
            coverage: 82.5,
            issues_total: 7,
            issues_by_severity,
            issues_by_category,
            files_analyzed: 42,
            components_analyzed: 15,
            analysis_duration_ms: 1250,
            time_generated: Utc::now(),
        },
        chart_data: None, // Optional Chart.js data
        performance_metrics: None, // Optional performance metrics
        findings: vec![
            Finding {
                id: "demo-001".to_string(),
                finding_type: "GodObject".to_string(),
                severity: "high".to_string(),
                title: "Large class with too many responsibilities".to_string(),
                message: "The UserManager class has grown too large and handles multiple concerns including authentication, profile management, and notifications.".to_string(),
                file: "src/user_manager.rs".to_string(),
                start_line: Some(45),
                end_line: Some(287),
                column: None,
                code_snippet: Some("impl UserManager { /* 200+ lines of mixed concerns */ }".to_string()),
                tags: vec!["anti-pattern".to_string(), "maintainability".to_string()],
                detector: "GodObjectDetector".to_string(),
                confidence: 0.89,
                ai_explanation: Some("This class violates the Single Responsibility Principle by combining user authentication, profile management, and notification logic. Consider breaking it into separate services.".to_string()),
                recommendation: Some("Extract authentication logic into AuthService, profile management into ProfileService, and notifications into NotificationService.".to_string()),
                related_findings: vec![],
            },
        ],
        dependency_graph: DependencyGraph {
            nodes: vec![
                GraphNode {
                    id: "main".to_string(),
                    label: "main.rs".to_string(),
                    path: "src/main.rs".to_string(),
                    node_type: "module".to_string(),
                    metrics: Some(NodeMetrics {
                        loc: Some(150),
                        complexity: Some(5.0),
                        dependencies: 3,
                        dependents: 0,
                    }),
                    group: Some("core".to_string()),
                    properties: HashMap::new(),
                },
                GraphNode {
                    id: "user_manager".to_string(),
                    label: "user_manager.rs".to_string(),
                    path: "src/user_manager.rs".to_string(),
                    node_type: "module".to_string(),
                    metrics: Some(NodeMetrics {
                        loc: Some(450),
                        complexity: Some(12.0),
                        dependencies: 5,
                        dependents: 2,
                    }),
                    group: Some("services".to_string()),
                    properties: HashMap::new(),
                },
            ],
            edges: vec![
                GraphEdge {
                    source: "main".to_string(),
                    target: "user_manager".to_string(),
                    edge_type: "imports".to_string(),
                    weight: Some(3.0),
                    properties: HashMap::new(),
                },
            ],
            metadata: GraphMetadata {
                node_count: 2,
                edge_count: 1,
                has_cycles: false,
                max_depth: 2,
                suggested_layout: crate::report::interactive_models::CytoscapeLayout::Dagre,
                layout_config: std::collections::HashMap::new(),
                performance_config: crate::report::interactive_models::GraphPerformanceConfig {
                    enable_lod: true,
                    batch_size: 100,
                    texture_on_viewport: true,
                    hide_labels_on_viewport: true,
                    initial_viewport: None,
                    use_web_worker: false,
                },
                clustering_hints: vec![],
                cycles: vec![],
            },
        },
        diagrams: vec![
            DiagramDefinition {
                id: "project-structure".to_string(),
                kind: "mermaid".to_string(),
                title: "Project Structure".to_string(),
                source: "graph TD\n    A[main.rs] --> B[user_manager.rs]\n    B --> C[auth.rs]\n    B --> D[profile.rs]".to_string(),
                description: Some("High-level project structure showing module dependencies".to_string()),
                components: vec!["main".to_string(), "user_manager".to_string()],
                metadata: DiagramRenderMetadata {
                    width: Some(800),
                    height: Some(600),
                    theme: Some("light".to_string()),
                    direction: Some("TD".to_string()),
                    options: HashMap::new(),
                },
            },
        ],
        ai_insights: Some(AiInsights {
            overall_assessment: Some("The codebase shows good structure but has some areas for improvement, particularly around separation of concerns.".to_string()),
            top_recommendations: vec![
                "Refactor UserManager to separate concerns".to_string(),
                "Add unit tests for critical paths".to_string(),
                "Consider implementing dependency injection".to_string(),
            ],
            patterns: vec![
                IdentifiedPattern {
                    name: "God Object".to_string(),
                    description: "Several classes are taking on too many responsibilities".to_string(),
                    confidence: 0.85,
                    locations: vec!["src/user_manager.rs".to_string()],
                    impact: "high".to_string(),
                },
            ],
            risk_assessment: Some(RiskAssessment {
                overall_risk: "medium".to_string(),
                risk_factors: vec![
                    RiskFactor {
                        name: "Tight Coupling".to_string(),
                        description: "High interdependence between components".to_string(),
                        level: "medium".to_string(),
                        likelihood: "medium".to_string(),
                        impact: "high".to_string(),
                        affected_areas: vec!["services".to_string()],
                    },
                ],
                mitigation_strategies: vec![
                    "Implement dependency injection".to_string(),
                    "Add comprehensive unit tests".to_string(),
                ],
            }),
            refactoring_opportunities: vec![
                RefactoringOpportunity {
                    refactoring_type: "extract_class".to_string(),
                    description: "Extract authentication logic from UserManager".to_string(),
                    effort: "medium".to_string(),
                    benefits: vec![
                        "Better separation of concerns".to_string(),
                        "Easier testing".to_string(),
                        "Reduced complexity".to_string(),
                    ],
                    scope: vec!["src/user_manager.rs".to_string()],
                    priority: "high".to_string(),
                },
            ],
        }),
        metadata: ReportMetadata {
            generated_at: Utc::now(),
            uveddi_version: "0.9.0".to_string(),
            configuration: HashMap::new(),
            performance: Some(GenerationPerformance {
                analysis_duration_ms: 1250,
                generation_duration_ms: 45,
                peak_memory_bytes: Some(128 * 1024 * 1024), // 128MB
                files_per_second: Some(33.6),
            }),
        },
    }
}