//! Error handling module for Uveddi
//!
//! This module provides comprehensive error handling for all Uveddi operations,
//! including the main UveddiError type and specialized error types for different
//! subsystems like rendering services.
//!
//! ## Unified Result Type
//!
//! All public APIs should use the unified `Result<T>` type for consistent error handling:
//!
//! ```rust
//! use uveddi::error::Result;
//!
//! pub fn analyze_file(path: &Path) -> Result<ParsedFile> {
//!     // Implementation using UveddiError
//! }
//! ```

pub mod helpers;
pub mod large_codebase;
pub mod main;
pub mod rendering;

pub use helpers::ErrorHelpers;
pub use large_codebase::{
    ErrorAggregator, ErrorContext, LargeCodebaseError, LargeCodebaseErrorHandler,
    NotificationSystem, ProgressState, ProgressTracker, RecoveryStrategies, RecoveryStrategy,
};
pub use main::{DeserializationError, ErrorCategory, ErrorHandler, ErrorSeverity, ExtractionError, UveddiError};
pub use rendering::RenderingServiceError;

// Re-export rusqlite error for convenience
pub use rusqlite::Error as RusqliteError;

/// Unified Result type for all Uveddi operations
///
/// This type alias provides consistent error handling across all public APIs.
/// All functions that can fail should return `Result<T>` instead of
/// `std::result::Result<T, SpecificError>`.
///
/// # Examples
///
/// ```rust
/// use uveddi::error::Result;
/// use std::path::Path;
///
/// pub fn parse_config_file(path: &Path) -> Result<Config> {
///     // Implementation that returns Result<Config, UveddiError>
/// }
/// ```
pub type Result<T> = std::result::Result<T, UveddiError>;
