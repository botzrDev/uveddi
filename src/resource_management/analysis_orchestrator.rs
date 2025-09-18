//! Analysis orchestrator with concurrent analysis limiting and resource management

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::time::timeout;
use uuid::Uuid;

use super::{
    error::{ResourceError, ResourceResult},
    memory_tracker::{MemoryGuard, MemoryTracker},
    resource_config::{ProcessingLimits, ResourceConfig},
};

/// Manages concurrent analysis execution with resource limits
pub struct AnalysisOrchestrator {
    memory_tracker: Arc<MemoryTracker>,
    concurrency_limiter: Arc<Semaphore>,
    config: Arc<Mutex<ProcessingLimits>>,
    active_analyses: Arc<Mutex<HashMap<Uuid, AnalysisInfo>>>,
    analysis_queue: Arc<Mutex<Vec<PendingAnalysis>>>,
    statistics: Arc<Mutex<OrchestrationStats>>,
}

/// Information about an active analysis
#[derive(Debug, Clone)]
struct AnalysisInfo {
    id: Uuid,
    component: String,
    started_at: Instant,
    memory_allocated: u64,
    timeout: Duration,
}

/// Pending analysis waiting for resources
#[derive(Debug)]
struct PendingAnalysis {
    id: Uuid,
    component: String,
    estimated_memory: u64,
    priority: AnalysisPriority,
    queued_at: Instant,
}

/// Priority levels for analysis execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalysisPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Statistics about orchestration performance
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrchestrationStats {
    pub total_analyses: u64,
    pub completed_analyses: u64,
    pub failed_analyses: u64,
    pub timeout_analyses: u64,
    pub queued_analyses: u64,
    pub peak_concurrent: usize,
    pub average_duration_ms: f64,
    pub memory_pressure_rejects: u64,
    pub concurrency_limit_hits: u64,
}

/// Concurrency limits that can be updated at runtime
#[derive(Debug, Clone)]
pub struct ConcurrencyLimit {
    pub max_concurrent: usize,
    pub max_queued: usize,
    pub timeout: Duration,
}

/// Analysis execution context
pub struct AnalysisExecution<'a> {
    id: Uuid,
    permit: Option<SemaphorePermit<'a>>,
    memory_guard: Option<MemoryGuard>,
    started_at: Instant,
    orchestrator: Arc<AnalysisOrchestrator>,
}

impl AnalysisOrchestrator {
    /// Creates a new analysis orchestrator
    pub fn new(
        memory_tracker: Arc<MemoryTracker>,
        config: Arc<tokio::sync::RwLock<ResourceConfig>>,
    ) -> ResourceResult<Self> {
        let config_guard = config.blocking_read();
        let processing_limits = config_guard.processing.clone();

        let concurrency_limiter =
            Arc::new(Semaphore::new(processing_limits.max_concurrent_analyses));

        Ok(Self {
            memory_tracker,
            concurrency_limiter,
            config: Arc::new(Mutex::new(processing_limits)),
            active_analyses: Arc::new(Mutex::new(HashMap::new())),
            analysis_queue: Arc::new(Mutex::new(Vec::new())),
            statistics: Arc::new(Mutex::new(OrchestrationStats::default())),
        })
    }

    /// Attempts to start a new analysis
    pub async fn start_analysis<'a>(
        &'a self,
        component: &str,
        estimated_memory: u64,
        priority: AnalysisPriority,
    ) -> ResourceResult<AnalysisExecution<'a>> {
        let analysis_id = Uuid::new_v4();

        // Check if we can start immediately or need to queue
        if self.can_start_immediately(estimated_memory).await {
            self.execute_analysis(analysis_id, component, estimated_memory)
                .await
        } else {
            self.queue_analysis(analysis_id, component, estimated_memory, priority)
                .await
        }
    }

    /// Checks if a new analysis can be accepted
    pub async fn can_accept_new_analysis(&self) -> bool {
        let stats = self.statistics.lock().unwrap();
        let config = self.config.lock().unwrap();

        // Check queue capacity
        if stats.queued_analyses >= (config.max_concurrent_analyses * 2) as u64 {
            return false;
        }

        // Check memory pressure
        let memory_pressure = self.memory_tracker.get_memory_pressure();
        if memory_pressure > 0.9 {
            return false;
        }

        true
    }

    /// Gets count of currently active analyses
    pub fn get_active_count(&self) -> usize {
        self.active_analyses.lock().unwrap().len()
    }

    /// Updates concurrency limits
    pub async fn update_limits(&self, config: &ResourceConfig) -> ResourceResult<()> {
        let mut current_config = self.config.lock().unwrap();
        *current_config = config.processing.clone();

        // Update semaphore permits (this is approximate since we can't directly change semaphore size)
        let current_permits = self.concurrency_limiter.available_permits();
        let target_permits = config.processing.max_concurrent_analyses;

        if target_permits > current_permits {
            // Add permits
            self.concurrency_limiter
                .add_permits(target_permits - current_permits);
        }
        // Note: We can't easily remove permits from a semaphore, so we rely on
        // natural completion of analyses to reduce active count

        Ok(())
    }

    /// Gets orchestration statistics
    pub fn get_statistics(&self) -> OrchestrationStats {
        self.statistics.lock().unwrap().clone()
    }

    /// Performs emergency cleanup of analyses
    pub async fn emergency_cleanup(&self) -> ResourceResult<()> {
        let mut active = self.active_analyses.lock().unwrap();
        let mut stats = self.statistics.lock().unwrap();

        // Cancel all non-critical analyses
        let to_cancel: Vec<_> = active.keys().cloned().collect();

        for analysis_id in to_cancel {
            // In a real implementation, we would have a way to cancel running analyses
            // For now, we just remove them from tracking
            active.remove(&analysis_id);
        }

        stats.failed_analyses += active.len() as u64;

        // Clear analysis queue except critical priority
        let mut queue = self.analysis_queue.lock().unwrap();
        let critical_analyses: Vec<_> = queue
            .drain(..)
            .filter(|analysis| analysis.priority == AnalysisPriority::Critical)
            .collect();

        queue.extend(critical_analyses);

        Ok(())
    }

    /// Processes queued analyses when resources become available
    pub async fn process_queue(&self) -> ResourceResult<usize> {
        let mut processed = 0;

        loop {
            let next_analysis = {
                let mut queue = self.analysis_queue.lock().unwrap();
                if queue.is_empty() {
                    break;
                }

                // Sort by priority and age
                queue.sort_by(|a, b| {
                    b.priority
                        .cmp(&a.priority)
                        .then_with(|| a.queued_at.cmp(&b.queued_at))
                });

                // Take the highest priority, oldest analysis
                let analysis = queue.remove(0);

                // Update statistics
                let mut stats = self.statistics.lock().unwrap();
                stats.queued_analyses -= 1;

                analysis
            };

            // Try to start the analysis
            if self
                .can_start_immediately(next_analysis.estimated_memory)
                .await
            {
                match self
                    .execute_analysis(
                        next_analysis.id,
                        &next_analysis.component,
                        next_analysis.estimated_memory,
                    )
                    .await
                {
                    Ok(_) => processed += 1,
                    Err(_) => {
                        // Put it back in the queue if it fails
                        let mut queue = self.analysis_queue.lock().unwrap();
                        queue.push(PendingAnalysis {
                            queued_at: Instant::now(), // Update queue time
                            ..next_analysis
                        });

                        let mut stats = self.statistics.lock().unwrap();
                        stats.queued_analyses += 1;
                        break;
                    }
                }
            } else {
                // Put it back in the queue
                let mut queue = self.analysis_queue.lock().unwrap();
                queue.push(next_analysis);

                let mut stats = self.statistics.lock().unwrap();
                stats.queued_analyses += 1;
                break;
            }
        }

        Ok(processed)
    }

    async fn can_start_immediately(&self, estimated_memory: u64) -> bool {
        // Check concurrency limit
        if self.concurrency_limiter.available_permits() == 0 {
            return false;
        }

        // Check memory availability
        let memory_stats = self.memory_tracker.get_usage_stats();
        if memory_stats.available < estimated_memory {
            return false;
        }

        // Check memory pressure
        let memory_pressure = self.memory_tracker.get_memory_pressure();
        if memory_pressure > 0.8 {
            return false;
        }

        true
    }

    async fn execute_analysis<'a>(
        &'a self,
        analysis_id: Uuid,
        component: &str,
        estimated_memory: u64,
    ) -> ResourceResult<AnalysisExecution<'a>> {
        // Acquire concurrency permit
        let permit = self.concurrency_limiter.acquire().await.map_err(|_| {
            ResourceError::ResourceUnavailable("Failed to acquire concurrency permit".to_string())
        })?;

        // Allocate memory
        let memory_guard = self.memory_tracker.allocate(component, estimated_memory)?;

        let config = self.config.lock().unwrap();
        let timeout_duration = config.analysis_timeout;

        // Register active analysis
        let analysis_info = AnalysisInfo {
            id: analysis_id,
            component: component.to_string(),
            started_at: Instant::now(),
            memory_allocated: estimated_memory,
            timeout: timeout_duration,
        };

        self.active_analyses
            .lock()
            .unwrap()
            .insert(analysis_id, analysis_info);

        // Update statistics
        {
            let mut stats = self.statistics.lock().unwrap();
            stats.total_analyses += 1;
            let current_active = self.active_analyses.lock().unwrap().len();
            if current_active > stats.peak_concurrent {
                stats.peak_concurrent = current_active;
            }
        }

        Ok(AnalysisExecution {
            id: analysis_id,
            permit: Some(permit),
            memory_guard: Some(memory_guard),
            started_at: Instant::now(),
            orchestrator: Arc::new(self.clone()),
        })
    }

    async fn queue_analysis<'a>(
        &'a self,
        analysis_id: Uuid,
        component: &str,
        estimated_memory: u64,
        priority: AnalysisPriority,
    ) -> ResourceResult<AnalysisExecution<'a>> {
        let pending = PendingAnalysis {
            id: analysis_id,
            component: component.to_string(),
            estimated_memory,
            priority,
            queued_at: Instant::now(),
        };

        // Add to queue
        {
            let mut queue = self.analysis_queue.lock().unwrap();
            let mut stats = self.statistics.lock().unwrap();

            // Check queue capacity
            let config = self.config.lock().unwrap();
            let max_queued = config.max_concurrent_analyses * 2;

            if queue.len() >= max_queued {
                return Err(ResourceError::ResourceLimitExceeded {
                    resource_type: "analysis_queue".to_string(),
                    limit: max_queued as u64,
                    requested: queue.len() as u64 + 1,
                });
            }

            queue.push(pending);
            stats.queued_analyses += 1;
        }

        // Wait for resources to become available and try again
        // In a real implementation, this would use event notification
        // For now, we'll use a simple retry mechanism
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            if self.can_start_immediately(estimated_memory).await {
                return self
                    .execute_analysis(analysis_id, component, estimated_memory)
                    .await;
            }
        }
    }
}

impl Clone for AnalysisOrchestrator {
    fn clone(&self) -> Self {
        Self {
            memory_tracker: self.memory_tracker.clone(),
            concurrency_limiter: self.concurrency_limiter.clone(),
            config: self.config.clone(),
            active_analyses: self.active_analyses.clone(),
            analysis_queue: self.analysis_queue.clone(),
            statistics: self.statistics.clone(),
        }
    }
}

impl<'a> AnalysisExecution<'a> {
    /// Gets the analysis ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Gets the elapsed time since analysis started
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Completes the analysis successfully
    pub fn complete(mut self) -> Duration {
        let duration = self.elapsed();
        self.cleanup(true, None);
        duration
    }

    /// Fails the analysis with an error
    pub fn fail(mut self, error: ResourceError) -> Duration {
        let duration = self.elapsed();
        self.cleanup(false, Some(error));
        duration
    }

    fn cleanup(&mut self, success: bool, error: Option<ResourceError>) {
        // Calculate duration before borrowing
        let duration_ms = self.elapsed().as_millis() as f64;

        // Remove from active analyses
        if let Some(orchestrator) = Arc::get_mut(&mut self.orchestrator) {
            orchestrator
                .active_analyses
                .lock()
                .unwrap()
                .remove(&self.id);

            // Update statistics
            let mut stats = orchestrator.statistics.lock().unwrap();
            if success {
                stats.completed_analyses += 1;
            } else {
                stats.failed_analyses += 1;

                if let Some(ResourceError::AnalysisTimeout { .. }) = error {
                    stats.timeout_analyses += 1;
                }
            }

            // Update average duration
            if stats.completed_analyses > 0 {
                stats.average_duration_ms = (stats.average_duration_ms
                    * (stats.completed_analyses - 1) as f64
                    + duration_ms)
                    / stats.completed_analyses as f64;
            } else {
                stats.average_duration_ms = duration_ms;
            }
        }

        // Release resources
        self.permit.take();
        self.memory_guard.take();
    }
}

impl<'a> Drop for AnalysisExecution<'a> {
    fn drop(&mut self) {
        // Ensure cleanup happens even if complete/fail wasn't called
        if self.permit.is_some() || self.memory_guard.is_some() {
            self.cleanup(
                false,
                Some(ResourceError::SystemError(
                    "Analysis dropped without completion".to_string(),
                )),
            );
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
    async fn test_analysis_orchestration() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let orchestrator = AnalysisOrchestrator::new(memory_tracker, config).unwrap();

        // Start an analysis
        let execution = orchestrator
            .start_analysis("test_analysis", 100, AnalysisPriority::Normal)
            .await
            .unwrap();

        assert_eq!(orchestrator.get_active_count(), 1);

        // Complete the analysis
        let duration = execution.complete();
        assert!(duration > Duration::from_nanos(0));

        // Check statistics
        let stats = orchestrator.get_statistics();
        assert_eq!(stats.completed_analyses, 1);
        assert_eq!(stats.total_analyses, 1);
    }

    #[tokio::test]
    async fn test_concurrency_limiting() {
        let memory_tracker = Arc::new(MemoryTracker::new(10000).unwrap());
        let config = create_test_config();
        let orchestrator = AnalysisOrchestrator::new(memory_tracker, config).unwrap();

        // Start multiple analyses up to the limit
        let mut executions = Vec::new();

        for i in 0..2 {
            // Testing config allows max 2 concurrent
            let execution = orchestrator
                .start_analysis(&format!("test_{}", i), 100, AnalysisPriority::Normal)
                .await
                .unwrap();

            executions.push(execution);
        }

        assert_eq!(orchestrator.get_active_count(), 2);

        // Clean up
        for execution in executions {
            execution.complete();
        }
    }

    #[tokio::test]
    async fn test_memory_limit_enforcement() {
        let memory_tracker = Arc::new(MemoryTracker::new(500).unwrap());
        let config = create_test_config();
        let orchestrator = AnalysisOrchestrator::new(memory_tracker, config).unwrap();

        // Try to start an analysis that would exceed memory
        let result = orchestrator
            .start_analysis(
                "large_analysis",
                600, // Exceeds the 500 byte limit
                AnalysisPriority::Normal,
            )
            .await;

        assert!(result.is_err());
    }
}
