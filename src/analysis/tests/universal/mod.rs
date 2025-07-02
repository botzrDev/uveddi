//! Universal anti-pattern detection tests
//! 
//! Tests for anti-patterns that apply across all programming languages.

pub mod resource_leak_detection;
pub mod global_state_pollution;
pub mod mutable_defaults_detection;
pub mod magic_values_detection;
pub mod code_duplication_detection;
pub mod complexity_analysis;

// TODO: Add remaining universal anti-pattern tests:
// - silent_failure_detection
// - error_information_loss
// - premature_optimization
// - inappropriate_exception_types
// - inconsistent_naming_conventions
// - tight_coupling
// - insufficient_access_control
