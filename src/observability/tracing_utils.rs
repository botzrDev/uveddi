//! Tracing utilities for correlation and context management
//!
//! Provides utilities for trace ID generation and correlation across the observability system.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use tracing::{field, Span};
use uuid::Uuid;

/// A unique trace identifier for correlating logs, metrics, and audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(Uuid);

impl TraceId {
    /// Create a new random trace ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a trace ID from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get the trace ID as a string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Parse a trace ID from a string
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for TraceId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<TraceId> for String {
    fn from(trace_id: TraceId) -> Self {
        trace_id.as_str()
    }
}

/// Generate a new trace ID
pub fn generate_trace_id() -> TraceId {
    TraceId::new()
}

/// Execute a closure with a trace ID attached to the current span
pub fn with_trace_id<F, R>(trace_id: TraceId, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::Span::current();
    span.record("trace_id", field::display(&trace_id));
    f()
}

/// Attach a trace ID to the current span
pub fn attach_trace_id(trace_id: TraceId) {
    let span = tracing::Span::current();
    span.record("trace_id", field::display(&trace_id));
}

/// Create a new span with a trace ID
#[macro_export]
macro_rules! traced_span {
    ($level:expr, $name:expr, $trace_id:expr) => {
        tracing::span!($level, $name, trace_id = %$trace_id)
    };
    ($level:expr, $name:expr, $trace_id:expr, $($field:tt)*) => {
        tracing::span!($level, $name, trace_id = %$trace_id, $($field)*)
    };
}

/// Create an info-level span with a trace ID
#[macro_export]
macro_rules! info_span_traced {
    ($name:expr, $trace_id:expr) => {
        $crate::traced_span!(tracing::Level::INFO, $name, $trace_id)
    };
    ($name:expr, $trace_id:expr, $($field:tt)*) => {
        $crate::traced_span!(tracing::Level::INFO, $name, $trace_id, $($field)*)
    };
}

/// Create a debug-level span with a trace ID
#[macro_export]
macro_rules! debug_span_traced {
    ($name:expr, $trace_id:expr) => {
        $crate::traced_span!(tracing::Level::DEBUG, $name, $trace_id)
    };
    ($name:expr, $trace_id:expr, $($field:tt)*) => {
        $crate::traced_span!(tracing::Level::DEBUG, $name, $trace_id, $($field)*)
    };
}

/// Create an error-level span with a trace ID
#[macro_export]
macro_rules! error_span_traced {
    ($name:expr, $trace_id:expr) => {
        $crate::traced_span!(tracing::Level::ERROR, $name, $trace_id)
    };
    ($name:expr, $trace_id:expr, $($field:tt)*) => {
        $crate::traced_span!(tracing::Level::ERROR, $name, $trace_id, $($field)*)
    };
}

/// Extract trace ID from the current span, if available
pub fn current_trace_id() -> Option<TraceId> {
    let span = Span::current();
    // This is a simplified implementation
    // In a real implementation, you'd need to extract the trace_id field from the span
    // For now, we'll return None and rely on explicit trace ID management
    None
}

/// Trait for types that can provide a trace ID
pub trait HasTraceId {
    fn trace_id(&self) -> TraceId;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_creation() {
        let trace_id = TraceId::new();
        assert!(!trace_id.as_str().is_empty());
    }

    #[test]
    fn test_trace_id_roundtrip() {
        let original = TraceId::new();
        let as_string = original.as_str();
        let parsed = TraceId::from_str(&as_string).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_trace_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let trace_id = TraceId::from_uuid(uuid);
        assert_eq!(trace_id.as_uuid(), uuid);
    }
}
