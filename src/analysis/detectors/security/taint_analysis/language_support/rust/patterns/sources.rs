//! Rust-specific taint source patterns

use crate::analysis::detectors::security::taint_analysis::types::TaintSource;
use crate::ast::SourceLanguage;

/// Get Rust-specific taint sources
pub fn get_rust_sources() -> Vec<TaintSource> {
    vec![
        // Command line and environment
        TaintSource::new(
            "rust_env_args".to_string(),
            "std::env::args".to_string(),
            "Command line arguments".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSource::new(
            "rust_env_var".to_string(),
            "std::env::var".to_string(),
            "Environment variables".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSource::new(
            "rust_env_vars_os".to_string(),
            "std::env::vars_os".to_string(),
            "All environment variables".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // File I/O
        TaintSource::new(
            "rust_file_read".to_string(),
            "std::fs::read_to_string".to_string(),
            "File content reading".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSource::new(
            "rust_tokio_read".to_string(),
            "tokio::fs::read_to_string".to_string(),
            "Async file reading".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // Standard input
        TaintSource::new(
            "rust_stdin".to_string(),
            "std::io::stdin".to_string(),
            "Standard input stream".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSource::new(
            "rust_read_line".to_string(),
            "std::io::BufRead::read_line".to_string(),
            "Line reading from input".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // Network and HTTP
        TaintSource::new(
            "rust_reqwest_body".to_string(),
            "reqwest::Response::text".to_string(),
            "HTTP response body".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSource::new(
            "rust_hyper_body".to_string(),
            "hyper::body::to_bytes".to_string(),
            "Hyper HTTP body".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        // Web frameworks
        TaintSource::new(
            "rust_actix_request".to_string(),
            "actix_web::HttpRequest".to_string(),
            "Actix web request".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSource::new(
            "rust_warp_query".to_string(),
            "warp::query::query".to_string(),
            "Warp query parameters".to_string(),
        )
        .with_language(SourceLanguage::Rust),
        TaintSource::new(
            "rust_rocket_form".to_string(),
            "rocket::form::Form".to_string(),
            "Rocket form data".to_string(),
        )
        .with_language(SourceLanguage::Rust),
    ]
}
