//! Language-specific security analysis support
//!
//! This module provides specialized security analysis for different programming
//! languages and frameworks.

pub mod web_frameworks;
pub mod api_security;
pub mod database_security;

// Re-exports
pub use web_frameworks::WebFrameworkAnalyzer;
pub use api_security::ApiSecurityAnalyzer;
pub use database_security::DatabaseSecurityAnalyzer;

use crate::ast::SourceLanguage;
use crate::analysis::AnalysisError;

/// Language support coordinator for OWASP analysis
pub struct LanguageSupportCoordinator {
    web_analyzer: WebFrameworkAnalyzer,
    api_analyzer: ApiSecurityAnalyzer,
    db_analyzer: DatabaseSecurityAnalyzer,
}

impl LanguageSupportCoordinator {
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            web_analyzer: WebFrameworkAnalyzer::new(),
            api_analyzer: ApiSecurityAnalyzer::new(),
            db_analyzer: DatabaseSecurityAnalyzer::new(),
        })
    }

    pub fn supports_language(&self, language: &SourceLanguage) -> bool {
        matches!(
            language,
            SourceLanguage::Rust | SourceLanguage::Python | SourceLanguage::JavaScript | SourceLanguage::TypeScript
        )
    }

    pub fn get_language_specific_patterns(&self, language: &SourceLanguage) -> Vec<String> {
        match language {
            SourceLanguage::Rust => vec![
                "actix-web".to_string(),
                "rocket".to_string(),
                "warp".to_string(),
                "axum".to_string(),
            ],
            SourceLanguage::Python => vec![
                "flask".to_string(),
                "django".to_string(),
                "fastapi".to_string(),
                "tornado".to_string(),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                "express".to_string(),
                "koa".to_string(),
                "nestjs".to_string(),
                "next.js".to_string(),
            ],
            _ => Vec::new(),
        }
    }
}