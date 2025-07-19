//! Failpoint injection system using the fail crate
//! 
//! This module provides controlled fault injection at the code level
//! using fail points. This is the most precise form of chaos engineering
//! as it can target specific functions and code paths.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Manager for failpoint injection
#[derive(Debug, Clone)]
pub struct FailpointManager {
    config: FailpointConfig,
    active_failpoints: Arc<RwLock<HashMap<String, ActiveFailpoint>>>,
    metrics: FailpointMetrics,
}

/// Configuration for failpoint behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailpointConfig {
    /// Whether failpoints are enabled globally
    pub enabled: bool,
    
    /// Default probability for failpoint activation
    pub default_probability: f64,
    
    /// Maximum duration for any failpoint
    pub max_duration: Duration,
    
    /// Failpoint-specific configurations
    pub failpoints: HashMap<String, FailpointSpec>,
}

impl Default for FailpointConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_probability: 0.0,
            max_duration: Duration::from_secs(300),
            failpoints: HashMap::new(),
        }
    }
}

/// Specification for a specific failpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailpointSpec {
    /// Human-readable description
    pub description: String,
    
    /// Probability of activation (0.0 to 1.0)
    pub probability: f64,
    
    /// Type of failure to inject
    pub failure_type: FailpointType,
    
    /// Maximum number of times this failpoint can be triggered
    pub max_triggers: Option<u32>,
    
    /// Duration for this failpoint to remain active
    pub duration: Option<Duration>,
}

/// Types of failures that can be injected via failpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailpointType {
    /// Return an error immediately
    Return(String),
    
    /// Panic with a message
    Panic(String),
    
    /// Introduce delay
    Delay(Duration),
    
    /// Return a specific value
    ReturnValue(String),
    
    /// Skip execution (no-op)
    Skip,
    
    /// Custom behavior
    Custom(String),
}

/// Active failpoint tracking
#[derive(Debug, Clone)]
struct ActiveFailpoint {
    spec: FailpointSpec,
    created_at: Instant,
    trigger_count: u32,
    last_triggered: Option<Instant>,
}

/// Metrics for failpoint activity
#[derive(Debug, Clone)]
pub struct FailpointMetrics {
    inner: Arc<RwLock<FailpointMetricsInner>>,
}

#[derive(Debug, Default)]
struct FailpointMetricsInner {
    total_triggers: u64,
    triggers_by_name: HashMap<String, u64>,
    failures_by_type: HashMap<String, u64>,
}

impl FailpointManager {
    /// Create a new failpoint manager
    pub fn new(config: FailpointConfig) -> Self {
        Self {
            config,
            active_failpoints: Arc::new(RwLock::new(HashMap::new())),
            metrics: FailpointMetrics::new(),
        }
    }

    /// Configure a failpoint
    pub fn configure_failpoint(&self, name: &str, spec: FailpointSpec) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let active_failpoint = ActiveFailpoint {
            spec: spec.clone(),
            created_at: Instant::now(),
            trigger_count: 0,
            last_triggered: None,
        };

        {
            let mut active = self.active_failpoints.write().unwrap();
            active.insert(name.to_string(), active_failpoint);
        }

        // Configure the actual failpoint using the fail crate
        #[cfg(feature = "chaos")]
        {
            let fail_config = self.build_fail_config(&spec);
            fail::cfg(name, &fail_config);
        }

        tracing::info!(
            failpoint = name,
            probability = spec.probability,
            failure_type = ?spec.failure_type,
            "Configured failpoint"
        );

        Ok(())
    }

    /// Remove a failpoint
    pub fn remove_failpoint(&self, name: &str) -> Result<()> {
        {
            let mut active = self.active_failpoints.write().unwrap();
            active.remove(name);
        }

        #[cfg(feature = "chaos")]
        {
            fail::cfg(name, "off");
        }

        tracing::info!(failpoint = name, "Removed failpoint");
        Ok(())
    }

    /// Remove all active failpoints
    pub fn clear_all_failpoints(&self) -> Result<()> {
        let failpoint_names: Vec<String> = {
            let active = self.active_failpoints.read().unwrap();
            active.keys().cloned().collect()
        };

        for name in failpoint_names {
            self.remove_failpoint(&name)?;
        }

        tracing::info!("Cleared all failpoints");
        Ok(())
    }

    /// Get status of all active failpoints
    pub fn get_active_failpoints(&self) -> HashMap<String, FailpointStatus> {
        let active = self.active_failpoints.read().unwrap();
        active
            .iter()
            .map(|(name, failpoint)| {
                let status = FailpointStatus {
                    name: name.clone(),
                    description: failpoint.spec.description.clone(),
                    probability: failpoint.spec.probability,
                    trigger_count: failpoint.trigger_count,
                    created_at: failpoint.created_at,
                    last_triggered: failpoint.last_triggered,
                    is_expired: self.is_failpoint_expired(failpoint),
                };
                (name.clone(), status)
            })
            .collect()
    }

    /// Record a failpoint trigger (called internally)
    pub fn record_trigger(&self, name: &str) {
        let mut trigger_recorded = false;

        {
            let mut active = self.active_failpoints.write().unwrap();
            if let Some(failpoint) = active.get_mut(name) {
                failpoint.trigger_count += 1;
                failpoint.last_triggered = Some(Instant::now());
                trigger_recorded = true;

                // Check if we should deactivate due to max triggers
                if let Some(max_triggers) = failpoint.spec.max_triggers {
                    if failpoint.trigger_count >= max_triggers {
                        tracing::info!(
                            failpoint = name,
                            trigger_count = failpoint.trigger_count,
                            "Failpoint reached max triggers, deactivating"
                        );
                    }
                }
            }
        }

        if trigger_recorded {
            self.metrics.record_trigger(name);
        }
    }

    /// Build fail crate configuration string
    #[cfg(feature = "chaos")]
    fn build_fail_config(&self, spec: &FailpointSpec) -> String {
        match &spec.failure_type {
            FailpointType::Return(msg) => {
                if spec.probability < 1.0 {
                    format!("{}%return({})", (spec.probability * 100.0) as u32, msg)
                } else {
                    format!("return({})", msg)
                }
            }
            FailpointType::Panic(msg) => {
                if spec.probability < 1.0 {
                    format!("{}%panic({})", (spec.probability * 100.0) as u32, msg)
                } else {
                    format!("panic({})", msg)
                }
            }
            FailpointType::Delay(duration) => {
                let ms = duration.as_millis();
                if spec.probability < 1.0 {
                    format!("{}%sleep({})", (spec.probability * 100.0) as u32, ms)
                } else {
                    format!("sleep({})", ms)
                }
            }
            FailpointType::Skip => {
                if spec.probability < 1.0 {
                    format!("{}%return", (spec.probability * 100.0) as u32)
                } else {
                    "return".to_string()
                }
            }
            FailpointType::ReturnValue(value) => {
                if spec.probability < 1.0 {
                    format!("{}%return({})", (spec.probability * 100.0) as u32, value)
                } else {
                    format!("return({})", value)
                }
            }
            FailpointType::Custom(config) => config.clone(),
        }
    }

    /// Check if a failpoint has expired
    fn is_failpoint_expired(&self, failpoint: &ActiveFailpoint) -> bool {
        if let Some(duration) = failpoint.spec.duration {
            failpoint.created_at.elapsed() > duration
        } else {
            false
        }
    }

    /// Clean up expired failpoints
    pub fn cleanup_expired_failpoints(&self) -> Result<()> {
        let expired_names: Vec<String> = {
            let active = self.active_failpoints.read().unwrap();
            active
                .iter()
                .filter(|(_, failpoint)| self.is_failpoint_expired(failpoint))
                .map(|(name, _)| name.clone())
                .collect()
        };

        for name in expired_names {
            self.remove_failpoint(&name)?;
            tracing::info!(failpoint = name, "Removed expired failpoint");
        }

        Ok(())
    }
}

/// Status information for a failpoint
#[derive(Debug, Clone, Serialize)]
pub struct FailpointStatus {
    pub name: String,
    pub description: String,
    pub probability: f64,
    pub trigger_count: u32,
    pub created_at: Instant,
    pub last_triggered: Option<Instant>,
    pub is_expired: bool,
}

impl FailpointMetrics {
    fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(FailpointMetricsInner::default())),
        }
    }

    fn record_trigger(&self, name: &str) {
        let mut inner = self.inner.write().unwrap();
        inner.total_triggers += 1;
        *inner.triggers_by_name.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn get_total_triggers(&self) -> u64 {
        let inner = self.inner.read().unwrap();
        inner.total_triggers
    }

    pub fn get_triggers_by_name(&self) -> HashMap<String, u64> {
        let inner = self.inner.read().unwrap();
        inner.triggers_by_name.clone()
    }
}

/// Macro for creating failpoints in application code
#[macro_export]
macro_rules! chaos_failpoint {
    ($name:expr) => {
        #[cfg(feature = "chaos")]
        {
            fail::fail_point!($name, |msg| {
                if let Some(manager) = $crate::chaos::get_failpoint_manager() {
                    manager.record_trigger($name);
                }
                msg
            });
        }
    };
    
    ($name:expr, $default:expr) => {
        #[cfg(feature = "chaos")]
        {
            fail::fail_point!($name, |msg| {
                if let Some(manager) = $crate::chaos::get_failpoint_manager() {
                    manager.record_trigger($name);
                }
                msg.unwrap_or($default)
            });
        }
    };
}

// Global failpoint manager instance (lazy static)
use std::sync::Once;
static INIT: Once = Once::new();
static mut FAILPOINT_MANAGER: Option<FailpointManager> = None;

/// Initialize the global failpoint manager
pub fn init_failpoint_manager(config: FailpointConfig) {
    unsafe {
        INIT.call_once(|| {
            FAILPOINT_MANAGER = Some(FailpointManager::new(config));
        });
    }
}

/// Get the global failpoint manager
pub fn get_failpoint_manager() -> Option<&'static FailpointManager> {
    unsafe { FAILPOINT_MANAGER.as_ref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failpoint_manager_creation() {
        let config = FailpointConfig::default();
        let manager = FailpointManager::new(config);
        
        assert_eq!(manager.get_active_failpoints().len(), 0);
    }

    #[test]
    fn test_failpoint_configuration() {
        let mut config = FailpointConfig::default();
        config.enabled = true;
        
        let manager = FailpointManager::new(config);
        
        let spec = FailpointSpec {
            description: "Test database error".to_string(),
            probability: 0.5,
            failure_type: FailpointType::Return("database_error".to_string()),
            max_triggers: Some(10),
            duration: Some(Duration::from_secs(60)),
        };
        
        manager.configure_failpoint("test_db_error", spec).unwrap();
        
        let active = manager.get_active_failpoints();
        assert_eq!(active.len(), 1);
        assert!(active.contains_key("test_db_error"));
        
        manager.remove_failpoint("test_db_error").unwrap();
        assert_eq!(manager.get_active_failpoints().len(), 0);
    }

    #[test]
    fn test_failpoint_metrics() {
        let manager = FailpointManager::new(FailpointConfig::default());
        
        assert_eq!(manager.metrics.get_total_triggers(), 0);
        
        manager.record_trigger("test_failpoint");
        manager.record_trigger("test_failpoint");
        manager.record_trigger("another_failpoint");
        
        assert_eq!(manager.metrics.get_total_triggers(), 3);
        
        let triggers_by_name = manager.metrics.get_triggers_by_name();
        assert_eq!(triggers_by_name.get("test_failpoint"), Some(&2));
        assert_eq!(triggers_by_name.get("another_failpoint"), Some(&1));
    }
}