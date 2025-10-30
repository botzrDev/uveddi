//! Core analysis workflow for coordinating code analysis operations
//!
//! This module provides the main workflow for executing codebase analysis,
//! including file discovery, processing, issue detection, and result storage.

use crate::application::configuration::AnalysisConfig;
use crate::application::orchestrator::{AnalysisOrchestrator, AnalysisResult};
use crate::application::services::{FileProcessor, ProgressTracker};
use crate::core::logging::{debug, error, info, warn};
use crate::error::UveddiError;
use std::sync::Arc;
use std::time::Instant;

use super::traits::{Cancellable, Workflow, WorkflowStatus};

/// Core analysis workflow coordinator
pub struct AnalysisWorkflow {
    /// Analysis orchestrator
    orchestrator: AnalysisOrchestrator,
    /// File processor service
    file_processor: FileProcessor,
    /// Progress tracker
    progress_tracker: Arc<ProgressTracker>,
    /// Workflow configuration
    config: AnalysisWorkflowConfig,
    /// Current workflow status
    status: WorkflowStatus,
    /// Cancellation flag
    is_cancelled: bool,
}

/// Configuration for analysis workflow
#[derive(Debug, Clone)]
pub struct AnalysisWorkflowConfig {
    /// Enable parallel file processing
    pub enable_parallel_processing: bool,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    /// Enable detailed progress reporting
    pub enable_progress_reporting: bool,
    /// Timeout for individual operations
    pub operation_timeout: std::time::Duration,
    /// Enable error recovery
    pub enable_error_recovery: bool,
    /// Maximum number of retry attempts
    pub max_retry_attempts: u32,
}

/// Input for analysis workflow
pub struct AnalysisWorkflowInput {
    /// Analysis configuration
    pub config: AnalysisConfig,
    /// Optional custom progress callback
    pub progress_callback: Option<Box<dyn Fn(f64, String) + Send + Sync>>,
}

impl std::fmt::Debug for AnalysisWorkflowInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisWorkflowInput")
            .field("config", &self.config)
            .field(
                "progress_callback",
                &self.progress_callback.as_ref().map(|_| "Some(callback)"),
            )
            .finish()
    }
}

/// Output from analysis workflow
#[derive(Debug)]
pub struct AnalysisWorkflowOutput {
    /// Analysis results
    pub result: AnalysisResult,
    /// Workflow execution metrics
    pub metrics: WorkflowMetrics,
    /// Any warnings encountered
    pub warnings: Vec<String>,
}

/// Metrics collected during workflow execution
#[derive(Debug, Clone)]
pub struct WorkflowMetrics {
    /// Total execution time
    pub total_duration: std::time::Duration,
    /// Time spent in each phase
    pub phase_durations: std::collections::HashMap<String, std::time::Duration>,
    /// Peak memory usage (if available)
    pub peak_memory_usage: Option<usize>,
    /// Files processed per second
    pub processing_rate: f64,
    /// Number of errors recovered from
    pub errors_recovered: u32,
}

impl AnalysisWorkflow {
    /// Create a new analysis workflow
    pub fn new(
        orchestrator: AnalysisOrchestrator,
        file_processor: FileProcessor,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        Self {
            orchestrator,
            file_processor,
            progress_tracker,
            config: AnalysisWorkflowConfig::default(),
            status: WorkflowStatus::Ready,
            is_cancelled: false,
        }
    }

    /// Create a new analysis workflow with custom configuration
    pub fn with_config(
        orchestrator: AnalysisOrchestrator,
        file_processor: FileProcessor,
        progress_tracker: Arc<ProgressTracker>,
        config: AnalysisWorkflowConfig,
    ) -> Self {
        Self {
            orchestrator,
            file_processor,
            progress_tracker,
            config,
            status: WorkflowStatus::Ready,
            is_cancelled: false,
        }
    }

    /// Execute the file discovery phase
    async fn execute_file_discovery(
        &mut self,
        input: &AnalysisWorkflowInput,
        metrics: &mut WorkflowMetrics,
    ) -> Result<(), UveddiError> {
        let phase_start = Instant::now();
        info!("Starting file discovery phase");

        self.progress_tracker
            .start_stage("file_discovery".to_string())
            .await?;

        // Discover files in the target path
        let discovered_files = self
            .file_processor
            .discover_files(&input.config.target_path)
            .await
            .map_err(|e| {
                error!("File discovery failed: {}", e);
                e
            })?;

        info!("Discovered {} files", discovered_files.len());

        self.progress_tracker
            .complete_stage("file_discovery")
            .await?;

        let phase_duration = phase_start.elapsed();
        metrics
            .phase_durations
            .insert("file_discovery".to_string(), phase_duration);

        debug!("File discovery phase completed in {:?}", phase_duration);
        Ok(())
    }

    /// Execute the core analysis phase
    async fn execute_core_analysis(
        &mut self,
        input: &AnalysisWorkflowInput,
        metrics: &mut WorkflowMetrics,
    ) -> Result<AnalysisResult, UveddiError> {
        let phase_start = Instant::now();
        info!("Starting core analysis phase");

        self.progress_tracker
            .start_stage("core_analysis".to_string())
            .await?;

        // Execute core analysis through orchestrator
        let result = self
            .orchestrator
            .execute_core_analysis(&input.config)
            .await
            .map_err(|e| {
                error!("Core analysis failed: {}", e);
                e
            })?;

        info!(
            "Core analysis completed: {} issues found in {} files",
            result.issues.len(),
            result.metadata.files_analyzed
        );

        self.progress_tracker
            .complete_stage("core_analysis")
            .await?;

        let phase_duration = phase_start.elapsed();
        metrics
            .phase_durations
            .insert("core_analysis".to_string(), phase_duration);

        // Calculate processing rate
        if phase_duration.as_secs() > 0 {
            metrics.processing_rate =
                result.metadata.files_analyzed as f64 / phase_duration.as_secs_f64();
        }

        debug!("Core analysis phase completed in {:?}", phase_duration);
        Ok(result)
    }

    /// Execute the validation phase
    async fn execute_validation(
        &mut self,
        result: &AnalysisResult,
        metrics: &mut WorkflowMetrics,
    ) -> Result<Vec<String>, UveddiError> {
        let phase_start = Instant::now();
        debug!("Starting validation phase");

        self.progress_tracker
            .start_stage("validation".to_string())
            .await?;

        let mut warnings = Vec::new();

        // Validate analysis results
        if result.issues.is_empty() {
            warnings.push(
                "No issues detected - this might indicate configuration problems".to_string(),
            );
        }

        if result.metadata.files_analyzed == 0 {
            warnings
                .push("No files were analyzed - check target path and file filters".to_string());
        }

        // Check for suspicious patterns in results
        let critical_issues = result
            .issues
            .iter()
            .filter(|issue| {
                issue.message.contains("critical") || issue.message.contains("Critical")
            })
            .count();

        if critical_issues > result.issues.len() / 2 {
            warnings.push(format!(
                "High proportion of critical issues ({}/{}) - verify detector sensitivity",
                critical_issues,
                result.issues.len()
            ));
        }

        self.progress_tracker.complete_stage("validation").await?;

        let phase_duration = phase_start.elapsed();
        metrics
            .phase_durations
            .insert("validation".to_string(), phase_duration);

        if !warnings.is_empty() {
            warn!("Validation completed with {} warnings", warnings.len());
            for warning in &warnings {
                warn!("Validation warning: {}", warning);
            }
        } else {
            debug!(
                "Validation phase completed successfully in {:?}",
                phase_duration
            );
        }

        Ok(warnings)
    }

    /// Handle workflow errors with optional recovery
    async fn handle_error(
        &mut self,
        error: UveddiError,
        phase: &str,
        attempt: u32,
    ) -> Result<bool, UveddiError> {
        error!("Error in {} phase (attempt {}): {}", phase, attempt, error);

        if !self.config.enable_error_recovery || attempt >= self.config.max_retry_attempts {
            return Err(error);
        }

        warn!("Attempting error recovery for {} phase", phase);

        // Add error to progress tracker
        self.progress_tracker
            .add_error(format!("{} phase error: {}", phase, error))
            .await;

        // Simple retry logic - in a real implementation, this might involve
        // more sophisticated recovery strategies
        tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt as u64)).await;

        info!("Retrying {} phase (attempt {})", phase, attempt + 1);
        Ok(true) // Indicate that retry should be attempted
    }

    /// Check for cancellation
    fn check_cancellation(&self) -> Result<(), UveddiError> {
        if self.is_cancelled {
            return Err(UveddiError::config_error(
                "Workflow was cancelled",
                "workflow execution",
            ));
        }
        Ok(())
    }
}

impl Workflow<AnalysisWorkflowInput, AnalysisWorkflowOutput> for AnalysisWorkflow {
    async fn execute(
        &mut self,
        input: AnalysisWorkflowInput,
    ) -> Result<AnalysisWorkflowOutput, UveddiError> {
        let start_time = Instant::now();
        info!(
            "Starting analysis workflow for: {}",
            input.config.target_path.display()
        );

        self.status = WorkflowStatus::Running;
        self.is_cancelled = false;

        let mut metrics = WorkflowMetrics::default();

        // Start progress tracking
        self.progress_tracker
            .start_operation("Code Analysis".to_string())
            .await?;

        let mut result = None;
        let mut warnings = Vec::new();

        // Execute workflow phases with retry logic
        for attempt in 0..=self.config.max_retry_attempts {
            self.check_cancellation()?;

            match self.execute_workflow_phases(&input, &mut metrics).await {
                Ok((analysis_result, phase_warnings)) => {
                    result = Some(analysis_result);
                    warnings = phase_warnings;
                    break;
                }
                Err(e) => {
                    if !self.handle_error(e, "workflow", attempt).await? {
                        break;
                    }
                    metrics.errors_recovered += 1;
                }
            }
        }

        let result = result.ok_or_else(|| {
            UveddiError::config_error(
                "Workflow failed after all retry attempts",
                "workflow execution",
            )
        })?;

        // Complete progress tracking
        self.progress_tracker.complete_operation().await?;

        // Finalize metrics
        metrics.total_duration = start_time.elapsed();

        self.status = WorkflowStatus::Completed;

        info!(
            "Analysis workflow completed successfully in {:?}",
            metrics.total_duration
        );

        Ok(AnalysisWorkflowOutput {
            result,
            metrics,
            warnings,
        })
    }

    fn name(&self) -> &str {
        "analysis_workflow"
    }

    fn can_handle(&self, input: &AnalysisWorkflowInput) -> bool {
        input.config.target_path.exists()
    }

    fn status(&self) -> WorkflowStatus {
        self.status.clone()
    }
}

impl AnalysisWorkflow {
    /// Execute all workflow phases
    async fn execute_workflow_phases(
        &mut self,
        input: &AnalysisWorkflowInput,
        metrics: &mut WorkflowMetrics,
    ) -> Result<(AnalysisResult, Vec<String>), UveddiError> {
        // Phase 1: File Discovery
        self.execute_file_discovery(input, metrics).await?;
        self.check_cancellation()?;

        // Phase 2: Core Analysis
        let result = self.execute_core_analysis(input, metrics).await?;
        self.check_cancellation()?;

        // Phase 3: Validation
        let warnings = self.execute_validation(&result, metrics).await?;

        Ok((result, warnings))
    }
}

impl Cancellable for AnalysisWorkflow {
    async fn cancel(&mut self) -> Result<(), UveddiError> {
        info!("Cancelling analysis workflow");
        self.is_cancelled = true;
        self.status = WorkflowStatus::Cancelled;

        // Add cancellation message to progress tracker
        self.progress_tracker
            .add_error("Workflow cancelled by user".to_string())
            .await;

        Ok(())
    }

    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }
}

impl Default for AnalysisWorkflowConfig {
    fn default() -> Self {
        Self {
            enable_parallel_processing: true,
            max_concurrent_operations: num_cpus::get(),
            enable_progress_reporting: true,
            operation_timeout: std::time::Duration::from_secs(300), // 5 minutes
            enable_error_recovery: true,
            max_retry_attempts: 3,
        }
    }
}

impl Default for WorkflowMetrics {
    fn default() -> Self {
        Self {
            total_duration: std::time::Duration::default(),
            phase_durations: std::collections::HashMap::new(),
            peak_memory_usage: None,
            processing_rate: 0.0,
            errors_recovered: 0,
        }
    }
}
