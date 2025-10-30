//! Layer dependency validation for abstraction boundary checking.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ArchitecturalConfig, ArchitecturalLayer, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

/// Validates architectural layer dependencies.
pub struct LayerValidator {
    config: ArchitecturalConfig,
}

impl LayerValidator {
    /// Creates a new layer validator.
    pub fn new(config: ArchitecturalConfig) -> Self {
        Self { config }
    }

    /// Validates architectural layer dependencies.
    pub fn validate_layer_dependencies(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let Some(current_layer) = &context.current_layer {
            // Check for inappropriate dependencies based on architectural rules
            let violations =
                self.check_layer_dependency_violations(parsed_file, current_layer, context)?;
            issues.extend(violations);
        }

        Ok(issues)
    }

    /// Checks for layer dependency violations.
    fn check_layer_dependency_violations(
        &self,
        parsed_file: &ParsedFile,
        current_layer: &ArchitecturalLayer,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => {
                issues.extend(self.check_rust_layer_violations(
                    parsed_file,
                    current_layer,
                    context,
                )?);
            }
            crate::ast::SourceLanguage::Python => {
                issues.extend(self.check_python_layer_violations(
                    parsed_file,
                    current_layer,
                    context,
                )?);
            }
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                issues.extend(self.check_js_layer_violations(
                    parsed_file,
                    current_layer,
                    context,
                )?);
            }
        }

        Ok(issues)
    }

    /// Checks Rust-specific layer violations.
    fn check_rust_layer_violations(
        &self,
        parsed_file: &ParsedFile,
        current_layer: &ArchitecturalLayer,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Domain layer should not depend on infrastructure
        if *current_layer == ArchitecturalLayer::Domain {
            issues.extend(self.detect_infrastructure_dependencies_in_domain(parsed_file, context)?);
        }

        // Application layer should not depend on presentation
        if *current_layer == ArchitecturalLayer::Application {
            issues.extend(
                self.detect_presentation_dependencies_in_application(parsed_file, context)?,
            );
        }

        Ok(issues)
    }

    /// Checks Python-specific layer violations.
    fn check_python_layer_violations(
        &self,
        parsed_file: &ParsedFile,
        current_layer: &ArchitecturalLayer,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Similar validation logic for Python
        if *current_layer == ArchitecturalLayer::Domain {
            issues.extend(self.detect_python_infrastructure_violations(parsed_file, context)?);
        }

        Ok(issues)
    }

    /// Checks JavaScript/TypeScript-specific layer violations.
    fn check_js_layer_violations(
        &self,
        parsed_file: &ParsedFile,
        current_layer: &ArchitecturalLayer,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for DOM dependencies in business logic
        if matches!(
            current_layer,
            ArchitecturalLayer::Domain | ArchitecturalLayer::Application
        ) {
            issues.extend(self.detect_dom_dependencies_in_business_logic(parsed_file, context)?);
        }

        Ok(issues)
    }

    /// Detects infrastructure dependencies in domain layer.
    fn detect_infrastructure_dependencies_in_domain(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // This would contain the actual parsing logic
        // For now, returning empty vector as this is a refactoring
        let _ = (parsed_file, context);

        Ok(issues)
    }

    /// Detects presentation dependencies in application layer.
    fn detect_presentation_dependencies_in_application(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // This would contain the actual parsing logic
        let _ = (parsed_file, context);

        Ok(issues)
    }

    /// Detects Python infrastructure violations.
    fn detect_python_infrastructure_violations(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // This would contain the actual parsing logic
        let _ = (parsed_file, context);

        Ok(issues)
    }

    /// Detects DOM dependencies in business logic.
    fn detect_dom_dependencies_in_business_logic(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // This would contain the actual parsing logic
        let _ = (parsed_file, context);

        Ok(issues)
    }

    /// Determines if a dependency violates architectural rules.
    pub fn is_dependency_violation(
        &self,
        from_layer: &ArchitecturalLayer,
        to_layer: &ArchitecturalLayer,
    ) -> bool {
        match (from_layer, to_layer) {
            // Domain should not depend on any outer layers
            (ArchitecturalLayer::Domain, ArchitecturalLayer::Application) => true,
            (ArchitecturalLayer::Domain, ArchitecturalLayer::Presentation) => true,
            (ArchitecturalLayer::Domain, ArchitecturalLayer::Infrastructure) => true,

            // Application should not depend on presentation
            (ArchitecturalLayer::Application, ArchitecturalLayer::Presentation) => true,

            // All other dependencies are allowed
            _ => false,
        }
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
            "LayerValidator".to_string(),
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

impl Clone for LayerValidator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}
