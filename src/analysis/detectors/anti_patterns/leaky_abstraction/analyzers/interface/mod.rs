//! Interface analysis coordination module.

pub mod js_interface;
pub mod python_interface;
pub mod rust_interface;

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, InterfaceAnalysisResult, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

pub use js_interface::JsInterfaceAnalyzer;
pub use python_interface::PythonInterfaceAnalyzer;
pub use rust_interface::RustInterfaceAnalyzer;

/// Analyzes public interfaces for potential abstraction leaks.
#[derive(Clone)]
pub struct InterfaceAnalyzer {
    rust_analyzer: RustInterfaceAnalyzer,
    python_analyzer: PythonInterfaceAnalyzer,
    js_analyzer: JsInterfaceAnalyzer,
}

impl InterfaceAnalyzer {
    /// Creates a new interface analyzer.
    pub fn new() -> Self {
        Self {
            rust_analyzer: RustInterfaceAnalyzer::new(),
            python_analyzer: PythonInterfaceAnalyzer::new(),
            js_analyzer: JsInterfaceAnalyzer::new(),
        }
    }

    /// Analyzes the public interface of a parsed file.
    pub fn analyze_interface(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => self
                .rust_analyzer
                .analyze_rust_interface(parsed_file, context),
            crate::ast::SourceLanguage::Python => self
                .python_analyzer
                .analyze_python_interface(parsed_file, context),
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                self.js_analyzer.analyze_js_interface(parsed_file, context)
            }
        }
    }

    /// Converts interface analysis results to architectural issues.
    pub fn convert_to_issues(
        &self,
        results: &InterfaceAnalysisResult,
        context: &AnalysisContext,
    ) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();

        // Convert visibility violations
        for violation in &results.visibility_violations {
            let issue = self.create_issue(
                context,
                &violation.description,
                violation.line_number,
                LeakType::VisibilityViolation,
                &violation.severity,
            );
            issues.push(issue);
        }

        // Convert contract violations
        for violation in &results.contract_violations {
            let issue = self.create_issue(
                context,
                &violation.description,
                violation.line_number,
                LeakType::ImplementationExposure,
                &violation.severity,
            );
            issues.push(issue);
        }

        issues
    }

    /// Helper function to create an architectural issue.
    fn create_issue(
        &self,
        context: &AnalysisContext,
        description: &str,
        line_number: u32,
        leak_type: LeakType,
        severity: &str,
    ) -> ArchitecturalIssue {
        let mut issue = ArchitecturalIssue::new(
            context.analysis_run_id,
            self.get_anti_pattern_id_for_leak_type(&leak_type),
            context.file_path.clone(),
            Some(line_number as i32),
            description.to_string(),
            "InterfaceAnalyzer".to_string(),
            severity.to_string(),
            description.to_string(),
        );
        issue.start_line = Some(line_number as i32);
        issue.end_line = Some(line_number as i32);
        issue
    }

    /// Maps a `LeakType` to its corresponding `anti_pattern_type_id`.
    fn get_anti_pattern_id_for_leak_type(&self, leak_type: &LeakType) -> i64 {
        match leak_type {
            LeakType::VisibilityViolation => 1,
            LeakType::LayerViolation => 2,
            LeakType::ImplementationExposure => 3,
            LeakType::FrameworkCoupling => 4,
            LeakType::ErrorPropagation => 5,
            LeakType::PerformanceLeak => 6,
        }
    }
}

impl Default for InterfaceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
