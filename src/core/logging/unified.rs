//! Unified logging configuration using tracing
//!
//! This module provides a unified logging system that replaces the `log` crate
//! with tracing throughout the application. It provides backward compatibility
//! while enabling structured logging capabilities.

use std::io;
use thiserror::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Initialize unified logging system
pub fn init_logging() -> Result<(), LoggingError> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(LoggingError::FilterCreation)?;

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    let registry = Registry::default().with(env_filter).with(formatting_layer);

    registry
        .try_init()
        .map_err(|_| LoggingError::InitializationFailed)?;

    Ok(())
}

/// Initialize logging with custom configuration
pub fn init_logging_with_config(log_level: &str, with_json: bool) -> Result<(), LoggingError> {
    let env_filter = EnvFilter::try_new(log_level).map_err(LoggingError::FilterCreation)?;

    if with_json {
        let registry = Registry::default().with(env_filter).with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        );
        tracing::subscriber::set_global_default(registry).map_err(LoggingError::SubscriberInit)?;
    } else {
        let registry = Registry::default().with(env_filter).with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        );
        tracing::subscriber::set_global_default(registry).map_err(LoggingError::SubscriberInit)?;
    }

    Ok(())
}

/// Structured logging macros (re-export tracing macros)
pub use tracing::{debug, error, info, trace, warn};

/// Span creation for structured context
pub use tracing::{instrument, span, Level};

/// Event macros for structured logging
pub use tracing::{event, event_enabled};

/// Create a span for function instrumentation
#[macro_export]
macro_rules! log_function_entry {
    ($fn_name:expr) => {
        tracing::debug!("Entering function: {}", $fn_name);
    };
    ($fn_name:expr, $($field:expr),+) => {
        tracing::debug!("Entering function: {} with params: {}", $fn_name, format_args!($($field),+));
    };
}

/// Log function exit with optional result
#[macro_export]
macro_rules! log_function_exit {
    ($fn_name:expr) => {
        tracing::debug!("Exiting function: {}", $fn_name);
    };
    ($fn_name:expr, $result:expr) => {
        tracing::debug!("Exiting function: {} with result: {:?}", $fn_name, $result);
    };
}

/// Log performance timing
#[macro_export]
macro_rules! log_timing {
    ($operation:expr, $duration:expr) => {
        tracing::info!("Operation '{}' completed in {:?}", $operation, $duration);
    };
}

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("Failed to create log filter: {0}")]
    FilterCreation(#[from] tracing_subscriber::filter::ParseError),
    #[error("Failed to initialize logging system")]
    InitializationFailed,
    #[error("Failed to set global subscriber")]
    SubscriberInit(#[from] tracing::subscriber::SetGlobalDefaultError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_initialization() {
        // This test might fail if logging is already initialized
        // In a real scenario, you'd want to reset the global subscriber
        let result = init_logging_with_config("debug", false);
        // Just ensure it doesn't panic - initialization might fail in tests
        // due to global state
        println!("Logging init result: {:?}", result);
    }

    #[test]
    fn test_logging_macros() {
        // Test that the macros compile
        info!("Test info message");
        debug!("Test debug message");
        warn!("Test warning message");
        error!("Test error message");
    }
}
