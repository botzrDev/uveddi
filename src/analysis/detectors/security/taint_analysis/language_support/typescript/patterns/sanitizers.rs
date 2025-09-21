//! TypeScript/JavaScript sanitizer patterns

use crate::analysis::detectors::security::taint_analysis::types::SanitizationPoint;
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::ast::SourceLanguage;

/// Get TypeScript/JavaScript-specific sanitizers
pub fn get_typescript_sanitizers() -> Vec<SanitizationPoint> {
    let language = SourceLanguage::TypeScript;

    vec![
        // HTML/XSS prevention
        SanitizationPoint::new(
            "js_dompurify".to_string(),
            "DOMPurify.sanitize".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(language)
        .with_effectiveness(0.95),
        SanitizationPoint::new(
            "js_he_encode".to_string(),
            "he.encode".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(language)
        .with_effectiveness(0.85),
        SanitizationPoint::new(
            "js_validator_escape".to_string(),
            "validator.escape".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(language)
        .with_effectiveness(0.80),
        // URL encoding
        SanitizationPoint::new(
            "js_encode_uri".to_string(),
            "encodeURIComponent".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(language)
        .with_effectiveness(0.75),
        SanitizationPoint::new(
            "js_encode_uri_full".to_string(),
            "encodeURI".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(language)
        .with_effectiveness(0.70),
        // Command sanitization (Node.js)
        SanitizationPoint::new(
            "js_shell_escape".to_string(),
            "shell-escape".to_string(),
            vec![SecurityIssueType::Injection],
        )
        .with_language(language)
        .with_effectiveness(0.85),
        SanitizationPoint::new(
            "js_shell_quote".to_string(),
            "shell-quote".to_string(),
            vec![SecurityIssueType::Injection],
        )
        .with_language(language)
        .with_effectiveness(0.85),
        // Path sanitization (Node.js)
        SanitizationPoint::new(
            "js_path_normalize".to_string(),
            "path.normalize".to_string(),
            vec![SecurityIssueType::PathTraversal],
        )
        .with_language(language)
        .with_effectiveness(0.70),
        SanitizationPoint::new(
            "js_path_resolve".to_string(),
            "path.resolve".to_string(),
            vec![SecurityIssueType::PathTraversal],
        )
        .with_language(language)
        .with_effectiveness(0.75),
        // JSON sanitization
        SanitizationPoint::new(
            "js_json_stringify".to_string(),
            "JSON.stringify".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        )
        .with_language(language)
        .with_effectiveness(0.60),
        // URL validation
        SanitizationPoint::new(
            "js_url_constructor".to_string(),
            "new URL(".to_string(),
            vec![SecurityIssueType::ServerSideRequestForgery],
        )
        .with_language(language)
        .with_effectiveness(0.70),
        SanitizationPoint::new(
            "js_validator_isurl".to_string(),
            "validator.isURL".to_string(),
            vec![SecurityIssueType::ServerSideRequestForgery],
        )
        .with_language(language)
        .with_effectiveness(0.80),
    ]
}
