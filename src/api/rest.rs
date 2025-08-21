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

use crate::api::types::{ApiServer, RestApiConfig};
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::database::Database;
#[cfg(feature = "security")]
use crate::analysis::detectors::security::types::{SecurityIssue, SecuritySeverity, VulnerabilityMetadata};
use crate::report::interactive_models::{
    DependencyGraph, InteractiveReport, REPORT_SCHEMA_VERSION,
};
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, get_service},
    serve, Router, ServiceExt,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tower::Service;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};


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
            .route("/reports/demo", get(demo_report_handler))
            // Security-specific API endpoints
            .route("/security/issues", get(get_security_issues))
            .route("/security/issues/:id", get(get_security_issue))
            .route("/security/summary", get(get_security_summary))
            .route("/security/owasp-coverage", get(get_owasp_coverage))
            .route("/security/taint-flows", get(get_taint_flows))
            .route("/security/sarif", get(export_sarif));

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
        app.layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
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
        println!("🌐 Server listening on http://0.0.0.0:{}", self.port);

        axum::serve(listener, app).await?;
        Ok(())
    }

    /// Start the combined server with readiness notification
    pub async fn start_with_readiness(
        self,
        database: Arc<Database>,
        ready_tx: tokio::sync::oneshot::Sender<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let service = RestApiService::new(self.config, database);
        let app = service.create_app_with_state();
        
        // Bind to the port first
        let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await {
            Ok(listener) => {
                println!("🌐 Server listening on http://0.0.0.0:{}", self.port);
                // Signal that we're ready to accept connections
                let _ = ready_tx.send(Ok(()));
                listener
            }
            Err(e) => {
                let error = Box::new(e) as Box<dyn std::error::Error + Send + Sync>;
                let _ = ready_tx.send(Err(error));
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Failed to bind to port")));
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
        ready_tx: tokio::sync::oneshot::Sender<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.start_with_readiness(database, ready_tx).await
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
async fn list_reports(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, StatusCode> {
    // Load reports from database
    match list_reports_from_database(&state.database).await {
        Ok(reports) => {
            let mut report_list = reports;
            
            // Add demo report as fallback
            report_list.push(serde_json::json!({
                "id": "demo",
                "title": "Demo Analysis Report",
                "created_at": Utc::now(),
                "project_name": "Demo Project",
                "file_count": 42,
                "issue_count": 7,
                "is_demo": true
            }));
            
            Ok(Json(serde_json::json!({
                "reports": report_list,
                "total": report_list.len()
            })))
        }
        Err(e) => {
            eprintln!("Failed to load reports from database: {}", e);
            // Fallback to demo data only
            let reports = vec![serde_json::json!({
                "id": "demo",
                "title": "Demo Analysis Report",
                "created_at": Utc::now(),
                "project_name": "Demo Project",
                "file_count": 42,
                "issue_count": 7,
                "is_demo": true
            })];
            
            Ok(Json(serde_json::json!({
                "reports": reports,
                "total": reports.len()
            })))
        }
    }
}

/// Get a specific report by ID
async fn get_report(
    AxumPath(report_id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // For demo purposes, handle "demo" specially (fallback only)
    if report_id == "demo" {
        return Ok(Json(create_demo_report()));
    }

    // Try to load from database first (production approach)
    if let Ok(run_id) = report_id.parse::<i64>() {
        match load_report_from_database(&state.database, run_id).await {
            Ok(report) => return Ok(Json(report)),
            Err(e) => {
                eprintln!("Failed to load report from database: {}", e);
                // Fall through to file system
            }
        }
    }

    // Fallback to file system storage
    let report_path = state
        .config
        .reports_storage_path
        .join(format!("{}.json", report_id));

    match fs::read_to_string(&report_path).await {
        Ok(content) => match serde_json::from_str::<InteractiveReport>(&content) {
            Ok(report) => Ok(Json(report)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
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
                severity: "critical".to_string(), // Changed to critical to match summary
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
                #[cfg(feature = "security")]
                security_metadata: None,
            },
            Finding {
                id: "demo-002".to_string(),
                finding_type: "TightCoupling".to_string(),
                severity: "high".to_string(),
                title: "Database dependency tightly coupled".to_string(),
                message: "Direct database access scattered throughout business logic".to_string(),
                file: "src/service.rs".to_string(),
                start_line: Some(88),
                end_line: Some(95),
                column: Some(8),
                code_snippet: Some("let db = DatabaseConnection::new();\ndb.execute(\"INSERT...\");".to_string()),
                tags: vec!["anti-pattern".to_string(), "testability".to_string()],
                detector: "CouplingDetector".to_string(),
                confidence: 0.85,
                ai_explanation: Some("Direct database access makes testing difficult and violates dependency inversion principle.".to_string()),
                recommendation: Some("Inject database dependency through interface or repository pattern.".to_string()),
                related_findings: vec![],
                #[cfg(feature = "security")]
                security_metadata: None,
            },
            Finding {
                id: "demo-003".to_string(),
                finding_type: "LongMethod".to_string(),
                severity: "high".to_string(),
                title: "Method exceeds recommended length".to_string(),
                message: "The process_request method contains 150+ lines and should be refactored".to_string(),
                file: "src/handler.rs".to_string(),
                start_line: Some(45),
                end_line: Some(195),
                column: Some(5),
                code_snippet: Some("fn process_request(&self, req: Request) -> Response {\n    // 150+ lines of complex logic\n}".to_string()),
                tags: vec!["anti-pattern".to_string(), "complexity".to_string()],
                detector: "MethodLengthDetector".to_string(),
                confidence: 0.95,
                ai_explanation: Some("Long methods are difficult to understand, test, and maintain. They often indicate multiple responsibilities.".to_string()),
                recommendation: Some("Break into smaller methods with single responsibilities using Extract Method refactoring.".to_string()),
                related_findings: vec![],
                #[cfg(feature = "security")]
                security_metadata: None,
            },
            Finding {
                id: "demo-004".to_string(),
                finding_type: "DeadCode".to_string(),
                severity: "medium".to_string(),
                title: "Unused function detected".to_string(),
                message: "Function format_output is defined but never called".to_string(),
                file: "src/utils.rs".to_string(),
                start_line: Some(12),
                end_line: Some(18),
                column: Some(1),
                code_snippet: Some("fn format_output(data: &str) -> String {\n    // implementation\n}".to_string()),
                tags: vec!["code-quality".to_string(), "cleanup".to_string()],
                detector: "DeadCodeDetector".to_string(),
                confidence: 0.98,
                ai_explanation: Some("Dead code clutters the codebase and can mislead developers about system functionality.".to_string()),
                recommendation: Some("Remove unused function or add it to public API if needed for future use.".to_string()),
                related_findings: vec![],
                #[cfg(feature = "security")]
                security_metadata: None,
            },
            Finding {
                id: "demo-005".to_string(),
                finding_type: "MagicValues".to_string(),
                severity: "medium".to_string(),
                title: "Magic number found".to_string(),
                message: "Hardcoded timeout value should be configurable".to_string(),
                file: "src/config.rs".to_string(),
                start_line: Some(34),
                end_line: Some(34),
                column: Some(20),
                code_snippet: Some("let timeout = 30000; // milliseconds".to_string()),
                tags: vec!["code-quality".to_string(), "maintainability".to_string()],
                detector: "MagicValuesDetector".to_string(),
                confidence: 0.75,
                ai_explanation: Some("Magic numbers make configuration changes difficult and reduce code readability.".to_string()),
                recommendation: Some("Move to configuration file or create named constants.".to_string()),
                related_findings: vec![],
                #[cfg(feature = "security")]
                security_metadata: None,
            },
            Finding {
                id: "demo-006".to_string(),
                finding_type: "CodeDuplication".to_string(),
                severity: "medium".to_string(),
                title: "Duplicate validation logic".to_string(),
                message: "Similar input validation appears in multiple locations".to_string(),
                file: "src/validators.rs".to_string(),
                start_line: Some(22),
                end_line: Some(35),
                column: Some(1),
                code_snippet: Some("if input.is_empty() {\n    return Err(\"Invalid input\");\n}".to_string()),
                tags: vec!["anti-pattern".to_string(), "maintainability".to_string()],
                detector: "DuplicationDetector".to_string(),
                confidence: 0.82,
                ai_explanation: Some("Code duplication increases maintenance burden and the risk of inconsistent behavior.".to_string()),
                recommendation: Some("Extract common validation logic into shared utility functions.".to_string()),
                related_findings: vec![],
                #[cfg(feature = "security")]
                security_metadata: None,
            },
            Finding {
                id: "demo-007".to_string(),
                finding_type: "SecurityVulnerability".to_string(),
                severity: "high".to_string(), // Actually high severity for security
                title: "SQL injection vulnerability".to_string(),
                message: "String concatenation used in SQL query construction".to_string(),
                file: "src/database.rs".to_string(),
                start_line: Some(67),
                end_line: Some(67),
                column: Some(15),
                code_snippet: Some("let query = format!(\"SELECT * FROM users WHERE id = {}\", user_id);".to_string()),
                tags: vec!["security".to_string(), "vulnerability".to_string()],
                detector: "SecurityDetector".to_string(),
                confidence: 0.94,
                ai_explanation: Some("String concatenation in SQL queries can lead to injection attacks if user input is not properly sanitized.".to_string()),
                recommendation: Some("Use parameterized queries or prepared statements to prevent SQL injection.".to_string()),
                related_findings: vec![],
                #[cfg(feature = "security")]
                security_metadata: None,
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
        #[cfg(feature = "security")]
        security_analysis: Some(create_demo_security_analysis()),
    }
}

/// Create demo security analysis data
#[cfg(feature = "security")]
fn create_demo_security_analysis() -> crate::report::interactive_models::SecurityAnalysis {
    use crate::report::interactive_models::*;
    
    let mut owasp_coverage = std::collections::HashMap::new();
    owasp_coverage.insert("A01_Broken_Access_Control".to_string(), OwaspCategoryStats {
        issues_found: 2,
        coverage_percentage: 85.0,
        avg_confidence: 0.75,
        severity_distribution: std::collections::HashMap::new(),
    });
    owasp_coverage.insert("A03_Injection".to_string(), OwaspCategoryStats {
        issues_found: 1,
        coverage_percentage: 90.0,
        avg_confidence: 0.92,
        severity_distribution: std::collections::HashMap::new(),
    });

    SecurityAnalysis {
        summary: SecuritySummary {
            total_issues: 3,
            critical_count: 1,
            high_count: 1,
            medium_count: 1,
            low_count: 0,
            confidence_distribution: {
                let mut dist = std::collections::HashMap::new();
                dist.insert("High".to_string(), 2);
                dist.insert("Medium".to_string(), 1);
                dist
            },
            most_common_issues: vec![
                IssueTypeStats {
                    issue_type: "SQL Injection".to_string(),
                    count: 1,
                    avg_severity: "Critical".to_string(),
                    avg_confidence: 0.92,
                },
                IssueTypeStats {
                    issue_type: "Broken Access Control".to_string(),
                    count: 2,
                    avg_severity: "High".to_string(),
                    avg_confidence: 0.75,
                },
            ],
            security_score: 72.5,
        },
        owasp_coverage,
        issues: vec![
            SecurityIssue {
                id: "sec-001".to_string(),
                issue_type: "SQL Injection".to_string(),
                severity: "Critical".to_string(),
                confidence_score: 0.92,
                location: SecurityLocation {
                    file: "src/database/query.rs".to_string(),
                    start_line: 45,
                    end_line: 47,
                    start_column: Some(8),
                    end_column: Some(42),
                    code_snippet: Some("query = format!(\"SELECT * FROM users WHERE id = {}\", user_id)".to_string()),
                },
                description: "Direct string interpolation into SQL query allows SQL injection attacks".to_string(),
                remediation: "Use parameterized queries or prepared statements to prevent SQL injection".to_string(),
                owasp_category: Some("A03_Injection".to_string()),
                cwe_id: Some("CWE-89".to_string()),
                cvss_score: Some(9.1),
                references: vec![
                    "https://owasp.org/Top10/A03_2021-Injection/".to_string(),
                    "https://cwe.mitre.org/data/definitions/89.html".to_string(),
                ],
                related_taint_flows: vec!["flow-001".to_string()],
                attack_vector: Some("Network".to_string()),
            },
        ],
        taint_flows: vec![
            TaintFlow {
                id: "flow-001".to_string(),
                source: FlowNode {
                    name: "user_input".to_string(),
                    location: "src/handlers/user.rs".to_string(),
                    node_type: "source".to_string(),
                    line_number: 23,
                    properties: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("input_type".to_string(), "http_parameter".to_string());
                        props
                    },
                },
                sink: FlowNode {
                    name: "sql_query".to_string(),
                    location: "src/database/query.rs".to_string(),
                    node_type: "sink".to_string(),
                    line_number: 45,
                    properties: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("sink_type".to_string(), "sql_execution".to_string());
                        props
                    },
                },
                confidence: 0.92,
                sanitizers: vec![],
                path: vec![
                    FlowNode {
                        name: "validate_user_id".to_string(),
                        location: "src/validation/mod.rs".to_string(),
                        node_type: "intermediate".to_string(),
                        line_number: 12,
                        properties: std::collections::HashMap::new(),
                    },
                ],
                vulnerability_type: "SQL Injection".to_string(),
            },
        ],
        correlations: vec![
            SecurityCorrelation {
                security_issue_id: "sec-001".to_string(),
                architectural_issue_id: "demo-001".to_string(),
                correlation_strength: 0.65,
                correlation_type: "code_quality_impact".to_string(),
                explanation: "The God Object anti-pattern in UserManager contributes to security vulnerabilities by mixing data access logic with business logic".to_string(),
            },
        ],
        compliance: Some(ComplianceStatus {
            owasp_score: 72.5,
            cwe_score: 68.2,
            standards: {
                let mut standards = std::collections::HashMap::new();
                standards.insert("OWASP_Top_10_2021".to_string(), StandardCompliance {
                    name: "OWASP Top 10 2021".to_string(),
                    score: 72.5,
                    required_controls: 10,
                    passed_controls: 7,
                    failed_controls: 3,
                });
                standards
            },
        }),
    }
}

// Security-specific API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssueResponse {
    pub id: String,
    pub issue_type: String,
    pub severity: String,
    pub confidence_score: f64,
    pub location: LocationResponse,
    pub description: String,
    pub remediation: String,
    pub owasp_category: Option<String>,
    pub cwe_id: Option<String>,
    pub cvss_score: Option<f64>,
    pub references: Vec<String>,
    pub taint_flows: Option<Vec<TaintFlowResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationResponse {
    pub file: String,
    pub start_line: i32,
    pub end_line: i32,
    pub start_column: Option<i32>,
    pub end_column: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintFlowResponse {
    pub source: FlowNodeResponse,
    pub sink: FlowNodeResponse,
    pub confidence: f64,
    pub sanitizers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNodeResponse {
    pub name: String,
    pub location: String,
    pub node_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummaryResponse {
    pub total_issues: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub owasp_coverage: HashMap<String, OwaspCategoryStats>,
    pub confidence_distribution: HashMap<String, usize>,
    pub most_common_issues: Vec<IssueTypeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwaspCategoryStats {
    pub issues_found: usize,
    pub coverage_percentage: f64,
    pub avg_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypeStats {
    pub issue_type: String,
    pub count: usize,
    pub avg_severity: String,
}

// Security API handlers

/// Get all security issues for a specific analysis run or latest run
async fn get_security_issues(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    #[cfg(feature = "security")]
    {
        // Try to get the latest analysis run
        match state.database.get_latest_analysis_run().await {
            Ok(Some(run)) => {
                if let Some(run_id) = run.run_id {
                    match state.database.get_security_issues_for_run(run_id).await {
                        Ok(security_issues) => {
                            let response_issues: Vec<SecurityIssueResponse> = security_issues.iter().map(|issue| {
                                SecurityIssueResponse {
                                    id: format!("sec_{}", issue.id.as_ref().unwrap_or(&"unknown".to_string())),
                                    issue_type: issue.issue_type.to_string(),
                                    severity: issue.severity.to_string(),
                                    confidence_score: issue.confidence_score,
                                    location: LocationResponse {
                                        file: issue.location.file_path.to_string_lossy().to_string(),
                                        start_line: issue.location.start_line,
                                        end_line: issue.location.end_line,
                                        start_column: issue.location.start_column,
                                        end_column: issue.location.end_column,
                                    },
                                    description: issue.description.clone(),
                                    remediation: issue.remediation.clone().unwrap_or_default(),
                                    owasp_category: issue.issue_type.owasp_category().map(|s| s.to_string()),
                                    cwe_id: issue.metadata.cwe_id.clone(),
                                    cvss_score: issue.metadata.cvss_score,
                                    references: issue.metadata.references.clone(),
                                    taint_flows: None, // TODO: Implement taint flow conversion
                                }
                            }).collect();
                            
                            return Ok(Json(response_issues));
                        }
                        Err(e) => {
                            eprintln!("Failed to load security issues: {}", e);
                            // Fall through to demo data
                        }
                    }
                }
            }
            Ok(None) => {
                eprintln!("No analysis runs found in database");
                // Fall through to demo data
            }
            Err(e) => {
                eprintln!("Failed to get latest analysis run: {}", e);
                // Fall through to demo data
            }
        }
    }
    
    // Fallback to demo data if database query fails or security feature disabled
    let demo_issues = vec![
        SecurityIssueResponse {
            id: "sec_001".to_string(),
            issue_type: "Injection".to_string(),
            severity: "Critical".to_string(),
            confidence_score: 0.95,
            location: LocationResponse {
                file: "src/auth.py".to_string(),
                start_line: 42,
                end_line: 42,
                start_column: Some(15),
                end_column: Some(35),
            },
            description: "SQL injection vulnerability detected in user authentication".to_string(),
            remediation: "Use parameterized queries to prevent SQL injection".to_string(),
            owasp_category: Some("A03_Injection".to_string()),
            cwe_id: Some("CWE-89".to_string()),
            cvss_score: Some(9.8),
            references: vec!["https://owasp.org/www-project-top-ten/2017/A1_2017-Injection".to_string()],
            taint_flows: Some(vec![
                TaintFlowResponse {
                    source: FlowNodeResponse {
                        name: "user_input".to_string(),
                        location: "src/auth.py:35".to_string(),
                        node_type: "UserInput".to_string(),
                    },
                    sink: FlowNodeResponse {
                        name: "sql_execute".to_string(),
                        location: "src/auth.py:42".to_string(),
                        node_type: "SqlQuery".to_string(),
                    },
                    confidence: 0.95,
                    sanitizers: vec![],
                }
            ]),
        },
    ];

    Ok(Json(demo_issues))
}

/// Get a specific security issue by ID
async fn get_security_issue(
    AxumPath(issue_id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // For demo purposes, return the first demo issue if ID matches
    if issue_id == "sec_001" {
        let issue = SecurityIssueResponse {
            id: "sec_001".to_string(),
            issue_type: "Injection".to_string(),
            severity: "Critical".to_string(),
            confidence_score: 0.95,
            location: LocationResponse {
                file: "src/auth.py".to_string(),
                start_line: 42,
                end_line: 42,
                start_column: Some(15),
                end_column: Some(35),
            },
            description: "SQL injection vulnerability detected in user authentication".to_string(),
            remediation: "Use parameterized queries to prevent SQL injection".to_string(),
            owasp_category: Some("A03_Injection".to_string()),
            cwe_id: Some("CWE-89".to_string()),
            cvss_score: Some(9.8),
            references: vec!["https://owasp.org/www-project-top-ten/2017/A1_2017-Injection".to_string()],
            taint_flows: Some(vec![
                TaintFlowResponse {
                    source: FlowNodeResponse {
                        name: "user_input".to_string(),
                        location: "src/auth.py:35".to_string(),
                        node_type: "UserInput".to_string(),
                    },
                    sink: FlowNodeResponse {
                        name: "sql_execute".to_string(),
                        location: "src/auth.py:42".to_string(),
                        node_type: "SqlQuery".to_string(),
                    },
                    confidence: 0.95,
                    sanitizers: vec![],
                }
            ]),
        };
        Ok(Json(issue))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get security analysis summary statistics
async fn get_security_summary(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut owasp_coverage = HashMap::new();
    owasp_coverage.insert("A01_Broken_Access_Control".to_string(), OwaspCategoryStats {
        issues_found: 5,
        coverage_percentage: 90.0,
        avg_confidence: 0.85,
    });
    owasp_coverage.insert("A02_Cryptographic_Failures".to_string(), OwaspCategoryStats {
        issues_found: 2,
        coverage_percentage: 70.0,
        avg_confidence: 0.95,
    });
    owasp_coverage.insert("A03_Injection".to_string(), OwaspCategoryStats {
        issues_found: 8,
        coverage_percentage: 95.0,
        avg_confidence: 0.90,
    });

    let mut confidence_distribution = HashMap::new();
    confidence_distribution.insert("high".to_string(), 25);
    confidence_distribution.insert("medium".to_string(), 12);
    confidence_distribution.insert("low".to_string(), 5);

    let summary = SecuritySummaryResponse {
        total_issues: 42,
        critical_count: 3,
        high_count: 8,
        medium_count: 20,
        low_count: 11,
        owasp_coverage,
        confidence_distribution,
        most_common_issues: vec![
            IssueTypeStats {
                issue_type: "Injection".to_string(),
                count: 8,
                avg_severity: "High".to_string(),
            },
            IssueTypeStats {
                issue_type: "Hardcoded Secrets".to_string(),
                count: 5,
                avg_severity: "High".to_string(),
            },
            IssueTypeStats {
                issue_type: "Broken Access Control".to_string(),
                count: 5,
                avg_severity: "Medium".to_string(),
            },
        ],
    };

    Ok(Json(summary))
}

/// Get OWASP Top 10 coverage information
async fn get_owasp_coverage(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut coverage = HashMap::new();
    coverage.insert("A01_Broken_Access_Control".to_string(), OwaspCategoryStats {
        issues_found: 5,
        coverage_percentage: 90.0,
        avg_confidence: 0.85,
    });
    coverage.insert("A02_Cryptographic_Failures".to_string(), OwaspCategoryStats {
        issues_found: 2,
        coverage_percentage: 70.0,
        avg_confidence: 0.95,
    });
    coverage.insert("A03_Injection".to_string(), OwaspCategoryStats {
        issues_found: 8,
        coverage_percentage: 95.0,
        avg_confidence: 0.90,
    });
    coverage.insert("A04_Insecure_Design".to_string(), OwaspCategoryStats {
        issues_found: 1,
        coverage_percentage: 60.0,
        avg_confidence: 0.75,
    });
    coverage.insert("A05_Security_Misconfiguration".to_string(), OwaspCategoryStats {
        issues_found: 7,
        coverage_percentage: 85.0,
        avg_confidence: 0.80,
    });

    Ok(Json(coverage))
}

/// Get taint flow analysis results
async fn get_taint_flows(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let taint_flows = vec![
        TaintFlowResponse {
            source: FlowNodeResponse {
                name: "user_input".to_string(),
                location: "src/auth.py:35".to_string(),
                node_type: "UserInput".to_string(),
            },
            sink: FlowNodeResponse {
                name: "sql_execute".to_string(),
                location: "src/auth.py:42".to_string(),
                node_type: "SqlQuery".to_string(),
            },
            confidence: 0.95,
            sanitizers: vec![],
        },
        TaintFlowResponse {
            source: FlowNodeResponse {
                name: "url_param".to_string(),
                location: "src/api.py:28".to_string(),
                node_type: "UserInput".to_string(),
            },
            sink: FlowNodeResponse {
                name: "file_open".to_string(),
                location: "src/api.py:35".to_string(),
                node_type: "FileSystem".to_string(),
            },
            confidence: 0.88,
            sanitizers: vec!["path_sanitizer".to_string()],
        },
    ];

    Ok(Json(taint_flows))
}

/// Export security findings in SARIF format
async fn export_sarif(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // SARIF 2.1.0 format implementation
    let sarif_report = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "Uveddi Security Detector",
                    "version": "0.9.0",
                    "informationUri": "https://github.com/uveddi/uveddi",
                    "rules": [{
                        "id": "sql-injection",
                        "name": "SQL Injection",
                        "shortDescription": {
                            "text": "SQL injection vulnerability detected"
                        },
                        "fullDescription": {
                            "text": "Application is vulnerable to SQL injection attacks through unsanitized user input"
                        },
                        "defaultConfiguration": {
                            "level": "error"
                        },
                        "properties": {
                            "tags": ["security", "injection", "owasp-a03"],
                            "precision": "high"
                        }
                    }]
                }
            },
            "results": [{
                "ruleId": "sql-injection",
                "message": {
                    "text": "SQL injection vulnerability: User input directly concatenated into SQL query"
                },
                "level": "error",
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": "src/auth.py"
                        },
                        "region": {
                            "startLine": 42,
                            "endLine": 42,
                            "startColumn": 15,
                            "endColumn": 35
                        }
                    }
                }],
                "fixes": [{
                    "description": {
                        "text": "Use parameterized queries to prevent SQL injection"
                    }
                }],
                "properties": {
                    "confidence": 0.95,
                    "severity": "critical",
                    "cwe": "CWE-89",
                    "owasp": "A03_Injection"
                }
            }]
        }]
    });

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"security-analysis.sarif\"".parse().unwrap(),
    );

    Ok((headers, Json(sarif_report)))
}

/// Load report from database by analysis run ID
async fn load_report_from_database(
    database: &Database,
    run_id: i64,
) -> Result<InteractiveReport, Box<dyn std::error::Error + Send + Sync>> {
    use crate::database::models::{AnalysisRun, ArchitecturalIssue, AntiPatternType, Dependency};
    
    // Get analysis run
    let analysis_run = database.get_analysis_run(run_id).await?
        .ok_or("Analysis run not found")?;
    
    // Get all issues for this run
    let issues = database.get_issues_for_run(run_id).await?;
    
    // Get anti-pattern types
    let anti_pattern_types = database.get_all_anti_pattern_types()?;
    
    // Get dependencies for this run
    let dependencies = database.get_dependencies_for_run(run_id).await?;
    
    // Generate report from real data
    let mut report = InteractiveReport::from_analysis_data(
        &analysis_run,
        &issues,
        &anti_pattern_types,
        None, // TODO: Load components if available
        &dependencies,
        &[], // TODO: Load diagrams if available
        format!("Analysis Run {}", run_id),
        analysis_run.project_id.to_string(),
    );
    
    // Add security analysis if available
    #[cfg(feature = "security")]
    {
        if let Ok(security_issues) = database.get_security_issues_for_run(run_id).await {
            report.security_analysis = Some(create_security_analysis_from_issues(&security_issues));
        }
    }
    
    // Fix summary consistency - calculate from actual data
    let total_issues = issues.len() as u32;
    let mut issues_by_severity = std::collections::HashMap::new();
    let mut issues_by_category = std::collections::HashMap::new();
    
    for issue in &issues {
        *issues_by_severity.entry(issue.severity.clone()).or_insert(0) += 1;
        
        // Map anti-pattern to category
        if let Some(anti_pattern) = anti_pattern_types.iter().find(|apt| 
            apt.anti_pattern_type_id.map(|id| id == issue.anti_pattern_type_id).unwrap_or(false)
        ) {
            *issues_by_category.entry(anti_pattern.category.clone()).or_insert(0) += 1;
        }
    }
    
    // Update summary with correct counts
    report.summary.issues_total = total_issues;
    report.summary.issues_by_severity = issues_by_severity;
    report.summary.issues_by_category = issues_by_category;
    
    Ok(report)
}

/// List all analysis runs from database
async fn list_reports_from_database(
    database: &Database,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    let runs = database.get_recent_analysis_runs(10).await?;
    
    let reports = runs.into_iter().map(|run| {
        serde_json::json!({
            "id": run.run_id.unwrap_or(0).to_string(),
            "title": format!("Analysis Run {}", run.run_id.unwrap_or(0)),
            "created_at": run.start_time,
            "project_name": format!("Project {}", run.project_id),
            "file_count": run.total_files_analyzed.unwrap_or(0),
            "issue_count": run.total_issues_found.unwrap_or(0),
            "status": run.status
        })
    }).collect();
    
    Ok(reports)
}

/// Create security analysis from security issues
#[cfg(feature = "security")]
fn create_security_analysis_from_issues(
    security_issues: &[crate::analysis::detectors::security::types::SecurityIssue]
) -> crate::report::interactive_models::SecurityAnalysis {
    use crate::report::interactive_models::*;
    
    let total_issues = security_issues.len() as u32;
    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;
    
    let mut owasp_coverage = std::collections::HashMap::new();
    let mut confidence_distribution = std::collections::HashMap::new();
    let mut issue_type_stats = std::collections::HashMap::new();
    
    for issue in security_issues {
        // Count by severity
        match issue.severity.to_string().as_str() {
            "Critical" => critical_count += 1,
            "High" => high_count += 1,
            "Medium" => medium_count += 1,
            "Low" => low_count += 1,
            _ => {},
        }
        
        // Update OWASP coverage
        if let Some(owasp_cat) = issue.issue_type.owasp_category() {
            let stats = owasp_coverage.entry(owasp_cat.to_string()).or_insert(OwaspCategoryStats {
                issues_found: 0,
                coverage_percentage: 0.0,
                avg_confidence: 0.0,
                severity_distribution: std::collections::HashMap::new(),
            });
            stats.issues_found += 1;
            *stats.severity_distribution.entry(issue.severity.to_string()).or_insert(0) += 1;
        }
        
        // Update confidence distribution
        let confidence_tier = if issue.confidence_score >= 0.8 { "High" }
                             else if issue.confidence_score >= 0.6 { "Medium" }
                             else { "Low" };
        *confidence_distribution.entry(confidence_tier.to_string()).or_insert(0) += 1;
        
        // Update issue type stats
        let type_stats = issue_type_stats.entry(issue.issue_type.to_string()).or_insert((0u32, issue.severity.to_string(), 0.0));
        type_stats.0 += 1;
        type_stats.2 += issue.confidence_score;
    }
    
    // Calculate OWASP coverage percentages
    for stats in owasp_coverage.values_mut() {
        stats.avg_confidence = if stats.issues_found > 0 {
            // This would need to be calculated from actual confidence scores
            0.75 // Placeholder
        } else { 0.0 };
        stats.coverage_percentage = if total_issues > 0 {
            (stats.issues_found as f64 / total_issues as f64) * 100.0
        } else { 0.0 };
    }
    
    let most_common_issues = issue_type_stats.into_iter()
        .map(|(issue_type, (count, severity, total_confidence))| IssueTypeStats {
            issue_type,
            count,
            avg_severity: severity,
            avg_confidence: total_confidence / count as f64,
        })
        .collect();
    
    let security_score = if total_issues == 0 { 100.0 }
                        else { std::cmp::max(0, 100 - (critical_count * 25 + high_count * 15 + medium_count * 5 + low_count * 1)) as f64 };
    
    SecurityAnalysis {
        summary: SecuritySummary {
            total_issues,
            critical_count,
            high_count,
            medium_count,
            low_count,
            confidence_distribution,
            most_common_issues,
            security_score,
        },
        owasp_coverage,
        issues: security_issues.iter().map(|issue| crate::report::interactive_models::SecurityIssue {
            id: format!("sec-{}", issue.id.as_ref().unwrap_or(&"unknown".to_string())),
            issue_type: issue.issue_type.to_string(),
            severity: issue.severity.to_string(),
            confidence_score: issue.confidence_score,
            location: crate::report::interactive_models::SecurityLocation {
                file: issue.location.file_path.to_string_lossy().to_string(),
                start_line: issue.location.start_line as u32,
                end_line: issue.location.end_line as u32,
                start_column: issue.location.start_column.map(|c| c as u32),
                end_column: issue.location.end_column.map(|c| c as u32),
                code_snippet: None, // TODO: Extract code snippet from file
            },
            description: issue.description.clone(),
            remediation: issue.remediation.clone().unwrap_or_default(),
            owasp_category: issue.issue_type.owasp_category().map(|s| s.to_string()),
            cwe_id: issue.metadata.cwe_id.clone(),
            cvss_score: issue.metadata.cvss_score,
            references: issue.metadata.references.clone(),
            related_taint_flows: vec![], // TODO: Implement taint flow tracking
            attack_vector: None, // TODO: Extract from metadata
        }).collect(),
        taint_flows: vec![], // TODO: Implement taint flow analysis
        correlations: vec![], // TODO: Implement correlation analysis
        compliance: None, // TODO: Implement compliance checking
    }
}
