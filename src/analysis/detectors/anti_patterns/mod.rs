//! Anti-pattern detectors
//!
//! This module contains detectors for various anti-patterns and code smells,
//! including God Objects, inappropriate inheritance, and other structural issues.

pub mod god_object;
pub mod code_duplication;
pub mod magic_values;
pub mod global_state;
pub mod resource_leak;
pub mod leaky_abstraction;
pub mod tight_coupling;
pub mod state_synchronization;
pub mod insufficient_access_control;
pub mod silent_failure;
pub mod premature_optimization;
pub mod error_information_loss;
pub mod inappropriate_exception_type;
pub mod inconsistent_naming_convention;

pub use god_object::GodObjectDetector;
pub use code_duplication::CodeDuplicationDetector;
pub use magic_values::MagicValuesDetector;
pub use global_state::GlobalStateDetector;
pub use resource_leak::ResourceLeakDetector;
pub use leaky_abstraction::LeakyAbstractionDetector;
pub use tight_coupling::TightCouplingDetector;
pub use state_synchronization::StateSynchronizationDetector;
pub use insufficient_access_control::InsufficientAccessControlDetector;
pub use silent_failure::SilentFailureDetector;
pub use premature_optimization::PrematureOptimizationDetector;
pub use error_information_loss::ErrorInformationLossDetector;
pub use inappropriate_exception_type::InappropriateExceptionTypeDetector;
pub use inconsistent_naming_convention::InconsistentNamingConventionDetector;
