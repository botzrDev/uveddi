//! A01:2021 – Injection Detector
//!
//! Detects injection vulnerabilities including SQL injection,
//! command injection, LDAP injection, and XSS attacks.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A01:2021 – Injection Detector
pub struct InjectionDetector {
    injection_patterns: HashMap<SourceLanguage, Vec<InjectionPattern>>,
}

#[derive(Debug, Clone)]
pub struct InjectionPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub injection_type: InjectionType,
}

#[derive(Debug, Clone)]
pub enum InjectionType {
    SqlInjection,
    CommandInjection,
    XssInjection,
    LdapInjection,
    NoSqlInjection,
    XPathInjection,
}

impl InjectionDetector {
    pub fn new() -> Self {
        Self {
            injection_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<InjectionPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<InjectionPattern> {
        vec![
            InjectionPattern {
                pattern: "format!(\"SELECT".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Potential SQL injection via string formatting".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Critical,
                injection_type: InjectionType::SqlInjection,
            },
            InjectionPattern {
                pattern: "Command::new".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Potential command injection vulnerability".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::High,
                injection_type: InjectionType::CommandInjection,
            },
        ]
    }

    fn python_patterns() -> Vec<InjectionPattern> {
        vec![
            InjectionPattern {
                pattern: "cursor.execute(f\"".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Potential SQL injection via f-string".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                injection_type: InjectionType::SqlInjection,
            },
            InjectionPattern {
                pattern: "os.system(".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Potential command injection via os.system".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Critical,
                injection_type: InjectionType::CommandInjection,
            },
            InjectionPattern {
                pattern: "eval(".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Dangerous use of eval() function".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                injection_type: InjectionType::CommandInjection,
            },
        ]
    }

    fn javascript_patterns() -> Vec<InjectionPattern> {
        vec![
            InjectionPattern {
                pattern: "query(`SELECT".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Potential SQL injection in template literal".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Critical,
                injection_type: InjectionType::SqlInjection,
            },
            InjectionPattern {
                pattern: "innerHTML =".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Potential XSS via innerHTML assignment".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::High,
                injection_type: InjectionType::XssInjection,
            },
            InjectionPattern {
                pattern: "eval(".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                description: "Dangerous use of eval() function".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                injection_type: InjectionType::CommandInjection,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &InjectionPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::Injection,
            pattern.vulnerability_type.clone(),
            "Injection Vulnerability".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.injection_type))
        .with_architectural_correlation(
            OwaspCategory::Injection
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(injection_type: &InjectionType) -> String {
        match injection_type {
            InjectionType::SqlInjection => {
                "Use parameterized queries or prepared statements. Never concatenate user input into SQL queries.".to_string()
            }
            InjectionType::CommandInjection => {
                "Avoid system calls with user input. Use safe APIs and validate/sanitize all input.".to_string()
            }
            InjectionType::XssInjection => {
                "Encode output and use secure DOM manipulation methods. Implement Content Security Policy.".to_string()
            }
            InjectionType::LdapInjection => {
                "Use parameterized LDAP queries and validate input against LDAP injection patterns.".to_string()
            }
            InjectionType::NoSqlInjection => {
                "Use parameterized NoSQL queries and validate input data types.".to_string()
            }
            InjectionType::XPathInjection => {
                "Use parameterized XPath expressions and validate XML input.".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "prepared", "parameterized", "bind", "placeholder", "escape",
            "sanitize", "validate", "?", "$1", "$2",
        ];

        safe_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for InjectionDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.injection_patterns.get(&file.language) {
            let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
                AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
            })?;

            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if line.contains(&pattern.pattern) && !self.is_likely_safe(line) {
                        let location = SecurityLocation::new(
                            file.file_path.as_ref().to_path_buf(),
                            line_num as i32 + 1,
                            line_num as i32 + 1,
                        );

                        let vulnerability = self.create_vulnerability(pattern, location);
                        vulnerabilities.push(vulnerability);
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_injection_detection() {
        let detector = InjectionDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: "cursor.execute(f\"SELECT * FROM users WHERE id = {user_id}\")".to_string(),
            tree: None,
        };

        std::fs::write("test.py", "cursor.execute(f\"SELECT * FROM users WHERE id = {user_id}\")").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.py").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::Injection);
        assert_eq!(vulnerabilities[0].severity, SecuritySeverity::Critical);
    }
}