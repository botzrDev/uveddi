//! Session Management Security Detector
//!
//! Specialized detector for session management vulnerabilities including
//! insecure session configuration, weak session handling, and authentication bypass.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// Session management pattern definition
#[derive(Debug, Clone)]
pub struct SessionPattern {
    pub pattern: String,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub session_issue: SessionIssueType,
    pub context: String,
}

#[derive(Debug, Clone)]
pub enum SessionIssueType {
    WeakSessionId,
    InsecureSessionConfig,
    SessionFixation,
    SessionHijacking,
    NoSessionTimeout,
    SessionDataExposure,
    ConcurrentSessionIssues,
    LogoutIncomplete,
}

impl SessionIssueType {
    fn description(&self) -> &'static str {
        match self {
            SessionIssueType::WeakSessionId => "Weak session ID generation",
            SessionIssueType::InsecureSessionConfig => "Insecure session configuration",
            SessionIssueType::SessionFixation => "Session fixation vulnerability",
            SessionIssueType::SessionHijacking => "Session hijacking vulnerability",
            SessionIssueType::NoSessionTimeout => "Missing session timeout configuration",
            SessionIssueType::SessionDataExposure => "Session data exposure vulnerability",
            SessionIssueType::ConcurrentSessionIssues => "Concurrent session handling issues",
            SessionIssueType::LogoutIncomplete => "Incomplete logout implementation",
        }
    }
}

/// Specialized session management detector
pub struct SessionManagementDetector {
    patterns: HashMap<SourceLanguage, Vec<SessionPattern>>,
}

impl SessionManagementDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<SessionPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<SessionPattern> {
        vec![
            SessionPattern {
                pattern: r#"Cookie::build.*\.secure\(false\)"#.to_string(),
                description: "Session cookie not marked as secure".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                session_issue: SessionIssueType::InsecureSessionConfig,
                context: "Cookie security settings".to_string(),
            },
            SessionPattern {
                pattern: r#"Cookie::build.*\.http_only\(false\)"#.to_string(),
                description: "Session cookie accessible via JavaScript".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::SessionHijacking,
                context: "Cookie HTTP-only settings".to_string(),
            },
            SessionPattern {
                pattern: r#"Uuid::new_v4\(\)\.to_string\(\).*session"#.to_string(),
                description: "Using UUID v4 for session ID (potentially predictable)".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::WeakSessionId,
                context: "Session ID generation".to_string(),
            },
            SessionPattern {
                pattern: r#"session\.insert.*password\|token\|secret"#.to_string(),
                description: "Storing sensitive data in session".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::SessionDataExposure,
                context: "Session data storage".to_string(),
            },
        ]
    }

    fn python_patterns() -> Vec<SessionPattern> {
        vec![
            SessionPattern {
                pattern: r#"SESSION_COOKIE_SECURE\s*=\s*False"#.to_string(),
                description: "Session cookies not marked as secure".to_string(),
                confidence: 0.95,
                severity: SecuritySeverity::High,
                session_issue: SessionIssueType::InsecureSessionConfig,
                context: "Django session settings".to_string(),
            },
            SessionPattern {
                pattern: r#"SESSION_COOKIE_HTTPONLY\s*=\s*False"#.to_string(),
                description: "Session cookies accessible via JavaScript".to_string(),
                confidence: 0.95,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::SessionHijacking,
                context: "Django session settings".to_string(),
            },
            SessionPattern {
                pattern: r#"SESSION_COOKIE_AGE\s*=\s*None"#.to_string(),
                description: "No session timeout configured".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::NoSessionTimeout,
                context: "Django session timeout".to_string(),
            },
            SessionPattern {
                pattern: r#"session\[.*\]\s*=.*password\|token\|secret"#.to_string(),
                description: "Storing sensitive data in session".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::SessionDataExposure,
                context: "Flask session data".to_string(),
            },
            SessionPattern {
                pattern: r#"session\.regenerate_id\(\)"#.to_string(),
                description: "Session ID regeneration (good practice check)".to_string(),
                confidence: 0.3,
                severity: SecuritySeverity::Info,
                session_issue: SessionIssueType::SessionFixation,
                context: "Session regeneration".to_string(),
            },
            SessionPattern {
                pattern: r#"SECRET_KEY\s*=\s*['""][a-zA-Z0-9]{1,10}['""]"#.to_string(),
                description: "Weak or default secret key for session signing".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                session_issue: SessionIssueType::WeakSessionId,
                context: "Session secret key".to_string(),
            },
        ]
    }

    fn javascript_patterns() -> Vec<SessionPattern> {
        vec![
            SessionPattern {
                pattern: r#"session\(.*secure:\s*false"#.to_string(),
                description: "Session middleware with secure flag disabled".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                session_issue: SessionIssueType::InsecureSessionConfig,
                context: "Express session configuration".to_string(),
            },
            SessionPattern {
                pattern: r#"session\(.*httpOnly:\s*false"#.to_string(),
                description: "Session cookies accessible via JavaScript".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::SessionHijacking,
                context: "Express session configuration".to_string(),
            },
            SessionPattern {
                pattern: r#"session\(.*maxAge:\s*undefined\|null"#.to_string(),
                description: "No session expiration configured".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::NoSessionTimeout,
                context: "Express session timeout".to_string(),
            },
            SessionPattern {
                pattern: r#"Math\.random\(\).*session.*id"#.to_string(),
                description: "Weak session ID generation using Math.random()".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                session_issue: SessionIssueType::WeakSessionId,
                context: "Session ID generation".to_string(),
            },
            SessionPattern {
                pattern: r#"req\.session\..*password\|token\|secret"#.to_string(),
                description: "Storing sensitive data in session".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                session_issue: SessionIssueType::SessionDataExposure,
                context: "Session data storage".to_string(),
            },
            SessionPattern {
                pattern: r#"req\.session\.destroy\(\)"#.to_string(),
                description: "Session destruction (logout check)".to_string(),
                confidence: 0.3,
                severity: SecuritySeverity::Info,
                session_issue: SessionIssueType::LogoutIncomplete,
                context: "Session logout".to_string(),
            },
            SessionPattern {
                pattern: r#"session\(.*secret:\s*['"][a-zA-Z0-9]{1,10}['"]"#.to_string(),
                description: "Weak session secret".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                session_issue: SessionIssueType::WeakSessionId,
                context: "Session secret configuration".to_string(),
            },
        ]
    }

    fn analyze_line(
        &self,
        line: &str,
        line_number: usize,
        patterns: &[SessionPattern],
    ) -> Vec<OwaspVulnerability> {
        let mut vulnerabilities = Vec::new();

        for pattern in patterns {
            if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                if regex.is_match(line) {
                    let location = SecurityLocation::new(
                        PathBuf::from("unknown"), // Will be updated by caller
                        line_number as i32,
                        0,
                    );

                    let mut metadata = VulnerabilityMetadata::new();
                    metadata.add_metadata(
                        "session_issue".to_string(),
                        pattern.session_issue.description().to_string(),
                    );
                    metadata.add_metadata("context".to_string(), pattern.context.clone());
                    metadata.add_metadata("pattern_matched".to_string(), pattern.pattern.clone());
                    metadata.add_metadata("line_content".to_string(), line.trim().to_string());

                    let remediation =
                        Self::generate_remediation(&pattern.session_issue, &pattern.context);

                    let vulnerability = OwaspVulnerability::new(
                        OwaspCategory::AuthenticationFailures,
                        SecurityIssueType::SessionManagement,
                        format!(
                            "Session Management: {}",
                            pattern.session_issue.description()
                        ),
                        pattern.description.clone(),
                        location,
                    )
                    .with_severity(pattern.severity)
                    .with_confidence(pattern.confidence)
                    .with_remediation(remediation)
                    .with_metadata(metadata);

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        vulnerabilities
    }

    fn generate_remediation(session_issue: &SessionIssueType, context: &str) -> String {
        let base_advice = match session_issue {
            SessionIssueType::WeakSessionId => {
                "Use cryptographically secure random number generators for session IDs."
            }
            SessionIssueType::InsecureSessionConfig => {
                "Configure session cookies with secure flag, HttpOnly, and appropriate SameSite settings."
            }
            SessionIssueType::SessionFixation => {
                "Regenerate session IDs after authentication and privilege changes."
            }
            SessionIssueType::SessionHijacking => {
                "Enable HttpOnly flag and use HTTPS to prevent session hijacking."
            }
            SessionIssueType::NoSessionTimeout => {
                "Implement appropriate session timeouts and idle timeout mechanisms."
            }
            SessionIssueType::SessionDataExposure => {
                "Avoid storing sensitive data in sessions; use server-side storage with session references."
            }
            SessionIssueType::ConcurrentSessionIssues => {
                "Implement concurrent session management and detection mechanisms."
            }
            SessionIssueType::LogoutIncomplete => {
                "Ensure complete session invalidation and cleanup during logout."
            }
        };

        let context_advice = match context {
            c if c.contains("Django") => " Use Django's built-in session security settings.",
            c if c.contains("Express") => {
                " Configure express-session middleware with secure options."
            }
            c if c.contains("Flask") => " Use Flask-Session with secure configuration.",
            c if c.contains("Cookie") => " Review all cookie security attributes.",
            _ => "",
        };

        format!(
            "{}{} Consider implementing session monitoring and anomaly detection.",
            base_advice, context_advice
        )
    }
}

#[async_trait]
impl OwaspCategoryDetector for SessionManagementDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let patterns = match self.patterns.get(&file.language) {
            Some(patterns) => patterns,
            None => return Ok(Vec::new()),
        };

        let mut vulnerabilities = Vec::new();

        for (line_number, line) in file.source.lines().enumerate() {
            let mut line_vulnerabilities = self.analyze_line(line, line_number + 1, patterns);

            // Update file path in location
            for vuln in &mut line_vulnerabilities {
                vuln.location.file_path = file.file_path.to_path_buf();
            }

            vulnerabilities.extend(line_vulnerabilities);
        }

        Ok(vulnerabilities)
    }
}

impl Default for SessionManagementDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_session_management_javascript() {
        let detector = SessionManagementDetector::new();
        let content = r#"
            app.use(session({
                secret: 'weak',
                secure: false,
                httpOnly: false,
                maxAge: undefined
            }));

            req.session.password = user.password;
            const sessionId = Math.random().toString(36);
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            source: content.to_string().into(),
            tree: None,
            custom_ast: std::sync::Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 4);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("Weak session secret")));
    }

    #[tokio::test]
    async fn test_session_management_python() {
        let detector = SessionManagementDetector::new();
        let content = r#"
            SESSION_COOKIE_SECURE = False
            SESSION_COOKIE_HTTPONLY = False
            SESSION_COOKIE_AGE = None
            SECRET_KEY = 'dev'

            session['user_token'] = secret_token
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            source: content.to_string().into(),
            tree: None,
            custom_ast: std::sync::Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 4);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.severity == SecuritySeverity::Critical));
    }

    #[tokio::test]
    async fn test_session_management_rust() {
        let detector = SessionManagementDetector::new();
        let content = r#"
            let cookie = Cookie::build("session", session_id)
                .secure(false)
                .http_only(false)
                .finish();

            session.insert("password", user_password);
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            source: content.to_string().into(),
            tree: None,
            custom_ast: std::sync::Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 3);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("secure")));
    }

    #[test]
    fn test_remediation_generation() {
        let remediation = SessionManagementDetector::generate_remediation(
            &SessionIssueType::InsecureSessionConfig,
            "Express session configuration",
        );
        assert!(remediation.contains("secure flag"));
        assert!(remediation.contains("express-session"));
    }
}
