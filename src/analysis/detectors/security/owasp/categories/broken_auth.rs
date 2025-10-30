//! A02:2021 – Broken Authentication Detector
//!
//! Detects authentication and session management vulnerabilities.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A02:2021 – Broken Authentication Detector
pub struct BrokenAuthDetector {
    auth_patterns: HashMap<SourceLanguage, Vec<AuthPattern>>,
}

#[derive(Debug, Clone)]
pub struct AuthPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub auth_type: AuthIssueType,
}

#[derive(Debug, Clone)]
pub enum AuthIssueType {
    WeakPasswords,
    InsecureSessionManagement,
    MissingMultiFactor,
    WeakTokenGeneration,
    HardcodedCredentials,
    SessionFixation,
}

impl BrokenAuthDetector {
    pub fn new() -> Self {
        Self {
            auth_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<AuthPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<AuthPattern> {
        vec![
            AuthPattern {
                pattern: "password = \"".to_string(),
                vulnerability_type: SecurityIssueType::AuthenticationFailures,
                description: "Hardcoded password detected".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                auth_type: AuthIssueType::HardcodedCredentials,
            },
            AuthPattern {
                pattern: "session_id = user_id".to_string(),
                vulnerability_type: SecurityIssueType::AuthenticationFailures,
                description: "Predictable session ID generation".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::High,
                auth_type: AuthIssueType::WeakTokenGeneration,
            },
        ]
    }

    fn python_patterns() -> Vec<AuthPattern> {
        vec![
            AuthPattern {
                pattern: "PASSWORD = \"".to_string(),
                vulnerability_type: SecurityIssueType::AuthenticationFailures,
                description: "Hardcoded password in configuration".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                auth_type: AuthIssueType::HardcodedCredentials,
            },
            AuthPattern {
                pattern: "session['user_id']".to_string(),
                vulnerability_type: SecurityIssueType::AuthenticationFailures,
                description: "Insecure session management".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::Medium,
                auth_type: AuthIssueType::InsecureSessionManagement,
            },
        ]
    }

    fn javascript_patterns() -> Vec<AuthPattern> {
        vec![
            AuthPattern {
                pattern: "password: \"".to_string(),
                vulnerability_type: SecurityIssueType::AuthenticationFailures,
                description: "Hardcoded password in JavaScript".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                auth_type: AuthIssueType::HardcodedCredentials,
            },
            AuthPattern {
                pattern: "Math.random()".to_string(),
                vulnerability_type: SecurityIssueType::AuthenticationFailures,
                description: "Weak random number generation for authentication".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                auth_type: AuthIssueType::WeakTokenGeneration,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &AuthPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::AuthenticationFailures,
            pattern.vulnerability_type.clone(),
            "Authentication Vulnerability".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.auth_type))
        .with_architectural_correlation(
            OwaspCategory::AuthenticationFailures
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(auth_type: &AuthIssueType) -> String {
        match auth_type {
            AuthIssueType::WeakPasswords => {
                "Implement strong password policies and validation".to_string()
            }
            AuthIssueType::InsecureSessionManagement => {
                "Use secure session management with proper timeout and invalidation".to_string()
            }
            AuthIssueType::MissingMultiFactor => {
                "Implement multi-factor authentication for sensitive operations".to_string()
            }
            AuthIssueType::WeakTokenGeneration => {
                "Use cryptographically secure random number generation".to_string()
            }
            AuthIssueType::HardcodedCredentials => {
                "Remove hardcoded credentials and use secure configuration management".to_string()
            }
            AuthIssueType::SessionFixation => {
                "Regenerate session IDs after authentication".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "bcrypt",
            "scrypt",
            "argon2",
            "pbkdf2",
            "hash",
            "salt",
            "SecureRandom",
            "crypto.randomBytes",
            "os.urandom",
        ];

        safe_indicators
            .iter()
            .any(|&indicator| line.contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for BrokenAuthDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.auth_patterns.get(&file.language) {
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
    use crate::ast::compatibility_shim::ParsedFileCompat;
    use std::path::PathBuf;

    fn make_parsed_file(path: &str, language: SourceLanguage, content: &str) -> ParsedFileCompat {
        ParsedFileCompat::new(PathBuf::from(path), language, content.to_string())
    }

    #[tokio::test]
    async fn test_hardcoded_password_detection() {
        let detector = BrokenAuthDetector::new();

        let file = make_parsed_file(
            "test.rs",
            SourceLanguage::Rust,
            "let password = \"admin123\";",
        );

        std::fs::write("test.rs", "let password = \"admin123\";").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.rs").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(
            vulnerabilities[0].category,
            OwaspCategory::AuthenticationFailures
        );
    }
}
