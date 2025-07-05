//! Shared test utilities for integration and unit tests in Uveddi
//!
//! This module provides common setup functions, fixtures, and helpers to reduce code duplication.

/// Returns a temporary directory for test isolation.
pub fn temp_test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp test dir")
}

/// Returns a sample config for tests.
pub fn sample_config() -> uveddi::config::Config {
    uveddi::config::Config {
        ollama_model: Some("deepseek-coder:6.7b-instruct-q4_0".to_string()),
    }
}

// Add more helpers as needed for database setup, mock data, etc.
