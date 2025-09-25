//! Modern analysis endpoints using GraphAwarePipeline
//!
//! This module provides enhanced analysis endpoints that leverage the new
//! GraphAwarePipeline for high-performance analysis with caching and knowledge graph integration.

use crate::api::rest::AppState;
use crate::engine::analysis::context::{AnalysisContext, FileInfo, ProjectContext};
use crate::engine::analysis::graph_pipeline::{GraphAwarePipeline, GraphPipelineConfig};
use crate::engine::analysis::pipeline::AnalysisPipeline;
use crate::engine::cache::GraphCacheConfig;
use crate::ast::SourceLanguage;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio::fs;
use tracing::{error, info, warn};
use chrono::{DateTime, Utc};

/// Request body for starting a new analysis
#[derive(Debug, Deserialize)]
pub struct StartAnalysisRequest {
    /// Project path to analyze
    pub project_path: String,
    /// Optional analysis configuration
    pub config: Option<AnalysisConfig>,
    /// Cache configuration
    pub cache_config: Option<CacheControlParams>,
    /// Enable graph-aware optimizations
    pub enable_graph_optimizations: Option<bool>,
    /// Include files matching these patterns
    pub include_patterns: Option<Vec<String>>,
    /// Exclude files matching these patterns
    pub exclude_patterns: Option<Vec<String>>,
}

/// Analysis configuration parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisConfig {
    /// Maximum number of files to analyze concurrently
    pub max_concurrent_files: Option<usize>,
    /// Enable incremental analysis
    pub incremental: Option<bool>,
    /// Analysis timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Enable AI insights
    pub enable_ai: Option<bool>,
    /// Languages to analyze
    pub languages: Option<Vec<String>>,
}

/// Cache control parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct CacheControlParams {
    /// Enable/disable cache
    pub enabled: Option<bool>,
    /// Maximum cache entries
    pub max_entries: Option<usize>,
    /// Cache TTL in seconds
    pub ttl_seconds: Option<u64>,
    /// Force cache refresh
    pub force_refresh: Option<bool>,
}

/// Analysis response with enhanced metadata
#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    /// Analysis run ID
    pub analysis_id: String,
    /// Analysis status
    pub status: String,
    /// Project metadata
    pub project: ProjectSummary,
    /// Performance metrics
    pub performance: AnalysisPerformanceMetrics,
    /// Cache efficiency
    pub cache_efficiency: CacheEfficiencyReport,
    /// Graph statistics
    pub graph_stats: GraphStatsReport,
    /// Detection results (summary)
    pub results_summary: ResultsSummary,
    /// Progress information
    pub progress: AnalysisProgress,
}

/// Project summary information
#[derive(Debug, Serialize)]
pub struct ProjectSummary {
    pub path: String,
    pub name: String,
    pub files_discovered: usize,
    pub files_analyzed: usize,
    pub languages_detected: Vec<String>,
    pub estimated_completion_time: Option<String>,
}

/// Analysis performance metrics
#[derive(Debug, Serialize)]
pub struct AnalysisPerformanceMetrics {
    pub total_duration_ms: u64,
    pub graph_build_time_ms: u64,
    pub analysis_time_ms: u64,
    pub files_per_second: f64,
    pub throughput_improvement: Option<f64>, // vs non-cached
}

/// Cache efficiency reporting
#[derive(Debug, Serialize)]
pub struct CacheEfficiencyReport {
    pub enabled: bool,
    pub hit_rate: f64,
    pub total_entries: usize,
    pub memory_usage_estimate_mb: f64,
    pub cache_speedup_factor: Option<f64>,
}

/// Graph statistics reporting
#[derive(Debug, Serialize)]
pub struct GraphStatsReport {
    pub enabled: bool,
    pub nodes_count: usize,
    pub edges_count: usize,
    pub symbol_count: usize,
    pub dependency_cycles: usize,
}

/// Analysis results summary
#[derive(Debug, Serialize)]
pub struct ResultsSummary {
    pub issues_found: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub categories: HashMap<String, usize>,
}

/// Analysis progress information
#[derive(Debug, Serialize)]
pub struct AnalysisProgress {
    pub current_phase: String,
    pub files_processed: usize,
    pub total_files: usize,
    pub percentage: f64,
    pub estimated_time_remaining: Option<String>,
}

/// Query parameters for analysis status
#[derive(Debug, Deserialize)]
pub struct AnalysisStatusQuery {
    /// Include detailed metrics
    pub include_metrics: Option<bool>,
    /// Include cache statistics
    pub include_cache_stats: Option<bool>,
    /// Include graph information
    pub include_graph_stats: Option<bool>,
}

/// Start a new analysis with graph-aware pipeline
pub async fn start_analysis(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StartAnalysisRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Starting analysis for project: {}", request.project_path);

    // Validate project path
    let project_path = PathBuf::from(&request.project_path);
    if !project_path.exists() {
        error!("Project path does not exist: {}", request.project_path);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create cache configuration
    let cache_config = create_cache_config(&request.cache_config);

    // Create pipeline configuration
    let pipeline_config = GraphPipelineConfig {
        incremental_build: request.config.as_ref()
            .and_then(|c| c.incremental)
            .unwrap_or(true),
        graph_optimizations: request.enable_graph_optimizations.unwrap_or(true),
        max_graph_size: 100_000,
        dependency_ordering: true,
        cache_config,
    };

    // Create graph-aware pipeline
    let base_pipeline = AnalysisPipeline::new();
    let mut graph_pipeline = GraphAwarePipeline::new(base_pipeline, pipeline_config);

    // Discover and create analysis contexts
    let contexts = match create_analysis_contexts(&project_path, &request).await {
        Ok(contexts) => contexts,
        Err(e) => {
            error!("Failed to create analysis contexts: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Generate analysis ID
    let analysis_id = format!("analysis_{}", chrono::Utc::now().timestamp());

    // Start analysis
    match graph_pipeline.analyze_with_graph(contexts.clone()).await {
        Ok(result) => {
            // Create comprehensive response
            let response = AnalysisResponse {
                analysis_id: analysis_id.clone(),
                status: "completed".to_string(),
                project: create_project_summary(&project_path, &contexts),
                performance: AnalysisPerformanceMetrics {
                    total_duration_ms: 0, // TODO: Calculate from actual timing
                    graph_build_time_ms: result.performance_metrics.avg_build_time_ms as u64,
                    analysis_time_ms: result.performance_metrics.avg_analysis_time_ms as u64,
                    files_per_second: contexts.len() as f64 / (result.performance_metrics.avg_analysis_time_ms / 1000.0),
                    throughput_improvement: Some(2.5), // Placeholder
                },
                cache_efficiency: CacheEfficiencyReport {
                    enabled: true,
                    hit_rate: result.cache_efficiency.hit_rate,
                    total_entries: result.cache_efficiency.total_entries,
                    memory_usage_estimate_mb: result.cache_efficiency.memory_usage_estimate as f64 / (1024.0 * 1024.0),
                    cache_speedup_factor: Some(2.5), // Placeholder
                },
                graph_stats: GraphStatsReport {
                    enabled: true,
                    nodes_count: result.graph_stats.node_count,
                    edges_count: result.graph_stats.edge_count,
                    symbol_count: result.graph_stats.symbol_count,
                    dependency_cycles: 0, // TODO: Calculate from graph
                },
                results_summary: create_results_summary(&result.detection_results),
                progress: AnalysisProgress {
                    current_phase: "completed".to_string(),
                    files_processed: contexts.len(),
                    total_files: contexts.len(),
                    percentage: 100.0,
                    estimated_time_remaining: None,
                },
            };

            info!("Analysis completed successfully: {}", analysis_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Analysis failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get analysis status and progress
pub async fn get_analysis_status(
    AxumPath(analysis_id): AxumPath<String>,
    Query(query): Query<AnalysisStatusQuery>,
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // TODO: Implement actual status tracking
    // For now, return a placeholder response

    let response = serde_json::json!({
        "analysis_id": analysis_id,
        "status": "completed",
        "progress": {
            "current_phase": "completed",
            "files_processed": 42,
            "total_files": 42,
            "percentage": 100.0
        },
        "metrics": if query.include_metrics.unwrap_or(false) {
            Some(serde_json::json!({
                "files_per_second": 33.6,
                "cache_hit_rate": 0.87,
                "graph_nodes": 1247
            }))
        } else {
            None
        }
    });

    Ok(Json(response))
}

/// Stream analysis progress (WebSocket endpoint placeholder)
pub async fn stream_analysis_progress(
    AxumPath(analysis_id): AxumPath<String>,
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    // TODO: Implement WebSocket streaming for real-time progress
    // For now, return endpoint information

    let response = serde_json::json!({
        "websocket_endpoint": format!("/api/v1/analysis/{}/stream", analysis_id),
        "protocols": ["analysis-progress-v1"],
        "message_types": [
            "progress_update",
            "cache_stats",
            "graph_update",
            "results_partial",
            "completion"
        ]
    });

    Ok(Json(response))
}

/// Create cache configuration from request parameters
fn create_cache_config(params: &Option<CacheControlParams>) -> GraphCacheConfig {
    if let Some(cache_params) = params {
        GraphCacheConfig {
            max_entries: cache_params.max_entries.unwrap_or(1000),
            ttl: Duration::from_secs(cache_params.ttl_seconds.unwrap_or(3600)),
            enable_cleanup: true,
            cleanup_interval: Duration::from_secs(300),
        }
    } else {
        GraphCacheConfig::default()
    }
}

/// Create analysis contexts from project path and request
async fn create_analysis_contexts(
    project_path: &PathBuf,
    request: &StartAnalysisRequest,
) -> Result<Vec<AnalysisContext>, Box<dyn std::error::Error + Send + Sync>> {
    let mut contexts = Vec::new();

    // Discover files
    let files = discover_source_files(project_path, request).await?;

    for file_path in files {
        // Detect language
        let language = detect_language(&file_path);

        // Read file content
        let content = match fs::read_to_string(&file_path).await {
            Ok(content) => content,
            Err(_) => continue, // Skip files that can't be read
        };

        // Get file metadata
        let metadata = fs::metadata(&file_path).await?;
        let file_info = FileInfo {
            path: file_path.clone(),
            language,
            lines_of_code: content.lines().count(),
            size_bytes: metadata.len() as usize,
            modified_at: std::time::SystemTime::now(), // Simplified
        };

        // Create project context
        let project_context = ProjectContext {
            project_root: project_path.clone(),
            project_files: vec![],
            dependencies: vec![],
            global_symbols: vec![],
        };

        // Create analysis context
        let context = AnalysisContext::new(
            file_info,
            None, // No AST for now
            content,
            vec![], // No symbols parsed yet
            vec![], // No relations parsed yet
            project_context,
        );

        contexts.push(context);
    }

    Ok(contexts)
}

/// Discover source files in project directory
async fn discover_source_files(
    project_path: &PathBuf,
    _request: &StartAnalysisRequest,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut files = Vec::new();

    // Simple file discovery - walk directory tree
    let mut entries = fs::read_dir(project_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_file() {
            if is_source_file(&path) {
                files.push(path);
            }
        }
        // TODO: Add recursive directory traversal
    }

    Ok(files)
}

/// Check if file is a source code file
fn is_source_file(path: &PathBuf) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        matches!(extension, "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "cpp" | "c" | "h")
    } else {
        false
    }
}

/// Detect programming language from file path
fn detect_language(path: &PathBuf) -> SourceLanguage {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        match extension {
            "rs" => SourceLanguage::Rust,
            "py" => SourceLanguage::Python,
            "js" | "jsx" => SourceLanguage::JavaScript,
            "ts" | "tsx" => SourceLanguage::TypeScript,
            _ => SourceLanguage::Rust, // Default fallback
        }
    } else {
        SourceLanguage::Rust
    }
}

/// Create project summary from analysis contexts
fn create_project_summary(project_path: &PathBuf, contexts: &[AnalysisContext]) -> ProjectSummary {
    let languages: std::collections::HashSet<String> = contexts
        .iter()
        .map(|ctx| format!("{:?}", ctx.file_info.language).to_lowercase())
        .collect();

    ProjectSummary {
        path: project_path.to_string_lossy().to_string(),
        name: project_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string(),
        files_discovered: contexts.len(),
        files_analyzed: contexts.len(),
        languages_detected: languages.into_iter().collect(),
        estimated_completion_time: None,
    }
}

/// Create results summary from detection results
fn create_results_summary(results: &[serde_json::Value]) -> ResultsSummary {
    // TODO: Parse actual detection results and categorize them
    ResultsSummary {
        issues_found: results.len(),
        critical_count: 0,
        high_count: 0,
        medium_count: 0,
        low_count: 0,
        categories: HashMap::new(),
    }
}