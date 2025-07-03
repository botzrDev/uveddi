//! Universal anti-pattern detection tests
//!
//! Tests for anti-patterns that apply across all programming languages.

pub mod code_duplication_detection;
pub mod complexity_analysis;
pub mod debug_test;
pub mod global_state_pollution;
pub mod magic_values_detection;
pub mod mutable_defaults_detection;
pub mod resource_leak_detection;

// TODO: Add remaining universal anti-pattern tests:
// - silent_failure_detection
// - error_information_loss
// - premature_optimization
// - inappropriate_exception_types
// - inconsistent_naming_conventions
// - tight_coupling
// - insufficient_access_control
