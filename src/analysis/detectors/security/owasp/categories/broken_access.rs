//! A05:2021 – Broken Access Control Detector
//!
//! Detects access control vulnerabilities and authorization failures.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A05:2021 – Broken Access Control Detector
pub struct BrokenAccessDetector {
    access_patterns: HashMap<SourceLanguage, Vec<AccessPattern>>,
}

#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub access_type: AccessControlIssue,
}

#[derive(Debug, Clone)]
pub enum AccessControlIssue {
    MissingAuthorization,
    PrivilegeEscalation,
    DirectObjectReference,
    CorsVulnerability,
    BypassableControls,
    ElevatedPermissions,
}

impl BrokenAccessDetector {
    pub fn new() -> Self {
        Self {
            access_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<AccessPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<AccessPattern> {
        vec![
            AccessPattern {
                pattern: "user_id = request".to_string(),
                vulnerability_type: SecurityIssueType::BrokenAccessControl,
                description: "Direct object reference without authorization check".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::High,
                access_type: AccessControlIssue::DirectObjectReference,
            },
            AccessPattern {
                pattern: "sudo".to_string(),
                vulnerability_type: SecurityIssueType::BrokenAccessControl,
                description: "Elevated permissions usage".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                access_type: AccessControlIssue::ElevatedPermissions,
            },
        ]
    }

    fn python_patterns() -> Vec<AccessPattern> {
        vec![
            AccessPattern {
                pattern: "if user.is_admin".to_string(),
                vulnerability_type: SecurityIssueType::BrokenAccessControl,
                description: "Simple role-based access control".to_string(),
                confidence: 0.4,
                severity: SecuritySeverity::Low,
                access_type: AccessControlIssue::BypassableControls,
            },
            AccessPattern {
                pattern: "os.chmod(777".to_string(),
                vulnerability_type: SecurityIssueType::BrokenAccessControl,
                description: "Overly permissive file permissions".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                access_type: AccessControlIssue::ElevatedPermissions,
            },
        ]
    }

    fn javascript_patterns() -> Vec<AccessPattern> {
        vec![
            AccessPattern {
                pattern: "Access-Control-Allow-Origin: *".to_string(),
                vulnerability_type: SecurityIssueType::BrokenAccessControl,
                description: "Overly permissive CORS configuration".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                access_type: AccessControlIssue::CorsVulnerability,
            },
            AccessPattern {
                pattern: "req.params.id".to_string(),
                vulnerability_type: SecurityIssueType::BrokenAccessControl,
                description: "Direct parameter access without validation".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                access_type: AccessControlIssue::DirectObjectReference,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &AccessPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::BrokenAccessControl,
            pattern.vulnerability_type.clone(),
            "Access Control Vulnerability".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.access_type))
        .with_architectural_correlation(
            OwaspCategory::BrokenAccessControl
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(access_type: &AccessControlIssue) -> String {
        match access_type {
            AccessControlIssue::MissingAuthorization => {
                "Implement proper authorization checks for all protected resources".to_string()
            }
            AccessControlIssue::PrivilegeEscalation => {
                "Enforce least privilege principle and validate user permissions".to_string()
            }
            AccessControlIssue::DirectObjectReference => {
                "Use indirect object references and validate user ownership".to_string()
            }
            AccessControlIssue::CorsVulnerability => {
                "Configure CORS with specific origins and restrict wildcard usage".to_string()
            }
            AccessControlIssue::BypassableControls => {
                "Implement server-side access controls that cannot be bypassed".to_string()
            }
            AccessControlIssue::ElevatedPermissions => {
                "Use least privilege principle and avoid unnecessary elevated permissions".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "authorize", "permission", "access_control", "rbac",
            "validate_user", "check_permission", "can_access",
        ];

        safe_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for BrokenAccessDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.access_patterns.get(&file.language) {
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
    async fn test_cors_detection() {
        let detector = BrokenAccessDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            content: "res.header('Access-Control-Allow-Origin: *');".to_string(),
            tree: None,
        };

        std::fs::write("test.js", "res.header('Access-Control-Allow-Origin: *');").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.js").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::BrokenAccessControl);
    }
}
