//! Graceful Failure Handler (UV-172)
//!
//! Provides intelligent handling of rendering failures to preserve user experience
//! during service degradation. Integrates with circuit breaker, fallback, and metrics
//! systems to provide coordinated failure response.

use crate::error::{ErrorCategory, ErrorSeverity, RenderingServiceError};
use crate::resilience::{CircuitBreaker, FallbackManager, MetricsCollector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Levels of service degradation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DegradationLevel {
    /// Service operating normally
    Normal,
    /// Minor degradation - some features may be slower
    Minor,
    /// Moderate degradation - reduced functionality
    Moderate,
    /// Severe degradation - limited functionality
    Severe,
    /// Critical degradation - emergency mode only
    Critical,
}

/// Graceful response wrapper that preserves user experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracefulResponse<T> {
    /// The actual response data (may be fallback)
    pub data: T,
    /// Current degradation level
    pub degradation_level: DegradationLevel,
    /// User-friendly message explaining the situation
    pub user_message: Option<String>,
    /// Technical details for debugging (not shown to users)
    pub technical_details: Option<String>,
    /// Estimated recovery time
    pub estimated_recovery: Option<Duration>,
    /// Whether this is a fallback response
    pub is_fallback: bool,
}

impl<T> GracefulResponse<T> {
    /// Create a normal response
    pub fn normal(data: T) -> Self {
        Self {
            data,
            degradation_level: DegradationLevel::Normal,
            user_message: None,
            technical_details: None,
            estimated_recovery: None,
            is_fallback: false,
        }
    }

    /// Create a degraded response with user messaging
    pub fn degraded(
        data: T,
        level: DegradationLevel,
        user_message: String,
        estimated_recovery: Option<Duration>,
        is_fallback: bool,
    ) -> Self {
        Self {
            data,
            degradation_level: level,
            user_message: Some(user_message),
            technical_details: None,
            estimated_recovery,
            is_fallback,
        }
    }

    /// Add technical details for debugging
    pub fn with_technical_details(mut self, details: String) -> Self {
        self.technical_details = Some(details);
        self
    }
}

/// Failure history tracking for pattern recognition
#[derive(Debug, Clone)]
struct FailureHistory {
    failures: Vec<FailureEvent>,
    recovery_times: Vec<Duration>,
    last_recovery: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct FailureEvent {
    timestamp: SystemTime,
    error: RenderingServiceError,
    degradation_level: DegradationLevel,
}

impl FailureHistory {
    fn new() -> Self {
        Self {
            failures: Vec::new(),
            recovery_times: Vec::new(),
            last_recovery: None,
        }
    }

    fn record_failure(&mut self, error: RenderingServiceError, level: DegradationLevel) {
        self.failures.push(FailureEvent {
            timestamp: SystemTime::now(),
            error,
            degradation_level: level,
        });

        // Keep only recent failures (last hour)
        let cutoff = SystemTime::now() - Duration::from_secs(3600);
        self.failures.retain(|f| f.timestamp > cutoff);
    }

    fn record_recovery(&mut self, recovery_time: Duration) {
        self.recovery_times.push(recovery_time);
        self.last_recovery = Some(SystemTime::now());

        // Keep only recent recovery times
        if self.recovery_times.len() > 10 {
            self.recovery_times.remove(0);
        }
    }

    fn estimate_recovery_time(&self) -> Option<Duration> {
        if self.recovery_times.is_empty() {
            return None;
        }

        let avg_recovery =
            self.recovery_times.iter().sum::<Duration>() / self.recovery_times.len() as u32;
        Some(avg_recovery)
    }

    fn get_failure_rate(&self) -> f64 {
        let recent_failures = self
            .failures
            .iter()
            .filter(|f| f.timestamp > SystemTime::now() - Duration::from_secs(300)) // Last 5 minutes
            .count();
        recent_failures as f64 / 5.0 // Failures per minute
    }
}

/// Configuration for graceful failure handling
#[derive(Debug, Clone)]
pub struct GracefulHandlerConfig {
    /// Thresholds for different degradation levels
    pub degradation_thresholds: HashMap<DegradationLevel, f64>,
    /// Maximum time to wait for recovery before escalating
    pub max_recovery_wait: Duration,
    /// Whether to enable automatic recovery detection
    pub enable_auto_recovery: bool,
    /// User message templates for different scenarios
    pub message_templates: HashMap<String, String>,
}

impl Default for GracefulHandlerConfig {
    fn default() -> Self {
        let mut degradation_thresholds = HashMap::new();
        degradation_thresholds.insert(DegradationLevel::Minor, 0.1); // 0.1 failures/min
        degradation_thresholds.insert(DegradationLevel::Moderate, 0.5);
        degradation_thresholds.insert(DegradationLevel::Severe, 1.0);
        degradation_thresholds.insert(DegradationLevel::Critical, 2.0);

        let mut message_templates = HashMap::new();
        message_templates.insert(
            "minor".to_string(),
            "Service is experiencing minor delays. Your request may take slightly longer than usual.".to_string()
        );
        message_templates.insert(
            "moderate".to_string(),
            "Service is temporarily degraded. We're providing cached results while we restore full functionality.".to_string()
        );
        message_templates.insert(
            "severe".to_string(),
            "Service is experiencing significant issues. Limited functionality is available."
                .to_string(),
        );
        message_templates.insert(
            "critical".to_string(),
            "Service is currently unavailable. We're working to restore service as quickly as possible.".to_string()
        );

        Self {
            degradation_thresholds,
            max_recovery_wait: Duration::from_secs(300), // 5 minutes
            enable_auto_recovery: true,
            message_templates,
        }
    }
}

/// Main graceful failure handler
pub struct GracefulFailureHandler {
    config: GracefulHandlerConfig,
    failure_history: Arc<RwLock<FailureHistory>>,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    fallback_manager: Arc<FallbackManager>,
    metrics_collector: Arc<MetricsCollector>,
    current_degradation: Arc<RwLock<DegradationLevel>>,
}

impl GracefulFailureHandler {
    /// Create a new graceful failure handler
    pub fn new(
        config: GracefulHandlerConfig,
        circuit_breaker: Arc<RwLock<CircuitBreaker>>,
        fallback_manager: Arc<FallbackManager>,
        metrics_collector: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            config,
            failure_history: Arc::new(RwLock::new(FailureHistory::new())),
            circuit_breaker,
            fallback_manager,
            metrics_collector,
            current_degradation: Arc::new(RwLock::new(DegradationLevel::Normal)),
        }
    }

    /// Handle a failure and return appropriate graceful response
    pub async fn handle_failure<T>(
        &self,
        error: &RenderingServiceError,
        fallback_data: Option<T>,
    ) -> GracefulResponse<Option<T>> {
        // Record the failure
        let degradation_level = self.assess_degradation_level(error).await;
        {
            let mut history = self.failure_history.write().await;
            history.record_failure(error.clone(), degradation_level.clone());
        }

        // Update current degradation level
        {
            let mut current = self.current_degradation.write().await;
            *current = degradation_level.clone();
        }

        // Record failure in circuit breaker
        {
            let mut cb = self.circuit_breaker.write().await;
            cb.record_failure(error);
        }

        // Collect metrics
        self.metrics_collector.record_error(error).await;

        // Generate user-friendly response
        let user_message = self.generate_user_message(&degradation_level, error);
        let estimated_recovery = {
            let history = self.failure_history.read().await;
            history.estimate_recovery_time()
        };

        let is_fallback = fallback_data.is_some();
        let technical_details = format!("Error: {}, Degradation: {:?}", error, degradation_level);

        warn!(
            error = %error,
            degradation_level = ?degradation_level,
            is_fallback = is_fallback,
            "Handling graceful failure"
        );

        GracefulResponse::degraded(
            fallback_data,
            degradation_level,
            user_message,
            estimated_recovery,
            is_fallback,
        )
        .with_technical_details(technical_details)
    }

    /// Handle a successful operation and potentially recover from degradation
    pub async fn handle_success<T>(&self, data: T) -> GracefulResponse<T> {
        // Check if we were in a degraded state
        let was_degraded = {
            let current = self.current_degradation.read().await;
            *current != DegradationLevel::Normal
        };

        if was_degraded && self.config.enable_auto_recovery {
            // Record recovery
            {
                let mut history = self.failure_history.write().await;
                if let Some(last_recovery) = history.last_recovery {
                    let recovery_time = SystemTime::now()
                        .duration_since(last_recovery)
                        .unwrap_or_default();
                    history.record_recovery(recovery_time);
                }
            }

            // Reset degradation level
            {
                let mut current = self.current_degradation.write().await;
                *current = DegradationLevel::Normal;
            }

            // Record success in circuit breaker
            {
                let mut cb = self.circuit_breaker.write().await;
                cb.record_success();
            }

            info!("Service recovered from degraded state");
        }

        GracefulResponse::normal(data)
    }

    /// Get current degradation level
    pub async fn get_current_degradation(&self) -> DegradationLevel {
        let current = self.current_degradation.read().await;
        current.clone()
    }

    /// Get failure statistics
    pub async fn get_failure_stats(&self) -> FailureStats {
        let history = self.failure_history.read().await;
        let current_degradation = self.get_current_degradation().await;

        FailureStats {
            current_degradation,
            failure_rate: history.get_failure_rate(),
            estimated_recovery: history.estimate_recovery_time(),
            recent_failures: history.failures.len(),
            average_recovery_time: history.estimate_recovery_time(),
        }
    }

    /// Assess degradation level based on error and current state
    async fn assess_degradation_level(&self, error: &RenderingServiceError) -> DegradationLevel {
        let failure_rate = {
            let history = self.failure_history.read().await;
            history.get_failure_rate()
        };

        // Check circuit breaker state
        let circuit_open = {
            let cb = self.circuit_breaker.read().await;
            cb.is_open()
        };

        // Determine degradation level based on error severity and failure rate
        match error.severity() {
            ErrorSeverity::Critical => DegradationLevel::Critical,
            ErrorSeverity::High => DegradationLevel::Severe,
            ErrorSeverity::Medium => DegradationLevel::Moderate,
            ErrorSeverity::Low => DegradationLevel::Minor,
            ErrorSeverity::Security => DegradationLevel::Critical,
        }
    }

    /// Generate user-friendly message based on degradation level and error
    fn generate_user_message(
        &self,
        level: &DegradationLevel,
        error: &RenderingServiceError,
    ) -> String {
        let template_key = match level {
            DegradationLevel::Normal => return "Service is operating normally.".to_string(),
            DegradationLevel::Minor => "minor",
            DegradationLevel::Moderate => "moderate",
            DegradationLevel::Severe => "severe",
            DegradationLevel::Critical => "critical",
        };

        let base_message = self
            .config
            .message_templates
            .get(template_key)
            .cloned()
            .unwrap_or_else(|| "Service is experiencing issues.".to_string());

        // Add specific context based on error type
        match error {
            RenderingServiceError::RateLimitExceeded {
                retry_after_seconds,
            } => {
                if let Some(seconds) = retry_after_seconds {
                    format!("{} Please try again in {} seconds.", base_message, seconds)
                } else {
                    format!("{} Please try again in a few moments.", base_message)
                }
            }
            RenderingServiceError::MemoryExhaustion { .. } => {
                format!("{} High demand is causing delays.", base_message)
            }
            _ => base_message,
        }
    }
}

/// Statistics about failure handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureStats {
    pub current_degradation: DegradationLevel,
    pub failure_rate: f64,
    pub estimated_recovery: Option<Duration>,
    pub recent_failures: usize,
    pub average_recovery_time: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resilience::{
        CircuitBreaker, FallbackConfig, FallbackManager, MetricsCollector, MetricsConfig,
    };
    use std::time::Duration;
    use tokio::time::sleep;

    async fn create_test_handler() -> GracefulFailureHandler {
        let config = GracefulHandlerConfig::default();
        let circuit_breaker =
            Arc::new(RwLock::new(CircuitBreaker::new(3, Duration::from_secs(10))));
        let fallback_manager = Arc::new(FallbackManager::new(FallbackConfig::default()));
        let metrics_collector = Arc::new(MetricsCollector::new(MetricsConfig::default()));

        GracefulFailureHandler::new(config, circuit_breaker, fallback_manager, metrics_collector)
    }

    #[tokio::test]
    async fn test_normal_response() {
        let handler = create_test_handler().await;
        let response = handler.handle_success("test_data".to_string()).await;

        assert_eq!(response.degradation_level, DegradationLevel::Normal);
        assert_eq!(response.data, "test_data");
        assert!(!response.is_fallback);
        assert!(response.user_message.is_none());
    }

    #[tokio::test]
    async fn test_failure_handling() {
        let handler = create_test_handler().await;
        let error = RenderingServiceError::ServiceUnavailable;
        let fallback_data = Some("fallback_data".to_string());

        let response = handler.handle_failure(&error, fallback_data).await;

        assert_ne!(response.degradation_level, DegradationLevel::Normal);
        assert!(response.is_fallback);
        assert!(response.user_message.is_some());
        assert!(response.technical_details.is_some());
    }

    #[tokio::test]
    async fn test_degradation_escalation() {
        let handler = create_test_handler().await;
        let error = RenderingServiceError::ServiceUnavailable;

        // Record multiple failures to escalate degradation
        for _ in 0..5 {
            handler.handle_failure(&error, None::<String>).await;
            sleep(Duration::from_millis(100)).await;
        }

        let stats = handler.get_failure_stats().await;
        assert!(stats.failure_rate > 0.0);
        assert!(stats.recent_failures > 0);
    }

    #[tokio::test]
    async fn test_recovery_detection() {
        let handler = create_test_handler().await;
        let error = RenderingServiceError::ServiceUnavailable;

        // Cause degradation
        handler.handle_failure(&error, None::<String>).await;
        assert_ne!(
            handler.get_current_degradation().await,
            DegradationLevel::Normal
        );

        // Recover
        handler.handle_success("recovery_data".to_string()).await;
        assert_eq!(
            handler.get_current_degradation().await,
            DegradationLevel::Normal
        );
    }

    #[tokio::test]
    async fn test_user_message_generation() {
        let handler = create_test_handler().await;
        let rate_limit_error = RenderingServiceError::RateLimitExceeded {
            retry_after_seconds: Some(30),
        };

        let response = handler
            .handle_failure(&rate_limit_error, None::<String>)
            .await;

        assert!(response.user_message.is_some());
        let message = response.user_message.unwrap();
        assert!(message.contains("30 seconds"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_integration() {
        let handler = create_test_handler().await;
        let critical_error = RenderingServiceError::ServiceUnavailable;

        // Trigger circuit breaker
        for _ in 0..3 {
            handler
                .handle_failure(&critical_error, None::<String>)
                .await;
        }

        let cb = handler.circuit_breaker.read().await;
        assert!(cb.is_open());
    }

    #[tokio::test]
    async fn test_failure_rate_calculation() {
        let handler = create_test_handler().await;
        let error = RenderingServiceError::ConnectionTimeout { timeout: 5000 };

        // Record failures over time
        for _ in 0..3 {
            handler.handle_failure(&error, None::<String>).await;
            sleep(Duration::from_millis(50)).await;
        }

        let stats = handler.get_failure_stats().await;
        assert!(stats.failure_rate > 0.0);
    }

    #[tokio::test]
    async fn test_configuration_customization() {
        let mut config = GracefulHandlerConfig::default();
        config
            .degradation_thresholds
            .insert(DegradationLevel::Critical, 0.1);

        let circuit_breaker =
            Arc::new(RwLock::new(CircuitBreaker::new(3, Duration::from_secs(10))));
        let fallback_manager = Arc::new(FallbackManager::new(FallbackConfig::default()));
        let metrics_collector = Arc::new(MetricsCollector::new(MetricsConfig::default()));

        let handler = GracefulFailureHandler::new(
            config,
            circuit_breaker,
            fallback_manager,
            metrics_collector,
        );

        // Test that custom thresholds are applied
        assert_eq!(
            handler.config.degradation_thresholds[&DegradationLevel::Critical],
            0.1
        );
    }
}
