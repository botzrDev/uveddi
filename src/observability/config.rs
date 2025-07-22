//! Observability Configuration
//!
//! Provides configuration structures for all observability components following
//! the UV-86 specification for enterprise-grade observability.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the observability system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Resilience configuration
    pub resilience: ResilienceConfig,
    /// Security integration configuration
    pub security: SecurityConfig,
}

/// Configuration for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Logging level (error, warn, info, debug, trace)
    pub level: String,
    /// Output format: "human" for development, "json" for production
    pub format: LogFormat,
    /// Directory for log files (if file output is enabled)
    pub log_dir: Option<PathBuf>,
    /// Enable asynchronous logging for performance
    pub async_logging: bool,
    /// PII redaction configuration
    pub pii_redaction: PiiRedactionConfig,
    /// Log rotation configuration
    pub rotation: LogRotationConfig,
}

/// Log output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogFormat {
    /// Human-readable format for development
    Human,
    /// JSON format for production log aggregation
    Json,
    /// Compact format for high-throughput scenarios
    Compact,
}

/// PII redaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiRedactionConfig {
    /// Enable PII redaction
    pub enabled: bool,
    /// Field patterns to redact (regex patterns)
    pub field_patterns: Vec<String>,
    /// Redaction strategy
    pub strategy: RedactionStrategy,
}

/// Strategy for redacting sensitive data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedactionStrategy {
    /// Replace with asterisks (e.g., "password" -> "****")
    Mask,
    /// Hash the value using SHA-256
    Hash,
    /// Completely remove the field
    Remove,
    /// Show only first/last characters (e.g., "email@domain.com" -> "e***@d***e.com")
    Partial,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum file size before rotation (in bytes)
    pub max_file_size: u64,
    /// Maximum number of archived log files
    pub max_files: u32,
    /// Compress rotated files
    pub compress: bool,
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics server bind address
    pub bind_address: String,
    /// Metrics server port
    pub port: u16,
    /// Metrics endpoint path (default: "/metrics")
    pub endpoint_path: String,
    /// Collection interval for system metrics
    pub collection_interval: Duration,
    /// Service Level Objectives configuration
    pub slo_config: SloConfig,
}

/// Service Level Objectives configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloConfig {
    /// Target availability percentage (e.g., 99.9)
    pub availability_target: f64,
    /// Target P99 latency in milliseconds
    pub latency_p99_target_ms: u64,
    /// Error budget window in days
    pub error_budget_window_days: u32,
}

/// Configuration for resilience patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Fallback configuration
    pub fallback: FallbackConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure rate threshold (0.0-1.0) to open the circuit
    pub failure_rate_threshold: f64,
    /// Minimum number of calls before circuit can open
    pub minimum_throughput: u32,
    /// Duration to wait before transitioning from Open to Half-Open
    pub wait_duration: Duration,
    /// Number of permitted calls in Half-Open state
    pub permitted_calls_in_half_open_state: u32,
    /// Sliding window size for failure rate calculation
    pub sliding_window_size: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Enable jitter to prevent thundering herd
    pub jitter: bool,
}

/// Fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Enable cache-based fallbacks
    pub enable_cache_fallback: bool,
    /// Cache TTL for fallback data
    pub cache_ttl: Duration,
    /// Enable partial result returns
    pub enable_partial_results: bool,
}

/// Configuration for security integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable integration with UV-247 audit system
    pub enable_audit_integration: bool,
    /// UV-247 audit service endpoint
    pub audit_service_url: Option<String>,
    /// Export security events as metrics
    pub export_security_metrics: bool,
    /// Security event sampling rate (0.0-1.0)
    pub event_sampling_rate: f64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
            resilience: ResilienceConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Json,
            log_dir: None,
            async_logging: true,
            pii_redaction: PiiRedactionConfig::default(),
            rotation: LogRotationConfig::default(),
        }
    }
}

impl Default for PiiRedactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            field_patterns: vec![
                r".*password.*".to_string(),
                r".*token.*".to_string(),
                r".*secret.*".to_string(),
                r".*key.*".to_string(),
                r".*email.*".to_string(),
                r".*phone.*".to_string(),
                r".*ssn.*".to_string(),
                r".*credit.*card.*".to_string(),
            ],
            strategy: RedactionStrategy::Mask,
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            compress: true,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "0.0.0.0".to_string(),
            port: 9090,
            endpoint_path: "/metrics".to_string(),
            collection_interval: Duration::from_secs(30),
            slo_config: SloConfig::default(),
        }
    }
}

impl Default for SloConfig {
    fn default() -> Self {
        Self {
            availability_target: 99.9,
            latency_p99_target_ms: 5000,
            error_budget_window_days: 28,
        }
    }
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            circuit_breaker: CircuitBreakerConfig::default(),
            retry: RetryConfig::default(),
            fallback: FallbackConfig::default(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_rate_threshold: 0.5, // 50% failure rate
            minimum_throughput: 10,
            wait_duration: Duration::from_secs(60),
            permitted_calls_in_half_open_state: 5,
            sliding_window_size: 100,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enable_cache_fallback: true,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            enable_partial_results: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_audit_integration: true,
            audit_service_url: None,
            export_security_metrics: true,
            event_sampling_rate: 1.0, // 100% sampling by default
        }
    }
}
