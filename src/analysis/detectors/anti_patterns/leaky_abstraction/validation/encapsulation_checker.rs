//! Encapsulation validation for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

/// Validates encapsulation strength and detects encapsulation violations.
pub struct EncapsulationChecker;

impl EncapsulationChecker {
    /// Creates a new encapsulation checker.
    pub fn new() -> Self {
        Self
    }

    /// Checks encapsulation in a parsed file.
    pub fn check_encapsulation(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => {
                issues.extend(self.check_rust_encapsulation(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::Python => {
                issues.extend(self.check_python_encapsulation(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                issues.extend(self.check_js_encapsulation(parsed_file, context)?);
            }
            _ => {}
        }

        Ok(issues)
    }

    /// Checks Rust-specific encapsulation patterns.
    fn check_rust_encapsulation(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for proper use of visibility modifiers
        // Check for appropriate use of modules for encapsulation
        // Check for proper trait boundaries
        // Implementation would go here

        Ok(issues)
    }

    /// Checks Python-specific encapsulation patterns.
    fn check_python_encapsulation(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for proper use of name mangling
        // Check for appropriate use of properties
        // Check for proper module-level encapsulation
        // Implementation would go here

        Ok(issues)
    }

    /// Checks JavaScript/TypeScript-specific encapsulation patterns.
    fn check_js_encapsulation(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for proper use of private fields/methods
        // Check for appropriate use of closures
        // Check for proper module exports
        // Implementation would go here

        Ok(issues)
    }

    /// Evaluates the encapsulation strength of a component.
    pub fn evaluate_encapsulation_strength(&self, component_path: &str) -> EncapsulationScore {
        // This would analyze the component and return a score
        // For now, returning a default score
        EncapsulationScore {
            overall_score: 0.5,
            visibility_score: 0.5,
            interface_score: 0.5,
            dependency_score: 0.5,
            details: vec![],
        }
    }

    /// Checks if a visibility modifier is appropriate for encapsulation.
    pub fn is_appropriate_visibility(&self, visibility: &str, context: &str) -> bool {
        match (visibility, context) {
            ("pub", "api") => true,
            ("pub", "internal") => false,
            ("", "internal") => true,
            _ => true,
        }
    }

    /// Analyzes method signatures for encapsulation violations.
    pub fn analyze_method_encapsulation(&self, method_signature: &str) -> Vec<String> {
        let mut violations = Vec::new();

        if method_signature.contains("&mut") && method_signature.contains("pub") {
            violations.push("Public method returns mutable reference".to_string());
        }

        if method_signature.contains("Box<dyn") && method_signature.contains("pub") {
            violations.push("Public method exposes trait object".to_string());
        }

        violations
    }

    /// Checks if a field declaration violates encapsulation.
    pub fn is_encapsulation_violating_field(&self, field_decl: &str, visibility: &str) -> bool {
        visibility == "pub"
            && (field_decl.contains("Vec<")
                || field_decl.contains("HashMap<")
                || field_decl.contains("internal")
                || field_decl.contains("impl"))
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
            "EncapsulationChecker".to_string(),
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

/// Represents an encapsulation quality score.
#[derive(Debug, Clone)]
pub struct EncapsulationScore {
    /// Overall encapsulation score (0.0 to 1.0)
    pub overall_score: f64,

    /// Score for visibility appropriateness
    pub visibility_score: f64,

    /// Score for interface design quality
    pub interface_score: f64,

    /// Score for dependency management
    pub dependency_score: f64,

    /// Detailed breakdown of issues
    pub details: Vec<String>,
}

impl EncapsulationScore {
    /// Creates a new encapsulation score.
    pub fn new() -> Self {
        Self {
            overall_score: 1.0,
            visibility_score: 1.0,
            interface_score: 1.0,
            dependency_score: 1.0,
            details: Vec::new(),
        }
    }

    /// Adds a violation that reduces the score.
    pub fn add_violation(&mut self, violation: &str, impact: f64) {
        self.details.push(violation.to_string());
        self.overall_score -= impact;
        if self.overall_score < 0.0 {
            self.overall_score = 0.0;
        }
    }

    /// Gets a descriptive quality level.
    pub fn quality_level(&self) -> &'static str {
        match self.overall_score {
            s if s >= 0.8 => "Excellent",
            s if s >= 0.6 => "Good",
            s if s >= 0.4 => "Fair",
            s if s >= 0.2 => "Poor",
            _ => "Critical",
        }
    }
}

impl Default for EncapsulationChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EncapsulationScore {
    fn default() -> Self {
        Self::new()
    }
}
