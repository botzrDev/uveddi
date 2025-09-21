//! TypeScript-specific God Object detection

mod classes;
mod interfaces;
mod namespaces;
mod patterns;
mod queries;

pub use classes::TypeScriptClassAnalyzer;
pub use interfaces::TypeScriptInterfaceAnalyzer;
pub use namespaces::TypeScriptNamespaceAnalyzer;
pub use patterns::TypeScriptPatternDetector;

use super::super::config::GodObjectConfig;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use std::collections::HashSet;
use tracing::debug;

/// TypeScript-specific God Object analyzer
pub struct TypeScriptGodObjectAnalyzer<'a> {
    config: &'a GodObjectConfig,
    class_analyzer: TypeScriptClassAnalyzer<'a>,
    interface_analyzer: TypeScriptInterfaceAnalyzer<'a>,
    namespace_analyzer: TypeScriptNamespaceAnalyzer<'a>,
    pattern_detector: TypeScriptPatternDetector<'a>,
}

impl<'a> TypeScriptGodObjectAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self {
            config,
            class_analyzer: TypeScriptClassAnalyzer::new(config),
            interface_analyzer: TypeScriptInterfaceAnalyzer::new(config),
            namespace_analyzer: TypeScriptNamespaceAnalyzer::new(config),
            pattern_detector: TypeScriptPatternDetector::new(config),
        }
    }

    /// Analyze a TypeScript file for God Objects
    pub fn analyze(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Stage 1: Framework Detection
        let detected_frameworks = self.pattern_detector.analyze_imports(parsed_file)?;
        debug!("Detected TypeScript frameworks: {:?}", detected_frameworks);

        // Analyze Classes
        issues.extend(
            self.class_analyzer
                .analyze_classes(parsed_file, &detected_frameworks)?,
        );

        // Analyze Interfaces (can be God Objects too)
        issues.extend(self.interface_analyzer.analyze_interfaces(parsed_file)?);

        // Analyze Namespaces
        issues.extend(self.namespace_analyzer.analyze_namespaces(parsed_file)?);

        Ok(issues)
    }
}
