//! Detection of inappropriate state exposure patterns.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

/// Detects patterns where internal state is inappropriately exposed.
pub struct StateExposurePattern;

impl StateExposurePattern {
    /// Creates a new state exposure pattern detector.
    pub fn new() -> Self {
        Self
    }

    /// Detects state exposure patterns in a parsed file.
    pub fn detect_patterns(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => {
                issues.extend(self.detect_rust_state_exposure(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::Python => {
                issues.extend(self.detect_python_state_exposure(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                issues.extend(self.detect_js_state_exposure(parsed_file, context)?);
            }
        }

        Ok(issues)
    }

    /// Detects Rust-specific state exposure patterns.
    fn detect_rust_state_exposure(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect public mutable fields
        // Detect public access to internal state
        // Detect getters that return mutable references to internal state
        // Implementation would go here

        Ok(issues)
    }

    /// Detects Python-specific state exposure patterns.
    fn detect_python_state_exposure(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect direct attribute access instead of properties
        // Detect mutable attributes that should be read-only
        // Detect class state exposed through methods
        // Implementation would go here

        Ok(issues)
    }

    /// Detects JavaScript/TypeScript-specific state exposure patterns.
    fn detect_js_state_exposure(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect direct property access on objects
        // Detect mutable state in functional components
        // Detect global state access in pure functions
        // Implementation would go here

        Ok(issues)
    }

    /// Checks if a field represents exposed state.
    pub fn is_exposed_state_field(&self, field_name: &str, visibility: &str) -> bool {
        visibility == "pub"
            && (field_name.contains("state")
                || field_name.contains("data")
                || field_name.contains("internal")
                || field_name.starts_with('_'))
    }

    /// Checks if a getter method exposes internal state.
    pub fn is_state_exposing_getter(&self, method_name: &str, return_type: &str) -> bool {
        method_name.starts_with("get_")
            && (return_type.contains("&mut")
                || return_type.contains("*mut")
                || self.is_mutable_reference_type(return_type))
    }

    /// Checks if a return type represents a mutable reference to internal state.
    fn is_mutable_reference_type(&self, type_str: &str) -> bool {
        type_str.contains("&mut")
            || type_str.contains("*mut")
            || type_str.contains("RefMut<")
            || type_str.contains("MutexGuard<")
    }

    /// Analyzes method for state exposure risks.
    pub fn analyze_method_state_exposure(&self, method_signature: &str) -> Vec<String> {
        let mut issues = Vec::new();

        if method_signature.contains("&mut self") && method_signature.contains("pub") {
            issues.push("Public method with mutable self reference".to_string());
        }

        if self.is_state_exposing_getter(method_signature, method_signature) {
            issues.push("Getter method returns mutable reference to internal state".to_string());
        }

        issues
    }

    /// Analyzes property access for state exposure.
    pub fn analyze_property_access(&self, property_access: &str) -> Option<String> {
        if property_access.contains(".state") || property_access.contains("._") {
            Some(format!(
                "Direct access to internal state: {}",
                property_access
            ))
        } else {
            None
        }
    }

    /// Checks if a variable assignment exposes state.
    pub fn is_state_exposing_assignment(&self, assignment: &str) -> bool {
        assignment.contains("= &mut")
            || assignment.contains("= self.")
            || assignment.contains("= this.")
    }

    /// Helper function to create an architectural issue.
    fn create_issue(
        &self,
        context: &AnalysisContext,
        description: &str,
        line_number: u32,
        severity: &str,
    ) -> ArchitecturalIssue {
        let mut issue = ArchitecturalIssue::new(
            context.analysis_run_id,
            self.get_anti_pattern_id_for_leak_type(&LeakType::ImplementationExposure),
            context.file_path.clone(),
            Some(line_number as i32),
            description.to_string(),
            "StateExposurePattern".to_string(),
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

impl Default for StateExposurePattern {
    fn default() -> Self {
        Self::new()
    }
}
