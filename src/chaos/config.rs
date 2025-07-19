//! Configuration for chaos engineering experiments

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Global configuration for the chaos engineering system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    /// Whether chaos engineering is enabled
    pub enabled: bool,
    
    /// Environment (test, staging, production)
    pub environment: Environment,
    
    /// Maximum duration for any single experiment
    pub max_experiment_duration: Duration,
    
    /// Services that are allowed to be targeted
    pub allowed_services: Vec<String>,
    
    /// Whether multi-service experiments are enabled
    pub multi_service_experiments_enabled: bool,
    
    /// Safety settings
    pub safety: SafetyConfig,
    
    /// Observability configuration
    pub observability: ObservabilityConfig,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            environment: Environment::Test,
            max_experiment_duration: Duration::from_secs(300), // 5 minutes max
            allowed_services: vec![
                "analysis-service".to_string(),
                "rendering-service".to_string(),
            ],
            multi_service_experiments_enabled: false,
            safety: SafetyConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

/// Environment where chaos engineering is running
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Test,
    Staging,
    Production,
}

/// Safety configuration to prevent dangerous experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Require manual approval for certain experiment types
    pub require_approval: bool,
    
    /// Maximum percentage of requests that can be affected
    pub max_traffic_percentage: f64,
    
    /// Automatic rollback triggers
    pub auto_rollback: AutoRollbackConfig,
    
    /// Circuit breaker settings
    pub circuit_breaker: CircuitBreakerConfig,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            require_approval: true,
            max_traffic_percentage: 10.0, // 10% max by default
            auto_rollback: AutoRollbackConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

/// Automatic rollback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRollbackConfig {
    /// Enable automatic rollback
    pub enabled: bool,
    
    /// Error rate threshold (percentage) that triggers rollback
    pub error_rate_threshold: f64,
    
    /// Latency threshold (milliseconds) that triggers rollback
    pub latency_threshold_ms: u64,
    
    /// Time window for measuring metrics
    pub measurement_window: Duration,
}

impl Default for AutoRollbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            error_rate_threshold: 5.0, // 5% error rate triggers rollback
            latency_threshold_ms: 1000, // 1 second latency triggers rollback
            measurement_window: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker configuration for chaos experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    pub enabled: bool,
    
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    
    /// Timeout before attempting to close circuit
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Observability configuration for chaos experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,
    
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    
    /// Metrics export interval
    pub metrics_interval: Duration,
    
    /// Trace sampling rate (0.0 to 1.0)
    pub trace_sampling_rate: f64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: true,
            metrics_interval: Duration::from_secs(10),
            trace_sampling_rate: 1.0, // Sample all traces during chaos experiments
        }
    }
}

/// Configuration for a specific chaos experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Blast radius of the experiment
    pub blast_radius: BlastRadius,
    
    /// Type of failure to inject
    pub failure_mode: FailureMode,
    
    /// Duration of the experiment
    pub duration: Duration,
    
    /// Safety checks to perform during the experiment
    pub safety_checks: Vec<SafetyCheck>,
}

/// Blast radius defines the scope of impact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlastRadius {
    /// Impact limited to unit tests
    UnitTest,
    
    /// Impact limited to a single service instance
    SingleService,
    
    /// Impact across multiple services
    MultiService,
}

/// Types of failures that can be injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureMode {
    /// Inject network latency
    NetworkLatency(Duration),
    
    /// Make service unavailable (return 503)
    ServiceUnavailable,
    
    /// Simulate database connection failure
    DatabaseUnavailable,
    
    /// Inject memory pressure
    MemoryPressure(usize),
    
    /// Inject CPU exhaustion
    CpuExhaustion,
    
    /// Simulate disk I/O failure
    DiskIoFailure,
    
    /// Timeout scenarios
    ServiceTimeout(Duration),
    
    /// Custom failure mode
    Custom(String),
}

/// Safety checks that can be performed during experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheck {
    /// Name of the safety check
    pub name: String,
    
    /// Type of check to perform
    pub check_type: SafetyCheckType,
    
    /// Threshold for the check
    pub threshold: f64,
    
    /// Action to take if check fails
    pub action: SafetyAction,
}

/// Types of safety checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyCheckType {
    /// Monitor error rate
    ErrorRate,
    
    /// Monitor response latency
    ResponseLatency,
    
    /// Monitor service availability
    ServiceAvailability,
    
    /// Monitor resource utilization
    ResourceUtilization,
    
    /// Custom check
    Custom(String),
}

/// Actions to take when safety checks fail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyAction {
    /// Log a warning
    LogWarning,
    
    /// Stop the experiment
    StopExperiment,
    
    /// Trigger emergency rollback
    EmergencyRollback,
    
    /// Send alert
    SendAlert,
}

impl ChaosConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();
        
        if let Ok(enabled) = std::env::var("CHAOS_ENABLED") {
            config.enabled = enabled.parse().unwrap_or(false);
        }
        
        if let Ok(env) = std::env::var("CHAOS_ENVIRONMENT") {
            config.environment = match env.as_str() {
                "test" => Environment::Test,
                "staging" => Environment::Staging,
                "production" => Environment::Production,
                _ => Environment::Test,
            };
        }
        
        if let Ok(duration) = std::env::var("CHAOS_MAX_DURATION_SECONDS") {
            if let Ok(seconds) = duration.parse::<u64>() {
                config.max_experiment_duration = Duration::from_secs(seconds);
            }
        }
        
        if let Ok(percentage) = std::env::var("CHAOS_MAX_TRAFFIC_PERCENTAGE") {
            if let Ok(pct) = percentage.parse::<f64>() {
                config.safety.max_traffic_percentage = pct;
            }
        }
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.safety.max_traffic_percentage < 0.0 || self.safety.max_traffic_percentage > 100.0 {
            return Err("max_traffic_percentage must be between 0 and 100".to_string());
        }
        
        if self.safety.auto_rollback.error_rate_threshold < 0.0 || self.safety.auto_rollback.error_rate_threshold > 100.0 {
            return Err("error_rate_threshold must be between 0 and 100".to_string());
        }
        
        if self.observability.trace_sampling_rate < 0.0 || self.observability.trace_sampling_rate > 1.0 {
            return Err("trace_sampling_rate must be between 0.0 and 1.0".to_string());
        }
        
        Ok(())
    }
    
    /// Check if experiments are allowed in the current environment
    pub fn is_experiment_allowed(&self, blast_radius: &BlastRadius) -> bool {
        if !self.enabled {
            return false;
        }
        
        match (&self.environment, blast_radius) {
            (Environment::Test, _) => true,
            (Environment::Staging, BlastRadius::UnitTest | BlastRadius::SingleService) => true,
            (Environment::Production, BlastRadius::UnitTest) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ChaosConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.environment, Environment::Test);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = ChaosConfig::default();
        
        // Test invalid traffic percentage
        config.safety.max_traffic_percentage = 150.0;
        assert!(config.validate().is_err());
        
        // Test invalid error rate threshold
        config.safety.max_traffic_percentage = 10.0;
        config.safety.auto_rollback.error_rate_threshold = -5.0;
        assert!(config.validate().is_err());
        
        // Test valid config
        config.safety.auto_rollback.error_rate_threshold = 5.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_experiment_permissions() {
        let mut config = ChaosConfig::default();
        config.enabled = true;
        
        // Test environment
        config.environment = Environment::Test;
        assert!(config.is_experiment_allowed(&BlastRadius::UnitTest));
        assert!(config.is_experiment_allowed(&BlastRadius::SingleService));
        assert!(config.is_experiment_allowed(&BlastRadius::MultiService));
        
        // Staging environment
        config.environment = Environment::Staging;
        assert!(config.is_experiment_allowed(&BlastRadius::UnitTest));
        assert!(config.is_experiment_allowed(&BlastRadius::SingleService));
        assert!(!config.is_experiment_allowed(&BlastRadius::MultiService));
        
        // Production environment
        config.environment = Environment::Production;
        assert!(config.is_experiment_allowed(&BlastRadius::UnitTest));
        assert!(!config.is_experiment_allowed(&BlastRadius::SingleService));
        assert!(!config.is_experiment_allowed(&BlastRadius::MultiService));
    }
}