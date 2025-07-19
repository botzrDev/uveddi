//! Chaos Engineering Framework for Uveddi (UV-82)
//! 
//! This module implements a comprehensive chaos engineering framework
//! designed for Rust microservices. It provides:
//! 
//! - Multi-tier fault injection (unit, service, system level)
//! - Safe failure scenarios with automatic rollback
//! - Comprehensive observability and measurement
//! - CI/CD pipeline integration

pub mod config;
pub mod experiments;
pub mod failpoints;

pub use config::{ChaosConfig, ExperimentConfig, BlastRadius, FailureMode};
pub use experiments::{ChaosExperiment, ExperimentResult, ExperimentRunner};
pub use failpoints::{FailpointManager, FailpointConfig};

use anyhow::Result;
use std::time::Duration;
use uuid::Uuid;

/// Main chaos engineering service
#[derive(Debug, Clone)]
pub struct ChaosEngine {
    config: ChaosConfig,
    experiment_runner: ExperimentRunner,
    active_experiments: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<Uuid, ChaosExperiment>>>,
}

impl ChaosEngine {
    /// Create a new chaos engineering engine
    pub fn new(config: ChaosConfig) -> Result<Self> {
        let experiment_runner = ExperimentRunner::new(config.clone())?;
        
        Ok(Self {
            config,
            experiment_runner,
            active_experiments: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        })
    }

    /// Start a chaos experiment
    pub async fn start_experiment(&self, experiment: ChaosExperiment) -> Result<Uuid> {
        let experiment_id = experiment.id;
        
        // Validate blast radius and safety constraints
        self.validate_experiment(&experiment)?;
        
        // Add to active experiments
        {
            let mut active = self.active_experiments.write().unwrap();
            active.insert(experiment_id, experiment.clone());
        }
        
        // Start the experiment
        let runner = self.experiment_runner.clone();
        
        tokio::spawn(async move {
            if let Err(e) = runner.start_experiment(experiment).await {
                tracing::error!(
                    experiment_id = %experiment_id,
                    error = %e,
                    "Chaos experiment failed"
                );
            }
        });
        
        tracing::info!(
            experiment_id = %experiment_id,
            "Chaos experiment started"
        );
        
        Ok(experiment_id)
    }

    /// Stop a running experiment
    pub async fn stop_experiment(&self, experiment_id: Uuid) -> Result<()> {
        // Remove from active experiments
        let experiment = {
            let mut active = self.active_experiments.write().unwrap();
            active.remove(&experiment_id)
        };

        if let Some(experiment) = experiment {
            self.experiment_runner.stop_experiment(&experiment).await?;
            
            tracing::info!(
                experiment_id = %experiment_id,
                "Chaos experiment stopped"
            );
        }

        Ok(())
    }

    /// Get status of all active experiments
    pub fn get_active_experiments(&self) -> Vec<ChaosExperiment> {
        let active = self.active_experiments.read().unwrap();
        active.values().cloned().collect()
    }

    /// Emergency stop all experiments
    pub async fn emergency_stop_all(&self) -> Result<()> {
        let experiments = {
            let mut active = self.active_experiments.write().unwrap();
            let experiments: Vec<_> = active.values().cloned().collect();
            active.clear();
            experiments
        };

        for experiment in experiments {
            if let Err(e) = self.experiment_runner.stop_experiment(&experiment).await {
                tracing::error!(
                    experiment_id = %experiment.id,
                    error = %e,
                    "Failed to stop experiment during emergency shutdown"
                );
            }
        }

        tracing::warn!("Emergency stop executed for all chaos experiments");
        Ok(())
    }

    /// Validate experiment safety constraints
    fn validate_experiment(&self, experiment: &ChaosExperiment) -> Result<()> {
        // Check if experiment is within allowed blast radius
        match &experiment.config.blast_radius {
            BlastRadius::UnitTest => {
                // Unit tests are always safe
                Ok(())
            }
            BlastRadius::SingleService => {
                // Ensure only targeting specific service
                if !self.config.allowed_services.contains(&experiment.target) {
                    return Err(anyhow::anyhow!("Service {} not in allowed list", experiment.target));
                }
                Ok(())
            }
            BlastRadius::MultiService => {
                // Require explicit approval for multi-service experiments
                if !self.config.multi_service_experiments_enabled {
                    return Err(anyhow::anyhow!("Multi-service experiments not enabled"));
                }
                Ok(())
            }
        }
    }

    /// Get experiment metrics and results
    pub fn get_experiment_metrics(&self, experiment_id: Uuid) -> Option<ExperimentResult> {
        self.experiment_runner.get_results(experiment_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chaos::config::FailureMode;

    #[tokio::test]
    async fn test_chaos_engine_creation() {
        let config = ChaosConfig::default();
        let engine = ChaosEngine::new(config).unwrap();
        
        assert_eq!(engine.get_active_experiments().len(), 0);
    }

    #[tokio::test]
    async fn test_experiment_lifecycle() {
        let config = ChaosConfig::default();
        let engine = ChaosEngine::new(config).unwrap();
        
        let experiment = ChaosExperiment {
            id: Uuid::new_v4(),
            name: "test_experiment".to_string(),
            target: "test_service".to_string(),
            config: ExperimentConfig {
                blast_radius: BlastRadius::UnitTest,
                failure_mode: FailureMode::NetworkLatency(Duration::from_millis(100)),
                duration: Duration::from_secs(30),
                safety_checks: vec![],
            },
            status: experiments::ExperimentStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        let experiment_id = engine.start_experiment(experiment).await.unwrap();
        assert_eq!(engine.get_active_experiments().len(), 1);
        
        engine.stop_experiment(experiment_id).await.unwrap();
        assert_eq!(engine.get_active_experiments().len(), 0);
    }
}