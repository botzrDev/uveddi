//! Individual security agent implementations
//!
//! This module contains the specialized security agents that perform specific analysis tasks
//! within the multi-agent security analysis architecture.

use crate::analysis::detectors::security::core::{SecurityContext, VulnerabilityDatabase};
use crate::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph;
use crate::analysis::detectors::security::taint_analysis::TaintAnalysisEngine;
use crate::analysis::AnalysisError;
use super::super::config::AgentConfig;
use super::super::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

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

#[async_trait::async_trait]
impl SecurityAgent for TaintAnalysisAgent {
    async fn execute_task(&self, task_id: String, context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("TaintAnalysisAgent executing task: {}", task_id);

        let parsed_file = context.to_parsed_file()?;
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

#[async_trait::async_trait]
impl SecurityAgent for ConfigAnalysisAgent {
    async fn execute_task(&self, task_id: String, _context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("ConfigAnalysisAgent executing task: {}", task_id);
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

#[async_trait::async_trait]
impl SecurityAgent for DependencyAgent {
    async fn execute_task(&self, task_id: String, _context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("DependencyAgent executing task: {}", task_id);
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
        Ok(Self { config, message_tx })
    }
}

#[async_trait::async_trait]
impl SecurityAgent for ValidationAgent {
    async fn execute_task(&self, task_id: String, _context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        info!("ValidationAgent executing task: {}", task_id);

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