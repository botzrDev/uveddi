//! Python language support coordination module.

pub mod class_analyzer;
pub mod framework_analyzer;
pub mod import_analyzer;

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::AnalysisContext;
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

pub use class_analyzer::ClassAnalyzer;
pub use framework_analyzer::FrameworkAnalyzer;
pub use import_analyzer::ImportAnalyzer;

/// Provides Python-specific leaky abstraction detection capabilities.
#[derive(Clone)]
pub struct PythonLanguageSupport {
    class_analyzer: ClassAnalyzer,
    import_analyzer: ImportAnalyzer,
    framework_analyzer: FrameworkAnalyzer,
}

impl PythonLanguageSupport {
    /// Creates a new Python language support instance.
    pub fn new() -> Self {
        Self {
            class_analyzer: ClassAnalyzer::new(),
            import_analyzer: ImportAnalyzer::new(),
            framework_analyzer: FrameworkAnalyzer::new(),
        }
    }

    /// Analyzes a Python file for leaky abstractions.
    pub fn analyze_file(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Analyze class internals and private methods
        issues.extend(
            self.class_analyzer
                .analyze_class_internals(parsed_file, context)?,
        );
        issues.extend(
            self.class_analyzer
                .analyze_private_methods(parsed_file, context)?,
        );

        // Analyze import violations
        issues.extend(
            self.import_analyzer
                .analyze_import_violations(parsed_file, context)?,
        );

        // Analyze framework coupling
        issues.extend(
            self.framework_analyzer
                .analyze_framework_coupling(parsed_file, context)?,
        );

        Ok(issues)
    }

    /// Extracts the module name from a Python import statement.
    pub fn extract_python_import_module(&self, import_text: &str) -> Option<String> {
        self.import_analyzer
            .extract_python_import_module(import_text)
    }
}

impl Default for PythonLanguageSupport {
    fn default() -> Self {
        Self::new()
    }
}
