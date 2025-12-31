//! Detection of network operation sinks (HTTP responses, XSS, SSRF)

use super::TaintSinkDetector;
use crate::analysis::detectors::security::taint_analysis::types::TaintSink;
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// Detector for network operation sinks
pub struct NetworkSinkDetector {
    patterns: HashMap<SourceLanguage, Vec<String>>,
}

impl NetworkSinkDetector {
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
                "reqwest::get".to_string(),
                "reqwest::post".to_string(),
                "reqwest::Client".to_string(),
                "hyper::client".to_string(),
                "actix_web::HttpResponse".to_string(),
                "warp::reply::html".to_string(),
                "rocket::response".to_string(),
                "tokio_tungstenite::connect_async".to_string(),
                "std::net::TcpStream".to_string(),
            ],
        );

        // Python patterns
        self.patterns.insert(
            SourceLanguage::Python,
            vec![
                "requests.get".to_string(),
                "requests.post".to_string(),
                "urllib.request.urlopen".to_string(),
                "http.client.HTTPConnection".to_string(),
                "socket.connect".to_string(),
                "flask.render_template_string".to_string(),
                "django.http.HttpResponse".to_string(),
                "jinja2.Template".to_string(),
                "tornado.web.write".to_string(),
                "websocket.send".to_string(),
            ],
        );

        // JavaScript/TypeScript patterns
        self.patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                "fetch(".to_string(),
                "XMLHttpRequest".to_string(),
                "axios.get".to_string(),
                "axios.post".to_string(),
                "res.send".to_string(),
                "res.write".to_string(),
                "res.json".to_string(),
                "innerHTML".to_string(),
                "outerHTML".to_string(),
                "document.write".to_string(),
                "eval(".to_string(),
                "Function(".to_string(),
                "setTimeout(".to_string(),
                "setInterval(".to_string(),
                "WebSocket".to_string(),
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

    /// Create taint sinks for outbound HTTP requests (SSRF)
    fn create_ssrf_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSink::new(
                    "rust_reqwest_get".to_string(),
                    "reqwest::get".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "HTTP GET request with user-controlled URL".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_reqwest_post".to_string(),
                    "reqwest::post".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "HTTP POST request with user-controlled URL".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_tcp_stream".to_string(),
                    "std::net::TcpStream".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "TCP connection with user-controlled address".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSink::new(
                    "python_requests_get".to_string(),
                    "requests.get".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "HTTP GET request with user URL".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_urllib_open".to_string(),
                    "urllib.request.urlopen".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "URL opening with user-controlled URL".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_socket_connect".to_string(),
                    "socket.connect".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "Socket connection with user address".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_fetch".to_string(),
                    "fetch(".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "Fetch request with user-controlled URL".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_axios_get".to_string(),
                    "axios.get".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "Axios GET request with user URL".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_websocket".to_string(),
                    "WebSocket".to_string(),
                    SecurityIssueType::ServerSideRequestForgery,
                    "WebSocket connection with user URL".to_string(),
                )
                .with_language(language),

            ],
            _ => vec![],
        }
    }

    /// Create taint sinks for XSS vulnerabilities
    fn create_xss_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSink::new(
                    "rust_actix_response".to_string(),
                    "actix_web::HttpResponse".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "HTTP response with user-controlled content".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_warp_html".to_string(),
                    "warp::reply::html".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "HTML response with user data".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSink::new(
                    "python_render_template_string".to_string(),
                    "flask.render_template_string".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "Template rendering with user-controlled template".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_http_response".to_string(),
                    "django.http.HttpResponse".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "HTTP response with user content".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_jinja_template".to_string(),
                    "jinja2.Template".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "Jinja2 template with user data".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_inner_html".to_string(),
                    "innerHTML".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "DOM manipulation with user-controlled HTML".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_outer_html".to_string(),
                    "outerHTML".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "DOM replacement with user HTML".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_document_write".to_string(),
                    "document.write".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "Document write with user content".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_res_send".to_string(),
                    "res.send".to_string(),
                    SecurityIssueType::CrossSiteScripting,
                    "Express response with user data".to_string(),
                )
                .with_language(language),
            ],
            _ => vec![],
        }
    }

    /// Create taint sinks for code execution via dynamic evaluation
    fn create_code_execution_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_eval".to_string(),
                    "eval(".to_string(),
                    SecurityIssueType::Injection,
                    "JavaScript eval with user input".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_function_constructor".to_string(),
                    "Function(".to_string(),
                    SecurityIssueType::Injection,
                    "Function constructor with user code".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_set_timeout".to_string(),
                    "setTimeout(".to_string(),
                    SecurityIssueType::Injection,
                    "setTimeout with user-controlled code".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_set_interval".to_string(),
                    "setInterval(".to_string(),
                    SecurityIssueType::Injection,
                    "setInterval with user-controlled code".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Rust | SourceLanguage::Python => Vec::new(), 
            _ => vec![],
        }
    }

    /// Create taint sinks for WebSocket communications
    fn create_websocket_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![TaintSink::new(
                "rust_websocket_connect".to_string(),
                "tokio_tungstenite::connect_async".to_string(),
                SecurityIssueType::ServerSideRequestForgery,
                "WebSocket connection with user URL".to_string(),
            )
            .with_language(language)],
            SourceLanguage::Python => vec![TaintSink::new(
                "python_websocket_send".to_string(),
                "websocket.send".to_string(),
                SecurityIssueType::CrossSiteScripting,
                "WebSocket message with user data".to_string(),
            )
            .with_language(language)],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                // Placeholder for JS/TS websocket sinks detection; currently none
            ],
            _ => vec![],
        }
    }
}

impl TaintSinkDetector for NetworkSinkDetector {
    fn detect_sinks(&self, file: &ParsedFile) -> Result<Vec<TaintSink>, AnalysisError> {
        let mut sinks = Vec::new();

        // Get different types of network operation sinks
        sinks.extend(self.create_ssrf_sinks(file.language));
        sinks.extend(self.create_xss_sinks(file.language));
        sinks.extend(self.create_code_execution_sinks(file.language));
        sinks.extend(self.create_websocket_sinks(file.language));

        // TODO: Implement actual AST-based detection

        Ok(sinks)
    }

    fn get_patterns_for_language(&self, language: SourceLanguage) -> Vec<String> {
        self.patterns
            .get(&language)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
}

impl Default for NetworkSinkDetector {
    fn default() -> Self {
        Self::new()
    }
}
