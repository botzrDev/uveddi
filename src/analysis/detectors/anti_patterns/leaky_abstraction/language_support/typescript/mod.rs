//! TypeScript language support coordination module.

pub mod class_analyzer;
pub mod interface_parser;
pub mod module_analyzer;

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

pub use class_analyzer::ClassAnalyzer;
pub use interface_parser::InterfaceParser;
pub use module_analyzer::ModuleAnalyzer;

/// Provides TypeScript-specific leaky abstraction detection capabilities.
#[derive(Clone)]
pub struct TypeScriptLanguageSupport {
    interface_parser: InterfaceParser,
    class_analyzer: ClassAnalyzer,
    module_analyzer: ModuleAnalyzer,
}

impl TypeScriptLanguageSupport {
    /// Creates a new TypeScript language support instance.
    pub fn new() -> Self {
        Self {
            interface_parser: InterfaceParser::new(),
            class_analyzer: ClassAnalyzer::new(),
            module_analyzer: ModuleAnalyzer::new(),
        }
    }

    /// Analyzes a TypeScript file for leaky abstractions.
    pub fn analyze_file(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Analyze interface violations
        issues.extend(
            self.interface_parser
                .analyze_interface_violations(parsed_file, context)?,
        );
        issues.extend(
            self.interface_parser
                .analyze_type_exposure(parsed_file, context)?,
        );

        // Analyze class structure
        issues.extend(
            self.class_analyzer
                .analyze_dom_coupling(parsed_file, context)?,
        );
        issues.extend(
            self.class_analyzer
                .analyze_class_inheritance(parsed_file, context)?,
        );
        issues.extend(
            self.class_analyzer
                .analyze_class_properties(parsed_file, context)?,
        );

        // Analyze module boundaries
        issues.extend(
            self.module_analyzer
                .analyze_import_violations(parsed_file, context)?,
        );
        issues.extend(
            self.module_analyzer
                .analyze_export_patterns(parsed_file, context)?,
        );

        Ok(issues)
    }
}

impl Default for TypeScriptLanguageSupport {
    fn default() -> Self {
        Self::new()
    }
}
