//! Agent detection module for identifying agent-like patterns in code

use super::{AnalysisConfig, AnalysisContext, AnalysisModule};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

/// Agent detector for identifying agent-like behavioral patterns
pub struct AgentDetector {
    config: AnalysisConfig,
}

impl AgentDetector {
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }

    /// Detect agent patterns in the given context
    pub async fn detect_agents(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Analyze for autonomous behavior patterns
        issues.extend(self.analyze_autonomous_behavior(context).await?);

        // Analyze for communication patterns
        issues.extend(self.analyze_communication_patterns(context).await?);

        // Analyze for decision-making patterns
        issues.extend(self.analyze_decision_patterns(context).await?);

        // Analyze for coordination patterns
        issues.extend(self.analyze_coordination_patterns(context).await?);

        Ok(issues)
    }

    async fn analyze_autonomous_behavior(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for autonomous execution patterns
        let autonomous_patterns = [
            "spawn",
            "thread::spawn",
            "tokio::spawn",
            "async fn",
            "background_task",
            "daemon",
            "service",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &autonomous_patterns {
                if line.contains(pattern) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Autonomous Execution Pattern Detected".to_string(),
                        format!(
                            "Detected autonomous execution pattern '{}' which may indicate agent-like behavior",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.6)
                    .with_remediation("Review autonomous execution patterns for legitimate use".to_string())
                    .with_detector("AgentDetector".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_communication_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for inter-agent communication patterns
        let comm_patterns = [
            "send(",
            "recv(",
            "channel(",
            "mpsc::",
            "broadcast::",
            "message_queue",
            "event_bus",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &comm_patterns {
                if line.contains(pattern) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Inter-Agent Communication Pattern".to_string(),
                        format!(
                            "Detected communication pattern '{}' that may facilitate agent coordination",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Low)
                    .with_confidence(0.5)
                    .with_remediation("Ensure communication channels are properly secured".to_string())
                    .with_detector("AgentDetector".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_decision_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for decision-making patterns
        let decision_patterns = [
            "if.*random",
            "match.*strategy",
            "decide",
            "choose",
            "select_action",
            "policy",
            "strategy_pattern",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &decision_patterns {
                if line.contains(pattern) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Decision-Making Pattern Detected".to_string(),
                        format!(
                            "Detected decision-making pattern '{}' that may indicate autonomous behavior",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Low)
                    .with_confidence(0.4)
                    .with_remediation("Review decision logic for potential security implications".to_string())
                    .with_detector("AgentDetector".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_coordination_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for coordination patterns
        let coordination_patterns = [
            "coordinator",
            "orchestrator",
            "leader_election",
            "consensus",
            "distributed",
            "cluster",
            "swarm",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &coordination_patterns {
                if line.contains(pattern) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Coordination Pattern Detected".to_string(),
                        format!(
                            "Detected coordination pattern '{}' that may indicate multi-agent system",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.7)
                    .with_remediation("Verify coordination mechanisms are legitimate and secure".to_string())
                    .with_detector("AgentDetector".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }
}

#[async_trait]
impl AnalysisModule for AgentDetector {
    async fn analyze(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!(
            "Starting agent detection analysis for {:?}",
            context.file_path
        );
        self.detect_agents(context).await
    }

    fn module_name(&self) -> &'static str {
        "AgentDetector"
    }

    fn can_analyze(&self, context: &SecurityContext) -> bool {
        matches!(
            context.language,
            SourceLanguage::Rust
                | SourceLanguage::Python
                | SourceLanguage::JavaScript
                | SourceLanguage::TypeScript
        )
    }
}
