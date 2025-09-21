//! Detection of tight coupling patterns that indicate leaky abstractions.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

/// Detects tight coupling patterns that violate abstraction boundaries.
pub struct TightCouplingPattern;

impl TightCouplingPattern {
    /// Creates a new tight coupling pattern detector.
    pub fn new() -> Self {
        Self
    }

    /// Detects tight coupling patterns in a parsed file.
    pub fn detect_patterns(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => {
                issues.extend(self.detect_rust_tight_coupling(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::Python => {
                issues.extend(self.detect_python_tight_coupling(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                issues.extend(self.detect_js_tight_coupling(parsed_file, context)?);
            }
        }

        Ok(issues)
    }

    /// Detects Rust-specific tight coupling patterns.
    fn detect_rust_tight_coupling(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect direct struct field access instead of methods
        // Detect concrete type dependencies instead of traits
        // Detect framework-specific types in function signatures
        // Implementation would go here

        Ok(issues)
    }

    /// Detects Python-specific tight coupling patterns.
    fn detect_python_tight_coupling(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect direct database model usage in views
        // Detect framework object usage in business logic
        // Detect concrete class dependencies instead of interfaces
        // Implementation would go here

        Ok(issues)
    }

    /// Detects JavaScript/TypeScript-specific tight coupling patterns.
    fn detect_js_tight_coupling(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect direct DOM manipulation in business logic
        // Detect framework component usage in domain models
        // Detect concrete class dependencies instead of interfaces
        // Implementation would go here

        Ok(issues)
    }

    /// Checks if a dependency indicates tight coupling.
    pub fn is_tight_coupling_dependency(&self, dependency: &str) -> bool {
        // Common patterns that indicate tight coupling
        let tight_coupling_indicators = [
            "diesel::",
            "sqlx::",
            "sea_orm::", // Database ORMs in business logic
            "axum::",
            "warp::",
            "actix_web::", // Web frameworks in domain
            "tokio::",
            "async_std::", // Async runtime in domain
            "serde::",
            "serde_json::", // Serialization in domain
        ];

        tight_coupling_indicators
            .iter()
            .any(|indicator| dependency.contains(indicator))
    }

    /// Checks if a type represents a framework-specific type.
    pub fn is_framework_type(&self, type_name: &str) -> bool {
        let framework_types = [
            "Request",
            "Response",
            "HttpRequest",
            "HttpResponse",
            "Model",
            "QuerySet",
            "Session",
            "Connection",
            "Component",
            "Props",
            "State",
            "Context",
        ];

        framework_types
            .iter()
            .any(|pattern| type_name.contains(pattern))
    }

    /// Checks if a method call indicates tight coupling.
    pub fn is_tight_coupling_method_call(&self, method_call: &str) -> bool {
        let coupling_methods = [
            ".execute(",
            ".query(",
            ".fetch(", // Direct database calls
            ".render(",
            ".redirect(", // Web framework calls
            ".getElementById(",
            ".querySelector(", // DOM manipulation
        ];

        coupling_methods
            .iter()
            .any(|pattern| method_call.contains(pattern))
    }

    /// Analyzes import statements for tight coupling indicators.
    pub fn analyze_import_coupling(&self, import_statement: &str) -> Option<String> {
        if self.is_tight_coupling_dependency(import_statement) {
            Some(format!(
                "Tight coupling detected in import: {}",
                import_statement
            ))
        } else {
            None
        }
    }

    /// Analyzes function signatures for tight coupling.
    pub fn analyze_function_signature_coupling(&self, signature: &str) -> Vec<String> {
        let mut coupling_issues = Vec::new();

        if self.is_framework_type(signature) {
            coupling_issues.push(format!(
                "Framework type in function signature: {}",
                signature
            ));
        }

        coupling_issues
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
            self.get_anti_pattern_id_for_leak_type(&LeakType::FrameworkCoupling),
            context.file_path.clone(),
            Some(line_number as i32),
            description.to_string(),
            "TightCouplingPattern".to_string(),
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

impl Default for TightCouplingPattern {
    fn default() -> Self {
        Self::new()
    }
}
