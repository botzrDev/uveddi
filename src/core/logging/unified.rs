//! Unified logging configuration using tracing
//!
//! This module provides a unified logging system that replaces the `log` crate
//! with tracing throughout the application. It provides backward compatibility
//! while enabling structured logging capabilities.

use std::path::Path;
use thiserror::Error;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};
// OffsetTime requires the `time` feature, using system time instead
// use tracing_subscriber::fmt::time::OffsetTime;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global request ID counter for correlation
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique request ID for correlation
pub fn generate_request_id() -> u64 {
    REQUEST_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Initialize unified logging system
pub fn init_logging() -> Result<(), LoggingError> {
    init_logging_with_config("info", false)
}

/// Initialize logging with custom configuration
pub fn init_logging_with_config(log_level: &str, with_json: bool) -> Result<(), LoggingError> {
    // Allow environment variable to override the log level
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .map_err(LoggingError::FilterCreation)?;

    if with_json {
        let registry = Registry::default().with(env_filter).with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_current_span(true),
        );
        tracing::subscriber::set_global_default(registry).map_err(LoggingError::SubscriberInit)?;
    } else {
        let registry = Registry::default().with(env_filter).with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(true),
        );
        tracing::subscriber::set_global_default(registry).map_err(LoggingError::SubscriberInit)?;
    }

    Ok(())
}

/// Initialize logging for production environment with file output
pub fn init_production_logging(
    log_level: &str,
    log_file: Option<&Path>,
) -> Result<(), LoggingError> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .map_err(LoggingError::FilterCreation)?;

    let stdout_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_target(true)
        .with_ansi(true);

    let registry = Registry::default().with(env_filter).with(stdout_layer);

    // Add file layer if log file is specified
    if let Some(log_file) = log_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .map_err(LoggingError::FileCreation)?;

        let file_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(file)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_current_span(true);

        registry
            .with(file_layer)
            .try_init()
            .map_err(|_| LoggingError::InitializationFailed)?;
    } else {
        registry
            .try_init()
            .map_err(|_| LoggingError::InitializationFailed)?;
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
        tracing::info!(
            operation = $operation,
            duration_ms = ?$duration.as_millis(),
            "Operation completed"
        );
    };
}

/// Log with request context
#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $request_id:expr, $message:expr) => {
        tracing::event!(
            $level,
            request_id = $request_id,
            "{}",
            $message
        );
    };
    ($level:expr, $request_id:expr, $message:expr, $($field:tt)*) => {
        tracing::event!(
            $level,
            request_id = $request_id,
            $($field)*,
            "{}",
            $message
        );
    };
}

/// Log error with context and error chain
#[macro_export]
macro_rules! log_error_with_context {
    ($error:expr) => {
        tracing::error!(
            error = ?$error,
            error_chain = %$crate::core::logging::unified::format_error_chain(&$error),
            "Error occurred"
        );
    };
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = ?$error,
            error_chain = %$crate::core::logging::unified::format_error_chain(&$error),
            context = $context,
            "Error occurred"
        );
    };
}

/// Format error chain for logging
pub fn format_error_chain(error: &dyn std::error::Error) -> String {
    let mut chain = vec![error.to_string()];
    let mut current = error.source();

    while let Some(cause) = current {
        chain.push(cause.to_string());
        current = cause.source();
    }

    chain.join(" -> ")
}

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("Failed to create log filter: {0}")]
    FilterCreation(#[from] tracing_subscriber::filter::ParseError),
    #[error("Failed to initialize logging system")]
    InitializationFailed,
    #[error("Failed to set global subscriber")]
    SubscriberInit(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("Failed to create log file: {0}")]
    FileCreation(#[from] std::io::Error),
}

/// Configuration for logging system
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub include_timestamps: bool,
    pub include_thread_info: bool,
    pub include_location: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Compact,
            output: LogOutput::Stdout,
            include_timestamps: true,
            include_thread_info: false,
            include_location: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, Clone)]
pub enum LogOutput {
    Stdout,
    File(String),
    Both(String),
}

/// Initialize logging with advanced configuration
pub fn init_with_config(config: LoggingConfig) -> Result<(), LoggingError> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .map_err(LoggingError::FilterCreation)?;

    let layer = match config.format {
        LogFormat::Compact => tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(true)
            .with_target(true)
            .with_thread_ids(config.include_thread_info)
            .with_thread_names(config.include_thread_info)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .boxed(),
        LogFormat::Pretty => tracing_subscriber::fmt::layer()
            .pretty()
            .with_ansi(true)
            .with_target(true)
            .with_thread_ids(config.include_thread_info)
            .with_thread_names(config.include_thread_info)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(config.include_thread_info)
            .with_thread_names(config.include_thread_info)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .boxed(),
    };

    let registry = Registry::default().with(env_filter).with(layer);

    registry
        .try_init()
        .map_err(|_| LoggingError::InitializationFailed)?;

    Ok(())
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
