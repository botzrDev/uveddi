//! Unified logging module for Uveddi
//!
//! This module provides a consistent logging interface across the entire
//! application, replacing the legacy `log` crate with `tracing` for better
//! structured logging capabilities.

pub mod unified;

pub use unified::{
    debug, error, info, init_logging, init_logging_with_config, instrument, span, trace, warn,
    Level, LoggingError,
};
