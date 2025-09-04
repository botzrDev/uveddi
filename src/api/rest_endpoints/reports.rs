//! Report-related endpoints for the REST API

use crate::api::rest::AppState;
use crate::database::Database;
use crate::report::interactive_models::*;
use crate::{security, error::UveddiError};
use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::fs;
use tracing::{error, warn};

/// List all available reports
pub async fn list_reports(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, StatusCode> {
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
            error!("Failed to load reports from database: {}", e);
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
pub async fn get_report(
    AxumPath(report_id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Enhanced input validation for report ID
    if let Err(e) = security::validate_input(&report_id, "report_id") {
        error!("Invalid report ID parameter: {}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate report ID format - must be "demo" or numeric ID
    if report_id != "demo" {
        if let Err(_) = report_id.parse::<i64>() {
            error!("Report ID must be numeric or 'demo': {}", report_id);
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Additional numeric validation for bounds checking
        if let Ok(id_num) = report_id.parse::<i64>() {
            if id_num < 0 || id_num > i32::MAX as i64 {
                error!("Report ID out of valid range: {}", id_num);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // For demo purposes, handle "demo" specially (fallback only)
    if report_id == "demo" {
        return Ok(Json(create_demo_report()));
    }

    // Try to load from database first (production approach)
    if let Ok(run_id) = report_id.parse::<i64>() {
        match load_report_from_database(&state.database, run_id).await {
            Ok(report) => return Ok(Json(report)),
            Err(e) => {
                error!("Failed to load report from database: {}", e);
                // Fall through to file system
            }
        }
    }

    // Fallback to file system storage
    let report_path = state
        .config
        .reports_storage_path
        .join(format!("{}.json", report_id));

    // TODO: Implement file system storage
    Err(StatusCode::NOT_FOUND)
}

/// Get dependency graph for a specific report
pub async fn get_dependency_graph(
    AxumPath(report_id): AxumPath<String>,
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Enhanced input validation for report ID
    if let Err(e) = security::validate_input(&report_id, "report_id") {
        error!("Invalid report ID parameter in dependency graph request: {}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate report ID format
    if report_id != "demo" {
        if let Err(_) = report_id.parse::<i64>() {
            error!("Report ID must be numeric or 'demo' for dependency graph: {}", report_id);
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Additional numeric validation
        if let Ok(id_num) = report_id.parse::<i64>() {
            if id_num < 0 || id_num > i32::MAX as i64 {
                error!("Report ID out of valid range for dependency graph: {}", id_num);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // For demo purposes, return demo dependency graph
    if report_id == "demo" {
        let demo_report = create_demo_report();
        return Ok(Json(demo_report.dependency_graph));
    }

    // TODO: Implement database-backed dependency graph retrieval
    Err(StatusCode::NOT_FOUND)
}

/// Demo report handler
pub async fn demo_report_handler() -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(create_demo_report()))
}

/// Load report from database by analysis run ID
async fn load_report_from_database(
    database: &Database,
    run_id: i64,
) -> Result<InteractiveReport, Box<dyn std::error::Error + Send + Sync>> {
    use crate::database::models::AntiPatternType;
    use crate::report::data_transformer::DataTransformer;

    // Get analysis run
    let analysis_run = database
        .get_analysis_run(run_id)
        .await?
        .ok_or("Analysis run not found")?;

    // Get all issues for this run
    let issues = database.get_issues_for_run(run_id).await?;

    // Get anti-pattern types
    let anti_pattern_types = database.get_all_anti_pattern_types()?;

    // Build HashMap for anti-pattern types
    let anti_pattern_map: std::collections::HashMap<i64, AntiPatternType> = anti_pattern_types
        .into_iter()
        .filter_map(|apt| apt.anti_pattern_type_id.map(|id| (id, apt)))
        .collect();

    // Get dependencies for this run
    let _dependencies = database.get_dependencies_for_run(run_id).await?;

    // Use the DataTransformer to create properly formatted report
    let mut report = DataTransformer::transform_to_interactive_report(
        &analysis_run,
        &issues,
        &anti_pattern_map,
        format!("Project {}", analysis_run.project_id),
        ".".to_string(), // TODO: Get actual project path from analysis run
    );

    // Add security analysis if available
    #[cfg(feature = "security")]
    {
        if let Ok(security_issues) = database.get_security_issues_for_run(run_id).await {
            report.security_analysis = Some(create_security_analysis_from_issues(&security_issues));
        }
    }

    Ok(report)
}

/// List all analysis runs from database
async fn list_reports_from_database(
    database: &Database,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    let runs = database.get_recent_analysis_runs(10).await?;

    let reports: Vec<serde_json::Value> = runs
        .into_iter()
        .map(|run| {
            serde_json::json!({
                "id": run.run_id.unwrap_or(0).to_string(),
                "title": format!("Analysis Run {}", run.run_id.unwrap_or(0)),
                "created_at": run.start_time,
                "project_name": format!("Project {}", run.project_id),
                "file_count": run.total_files_analyzed.unwrap_or(0),
                "issue_count": run.total_issues_found.unwrap_or(0),
                "status": run.status
            })
        })
        .collect::<Vec<serde_json::Value>>(); // Explicit collection type for readability

    Ok(reports)
}

/// Create security analysis from security issues
#[cfg(feature = "security")]
fn create_security_analysis_from_issues(
    security_issues: &[crate::analysis::detectors::security::types::SecurityIssue],
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
            "critical" => critical_count += 1,
            "high" => high_count += 1,
            "medium" => medium_count += 1,
            "low" => low_count += 1,
            _ => {}
        }

        // Update OWASP coverage stats
        let owasp_key = format!("{:?}", issue.owasp_category);
        let entry = owasp_coverage.entry(owasp_key).or_insert(OwaspCategoryStats {
            issues_found: 0,
            coverage_percentage: 0.0,
            avg_confidence: 0.0,
            severity_distribution: std::collections::HashMap::new(),
        });
        entry.issues_found += 1;

        // Update confidence distribution
        let confidence_bucket = ((issue.confidence * 10.0).floor() / 10.0 * 100.0) as u32;
        *confidence_distribution.entry(confidence_bucket).or_insert(0) += 1;

        // Update issue type stats
        let issue_type = format!("{:?}", issue.issue_type);
        *issue_type_stats.entry(issue_type).or_insert(0) += 1;
    }

    let mut severity_distribution = std::collections::HashMap::new();
    severity_distribution.insert("critical".to_string(), critical_count);
    severity_distribution.insert("high".to_string(), high_count);
    severity_distribution.insert("medium".to_string(), medium_count);
    severity_distribution.insert("low".to_string(), low_count);

    SecurityAnalysis {
        summary: SecuritySummary {
            total_issues,
            severity_distribution,
            confidence_distribution,
            owasp_coverage: owasp_coverage.clone(),
            issue_type_stats,
        },
        issues: security_issues.iter().map(|issue| {
            SecurityIssue {
                id: issue.id.clone(),
                issue_type: format!("{:?}", issue.issue_type),
                severity: issue.severity.to_string(),
                title: issue.title.clone(),
                description: issue.description.clone(),
                file_path: issue.location.file_path.clone(),
                start_line: issue.location.line_start,
                end_line: issue.location.line_end,
                column_start: issue.location.column_start,
                column_end: issue.location.column_end,
                owasp_category: format!("{:?}", issue.owasp_category),
                confidence: issue.confidence,
                recommendation: issue.remediation.clone(),
                references: issue.references.clone(),
                taint_flow: issue.taint_flow.as_ref().map(|tf| TaintFlow {
                    source: FlowNode {
                        location: LocationData {
                            file: tf.source.file_path.clone(),
                            line: tf.source.line_start,
                            column: tf.source.column_start,
                        },
                        value: tf.source.snippet.clone().unwrap_or_default(),
                        taint_type: "source".to_string(),
                    },
                    sink: FlowNode {
                        location: LocationData {
                            file: tf.sink.file_path.clone(),
                            line: tf.sink.line_start,
                            column: tf.sink.column_start,
                        },
                        value: tf.sink.snippet.clone().unwrap_or_default(),
                        taint_type: "sink".to_string(),
                    },
                    path: tf.path.iter().map(|step| FlowNode {
                        location: LocationData {
                            file: step.file_path.clone(),
                            line: step.line_start,
                            column: step.column_start,
                        },
                        value: step.snippet.clone().unwrap_or_default(),
                        taint_type: "intermediate".to_string(),
                    }).collect(),
                }),
            }
        }).collect(),
        owasp_coverage,
        taint_flows: Vec::new(), // TODO: Implement if needed
    }
}

/// Create a demo report for testing and development
pub fn create_demo_report() -> InteractiveReport {
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
    owasp_coverage.insert(
        "A01_Broken_Access_Control".to_string(),
        OwaspCategoryStats {
            issues_found: 2,
            coverage_percentage: 85.0,
            avg_confidence: 0.75,
            severity_distribution: std::collections::HashMap::new(),
        },
    );
    owasp_coverage.insert(
        "A03_Injection".to_string(),
        OwaspCategoryStats {
            issues_found: 1,
            coverage_percentage: 90.0,
            avg_confidence: 0.92,
            severity_distribution: std::collections::HashMap::new(),
        },
    );

    SecurityAnalysis {
        summary: SecuritySummary {
            total_issues: 3,
            severity_distribution: {
                let mut dist = std::collections::HashMap::new();
                dist.insert("high".to_string(), 2);
                dist.insert("medium".to_string(), 1);
                dist
            },
            confidence_distribution: {
                let mut dist = std::collections::HashMap::new();
                dist.insert(80, 2);
                dist.insert(90, 1);
                dist
            },
            owasp_coverage: owasp_coverage.clone(),
            issue_type_stats: {
                let mut stats = std::collections::HashMap::new();
                stats.insert("injection".to_string(), 1);
                stats.insert("access_control".to_string(), 2);
                stats
            },
        },
        issues: vec![
            SecurityIssue {
                id: "sec-001".to_string(),
                issue_type: "SQLInjection".to_string(),
                severity: "high".to_string(),
                title: "SQL Injection Vulnerability".to_string(),
                description: "User input is directly concatenated into SQL queries".to_string(),
                file_path: "src/database.rs".to_string(),
                start_line: 67,
                end_line: 67,
                column_start: Some(15),
                column_end: Some(50),
                owasp_category: "A03_Injection".to_string(),
                confidence: 0.94,
                recommendation: Some("Use parameterized queries".to_string()),
                references: vec!["https://owasp.org/Top10/A03_2021-Injection/".to_string()],
                taint_flow: None,
            },
        ],
        owasp_coverage,
        taint_flows: vec![],
    }
}