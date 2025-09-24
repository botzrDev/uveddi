//! API endpoints organized by domain
//!
//! This module contains the refactored endpoint implementations, split by domain:
//! - health: Health check endpoints
//! - reports: Report and analysis endpoints
//! - security: Security analysis endpoints
//! - projects: Project management endpoints (using repository pattern)

pub mod health;
pub mod reports;
pub mod security;
pub mod projects;

// Re-export all endpoint functions for easy access
pub use health::health_check;
pub use reports::{demo_report_handler, get_dependency_graph, get_report, list_reports};
pub use security::{
    export_sarif, get_owasp_coverage, get_security_issue, get_security_issues,
    get_security_summary, get_taint_flows,
};
pub use projects::{
    list_projects_repository, get_project_repository, create_project_repository,
};
