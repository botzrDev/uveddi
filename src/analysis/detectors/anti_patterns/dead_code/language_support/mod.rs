//! Language-specific analysis support for dead code detection

pub mod javascript;
pub mod python;
pub mod rust;

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::Node;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashSet;

use super::types::Symbol;

/// Trait for language-specific dead code analysis
pub trait LanguageAnalyzer: Send + Sync {
    /// Extract symbols from a parsed file
    fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError>;

    /// Extract references from a parsed file
    fn extract_references(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError>;

    /// Check if a symbol is exported/public
    fn is_exported(&self, node: &Node, source: &[u8]) -> bool;

    /// Calculate confidence score for a symbol
    fn calculate_confidence(&self, symbol: &Symbol) -> f64;

    /// Identify entry points in the language
    fn identify_entry_points(&self, symbols: &[Symbol]) -> Vec<String>;
}

/// Factory for creating language-specific analyzers
pub struct LanguageAnalyzerFactory;

impl LanguageAnalyzerFactory {
    /// Create an analyzer for the specified language
    pub fn create(language: SourceLanguage) -> Box<dyn LanguageAnalyzer> {
        match language {
            SourceLanguage::Rust => Box::new(rust::RustAnalyzer::new()),
            SourceLanguage::Python => Box::new(python::PythonAnalyzer::new()),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                Box::new(javascript::JavaScriptAnalyzer::new())
            }
            _ => Box::new(NoopAnalyzer),
        }
    }
}

pub struct NoopAnalyzer;

impl LanguageAnalyzer for NoopAnalyzer {
    fn extract_symbols(&self, _parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        Ok(Vec::new())
    }
    fn extract_references(
        &self,
        _parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        Ok(HashSet::new())
    }
    fn is_exported(&self, _node: &Node, _source: &[u8]) -> bool {
        true
    }
    fn calculate_confidence(&self, _symbol: &Symbol) -> f64 {
        0.0
    }
    fn identify_entry_points(&self, _symbols: &[Symbol]) -> Vec<String> {
        Vec::new()
    }
}
