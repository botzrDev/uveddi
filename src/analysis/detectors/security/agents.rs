//! Multi-agent system for coordinated security analysis
//!
//! This module implements a sophisticated multi-agent system (MAS) that serves as the core
//! of the security detector. The orchestrator agent decomposes tasks and delegates them to
//! specialized worker agents that collaborate via a central knowledge graph to provide
//! comprehensive security analysis with high precision and minimal false positives.
//!
//! ## Agent Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   Security Orchestrator                        │
//! │         (Task Decomposition & Result Synthesis)                │
//! └─────────────────────┬───────────────────────────────────────────┘
//!                       │
//!        ┌──────────────┼──────────────┐
//!        │              │              │
//! ┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼──────┐
//! │ Taint Agent │ │Config Agent│ │Dependency  │
//! │             │ │            │ │   Agent    │
//! └──────┬──────┘ └─────┬─────┘ └─────┬──────┘
//!        │              │              │
//!        └──────────────┼──────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────────┐
//! │           Knowledge Graph (Code-Centric RAG)                   │
//! │      (Structural Facts + Semantic Enrichments)                │
//! └─────────────────────┬───────────────────────────────────────────┘
//!                       │
//!        ┌──────────────┼──────────────┐
//!        │              │              │
//! ┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼──────┐
//! │Validation   │ │AI Agent   │ │Integration │
//! │   Agent     │ │           │ │   Agent    │
//! └─────────────┘ └───────────┘ └────────────┘
//! ```

use crate::analysis::detectors::security::config::{MultiAgentConfig, AgentConfig};
use crate::analysis::detectors::security::core::{SecurityAnalysisResult, SecurityContext, VulnerabilityDatabase};
use crate::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph;
use crate::analysis::detectors::security::owasp::OwaspVulnerability;
use crate::analysis::detectors::security::taint_analysis::TaintAnalysisEngine;
use crate::analysis::detectors::security::types::{SecurityIssue, SecuritySeverity};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub validated_issues: Vec<SecurityIssue>,
    pub false_positives: Vec<String>,
    pub confidence_adjustments: HashMap<String, f64>,
    pub cross_validation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEnhancementReport {
    pub enhanced_issues: Vec<SecurityIssue>,
    pub contextual_explanations: HashMap<String, String>,
    pub remediation_suggestions: HashMap<String, String>,
    pub risk_assessments: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationReport {
    pub architectural_correlations: HashMap<String, Vec<String>>,
    pub pattern_mappings: HashMap<String, String>,
    pub amplification_factors: HashMap<String, f64>,
}

/// Main orchestrator agent that coordinates the multi-agent security analysis
pub struct SecurityOrchestrator {
    config: MultiAgentConfig,
    agents: HashMap<String, Box<dyn SecurityAgent>>,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
    message_rx: Arc<Mutex<mpsc::UnboundedReceiver<AgentMessage>>>,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
    vulnerability_db: Arc<VulnerabilityDatabase>,
    active_tasks: Arc<RwLock<HashMap<String, TaskMetadata>>>,
}

#[derive(Debug, Clone)]
struct TaskMetadata {
    task_id: String,
    task_type: TaskType,
    assigned_agent: String,
    start_time: Instant,
    deadline: Option<std::time::SystemTime>,
    status: TaskStatus,
}

#[derive(Debug, Clone)]
enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

impl SecurityOrchestrator {
    pub fn new(
        config: MultiAgentConfig,
        knowledge_graph: Arc<SecurityKnowledgeGraph>,
        vulnerability_db: Arc<VulnerabilityDatabase>,
    ) -> Result<Self, AnalysisError> {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let message_rx = Arc::new(Mutex::new(message_rx));

        let mut orchestrator = Self {
            config,
            agents: HashMap::new(),
            message_tx,
            message_rx,
            knowledge_graph,
            vulnerability_db,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        };

        orchestrator.initialize_agents()?;
        Ok(orchestrator)
    }

    /// Initialize all configured agents
    fn initialize_agents(&mut self) -> Result<(), AnalysisError> {
        info!("Initializing security analysis agents");

        if self.config.enable_taint_agent {
            let agent = TaintAnalysisAgent::new(
                AgentConfig::new("TaintAgent".to_string()),
                self.message_tx.clone(),
                self.knowledge_graph.clone(),
            )?;
            self.agents.insert("TaintAgent".to_string(), Box::new(agent));
        }

        if self.config.enable_config_agent {
            let agent = ConfigAnalysisAgent::new(
                AgentConfig::new("ConfigAgent".to_string()),
                self.message_tx.clone(),
                self.knowledge_graph.clone(),
            )?;
            self.agents.insert("ConfigAgent".to_string(), Box::new(agent));
        }

        if self.config.enable_dependency_agent {
            let agent = DependencyAgent::new(
                AgentConfig::new("DependencyAgent".to_string()),
                self.message_tx.clone(),
                self.vulnerability_db.clone(),
            )?;
            self.agents.insert("DependencyAgent".to_string(), Box::new(agent));
        }

        if self.config.enable_validation_agent {
            let agent = ValidationAgent::new(
                AgentConfig::new("ValidationAgent".to_string()),
                self.message_tx.clone(),
            )?;
            self.agents.insert("ValidationAgent".to_string(), Box::new(agent));
        }

        info!("Initialized {} agents", self.agents.len());
        Ok(())
    }

    /// Analyze a security context using the multi-agent system
    #[instrument(skip(self, context))]
    pub async fn analyze_file(&self, context: &SecurityContext) -> Result<SecurityAnalysisResult, AnalysisError> {
        info!("Starting multi-agent security analysis for file: {:?}", context.file_path);

        let analysis_id = Uuid::new_v4().to_string();
        let mut results = SecurityAnalysisResult::new();

        // Decompose the analysis into subtasks
        let subtasks = self.decompose_analysis_tasks(context).await?;
        info!("Decomposed analysis into {} subtasks", subtasks.len());

        // Execute subtasks concurrently
        let mut task_handles = Vec::new();
        for subtask in subtasks {
            let handle = self.execute_subtask(subtask).await?;
            task_handles.push(handle);
        }

        // Collect and synthesize results
        let mut agent_results = Vec::new();
        for handle in task_handles {
            match handle.await {
                Ok(Ok(result)) => agent_results.push(result),
                Ok(Err(e)) => {
                    warn!("Subtask failed: {}", e);
                    // Continue with other subtasks
                }
                Err(e) => {
                    warn!("Task handle join failed: {}", e);
                    // Continue with other subtasks
                }
            }
        }

        // Synthesize final results
        results = self.synthesize_results(agent_results, context).await?;

        info!("Multi-agent analysis completed: {} vulnerabilities found", results.vulnerabilities.len());
        Ok(results)
    }

    /// Decompose analysis into subtasks for different agents
    async fn decompose_analysis_tasks(&self, context: &SecurityContext) -> Result<Vec<SubTask>, AnalysisError> {
        let mut subtasks = Vec::new();

        // Always include OWASP analysis as it's comprehensive
        subtasks.push(SubTask {
            task_id: Uuid::new_v4().to_string(),
            task_type: TaskType::OwaspAnalysis,
            agent_id: "OwaspAgent".to_string(),
            context: context.clone(),
            priority: 1,
        });

        // Add taint analysis if enabled
        if self.config.enable_taint_agent {
            subtasks.push(SubTask {
                task_id: Uuid::new_v4().to_string(),
                task_type: TaskType::TaintAnalysis,
                agent_id: "TaintAgent".to_string(),
                context: context.clone(),
                priority: 2,
            });
        }

        // Add configuration analysis if enabled and relevant
        if self.config.enable_config_agent && self.is_configuration_file(context) {
            subtasks.push(SubTask {
                task_id: Uuid::new_v4().to_string(),
                task_type: TaskType::ConfigurationAnalysis,
                agent_id: "ConfigAgent".to_string(),
                context: context.clone(),
                priority: 3,
            });
        }

        // Add dependency analysis if enabled and relevant
        if self.config.enable_dependency_agent && self.is_dependency_file(context) {
            subtasks.push(SubTask {
                task_id: Uuid::new_v4().to_string(),
                task_type: TaskType::DependencyAnalysis,
                agent_id: "DependencyAgent".to_string(),
                context: context.clone(),
                priority: 4,
            });
        }

        Ok(subtasks)
    }

    fn is_configuration_file(&self, context: &SecurityContext) -> bool {
        let file_name = context.file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        matches!(file_name, 
            "config.toml" | "Cargo.toml" | "package.json" | 
            "requirements.txt" | "settings.py" | "Dockerfile"
        ) || file_name.ends_with(".toml") || 
             file_name.ends_with(".json") || 
             file_name.ends_with(".yml") ||
             file_name.ends_with(".yaml")
    }

    fn is_dependency_file(&self, context: &SecurityContext) -> bool {
        let file_name = context.file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        matches!(file_name, 
            "Cargo.toml" | "Cargo.lock" | "package.json" | 
            "package-lock.json" | "requirements.txt" | 
            "Pipfile" | "Pipfile.lock"
        )
    }

    /// Execute a subtask by assigning it to the appropriate agent
    async fn execute_subtask(&self, subtask: SubTask) -> Result<tokio::task::JoinHandle<Result<AgentResult, AnalysisError>>, AnalysisError> {
        let task_metadata = TaskMetadata {
            task_id: subtask.task_id.clone(),
            task_type: subtask.task_type.clone(),
            assigned_agent: subtask.agent_id.clone(),
            start_time: Instant::now(),
            deadline: None,
            status: TaskStatus::Pending,
        };

        // Store task metadata
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(subtask.task_id.clone(), task_metadata);
        }

        // Create and spawn the task
        let task_id = subtask.task_id.clone();
        
        let handle = match subtask.task_type {
            TaskType::TaintAnalysis => {
                if let Some(agent) = self.agents.get("TaintAgent") {
                    let agent = agent.clone_box();
                    let context = subtask.context.clone();
                    tokio::spawn(async move {
                        agent.execute_task(task_id, context).await
                    })
                } else {
                    return Err(AnalysisError::DetectionError("TaintAgent not available".to_string()));
                }
            }
            TaskType::ConfigurationAnalysis => {
                if let Some(agent) = self.agents.get("ConfigAgent") {
                    let agent = agent.clone_box();
                    let context = subtask.context.clone();
                    tokio::spawn(async move {
                        agent.execute_task(task_id, context).await
                    })
                } else {
                    return Err(AnalysisError::DetectionError("ConfigAgent not available".to_string()));
                }
            }
            // Add other task types...
            _ => {
                return Err(AnalysisError::DetectionError(format!("Unsupported task type: {:?}", subtask.task_type)));
            }
        };

        Ok(handle)
    }

    /// Synthesize results from multiple agents
    async fn synthesize_results(
        &self,
        agent_results: Vec<AgentResult>,
        _context: &SecurityContext,
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        let mut final_result = SecurityAnalysisResult::new();

        for result in agent_results {
            match result {
                AgentResult::TaintAnalysis(issues) => {
                    final_result.vulnerabilities.extend(issues.into_iter().map(|issue| {
                        OwaspVulnerability {
                            category: crate::analysis::detectors::security::owasp::OwaspCategory::Injection,
                            issue_type: issue.issue_type,
                            title: issue.title,
                            description: issue.description,
                            severity: issue.severity,
                            confidence_score: issue.confidence_score,
                            location: crate::analysis::detectors::security::types::SecurityLocation {
                                file_path: issue.location.file_path,
                                start_line: issue.location.start_line,
                                end_line: issue.location.end_line,
                                start_column: issue.location.start_column,
                                end_column: issue.location.end_column,
                                function_name: issue.location.function_name,
                                class_name: issue.location.class_name,
                                module_name: issue.location.module_name,
                            },
                            remediation: issue.remediation,
                            metadata: issue.metadata,
                            architectural_correlation: issue.correlation_id.map(|id| vec![id]).unwrap_or_default(),
                        }
                    }));
                }
                AgentResult::OwaspAnalysis(vulnerabilities) => {
                    final_result.vulnerabilities.extend(vulnerabilities);
                }
                AgentResult::ValidationResult(validation_report) => {
                    // Apply validation results to adjust confidence scores
                    for (issue_id, confidence_adjustment) in validation_report.confidence_adjustments {
                        // Find and update corresponding vulnerability
                        // This would require a more sophisticated matching mechanism
                    }
                }
                // Handle other result types...
                _ => {
                    debug!("Unhandled agent result type in synthesis");
                }
            }
        }

        // Apply deduplication and final filtering
        final_result = self.deduplicate_and_filter(final_result).await?;

        Ok(final_result)
    }

    /// Remove duplicates and apply final filtering
    async fn deduplicate_and_filter(
        &self,
        mut result: SecurityAnalysisResult,
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        // Simple deduplication based on location and issue type
        result.vulnerabilities.sort_by(|a, b| {
            a.location.file_path.cmp(&b.location.file_path)
                .then(a.location.start_line.cmp(&b.location.start_line))
                .then(a.issue_type.to_string().cmp(&b.issue_type.to_string()))
        });

        result.vulnerabilities.dedup_by(|a, b| {
            a.location.file_path == b.location.file_path &&
            a.location.start_line == b.location.start_line &&
            a.issue_type == b.issue_type
        });

        // Filter by confidence threshold
        let min_confidence = 0.5; // TODO: Make this configurable
        result.vulnerabilities.retain(|v| v.confidence_score >= min_confidence);

        info!("After deduplication and filtering: {} vulnerabilities remain", result.vulnerabilities.len());
        Ok(result)
    }
}

#[derive(Debug, Clone)]
struct SubTask {
    task_id: String,
    task_type: TaskType,
    agent_id: String,
    context: SecurityContext,
    priority: i32,
}

/// Trait that all security agents must implement
#[async_trait]
pub trait SecurityAgent: Send + Sync {
    async fn execute_task(&self, task_id: String, context: SecurityContext) -> Result<AgentResult, AnalysisError>;
    fn get_agent_id(&self) -> &str;
    fn get_capabilities(&self) -> Vec<TaskType>;
    fn clone_box(&self) -> Box<dyn SecurityAgent>;
}

/// Taint Analysis Agent - specialized for data flow analysis
#[derive(Clone)]
pub struct TaintAnalysisAgent {
    config: AgentConfig,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
    taint_engine: Arc<TaintAnalysisEngine>,
}

impl TaintAnalysisAgent {
    pub fn new(
        config: AgentConfig,
        message_tx: mpsc::UnboundedSender<AgentMessage>,
        knowledge_graph: Arc<SecurityKnowledgeGraph>,
    ) -> Result<Self, AnalysisError> {
        let taint_config = crate::analysis::detectors::security::config::TaintAnalysisConfig::production();
        let taint_engine = Arc::new(TaintAnalysisEngine::new(taint_config)?);

        Ok(Self {
            config,
            message_tx,
            knowledge_graph,
            taint_engine,
        })
    }
}

#[async_trait]
impl SecurityAgent for TaintAnalysisAgent {
    async fn execute_task(&self, task_id: String, context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("TaintAnalysisAgent executing task: {}", task_id);

        // Create a ParsedFile from SecurityContext for taint analysis
        let parsed_file = context.to_parsed_file()?;
        
        // Run taint analysis
        let issues = self.taint_engine.analyze_file(&parsed_file).await?;
        
        info!("TaintAnalysisAgent completed task {}: {} issues found", task_id, issues.len());
        Ok(AgentResult::TaintAnalysis(issues))
    }

    fn get_agent_id(&self) -> &str {
        &self.config.name
    }

    fn get_capabilities(&self) -> Vec<TaskType> {
        vec![TaskType::TaintAnalysis]
    }

    fn clone_box(&self) -> Box<dyn SecurityAgent> {
        Box::new(self.clone())
    }
}

/// Configuration Analysis Agent - specialized for configuration file security
#[derive(Clone)]
pub struct ConfigAnalysisAgent {
    config: AgentConfig,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
}

impl ConfigAnalysisAgent {
    pub fn new(
        config: AgentConfig,
        message_tx: mpsc::UnboundedSender<AgentMessage>,
        knowledge_graph: Arc<SecurityKnowledgeGraph>,
    ) -> Result<Self, AnalysisError> {
        Ok(Self {
            config,
            message_tx,
            knowledge_graph,
        })
    }
}

#[async_trait]
impl SecurityAgent for ConfigAnalysisAgent {
    async fn execute_task(&self, task_id: String, _context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("ConfigAnalysisAgent executing task: {}", task_id);
        
        // TODO: Implement configuration file analysis
        let issues = Vec::new();
        
        Ok(AgentResult::ConfigAnalysis(issues))
    }

    fn get_agent_id(&self) -> &str {
        &self.config.name
    }

    fn get_capabilities(&self) -> Vec<TaskType> {
        vec![TaskType::ConfigurationAnalysis]
    }

    fn clone_box(&self) -> Box<dyn SecurityAgent> {
        Box::new(self.clone())
    }
}

/// Dependency Analysis Agent - specialized for Software Composition Analysis
#[derive(Clone)]
pub struct DependencyAgent {
    config: AgentConfig,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
    vulnerability_db: Arc<VulnerabilityDatabase>,
}

impl DependencyAgent {
    pub fn new(
        config: AgentConfig,
        message_tx: mpsc::UnboundedSender<AgentMessage>,
        vulnerability_db: Arc<VulnerabilityDatabase>,
    ) -> Result<Self, AnalysisError> {
        Ok(Self {
            config,
            message_tx,
            vulnerability_db,
        })
    }
}

#[async_trait]
impl SecurityAgent for DependencyAgent {
    async fn execute_task(&self, task_id: String, _context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("DependencyAgent executing task: {}", task_id);
        
        // TODO: Implement dependency vulnerability analysis
        let issues = Vec::new();
        
        Ok(AgentResult::DependencyAnalysis(issues))
    }

    fn get_agent_id(&self) -> &str {
        &self.config.name
    }

    fn get_capabilities(&self) -> Vec<TaskType> {
        vec![TaskType::DependencyAnalysis]
    }

    fn clone_box(&self) -> Box<dyn SecurityAgent> {
        Box::new(self.clone())
    }
}

/// Validation Agent - cross-validates findings from other agents
#[derive(Clone)]
pub struct ValidationAgent {
    config: AgentConfig,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
}

impl ValidationAgent {
    pub fn new(
        config: AgentConfig,
        message_tx: mpsc::UnboundedSender<AgentMessage>,
    ) -> Result<Self, AnalysisError> {
        Ok(Self {
            config,
            message_tx,
        })
    }
}

#[async_trait]
impl SecurityAgent for ValidationAgent {
    async fn execute_task(&self, task_id: String, _context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("ValidationAgent executing task: {}", task_id);
        
        // TODO: Implement cross-validation logic
        let report = ValidationReport {
            validated_issues: Vec::new(),
            false_positives: Vec::new(),
            confidence_adjustments: HashMap::new(),
            cross_validation_score: 0.8,
        };
        
        Ok(AgentResult::ValidationResult(report))
    }

    fn get_agent_id(&self) -> &str {
        &self.config.name
    }

    fn get_capabilities(&self) -> Vec<TaskType> {
        vec![TaskType::ValidationAnalysis]
    }

    fn clone_box(&self) -> Box<dyn SecurityAgent> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::core::{SecurityContext, VulnerabilityDatabase};
    use crate::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = MultiAgentConfig::development();
        let knowledge_graph = Arc::new(SecurityKnowledgeGraph::new().unwrap());
        let vulnerability_db = Arc::new(VulnerabilityDatabase::new().unwrap());

        let orchestrator = SecurityOrchestrator::new(config, knowledge_graph, vulnerability_db);
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_agent_message_serialization() {
        let message = AgentMessage::TaskAssignment {
            task_id: "test-task".to_string(),
            agent_id: "test-agent".to_string(),
            task_type: TaskType::TaintAnalysis,
            context: SecurityContext {
                file_path: PathBuf::from("test.rs"),
                content: "test content".to_string(),
                language: crate::ast::SourceLanguage::Rust,
                metadata: HashMap::new(),
            },
            deadline: None,
        };

        let serialized = serde_json::to_string(&message);
        assert!(serialized.is_ok());

        let deserialized: AgentMessage = serde_json::from_str(&serialized.unwrap()).unwrap();
        match deserialized {
            AgentMessage::TaskAssignment { task_id, .. } => {
                assert_eq!(task_id, "test-task");
            }
            _ => panic!("Wrong message type after deserialization"),
        }
    }

    #[tokio::test]
    async fn test_agent_capabilities() {
        let config = AgentConfig::new("TestAgent".to_string());
        let (tx, _rx) = mpsc::unbounded_channel();
        let knowledge_graph = Arc::new(SecurityKnowledgeGraph::new().unwrap());

        let agent = TaintAnalysisAgent::new(config, tx, knowledge_graph).unwrap();
        
        assert_eq!(agent.get_agent_id(), "TestAgent");
        assert_eq!(agent.get_capabilities(), vec![TaskType::TaintAnalysis]);
    }
}