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

pub mod analysis;
pub mod config;
pub mod detector;
pub mod language_support;
pub mod patterns;
pub mod types;

// Re-export public types and interfaces
pub use config::{AgentConfig, MultiAgentConfig};
pub use detector::{
    ConfigAnalysisAgent, DependencyAgent, SecurityOrchestrator, TaintAnalysisAgent, ValidationAgent,
};
pub use types::{
    AgentBehaviorPattern, AgentMessage, AgentResult, AiEnhancementReport, CorrelationReport,
    MaliciousAgentDetection, SecurityAgent, SubTask, TaskMetadata, TaskStatus, TaskType,
    ValidationReport,
};

// Re-export analysis modules
pub use analysis::{
    AgentDetector, AnalysisConfig, AnalysisModule, BehaviorAnalyzer, PatternMatcher,
};

// Re-export language support
pub use language_support::{
    get_language_analyzer, JavaScriptAgentAnalyzer, LanguageAgentAnalyzer, PythonAgentAnalyzer,
    RustAgentAnalyzer,
};

// Re-export pattern matching
pub use patterns::{
    AgentPattern, AgentPatternDatabase, MaliciousPattern, MaliciousPatternDatabase, PatternConfig,
    PatternMatch, PatternMatcher as PatternMatcherTrait,
};

use crate::analysis::detectors::security::core::{SecurityAnalysisResult, SecurityContext};
use crate::analysis::AnalysisError;

/// Main entry point for multi-agent security analysis
pub async fn analyze_with_agents(
    context: &SecurityContext,
    config: MultiAgentConfig,
) -> Result<SecurityAnalysisResult, AnalysisError> {
    // Create knowledge graph and vulnerability database
    let knowledge_graph = std::sync::Arc::new(
        crate::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph::new()?,
    );
    let vulnerability_db = std::sync::Arc::new(
        crate::analysis::detectors::security::core::VulnerabilityDatabase::new()?,
    );

    // Create orchestrator
    let orchestrator = SecurityOrchestrator::new(config, knowledge_graph, vulnerability_db)?;

    // Run analysis
    orchestrator.analyze_file(context).await
}

/// Create a development configuration for testing
pub fn development_config() -> MultiAgentConfig {
    MultiAgentConfig::development()
}

/// Create a production configuration for full analysis
pub fn production_config() -> MultiAgentConfig {
    MultiAgentConfig::production()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::core::SecurityContext;
    use crate::ast::SourceLanguage;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = MultiAgentConfig::development();
        let knowledge_graph = std::sync::Arc::new(
            crate::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph::new()
                .unwrap(),
        );
        let vulnerability_db = std::sync::Arc::new(
            crate::analysis::detectors::security::core::VulnerabilityDatabase::new().unwrap(),
        );

        let orchestrator = SecurityOrchestrator::new(config, knowledge_graph, vulnerability_db);
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_config_creation() {
        let dev_config = development_config();
        assert!(dev_config.enable_taint_agent);

        let prod_config = production_config();
        assert!(prod_config.enable_validation_agent);
    }

    #[tokio::test]
    async fn test_agent_analysis() {
        let context = SecurityContext {
            file_path: PathBuf::from("test.rs"),
            content: "fn test() { spawn_agent(); }".to_string(),
            language: SourceLanguage::Rust,
            metadata: HashMap::new(),
        };

        let config = development_config();
        let result = analyze_with_agents(&context, config).await;
        assert!(result.is_ok());
    }
}
