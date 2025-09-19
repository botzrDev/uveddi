//! Pattern matching module for detecting specific agent signatures

use super::{AnalysisConfig, AnalysisModule};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use tracing::debug;

/// Pattern matcher for detecting specific agent signatures and behaviors
pub struct PatternMatcher {
    config: AnalysisConfig,
    regex_patterns: HashMap<String, Regex>,
}

impl PatternMatcher {
    pub fn new(config: AnalysisConfig) -> Result<Self, AnalysisError> {
        let mut regex_patterns = HashMap::new();

        // Compile common patterns
        let patterns = [
            ("suspicious_network", r"(?i)(socket|connect|bind|listen|accept)\s*\("),
            ("file_operations", r"(?i)(open|read|write|delete|remove)\s*\("),
            ("process_creation", r"(?i)(spawn|exec|fork|create_process)\s*\("),
            ("crypto_operations", r"(?i)(encrypt|decrypt|hash|sign|verify)\s*\("),
            ("system_calls", r"(?i)(system|shell|cmd|execute)\s*\("),
        ];

        for (name, pattern) in &patterns {
            match Regex::new(pattern) {
                Ok(regex) => {
                    regex_patterns.insert(name.to_string(), regex);
                }
                Err(e) => {
                    return Err(AnalysisError::DetectionError(format!(
                        "Failed to compile regex pattern '{}': {}",
                        name, e
                    )));
                }
            }
        }

        Ok(Self {
            config,
            regex_patterns,
        })
    }

    /// Match patterns in the given context
    pub async fn match_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Apply regex patterns
        issues.extend(self.apply_regex_patterns(context).await?);

        // Apply signature patterns
        issues.extend(self.apply_signature_patterns(context).await?);

        // Apply behavioral patterns
        issues.extend(self.apply_behavioral_patterns(context).await?);

        Ok(issues)
    }

    async fn apply_regex_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        for (pattern_name, regex) in &self.regex_patterns {
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    ).with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        format!("Suspicious Pattern Detected: {}", pattern_name),
                        format!(
                            "Detected suspicious pattern '{}' that may indicate malicious agent behavior",
                            pattern_name
                        ),
                        location,
                    )
                    .with_severity(self.determine_severity(pattern_name))
                    .with_confidence(self.determine_confidence(pattern_name))
                    .with_remediation(format!("Review {} for legitimate use", pattern_name))
                    .with_detector("PatternMatcher".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    async fn apply_signature_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Known malicious agent signatures
        let signatures = [
            ("RAT_SIGNATURE", "remote_access_tool"),
            ("KEYLOGGER_SIGNATURE", "keylog"),
            ("BACKDOOR_SIGNATURE", "backdoor"),
            ("TROJAN_SIGNATURE", "trojan"),
            ("BOTNET_SIGNATURE", "botnet"),
        ];

        for (signature_name, signature) in &signatures {
            if content.to_lowercase().contains(&signature.to_lowercase()) {
                let location = SecurityLocation::new(
                    context.file_path.clone(),
                    1,
                    1,
                ).with_columns(0, 0);

                let issue = SecurityIssue::new(
                    SecurityIssueType::PotentialMaliciousAgent,
                    VulnerabilityType::Static,
                    format!("Malicious Signature Detected: {}", signature_name),
                    format!(
                        "Detected known malicious signature '{}' in code",
                        signature
                    ),
                    location,
                )
                .with_severity(SecuritySeverity::Critical)
                .with_confidence(0.95)
                .with_remediation("Immediately investigate and remove malicious code".to_string())
                .with_detector("PatternMatcher".to_string());
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    async fn apply_behavioral_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Multi-line behavioral patterns
        let lines: Vec<&str> = content.lines().collect();

        // Look for command and control patterns
        if self.detect_cnc_pattern(&lines) {
            let location = SecurityLocation::new(
                context.file_path.clone(),
                1,
                lines.len() as i32,
            ).with_columns(0, 0);

            let issue = SecurityIssue::new(
                SecurityIssueType::PotentialMaliciousAgent,
                VulnerabilityType::Static,
                "Command and Control Pattern".to_string(),
                "Detected potential command and control communication pattern".to_string(),
                location,
            )
            .with_severity(SecuritySeverity::Critical)
            .with_confidence(0.85)
            .with_remediation("Investigate command and control infrastructure".to_string())
            .with_detector("PatternMatcher".to_string());
            issues.push(issue);
        }

        // Look for data collection patterns
        if self.detect_data_collection_pattern(&lines) {
            let location = SecurityLocation::new(
                context.file_path.clone(),
                1,
                lines.len() as i32,
            ).with_columns(0, 0);

            let issue = SecurityIssue::new(
                SecurityIssueType::PotentialMaliciousAgent,
                VulnerabilityType::Static,
                "Data Collection Pattern".to_string(),
                "Detected systematic data collection behavior".to_string(),
                location,
            )
            .with_severity(SecuritySeverity::High)
            .with_confidence(0.75)
            .with_remediation("Review data collection for privacy compliance".to_string())
            .with_detector("PatternMatcher".to_string());
            issues.push(issue);
        }

        Ok(issues)
    }

    fn detect_cnc_pattern(&self, lines: &[&str]) -> bool {
        let cnc_indicators = ["http://", "https://", "tcp://", "udp://"];
        let command_indicators = ["command", "cmd", "execute", "run"];

        let has_network = lines.iter().any(|line| {
            cnc_indicators.iter().any(|indicator| line.contains(indicator))
        });

        let has_commands = lines.iter().any(|line| {
            command_indicators.iter().any(|indicator| line.to_lowercase().contains(indicator))
        });

        has_network && has_commands
    }

    fn detect_data_collection_pattern(&self, lines: &[&str]) -> bool {
        let collection_indicators = ["collect", "gather", "scan", "enumerate"];
        let data_indicators = ["file", "directory", "process", "registry", "memory"];

        let has_collection = lines.iter().any(|line| {
            collection_indicators.iter().any(|indicator| line.to_lowercase().contains(indicator))
        });

        let has_data_targets = lines.iter().any(|line| {
            data_indicators.iter().any(|indicator| line.to_lowercase().contains(indicator))
        });

        has_collection && has_data_targets
    }

    fn determine_severity(&self, pattern_name: &str) -> SecuritySeverity {
        match pattern_name {
            "suspicious_network" => SecuritySeverity::High,
            "file_operations" => SecuritySeverity::Medium,
            "process_creation" => SecuritySeverity::High,
            "crypto_operations" => SecuritySeverity::Medium,
            "system_calls" => SecuritySeverity::High,
            _ => SecuritySeverity::Low,
        }
    }

    fn determine_confidence(&self, pattern_name: &str) -> f64 {
        match pattern_name {
            "suspicious_network" => 0.7,
            "file_operations" => 0.5,
            "process_creation" => 0.8,
            "crypto_operations" => 0.6,
            "system_calls" => 0.8,
            _ => 0.4,
        }
    }
}

#[async_trait]
impl AnalysisModule for PatternMatcher {
    async fn analyze(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!("Starting pattern matching analysis for {:?}", context.file_path);
        self.match_patterns(context).await
    }

    fn module_name(&self) -> &'static str {
        "PatternMatcher"
    }

    fn can_analyze(&self, context: &SecurityContext) -> bool {
        matches!(
            context.language,
            SourceLanguage::Rust
            | SourceLanguage::Python
            | SourceLanguage::JavaScript
            | SourceLanguage::TypeScript
            | SourceLanguage::C
            | SourceLanguage::Cpp
        )
    }
}