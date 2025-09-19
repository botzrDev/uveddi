use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::Tree;
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::path::Path;
use tracing::debug;

use super::super::types::Dependency;
use super::{
    interface_extractor::InterfaceExtractor,
    interface_validator::{InterfaceUsage, InterfaceValidator},
};

/// Analyzes interface usage and abstractions to identify coupling patterns
#[derive(Debug, Clone)]
pub struct InterfaceAnalyzer {
    extractor: InterfaceExtractor,
    validator: InterfaceValidator,
}

impl InterfaceAnalyzer {
    pub fn new() -> Self {
        Self {
            extractor: InterfaceExtractor::new(),
            validator: InterfaceValidator::new(),
        }
    }

    /// Analyze interface and trait usage patterns
    pub fn analyze_interface_usage(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<InterfaceUsage>, AnalysisError> {
        debug!("Analyzing interface usage for {}", file_path.display());
        self.validator.analyze_interface_usage(file_path, tree, source, language)
    }

    /// Extract trait/interface implementations and their dependencies
    pub fn extract_interface_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        debug!("Extracting interface dependencies for {}", file_path.display());
        self.extractor.extract_interface_dependencies(file_path, tree, source, language)
    }

    /// Detect direct field access which indicates tight coupling
    pub fn detect_field_access_coupling(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        debug!("Detecting field access coupling for {}", file_path.display());
        self.extractor.detect_field_access_coupling(file_path, tree, source, language)
    }

    /// Perform comprehensive interface analysis
    pub fn analyze_comprehensive(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
        let usage_patterns = self.analyze_interface_usage(file_path, tree, source, language)?;
        let dependencies = self.extract_interface_dependencies(file_path, tree, source, language)?;
        let field_accesses = self.detect_field_access_coupling(file_path, tree, source, language)?;

        Ok(InterfaceAnalysisResult {
            usage_patterns,
            dependencies,
            field_accesses,
        })
    }
}

/// Comprehensive interface analysis results
#[derive(Debug)]
pub struct InterfaceAnalysisResult {
    pub usage_patterns: Vec<InterfaceUsage>,
    pub dependencies: Vec<Dependency>,
    pub field_accesses: Vec<Dependency>,
}

impl Default for InterfaceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}