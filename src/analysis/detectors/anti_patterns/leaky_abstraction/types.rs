//! Type definitions for leaky abstraction detection.

use std::collections::{HashMap, HashSet};
use strum_macros::EnumString;

/// Defines the configuration for architectural layers and boundaries.
///
/// This struct provides the necessary context for the `LeakyAbstractionDetector`
/// to understand the intended architecture of a project. It specifies how to map
/// file paths to architectural layers and identifies known infrastructure dependencies.
#[derive(Debug, Clone)]
pub struct ArchitecturalConfig {
    /// A mapping of glob patterns to `ArchitecturalLayer` enums.
    ///
    /// This is the primary mechanism for defining the architecture. For example:
    /// `{"**/controllers/**": Presentation, "**/services/**": Application}`.
    pub layer_mappings: HashMap<String, ArchitecturalLayer>,

    /// A set of module names or prefixes that are considered infrastructure.
    ///
    /// This set helps identify dependencies on frameworks, databases, or other
    /// external systems (e.g., "django", "sqlx", "react").
    pub infrastructure_modules: HashSet<String>,

    /// A list of patterns used to identify internal or private modules.
    ///
    /// Accessing modules whose paths contain these patterns from outside their
    /// parent component is considered a visibility violation (e.g., "_internal").
    pub internal_patterns: Vec<String>,
}

/// Represents the distinct architectural layers of a system, inspired by Clean Architecture.
///
/// Each layer has a specific responsibility, and dependencies should generally flow
/// from outer layers (e.g., `Presentation`) to inner layers (e.g., `Domain`).
#[derive(Debug, Clone, PartialEq, EnumString)]
pub enum ArchitecturalLayer {
    /// The outermost layer, responsible for UI, API endpoints, and user interaction.
    /// It translates user input into application-level commands.
    Presentation,

    /// The layer that contains application-specific business logic and use cases.
    /// It orchestrates the domain layer to perform tasks and is the primary entry
    /// point for application operations.
    Application,

    /// The core of the application, containing enterprise-wide business logic and entities.
    /// This layer should be independent of any framework, UI, or database.
    Domain,

    /// The layer that contains all external concerns and implementation details, such as
    /// databases, file systems, and third-party API clients. It implements interfaces
    /// defined by the application or domain layers.
    Infrastructure,
}

/// Enumerates the specific types of leaky abstraction violations detected by this module.
#[derive(Debug, Clone)]
pub enum LeakType {
    /// A violation of encapsulation where code accesses a private or internal item
    /// from an outside module.
    VisibilityViolation,

    /// A dependency that flows in the wrong direction between architectural layers,
    /// such as a domain module depending on a presentation module.
    LayerViolation,

    /// Occurs when internal implementation types (e.g., a database model or ORM entity)
    /// are exposed through a module's public API.
    ImplementationExposure,

    /// Occurs when core business logic (domain or application layers) becomes directly
    /// dependent on types defined by a specific framework (e.g., using an Express `Request`
    /// object in a service class).
    FrameworkCoupling,

    /// Occurs when low-level, implementation-specific error types (e.g., `sql::Error`)
    /// are propagated across abstraction boundaries instead of being wrapped in
    /// domain-specific errors.
    ErrorPropagation,

    /// An abstraction that introduces significant, unexpected performance overhead,
    /// such as an Object-Relational Mapper (ORM) causing an N+1 query problem.
    PerformanceLeak,
}

/// Analysis context for leak detection operations.
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// The analysis run identifier
    pub analysis_run_id: i64,

    /// The current file path being analyzed
    pub file_path: String,

    /// The detected architectural layer for the current file
    pub current_layer: Option<ArchitecturalLayer>,
}

/// Result of interface analysis for a specific component.
#[derive(Debug, Clone)]
pub struct InterfaceAnalysisResult {
    /// Public API surface exposed by the component
    pub public_api: Vec<ApiElement>,

    /// Visibility violations detected
    pub visibility_violations: Vec<VisibilityViolation>,

    /// Interface contract issues
    pub contract_violations: Vec<ContractViolation>,
}

/// Represents an element in a public API.
#[derive(Debug, Clone)]
pub struct ApiElement {
    /// Name of the API element
    pub name: String,

    /// Type of the API element (function, struct, enum, etc.)
    pub element_type: String,

    /// Whether this element exposes internal implementation details
    pub exposes_internals: bool,

    /// Line number where the element is defined
    pub line_number: u32,
}

/// Represents a visibility violation.
#[derive(Debug, Clone)]
pub struct VisibilityViolation {
    /// Description of the violation
    pub description: String,

    /// The internal element being accessed inappropriately
    pub accessed_element: String,

    /// Line number where the violation occurs
    pub line_number: u32,

    /// Severity of the violation
    pub severity: String,
}

/// Represents a contract violation.
#[derive(Debug, Clone)]
pub struct ContractViolation {
    /// Description of the violation
    pub description: String,

    /// The contract that was violated
    pub violated_contract: String,

    /// Line number where the violation occurs
    pub line_number: u32,

    /// Severity of the violation
    pub severity: String,
}

/// Result of implementation analysis.
#[derive(Debug, Clone)]
pub struct ImplementationAnalysisResult {
    /// Detected implementation exposures
    pub implementation_exposures: Vec<ImplementationExposure>,

    /// Type leakages found
    pub type_leakages: Vec<TypeLeakage>,

    /// Implementation detail visibility issues
    pub visibility_issues: Vec<ImplementationVisibilityIssue>,
}

/// Represents an implementation exposure.
#[derive(Debug, Clone)]
pub struct ImplementationExposure {
    /// Description of the exposure
    pub description: String,

    /// The exposed implementation detail
    pub exposed_detail: String,

    /// Line number where the exposure occurs
    pub line_number: u32,

    /// Severity of the exposure
    pub severity: String,
}

/// Represents a type leakage.
#[derive(Debug, Clone)]
pub struct TypeLeakage {
    /// Description of the leakage
    pub description: String,

    /// The leaked type
    pub leaked_type: String,

    /// Line number where the leakage occurs
    pub line_number: u32,

    /// Severity of the leakage
    pub severity: String,
}

/// Represents an implementation visibility issue.
#[derive(Debug, Clone)]
pub struct ImplementationVisibilityIssue {
    /// Description of the issue
    pub description: String,

    /// The visibility problem
    pub visibility_problem: String,

    /// Line number where the issue occurs
    pub line_number: u32,

    /// Severity of the issue
    pub severity: String,
}
