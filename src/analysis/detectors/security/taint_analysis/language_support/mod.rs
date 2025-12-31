//! Language-specific taint analysis support

pub mod python;
pub mod rust;
pub mod typescript;

pub use python::PythonTaintAnalyzer;
pub use rust::RustTaintAnalyzer;
pub use typescript::TypeScriptTaintAnalyzer;

use crate::analysis::detectors::security::taint_analysis::types::{
    LanguageTaintPatterns, SanitizationPoint, TaintSink, TaintSource,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};

/// Trait for language-specific taint analysis
pub trait LanguageTaintAnalyzer {
    /// Get language-specific taint sources
    fn get_taint_sources(&self) -> Vec<TaintSource>;

    /// Get language-specific taint sinks
    fn get_taint_sinks(&self) -> Vec<TaintSink>;

    /// Get language-specific sanitizers
    fn get_sanitizers(&self) -> Vec<SanitizationPoint>;

    /// Get taint patterns for this language
    fn get_taint_patterns(&self) -> LanguageTaintPatterns;

    /// Analyze language-specific constructs for taint
    fn analyze_language_constructs(
        &self,
        file: &ParsedFile,
    ) -> Result<LanguageAnalysisResult, AnalysisError>;
}

/// Result of language-specific taint analysis
#[derive(Debug, Clone)]
pub struct LanguageAnalysisResult {
    pub sources_found: Vec<TaintSource>,
    pub sinks_found: Vec<TaintSink>,
    pub sanitizers_found: Vec<SanitizationPoint>,
    pub language_specific_issues: Vec<LanguageSpecificIssue>,
}

/// Language-specific taint analysis issue
#[derive(Debug, Clone)]
pub struct LanguageSpecificIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: f64,
    pub location:
        Option<crate::analysis::detectors::security::taint_analysis::types::SourceLocation>,
    pub language: SourceLanguage,
}

/// Factory for creating language-specific analyzers
pub struct LanguageAnalyzerFactory;

impl LanguageAnalyzerFactory {
    /// Create appropriate analyzer for the given language
    pub fn create_analyzer(language: SourceLanguage) -> Box<dyn LanguageTaintAnalyzer> {
        match language {
            SourceLanguage::Rust => Box::new(RustTaintAnalyzer::new()),
            SourceLanguage::Python => Box::new(PythonTaintAnalyzer::new()),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                Box::new(TypeScriptTaintAnalyzer::new())
            }
            _ => Box::new(GenericTaintAnalyzer::new()),
        }
    }

    /// Get all supported languages
    pub fn supported_languages() -> Vec<SourceLanguage> {
        vec![
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ]
    }
}

/// Generic analyzer for unsupported languages
pub struct GenericTaintAnalyzer;

impl GenericTaintAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageTaintAnalyzer for GenericTaintAnalyzer {
    fn get_taint_sources(&self) -> Vec<TaintSource> {
        Vec::new()
    }

    fn get_taint_sinks(&self) -> Vec<TaintSink> {
        Vec::new()
    }

    fn get_sanitizers(&self) -> Vec<SanitizationPoint> {
        Vec::new()
    }

    fn get_taint_patterns(&self) -> LanguageTaintPatterns {
        LanguageTaintPatterns::new()
    }

    fn analyze_language_constructs(
        &self,
        _file: &ParsedFile,
    ) -> Result<LanguageAnalysisResult, AnalysisError> {
        Ok(LanguageAnalysisResult {
            sources_found: Vec::new(),
            sinks_found: Vec::new(),
            sanitizers_found: Vec::new(),
            language_specific_issues: Vec::new(),
        })
    }
}

impl Default for GenericTaintAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
