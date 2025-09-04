//! Resource management module for memory and system resource control
//! 
//! Provides comprehensive resource tracking, limiting, and graceful degradation
//! to ensure system stability under all load conditions.

pub mod memory_tracker;
pub mod resource_config;
pub mod file_processor;
pub mod analysis_orchestrator;
pub mod resource_monitor;
pub mod degradation_manager;
pub mod metrics;
pub mod error;

pub use memory_tracker::{MemoryTracker, MemoryGuard, MemoryStats};
pub use resource_config::{ResourceConfig, ResourceLimits};
pub use file_processor::{StreamingFileProcessor, ProcessingStrategy};
pub use analysis_orchestrator::{AnalysisOrchestrator, ConcurrencyLimit};
pub use resource_monitor::{ResourceMonitor, ResourceMetrics, AlertThresholds};
pub use degradation_manager::{GracefulDegradationManager, DegradationLevel};
pub use metrics::{ResourceUsage, SystemMetrics};
pub use error::{ResourceError, ResourceResult};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Main resource manager that coordinates all resource management components
pub struct ResourceManager {
    memory_tracker: Arc<MemoryTracker>,
    config: Arc<RwLock<ResourceConfig>>,
    monitor: Arc<ResourceMonitor>,
    degradation_manager: Arc<GracefulDegradationManager>,
    orchestrator: Arc<AnalysisOrchestrator>,
}

impl ResourceManager {
    /// Creates a new resource manager with the given configuration
    pub fn new(config: ResourceConfig) -> ResourceResult<Self> {
        let config = Arc::new(RwLock::new(config));
        let memory_tracker = Arc::new(MemoryTracker::new(
            config.blocking_read().max_memory_bytes,
        )?);
        
        let monitor = Arc::new(ResourceMonitor::new(
            memory_tracker.clone(),
            config.clone(),
        ));
        
        let degradation_manager = Arc::new(GracefulDegradationManager::new(
            memory_tracker.clone(),
            config.clone(),
        ));
        
        let orchestrator = Arc::new(AnalysisOrchestrator::new(
            memory_tracker.clone(),
            config.clone(),
        )?);
        
        Ok(Self {
            memory_tracker,
            config,
            monitor,
            degradation_manager,
            orchestrator,
        })
    }
    
    /// Starts resource monitoring in the background
    pub async fn start_monitoring(&self) -> ResourceResult<()> {
        self.monitor.start_monitoring().await
    }
    
    /// Gets current resource usage statistics
    pub fn get_usage_stats(&self) -> ResourceUsage {
        ResourceUsage {
            memory: self.memory_tracker.get_usage_stats(),
            active_analyses: self.orchestrator.get_active_count(),
            degradation_level: self.degradation_manager.get_current_level(),
        }
    }
    
    /// Allocates memory for a specific component
    pub fn allocate_memory(&self, component: &str, size_bytes: u64) -> ResourceResult<MemoryGuard> {
        self.memory_tracker.allocate(component, size_bytes)
    }
    
    /// Checks if the system can handle a new analysis
    pub async fn can_accept_analysis(&self) -> bool {
        self.orchestrator.can_accept_new_analysis().await
    }
    
    /// Updates resource configuration
    pub async fn update_config(&self, config: ResourceConfig) -> ResourceResult<()> {
        let mut current = self.config.write().await;
        *current = config;
        
        // Update components with new config
        self.memory_tracker.update_limit(current.max_memory_bytes)?;
        self.orchestrator.update_limits(&*current).await?;
        
        Ok(())
    }
    
    /// Triggers emergency cleanup when resources are critically low
    pub async fn emergency_cleanup(&self) -> ResourceResult<()> {
        self.degradation_manager.trigger_emergency_cleanup().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_resource_manager_creation() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).unwrap();
        
        let stats = manager.get_usage_stats();
        assert_eq!(stats.memory.current, 0);
        assert_eq!(stats.active_analyses, 0);
    }
}