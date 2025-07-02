//! Language-specific anti-pattern detection tests
//!
//! Tests for anti-patterns specific to individual programming languages.

// JavaScript tests
pub mod js_async_antipatterns;
pub mod js_scope_issues;
pub mod js_type_coercion;

// Python tests
pub mod py_data_structure_misuse;

// Rust tests
pub mod rust_error_handling;
pub mod rust_ownership_issues;

// Java tests
pub mod java_oop_issues;

// TODO: Add remaining language-specific tests:
// JavaScript:
// - js_dom_issues
// - js_module_dependency_issues
//
// Python:
// - py_oop_issues
// - py_import_issues
// - py_performance_issues
// - py_exception_handling
//
// Java:
// - java_concurrency_issues
// - java_type_system_issues
// - java_resource_management
// - java_design_pattern_misuse
//
// Rust:
// - rust_memory_issues
// - rust_unsafe_issues
// - rust_type_system_misuse
// - rust_concurrency_issues
