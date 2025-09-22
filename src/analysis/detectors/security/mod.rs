//! Security vulnerability detection framework
//!
//! This module implements a comprehensive security detector for Uveddi's analysis engine
//! that leverages existing architectural intelligence capabilities and integrates advanced
//! security analysis techniques. The system is privacy-first, developer-centric, and
//! provides actionable, contextually rich insights into security vulnerabilities.
//!
//! ## Architecture Overview
//!
//! The security detector follows a multi-layered architecture combining:
//! - **Deterministic Analysis**: Tree-sitter AST parsing, taint analysis, configuration analysis
//! - **AI-Powered Enhancement**: Contextual explanation, risk assessment, intelligent remediation
//! - **Anti-Pattern Correlation**: Maps architectural issues to security vulnerabilities
//! - **Knowledge Graph Integration**: Code-centric RAG for contextual analysis
//!
//! ## Core Components
//!
//! ### Multi-Agent System (MAS)
//! - **SecurityOrchestrator**: Orchestrates security analysis tasks
//! - **TaintAnalysisAgent**: Tracks data flow from sources to sinks
//! - **ConfigAnalysisAgent**: Scans configuration files for security issues
//! - **DependencyAgent**: Performs Software Composition Analysis (SCA)
//! - **ValidationAgent**: Cross-validates findings to reduce false positives
//!
//! ### Detection Strategies
//! - **OWASP Top 10 Coverage**: Comprehensive coverage of OWASP 2021 and LLM security issues
//! - **Architectural Correlation**: Links security issues to architectural anti-patterns
//! - **Confidence Scoring**: Multi-factor probabilistic model for finding certainty
//! - **Language-Agnostic IR**: Unified analysis across Rust, Python, JavaScript/TypeScript
//!
//! ## Usage Examples
//!
//! ### Basic Security Analysis
//! ```rust
//! use uveddi::analysis::detectors::security::{SecurityDetector, SecurityConfig};
//!
//! let config = SecurityConfig::default();
//! let detector = SecurityDetector::new(config)?;
//!
//! let issues = detector.detect_issues(&parsed_file).await?;
//! for issue in issues {
//!     println!("Security Issue: {} (Confidence: {:.2})", issue.title, issue.confidence_score);
//! }
//! ```
//!
//! ### Advanced Multi-Agent Analysis
//! ```rust
//! use uveddi::analysis::detectors::security::{SecurityOrchestrator, MultiAgentConfig};
//!
//! let orchestrator = SecurityOrchestrator::new(MultiAgentConfig {
//!     enable_taint_analysis: true,
//!     enable_sca: true,
//!     enable_ai_enhancement: true,
//!     confidence_threshold: 0.7,
//! })?;
//!
//! let analysis_result = orchestrator.analyze_codebase(&dependency_graph).await?;
//! ```

pub mod agents;
pub mod config;
pub mod core;
pub mod detector;
pub mod knowledge_graph;
pub mod owasp;
pub mod strategies;
pub mod sql_injection;
pub mod taint_analysis;
pub mod types;
pub mod validation;

// Re-exports for convenience
pub use agents::{
    ConfigAnalysisAgent, DependencyAgent, SecurityOrchestrator, TaintAnalysisAgent, ValidationAgent,
};
pub use config::{
    AgentConfig, FalsePositiveConfig, LanguageConfig, MultiAgentConfig, SecurityConfig,
    TaintAnalysisConfig,
};
pub use core::{ConfidenceScore, SecurityAnalysisResult, SecurityContext, VulnerabilityDatabase};
pub use detector::SecurityDetector;
pub use knowledge_graph::{KnowledgeGraphBuilder, SecurityKnowledgeGraph, StructuralSemanticGraph};
pub use owasp::{OwaspCategory, OwaspConfig, OwaspDetector, OwaspVulnerability};
pub use sql_injection::{SqlInjectionConfig, SqlInjectionDetector};
pub use strategies::{
    ConfigFileAnalyzer, DeterministicPatternMatcher, SoftwareCompositionAnalyzer,
    VulnerabilityCorrelationEngine,
};
pub use taint_analysis::{
    DataFlowGraph, SanitizationPoint, TaintAnalysisEngine, TaintFlow, TaintLevel, TaintSink,
    TaintSource, UnifiedSinkDetector, UnifiedSourceDetector,
};
pub use types::{
    SecurityIssue, SecurityIssueType, SecuritySeverity, VulnerabilityMetadata, VulnerabilityType,
};
pub use validation::{
    BayesianOptimizer, ConfidenceCalculator, FalsePositiveMitigator, ValidationEngine,
};

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Main security detector that integrates all security analysis components
///
/// The `SecurityDetector` serves as the primary entry point for security analysis
/// in the Uveddi system. It orchestrates multiple detection strategies and agents
/// to provide comprehensive security vulnerability detection with high precision
/// and minimal false positives.
///
/// ## Features
///
/// - **Multi-layered Detection**: Combines deterministic and AI-powered analysis
/// - **OWASP Top 10 Coverage**: Comprehensive security vulnerability detection
/// - **Architectural Correlation**: Links security issues to design patterns
/// - **Language Support**: Rust, Python, JavaScript, TypeScript analysis
/// - **Confidence Scoring**: Probabilistic confidence in findings
/// - **False Positive Mitigation**: Advanced filtering and validation
///
/// ## Architecture
///
/// The detector uses a multi-agent system where specialized agents collaborate
/// through a central knowledge graph to provide comprehensive analysis:
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                    Security Detector                           │
/// ├─────────────────────────────────────────────────────────────────┤
/// │  ┌───────────────┐    ┌──────────────────┐    ┌──────────────┐  │
/// │  │ Taint Analysis│    │  Config Analysis │    │ SCA Analysis │  │
/// │  │    Agent      │    │      Agent       │    │    Agent     │  │
/// │  └───────────────┘    └──────────────────┘    └──────────────┘  │
/// │           │                     │                       │       │
/// │           └─────────────────────┼───────────────────────┘       │
/// │                                 │                               │
/// │  ┌─────────────────────────────────────────────────────────────┐ │
/// │  │            Knowledge Graph (Code-Centric RAG)              │ │
/// │  └─────────────────────────────────────────────────────────────┘ │
/// │                                 │                               │
/// │  ┌───────────────┐    ┌──────────────────┐    ┌──────────────┐  │
/// │  │   Validation  │    │   AI Enhancement │    │ Architectural│  │
/// │  │    Engine     │    │      Layer       │    │ Correlation  │  │
/// │  └───────────────┘    └──────────────────┘    └──────────────┘  │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
pub struct MainSecurityDetector {
    orchestrator: SecurityOrchestrator,
    config: SecurityConfig,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
    vulnerability_db: Arc<VulnerabilityDatabase>,
}

impl MainSecurityDetector {
    /// Create a new security detector with default configuration
    pub fn new() -> Result<Self, AnalysisError> {
        Self::with_config(SecurityConfig::default())
    }

    /// Create a new security detector with custom configuration
    pub fn with_config(config: SecurityConfig) -> Result<Self, AnalysisError> {
        info!("Initializing Security Detector with config: {:?}", config);

        let knowledge_graph = Arc::new(SecurityKnowledgeGraph::new()?);
        let vulnerability_db = Arc::new(VulnerabilityDatabase::new()?);

        let orchestrator = SecurityOrchestrator::new(
            config.multi_agent.clone(),
            knowledge_graph.clone(),
            vulnerability_db.clone(),
        )?;

        Ok(Self {
            orchestrator,
            config,
            knowledge_graph,
            vulnerability_db,
        })
    }

    /// Analyze a single file for security vulnerabilities
    async fn analyze_file(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!(
            "Analyzing file for security issues: {}",
            file.file_path.display()
        );

        // Create security context from file
        let context = SecurityContext::from_parsed_file(file)?;

        // Run multi-agent analysis
        let analysis_result = self.orchestrator.analyze_file(&context).await?;

        // Convert to security issues with confidence scoring
        let mut issues = Vec::new();
        for vulnerability in analysis_result.vulnerabilities {
            let security_issue =
                SecurityIssue::from_vulnerability(vulnerability, &file.file_path, file.language)?;
            issues.push(security_issue);
        }

        debug!(
            "Found {} security issues in {}",
            issues.len(),
            file.file_path.display()
        );
        Ok(issues)
    }

    /// Convert security issues to architectural issues for Uveddi compatibility
    fn convert_to_architectural_issues(
        &self,
        security_issues: Vec<SecurityIssue>,
        file_path: &std::path::Path,
    ) -> Vec<ArchitecturalIssue> {
        security_issues
            .into_iter()
            .map(|issue| ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0,        // Will be set by the analysis engine
                anti_pattern_type_id: 100, // Default security pattern type ID
                file_path: file_path.to_string_lossy().to_string(),
                start_line: Some(issue.location.start_line),
                end_line: Some(issue.location.end_line),
                line_number: Some(issue.location.start_line),
                column_number: issue.location.start_column,
                message: issue.title,
                metadata: serde_json::to_string(&issue.metadata).unwrap_or_default(),
                detector_name: "SecurityDetector".to_string(),
                created_at: chrono::Utc::now(),
                severity: issue.severity.to_string(),
                description: issue.description,
                code_snippet: None,
                ai_explanation: issue.remediation,
            })
            .collect()
    }
}

#[async_trait]
impl AnalysisDetector for MainSecurityDetector {
    async fn detect_issues(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        info!("Running security analysis on: {}", file.file_path.display());

        match self.analyze_file(file).await {
            Ok(security_issues) => {
                let architectural_issues =
                    self.convert_to_architectural_issues(security_issues, &file.file_path);

                info!(
                    "Security analysis completed for {}: {} issues found",
                    file.file_path.display(),
                    architectural_issues.len()
                );

                Ok(architectural_issues)
            }
            Err(e) => {
                error!(
                    "Security analysis failed for {}: {}",
                    file.file_path.display(),
                    e
                );
                Err(e)
            }
        }
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![
            // OWASP Top 10 2021
            AntiPatternType {
                anti_pattern_type_id: Some(100),
                name: "Broken Access Control".to_string(),
                description: "Access control enforces policy such that users cannot act outside of their intended permissions".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(101),
                name: "Cryptographic Failures".to_string(),
                description: "Failures related to cryptography which often leads to sensitive data exposure".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(102),
                name: "Injection".to_string(),
                description: "Application is vulnerable to injection attacks like SQL, NoSQL, OS injection".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(103),
                name: "Insecure Design".to_string(),
                description: "Risks related to design and architectural flaws".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(104),
                name: "Security Misconfiguration".to_string(),
                description: "Security misconfiguration is the most commonly seen issue".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(105),
                name: "Vulnerable Components".to_string(),
                description: "Components with known vulnerabilities".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(106),
                name: "Authentication Failures".to_string(),
                description: "Application functions related to authentication and session management".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(107),
                name: "Software Data Integrity Failures".to_string(),
                description: "Code and infrastructure that does not protect against integrity violations".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(108),
                name: "Security Logging Failures".to_string(),
                description: "Insufficient logging and monitoring".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(109),
                name: "Server Side Request Forgery".to_string(),
                description: "SSRF flaws occur whenever a web application is fetching a remote resource without validating the user-supplied URL".to_string(),
                category: "Security".to_string(),
            },
            // Additional security patterns
            AntiPatternType {
                anti_pattern_type_id: Some(110),
                name: "Hardcoded Secrets".to_string(),
                description: "Credentials, API keys, or secrets hardcoded in source code".to_string(),
                category: "Security".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(111),
                name: "Insecure Dependencies".to_string(),
                description: "Use of outdated or vulnerable third-party dependencies".to_string(),
                category: "Security".to_string(),
            },
        ]
    }

    fn get_detector_name(&self) -> &'static str {
        "SecurityDetector"
    }
}

impl Default for MainSecurityDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create default SecurityDetector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ParsedFile;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_security_detector_creation() {
        let detector = MainSecurityDetector::new();
        assert!(detector.is_ok());
    }

    #[tokio::test]
    async fn test_security_detector_with_custom_config() {
        let config = SecurityConfig {
            enable_taint_analysis: true,
            enable_sca: true,
            confidence_threshold: 0.8,
            ..Default::default()
        };

        let detector = MainSecurityDetector::with_config(config);
        assert!(detector.is_ok());
    }

    #[tokio::test]
    async fn test_get_anti_pattern_types() {
        let detector = MainSecurityDetector::new().unwrap();
        let types = detector.get_anti_pattern_types();

        assert!(!types.is_empty());
        assert!(types.iter().any(|t| t.name == "Injection"));
        assert!(types.iter().any(|t| t.name == "Broken Access Control"));
    }

    #[tokio::test]
    async fn test_detector_name() {
        let detector = MainSecurityDetector::new().unwrap();
        assert_eq!(detector.get_detector_name(), "SecurityDetector");
    }
}
