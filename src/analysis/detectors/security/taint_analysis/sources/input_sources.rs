//! Detection of input-based taint sources (HTTP requests, forms, etc.)

use super::TaintSourceDetector;
use crate::analysis::detectors::security::taint_analysis::types::TaintSource;
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// Detector for input-based taint sources
pub struct InputSourceDetector {
    patterns: HashMap<SourceLanguage, Vec<String>>,
}

impl InputSourceDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            patterns: HashMap::new(),
        };
        detector.initialize_patterns();
        detector
    }

    fn initialize_patterns(&mut self) {
        // Rust patterns
        self.patterns.insert(
            SourceLanguage::Rust,
            vec![
                "request.body".to_string(),
                "req.body".to_string(),
                "request.form".to_string(),
                "request.query".to_string(),
                "request.headers".to_string(),
                "request.cookies".to_string(),
                "actix_web::web::Form".to_string(),
                "actix_web::web::Query".to_string(),
                "actix_web::web::Json".to_string(),
                "warp::body::json".to_string(),
                "hyper::body::to_bytes".to_string(),
            ],
        );

        // Python patterns
        self.patterns.insert(
            SourceLanguage::Python,
            vec![
                "request.".to_string(),
                "req.".to_string(),
                "flask.request".to_string(),
                "django.request".to_string(),
                "request.form".to_string(),
                "request.args".to_string(),
                "request.json".to_string(),
                "request.data".to_string(),
                "request.files".to_string(),
                "request.cookies".to_string(),
                "request.headers".to_string(),
                "cherrypy.request".to_string(),
                "bottle.request".to_string(),
            ],
        );

        // JavaScript/TypeScript patterns
        self.patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                "req.body".to_string(),
                "req.query".to_string(),
                "req.params".to_string(),
                "req.headers".to_string(),
                "req.cookies".to_string(),
                "request.body".to_string(),
                "express.Request".to_string(),
                "location.search".to_string(),
                "window.location.search".to_string(),
                "URLSearchParams".to_string(),
                "document.forms".to_string(),
                "FormData".to_string(),
            ],
        );

        self.patterns.insert(
            SourceLanguage::TypeScript,
            self.patterns
                .get(&SourceLanguage::JavaScript)
                .unwrap()
                .clone(),
        );
    }

    /// Create taint sources for HTTP request patterns
    fn create_http_sources(&self, language: SourceLanguage) -> Vec<TaintSource> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSource::new(
                    "rust_http_body".to_string(),
                    "request.body".to_string(),
                    "HTTP request body data".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "rust_http_query".to_string(),
                    "request.query".to_string(),
                    "HTTP query parameters".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "rust_http_headers".to_string(),
                    "request.headers".to_string(),
                    "HTTP request headers".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "rust_actix_form".to_string(),
                    "actix_web::web::Form".to_string(),
                    "Actix web form data".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSource::new(
                    "python_flask_request".to_string(),
                    "flask.request".to_string(),
                    "Flask request object".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "python_request_form".to_string(),
                    "request.form".to_string(),
                    "Form data from request".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "python_request_args".to_string(),
                    "request.args".to_string(),
                    "URL arguments from request".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "python_request_json".to_string(),
                    "request.json".to_string(),
                    "JSON data from request".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSource::new(
                    "js_req_body".to_string(),
                    "req.body".to_string(),
                    "Express.js request body".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_req_query".to_string(),
                    "req.query".to_string(),
                    "Express.js query parameters".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_req_params".to_string(),
                    "req.params".to_string(),
                    "Express.js route parameters".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_location_search".to_string(),
                    "location.search".to_string(),
                    "Browser URL search parameters".to_string(),
                )
                .with_language(language),
                TaintSource::new(
                    "js_form_data".to_string(),
                    "FormData".to_string(),
                    "Browser form data".to_string(),
                )
                .with_language(language),
            ],
            _ => vec![],
        }
    }
}

impl TaintSourceDetector for InputSourceDetector {
    fn detect_sources(&self, file: &ParsedFile) -> Result<Vec<TaintSource>, AnalysisError> {
        let mut sources = Vec::new();

        // Get language-specific HTTP input sources
        sources.extend(self.create_http_sources(file.language));

        // TODO: Implement actual AST-based detection
        // This would involve:
        // 1. Walking the AST to find variable assignments
        // 2. Matching against patterns for each language
        // 3. Creating TaintSource objects with proper location info

        Ok(sources)
    }

    fn get_patterns_for_language(&self, language: SourceLanguage) -> Vec<String> {
        self.patterns
            .get(&language)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
}

impl Default for InputSourceDetector {
    fn default() -> Self {
        Self::new()
    }
}
