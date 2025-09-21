//! JavaScript/TypeScript-specific OWASP vulnerability analysis
//!
//! This module provides JavaScript and TypeScript-specific security analysis for OWASP Top 10 vulnerabilities.

use crate::analysis::detectors::security::owasp::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// JavaScript/TypeScript-specific OWASP analyzer
pub struct JavaScriptOwaspAnalyzer {
    patterns: Vec<JSSecurityPattern>,
}

#[derive(Clone, Debug)]
struct JSSecurityPattern {
    name: String,
    pattern: String,
    category: OwaspCategory,
    severity: SecuritySeverity,
    description: String,
}

impl JavaScriptOwaspAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> Vec<JSSecurityPattern> {
        vec![
            JSSecurityPattern {
                name: "Eval Usage".to_string(),
                pattern: "eval(".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::Critical,
                description: "eval() executes arbitrary JavaScript code".to_string(),
            },
            JSSecurityPattern {
                name: "innerHTML Assignment".to_string(),
                pattern: "innerHTML =".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::High,
                description: "Direct innerHTML assignment can lead to XSS".to_string(),
            },
            JSSecurityPattern {
                name: "Document Write".to_string(),
                pattern: "document.write(".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::High,
                description: "document.write() is vulnerable to XSS attacks".to_string(),
            },
            JSSecurityPattern {
                name: "SQL Template Literal".to_string(),
                pattern: "query(`SELECT".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::Critical,
                description: "SQL in template literals enables injection".to_string(),
            },
            JSSecurityPattern {
                name: "Child Process Exec".to_string(),
                pattern: "child_process.exec(".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::Critical,
                description: "Process execution with user input enables command injection"
                    .to_string(),
            },
            JSSecurityPattern {
                name: "Crypto Random".to_string(),
                pattern: "Math.random()".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::Medium,
                description: "Math.random() is not cryptographically secure".to_string(),
            },
            JSSecurityPattern {
                name: "HTTP Protocol".to_string(),
                pattern: "http://".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::Medium,
                description: "HTTP protocol transmits data in plaintext".to_string(),
            },
            JSSecurityPattern {
                name: "Hardcoded Credential".to_string(),
                pattern: "password:".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::Critical,
                description: "Hardcoded credentials should use environment variables".to_string(),
            },
            JSSecurityPattern {
                name: "CORS Wildcard".to_string(),
                pattern: "Access-Control-Allow-Origin: *".to_string(),
                category: OwaspCategory::BrokenAccessControl,
                severity: SecuritySeverity::High,
                description: "CORS wildcard allows any origin access".to_string(),
            },
            JSSecurityPattern {
                name: "Console Log".to_string(),
                pattern: "console.log(".to_string(),
                category: OwaspCategory::SecurityLoggingMonitoringFailures,
                severity: SecuritySeverity::Low,
                description: "Console logs may expose sensitive information".to_string(),
            },
        ]
    }

    pub async fn analyze(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        if !matches!(
            file.language,
            SourceLanguage::JavaScript | SourceLanguage::TypeScript
        ) {
            return Ok(vec![]);
        }

        let mut vulnerabilities = Vec::new();
        let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
            AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
        })?;

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if line.contains(&pattern.pattern) && !Self::is_safe_usage(line, &pattern.pattern) {
                    let location = SecurityLocation::new(
                        file.file_path.as_ref().to_path_buf(),
                        line_num as i32 + 1,
                        line_num as i32 + 1,
                    );

                    let vulnerability = OwaspVulnerability::new(
                        pattern.category.clone(),
                        SecurityIssueType::from(pattern.category.clone()),
                        pattern.name.clone(),
                        pattern.description.clone(),
                        location,
                    )
                    .with_severity(pattern.severity)
                    .with_confidence(0.7);

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        Ok(vulnerabilities)
    }

    fn is_safe_usage(line: &str, pattern: &str) -> bool {
        // Check for safe alternatives or comments indicating intentional usage
        match pattern {
            "innerHTML =" => line.contains("textContent") || line.contains("DOMPurify"),
            "Math.random()" => line.contains("crypto.") || line.contains("// ok"),
            "console.log(" => line.contains("// debug") || line.contains("development"),
            _ => line.contains("// eslint-disable") || line.contains("// nosec"),
        }
    }

    /// Get JavaScript-specific security recommendations
    pub fn get_recommendations() -> HashMap<OwaspCategory, Vec<String>> {
        let mut recommendations = HashMap::new();

        recommendations.insert(
            OwaspCategory::Injection,
            vec![
                "Use parameterized queries instead of string concatenation".to_string(),
                "Use textContent instead of innerHTML".to_string(),
                "Sanitize user input with DOMPurify".to_string(),
                "Implement Content Security Policy (CSP)".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::CryptographicFailures,
            vec![
                "Use crypto.getRandomValues() for secure randomness".to_string(),
                "Always use HTTPS in production".to_string(),
                "Store secrets in environment variables".to_string(),
                "Use bcrypt for password hashing".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::BrokenAccessControl,
            vec![
                "Configure CORS properly, avoid wildcards".to_string(),
                "Implement proper authentication middleware".to_string(),
                "Use JWT tokens with proper expiration".to_string(),
                "Validate all user inputs on the server side".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::SecurityMisconfiguration,
            vec![
                "Remove console.log statements in production".to_string(),
                "Configure security headers (HSTS, CSP, etc.)".to_string(),
                "Keep dependencies updated".to_string(),
                "Use helmet.js for Express.js security".to_string(),
            ],
        );

        recommendations
    }

    /// Check if a JavaScript package is known to be vulnerable
    pub fn check_vulnerable_packages(package: &str, version: &str) -> Option<String> {
        let vulnerable_packages = vec![
            (
                "lodash",
                "4.17.0",
                "Lodash < 4.17.21 has prototype pollution",
            ),
            (
                "express",
                "4.16.0",
                "Express < 4.17.1 has security vulnerabilities",
            ),
            ("axios", "0.18.0", "Axios < 0.21.1 has SSRF vulnerability"),
            ("jquery", "3.3.0", "jQuery < 3.5.0 has XSS vulnerabilities"),
            (
                "moment",
                "2.24.0",
                "Moment.js is deprecated, use date-fns or dayjs",
            ),
        ];

        for (name, vuln_version, message) in vulnerable_packages {
            if package == name && version.starts_with(vuln_version) {
                return Some(message.to_string());
            }
        }

        None
    }

    /// Detect framework-specific issues
    pub fn check_framework_issues(content: &str, framework: &str) -> Vec<String> {
        let mut issues = Vec::new();

        match framework {
            "express" => {
                if !content.contains("helmet(") {
                    issues.push("Consider using helmet.js for security headers".to_string());
                }
                if content.contains("app.use(cors())") {
                    issues.push("CORS configured without restrictions".to_string());
                }
                if !content.contains("express.json({ limit:") {
                    issues.push("No body size limit configured".to_string());
                }
            }
            "react" => {
                if content.contains("dangerouslySetInnerHTML") {
                    issues.push("dangerouslySetInnerHTML usage detected".to_string());
                }
                if content.contains("eval(") {
                    issues.push("Avoid eval() in React components".to_string());
                }
            }
            "vue" => {
                if content.contains("v-html") {
                    issues.push("v-html directive can lead to XSS".to_string());
                }
            }
            "angular" => {
                if content.contains("bypassSecurityTrust") {
                    issues.push("Security bypassed in Angular template".to_string());
                }
            }
            _ => {}
        }

        issues
    }

    /// Detect Node.js specific security patterns
    pub fn check_nodejs_security(content: &str) -> Vec<String> {
        let mut issues = Vec::new();

        if content.contains("require('child_process')") && content.contains(".exec(") {
            issues.push("child_process.exec() usage detected".to_string());
        }

        if content.contains("process.env") && !content.contains("dotenv") {
            issues.push("Consider using dotenv for environment management".to_string());
        }

        if content.contains("JSON.parse(") && !content.contains("try") {
            issues.push("JSON.parse without error handling".to_string());
        }

        issues
    }
}

impl Default for JavaScriptOwaspAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_javascript_patterns() {
        let analyzer = JavaScriptOwaspAnalyzer::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            content: "eval(userInput)".to_string(),
            tree: None,
        };

        std::fs::write("test.js", "eval(userInput)").unwrap();
        let vulnerabilities = analyzer.analyze(&file).await.unwrap();
        std::fs::remove_file("test.js").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::Injection);
    }

    #[test]
    fn test_vulnerable_packages() {
        assert!(JavaScriptOwaspAnalyzer::check_vulnerable_packages("lodash", "4.17.0").is_some());
        assert!(JavaScriptOwaspAnalyzer::check_vulnerable_packages("lodash", "4.17.21").is_none());
    }

    #[test]
    fn test_framework_issues() {
        let express_code = "app.use(cors())";
        let issues = JavaScriptOwaspAnalyzer::check_framework_issues(express_code, "express");
        assert!(!issues.is_empty());
    }
}
