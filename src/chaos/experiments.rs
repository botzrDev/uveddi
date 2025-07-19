//! Chaos Experiments Implementation for UV-82
//! 
//! This module provides the core experiment execution engine for chaos engineering,
//! including experiment lifecycle management, safety mechanisms, and result tracking.

use crate::chaos::config::{ExperimentConfig, BlastRadius, FailureMode};
use crate::chaos::failpoints::FailpointManager;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents a chaos experiment with its current state and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosExperiment {
    pub id: Uuid,
    pub name: String,
    pub config: ExperimentConfig,
    pub state: ExperimentState,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub results: Option<ExperimentResult>,
    pub safety_checks: Vec<SafetyCheck>,
}

/// Current state of a chaos experiment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentState {
    Created,
    Running,
    Paused,
    Completed,
    Failed,
    Aborted,
}

/// Safety check configuration and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheck {
    pub name: String,
    pub check_type: SafetyCheckType,
    pub threshold: f64,
    pub current_value: Option<f64>,
    pub status: SafetyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyCheckType {
    ErrorRate,
    ResponseTime,
    Availability,
    ResourceUtilization,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SafetyStatus {
    Passing,
    Warning,
    Critical,
    Unknown,
}

/// Results and metrics from a completed experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub success: bool,
    pub duration: Duration,
    pub metrics: HashMap<String, f64>,
    pub observations: Vec<String>,
    pub recovery_time: Option<Duration>,
    pub blast_radius_actual: BlastRadius,
    pub error_details: Option<String>,
}

/// Manages the execution of chaos experiments
#[derive(Debug)]
pub struct ExperimentRunner {
    failpoint_manager: Arc<FailpointManager>,
    active_experiments: Arc<RwLock<HashMap<Uuid, ChaosExperiment>>>,
    safety_monitor: Arc<SafetyMonitor>,
}

/// Monitors safety conditions during experiments
#[derive(Debug)]
pub struct SafetyMonitor {
    checks: Vec<SafetyCheck>,
    abort_threshold: f64,
}

impl ChaosExperiment {
    /// Create a new chaos experiment
    pub fn new(name: String, config: ExperimentConfig) -> Self {
        let safety_checks = Self::create_default_safety_checks(&config);
        
        Self {
            id: Uuid::new_v4(),
            name,
            config,
            state: ExperimentState::Created,
            start_time: None,
            end_time: None,
            results: None,
            safety_checks,
        }
    }

    /// Create default safety checks based on experiment configuration
    fn create_default_safety_checks(config: &ExperimentConfig) -> Vec<SafetyCheck> {
        vec![
            SafetyCheck {
                name: "Error Rate".to_string(),
                check_type: SafetyCheckType::ErrorRate,
                threshold: 0.05, // 5% error rate threshold
                current_value: None,
                status: SafetyStatus::Unknown,
            },
            SafetyCheck {
                name: "Response Time P95".to_string(),
                check_type: SafetyCheckType::ResponseTime,
                threshold: 2000.0, // 2 second threshold
                current_value: None,
                status: SafetyStatus::Unknown,
            },
            SafetyCheck {
                name: "Service Availability".to_string(),
                check_type: SafetyCheckType::Availability,
                threshold: 0.95, // 95% availability threshold
                current_value: None,
                status: SafetyStatus::Unknown,
            },
        ]
    }

    /// Update the state of the experiment
    pub fn update_state(&mut self, new_state: ExperimentState) {
        self.state = new_state;
        
        match self.state {
            ExperimentState::Running => {
                self.start_time = Some(SystemTime::now());
            }
            ExperimentState::Completed | ExperimentState::Failed | ExperimentState::Aborted => {
                self.end_time = Some(SystemTime::now());
            }
            _ => {}
        }
    }

    /// Check if the experiment is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.state, ExperimentState::Running | ExperimentState::Paused)
    }

    /// Get the duration of the experiment if it has started
    pub fn duration(&self) -> Option<Duration> {
        self.start_time.and_then(|start| {
            let end = self.end_time.unwrap_or_else(SystemTime::now);
            end.duration_since(start).ok()
        })
    }
}

impl ExperimentRunner {
    /// Create a new experiment runner
    pub fn new(failpoint_manager: Arc<FailpointManager>) -> Self {
        Self {
            failpoint_manager,
            active_experiments: Arc::new(RwLock::new(HashMap::new())),
            safety_monitor: Arc::new(SafetyMonitor::new()),
        }
    }

    /// Start a chaos experiment
    pub async fn start_experiment(&self, mut experiment: ChaosExperiment) -> Result<Uuid> {
        // Pre-flight safety checks
        self.safety_monitor.validate_experiment(&experiment).await?;

        // Activate the failure mode
        self.activate_failure_mode(&experiment.config.failure_mode).await?;

        // Update experiment state
        experiment.update_state(ExperimentState::Running);
        let experiment_id = experiment.id;

        // Store the active experiment
        {
            let mut active = self.active_experiments.write().await;
            active.insert(experiment_id, experiment);
        }

        // Start monitoring
        self.start_monitoring(experiment_id).await?;

        Ok(experiment_id)
    }

    /// Stop a running experiment
    pub async fn stop_experiment(&self, experiment_id: Uuid) -> Result<ExperimentResult> {
        let mut experiment = {
            let mut active = self.active_experiments.write().await;
            active.remove(&experiment_id)
                .ok_or_else(|| anyhow!("Experiment {} not found", experiment_id))?
        };

        // Deactivate failure mode
        self.deactivate_failure_mode(&experiment.config.failure_mode).await?;

        // Collect results
        let result = self.collect_experiment_results(&experiment).await?;
        experiment.results = Some(result.clone());
        experiment.update_state(ExperimentState::Completed);

        Ok(result)
    }

    /// Abort an experiment due to safety violations
    pub async fn abort_experiment(&self, experiment_id: Uuid, reason: String) -> Result<()> {
        let mut experiment = {
            let mut active = self.active_experiments.write().await;
            active.remove(&experiment_id)
                .ok_or_else(|| anyhow!("Experiment {} not found", experiment_id))?
        };

        // Emergency deactivation
        self.deactivate_failure_mode(&experiment.config.failure_mode).await?;

        // Mark as aborted with reason
        let result = ExperimentResult {
            success: false,
            duration: experiment.duration().unwrap_or_default(),
            metrics: HashMap::new(),
            observations: vec![format!("Aborted: {}", reason)],
            recovery_time: None,
            blast_radius_actual: experiment.config.blast_radius.clone(),
            error_details: Some(reason),
        };

        experiment.results = Some(result);
        experiment.update_state(ExperimentState::Aborted);

        Ok(())
    }

    /// Get all active experiments
    pub async fn get_active_experiments(&self) -> Vec<ChaosExperiment> {
        let active = self.active_experiments.read().await;
        active.values().cloned().collect()
    }

    /// Activate a specific failure mode
    async fn activate_failure_mode(&self, failure_mode: &FailureMode) -> Result<()> {
        match failure_mode {
            FailureMode::NetworkLatency(delay) => {
                self.failpoint_manager.activate_network_latency(*delay).await?;
            }
            FailureMode::DatabaseUnavailable => {
                self.failpoint_manager.activate_database_failure().await?;
            }
            FailureMode::MemoryPressure(bytes) => {
                self.failpoint_manager.activate_memory_pressure(*bytes).await?;
            }
            FailureMode::CpuExhaustion => {
                self.failpoint_manager.activate_cpu_exhaustion().await?;
            }
            FailureMode::DiskIoFailure => {
                self.failpoint_manager.activate_disk_failure().await?;
            }
            FailureMode::ServiceTimeout => {
                self.failpoint_manager.activate_service_timeout().await?;
            }
        }
        Ok(())
    }

    /// Deactivate a specific failure mode
    async fn deactivate_failure_mode(&self, failure_mode: &FailureMode) -> Result<()> {
        match failure_mode {
            FailureMode::NetworkLatency(_) => {
                self.failpoint_manager.deactivate_network_latency().await?;
            }
            FailureMode::DatabaseUnavailable => {
                self.failpoint_manager.deactivate_database_failure().await?;
            }
            FailureMode::MemoryPressure(_) => {
                self.failpoint_manager.deactivate_memory_pressure().await?;
            }
            FailureMode::CpuExhaustion => {
                self.failpoint_manager.deactivate_cpu_exhaustion().await?;
            }
            FailureMode::DiskIoFailure => {
                self.failpoint_manager.deactivate_disk_failure().await?;
            }
            FailureMode::ServiceTimeout => {
                self.failpoint_manager.deactivate_service_timeout().await?;
            }
        }
        Ok(())
    }

    /// Start monitoring an experiment for safety violations
    async fn start_monitoring(&self, experiment_id: Uuid) -> Result<()> {
        let safety_monitor = Arc::clone(&self.safety_monitor);
        let experiments = Arc::clone(&self.active_experiments);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Check if experiment is still active
                let experiment_exists = {
                    let active = experiments.read().await;
                    active.contains_key(&experiment_id)
                };
                
                if !experiment_exists {
                    break;
                }
                
                // Perform safety checks
                if let Err(e) = safety_monitor.check_safety_conditions(experiment_id).await {
                    eprintln!("Safety check failed for experiment {}: {}", experiment_id, e);
                    // In a real implementation, this would trigger an abort
                }
            }
        });
        
        Ok(())
    }

    /// Collect results and metrics from a completed experiment
    async fn collect_experiment_results(&self, experiment: &ChaosExperiment) -> Result<ExperimentResult> {
        let duration = experiment.duration().unwrap_or_default();
        
        // Collect metrics (in a real implementation, this would query monitoring systems)
        let mut metrics = HashMap::new();
        metrics.insert("duration_ms".to_string(), duration.as_millis() as f64);
        metrics.insert("blast_radius_score".to_string(), 1.0); // Placeholder
        
        // Collect observations
        let observations = vec![
            format!("Experiment {} completed", experiment.name),
            format!("Duration: {:?}", duration),
            format!("Failure mode: {:?}", experiment.config.failure_mode),
        ];

        Ok(ExperimentResult {
            success: true,
            duration,
            metrics,
            observations,
            recovery_time: Some(Duration::from_secs(1)), // Placeholder
            blast_radius_actual: experiment.config.blast_radius.clone(),
            error_details: None,
        })
    }
}

impl SafetyMonitor {
    /// Create a new safety monitor
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            abort_threshold: 0.8, // Abort if 80% of checks fail
        }
    }

    /// Validate an experiment before starting
    pub async fn validate_experiment(&self, experiment: &ChaosExperiment) -> Result<()> {
        // Check blast radius constraints
        match &experiment.config.blast_radius {
            BlastRadius::Single => {
                // Single service impact is always safe
            }
            BlastRadius::Service => {
                // Service-level impact requires additional validation
                if experiment.config.duration > Duration::from_minutes(10) {
                    return Err(anyhow!("Service-level experiments limited to 10 minutes"));
                }
            }
            BlastRadius::System => {
                // System-level impact requires strict controls
                if experiment.config.duration > Duration::from_minutes(5) {
                    return Err(anyhow!("System-level experiments limited to 5 minutes"));
                }
            }
        }

        Ok(())
    }

    /// Check safety conditions during experiment execution
    pub async fn check_safety_conditions(&self, _experiment_id: Uuid) -> Result<()> {
        // In a real implementation, this would:
        // 1. Query monitoring systems for current metrics
        // 2. Compare against safety thresholds
        // 3. Trigger abort if thresholds are exceeded
        
        // Placeholder implementation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chaos::config::ChaosConfig;

    #[test]
    fn test_experiment_creation() {
        let config = ExperimentConfig {
            name: "test-experiment".to_string(),
            failure_mode: FailureMode::NetworkLatency(Duration::from_millis(100)),
            blast_radius: BlastRadius::Single,
            duration: Duration::from_secs(60),
            safety_checks: Vec::new(),
        };

        let experiment = ChaosExperiment::new("Test Experiment".to_string(), config);
        
        assert_eq!(experiment.state, ExperimentState::Created);
        assert_eq!(experiment.name, "Test Experiment");
        assert!(!experiment.is_active());
        assert_eq!(experiment.safety_checks.len(), 3); // Default safety checks
    }

    #[test]
    fn test_experiment_state_transitions() {
        let config = ExperimentConfig {
            name: "test-experiment".to_string(),
            failure_mode: FailureMode::NetworkLatency(Duration::from_millis(100)),
            blast_radius: BlastRadius::Single,
            duration: Duration::from_secs(60),
            safety_checks: Vec::new(),
        };

        let mut experiment = ChaosExperiment::new("Test Experiment".to_string(), config);
        
        // Test state transitions
        experiment.update_state(ExperimentState::Running);
        assert_eq!(experiment.state, ExperimentState::Running);
        assert!(experiment.is_active());
        assert!(experiment.start_time.is_some());

        experiment.update_state(ExperimentState::Completed);
        assert_eq!(experiment.state, ExperimentState::Completed);
        assert!(!experiment.is_active());
        assert!(experiment.end_time.is_some());
    }

    #[tokio::test]
    async fn test_safety_monitor_validation() {
        let monitor = SafetyMonitor::new();
        
        // Test valid experiment
        let config = ExperimentConfig {
            name: "test-experiment".to_string(),
            failure_mode: FailureMode::NetworkLatency(Duration::from_millis(100)),
            blast_radius: BlastRadius::Single,
            duration: Duration::from_secs(60),
            safety_checks: Vec::new(),
        };
        let experiment = ChaosExperiment::new("Test".to_string(), config);
        
        assert!(monitor.validate_experiment(&experiment).await.is_ok());
        
        // Test invalid experiment (too long duration for system-level)
        let config = ExperimentConfig {
            name: "test-experiment".to_string(),
            failure_mode: FailureMode::NetworkLatency(Duration::from_millis(100)),
            blast_radius: BlastRadius::System,
            duration: Duration::from_minutes(10), // Too long
            safety_checks: Vec::new(),
        };
        let experiment = ChaosExperiment::new("Test".to_string(), config);
        
        assert!(monitor.validate_experiment(&experiment).await.is_err());
    }
}