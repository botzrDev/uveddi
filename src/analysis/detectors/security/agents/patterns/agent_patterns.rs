//! Agent pattern definitions and matching logic

use super::{PatternConfig, PatternMatch, PatternMatcher};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPattern {
    pub name: String,
    pub description: String,
    pub pattern_type: AgentPatternType,
    pub keywords: Vec<String>,
    pub regex_patterns: Vec<String>,
    pub severity: SecuritySeverity,
    pub confidence_base: f64,
    pub language_specific: Option<SourceLanguage>,
}

/// Types of agent patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentPatternType {
    Autonomous,
    Communication,
    Coordination,
    DecisionMaking,
    Learning,
    Adaptation,
    MultiAgent,
}

/// Database of agent patterns
pub struct AgentPatternDatabase {
    patterns: Vec<AgentPattern>,
    config: PatternConfig,
}

impl AgentPatternDatabase {
    pub fn new(config: PatternConfig) -> Self {
        let patterns = Self::load_default_patterns();
        Self { patterns, config }
    }

    /// Load default agent patterns
    fn load_default_patterns() -> Vec<AgentPattern> {
        vec![
            AgentPattern {
                name: "autonomous_execution".to_string(),
                description: "Patterns indicating autonomous code execution".to_string(),
                pattern_type: AgentPatternType::Autonomous,
                keywords: vec![
                    "autonomous".to_string(),
                    "self_execute".to_string(),
                    "auto_run".to_string(),
                    "background_task".to_string(),
                    "daemon_process".to_string(),
                ],
                regex_patterns: vec![
                    r"(?i)autonomous.*exec".to_string(),
                    r"(?i)self.*running".to_string(),
                    r"(?i)auto.*start".to_string(),
                ],
                severity: SecuritySeverity::Medium,
                confidence_base: 0.7,
                language_specific: None,
            },
            AgentPattern {
                name: "inter_agent_communication".to_string(),
                description: "Patterns for communication between agents".to_string(),
                pattern_type: AgentPatternType::Communication,
                keywords: vec![
                    "message_passing".to_string(),
                    "agent_comm".to_string(),
                    "inter_agent".to_string(),
                    "agent_channel".to_string(),
                    "agent_protocol".to_string(),
                ],
                regex_patterns: vec![
                    r"(?i)agent.*comm".to_string(),
                    r"(?i)message.*agent".to_string(),
                    r"(?i)agent.*channel".to_string(),
                ],
                severity: SecuritySeverity::Medium,
                confidence_base: 0.6,
                language_specific: None,
            },
            AgentPattern {
                name: "coordination_pattern".to_string(),
                description: "Patterns for agent coordination and orchestration".to_string(),
                pattern_type: AgentPatternType::Coordination,
                keywords: vec![
                    "coordinator".to_string(),
                    "orchestrator".to_string(),
                    "leader_election".to_string(),
                    "consensus".to_string(),
                    "distributed_system".to_string(),
                ],
                regex_patterns: vec![
                    r"(?i)coord.*agent".to_string(),
                    r"(?i)agent.*orchestr".to_string(),
                    r"(?i)leader.*elect".to_string(),
                ],
                severity: SecuritySeverity::High,
                confidence_base: 0.8,
                language_specific: None,
            },
            AgentPattern {
                name: "decision_making".to_string(),
                description: "Patterns for autonomous decision making".to_string(),
                pattern_type: AgentPatternType::DecisionMaking,
                keywords: vec![
                    "decision_tree".to_string(),
                    "strategy_pattern".to_string(),
                    "policy_engine".to_string(),
                    "rule_engine".to_string(),
                    "inference_engine".to_string(),
                ],
                regex_patterns: vec![
                    r"(?i)decision.*engine".to_string(),
                    r"(?i)policy.*eval".to_string(),
                    r"(?i)rule.*apply".to_string(),
                ],
                severity: SecuritySeverity::Medium,
                confidence_base: 0.6,
                language_specific: None,
            },
            AgentPattern {
                name: "learning_adaptation".to_string(),
                description: "Patterns for learning and adaptation behavior".to_string(),
                pattern_type: AgentPatternType::Learning,
                keywords: vec![
                    "machine_learning".to_string(),
                    "neural_network".to_string(),
                    "reinforcement_learning".to_string(),
                    "adaptive_behavior".to_string(),
                    "self_improving".to_string(),
                ],
                regex_patterns: vec![
                    r"(?i)learn.*adapt".to_string(),
                    r"(?i)neural.*net".to_string(),
                    r"(?i)reinforce.*learn".to_string(),
                ],
                severity: SecuritySeverity::High,
                confidence_base: 0.7,
                language_specific: None,
            },
        ]
    }

    /// Match agent patterns in context
    pub async fn match_agent_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for pattern in &self.patterns {
            if let Some(language) = &pattern.language_specific {
                if context.language != *language {
                    continue;
                }
            }

            let matches = self.find_pattern_matches(pattern, context).await?;
            for pattern_match in matches {
                let issue = self.create_issue_from_match(pattern, &pattern_match, context);
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    async fn find_pattern_matches(
        &self,
        pattern: &AgentPattern,
        context: &SecurityContext,
    ) -> Result<Vec<PatternMatch>, AnalysisError> {
        let mut matches = Vec::new();
        let content = &context.content;

        // Search for keyword matches
        for (line_num, line) in content.lines().enumerate() {
            for keyword in &pattern.keywords {
                if line.to_lowercase().contains(&keyword.to_lowercase()) {
                    let pattern_match = PatternMatch {
                        pattern_name: pattern.name.clone(),
                        pattern_type: format!("{:?}", pattern.pattern_type),
                        confidence_score: pattern.confidence_base,
                        line_number: line_num + 1,
                        column_start: 0,
                        column_end: line.len(),
                        matched_text: line.to_string(),
                    };
                    matches.push(pattern_match);
                    break; // Only one match per line per pattern
                }
            }
        }

        // TODO: Add regex pattern matching when regex crate is available

        Ok(matches)
    }

    fn create_issue_from_match(
        &self,
        pattern: &AgentPattern,
        pattern_match: &PatternMatch,
        context: &SecurityContext,
    ) -> SecurityIssue {
        SecurityIssue {
            issue_type: SecurityIssueType::PotentialMaliciousAgent,
            title: format!("Agent Pattern Detected: {}", pattern.name),
            description: format!(
                "{} (Pattern: {})",
                pattern.description, pattern_match.pattern_name
            ),
            severity: pattern.severity.clone(),
            confidence_score: pattern_match.confidence_score,
            location: SecurityLocation {
                file_path: context.file_path.clone(),
                start_line: pattern_match.line_number,
                end_line: pattern_match.line_number,
                start_column: pattern_match.column_start,
                end_column: pattern_match.column_end,
                function_name: None,
                class_name: None,
                module_name: None,
            },
            remediation: Some(format!(
                "Review {} behavior for legitimate use case",
                pattern.pattern_type.to_string().to_lowercase()
            )),
            metadata: HashMap::new(),
            correlation_id: None,
        }
    }

    pub fn add_pattern(&mut self, pattern: AgentPattern) {
        self.patterns.push(pattern);
    }

    pub fn get_patterns(&self) -> &[AgentPattern] {
        &self.patterns
    }

    pub fn get_patterns_by_type(&self, pattern_type: &AgentPatternType) -> Vec<&AgentPattern> {
        self.patterns
            .iter()
            .filter(|p| std::mem::discriminant(&p.pattern_type) == std::mem::discriminant(pattern_type))
            .collect()
    }
}

#[async_trait]
impl PatternMatcher for AgentPatternDatabase {
    async fn match_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        self.match_agent_patterns(context).await
    }

    fn matcher_name(&self) -> &'static str {
        "AgentPatternDatabase"
    }

    fn can_match(&self, _context: &SecurityContext) -> bool {
        true // Can match all contexts
    }
}

impl ToString for AgentPatternType {
    fn to_string(&self) -> String {
        match self {
            AgentPatternType::Autonomous => "Autonomous".to_string(),
            AgentPatternType::Communication => "Communication".to_string(),
            AgentPatternType::Coordination => "Coordination".to_string(),
            AgentPatternType::DecisionMaking => "DecisionMaking".to_string(),
            AgentPatternType::Learning => "Learning".to_string(),
            AgentPatternType::Adaptation => "Adaptation".to_string(),
            AgentPatternType::MultiAgent => "MultiAgent".to_string(),
        }
    }
}