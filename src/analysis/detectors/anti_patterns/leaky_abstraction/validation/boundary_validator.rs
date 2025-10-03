//! Boundary validation for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ArchitecturalConfig, ArchitecturalLayer, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

/// Validates architectural boundaries and detects boundary violations.
pub struct BoundaryValidator {
    config: ArchitecturalConfig,
}

impl BoundaryValidator {
    /// Creates a new boundary validator.
    pub fn new(config: ArchitecturalConfig) -> Self {
        Self { config }
    }

    /// Validates boundaries in a parsed file.
    pub fn validate_boundaries(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Validate layer boundaries
        issues.extend(self.validate_layer_boundaries(parsed_file, context)?);

        // Validate module boundaries
        issues.extend(self.validate_module_boundaries(parsed_file, context)?);

        // Validate interface boundaries
        issues.extend(self.validate_interface_boundaries(parsed_file, context)?);

        Ok(issues)
    }

    /// Validates architectural layer boundaries.
    fn validate_layer_boundaries(
        &self,
        _parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let Some(current_layer) = &context.current_layer {
            // Check for violations of layer dependency rules
            issues.extend(self.check_layer_dependency_violations(current_layer, context)?);
        }

        Ok(issues)
    }

    /// Validates module boundaries.
    fn validate_module_boundaries(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for inappropriate cross-module dependencies
        // Check for violation of module privacy
        // Implementation would go here

        Ok(issues)
    }

    /// Validates interface boundaries.
    fn validate_interface_boundaries(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for appropriate interface design
        // Check for interface contract violations
        // Implementation would go here

        Ok(issues)
    }

    /// Checks for layer dependency violations.
    fn check_layer_dependency_violations(
        &self,
        current_layer: &ArchitecturalLayer,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Example: Domain layer should not depend on presentation or infrastructure
        match current_layer {
            ArchitecturalLayer::Domain => {
                // Domain should be dependency-free of outer layers
                if self.has_presentation_dependencies(context) {
                    issues.push(self.create_issue(
                        context,
                        "Domain layer depends on presentation layer",
                        1,
                        LeakType::LayerViolation,
                        "high",
                    ));
                }
                if self.has_infrastructure_dependencies(context) {
                    issues.push(self.create_issue(
                        context,
                        "Domain layer depends on infrastructure layer",
                        1,
                        LeakType::LayerViolation,
                        "high",
                    ));
                }
            }
            ArchitecturalLayer::Application => {
                // Application should not depend on presentation
                if self.has_presentation_dependencies(context) {
                    issues.push(self.create_issue(
                        context,
                        "Application layer depends on presentation layer",
                        1,
                        LeakType::LayerViolation,
                        "high",
                    ));
                }
            }
            _ => {
                // Other layers have more flexible dependency rules
            }
        }

        Ok(issues)
    }

    /// Checks if the context indicates presentation layer dependencies.
    fn has_presentation_dependencies(&self, _context: &AnalysisContext) -> bool {
        // This would analyze imports/dependencies to detect presentation layer usage
        // For now, returning false as placeholder
        false
    }

    /// Checks if the context indicates infrastructure layer dependencies.
    fn has_infrastructure_dependencies(&self, _context: &AnalysisContext) -> bool {
        // This would analyze imports/dependencies to detect infrastructure usage
        // For now, returning false as placeholder
        false
    }

    /// Evaluates boundary integrity for a component.
    pub fn evaluate_boundary_integrity(&self, component_path: &str) -> BoundaryIntegrityScore {
        // This would analyze the component and return a score
        // For now, returning a default score
        BoundaryIntegrityScore {
            overall_score: 0.5,
            layer_compliance_score: 0.5,
            module_isolation_score: 0.5,
            interface_clarity_score: 0.5,
            violations: vec![],
        }
    }

    /// Checks if a dependency violates boundary rules.
    pub fn is_boundary_violating_dependency(
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

    /// Gets the architectural layer for a file path.
    pub fn get_layer_from_path(&self, file_path: &str) -> Option<ArchitecturalLayer> {
        for (pattern, layer) in &self.config.layer_mappings {
            if self.matches_pattern(file_path, pattern) {
                return Some(layer.clone());
            }
        }
        None
    }

    /// Simple glob-like pattern matcher.
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            let middle = &pattern[3..pattern.len() - 3];
            path.contains(&format!("/{}/", middle)) || path.contains(&format!("\\{}/", middle))
        } else if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            path.ends_with(suffix)
        } else if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            path.starts_with(prefix)
        } else {
            path.contains(pattern)
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
            "BoundaryValidator".to_string(),
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

/// Represents boundary integrity quality metrics.
#[derive(Debug, Clone)]
pub struct BoundaryIntegrityScore {
    /// Overall boundary integrity score (0.0 to 1.0)
    pub overall_score: f64,

    /// Layer compliance score
    pub layer_compliance_score: f64,

    /// Module isolation score
    pub module_isolation_score: f64,

    /// Interface clarity score
    pub interface_clarity_score: f64,

    /// List of boundary violations
    pub violations: Vec<String>,
}

impl BoundaryIntegrityScore {
    /// Creates a new boundary integrity score.
    pub fn new() -> Self {
        Self {
            overall_score: 1.0,
            layer_compliance_score: 1.0,
            module_isolation_score: 1.0,
            interface_clarity_score: 1.0,
            violations: Vec::new(),
        }
    }

    /// Adds a boundary violation.
    pub fn add_violation(&mut self, violation: &str, impact: f64) {
        self.violations.push(violation.to_string());
        self.overall_score -= impact;
        if self.overall_score < 0.0 {
            self.overall_score = 0.0;
        }
    }

    /// Gets a descriptive integrity level.
    pub fn integrity_level(&self) -> &'static str {
        match self.overall_score {
            s if s >= 0.8 => "Excellent",
            s if s >= 0.6 => "Good",
            s if s >= 0.4 => "Fair",
            s if s >= 0.2 => "Poor",
            _ => "Critical",
        }
    }
}

impl Clone for BoundaryValidator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

impl Default for BoundaryIntegrityScore {
    fn default() -> Self {
        Self::new()
    }
}
