//! Error handling module for Uveddi
//!
//! This module provides comprehensive error handling for all Uveddi operations,
//! including the main UveddiError type and specialized error types for different
//! subsystems like rendering services.

pub mod main;
pub mod rendering;

pub use main::UveddiError;
pub use rendering::{ErrorCategory, ErrorSeverity, RenderingServiceError};
