use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Dependency relationship between code modules
///
/// Represents a dependency edge in the codebase dependency graph.
/// Dependencies track how different parts of the code reference each other,
/// enabling architectural analysis and cycle detection.
///
/// # Usage in Analysis
///
/// Dependencies are used for:
/// - Cycle detection in module relationships
/// - Architectural pattern recognition
/// - Impact analysis for changes
/// - Complexity measurement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    /// Path to the source file that has the dependency
    pub from_file: PathBuf,
    /// Name or path of the target module being depended upon
    pub to_module: String,
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    /// Line number where the dependency is declared (optional)
    pub line_number: Option<u32>,
}

/// Types of dependency relationships
///
/// Categorizes different ways that code can depend on other code,
/// enabling more sophisticated analysis of architectural patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub enum DependencyType {
    /// Rust `use` statement or similar language-specific import
    Use,
    /// Rust `mod` declaration for module inclusion
    Mod,
    /// External crate/package dependency
    External,
    /// Generic import statement (Python, JavaScript, etc.)
    Import,
    /// Data flow dependency (e.g., variable assignments, data transformations)
    DataFlow, // UV-2: Added for semantic dependency classification
    /// Control flow dependency (e.g., function calls, conditional execution)
    ControlFlow, // UV-2: Added for semantic dependency classification
}
