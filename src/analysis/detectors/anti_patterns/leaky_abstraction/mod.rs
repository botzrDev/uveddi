//! Advanced Leaky Abstraction Detector
//!
//! This detector implements a comprehensive analysis framework for identifying leaky abstractions
//! across multiple programming languages (Rust, Python, JavaScript/TypeScript). It uses a
//! multi-signal approach combining AST analysis, dependency tracking, and architectural pattern
//! recognition to detect violations of abstraction boundaries.
//!
//! ## Detection Capabilities
//!
//! ### Rust-Specific Patterns
//! - Visibility violations (pub vs private boundaries)
//! - Framework-specific types in public APIs
//! - ORM/Database types leaking into business logic
//! - Error type propagation across layers
//! - Async runtime details in interfaces
//!
//! ### Python-Specific Patterns
//! - Direct database model usage in views/controllers
//! - Framework objects in business logic (Flask request, Django models)
//! - File system paths in public interfaces
//! - Import violations across architectural layers
//!
//! ### JavaScript/TypeScript Patterns
//! - DOM manipulation in business logic
//! - Framework-specific objects in domain models
//! - Infrastructure dependencies in application layer
//! - Type definition leaks and generic pollution
//!
//! ## Architecture
//!
//! The detector uses a layered analysis approach:
//! 1. **Syntactic Analysis**: Tree-sitter queries for pattern matching
//! 2. **Semantic Analysis**: Symbol resolution and type flow tracking
//! 3. **Architectural Analysis**: Layer boundary validation
//! 4. **Cross-file Analysis**: Dependency graph traversal
//!
//! ## Configuration
//!
//! The detector requires architectural configuration to understand the intended
//! layer boundaries and infrastructure dependencies:
//!
//! ```rust
//! use uveddi::analysis::detectors::anti_patterns::leaky_abstraction::{
//!     LeakyAbstractionDetector, ArchitecturalConfig, ArchitecturalLayer
//! };
//! use std::collections::{HashMap, HashSet};
//!
//! let mut layer_mappings = HashMap::new();
//! layer_mappings.insert("**/controllers/**".to_string(), ArchitecturalLayer::Presentation);
//! layer_mappings.insert("**/services/**".to_string(), ArchitecturalLayer::Application);
//! layer_mappings.insert("**/domain/**".to_string(), ArchitecturalLayer::Domain);
//!
//! let mut infrastructure_modules = HashSet::new();
//! infrastructure_modules.insert("sqlx".to_string());
//! infrastructure_modules.insert("tokio".to_string());
//! infrastructure_modules.insert("serde".to_string());
//!
//! let config = ArchitecturalConfig {
//!     layer_mappings,
//!     infrastructure_modules,
//!     internal_patterns: vec!["_internal".to_string(), "private".to_string()],
//! };
//!
//! let detector = LeakyAbstractionDetector::with_config(config);
//! ```
//!
//! ## Performance Considerations
//! - **Time Complexity**: O(n*m) where n is AST nodes and m is architectural rules
//! - **Space Complexity**: O(k) where k is the number of detected violations
//! - **Optimization Notes**: Uses efficient pattern matching and caches rule evaluations

// Re-export main components for public API
pub mod types;
pub mod config;
pub mod detector;
pub mod analyzers;
pub mod patterns;
pub mod language_support;
pub mod validation;

// Public API exports
pub use detector::LeakyAbstractionDetector;
pub use types::{
    ArchitecturalConfig, ArchitecturalLayer, LeakType, AnalysisContext,
    InterfaceAnalysisResult, ImplementationAnalysisResult,
    ApiElement, VisibilityViolation, ContractViolation,
    ImplementationExposure, TypeLeakage, ImplementationVisibilityIssue,
};
pub use config::{ConfigBuilder, create_default_config};

// Analyzer exports
pub use analyzers::{
    InterfaceAnalyzer, ImplementationAnalyzer, AbstractionValidator, LeakDetector
};

// Pattern exports
pub use patterns::{
    ExposedInternalsPattern, TightCouplingPattern, StateExposurePattern, DataStructureLeaksPattern
};

// Language support exports
pub use language_support::{
    RustLanguageSupport, PythonLanguageSupport, TypeScriptLanguageSupport
};

// Validation exports
pub use validation::{
    EncapsulationChecker, BoundaryValidator, AbstractionScorer,
    EncapsulationScore, BoundaryIntegrityScore, AbstractionQualityScore,
};

/// Convenience function to create a new detector with default configuration.
pub fn new_detector() -> LeakyAbstractionDetector {
    LeakyAbstractionDetector::new()
}

/// Convenience function to create a new detector with custom configuration.
pub fn new_detector_with_config(config: ArchitecturalConfig) -> LeakyAbstractionDetector {
    LeakyAbstractionDetector::with_config(config)
}

/// Utility function to create a basic configuration for common project structures.
pub fn create_basic_config() -> ArchitecturalConfig {
    use std::collections::{HashMap, HashSet};

    let mut layer_mappings = HashMap::new();
    layer_mappings.insert("**/controllers/**".to_string(), ArchitecturalLayer::Presentation);
    layer_mappings.insert("**/services/**".to_string(), ArchitecturalLayer::Application);
    layer_mappings.insert("**/domain/**".to_string(), ArchitecturalLayer::Domain);
    layer_mappings.insert("**/infrastructure/**".to_string(), ArchitecturalLayer::Infrastructure);

    let mut infrastructure_modules = HashSet::new();
    infrastructure_modules.insert("sqlx".to_string());
    infrastructure_modules.insert("diesel".to_string());
    infrastructure_modules.insert("tokio".to_string());
    infrastructure_modules.insert("axum".to_string());
    infrastructure_modules.insert("serde".to_string());

    ArchitecturalConfig {
        layer_mappings,
        infrastructure_modules,
        internal_patterns: vec![
            "internal".to_string(),
            "_internal".to_string(),
            "private".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = new_detector();
        assert_eq!(detector.get_detector_name(), "LeakyAbstractionDetector");
    }

    #[test]
    fn test_detector_with_config() {
        let config = create_basic_config();
        let detector = new_detector_with_config(config);
        assert_eq!(detector.get_detector_name(), "LeakyAbstractionDetector");
    }

    #[test]
    fn test_basic_config_creation() {
        let config = create_basic_config();
        assert!(!config.layer_mappings.is_empty());
        assert!(!config.infrastructure_modules.is_empty());
        assert!(!config.internal_patterns.is_empty());
    }

    #[test]
    fn test_anti_pattern_types() {
        let detector = new_detector();
        let anti_pattern_types = detector.get_anti_pattern_types();
        assert_eq!(anti_pattern_types.len(), 6);

        let names: Vec<&str> = anti_pattern_types.iter()
            .map(|apt| apt.name.as_str())
            .collect();

        assert!(names.contains(&"Visibility Violation"));
        assert!(names.contains(&"Layer Violation"));
        assert!(names.contains(&"Implementation Exposure"));
        assert!(names.contains(&"Framework Coupling"));
        assert!(names.contains(&"Error Propagation"));
        assert!(names.contains(&"Performance Leak"));
    }
}