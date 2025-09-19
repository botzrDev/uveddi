//! Language-specific support modules for agent detection
//!
//! This module provides language-specific analysis capabilities for detecting
//! agent patterns in different programming languages.

pub mod rust;
pub mod python;
pub mod javascript;

pub use rust::RustAgentAnalyzer;
pub use python::PythonAgentAnalyzer;
pub use javascript::JavaScriptAgentAnalyzer;

use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;

/// Trait for language-specific agent analysis
pub trait LanguageAgentAnalyzer: Send + Sync {
    /// Get the supported language
    fn supported_language(&self) -> SourceLanguage;

    /// Analyze agent patterns specific to this language
    async fn analyze_language_specific(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError>;

    /// Check if this analyzer can handle the given context
    fn can_analyze(&self, context: &SecurityContext) -> bool {
        context.language == self.supported_language()
    }
}

/// Get the appropriate language analyzer for the given context
pub fn get_language_analyzer(language: SourceLanguage) -> Option<Box<dyn LanguageAgentAnalyzer>> {
    match language {
        SourceLanguage::Rust => Some(Box::new(RustAgentAnalyzer::new())),
        SourceLanguage::Python => Some(Box::new(PythonAgentAnalyzer::new())),
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
            Some(Box::new(JavaScriptAgentAnalyzer::new()))
        }
        _ => None,
    }
}

/// Language detection patterns for agent-related code
pub struct LanguagePatterns {
    pub async_patterns: Vec<&'static str>,
    pub network_patterns: Vec<&'static str>,
    pub process_patterns: Vec<&'static str>,
    pub crypto_patterns: Vec<&'static str>,
}