use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::Tree;
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::path::Path;

use super::super::types::Dependency;
use super::{import_detector::ImportDetector, wildcard_analyzer::WildcardAnalyzer};

/// Coordinates import analysis using specialized analyzers
#[derive(Debug, Clone)]
pub struct ImportAnalyzer {
    detector: ImportDetector,
    wildcard_analyzer: WildcardAnalyzer,
}

impl ImportAnalyzer {
    pub fn new() -> Self {
        Self {
            detector: ImportDetector::new(),
            wildcard_analyzer: WildcardAnalyzer::new(),
        }
    }

    /// Analyze imports in a file and detect potential coupling issues
    pub fn analyze_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        self.detector.analyze_imports(file_path, tree, source, language)
    }

    /// Detect wildcard imports which can create tight coupling
    pub fn detect_wildcard_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        self.wildcard_analyzer.detect_wildcard_imports(file_path, tree, source, language)
    }

    /// Detect unused imports that increase coupling without benefit
    pub fn detect_unused_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<String>, AnalysisError> {
        self.detector.detect_unused_imports(file_path, tree, source, language)
    }
}

impl Default for ImportAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}