//! Implementation analysis coordination module.

pub mod dependency_analyzer;
pub mod field_analyzer;
pub mod method_analyzer;
pub mod visibility_analyzer;

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ImplementationAnalysisResult, ImplementationExposure,
    ImplementationVisibilityIssue, LeakType, TypeLeakage,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

pub use dependency_analyzer::DependencyAnalyzer;
pub use field_analyzer::FieldAnalyzer;
pub use method_analyzer::MethodAnalyzer;
pub use visibility_analyzer::VisibilityAnalyzer;

/// Coordinates implementation analysis across multiple specialized analyzers.
#[derive(Clone)]
pub struct ImplementationAnalyzer {
    field_analyzer: FieldAnalyzer,
    method_analyzer: MethodAnalyzer,
    dependency_analyzer: DependencyAnalyzer,
    visibility_analyzer: VisibilityAnalyzer,
}

impl ImplementationAnalyzer {
    /// Creates a new implementation analyzer.
    pub fn new() -> Self {
        Self {
            field_analyzer: FieldAnalyzer::new(),
            method_analyzer: MethodAnalyzer::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            visibility_analyzer: VisibilityAnalyzer::new(),
        }
    }

    /// Analyzes implementation details in a parsed file.
    pub fn analyze_implementation(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<ImplementationAnalysisResult, AnalysisError> {
        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => {
                self.analyze_rust_implementation(parsed_file, context)
            }
            crate::ast::SourceLanguage::Python => {
                self.analyze_python_implementation(parsed_file, context)
            }
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                self.analyze_js_implementation(parsed_file, context)
            }
        }
    }

    /// Analyzes Rust implementation for potential leaks.
    fn analyze_rust_implementation(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<ImplementationAnalysisResult, AnalysisError> {
        let mut implementation_exposures = Vec::new();
        let mut type_leakages = Vec::new();
        let mut visibility_issues = Vec::new();

        // Analyze fields
        let (field_exposures, field_leakages) = self
            .field_analyzer
            .analyze_rust_fields(parsed_file, context)?;
        implementation_exposures.extend(field_exposures);
        type_leakages.extend(field_leakages);

        // Analyze methods
        let (method_leakages, method_visibility) = self
            .method_analyzer
            .analyze_rust_methods(parsed_file, context)?;
        type_leakages.extend(method_leakages);
        visibility_issues.extend(method_visibility);

        // Analyze visibility
        let vis_issues = self
            .visibility_analyzer
            .analyze_rust_visibility(parsed_file, context)?;
        visibility_issues.extend(vis_issues);

        Ok(ImplementationAnalysisResult {
            implementation_exposures,
            type_leakages,
            visibility_issues,
        })
    }

    /// Analyzes Python implementation for potential leaks.
    fn analyze_python_implementation(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<ImplementationAnalysisResult, AnalysisError> {
        let mut implementation_exposures = Vec::new();
        let mut type_leakages = Vec::new();
        let mut visibility_issues = Vec::new();

        // Analyze fields
        let (field_exposures, field_leakages) = self
            .field_analyzer
            .analyze_python_fields(parsed_file, context)?;
        implementation_exposures.extend(field_exposures);
        type_leakages.extend(field_leakages);

        // Analyze methods
        let method_visibility = self
            .method_analyzer
            .analyze_python_methods(parsed_file, context)?;
        visibility_issues.extend(method_visibility);

        // Analyze dependencies
        let dep_exposures = self
            .dependency_analyzer
            .analyze_python_dependencies(parsed_file, context)?;
        implementation_exposures.extend(dep_exposures);

        // Analyze visibility
        let vis_issues = self
            .visibility_analyzer
            .analyze_python_visibility(parsed_file, context)?;
        visibility_issues.extend(vis_issues);

        Ok(ImplementationAnalysisResult {
            implementation_exposures,
            type_leakages,
            visibility_issues,
        })
    }

    /// Analyzes JavaScript/TypeScript implementation for potential leaks.
    fn analyze_js_implementation(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<ImplementationAnalysisResult, AnalysisError> {
        let mut implementation_exposures = Vec::new();
        let mut type_leakages = Vec::new();
        let mut visibility_issues = Vec::new();

        // Analyze methods
        let method_visibility = self
            .method_analyzer
            .analyze_js_methods(parsed_file, context)?;
        visibility_issues.extend(method_visibility);

        // Analyze dependencies
        let dep_exposures = self
            .dependency_analyzer
            .analyze_js_dependencies(parsed_file, context)?;
        implementation_exposures.extend(dep_exposures);

        // Analyze visibility
        let vis_issues = self
            .visibility_analyzer
            .analyze_js_visibility(parsed_file, context)?;
        visibility_issues.extend(vis_issues);

        Ok(ImplementationAnalysisResult {
            implementation_exposures,
            type_leakages,
            visibility_issues,
        })
    }

    /// Converts implementation analysis results to architectural issues.
    pub fn convert_to_issues(
        &self,
        results: &ImplementationAnalysisResult,
        context: &AnalysisContext,
    ) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();

        // Convert implementation exposures
        for exposure in &results.implementation_exposures {
            let issue = self.create_issue(
                context,
                &exposure.description,
                exposure.line_number,
                LeakType::ImplementationExposure,
                &exposure.severity,
            );
            issues.push(issue);
        }

        // Convert type leakages
        for leakage in &results.type_leakages {
            let issue = self.create_issue(
                context,
                &leakage.description,
                leakage.line_number,
                LeakType::ErrorPropagation,
                &leakage.severity,
            );
            issues.push(issue);
        }

        // Convert visibility issues
        for visibility_issue in &results.visibility_issues {
            let issue = self.create_issue(
                context,
                &visibility_issue.description,
                visibility_issue.line_number,
                LeakType::VisibilityViolation,
                &visibility_issue.severity,
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
            "ImplementationAnalyzer".to_string(),
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

impl Default for ImplementationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
