//! A06:2021 – Security Misconfiguration Detector
//!
//! Detects security misconfiguration vulnerabilities.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A06:2021 – Security Misconfiguration Detector
pub struct SecurityMisconfigDetector {
    misconfig_patterns: HashMap<SourceLanguage, Vec<MisconfigPattern>>,
}

#[derive(Debug, Clone)]
pub struct MisconfigPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub misconfig_type: MisconfigurationType,
}

#[derive(Debug, Clone)]
pub enum MisconfigurationType {
    DebugModeEnabled,
    DefaultCredentials,
    UnnecessaryFeatures,
    MissingSecurityHeaders,
    VerboseErrorMessages,
    InsecureDefaults,
}

impl SecurityMisconfigDetector {
    pub fn new() -> Self {
        Self {
            misconfig_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<MisconfigPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<MisconfigPattern> {
        vec![
            MisconfigPattern {
                pattern: "debug = true".to_string(),
                vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
                description: "Debug mode enabled in configuration".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                misconfig_type: MisconfigurationType::DebugModeEnabled,
            },
            MisconfigPattern {
                pattern: "password = \"admin\"".to_string(),
                vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
                description: "Default password configuration".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                misconfig_type: MisconfigurationType::DefaultCredentials,
            },
        ]
    }

    fn python_patterns() -> Vec<MisconfigPattern> {
        vec![
            MisconfigPattern {
                pattern: "DEBUG = True".to_string(),
                vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
                description: "Django DEBUG mode enabled".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                misconfig_type: MisconfigurationType::DebugModeEnabled,
            },
            MisconfigPattern {
                pattern: "app.run(debug=True".to_string(),
                vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
                description: "Flask debug mode enabled".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                misconfig_type: MisconfigurationType::DebugModeEnabled,
            },
        ]
    }

    fn javascript_patterns() -> Vec<MisconfigPattern> {
        vec![
            MisconfigPattern {
                pattern: "process.env.NODE_ENV !== 'production'".to_string(),
                vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
                description: "Development mode check in production code".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::Medium,
                misconfig_type: MisconfigurationType::DebugModeEnabled,
            },
            MisconfigPattern {
                pattern: "console.error".to_string(),
                vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
                description: "Verbose error logging that may expose sensitive information".to_string(),
                confidence: 0.4,
                severity: SecuritySeverity::Low,
                misconfig_type: MisconfigurationType::VerboseErrorMessages,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &MisconfigPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::SecurityMisconfiguration,
            pattern.vulnerability_type.clone(),
            "Security Misconfiguration".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.misconfig_type))
        .with_architectural_correlation(
            OwaspCategory::SecurityMisconfiguration
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(misconfig_type: &MisconfigurationType) -> String {
        match misconfig_type {
            MisconfigurationType::DebugModeEnabled => {
                "Disable debug mode in production environments".to_string()
            }
            MisconfigurationType::DefaultCredentials => {
                "Change default credentials and use strong, unique passwords".to_string()
            }
            MisconfigurationType::UnnecessaryFeatures => {
                "Disable unnecessary features and services to reduce attack surface".to_string()
            }
            MisconfigurationType::MissingSecurityHeaders => {
                "Implement proper security headers (HSTS, CSP, X-Frame-Options)".to_string()
            }
            MisconfigurationType::VerboseErrorMessages => {
                "Configure error handling to avoid exposing sensitive information".to_string()
            }
            MisconfigurationType::InsecureDefaults => {
                "Review and harden default configurations before deployment".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "production", "secure", "hardened", "encrypted",
            "if not debug", "unless debug", "disable_debug",
        ];

        safe_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for SecurityMisconfigDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.misconfig_patterns.get(&file.language) {
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
        OwaspCategory::SecurityMisconfiguration
    }

    fn description(&self) -> &str {
        "Detects security misconfiguration vulnerabilities"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_debug_mode_detection() {
        let detector = SecurityMisconfigDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: "DEBUG = True".to_string(),
            tree: None,
        };

        std::fs::write("test.py", "DEBUG = True").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.py").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::SecurityMisconfiguration);
    }
}