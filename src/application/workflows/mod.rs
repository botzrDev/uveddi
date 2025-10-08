//! Workflow orchestration modules for coordinating analysis processes
//!
//! This module provides workflow coordinators that manage the execution
//! of complex, multi-step analysis operations including core analysis,
//! report generation, and AI integration.

pub mod ai_workflow;
pub mod analysis_workflow;
pub mod report_workflow;

pub use ai_workflow::{AiWorkflow, AiWorkflowConfig};
pub use analysis_workflow::{AnalysisWorkflow, AnalysisWorkflowConfig};
pub use report_workflow::{ReportWorkflow, ReportWorkflowConfig};

use crate::error::UveddiError;

/// Common workflow traits and interfaces
pub mod traits {
    use crate::error::UveddiError;

    /// Trait for workflow execution
    pub trait Workflow<TInput, TOutput>: Send + Sync {
        /// Execute the workflow with the given input
        fn execute(&mut self, input: TInput) -> impl std::future::Future<Output = Result<TOutput, UveddiError>> + Send;

        /// Get the workflow name
        fn name(&self) -> &str;

        /// Check if the workflow can handle the given input
        fn can_handle(&self, input: &TInput) -> bool;

        /// Get workflow status
        fn status(&self) -> WorkflowStatus;
    }

    /// Workflow execution status
    #[derive(Debug, Clone, PartialEq)]
    pub enum WorkflowStatus {
        Ready,
        Running,
        Completed,
        Failed(String),
        Cancelled,
    }

    /// Trait for workflows that support cancellation
    pub trait Cancellable {
        /// Cancel the currently running workflow
        fn cancel(&mut self) -> impl std::future::Future<Output = Result<(), UveddiError>> + Send;

        /// Check if the workflow is cancelled
        fn is_cancelled(&self) -> bool;
    }

    /// Trait for workflows that can be paused and resumed
    pub trait Pausable {
        /// Pause the workflow execution
        fn pause(&mut self) -> impl std::future::Future<Output = Result<(), UveddiError>> + Send;

        /// Resume the workflow execution
        fn resume(&mut self) -> impl std::future::Future<Output = Result<(), UveddiError>> + Send;

        /// Check if the workflow is paused
        fn is_paused(&self) -> bool;
    }
}

/// Workflow coordinator for managing multiple workflows
pub struct WorkflowCoordinator {
    /// Currently active workflows
    active_workflows: std::collections::HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    /// Workflow execution history
    execution_history: Vec<WorkflowExecution>,
}

/// Record of a workflow execution
#[derive(Debug, Clone)]
pub struct WorkflowExecution {
    /// Unique execution ID
    pub id: String,
    /// Workflow name
    pub workflow_name: String,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time (if completed)
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Execution status
    pub status: traits::WorkflowStatus,
    /// Execution duration
    pub duration: Option<std::time::Duration>,
    /// Any error messages
    pub error_message: Option<String>,
}

impl WorkflowCoordinator {
    /// Create a new workflow coordinator
    pub fn new() -> Self {
        Self {
            active_workflows: std::collections::HashMap::new(),
            execution_history: Vec::new(),
        }
    }

    /// Execute multiple workflows in sequence
    pub async fn execute_sequence(
        &mut self,
        workflow_names: Vec<String>,
    ) -> Result<Vec<WorkflowExecution>, UveddiError> {
        let mut results = Vec::new();

        for workflow_name in workflow_names {
            let execution = WorkflowExecution {
                id: format!("exec_{}", results.len()), // Simple ID generation without uuid
                workflow_name: workflow_name.clone(),
                start_time: chrono::Utc::now(),
                end_time: None,
                status: traits::WorkflowStatus::Running,
                duration: None,
                error_message: None,
            };

            // TODO: Execute workflow based on name
            // This would involve looking up registered workflows and executing them

            results.push(execution);
        }

        Ok(results)
    }

    /// Execute multiple workflows in parallel
    pub async fn execute_parallel(
        &mut self,
        workflow_names: Vec<String>,
    ) -> Result<Vec<WorkflowExecution>, UveddiError> {
        // TODO: Implement parallel workflow execution
        self.execute_sequence(workflow_names).await
    }

    /// Get execution history
    pub fn get_execution_history(&self) -> &[WorkflowExecution] {
        &self.execution_history
    }

    /// Get currently active workflows
    pub fn get_active_workflows(&self) -> Vec<String> {
        self.active_workflows.keys().cloned().collect()
    }

    /// Cancel all active workflows
    pub async fn cancel_all(&mut self) -> Result<(), UveddiError> {
        // TODO: Implement cancellation for all active workflows
        self.active_workflows.clear();
        Ok(())
    }
}

impl Default for WorkflowCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
