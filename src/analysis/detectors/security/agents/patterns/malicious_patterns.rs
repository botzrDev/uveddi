//! Malicious pattern definitions and detection logic

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

/// Malicious pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousPattern {
    pub name: String,
    pub description: String,
    pub pattern_type: MaliciousPatternType,
    pub signatures: Vec<String>,
    pub indicators: Vec<String>,
    pub severity: SecuritySeverity,
    pub confidence_base: f64,
    pub threat_level: ThreatLevel,
}

/// Types of malicious patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaliciousPatternType {
    Backdoor,
    Keylogger,
    DataExfiltration,
    RemoteAccess,
    Persistence,
    Evasion,
    Rootkit,
    Botnet,
}

/// Threat levels for malicious patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Database of malicious patterns
pub struct MaliciousPatternDatabase {
    patterns: Vec<MaliciousPattern>,
    config: PatternConfig,
}

impl MaliciousPatternDatabase {
    pub fn new(config: PatternConfig) -> Self {
        let patterns = Self::load_default_patterns();
        Self { patterns, config }
    }

    /// Load default malicious patterns
    fn load_default_patterns() -> Vec<MaliciousPattern> {
        vec![
            MaliciousPattern {
                name: "backdoor_signature".to_string(),
                description: "Known backdoor signatures and patterns".to_string(),
                pattern_type: MaliciousPatternType::Backdoor,
                signatures: vec![
                    "backdoor".to_string(),
                    "remote_shell".to_string(),
                    "reverse_shell".to_string(),
                    "cmd_shell".to_string(),
                    "hidden_access".to_string(),
                ],
                indicators: vec![
                    "bind_shell".to_string(),
                    "connect_back".to_string(),
                    "shell_code".to_string(),
                ],
                severity: SecuritySeverity::Critical,
                confidence_base: 0.9,
                threat_level: ThreatLevel::Critical,
            },
            MaliciousPattern {
                name: "keylogger_pattern".to_string(),
                description: "Keylogging and input capture patterns".to_string(),
                pattern_type: MaliciousPatternType::Keylogger,
                signatures: vec![
                    "keylog".to_string(),
                    "keystroke".to_string(),
                    "input_capture".to_string(),
                    "keyboard_hook".to_string(),
                ],
                indicators: vec![
                    "key_press".to_string(),
                    "input_monitor".to_string(),
                    "keyboard_spy".to_string(),
                ],
                severity: SecuritySeverity::High,
                confidence_base: 0.85,
                threat_level: ThreatLevel::High,
            },
            MaliciousPattern {
                name: "data_exfiltration".to_string(),
                description: "Data theft and exfiltration patterns".to_string(),
                pattern_type: MaliciousPatternType::DataExfiltration,
                signatures: vec![
                    "exfiltrate".to_string(),
                    "data_theft".to_string(),
                    "steal_data".to_string(),
                    "upload_stolen".to_string(),
                ],
                indicators: vec![
                    "collect_files".to_string(),
                    "gather_info".to_string(),
                    "transmit_data".to_string(),
                ],
                severity: SecuritySeverity::Critical,
                confidence_base: 0.8,
                threat_level: ThreatLevel::Critical,
            },
            MaliciousPattern {
                name: "remote_access_tool".to_string(),
                description: "Remote access trojan (RAT) patterns".to_string(),
                pattern_type: MaliciousPatternType::RemoteAccess,
                signatures: vec![
                    "remote_access".to_string(),
                    "rat_client".to_string(),
                    "remote_control".to_string(),
                    "admin_panel".to_string(),
                ],
                indicators: vec![
                    "remote_desktop".to_string(),
                    "vnc_server".to_string(),
                    "remote_cmd".to_string(),
                ],
                severity: SecuritySeverity::Critical,
                confidence_base: 0.9,
                threat_level: ThreatLevel::Critical,
            },
            MaliciousPattern {
                name: "persistence_mechanism".to_string(),
                description: "System persistence and startup mechanisms".to_string(),
                pattern_type: MaliciousPatternType::Persistence,
                signatures: vec![
                    "auto_start".to_string(),
                    "registry_persist".to_string(),
                    "startup_folder".to_string(),
                    "service_install".to_string(),
                ],
                indicators: vec![
                    "cron_job".to_string(),
                    "scheduled_task".to_string(),
                    "systemd_service".to_string(),
                ],
                severity: SecuritySeverity::High,
                confidence_base: 0.75,
                threat_level: ThreatLevel::High,
            },
            MaliciousPattern {
                name: "evasion_technique".to_string(),
                description: "Anti-analysis and evasion techniques".to_string(),
                pattern_type: MaliciousPatternType::Evasion,
                signatures: vec![
                    "anti_debug".to_string(),
                    "vm_detect".to_string(),
                    "sandbox_evasion".to_string(),
                    "obfuscation".to_string(),
                ],
                indicators: vec![
                    "sleep_evasion".to_string(),
                    "timing_check".to_string(),
                    "env_check".to_string(),
                ],
                severity: SecuritySeverity::High,
                confidence_base: 0.8,
                threat_level: ThreatLevel::High,
            },
        ]
    }

    /// Match malicious patterns in context
    pub async fn match_malicious_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for pattern in &self.patterns {
            let matches = self.find_malicious_matches(pattern, context).await?;
            for pattern_match in matches {
                let issue = self.create_issue_from_match(pattern, &pattern_match, context);
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    async fn find_malicious_matches(
        &self,
        pattern: &MaliciousPattern,
        context: &SecurityContext,
    ) -> Result<Vec<PatternMatch>, AnalysisError> {
        let mut matches = Vec::new();
        let content = &context.content;

        // Search for signature matches
        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            // Check signatures (high confidence)
            for signature in &pattern.signatures {
                if line_lower.contains(&signature.to_lowercase()) {
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

            // Check indicators (lower confidence)
            for indicator in &pattern.indicators {
                if line_lower.contains(&indicator.to_lowercase()) {
                    let pattern_match = PatternMatch {
                        pattern_name: pattern.name.clone(),
                        pattern_type: format!("{:?}", pattern.pattern_type),
                        confidence_score: pattern.confidence_base * 0.7, // Reduce confidence for indicators
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

        Ok(matches)
    }

    fn create_issue_from_match(
        &self,
        pattern: &MaliciousPattern,
        pattern_match: &PatternMatch,
        context: &SecurityContext,
    ) -> SecurityIssue {
        SecurityIssue {
            issue_type: SecurityIssueType::PotentialMaliciousAgent,
            title: format!("Malicious Pattern Detected: {}", pattern.name),
            description: format!(
                "{} (Threat Level: {:?})",
                pattern.description, pattern.threat_level
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
                "Immediately investigate and remove suspected {} code",
                pattern.pattern_type.to_string().to_lowercase()
            )),
            metadata: HashMap::new(),
            correlation_id: None,
        }
    }

    pub fn add_pattern(&mut self, pattern: MaliciousPattern) {
        self.patterns.push(pattern);
    }

    pub fn get_patterns(&self) -> &[MaliciousPattern] {
        &self.patterns
    }

    pub fn get_patterns_by_type(&self, pattern_type: &MaliciousPatternType) -> Vec<&MaliciousPattern> {
        self.patterns
            .iter()
            .filter(|p| std::mem::discriminant(&p.pattern_type) == std::mem::discriminant(pattern_type))
            .collect()
    }

    pub fn get_critical_patterns(&self) -> Vec<&MaliciousPattern> {
        self.patterns
            .iter()
            .filter(|p| matches!(p.threat_level, ThreatLevel::Critical))
            .collect()
    }
}

#[async_trait]
impl PatternMatcher for MaliciousPatternDatabase {
    async fn match_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        self.match_malicious_patterns(context).await
    }

    fn matcher_name(&self) -> &'static str {
        "MaliciousPatternDatabase"
    }

    fn can_match(&self, _context: &SecurityContext) -> bool {
        true // Can match all contexts
    }
}

impl ToString for MaliciousPatternType {
    fn to_string(&self) -> String {
        match self {
            MaliciousPatternType::Backdoor => "Backdoor".to_string(),
            MaliciousPatternType::Keylogger => "Keylogger".to_string(),
            MaliciousPatternType::DataExfiltration => "DataExfiltration".to_string(),
            MaliciousPatternType::RemoteAccess => "RemoteAccess".to_string(),
            MaliciousPatternType::Persistence => "Persistence".to_string(),
            MaliciousPatternType::Evasion => "Evasion".to_string(),
            MaliciousPatternType::Rootkit => "Rootkit".to_string(),
            MaliciousPatternType::Botnet => "Botnet".to_string(),
        }
    }
}