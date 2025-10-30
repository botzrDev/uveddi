//! Rust language support coordination module.

pub mod module_analyzer;
pub mod trait_analyzer;

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::AnalysisContext;
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

pub use module_analyzer::ModuleAnalyzer;
pub use trait_analyzer::TraitAnalyzer;

/// Provides Rust-specific leaky abstraction detection capabilities.
#[derive(Clone)]
pub struct RustLanguageSupport {
    trait_analyzer: TraitAnalyzer,
    module_analyzer: ModuleAnalyzer,
}

impl RustLanguageSupport {
    /// Creates a new Rust language support instance.
    pub fn new() -> Self {
        Self {
            trait_analyzer: TraitAnalyzer::new(),
            module_analyzer: ModuleAnalyzer::new(),
        }
    }

    /// Analyzes a Rust file for leaky abstractions.
    pub fn analyze_file(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Analyze trait exposure and implementation details
        issues.extend(
            self.trait_analyzer
                .analyze_trait_exposure(parsed_file, context)?,
        );
        issues.extend(
            self.trait_analyzer
                .analyze_impl_details(parsed_file, context)?,
        );

        // Analyze error propagation and module privacy
        issues.extend(
            self.module_analyzer
                .analyze_error_propagation(parsed_file, context)?,
        );
        issues.extend(
            self.module_analyzer
                .analyze_module_privacy(parsed_file, context)?,
        );

        Ok(issues)
    }

    /// Extracts the module name from a use statement.
    pub fn extract_module_from_use_statement(&self, use_text: &str) -> Option<String> {
        self.module_analyzer
            .extract_module_from_use_statement(use_text)
    }
}

impl Default for RustLanguageSupport {
    fn default() -> Self {
        Self::new()
    }
}
