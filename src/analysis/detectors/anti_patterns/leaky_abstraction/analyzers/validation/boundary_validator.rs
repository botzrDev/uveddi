//! Boundary and encapsulation validation for abstraction checking.

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ArchitecturalConfig, ArchitecturalLayer, LeakType
};

/// Validates encapsulation boundaries and interface contracts.
pub struct BoundaryValidator {
    config: ArchitecturalConfig,
}

impl BoundaryValidator {
    /// Creates a new boundary validator.
    pub fn new(config: ArchitecturalConfig) -> Self {
        Self { config }
    }

    /// Validates encapsulation boundaries.
    pub fn validate_encapsulation(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Validate that internal implementation details are not exposed
        // This would contain the actual validation logic

        Ok(issues)
    }

    /// Validates interface contracts.
    pub fn validate_interface_contracts(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Validate that interfaces maintain their contracts
        // This would contain the actual validation logic

        Ok(issues)
    }

    /// Checks if a module is considered internal.
    pub fn is_internal_module(&self, module_path: &str) -> bool {
        self.config
            .internal_patterns
            .iter()
            .any(|pattern| module_path.contains(pattern))
    }

    /// Checks if a module is infrastructure-related.
    pub fn is_infrastructure_module(&self, module_name: &str) -> bool {
        self.config.infrastructure_modules.contains(module_name)
            || self
                .config
                .infrastructure_modules
                .iter()
                .any(|infra| module_name.starts_with(infra))
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

impl Clone for BoundaryValidator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}