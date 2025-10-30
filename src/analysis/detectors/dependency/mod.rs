pub mod analyzers;
pub mod config;
pub mod detector;
pub mod language_support;
pub mod licenses;
pub mod types;
pub mod vulnerabilities;

pub use config::*;
pub use detector::{DependencyDetector, ExtractionError};
pub use types::*;

pub use analyzers::{
    AnalysisOutput, AnalyzerRegistry, CircularDependencyAnalysis, CircularDependencyDetector,
    DependencyGraphAnalyzer, GraphAnalysisResult, OutdatedAnalysis, OutdatedDependencyChecker,
    TransitiveAnalysisResult, TransitiveAnalyzer, VersionAnalysis, VersionAnalyzer,
};

pub use vulnerabilities::{
    AdvisoryScanResult, AdvisoryScanner, CveScanResult, CveScanner, IntegrityCheckResult,
    IntegrityChecker, MalwareScanResult, MalwareScanner, SupplyChainAnalysis, SupplyChainAnalyzer,
    VulnerabilityScanOutput, VulnerabilityScannerRegistry,
};

pub use licenses::{
    ComplianceChecker, ComplianceResult, ConflictAnalysis, ConflictDetector, LicenseAnalysisResult,
    LicenseAnalyzer, LicenseCheckOutput, LicenseCheckerRegistry, PolicyEnforcer, PolicyResult,
};

pub use language_support::{
    CargoManifest, JavaScriptDependencyParser, LanguageParserRegistry, PackageJson,
    PythonDependencyParser, PythonRequirement, RustDependencyParser,
};

pub use crate::database::models::{Dependency, DependencyType};

use std::path::Path;

pub struct DependencyExtractor {
    detector: DependencyDetector,
}

impl DependencyExtractor {
    pub fn new() -> Result<Self, ExtractionError> {
        let detector = DependencyDetector::new()?;
        Ok(Self { detector })
    }

    pub fn extract_from_file(
        &mut self,
        file_path: &Path,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        self.detector.analyze_file(file_path)
    }

    pub fn extract_dependencies(
        &self,
        parsed_file: &crate::analysis::components::ast_provider::ParsedFile,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        let ast_parsed_file = crate::ast::tree_sitter_impl::ParsedFile {
            file_path: std::sync::Arc::clone(&parsed_file.file_path),
            language: parsed_file.language,
            source: std::sync::Arc::clone(&parsed_file.source),
            tree: parsed_file.tree.as_ref().map(|arc| arc.as_ref().clone()),
            custom_ast: std::sync::Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        self.detector.extract_from_ast(&ast_parsed_file)
    }

    pub fn extract_from_ast(
        &self,
        parsed_file: &crate::ast::tree_sitter_impl::ParsedFile,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        self.detector.extract_from_ast(parsed_file)
    }
}
