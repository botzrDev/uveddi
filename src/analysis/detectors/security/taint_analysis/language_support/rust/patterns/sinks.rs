//! Rust-specific taint sink patterns

use crate::analysis::detectors::security::taint_analysis::types::TaintSink;
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::ast::SourceLanguage;

/// Get Rust-specific taint sinks
pub fn get_rust_sinks() -> Vec<TaintSink> {
    vec![
        // SQL injection
        TaintSink::new(
            "rust_sqlx_query_raw".to_string(),
            "sqlx::query".to_string(),
            SecurityIssueType::Injection,
            "Raw SQL query execution".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSink::new(
            "rust_diesel_sql".to_string(),
            "diesel::sql_query".to_string(),
            SecurityIssueType::Injection,
            "Diesel raw SQL query".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // Command injection
        TaintSink::new(
            "rust_command_new".to_string(),
            "std::process::Command::new".to_string(),
            SecurityIssueType::Injection,
            "Process command execution".to_string(),
        )
        .with_language(SourceLanguage::Rust)
        .with_vulnerable_params(vec![0]),
        TaintSink::new(
            "rust_command_arg".to_string(),
            "std::process::Command::arg".to_string(),
            SecurityIssueType::Injection,
            "Command argument injection".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // File operations
        TaintSink::new(
            "rust_fs_write".to_string(),
            "std::fs::write".to_string(),
            SecurityIssueType::PathTraversal,
            "File write operation".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSink::new(
            "rust_file_create".to_string(),
            "std::fs::File::create".to_string(),
            SecurityIssueType::PathTraversal,
            "File creation".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // Network requests (SSRF)
        TaintSink::new(
            "rust_reqwest_get".to_string(),
            "reqwest::get".to_string(),
            SecurityIssueType::ServerSideRequestForgery,
            "HTTP GET request".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSink::new(
            "rust_tcp_connect".to_string(),
            "std::net::TcpStream::connect".to_string(),
            SecurityIssueType::ServerSideRequestForgery,
            "TCP connection".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // Serialization
        TaintSink::new(
            "rust_serde_from_str".to_string(),
            "serde_json::from_str".to_string(),
            SecurityIssueType::Injection,
            "JSON deserialization".to_string(),
        )
        .with_language(SourceLanguage::Rust),
    ]
}
