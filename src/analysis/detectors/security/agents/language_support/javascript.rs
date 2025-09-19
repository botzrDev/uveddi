//! JavaScript/TypeScript-specific agent pattern analysis

use super::{LanguageAgentAnalyzer, LanguagePatterns};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use std::collections::HashMap;

/// JavaScript/TypeScript-specific agent pattern analyzer
pub struct JavaScriptAgentAnalyzer {
    patterns: LanguagePatterns,
}

impl JavaScriptAgentAnalyzer {
    pub fn new() -> Self {
        let patterns = LanguagePatterns {
            async_patterns: vec![
                "async function",
                "async (",
                "await ",
                "Promise",
                "setTimeout",
                "setInterval",
                "setImmediate",
                "process.nextTick",
            ],
            network_patterns: vec![
                "fetch(",
                "XMLHttpRequest",
                "WebSocket",
                "EventSource",
                "http.request",
                "https.request",
                "net.connect",
                "dgram.createSocket",
            ],
            process_patterns: vec![
                "child_process",
                "spawn(",
                "exec(",
                "execSync(",
                "fork(",
                "Worker(",
                "cluster",
            ],
            crypto_patterns: vec![
                "crypto.createHash",
                "crypto.createHmac",
                "crypto.createCipher",
                "crypto.randomBytes",
                "bcrypt",
                "scrypt",
                "pbkdf2",
            ],
        };

        Self { patterns }
    }

    /// Analyze JavaScript/TypeScript-specific agent patterns
    async fn analyze_js_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for dangerous JavaScript patterns
        issues.extend(self.analyze_dangerous_functions(context).await?);

        // Check for code execution patterns
        issues.extend(self.analyze_code_execution_patterns(context).await?);

        // Check for DOM manipulation patterns
        issues.extend(self.analyze_dom_patterns(context).await?);

        // Check for browser API abuse
        issues.extend(self.analyze_browser_api_patterns(context).await?);

        Ok(issues)
    }

    async fn analyze_dangerous_functions(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        let dangerous_functions = [
            "eval(",
            "Function(",
            "setTimeout(",
            "setInterval(",
            "document.write(",
            "innerHTML",
            "outerHTML",
            "insertAdjacentHTML",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for func in &dangerous_functions {
                if line.contains(func) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    ).with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Dangerous JavaScript Function".to_string(),
                        format!(
                            "Detected potentially dangerous function '{}' that could execute arbitrary code",
                            func
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::High)
                    .with_confidence(0.8)
                    .with_remediation("Validate and sanitize any dynamic code execution".to_string())
                    .with_detector("JavaScriptAgentAnalyzer".to_string());
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

        // Check for Node.js code execution patterns
        let execution_patterns = [
            "require(",
            "import(",
            "vm.runInThisContext",
            "vm.runInNewContext",
            "vm.createScript",
            "module.require",
            "global.require",
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
                        "JavaScript Code Execution Pattern".to_string(),
                        format!(
                            "Detected code execution pattern '{}' that could load or execute arbitrary code",
                            pattern
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.7)
                    .with_remediation("Review dynamic code loading for security implications".to_string())
                    .with_detector("JavaScriptAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }

    async fn analyze_dom_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        let dom_patterns = [
            "document.createElement",
            "document.getElementById",
            "document.querySelector",
            "element.appendChild",
            "element.insertBefore",
            "element.replaceChild",
            "element.removeChild",
        ];

        let mut dom_manipulation_count = 0;
        for line in content.lines() {
            for pattern in &dom_patterns {
                if line.contains(pattern) {
                    dom_manipulation_count += 1;
                    break;
                }
            }
        }

        // Only flag if there's significant DOM manipulation
        if dom_manipulation_count > 5 {
            let location = SecurityLocation::new(
                context.file_path.clone(),
                1,
                1,
            ).with_columns(0, 0);

            let issue = SecurityIssue::new(
                SecurityIssueType::PotentialMaliciousAgent,
                VulnerabilityType::Static,
                "Extensive DOM Manipulation".to_string(),
                format!(
                    "Detected extensive DOM manipulation ({} instances) that could be used for malicious UI changes",
                    dom_manipulation_count
                ),
                location,
            )
            .with_severity(SecuritySeverity::Medium)
            .with_confidence(0.6)
            .with_remediation("Review DOM manipulation for legitimate UI purposes".to_string())
            .with_detector("JavaScriptAgentAnalyzer".to_string());
            issues.push(issue);
        }

        Ok(issues)
    }

    async fn analyze_browser_api_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let content = &context.content;

        let sensitive_apis = [
            "navigator.geolocation",
            "navigator.mediaDevices",
            "navigator.permissions",
            "localStorage",
            "sessionStorage",
            "indexedDB",
            "webkitRequestFileSystem",
            "requestFileSystem",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for api in &sensitive_apis {
                if line.contains(api) {
                    let location = SecurityLocation::new(
                        context.file_path.clone(),
                        (line_num + 1) as i32,
                        (line_num + 1) as i32,
                    ).with_columns(0, line.len() as i32);

                    let issue = SecurityIssue::new(
                        SecurityIssueType::PotentialMaliciousAgent,
                        VulnerabilityType::Static,
                        "Sensitive Browser API Usage".to_string(),
                        format!(
                            "Detected usage of sensitive browser API '{}' that could access private user data",
                            api
                        ),
                        location,
                    )
                    .with_severity(SecuritySeverity::Medium)
                    .with_confidence(0.7)
                    .with_remediation("Ensure proper user consent and data protection for sensitive APIs".to_string())
                    .with_detector("JavaScriptAgentAnalyzer".to_string());
                    issues.push(issue);
                    break; // Only report once per line
                }
            }
        }

        Ok(issues)
    }
}

impl Default for JavaScriptAgentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAgentAnalyzer for JavaScriptAgentAnalyzer {
    fn supported_language(&self) -> SourceLanguage {
        SourceLanguage::JavaScript
    }

    async fn analyze_language_specific(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        self.analyze_js_patterns(context).await
    }

    fn can_analyze(&self, context: &SecurityContext) -> bool {
        matches!(context.language, SourceLanguage::JavaScript | SourceLanguage::TypeScript)
    }
}