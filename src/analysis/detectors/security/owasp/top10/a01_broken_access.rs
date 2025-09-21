//! A01:2021 – Broken Access Control Detector
//!
//! This module detects access control vulnerabilities including missing
//! authorization checks, privilege escalation, and CORS misconfigurations.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A01:2021 – Broken Access Control Detector
pub struct BrokenAccessControlDetector {
    patterns: HashMap<SourceLanguage, Vec<AccessControlPattern>>,
}

#[derive(Debug, Clone)]
struct AccessControlPattern {
    pattern: String,
    description: String,
    confidence: f64,
    severity: SecuritySeverity,
}

impl BrokenAccessControlDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(
            SourceLanguage::Rust,
            vec![
                AccessControlPattern {
                    pattern: "pub fn ".to_string(),
                    description: "Public function without access control checks".to_string(),
                    confidence: 0.3,
                    severity: SecuritySeverity::Medium,
                },
                AccessControlPattern {
                    pattern: "actix_web::web::get".to_string(),
                    description: "HTTP GET endpoint without authentication middleware".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "actix_web::web::post".to_string(),
                    description: "HTTP POST endpoint without authentication middleware".to_string(),
                    confidence: 0.7,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "rocket::get".to_string(),
                    description: "Rocket GET route without authentication guard".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "rocket::post".to_string(),
                    description: "Rocket POST route without authentication guard".to_string(),
                    confidence: 0.7,
                    severity: SecuritySeverity::High,
                },
            ],
        );

        // Python patterns
        patterns.insert(
            SourceLanguage::Python,
            vec![
                AccessControlPattern {
                    pattern: "@app.route".to_string(),
                    description: "Flask route without authentication decorator".to_string(),
                    confidence: 0.5,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "@api_view".to_string(),
                    description: "Django REST API view without permission classes".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "def ".to_string(),
                    description: "Function that may need access control".to_string(),
                    confidence: 0.2,
                    severity: SecuritySeverity::Low,
                },
                AccessControlPattern {
                    pattern: "admin.site.register".to_string(),
                    description: "Django admin registration without permission checks".to_string(),
                    confidence: 0.4,
                    severity: SecuritySeverity::Medium,
                },
            ],
        );

        // JavaScript patterns
        patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                AccessControlPattern {
                    pattern: "app.get(".to_string(),
                    description: "Express GET route without authentication middleware".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "app.post(".to_string(),
                    description: "Express POST route without authentication".to_string(),
                    confidence: 0.7,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "router.get(".to_string(),
                    description: "Express router GET without auth middleware".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "router.post(".to_string(),
                    description: "Express router POST without auth middleware".to_string(),
                    confidence: 0.7,
                    severity: SecuritySeverity::High,
                },
            ],
        );

        // TypeScript patterns (same as JavaScript with additional patterns)
        patterns.insert(
            SourceLanguage::TypeScript,
            vec![
                AccessControlPattern {
                    pattern: "app.get(".to_string(),
                    description: "Express GET route without authentication middleware".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "app.post(".to_string(),
                    description: "Express POST route without authentication".to_string(),
                    confidence: 0.7,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "@Get(".to_string(),
                    description: "NestJS GET decorator without auth guards".to_string(),
                    confidence: 0.6,
                    severity: SecuritySeverity::High,
                },
                AccessControlPattern {
                    pattern: "@Post(".to_string(),
                    description: "NestJS POST decorator without auth guards".to_string(),
                    confidence: 0.7,
                    severity: SecuritySeverity::High,
                },
            ],
        );

        Self { patterns }
    }

    fn create_vulnerability(
        &self,
        pattern: &AccessControlPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::BrokenAccessControl,
            SecurityIssueType::BrokenAccessControl,
            "Potential Access Control Issue".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.pattern))
        .with_architectural_correlation(
            OwaspCategory::BrokenAccessControl
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    fn get_remediation_advice(pattern: &str) -> String {
        match pattern {
            p if p.contains("app.get") || p.contains("app.post") => {
                "Add authentication middleware before route handlers. Use passport.js or similar authentication library.".to_string()
            }
            p if p.contains("@app.route") => {
                "Add @login_required decorator or implement custom authentication decorator.".to_string()
            }
            p if p.contains("actix_web") => {
                "Implement authentication middleware using actix-web-httpauth or custom guards.".to_string()
            }
            p if p.contains("rocket") => {
                "Use Rocket request guards to implement authentication and authorization.".to_string()
            }
            _ => "Implement proper access control checks and authentication mechanisms.".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for BrokenAccessControlDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.patterns.get(&file.language) {
            let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
                AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
            })?;

            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if line.contains(&pattern.pattern) && !self.has_auth_context(line) {
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

impl BrokenAccessControlDetector {
    fn has_auth_context(&self, line: &str) -> bool {
        let auth_keywords = [
            "auth", "authenticate", "login", "permission", "authorize",
            "guard", "middleware", "token", "jwt", "session",
        ];

        auth_keywords.iter().any(|&keyword| line.to_lowercase().contains(keyword))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_broken_access_control_detection() {
        let detector = BrokenAccessControlDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            content: "pub fn dangerous_function() { }".to_string(),
            tree: None,
        };

        std::fs::write("test.rs", "pub fn dangerous_function() { }").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.rs").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::BrokenAccessControl);
    }

    #[test]
    fn test_auth_context_detection() {
        let detector = BrokenAccessControlDetector::new();

        assert!(detector.has_auth_context("app.get('/api/data', authenticate, handler)"));
        assert!(detector.has_auth_context("@login_required"));
        assert!(!detector.has_auth_context("app.get('/public', handler)"));
    }
}