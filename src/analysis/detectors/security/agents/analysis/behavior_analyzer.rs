//! Behavior analysis module for detecting suspicious agent behaviors

use super::{AnalysisConfig, AnalysisModule};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

/// Behavior analyzer for detecting suspicious agent behaviors
pub struct BehaviorAnalyzer {
    config: AnalysisConfig,
}

impl BehaviorAnalyzer {
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }

    /// Analyze behavioral patterns in code
    pub async fn analyze_behavior(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Analyze for stealth behaviors
        issues.extend(self.analyze_stealth_behavior(context).await?);

        // Analyze for persistence behaviors
        issues.extend(self.analyze_persistence_behavior(context).await?);

        // Analyze for evasion behaviors
        issues.extend(self.analyze_evasion_behavior(context).await?);

        // Analyze for data exfiltration behaviors
        issues.extend(self.analyze_exfiltration_behavior(context).await?);

        Ok(issues)
    }

    async fn analyze_stealth_behavior(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for stealth patterns
        let stealth_patterns = [
            "hide",
            "obfuscate",
            "encode",
            "encrypt",
            "base64",
            "hex_decode",
            "rot13",
            "xor",
            ".hidden",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &stealth_patterns {
                if line.to_lowercase().contains(&pattern.to_lowercase()) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Stealth Behavior Pattern".to_string(),
                        format!(
                            "Detected stealth behavior pattern '{}' that may indicate hiding malicious activity",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::High)
                    .with_confidence(0.8)
                    .with_remediation("Review stealth mechanisms for legitimate security purposes".to_string())
                    .with_detector("BehaviorAnalyzer".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_persistence_behavior(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for persistence patterns
        let persistence_patterns = [
            "startup",
            "autostart",
            "registry",
            "cron",
            "systemd",
            "service_install",
            "persist",
            "daemon",
            "background",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &persistence_patterns {
                if line.to_lowercase().contains(&pattern.to_lowercase()) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Persistence Behavior Pattern".to_string(),
                        format!(
                            "Detected persistence behavior pattern '{}' that may indicate attempts to maintain access",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::High)
                    .with_confidence(0.7)
                    .with_remediation("Verify persistence mechanisms are authorized and necessary".to_string())
                    .with_detector("BehaviorAnalyzer".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_evasion_behavior(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for evasion patterns
        let evasion_patterns = [
            "bypass",
            "evade",
            "anti_debug",
            "anti_virus",
            "sandbox_detect",
            "vm_detect",
            "sleep",
            "delay",
            "timing_attack",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &evasion_patterns {
                if line.to_lowercase().contains(&pattern.to_lowercase()) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Evasion Behavior Pattern".to_string(),
                        format!(
                            "Detected evasion behavior pattern '{}' that may indicate attempts to avoid detection",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::High)
                    .with_confidence(0.9)
                    .with_remediation("Investigate evasion techniques for malicious intent".to_string())
                    .with_detector("BehaviorAnalyzer".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_exfiltration_behavior(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for data exfiltration patterns
        let exfiltration_patterns = [
            "upload",
            "send_data",
            "transmit",
            "http_post",
            "ftp_upload",
            "email_send",
            "dns_tunnel",
            "covert_channel",
            "exfiltrate",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &exfiltration_patterns {
                if line.to_lowercase().contains(&pattern.to_lowercase()) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    )
                    .with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Data Exfiltration Pattern".to_string(),
                        format!(
                            "Detected data exfiltration pattern '{}' that may indicate unauthorized data transmission",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Critical)
                    .with_confidence(0.8)
                    .with_remediation("Review data transmission for proper authorization and encryption".to_string())
                    .with_detector("BehaviorAnalyzer".to_string());
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }
}

#[async_trait]
impl AnalysisModule for BehaviorAnalyzer {
    async fn analyze(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!("Starting behavior analysis for {:?}", context.file_path);
        self.analyze_behavior(context).await
    }

    fn module_name(&self) -> &'static str {
        "BehaviorAnalyzer"
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
