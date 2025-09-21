//! A07:2021 – Cross-Site Scripting (XSS) Detector
//!
//! Detects Cross-Site Scripting vulnerabilities and unsafe output handling.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A07:2021 – Cross-Site Scripting Detector
pub struct XSSDetector {
    xss_patterns: HashMap<SourceLanguage, Vec<XSSPattern>>,
}

#[derive(Debug, Clone)]
pub struct XSSPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub xss_type: XSSIssueType,
}

#[derive(Debug, Clone)]
pub enum XSSIssueType {
    ReflectedXss,
    StoredXss,
    DomBasedXss,
    UnsafeOutputRendering,
    MissingContentSecurityPolicy,
    UnsafeDomManipulation,
}

impl XSSDetector {
    pub fn new() -> Self {
        Self {
            xss_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<XSSPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<XSSPattern> {
        vec![
            XSSPattern {
                pattern: "format!(\"<script>".to_string(),
                vulnerability_type: SecurityIssueType::CrossSiteScripting,
                description: "Potential XSS via HTML generation".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                xss_type: XSSIssueType::ReflectedXss,
            },
            XSSPattern {
                pattern: "unsafe { ".to_string(),
                vulnerability_type: SecurityIssueType::CrossSiteScripting,
                description: "Unsafe block that may bypass XSS protections".to_string(),
                confidence: 0.4,
                severity: SecuritySeverity::Medium,
                xss_type: XSSIssueType::UnsafeOutputRendering,
            },
        ]
    }

    fn python_patterns() -> Vec<XSSPattern> {
        vec![
            XSSPattern {
                pattern: "render_template_string".to_string(),
                vulnerability_type: SecurityIssueType::CrossSiteScripting,
                description: "Unsafe template rendering that may allow XSS".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::High,
                xss_type: XSSIssueType::ReflectedXss,
            },
            XSSPattern {
                pattern: "Markup(".to_string(),
                vulnerability_type: SecurityIssueType::CrossSiteScripting,
                description: "Direct markup injection without escaping".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                xss_type: XSSIssueType::UnsafeOutputRendering,
            },
        ]
    }

    fn javascript_patterns() -> Vec<XSSPattern> {
        vec![
            XSSPattern {
                pattern: "innerHTML =".to_string(),
                vulnerability_type: SecurityIssueType::CrossSiteScripting,
                description: "Potential DOM-based XSS via innerHTML".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::High,
                xss_type: XSSIssueType::DomBasedXss,
            },
            XSSPattern {
                pattern: "document.write(".to_string(),
                vulnerability_type: SecurityIssueType::CrossSiteScripting,
                description: "Potential XSS via document.write".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                xss_type: XSSIssueType::DomBasedXss,
            },
            XSSPattern {
                pattern: "eval(".to_string(),
                vulnerability_type: SecurityIssueType::CrossSiteScripting,
                description: "Dangerous eval() usage that may lead to XSS".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                xss_type: XSSIssueType::DomBasedXss,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &XSSPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::Injection,
            pattern.vulnerability_type.clone(),
            "Cross-Site Scripting Vulnerability".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.xss_type))
        .with_architectural_correlation(
            OwaspCategory::Injection
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(xss_type: &XSSIssueType) -> String {
        match xss_type {
            XSSIssueType::ReflectedXss => {
                "Validate and encode all user input before reflecting it in responses".to_string()
            }
            XSSIssueType::StoredXss => {
                "Sanitize and validate data before storing, encode output when displaying".to_string()
            }
            XSSIssueType::DomBasedXss => {
                "Use safe DOM manipulation methods and avoid eval() or innerHTML with user data".to_string()
            }
            XSSIssueType::UnsafeOutputRendering => {
                "Use context-aware output encoding and template engines with auto-escaping".to_string()
            }
            XSSIssueType::MissingContentSecurityPolicy => {
                "Implement Content Security Policy (CSP) headers to prevent XSS attacks".to_string()
            }
            XSSIssueType::UnsafeDomManipulation => {
                "Use textContent instead of innerHTML, and validate all DOM modifications".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "escape", "encode", "sanitize", "textContent", "createTextNode",
            "DOMPurify", "xss", "csp", "content-security-policy",
        ];

        safe_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for XSSDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.xss_patterns.get(&file.language) {
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

    fn category(&self) -> OwaspCategory {
        OwaspCategory::Injection
    }

    fn description(&self) -> &str {
        "Detects Cross-Site Scripting (XSS) vulnerabilities"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_xss_detection() {
        let detector = XSSDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            content: "element.innerHTML = userInput;".to_string(),
            tree: None,
        };

        std::fs::write("test.js", "element.innerHTML = userInput;").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.js").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::Injection);
    }
}