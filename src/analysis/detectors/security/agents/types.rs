//! Core data structures for the multi-agent security analysis system

use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::owasp::OwaspVulnerability;
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Message types for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Task assignment from orchestrator to worker agent
    TaskAssignment {
        task_id: String,
        agent_id: String,
        task_type: TaskType,
        context: SecurityContext,
        deadline: Option<std::time::SystemTime>,
    },
    /// Progress update from worker agent to orchestrator
    ProgressUpdate {
        task_id: String,
        agent_id: String,
        progress: f64,
        status: String,
    },
    /// Task completion notification
    TaskComplete {
        task_id: String,
        agent_id: String,
        result: AgentResult,
        processing_time: Duration,
    },
    /// Error notification
    Error {
        task_id: String,
        agent_id: String,
        error: String,
    },
    /// Knowledge sharing between agents
    KnowledgeShare {
        from_agent: String,
        to_agent: String,
        knowledge_type: String,
        data: serde_json::Value,
    },
    /// Termination signal
    Shutdown,
}

/// Types of security analysis tasks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    TaintAnalysis,
    ConfigurationAnalysis,
    DependencyAnalysis,
    ValidationAnalysis,
    AiEnhancement,
    OwaspAnalysis,
    ArchitecturalCorrelation,
}

/// Result from an agent's analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentResult {
    TaintAnalysis(Vec<SecurityIssue>),
    ConfigAnalysis(Vec<SecurityIssue>),
    DependencyAnalysis(Vec<SecurityIssue>),
    ValidationResult(ValidationReport),
    AiEnhancement(AiEnhancementReport),
    OwaspAnalysis(Vec<OwaspVulnerability>),
    ArchitecturalCorrelation(CorrelationReport),
    Error(String),
}

/// Validation report from cross-validation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub validated_issues: Vec<SecurityIssue>,
    pub false_positives: Vec<String>,
    pub confidence_adjustments: HashMap<String, f64>,
    pub cross_validation_score: f64,
}

/// AI enhancement report with contextual insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEnhancementReport {
    pub enhanced_issues: Vec<SecurityIssue>,
    pub contextual_explanations: HashMap<String, String>,
    pub remediation_suggestions: HashMap<String, String>,
    pub risk_assessments: HashMap<String, f64>,
}

/// Correlation report for architectural patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationReport {
    pub architectural_correlations: HashMap<String, Vec<String>>,
    pub pattern_mappings: HashMap<String, String>,
    pub amplification_factors: HashMap<String, f64>,
}

/// Subtask for decomposed analysis
#[derive(Debug, Clone)]
pub struct SubTask {
    pub task_id: String,
    pub task_type: TaskType,
    pub agent_id: String,
    pub context: SecurityContext,
    pub priority: i32,
}

/// Metadata for tracking task execution
#[derive(Debug, Clone)]
pub struct TaskMetadata {
    pub task_id: String,
    pub task_type: TaskType,
    pub assigned_agent: String,
    pub start_time: Instant,
    pub deadline: Option<std::time::SystemTime>,
    pub status: TaskStatus,
}

/// Status of a task execution
#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

/// Trait that all security agents must implement
#[async_trait]
pub trait SecurityAgent: Send + Sync {
    /// Execute a security analysis task
    async fn execute_task(
        &self,
        task_id: String,
        context: SecurityContext,
    ) -> Result<AgentResult, AnalysisError>;

    /// Get the unique identifier for this agent
    fn get_agent_id(&self) -> &str;

    /// Get the list of task types this agent can handle
    fn get_capabilities(&self) -> Vec<TaskType>;

    /// Clone this agent into a new boxed instance
    fn clone_box(&self) -> Box<dyn SecurityAgent>;
}

/// Agent behavior pattern for malicious detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehaviorPattern {
    pub pattern_name: String,
    pub description: String,
    pub indicators: Vec<String>,
    pub risk_level: f64,
    pub language_specific: bool,
}

/// Malicious agent detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousAgentDetection {
    pub detected_patterns: Vec<AgentBehaviorPattern>,
    pub confidence_score: f64,
    pub threat_assessment: String,
    pub recommended_actions: Vec<String>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            validated_issues: Vec::new(),
            false_positives: Vec::new(),
            confidence_adjustments: HashMap::new(),
            cross_validation_score: 0.0,
        }
    }
}

impl Default for AiEnhancementReport {
    fn default() -> Self {
        Self {
            enhanced_issues: Vec::new(),
            contextual_explanations: HashMap::new(),
            remediation_suggestions: HashMap::new(),
            risk_assessments: HashMap::new(),
        }
    }
}

impl Default for CorrelationReport {
    fn default() -> Self {
        Self {
            architectural_correlations: HashMap::new(),
            pattern_mappings: HashMap::new(),
            amplification_factors: HashMap::new(),
        }
    }
}