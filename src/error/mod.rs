//! Error handling module for Uveddi
//!
//! This module provides comprehensive error handling for all Uveddi operations,
//! including the main UveddiError type and specialized error types for different
//! subsystems like rendering services.

pub mod main;
pub mod rendering;

pub use main::{UveddiError, ErrorCategory, ErrorSeverity, ExtractionError};
pub use rendering::RenderingServiceError;

// Re-export rusqlite error for convenience
pub use rusqlite::Error as RusqliteError;
