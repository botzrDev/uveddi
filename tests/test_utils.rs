//! Shared test utilities for integration and unit tests in Uveddi
//!
//! This module provides common setup functions, fixtures, and helpers to reduce code duplication.

use std::path::PathBuf;

/// Returns a temporary directory for test isolation.
pub fn temp_test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp test dir")
}

/// Returns a sample config for tests.
pub fn sample_config() -> uveddi::config::Config {
    uveddi::config::Config {
        openai_api_key: Some("test-key".to_string()),
        anthropic_api_key: None,
        gemini_api_key: None,
        ollama_model: None,
    }
}

// Add more helpers as needed for database setup, mock data, etc.
