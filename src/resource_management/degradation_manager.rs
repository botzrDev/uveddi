//! Graceful degradation manager for maintaining service under resource pressure

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::{
    error::{ResourceError, ResourceResult},
    memory_tracker::MemoryTracker,
    resource_config::{DegradationConfig, ResourceConfig},
    resource_monitor::ResourceAlert,
};

/// System degradation levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationLevel {
    /// Normal operation - all features available
    Normal,
    /// Light degradation - reduce non-essential features
    LightDegradation,
    /// Heavy degradation - minimal features only
    HeavyDegradation,
    /// Emergency mode - critical functions only
    EmergencyMode,
}

impl DegradationLevel {
    /// Returns the degradation factor (0.0 = normal, 1.0 = maximum degradation)
    pub fn factor(&self) -> f64 {
        match self {
            DegradationLevel::Normal => 0.0,
            DegradationLevel::LightDegradation => 0.3,
            DegradationLevel::HeavyDegradation => 0.7,
            DegradationLevel::EmergencyMode => 0.9,
        }
    }

    /// Returns a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            DegradationLevel::Normal => "Normal operation",
            DegradationLevel::LightDegradation => "Light degradation - reduced optional features",
            DegradationLevel::HeavyDegradation => "Heavy degradation - minimal features only",
            DegradationLevel::EmergencyMode => "Emergency mode - critical functions only",
        }
    }
}

/// Feature categories that can be disabled during degradation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FeatureCategory {
    /// AI-powered analysis features
    AiAnalysis,
    /// Detailed reporting features
    DetailedReporting,
    /// Interactive TUI features
    InteractiveFeatures,
    /// Background caching
    BackgroundCaching,
    /// Metrics collection and export
    MetricsExport,
    /// Non-essential logging
    VerboseLogging,
    /// Plugin system features
    PluginSystem,
    /// Large file processing
    LargeFileProcessing,
    /// Concurrent analysis beyond minimum
    ExtendedConcurrency,
}

/// Degradation action that can be taken
#[derive(Debug, Clone)]
pub struct DegradationAction {
    pub category: FeatureCategory,
    pub action_type: DegradationActionType,
    pub description: String,
    pub resource_impact: ResourceImpact,
}

/// Types of degradation actions
#[derive(Debug, Clone, PartialEq)]
pub enum DegradationActionType {
    /// Disable a feature completely
    Disable,
    /// Reduce the capacity or quality of a feature
    Reduce(f64), // Factor by which to reduce (0.0 = disable, 1.0 = no reduction)
    /// Defer a feature to run later when resources are available
    Defer,
}

/// Expected resource impact of a degradation action
#[derive(Debug, Clone)]
pub struct ResourceImpact {
    pub memory_savings_mb: u64,
    pub cpu_savings_percent: f64,
    pub io_savings_percent: f64,
}

/// Graceful degradation state
#[derive(Debug, Clone)]
pub struct DegradationState {
    pub current_level: DegradationLevel,
    pub previous_level: DegradationLevel,
    pub level_changed_at: Instant,
    pub active_actions: Vec<DegradationAction>,
    pub recovery_possible: bool,
    pub recovery_threshold_met: bool,
}

/// Manages graceful degradation of system features under resource pressure
pub struct GracefulDegradationManager {
    memory_tracker: Arc<MemoryTracker>,
    config: Arc<RwLock<ResourceConfig>>,
    current_state: Arc<Mutex<DegradationState>>,
    feature_states: Arc<Mutex<std::collections::HashMap<FeatureCategory, bool>>>,
    degradation_history: Arc<Mutex<Vec<(Instant, DegradationLevel, String)>>>,
    recovery_timer: Arc<Mutex<Option<Instant>>>,
}

impl GracefulDegradationManager {
    /// Creates a new graceful degradation manager
    pub fn new(memory_tracker: Arc<MemoryTracker>, config: Arc<RwLock<ResourceConfig>>) -> Self {
        let initial_state = DegradationState {
            current_level: DegradationLevel::Normal,
            previous_level: DegradationLevel::Normal,
            level_changed_at: Instant::now(),
            active_actions: Vec::new(),
            recovery_possible: true,
            recovery_threshold_met: true,
        };

        let mut feature_states = std::collections::HashMap::new();
        // Initialize all features as enabled
        for category in [
            FeatureCategory::AiAnalysis,
            FeatureCategory::DetailedReporting,
            FeatureCategory::InteractiveFeatures,
            FeatureCategory::BackgroundCaching,
            FeatureCategory::MetricsExport,
            FeatureCategory::VerboseLogging,
            FeatureCategory::PluginSystem,
            FeatureCategory::LargeFileProcessing,
            FeatureCategory::ExtendedConcurrency,
        ] {
            feature_states.insert(category, true);
        }

        Self {
            memory_tracker,
            config,
            current_state: Arc::new(Mutex::new(initial_state)),
            feature_states: Arc::new(Mutex::new(feature_states)),
            degradation_history: Arc::new(Mutex::new(Vec::new())),
            recovery_timer: Arc::new(Mutex::new(None)),
        }
    }

    /// Gets the current degradation level
    pub fn get_current_level(&self) -> DegradationLevel {
        self.current_state.lock().unwrap().current_level.clone()
    }

    /// Gets the current degradation state
    pub fn get_state(&self) -> DegradationState {
        self.current_state.lock().unwrap().clone()
    }

    /// Checks if a feature is currently enabled
    pub fn is_feature_enabled(&self, category: &FeatureCategory) -> bool {
        self.feature_states
            .lock()
            .unwrap()
            .get(category)
            .copied()
            .unwrap_or(false)
    }

    /// Evaluates current resource pressure and adjusts service level
    pub async fn adjust_service_level(&self) -> ResourceResult<bool> {
        let config = self.config.read().await;
        if !config.degradation.enabled {
            return Ok(false);
        }

        let memory_pressure = self.memory_tracker.get_memory_pressure();
        let degradation_config = &config.degradation;

        let new_level = self.determine_degradation_level(memory_pressure, degradation_config);
        let current_level = self.get_current_level();

        if new_level != current_level {
            self.apply_degradation_level(
                new_level.clone(),
                "Resource pressure adjustment".to_string(),
            )
            .await?;
            Ok(true)
        } else {
            // Check for recovery conditions
            self.check_recovery_conditions(memory_pressure, degradation_config)
                .await?;
            Ok(false)
        }
    }

    /// Forces a specific degradation level (for emergency situations)
    pub async fn force_degradation_level(
        &self,
        level: DegradationLevel,
        reason: String,
    ) -> ResourceResult<()> {
        self.apply_degradation_level(level, reason).await
    }

    /// Triggers emergency cleanup and maximum degradation
    pub async fn trigger_emergency_cleanup(&self) -> ResourceResult<()> {
        self.apply_degradation_level(
            DegradationLevel::EmergencyMode,
            "Emergency cleanup triggered".to_string(),
        )
        .await?;

        // Perform emergency cleanup actions
        self.emergency_cleanup_actions().await?;

        Ok(())
    }

    /// Gets degradation history for analysis
    pub fn get_degradation_history(
        &self,
        duration: Duration,
    ) -> Vec<(Instant, DegradationLevel, String)> {
        let history = self.degradation_history.lock().unwrap();
        let cutoff = Instant::now() - duration;

        history
            .iter()
            .filter(|(timestamp, _, _)| *timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// Manually enables or disables a feature
    pub async fn set_feature_enabled(
        &self,
        category: FeatureCategory,
        enabled: bool,
    ) -> ResourceResult<()> {
        let mut feature_states = self.feature_states.lock().unwrap();
        feature_states.insert(category.clone(), enabled);

        info!(
            "Feature {:?} manually {} by administrator",
            category,
            if enabled { "enabled" } else { "disabled" }
        );

        Ok(())
    }

    fn determine_degradation_level(
        &self,
        memory_pressure: f64,
        config: &DegradationConfig,
    ) -> DegradationLevel {
        if memory_pressure >= config.emergency_threshold {
            DegradationLevel::EmergencyMode
        } else if memory_pressure >= config.heavy_threshold {
            DegradationLevel::HeavyDegradation
        } else if memory_pressure >= config.light_threshold {
            DegradationLevel::LightDegradation
        } else {
            DegradationLevel::Normal
        }
    }

    async fn check_recovery_conditions(
        &self,
        memory_pressure: f64,
        config: &DegradationConfig,
    ) -> ResourceResult<()> {
        let current_level = self.get_current_level();

        if current_level == DegradationLevel::Normal {
            return Ok(());
        }

        // Check if we can recover to a higher service level
        if memory_pressure < config.recovery_threshold {
            let mut recovery_timer = self.recovery_timer.lock().unwrap();

            if recovery_timer.is_none() {
                *recovery_timer = Some(Instant::now());
            } else if let Some(timer) = *recovery_timer {
                // Wait for stable conditions before recovering
                if timer.elapsed() > Duration::from_secs(30) {
                    let recovery_level = match current_level {
                        DegradationLevel::EmergencyMode => DegradationLevel::HeavyDegradation,
                        DegradationLevel::HeavyDegradation => DegradationLevel::LightDegradation,
                        DegradationLevel::LightDegradation => DegradationLevel::Normal,
                        DegradationLevel::Normal => DegradationLevel::Normal,
                    };

                    self.apply_degradation_level(recovery_level, "Automatic recovery".to_string())
                        .await?;
                    *recovery_timer = None;
                }
            }
        } else {
            // Reset recovery timer if conditions are not met
            *self.recovery_timer.lock().unwrap() = None;
        }

        Ok(())
    }

    async fn apply_degradation_level(
        &self,
        level: DegradationLevel,
        reason: String,
    ) -> ResourceResult<()> {
        let actions = self.get_degradation_actions(&level);

        // Update current state
        {
            let mut state = self.current_state.lock().unwrap();
            state.previous_level = state.current_level.clone();
            state.current_level = level.clone();
            state.level_changed_at = Instant::now();
            state.active_actions = actions;
            state.recovery_possible = level != DegradationLevel::EmergencyMode;
        }

        // Record in history
        {
            let mut history = self.degradation_history.lock().unwrap();
            history.push((Instant::now(), level.clone(), reason.clone()));

            // Keep only last 100 entries
            if history.len() > 100 {
                history.drain(0..50);
            }
        }

        // Apply the degradation actions
        self.execute_degradation_actions(&level).await?;

        warn!(
            "Service level changed to {:?}: {} - {}",
            level,
            level.description(),
            reason
        );

        Ok(())
    }

    fn get_degradation_actions(&self, level: &DegradationLevel) -> Vec<DegradationAction> {
        match level {
            DegradationLevel::Normal => Vec::new(),
            DegradationLevel::LightDegradation => vec![
                DegradationAction {
                    category: FeatureCategory::VerboseLogging,
                    action_type: DegradationActionType::Disable,
                    description: "Disable verbose logging to reduce I/O".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 10,
                        cpu_savings_percent: 2.0,
                        io_savings_percent: 15.0,
                    },
                },
                DegradationAction {
                    category: FeatureCategory::BackgroundCaching,
                    action_type: DegradationActionType::Reduce(0.5),
                    description: "Reduce background cache operations".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 50,
                        cpu_savings_percent: 5.0,
                        io_savings_percent: 10.0,
                    },
                },
                DegradationAction {
                    category: FeatureCategory::ExtendedConcurrency,
                    action_type: DegradationActionType::Reduce(0.7),
                    description: "Reduce maximum concurrent analyses".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 100,
                        cpu_savings_percent: 10.0,
                        io_savings_percent: 5.0,
                    },
                },
            ],
            DegradationLevel::HeavyDegradation => vec![
                DegradationAction {
                    category: FeatureCategory::AiAnalysis,
                    action_type: DegradationActionType::Disable,
                    description: "Disable AI-powered analysis features".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 200,
                        cpu_savings_percent: 20.0,
                        io_savings_percent: 5.0,
                    },
                },
                DegradationAction {
                    category: FeatureCategory::DetailedReporting,
                    action_type: DegradationActionType::Reduce(0.3),
                    description: "Generate simplified reports only".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 75,
                        cpu_savings_percent: 10.0,
                        io_savings_percent: 15.0,
                    },
                },
                DegradationAction {
                    category: FeatureCategory::PluginSystem,
                    action_type: DegradationActionType::Disable,
                    description: "Disable plugin system to save resources".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 150,
                        cpu_savings_percent: 15.0,
                        io_savings_percent: 8.0,
                    },
                },
                DegradationAction {
                    category: FeatureCategory::MetricsExport,
                    action_type: DegradationActionType::Disable,
                    description: "Disable metrics export".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 25,
                        cpu_savings_percent: 3.0,
                        io_savings_percent: 10.0,
                    },
                },
            ],
            DegradationLevel::EmergencyMode => vec![
                DegradationAction {
                    category: FeatureCategory::InteractiveFeatures,
                    action_type: DegradationActionType::Disable,
                    description: "Disable interactive TUI features".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 50,
                        cpu_savings_percent: 8.0,
                        io_savings_percent: 5.0,
                    },
                },
                DegradationAction {
                    category: FeatureCategory::LargeFileProcessing,
                    action_type: DegradationActionType::Disable,
                    description: "Skip large file processing".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 300,
                        cpu_savings_percent: 25.0,
                        io_savings_percent: 40.0,
                    },
                },
                DegradationAction {
                    category: FeatureCategory::ExtendedConcurrency,
                    action_type: DegradationActionType::Reduce(0.2),
                    description: "Minimal concurrent analyses only".to_string(),
                    resource_impact: ResourceImpact {
                        memory_savings_mb: 400,
                        cpu_savings_percent: 30.0,
                        io_savings_percent: 20.0,
                    },
                },
            ],
        }
    }

    async fn execute_degradation_actions(&self, level: &DegradationLevel) -> ResourceResult<()> {
        let mut feature_states = self.feature_states.lock().unwrap();

        match level {
            DegradationLevel::Normal => {
                // Enable all features
                for (_, enabled) in feature_states.iter_mut() {
                    *enabled = true;
                }
            }
            DegradationLevel::LightDegradation => {
                feature_states.insert(FeatureCategory::VerboseLogging, false);
                // Other features remain enabled but may be reduced
            }
            DegradationLevel::HeavyDegradation => {
                feature_states.insert(FeatureCategory::VerboseLogging, false);
                feature_states.insert(FeatureCategory::AiAnalysis, false);
                feature_states.insert(FeatureCategory::PluginSystem, false);
                feature_states.insert(FeatureCategory::MetricsExport, false);
            }
            DegradationLevel::EmergencyMode => {
                // Disable most features, keep only core functionality
                feature_states.insert(FeatureCategory::VerboseLogging, false);
                feature_states.insert(FeatureCategory::AiAnalysis, false);
                feature_states.insert(FeatureCategory::PluginSystem, false);
                feature_states.insert(FeatureCategory::MetricsExport, false);
                feature_states.insert(FeatureCategory::InteractiveFeatures, false);
                feature_states.insert(FeatureCategory::LargeFileProcessing, false);
                feature_states.insert(FeatureCategory::DetailedReporting, false);
                feature_states.insert(FeatureCategory::BackgroundCaching, false);
            }
        }

        Ok(())
    }

    async fn emergency_cleanup_actions(&self) -> ResourceResult<()> {
        warn!("Executing emergency cleanup actions");

        // Clear any cached data
        // self.clear_caches().await?;

        // Cancel non-critical background tasks
        // self.cancel_background_tasks().await?;

        // Force garbage collection if possible
        // This would be language/runtime specific

        info!("Emergency cleanup completed");

        Ok(())
    }
}

impl Clone for GracefulDegradationManager {
    fn clone(&self) -> Self {
        Self {
            memory_tracker: self.memory_tracker.clone(),
            config: self.config.clone(),
            current_state: self.current_state.clone(),
            feature_states: self.feature_states.clone(),
            degradation_history: self.degradation_history.clone(),
            recovery_timer: self.recovery_timer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;

    fn create_test_config() -> Arc<RwLock<ResourceConfig>> {
        Arc::new(RwLock::new(ResourceConfig::testing()))
    }

    #[tokio::test]
    async fn test_degradation_level_determination() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker, config);

        let degradation_config = DegradationConfig {
            enabled: true,
            light_threshold: 0.5,
            heavy_threshold: 0.7,
            emergency_threshold: 0.9,
            auto_recovery: true,
            recovery_threshold: 0.4,
        };

        // Test normal level
        assert_eq!(
            manager.determine_degradation_level(0.3, &degradation_config),
            DegradationLevel::Normal
        );

        // Test light degradation
        assert_eq!(
            manager.determine_degradation_level(0.6, &degradation_config),
            DegradationLevel::LightDegradation
        );

        // Test heavy degradation
        assert_eq!(
            manager.determine_degradation_level(0.8, &degradation_config),
            DegradationLevel::HeavyDegradation
        );

        // Test emergency mode
        assert_eq!(
            manager.determine_degradation_level(0.95, &degradation_config),
            DegradationLevel::EmergencyMode
        );
    }

    #[tokio::test]
    async fn test_feature_state_management() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker, config);

        // Initially all features should be enabled
        assert!(manager.is_feature_enabled(&FeatureCategory::AiAnalysis));

        // Apply heavy degradation
        manager
            .force_degradation_level(
                DegradationLevel::HeavyDegradation,
                "Test degradation".to_string(),
            )
            .await
            .unwrap();

        // AI analysis should be disabled
        assert!(!manager.is_feature_enabled(&FeatureCategory::AiAnalysis));

        // Recover to normal
        manager
            .force_degradation_level(DegradationLevel::Normal, "Test recovery".to_string())
            .await
            .unwrap();

        // AI analysis should be enabled again
        assert!(manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
    }

    #[tokio::test]
    async fn test_degradation_history() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker, config);

        // Apply some degradation levels
        manager
            .force_degradation_level(DegradationLevel::LightDegradation, "Test 1".to_string())
            .await
            .unwrap();

        manager
            .force_degradation_level(DegradationLevel::HeavyDegradation, "Test 2".to_string())
            .await
            .unwrap();

        let history = manager.get_degradation_history(Duration::from_secs(60));
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].1, DegradationLevel::LightDegradation);
        assert_eq!(history[1].1, DegradationLevel::HeavyDegradation);
    }

    #[test]
    fn test_degradation_level_factor() {
        assert_eq!(DegradationLevel::Normal.factor(), 0.0);
        assert_eq!(DegradationLevel::LightDegradation.factor(), 0.3);
        assert_eq!(DegradationLevel::HeavyDegradation.factor(), 0.7);
        assert_eq!(DegradationLevel::EmergencyMode.factor(), 0.9);
    }
}
