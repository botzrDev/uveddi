//! Unit tests for multi-agent security system

use uveddi::analysis::detectors::security::agents::*;
use uveddi::analysis::detectors::security::types::*;
use uveddi::analysis::detectors::security::config::*;
use uveddi::analysis::detectors::security::core::*;
use uveddi::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[test]
fn test_agent_message_creation() {
    let message = AgentMessage {
        id: "test-123".to_string(),
        agent_type: AgentType::TaintAnalysis,
        task_type: TaskType::AnalyzeFile,
        payload: serde_json::json!({"file_path": "/test/file.rs"}),
        priority: MessagePriority::High,
        correlation_id: Some("corr-456".to_string()),
        timeout_ms: Some(30000),
    };
    
    assert_eq!(message.id, "test-123");
    assert_eq!(message.agent_type, AgentType::TaintAnalysis);
    assert_eq!(message.task_type, TaskType::AnalyzeFile);
    assert_eq!(message.priority, MessagePriority::High);
    assert_eq!(message.correlation_id, Some("corr-456".to_string()));
    assert_eq!(message.timeout_ms, Some(30000));
}

#[test]
fn test_agent_response_creation() {
    let response = AgentResponse {
        message_id: "test-123".to_string(),
        agent_id: "taint-agent-1".to_string(),
        status: AgentStatus::Completed,
        result: Some(serde_json::json!({"issues_found": 3})),
        error: None,
        execution_time_ms: 1500,
        metadata: std::collections::HashMap::new(),
    };
    
    assert_eq!(response.message_id, "test-123");
    assert_eq!(response.agent_id, "taint-agent-1");
    assert_eq!(response.status, AgentStatus::Completed);
    assert_eq!(response.execution_time_ms, 1500);
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_security_orchestrator_creation() {
    let config = MultiAgentConfig::production();
    let knowledge_graph = Arc::new(
        uveddi::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph::new()
    );
    let vuln_db = Arc::new(MockVulnerabilityDatabase::new());
    
    let orchestrator = SecurityOrchestrator::new(config, knowledge_graph, vuln_db);
    
    assert_eq!(orchestrator.agent_pool.len(), 0); // No agents created yet
    assert!(orchestrator.message_queue_rx.is_some());
}

#[tokio::test]
async fn test_taint_analysis_agent_creation() {
    let config = TaintAnalysisConfig::default();
    let agent = TaintAnalysisAgent::new("taint-1".to_string(), config);
    
    assert_eq!(agent.id, "taint-1");
    assert_eq!(agent.agent_type, AgentType::TaintAnalysis);
    assert!(!agent.is_busy);
    assert!(agent.last_activity.is_some());
}

#[tokio::test]
async fn test_config_analysis_agent_creation() {
    let patterns = std::collections::HashMap::new();
    let agent = ConfigAnalysisAgent::new("config-1".to_string(), patterns);
    
    assert_eq!(agent.id, "config-1");
    assert_eq!(agent.agent_type, AgentType::ConfigAnalysis);
    assert!(!agent.is_busy);
}

#[tokio::test]
async fn test_dependency_agent_creation() {
    let vuln_db = Arc::new(MockVulnerabilityDatabase::new());
    let agent = DependencyAgent::new("dep-1".to_string(), vuln_db);
    
    assert_eq!(agent.id, "dep-1");
    assert_eq!(agent.agent_type, AgentType::DependencyAnalysis);
    assert!(!agent.is_busy);
}

#[tokio::test]
async fn test_validation_agent_creation() {
    let config = FalsePositiveConfig::balanced();
    let agent = ValidationAgent::new("val-1".to_string(), config);
    
    assert_eq!(agent.id, "val-1");
    assert_eq!(agent.agent_type, AgentType::Validation);
    assert!(!agent.is_busy);
}

#[test]
fn test_agent_type_variants() {
    let types = vec![
        AgentType::Orchestrator,
        AgentType::TaintAnalysis,
        AgentType::ConfigAnalysis,
        AgentType::DependencyAnalysis,
        AgentType::Validation,
    ];
    
    for agent_type in types {
        // Test that all variants can be created and compared
        match agent_type {
            AgentType::Orchestrator => assert_eq!(agent_type, AgentType::Orchestrator),
            AgentType::TaintAnalysis => assert_eq!(agent_type, AgentType::TaintAnalysis),
            AgentType::ConfigAnalysis => assert_eq!(agent_type, AgentType::ConfigAnalysis),
            AgentType::DependencyAnalysis => assert_eq!(agent_type, AgentType::DependencyAnalysis),
            AgentType::Validation => assert_eq!(agent_type, AgentType::Validation),
        }
    }
}

#[test]
fn test_task_type_variants() {
    let tasks = vec![
        TaskType::AnalyzeFile,
        TaskType::ValidateResults,
        TaskType::CorrelateFindings,
        TaskType::UpdateKnowledgeGraph,
        TaskType::CheckDependencies,
    ];
    
    for task_type in tasks {
        match task_type {
            TaskType::AnalyzeFile => assert_eq!(task_type, TaskType::AnalyzeFile),
            TaskType::ValidateResults => assert_eq!(task_type, TaskType::ValidateResults),
            TaskType::CorrelateFindings => assert_eq!(task_type, TaskType::CorrelateFindings),
            TaskType::UpdateKnowledgeGraph => assert_eq!(task_type, TaskType::UpdateKnowledgeGraph),
            TaskType::CheckDependencies => assert_eq!(task_type, TaskType::CheckDependencies),
        }
    }
}

#[test]
fn test_message_priority_ordering() {
    assert!(MessagePriority::Critical > MessagePriority::High);
    assert!(MessagePriority::High > MessagePriority::Normal);
    assert!(MessagePriority::Normal > MessagePriority::Low);
}

#[test]
fn test_agent_status_variants() {
    let statuses = vec![
        AgentStatus::Idle,
        AgentStatus::Processing,
        AgentStatus::Completed,
        AgentStatus::Failed,
        AgentStatus::Timeout,
    ];
    
    for status in statuses {
        match status {
            AgentStatus::Idle => assert_eq!(status, AgentStatus::Idle),
            AgentStatus::Processing => assert_eq!(status, AgentStatus::Processing),
            AgentStatus::Completed => assert_eq!(status, AgentStatus::Completed),
            AgentStatus::Failed => assert_eq!(status, AgentStatus::Failed),
            AgentStatus::Timeout => assert_eq!(status, AgentStatus::Timeout),
        }
    }
}

#[tokio::test]
async fn test_agent_pool_management() {
    let config = MultiAgentConfig::production();
    let knowledge_graph = Arc::new(
        uveddi::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph::new()
    );
    let vuln_db = Arc::new(MockVulnerabilityDatabase::new());
    
    let mut orchestrator = SecurityOrchestrator::new(config.clone(), knowledge_graph, vuln_db);
    
    // Initialize agent pool
    orchestrator.initialize_agents().await;
    
    // Should have created agents based on config
    if config.enable_taint_agent {
        assert!(orchestrator.agent_pool.iter().any(|agent| 
            matches!(agent.agent_type, AgentType::TaintAnalysis)));
    }
    
    if config.enable_config_agent {
        assert!(orchestrator.agent_pool.iter().any(|agent| 
            matches!(agent.agent_type, AgentType::ConfigAnalysis)));
    }
    
    if config.enable_dependency_agent {
        assert!(orchestrator.agent_pool.iter().any(|agent| 
            matches!(agent.agent_type, AgentType::DependencyAnalysis)));
    }
    
    if config.enable_validation_agent {
        assert!(orchestrator.agent_pool.iter().any(|agent| 
            matches!(agent.agent_type, AgentType::Validation)));
    }
}

#[tokio::test]
async fn test_message_serialization() {
    let message = AgentMessage {
        id: "test-123".to_string(),
        agent_type: AgentType::TaintAnalysis,
        task_type: TaskType::AnalyzeFile,
        payload: serde_json::json!({"file_path": "/test/file.rs"}),
        priority: MessagePriority::High,
        correlation_id: Some("corr-456".to_string()),
        timeout_ms: Some(30000),
    };
    
    // Test serialization to JSON
    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("test-123"));
    assert!(json.contains("TaintAnalysis"));
    
    // Test deserialization from JSON
    let deserialized: AgentMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, message.id);
    assert_eq!(deserialized.agent_type, message.agent_type);
}

#[tokio::test]
async fn test_agent_response_serialization() {
    let response = AgentResponse {
        message_id: "test-123".to_string(),
        agent_id: "taint-agent-1".to_string(),
        status: AgentStatus::Completed,
        result: Some(serde_json::json!({"issues_found": 3})),
        error: None,
        execution_time_ms: 1500,
        metadata: std::collections::HashMap::new(),
    };
    
    let json = serde_json::to_string(&response).unwrap();
    let deserialized: AgentResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.message_id, response.message_id);
    assert_eq!(deserialized.agent_id, response.agent_id);
    assert_eq!(deserialized.status, response.status);
    assert_eq!(deserialized.execution_time_ms, response.execution_time_ms);
}

#[tokio::test]
async fn test_agent_communication_protocol() {
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentMessage>();
    
    // Send a test message
    let message = AgentMessage {
        id: "test-msg".to_string(),
        agent_type: AgentType::TaintAnalysis,
        task_type: TaskType::AnalyzeFile,
        payload: serde_json::json!({"test": "data"}),
        priority: MessagePriority::Normal,
        correlation_id: None,
        timeout_ms: None,
    };
    
    tx.send(message.clone()).unwrap();
    
    // Receive and verify the message
    let received = rx.recv().await.unwrap();
    assert_eq!(received.id, message.id);
    assert_eq!(received.agent_type, message.agent_type);
}

#[test]
fn test_multi_agent_config_validation() {
    let config = MultiAgentConfig::production();
    
    assert!(config.max_concurrent_agents > 0);
    assert!(config.agent_timeout_seconds > 0);
    assert!(config.message_queue_size > 0);
    assert!(config.retry_attempts > 0);
    
    // At least orchestrator should be enabled
    assert!(config.enable_orchestrator);
}

// Mock implementation for testing
struct MockVulnerabilityDatabase;

impl MockVulnerabilityDatabase {
    fn new() -> Self {
        Self
    }
}

impl VulnerabilityDatabase for MockVulnerabilityDatabase {
    async fn check_dependency(&self, _name: &str, _version: &str) -> Result<Vec<VulnerabilityMetadata>, crate::analysis::AnalysisError> {
        Ok(Vec::new())
    }
    
    async fn update_database(&self) -> Result<(), crate::analysis::AnalysisError> {
        Ok(())
    }
    
    async fn get_vulnerability_details(&self, _cve_id: &str) -> Result<Option<VulnerabilityMetadata>, crate::analysis::AnalysisError> {
        Ok(None)
    }
}