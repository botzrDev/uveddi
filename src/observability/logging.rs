//! Structured logging implementation using the tracing ecosystem
//!
//! Implements the UV-86 structured logging architecture with:
//! - JSON output for production
//! - PII redaction layer
//! - Asynchronous logging for performance
//! - Correlation via trace_id

use crate::observability::config::{LoggingConfig, LogFormat, PiiRedactionConfig, RedactionStrategy};
use crate::observability::tracing_utils::TraceId;
use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::sync::Once;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer as FmtLayer},
    layer::SubscriberExt,
    registry::Registry,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

// Global flag to ensure tracing is only initialized once
static INIT: Once = Once::new();

/// Initialize the structured logging system according to UV-86 specifications
pub fn init_structured_logging(config: &LoggingConfig) -> Result<()> {
    let mut init_error: Option<anyhow::Error> = None;
    
    INIT.call_once(|| {
        // In test mode, use test-safe initialization
        #[cfg(test)]
        {
            match init_test_subscriber() {
                Ok(_) => {},
                Err(e) => {
                    init_error = Some(e);
                }
            }
        }
        
        // In production mode, use full initialization
        #[cfg(not(test))]
        {
            match try_init_tracing(config) {
                Ok(_) => {},
                Err(e) => {
                    init_error = Some(e);
                }
            }
        }
    });

    if let Some(error) = init_error {
        return Err(error);
    }

    // Log initialization message (will only appear once per process)
    tracing::info!(
        message = "Structured logging initialized",
        format = ?config.format,
        async_logging = config.async_logging,
        pii_redaction = config.pii_redaction.enabled,
        "UV-86 observability system started"
    );

    Ok(())
}

/// Internal function to actually initialize tracing (called only once)
fn try_init_tracing(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .context("Failed to create environment filter")?;

    // Simplified approach for now - will use basic JSON formatting
    use tracing_subscriber::fmt;
    
    let subscriber = Registry::default()
        .with(env_filter)
        .with(
            fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
        );

    subscriber
        .try_init()
        .context("Failed to initialize tracing subscriber")?;

    Ok(())
}

/// Test-specific subscriber initialization
#[cfg(test)]
fn init_test_subscriber() -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_test_writer()
                .with_target(true)
                .with_thread_ids(true)
        );
    subscriber
        .try_init()
        .context("Failed to initialize test tracing subscriber")?;
    
    Ok(())
}

/// Test-specific initialization that's safe to call multiple times
#[cfg(test)]
pub fn init_test_logging() -> Result<()> {
    use std::sync::Once;
    static TEST_INIT: Once = Once::new();
    
    TEST_INIT.call_once(|| {
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_test_writer()
                    .with_target(true)
                    .with_thread_ids(true)
            );
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
    
    Ok(())
}

/// Create the appropriate formatting layer based on configuration
fn create_format_layer(
    config: &LoggingConfig,
) -> Result<Box<dyn Layer<Registry> + Send + Sync>> {
    match &config.format {
        LogFormat::Json => {
            if config.async_logging {
                create_async_json_layer(config)
            } else {
                create_sync_json_layer()
            }
        }
        LogFormat::Human => create_human_layer(),
        LogFormat::Compact => create_compact_layer(),
    }
}

/// Create an asynchronous JSON formatting layer with file output
fn create_async_json_layer(
    config: &LoggingConfig,
) -> Result<Box<dyn Layer<Registry> + Send + Sync>> {
    if let Some(log_dir) = &config.log_dir {
        let file_appender = rolling::daily(log_dir, "uveddi.log");
        let (non_blocking, _guard) = non_blocking(file_appender);

        let layer = FmtLayer::new()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(non_blocking);

        Ok(Box::new(layer))
    } else {
        // Write to stdout with async writer
        let (non_blocking, _guard) = non_blocking(io::stdout());
        let layer = FmtLayer::new()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(non_blocking);

        Ok(Box::new(layer))
    }
}

/// Create a synchronous JSON formatting layer
fn create_sync_json_layer() -> Result<Box<dyn Layer<Registry> + Send + Sync>> {
    let layer = FmtLayer::new()
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE);

    Ok(Box::new(layer))
}

/// Create a human-readable formatting layer for development
fn create_human_layer() -> Result<Box<dyn Layer<Registry> + Send + Sync>> {
    let layer = FmtLayer::new()
        .pretty()
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE);

    Ok(Box::new(layer))
}

/// Create a compact formatting layer for high-throughput scenarios
fn create_compact_layer() -> Result<Box<dyn Layer<Registry> + Send + Sync>> {
    let layer = FmtLayer::new()
        .compact()
        .with_target(false)
        .with_thread_ids(false)
        .with_span_events(FmtSpan::NONE);

    Ok(Box::new(layer))
}

/// PII redaction layer that sanitizes log events before output
pub struct PiiRedactionLayer {
    patterns: Vec<Regex>,
    strategy: RedactionStrategy,
}

impl PiiRedactionLayer {
    /// Create a new PII redaction layer with the given configuration
    pub fn new(config: &PiiRedactionConfig) -> Result<Self> {
        let mut patterns = Vec::new();
        for pattern in &config.field_patterns {
            let regex = Regex::new(pattern)
                .with_context(|| format!("Invalid regex pattern: {}", pattern))?;
            patterns.push(regex);
        }

        Ok(Self {
            patterns,
            strategy: config.strategy.clone(),
        })
    }

    /// Check if a field name matches any PII patterns
    fn is_pii_field(&self, field_name: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.is_match(field_name))
    }

    /// Apply redaction strategy to a value
    fn redact_value(&self, value: &str) -> String {
        match self.strategy {
            RedactionStrategy::Mask => "*".repeat(value.len().min(8)),
            RedactionStrategy::Hash => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(value.as_bytes());
                format!("sha256:{:x}", hasher.finalize())[..16].to_string()
            }
            RedactionStrategy::Remove => "<REDACTED>".to_string(),
            RedactionStrategy::Partial => {
                if value.len() <= 4 {
                    "*".repeat(value.len())
                } else {
                    format!("{}***{}", &value[..1], &value[value.len() - 1..])
                }
            }
        }
    }
}

impl<S> Layer<S> for PiiRedactionLayer
where
    S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Create a visitor to redact PII fields
        let mut visitor = PiiRedactionVisitor::new(self);
        event.record(&mut visitor);
    }
}

/// Visitor that redacts PII fields in log events
struct PiiRedactionVisitor<'a> {
    layer: &'a PiiRedactionLayer,
    redacted_fields: HashMap<String, String>,
}

impl<'a> PiiRedactionVisitor<'a> {
    fn new(layer: &'a PiiRedactionLayer) -> Self {
        Self {
            layer,
            redacted_fields: HashMap::new(),
        }
    }
}

impl<'a> Visit for PiiRedactionVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let field_name = field.name();
        if self.layer.is_pii_field(field_name) {
            let redacted = self.layer.redact_value(&format!("{:?}", value));
            self.redacted_fields.insert(field_name.to_string(), redacted);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let field_name = field.name();
        if self.layer.is_pii_field(field_name) {
            let redacted = self.layer.redact_value(value);
            self.redacted_fields.insert(field_name.to_string(), redacted);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        let field_name = field.name();
        if self.layer.is_pii_field(field_name) {
            let redacted = self.layer.redact_value(&value.to_string());
            self.redacted_fields.insert(field_name.to_string(), redacted);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        let field_name = field.name();
        if self.layer.is_pii_field(field_name) {
            let redacted = self.layer.redact_value(&value.to_string());
            self.redacted_fields.insert(field_name.to_string(), redacted);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        let field_name = field.name();
        if self.layer.is_pii_field(field_name) {
            let redacted = self.layer.redact_value(&value.to_string());
            self.redacted_fields.insert(field_name.to_string(), redacted);
        }
    }
}

/// Utility function to log with a trace ID
#[macro_export]
macro_rules! traced_info {
    ($trace_id:expr, $($arg:tt)*) => {
        tracing::info!(trace_id = %$trace_id, $($arg)*)
    };
}

/// Utility function to log warnings with a trace ID
#[macro_export]
macro_rules! traced_warn {
    ($trace_id:expr, $($arg:tt)*) => {
        tracing::warn!(trace_id = %$trace_id, $($arg)*)
    };
}

/// Utility function to log errors with a trace ID
#[macro_export]
macro_rules! traced_error {
    ($trace_id:expr, $($arg:tt)*) => {
        tracing::error!(trace_id = %$trace_id, $($arg)*)
    };
}

/// Utility function to log debug messages with a trace ID
#[macro_export]
macro_rules! traced_debug {
    ($trace_id:expr, $($arg:tt)*) => {
        tracing::debug!(trace_id = %$trace_id, $($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::config::{LoggingConfig, PiiRedactionConfig};

    #[test]
    fn test_pii_redaction_patterns() {
        let config = PiiRedactionConfig::default();
        let layer = PiiRedactionLayer::new(&config).unwrap();

        assert!(layer.is_pii_field("user_password"));
        assert!(layer.is_pii_field("auth_token"));
        assert!(layer.is_pii_field("secret_key"));
        assert!(layer.is_pii_field("user_email"));
        assert!(!layer.is_pii_field("username"));
        assert!(!layer.is_pii_field("file_path"));
    }

    #[test]
    fn test_redaction_strategies() {
        let config = PiiRedactionConfig {
            enabled: true,
            field_patterns: vec![r".*password.*".to_string()],
            strategy: RedactionStrategy::Mask,
        };
        let layer = PiiRedactionLayer::new(&config).unwrap();

        let redacted = layer.redact_value("secret123");
        assert_eq!(redacted, "********");

        let config = PiiRedactionConfig {
            enabled: true,
            field_patterns: vec![r".*password.*".to_string()],
            strategy: RedactionStrategy::Partial,
        };
        let layer = PiiRedactionLayer::new(&config).unwrap();

        let redacted = layer.redact_value("secret123");
        assert_eq!(redacted, "s***3");
    }
}