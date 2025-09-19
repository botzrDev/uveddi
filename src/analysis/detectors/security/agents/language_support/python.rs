//! Python-specific agent pattern analysis

use super::{LanguageAgentAnalyzer, LanguagePatterns};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use std::collections::HashMap;

/// Python-specific agent pattern analyzer
pub struct PythonAgentAnalyzer {
    patterns: LanguagePatterns,
}

impl PythonAgentAnalyzer {
    pub fn new() -> Self {
        let patterns = LanguagePatterns {
            async_patterns: vec![
                "asyncio",
                "async def",
                "await ",
                "aiohttp",
                "trio",
                "curio",
                "threading",
                "multiprocessing",
            ],
            network_patterns: vec![
                "socket",
                "requests",
                "urllib",
                "http.client",
                "ftplib",
                "smtplib",
                "telnetlib",
                "xmlrpc",
            ],
            process_patterns: vec![
                "subprocess",
                "os.system",
                "os.popen",
                "os.spawn",
                "multiprocessing",
                "threading",
                "concurrent.futures",
            ],
            crypto_patterns: vec![
                "cryptography",
                "hashlib",
                "hmac",
                "secrets",
                "ssl",
                "pycrypto",
                "nacl",
            ],
        };

        Self { patterns }
    }

    /// Analyze Python-specific agent patterns
    async fn analyze_python_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for Python-specific dangerous patterns
        issues.extend(self.analyze_dangerous_imports(context).await?);

        // Check for code execution patterns
        issues.extend(self.analyze_code_execution_patterns(context).await?);

        // Check for obfuscation patterns
        issues.extend(self.analyze_obfuscation_patterns(context).await?);

        // Check for persistence patterns
        issues.extend(self.analyze_persistence_patterns(context).await?);

        Ok(issues)
    }

    async fn analyze_dangerous_imports(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        let dangerous_imports = [
            "import os",
            "import subprocess",
            "import sys",
            "import socket",
            "import pickle",
            "import marshal",
            "import ctypes",
            "from ctypes",
            "import _ctypes",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for import in &dangerous_imports {
                if line.trim().starts_with(import) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    ).with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Dangerous Python Import Detected".to_string(),
                        format!(
                            "Detected potentially dangerous import '{}' that could enable malicious behavior",
                            import
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.6)
                    .with_remediation("Review import usage for legitimate purposes".to_string())
                    .with_detector("PythonAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_code_execution_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        let execution_patterns = [
            "eval(",
            "exec(",
            "compile(",
            "__import__(",
            "getattr(",
            "setattr(",
            "hasattr(",
            "delattr(",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &execution_patterns {
                if line.contains(pattern) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    ).with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Python Code Execution Pattern".to_string(),
                        format!(
                            "Detected code execution pattern '{}' that could execute arbitrary code",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::High)
                    .with_confidence(0.8)
                    .with_remediation("Validate and sanitize any dynamic code execution".to_string())
                    .with_detector("PythonAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_obfuscation_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        // Check for base64 encoding/decoding
        if content.contains("base64") && (content.contains("decode") || content.contains("encode")) {
            let location = SecurityLocation::new(
                context.file_path.clone(),
                1,
                1,
            ).with_columns(0, 0);

            let issue = SecurityIssue::new(
                SecurityIssueType::PotentialMaliciousAgent,
                VulnerabilityType::Static,
                "Python Obfuscation Pattern".to_string(),
                "Detected base64 encoding/decoding which may be used for obfuscation".to_string(),
                location,
            )
            .with_severity(SecuritySeverity::Medium)
            .with_confidence(0.7)
            .with_remediation("Review encoding/decoding for legitimate use cases".to_string())
            .with_detector("PythonAgentAnalyzer".to_string());
            issues.push(issue);
        }

        // Check for string manipulation that might hide malicious code
        let obfuscation_patterns = [
            "chr(",
            "ord(",
            "hex(",
            "oct(",
            "bin(",
            "bytes.fromhex(",
            "codecs.decode(",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &obfuscation_patterns {
                if line.contains(pattern) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    ).with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Python Obfuscation Function".to_string(),
                        format!(
                            "Detected obfuscation function '{}' that may hide malicious content",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.6)
                    .with_remediation("Review string manipulation for obfuscation attempts".to_string())
                    .with_detector("PythonAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_persistence_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        let persistence_patterns = [
            "crontab",
            "systemd",
            "startup",
            "autostart",
            "registry",
            "scheduled_task",
            "daemon",
            "service",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &persistence_patterns {
                if line.to_lowercase().contains(&pattern.to_lowercase()) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    ).with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Python Persistence Pattern".to_string(),
                        format!(
                            "Detected persistence pattern '{}' that may establish system persistence",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::High)
                    .with_confidence(0.7)
                    .with_remediation("Verify persistence mechanisms are authorized".to_string())
                    .with_detector("PythonAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }
}

impl Default for PythonAgentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAgentAnalyzer for PythonAgentAnalyzer {
    fn supported_language(&self) -> SourceLanguage {
        SourceLanguage::Python
    }

    async fn analyze_language_specific(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        self.analyze_python_patterns(context).await
    }
}