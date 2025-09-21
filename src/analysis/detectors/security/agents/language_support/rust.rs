//! Rust-specific agent pattern analysis

use super::{LanguageAgentAnalyzer, LanguagePatterns};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use std::collections::HashMap;

/// Rust-specific agent pattern analyzer
pub struct RustAgentAnalyzer {
    patterns: LanguagePatterns,
}

impl RustAgentAnalyzer {
    pub fn new() -> Self {
        let patterns = LanguagePatterns {
            async_patterns: vec![
                "tokio::spawn",
                "async fn",
                "async move",
                "futures::",
                "tokio::task::",
                "async_std::",
                ".await",
            ],
            network_patterns: vec![
                "TcpStream::",
                "UdpSocket::",
                "reqwest::",
                "hyper::",
                "tonic::",
                "std::net::",
                "tokio::net::",
            ],
            process_patterns: vec![
                "std::process::",
                "Command::new",
                "spawn()",
                "std::thread::",
                "crossbeam::",
                "rayon::",
            ],
            crypto_patterns: vec![
                "ring::",
                "rustls::",
                "openssl::",
                "sha2::",
                "aes::",
                "rsa::",
                "rand::",
            ],
        };

        Self { patterns }
    }

    /// Analyze Rust-specific agent patterns
    async fn analyze_rust_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for unsafe Rust usage in agent contexts
        issues.extend(self.analyze_unsafe_patterns(context).await?);

        // Check for Rust-specific async patterns
        issues.extend(self.analyze_async_patterns(context).await?);

        // Check for Rust FFI usage
        issues.extend(self.analyze_ffi_patterns(context).await?);

        // Check for Rust macro usage
        issues.extend(self.analyze_macro_patterns(context).await?);

        Ok(issues)
    }

    async fn analyze_unsafe_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("unsafe") {
                let location = SecurityLocation::new(
                    context.file_path.clone(),
                    (line_num + 1) as i32,
                    (line_num + 1) as i32,
                )
                .with_columns(0, line.len() as i32);

                let issue = SecurityIssue::new(
                    SecurityIssueType::PotentialMaliciousAgent,
                    VulnerabilityType::Static,
                    "Unsafe Rust Code Detected".to_string(),
                    "Detected unsafe Rust code which could bypass safety guarantees for malicious purposes".to_string(),
                    location,
                )
                .with_severity(SecuritySeverity::High)
                .with_confidence(0.7)
                .with_remediation("Review unsafe code blocks for legitimate use and proper safety guarantees".to_string())
                .with_detector("RustAgentAnalyzer".to_string());
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    async fn analyze_async_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        for pattern in &self.patterns.async_patterns {
            for (line_num, line) in content.lines().enumerate() {
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
                        "Rust Async Pattern Detected".to_string(),
                        format!(
                            "Detected Rust async pattern '{}' that could enable autonomous agent behavior",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.6)
                    .with_remediation("Review async usage for potential agent behavior".to_string())
                    .with_detector("RustAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_ffi_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        let ffi_patterns = [
            "extern \"C\"",
            "libc::",
            "std::ffi::",
            "CString::",
            "CStr::",
        ];

        for pattern in &ffi_patterns {
            for (line_num, line) in content.lines().enumerate() {
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
                        "Rust FFI Usage Detected".to_string(),
                        format!(
                            "Detected Rust FFI pattern '{}' that could interact with native code for malicious purposes",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::High)
                    .with_confidence(0.8)
                    .with_remediation("Review FFI usage for security implications and proper validation".to_string())
                    .with_detector("RustAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_macro_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Look for procedural macros that could hide malicious code
        let macro_patterns = ["proc_macro", "macro_rules!", "#[derive(", "quote!", "syn::"];

        for pattern in &macro_patterns {
            for (line_num, line) in content.lines().enumerate() {
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
                        "Rust Macro Usage Detected".to_string(),
                        format!(
                            "Detected Rust macro pattern '{}' that could generate or hide malicious code",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.5)
                    .with_remediation("Review macro usage and generated code for malicious content".to_string())
                    .with_detector("RustAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }
}

impl Default for RustAgentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAgentAnalyzer for RustAgentAnalyzer {
    fn supported_language(&self) -> SourceLanguage {
        SourceLanguage::Rust
    }

    async fn analyze_language_specific(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        self.analyze_rust_patterns(context).await
    }
}
