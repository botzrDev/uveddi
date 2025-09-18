//! Comprehensive test suite for God Object Anti-Pattern Detector

mod unit_tests;
mod language_tests;
mod edge_case_tests;
mod integration_tests;

// Re-export test utilities for other modules
pub use unit_tests::create_parsed_file;

// Test configuration constants
pub const DEFAULT_METHOD_THRESHOLD: usize = 10;
pub const DEFAULT_FIELD_THRESHOLD: usize = 8;