//! Progress tracking service for monitoring analysis workflows
//!
//! This module provides comprehensive progress tracking capabilities
//! for long-running analysis operations, with support for nested
//! operations and real-time progress reporting.

use crate::error::UveddiError;
use crate::core::logging::{debug, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

use super::traits::{Service, HealthCheck, ServiceHealth};

/// Progress tracking service for monitoring analysis operations
pub struct ProgressTracker {
    /// Current progress state
    state: Arc<RwLock<ProgressState>>,
    /// Progress update channel
    update_sender: Option<mpsc::UnboundedSender<ProgressUpdate>>,
    /// Service running state
    is_running: bool,
    /// Configuration
    config: ProgressConfig,
}

/// Configuration for progress tracking
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// Maximum number of progress updates to buffer
    pub max_buffer_size: usize,
    /// Minimum interval between progress updates (to avoid spam)
    pub min_update_interval: Duration,
    /// Enable detailed stage tracking
    pub enable_stage_tracking: bool,
    /// Enable performance metrics collection
    pub enable_metrics: bool,
}

/// Current progress state
#[derive(Debug, Clone)]
pub struct ProgressState {
    /// Overall progress (0.0 to 1.0)
    pub overall_progress: f64,
    /// Current stage name
    pub current_stage: String,
    /// Current operation description
    pub current_operation: String,
    /// Progress by stage
    pub stage_progress: HashMap<String, StageProgress>,
    /// Start time of the overall operation
    pub start_time: Instant,
    /// Performance metrics
    pub metrics: ProgressMetrics,
    /// Whether the operation is complete
    pub is_complete: bool,
    /// Any errors encountered
    pub errors: Vec<String>,
}

/// Progress information for a specific stage
#[derive(Debug, Clone)]
pub struct StageProgress {
    /// Current progress within this stage (0.0 to 1.0)
    pub progress: f64,
    /// Current item being processed
    pub current_item: Option<String>,
    /// Total items to process (if known)
    pub total_items: Option<usize>,
    /// Items completed
    pub completed_items: usize,
    /// Stage start time
    pub start_time: Instant,
    /// Estimated time remaining
    pub estimated_remaining: Option<Duration>,
}

/// Performance metrics for progress tracking
#[derive(Debug, Clone, Default)]
pub struct ProgressMetrics {
    /// Items processed per second
    pub items_per_second: f64,
    /// Total processing time
    pub total_duration: Duration,
    /// Time spent in each stage
    pub stage_durations: HashMap<String, Duration>,
    /// Peak memory usage (if available)
    pub peak_memory_usage: Option<usize>,
}

/// Progress update message
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    /// Current progress value
    pub current: usize,
    /// Total expected items (if known)
    pub total: Option<usize>,
    /// Human-readable message
    pub message: String,
    /// Stage identifier
    pub stage: String,
}

/// Progress callback trait for receiving updates
pub trait ProgressCallback: Send + Sync {
    /// Called when progress is updated
    fn on_progress(&self, state: &ProgressState);

    /// Called when a stage is completed
    fn on_stage_complete(&self, stage: &str, duration: Duration);

    /// Called when an error occurs
    fn on_error(&self, error: &str);
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ProgressState::new())),
            update_sender: None,
            is_running: false,
            config: ProgressConfig::default(),
        }
    }

    /// Create a new progress tracker with custom configuration
    pub fn with_config(config: ProgressConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(ProgressState::new())),
            update_sender: None,
            is_running: false,
            config,
        }
    }

    /// Start a new operation tracking
    pub async fn start_operation(&mut self, operation_name: String) -> Result<(), UveddiError> {
        if !self.is_running {
            return Err(UveddiError::config_error(
                "Progress tracker is not running",
                "service state",
            ));
        }

        debug!("Starting operation tracking: {}", operation_name);

        let mut state = self.state.write().await;
        state.current_operation = operation_name;
        state.start_time = Instant::now();
        state.overall_progress = 0.0;
        state.is_complete = false;
        state.stage_progress.clear();
        state.errors.clear();
        state.metrics = ProgressMetrics::default();

        info!("Operation tracking started: {}", state.current_operation);
        Ok(())
    }

    /// Start tracking a new stage
    pub async fn start_stage(&mut self, stage_name: String) -> Result<(), UveddiError> {
        debug!("Starting stage: {}", stage_name);

        let mut state = self.state.write().await;
        state.current_stage = stage_name.clone();

        if self.config.enable_stage_tracking {
            state.stage_progress.insert(stage_name.clone(), StageProgress {
                progress: 0.0,
                current_item: None,
                total_items: None,
                completed_items: 0,
                start_time: Instant::now(),
                estimated_remaining: None,
            });
        }

        info!("Stage started: {}", stage_name);
        Ok(())
    }

    /// Update progress for the current stage
    pub async fn update(&self, update: ProgressUpdate) {
        if !self.is_running {
            return;
        }

        let mut state = self.state.write().await;

        // Update stage progress
        if let Some(stage_progress) = state.stage_progress.get_mut(&update.stage) {
            if let Some(total) = update.total {
                stage_progress.total_items = Some(total);
                stage_progress.progress = if total > 0 {
                    update.current as f64 / total as f64
                } else {
                    1.0
                };
            }
            stage_progress.completed_items = update.current;
            stage_progress.current_item = Some(update.message.clone());

            // Calculate estimated remaining time
            if let Some(total) = update.total {
                let elapsed = stage_progress.start_time.elapsed();
                let rate = if elapsed.as_secs() > 0 {
                    update.current as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };

                if rate > 0.0 {
                    let remaining_items = total.saturating_sub(update.current);
                    let estimated_seconds = remaining_items as f64 / rate;
                    stage_progress.estimated_remaining = Some(Duration::from_secs_f64(estimated_seconds));
                }
            }
        }

        // Update overall progress
        if !state.stage_progress.is_empty() {
            let total_progress: f64 = state.stage_progress.values()
                .map(|sp| sp.progress)
                .sum();
            state.overall_progress = total_progress / state.stage_progress.len() as f64;
        }

        // Update metrics if enabled
        if self.config.enable_metrics {
            let elapsed = state.start_time.elapsed();
            let total_items: usize = state.stage_progress.values()
                .map(|sp| sp.completed_items)
                .sum();

            if elapsed.as_secs() > 0 {
                state.metrics.items_per_second = total_items as f64 / elapsed.as_secs_f64();
            }
            state.metrics.total_duration = elapsed;
        }

        debug!("Progress updated: {} - {}", update.stage, update.message);
    }

    /// Complete the current stage
    pub async fn complete_stage(&mut self, stage_name: &str) -> Result<(), UveddiError> {
        debug!("Completing stage: {}", stage_name);

        let mut state = self.state.write().await;

        if let Some(stage_progress) = state.stage_progress.get_mut(stage_name) {
            stage_progress.progress = 1.0;
            let duration = stage_progress.start_time.elapsed();

            if self.config.enable_metrics {
                state.metrics.stage_durations.insert(stage_name.to_string(), duration);
            }

            info!("Stage completed: {} (duration: {:?})", stage_name, duration);
        }

        Ok(())
    }

    /// Complete the entire operation
    pub async fn complete_operation(&mut self) -> Result<(), UveddiError> {
        debug!("Completing operation");

        let mut state = self.state.write().await;
        state.overall_progress = 1.0;
        state.is_complete = true;

        if self.config.enable_metrics {
            state.metrics.total_duration = state.start_time.elapsed();
        }

        info!("Operation completed: {} (total duration: {:?})",
              state.current_operation, state.metrics.total_duration);
        Ok(())
    }

    /// Add an error to the progress tracking
    pub async fn add_error(&self, error: String) {
        let mut state = self.state.write().await;
        state.errors.push(error.clone());
        debug!("Error added to progress tracking: {}", error);
    }

    /// Get the current progress state
    pub async fn get_state(&self) -> ProgressState {
        self.state.read().await.clone()
    }

    /// Get progress for a specific stage
    pub async fn get_stage_progress(&self, stage_name: &str) -> Option<StageProgress> {
        let state = self.state.read().await;
        state.stage_progress.get(stage_name).cloned()
    }

    /// Check if the operation is complete
    pub async fn is_complete(&self) -> bool {
        let state = self.state.read().await;
        state.is_complete
    }

    /// Get the overall progress percentage
    pub async fn get_progress_percentage(&self) -> f64 {
        let state = self.state.read().await;
        (state.overall_progress * 100.0).round()
    }

    /// Register a progress callback
    pub async fn register_callback<C>(&mut self, _callback: C) -> Result<(), UveddiError>
    where
        C: ProgressCallback + 'static,
    {
        // TODO: Implement callback registration
        // This would involve storing callbacks and calling them on updates
        Ok(())
    }

    /// Reset the progress tracker state
    pub async fn reset(&mut self) {
        let mut state = self.state.write().await;
        *state = ProgressState::new();
        debug!("Progress tracker state reset");
    }
}

impl ProgressState {
    /// Create a new progress state
    pub fn new() -> Self {
        Self {
            overall_progress: 0.0,
            current_stage: String::new(),
            current_operation: String::new(),
            stage_progress: HashMap::new(),
            start_time: Instant::now(),
            metrics: ProgressMetrics::default(),
            is_complete: false,
            errors: Vec::new(),
        }
    }

    /// Get a human-readable progress summary
    pub fn get_summary(&self) -> String {
        let percentage = (self.overall_progress * 100.0).round();
        if self.is_complete {
            format!("Completed: {} (100%)", self.current_operation)
        } else if self.current_stage.is_empty() {
            format!("{}% - {}", percentage, self.current_operation)
        } else {
            format!("{}% - {} - {}", percentage, self.current_stage, self.current_operation)
        }
    }

    /// Get estimated time remaining for the entire operation
    pub fn get_estimated_remaining(&self) -> Option<Duration> {
        if self.overall_progress <= 0.0 {
            return None;
        }

        let elapsed = self.start_time.elapsed();
        let estimated_total = elapsed.as_secs_f64() / self.overall_progress;
        let remaining = estimated_total - elapsed.as_secs_f64();

        if remaining > 0.0 {
            Some(Duration::from_secs_f64(remaining))
        } else {
            None
        }
    }
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1000,
            min_update_interval: Duration::from_millis(100),
            enable_stage_tracking: true,
            enable_metrics: true,
        }
    }
}

impl Service for ProgressTracker {
    async fn start(&mut self) -> Result<(), UveddiError> {
        if self.is_running {
            return Ok(());
        }

        debug!("Starting progress tracker service");
        self.is_running = true;
        info!("Progress tracker service started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), UveddiError> {
        if !self.is_running {
            return Ok(());
        }

        debug!("Stopping progress tracker service");
        self.is_running = false;
        info!("Progress tracker service stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn name(&self) -> &str {
        "progress_tracker"
    }
}

impl HealthCheck for ProgressTracker {
    async fn health_check(&self) -> Result<ServiceHealth, UveddiError> {
        if !self.is_running {
            return Ok(ServiceHealth::Unhealthy("Service not running".to_string()));
        }

        // Check if we can read the state
        match self.state.try_read() {
            Ok(_) => Ok(ServiceHealth::Healthy),
            Err(_) => Ok(ServiceHealth::Degraded("State lock contention".to_string())),
        }
    }
}