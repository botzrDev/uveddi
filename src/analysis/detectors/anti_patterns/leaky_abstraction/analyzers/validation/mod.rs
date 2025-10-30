//! Validation coordination module for abstraction boundary checking.

pub mod boundary_validator;
pub mod layer_validator;

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ArchitecturalConfig, ArchitecturalLayer,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

pub use boundary_validator::BoundaryValidator;
pub use layer_validator::LayerValidator;

/// Validates abstraction boundaries and quality.
pub struct AbstractionValidator {
    layer_validator: LayerValidator,
    boundary_validator: BoundaryValidator,
}

impl AbstractionValidator {
    /// Creates a new abstraction validator.
    pub fn new(config: ArchitecturalConfig) -> Self {
        Self {
            layer_validator: LayerValidator::new(config.clone()),
            boundary_validator: BoundaryValidator::new(config),
        }
    }

    /// Validates abstraction boundaries in a parsed file.
    pub fn validate_abstractions(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Validate architectural layer dependencies
        issues.extend(
            self.layer_validator
                .validate_layer_dependencies(parsed_file, context)?,
        );

        // Validate encapsulation boundaries
        issues.extend(
            self.boundary_validator
                .validate_encapsulation(parsed_file, context)?,
        );

        // Validate interface contracts
        issues.extend(
            self.boundary_validator
                .validate_interface_contracts(parsed_file, context)?,
        );

        Ok(issues)
    }

    /// Determines if a dependency violates architectural rules.
    pub fn is_dependency_violation(
        &self,
        from_layer: &ArchitecturalLayer,
        to_layer: &ArchitecturalLayer,
    ) -> bool {
        self.layer_validator
            .is_dependency_violation(from_layer, to_layer)
    }

    /// Checks if a module is considered internal.
    pub fn is_internal_module(&self, module_path: &str) -> bool {
        self.boundary_validator.is_internal_module(module_path)
    }

    /// Checks if a module is infrastructure-related.
    pub fn is_infrastructure_module(&self, module_name: &str) -> bool {
        self.boundary_validator
            .is_infrastructure_module(module_name)
    }

    /// Gets the architectural layer for a file path.
    pub fn get_layer_from_path(&self, file_path: &str) -> Option<ArchitecturalLayer> {
        self.boundary_validator.get_layer_from_path(file_path)
    }
}

impl Clone for AbstractionValidator {
    fn clone(&self) -> Self {
        Self {
            layer_validator: self.layer_validator.clone(),
            boundary_validator: self.boundary_validator.clone(),
        }
    }
}
