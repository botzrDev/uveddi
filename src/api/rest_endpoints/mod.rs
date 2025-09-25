//! API endpoints organized by domain
//!
//! This module contains the refactored endpoint implementations, split by domain:
//! - analysis: Modern analysis endpoints with GraphAwarePipeline
//! - cache: Cache management and monitoring endpoints
//! - health: Health check endpoints
//! - reports: Report and analysis endpoints
//! - security: Security analysis endpoints
//! - projects: Project management endpoints (using repository pattern)

pub mod analysis;
pub mod cache;
pub mod health;
pub mod knowledge_graph;
pub mod projects;
pub mod reports;
pub mod security;
pub mod streaming;

// Re-export all endpoint functions for easy access
pub use analysis::{get_analysis_status, start_analysis, stream_analysis_progress};
pub use cache::{
    control_cache, get_cache_efficiency, get_cache_health, get_cache_stats, warmup_cache,
};
pub use health::health_check;
pub use knowledge_graph::{
    execute_graph_query, get_graph_analytics, get_graph_visualization, update_graph_incremental,
};
pub use projects::{create_project_repository, get_project_repository, list_projects_repository};
pub use reports::{demo_report_handler, get_dependency_graph, get_report, list_reports};
pub use security::{
    export_sarif, get_owasp_coverage, get_security_issue, get_security_issues,
    get_security_summary, get_taint_flows,
};
pub use streaming::{
    get_streaming_info, stream_analysis_progress_ws, stream_cache_stats_ws, stream_events_ws,
};
