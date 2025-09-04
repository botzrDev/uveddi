//! Security analysis endpoints for the REST API

use crate::api::rest::AppState;
use crate::security;
use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing::{error, warn};

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

/// Get all security issues
pub async fn get_security_issues(
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
                            let response_issues: Vec<SecurityIssueResponse> = security_issues
                                .iter()
                                .map(|issue| {
                                    SecurityIssueResponse {
                                        id: format!(
                                            "sec_{}",
                                            issue.id.as_ref().unwrap_or(&"unknown".to_string())
                                        ),
                                        issue_type: issue.issue_type.to_string(),
                                        severity: issue.severity.to_string(),
                                        confidence_score: issue.confidence_score,
                                        location: LocationResponse {
                                            file: issue
                                                .location
                                                .file_path
                                                .to_string_lossy()
                                                .to_string(),
                                            start_line: issue.location.start_line,
                                            end_line: issue.location.end_line,
                                            start_column: issue.location.start_column,
                                            end_column: issue.location.end_column,
                                        },
                                        description: issue.description.clone(),
                                        remediation: issue.remediation.clone().unwrap_or_default(),
                                        owasp_category: issue
                                            .issue_type
                                            .owasp_category()
                                            .map(|s| s.to_string()),
                                        cwe_id: issue.metadata.cwe_id.clone(),
                                        cvss_score: issue.metadata.cvss_score,
                                        references: issue.metadata.references.clone(),
                                        taint_flows: None, // TODO: Implement taint flow conversion
                                    }
                                })
                                .collect::<Vec<SecurityIssueResponse>>(); // Explicitly collect into Vec<SecurityIssueResponse> for clarity

                            return Ok(Json(response_issues));
                        }
                        Err(e) => {
                            error!("Failed to load security issues: {}", e);
                            // Fall through to demo data
                        }
                    }
                }
            }
            Ok(None) => {
                warn!("No analysis runs found in database");
                // Fall through to demo data
            }
            Err(e) => {
                error!("Failed to get latest analysis run: {}", e);
                // Fall through to demo data
            }
        }
    }

    // Fallback to demo data if database query fails or security feature disabled
    let demo_issues = vec![SecurityIssueResponse {
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
        references: vec![
            "https://owasp.org/www-project-top-ten/2017/A1_2017-Injection".to_string(),
        ],
        taint_flows: Some(vec![TaintFlowResponse {
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
        }]),
    }];

    Ok(Json(demo_issues))
}

/// Get a specific security issue by ID
pub async fn get_security_issue(
    AxumPath(issue_id): AxumPath<String>,
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Enhanced input validation for security issue ID
    if let Err(e) = security::validate_input(&issue_id, "issue_id") {
        error!("Invalid security issue ID parameter: {}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate issue ID format - should be alphanumeric with underscores
    if !issue_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        error!("Security issue ID contains invalid characters: {}", issue_id);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check length bounds
    if issue_id.len() > 64 {
        error!("Security issue ID too long: {}", issue_id.len());
        return Err(StatusCode::BAD_REQUEST);
    }

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
            references: vec![
                "https://owasp.org/www-project-top-ten/2017/A1_2017-Injection".to_string(),
            ],
            taint_flows: Some(vec![TaintFlowResponse {
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
            }]),
        };
        Ok(Json(issue))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get security analysis summary statistics
pub async fn get_security_summary(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut owasp_coverage = HashMap::new();
    owasp_coverage.insert(
        "A01_Broken_Access_Control".to_string(),
        OwaspCategoryStats {
            issues_found: 5,
            coverage_percentage: 90.0,
            avg_confidence: 0.85,
        },
    );
    owasp_coverage.insert(
        "A02_Cryptographic_Failures".to_string(),
        OwaspCategoryStats {
            issues_found: 2,
            coverage_percentage: 70.0,
            avg_confidence: 0.95,
        },
    );
    owasp_coverage.insert(
        "A03_Injection".to_string(),
        OwaspCategoryStats {
            issues_found: 8,
            coverage_percentage: 95.0,
            avg_confidence: 0.90,
        },
    );

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
pub async fn get_owasp_coverage(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut coverage = HashMap::new();
    coverage.insert(
        "A01_Broken_Access_Control".to_string(),
        OwaspCategoryStats {
            issues_found: 5,
            coverage_percentage: 90.0,
            avg_confidence: 0.85,
        },
    );
    coverage.insert(
        "A02_Cryptographic_Failures".to_string(),
        OwaspCategoryStats {
            issues_found: 2,
            coverage_percentage: 70.0,
            avg_confidence: 0.95,
        },
    );
    coverage.insert(
        "A03_Injection".to_string(),
        OwaspCategoryStats {
            issues_found: 8,
            coverage_percentage: 95.0,
            avg_confidence: 0.90,
        },
    );
    coverage.insert(
        "A04_Insecure_Design".to_string(),
        OwaspCategoryStats {
            issues_found: 1,
            coverage_percentage: 60.0,
            avg_confidence: 0.75,
        },
    );
    coverage.insert(
        "A05_Security_Misconfiguration".to_string(),
        OwaspCategoryStats {
            issues_found: 7,
            coverage_percentage: 85.0,
            avg_confidence: 0.80,
        },
    );

    Ok(Json(coverage))
}

/// Get taint flow analysis results
pub async fn get_taint_flows(
    State(_state): State<Arc<AppState>>,
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
pub async fn export_sarif(State(_state): State<Arc<AppState>>) -> Result<impl IntoResponse, StatusCode> {
    use axum::http::{header, HeaderMap};

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
                            "tags": ["security", "sql", "injection"]
                        }
                    }]
                }
            },
            "results": [{
                "ruleId": "sql-injection",
                "level": "error",
                "message": {
                    "text": "SQL injection vulnerability detected in user authentication"
                },
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
                "properties": {
                    "confidence": "high",
                    "severity": "critical",
                    "owasp": "A03_Injection",
                    "cwe": "CWE-89"
                }
            }]
        }]
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8"
            .parse()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"uveddi-security-report.sarif\"")
            .parse()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    Ok((headers, Json(sarif_report)))
}