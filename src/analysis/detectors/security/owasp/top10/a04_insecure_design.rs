//! A04:2021 – Insecure Design Detector
//!
//! This module detects insecure design patterns and architectural flaws
//! that indicate missing security controls or poor threat modeling.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A04:2021 – Insecure Design Detector
pub struct InsecureDesignDetector {
    design_patterns: HashMap<SourceLanguage, Vec<DesignPattern>>,
}

#[derive(Debug, Clone)]
struct DesignPattern {
    pattern: String,
    description: String,
    confidence: f64,
    severity: SecuritySeverity,
    design_flaw: DesignFlawType,
}

#[derive(Debug, Clone)]
enum DesignFlawType {
    MissingSecurityControls,
    OverPrivileged,
    NoRateLimiting,
    WeakBusinessLogic,
    MissingValidation,
    ImproperErrorHandling,
}

impl InsecureDesignDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Common patterns across languages
        let common_patterns = vec![
            DesignPattern {
                pattern: "admin".to_string(),
                description: "Potential overprivileged admin functionality".to_string(),
                confidence: 0.4,
                severity: SecuritySeverity::Medium,
                design_flaw: DesignFlawType::OverPrivileged,
            },
            DesignPattern {
                pattern: "unlimited".to_string(),
                description: "Potential unlimited resource access".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                design_flaw: DesignFlawType::NoRateLimiting,
            },
            DesignPattern {
                pattern: "try:".to_string(),
                description: "Exception handling that may leak information".to_string(),
                confidence: 0.3,
                severity: SecuritySeverity::Low,
                design_flaw: DesignFlawType::ImproperErrorHandling,
            },
        ];

        // Rust-specific patterns
        patterns.insert(
            SourceLanguage::Rust,
            vec![
                DesignPattern {
                    pattern: "unwrap()".to_string(),
                    description: "Unsafe error handling that can cause panics".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::Medium,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
                DesignPattern {
                    pattern: "expect(".to_string(),
                    description: "Error handling that may expose internal details".to_string(),
                    confidence: 0.4,
                    severity: SecuritySeverity::Low,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
                DesignPattern {
                    pattern: "unsafe ".to_string(),
                    description: "Unsafe code block requiring security review".to_string(),
                    confidence: 0.7,
                    severity: SecuritySeverity::High,
                    design_flaw: DesignFlawType::MissingSecurityControls,
                },
            ],
        );

        // Python-specific patterns
        patterns.insert(
            SourceLanguage::Python,
            vec![
                DesignPattern {
                    pattern: "except:".to_string(),
                    description: "Broad exception handling that may hide security issues".to_string(),
                    confidence: 0.5,
                    severity: SecuritySeverity::Medium,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
                DesignPattern {
                    pattern: "pass".to_string(),
                    description: "Empty exception handler that ignores errors".to_string(),
                    confidence: 0.4,
                    severity: SecuritySeverity::Medium,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
                DesignPattern {
                    pattern: "print(".to_string(),
                    description: "Debug output that may leak sensitive information".to_string(),
                    confidence: 0.3,
                    severity: SecuritySeverity::Low,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
            ],
        );

        // JavaScript patterns
        patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                DesignPattern {
                    pattern: "catch (e) {}".to_string(),
                    description: "Empty catch block ignoring errors".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::Medium,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
                DesignPattern {
                    pattern: "console.log(".to_string(),
                    description: "Debug logging that may expose sensitive data".to_string(),
                    confidence: 0.3,
                    severity: SecuritySeverity::Low,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
                DesignPattern {
                    pattern: "alert(".to_string(),
                    description: "Client-side alert that may expose system information".to_string(),
                    confidence: 0.4,
                    severity: SecuritySeverity::Low,
                    design_flaw: DesignFlawType::ImproperErrorHandling,
                },
            ],
        );

        // TypeScript patterns (similar to JavaScript)
        patterns.insert(SourceLanguage::TypeScript, patterns[&SourceLanguage::JavaScript].clone());

        // Add common patterns to all languages
        for (_, lang_patterns) in patterns.iter_mut() {
            lang_patterns.extend(common_patterns.clone());
        }

        Self {
            design_patterns: patterns,
        }
    }

    fn get_remediation_advice(design_flaw: &DesignFlawType) -> String {
        match design_flaw {
            DesignFlawType::MissingSecurityControls => {
                "Implement proper security controls and defense in depth. Conduct threat modeling.".to_string()
            }
            DesignFlawType::OverPrivileged => {
                "Follow principle of least privilege. Implement proper role-based access control.".to_string()
            }
            DesignFlawType::NoRateLimiting => {
                "Implement rate limiting and throttling to prevent abuse and DoS attacks.".to_string()
            }
            DesignFlawType::WeakBusinessLogic => {
                "Review business logic for security implications and implement proper validation.".to_string()
            }
            DesignFlawType::MissingValidation => {
                "Implement comprehensive input validation and sanitization.".to_string()
            }
            DesignFlawType::ImproperErrorHandling => {
                "Implement secure error handling that doesn't leak sensitive information.".to_string()
            }
        }
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for InsecureDesignDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.design_patterns.get(&file.language) {
            let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
                AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
            })?;

            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if line.contains(&pattern.pattern) {
                        let location = SecurityLocation::new(
                            file.file_path.as_ref().to_path_buf(),
                            line_num as i32 + 1,
                            line_num as i32 + 1,
                        );

                        let vulnerability = OwaspVulnerability::new(
                            OwaspCategory::InsecureDesign,
                            SecurityIssueType::InsecureDesign,
                            "Insecure Design Pattern".to_string(),
                            pattern.description.clone(),
                            location,
                        )
                        .with_confidence(pattern.confidence)
                        .with_severity(pattern.severity)
                        .with_remediation(Self::get_remediation_advice(&pattern.design_flaw))
                        .with_architectural_correlation(
                            OwaspCategory::InsecureDesign
                                .related_anti_patterns()
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                        );

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
    async fn test_insecure_design_detection() {
        let detector = InsecureDesignDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            content: "let result = dangerous_call().unwrap();".to_string(),
            tree: None,
        };

        std::fs::write("test.rs", "let result = dangerous_call().unwrap();").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.rs").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::InsecureDesign);
    }
}