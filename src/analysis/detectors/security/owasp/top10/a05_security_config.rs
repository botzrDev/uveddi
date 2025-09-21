//! A05:2021 – Security Misconfiguration Detector
//!
//! This module detects security misconfigurations including default credentials,
//! unnecessary features enabled, and improper error handling.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A05:2021 – Security Misconfiguration Detector
pub struct SecurityMisconfigurationDetector {
    config_patterns: HashMap<SourceLanguage, Vec<ConfigPattern>>,
}

#[derive(Debug, Clone)]
struct ConfigPattern {
    pattern: String,
    description: String,
    confidence: f64,
    severity: SecuritySeverity,
}

impl SecurityMisconfigurationDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Common configuration issues
        let common_patterns = vec![
            ConfigPattern {
                pattern: "debug = true".to_string(),
                description: "Debug mode enabled in production".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
            },
            ConfigPattern {
                pattern: "password = \"\"".to_string(),
                description: "Empty password configuration".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
            },
            ConfigPattern {
                pattern: "admin".to_string(),
                description: "Default admin credentials".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::High,
            },
        ];

        // Add common patterns to all supported languages
        patterns.insert(SourceLanguage::Rust, common_patterns.clone());
        patterns.insert(SourceLanguage::Python, common_patterns.clone());
        patterns.insert(SourceLanguage::JavaScript, common_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, common_patterns);

        Self {
            config_patterns: patterns,
        }
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for SecurityMisconfigurationDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.config_patterns.get(&file.language) {
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
                            OwaspCategory::SecurityMisconfiguration,
                            SecurityIssueType::SecurityMisconfiguration,
                            "Security Misconfiguration".to_string(),
                            pattern.description.clone(),
                            location,
                        )
                        .with_confidence(pattern.confidence)
                        .with_severity(pattern.severity)
                        .with_remediation(
                            "Review and harden configuration settings. Remove default credentials.".to_string(),
                        );

                        vulnerabilities.push(vulnerability);
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }
}