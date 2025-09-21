//! Rust-specific sanitizer patterns

use crate::analysis::detectors::security::taint_analysis::types::SanitizationPoint;
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::ast::SourceLanguage;

/// Get Rust-specific sanitizers
pub fn get_rust_sanitizers() -> Vec<SanitizationPoint> {
    vec![
        // SQL sanitization
        SanitizationPoint::new(
            "rust_sqlx_bind".to_string(),
            "sqlx::query!".to_string(),
            vec![SecurityIssueType::Injection],
        )
        .with_language(SourceLanguage::Rust)
        .with_effectiveness(0.95),
        // HTML/XSS prevention
        SanitizationPoint::new(
            "rust_html_escape".to_string(),
            "html_escape::encode_text".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(SourceLanguage::Rust)
        .with_effectiveness(0.90),
        // Command sanitization
        SanitizationPoint::new(
            "rust_shell_escape".to_string(),
            "shell_escape::escape".to_string(),
            vec![SecurityIssueType::Injection],
        )
        .with_language(SourceLanguage::Rust)
        .with_effectiveness(0.85),
        // Path sanitization
        SanitizationPoint::new(
            "rust_path_canonicalize".to_string(),
            "std::path::Path::canonicalize".to_string(),
            vec![SecurityIssueType::PathTraversal],
        )
        .with_language(SourceLanguage::Rust)
        .with_effectiveness(0.80),
        // URL encoding
        SanitizationPoint::new(
            "rust_url_encode".to_string(),
            "url::percent_encoding::utf8_percent_encode".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(SourceLanguage::Rust)
        .with_effectiveness(0.75),
    ]
}
