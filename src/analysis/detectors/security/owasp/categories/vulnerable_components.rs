//! A09:2021 – Vulnerable and Outdated Components Detector
//!
//! Detects vulnerable components and dependency security issues.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A09:2021 – Vulnerable Components Detector
pub struct VulnerableComponentsDetector {
    component_patterns: HashMap<SourceLanguage, Vec<ComponentPattern>>,
}

#[derive(Debug, Clone)]
pub struct ComponentPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub component_type: ComponentIssue,
}

#[derive(Debug, Clone)]
pub enum ComponentIssue {
    OutdatedDependency,
    KnownVulnerableLibrary,
    UnpatchedComponent,
    UnsupportedVersion,
    MissingSecurityUpdates,
    DeprecatedLibrary,
}

impl VulnerableComponentsDetector {
    pub fn new() -> Self {
        Self {
            component_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<ComponentPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<ComponentPattern> {
        vec![
            ComponentPattern {
                pattern: "openssl = \"0.10\"".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Potentially outdated OpenSSL version".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::High,
                component_type: ComponentIssue::OutdatedDependency,
            },
            ComponentPattern {
                pattern: "serde = \"1.0.0\"".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Potentially outdated serde version".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                component_type: ComponentIssue::OutdatedDependency,
            },
        ]
    }

    fn python_patterns() -> Vec<ComponentPattern> {
        vec![
            ComponentPattern {
                pattern: "django==1.".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Potentially vulnerable Django 1.x version".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Critical,
                component_type: ComponentIssue::KnownVulnerableLibrary,
            },
            ComponentPattern {
                pattern: "requests==2.6".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Known vulnerable requests library version".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                component_type: ComponentIssue::KnownVulnerableLibrary,
            },
            ComponentPattern {
                pattern: "pyyaml==3.".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Vulnerable PyYAML version with known issues".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                component_type: ComponentIssue::KnownVulnerableLibrary,
            },
        ]
    }

    fn javascript_patterns() -> Vec<ComponentPattern> {
        vec![
            ComponentPattern {
                pattern: "\"lodash\": \"4.17.0\"".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Known vulnerable lodash version".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                component_type: ComponentIssue::KnownVulnerableLibrary,
            },
            ComponentPattern {
                pattern: "\"express\": \"4.0".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Potentially vulnerable Express.js version".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::Medium,
                component_type: ComponentIssue::OutdatedDependency,
            },
            ComponentPattern {
                pattern: "\"jquery\": \"1.".to_string(),
                vulnerability_type: SecurityIssueType::VulnerableComponents,
                description: "Outdated jQuery version with known vulnerabilities".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                component_type: ComponentIssue::KnownVulnerableLibrary,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &ComponentPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::VulnerableComponents,
            pattern.vulnerability_type.clone(),
            "Vulnerable Component".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.component_type))
        .with_architectural_correlation(
            OwaspCategory::VulnerableComponents
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(component_type: &ComponentIssue) -> String {
        match component_type {
            ComponentIssue::OutdatedDependency => {
                "Update to the latest stable version of the dependency".to_string()
            }
            ComponentIssue::KnownVulnerableLibrary => {
                "Upgrade to a patched version or find alternative library".to_string()
            }
            ComponentIssue::UnpatchedComponent => {
                "Apply security patches or upgrade to supported version".to_string()
            }
            ComponentIssue::UnsupportedVersion => {
                "Migrate to a supported version with security updates".to_string()
            }
            ComponentIssue::MissingSecurityUpdates => {
                "Enable automatic security updates or monitor for vulnerabilities".to_string()
            }
            ComponentIssue::DeprecatedLibrary => {
                "Replace with actively maintained alternative library".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "latest", "^", "~", ">=", "audit", "security", "updated", "patched", "secure",
        ];

        safe_indicators
            .iter()
            .any(|&indicator| line.contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for VulnerableComponentsDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        // Only check dependency files
        let filename = file
            .file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        let is_dependency_file = matches!(
            filename,
            "Cargo.toml"
                | "requirements.txt"
                | "package.json"
                | "package-lock.json"
                | "yarn.lock"
                | "Pipfile"
                | "setup.py"
                | "pyproject.toml"
        );

        if !is_dependency_file {
            return Ok(vulnerabilities);
        }

        if let Some(patterns) = self.component_patterns.get(&file.language) {
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
    async fn test_vulnerable_component_detection() {
        let detector = VulnerableComponentsDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("package.json")),
            language: SourceLanguage::JavaScript,
            content: "\"lodash\": \"4.17.0\"".to_string(),
            tree: None,
        };

        std::fs::write("package.json", "\"lodash\": \"4.17.0\"").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("package.json").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(
            vulnerabilities[0].category,
            OwaspCategory::VulnerableComponents
        );
    }
}
