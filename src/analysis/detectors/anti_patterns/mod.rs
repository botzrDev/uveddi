//! Anti-pattern detectors
//!
//! This module contains detectors for various anti-patterns and code smells,
//! including God Objects, inappropriate inheritance, and other structural issues.
//!
//! The detectors are organized into:
//! - **Universal detectors**: Work across multiple languages (in this directory)
//! - **Language-specific detectors**: Specialized for particular languages (in subdirectories)

// Universal anti-pattern detectors
pub mod code_duplication;
pub mod error_information_loss;
pub mod global_state;
pub mod god_object;
pub mod inappropriate_exception_type;
pub mod inconsistent_naming_convention;
pub mod insufficient_access_control;
pub mod leaky_abstraction;
pub mod magic_values;
pub mod premature_optimization;
pub mod resource_leak;
pub mod silent_failure;
pub mod state_synchronization;
pub mod tight_coupling;

// Language-specific detector modules
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

// Re-exports for universal detectors
pub use code_duplication::CodeDuplicationDetector;
pub use error_information_loss::ErrorInformationLossDetector;
pub use global_state::GlobalStateDetector;
pub use god_object::GodObjectDetector;
pub use inappropriate_exception_type::InappropriateExceptionTypeDetector;
pub use inconsistent_naming_convention::InconsistentNamingConventionDetector;
pub use insufficient_access_control::InsufficientAccessControlDetector;
pub use leaky_abstraction::LeakyAbstractionDetector;
pub use magic_values::MagicValuesDetector;
pub use premature_optimization::PrematureOptimizationDetector;
pub use resource_leak::ResourceLeakDetector;
pub use silent_failure::SilentFailureDetector;
pub use state_synchronization::StateSynchronizationDetector;
pub use tight_coupling::TightCouplingDetector;

// Re-exports for language-specific detectors
pub use java::{
    JavaConcurrencyDetector, JavaOopIssuesDetector, JavaResourceManagementDetector,
    JavaTypeSystemDetector,
};
pub use javascript::{
    JsAsyncAntipatternsDetector, JsDomIssuesDetector, JsModuleDependencyDetector,
    JsScopeIssuesDetector, JsTypeCoercionDetector,
};
pub use python::{
    PyDataStructureMisuseDetector, PyExceptionHandlingDetector, PyOopIssuesDetector,
    PyPerformanceIssuesDetector,
};
pub use rust::{
    CloneAbuseDetector, LifetimeComplexityDetector, MemoryManagementDetector, UnwrapAbuseDetector,
};
