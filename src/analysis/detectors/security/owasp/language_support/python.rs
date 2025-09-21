//! Python-specific OWASP vulnerability analysis
//!
//! This module provides Python-specific security analysis for OWASP Top 10 vulnerabilities.

use crate::analysis::detectors::security::owasp::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// Python-specific OWASP analyzer
pub struct PythonOwaspAnalyzer {
    patterns: Vec<PythonSecurityPattern>,
}

#[derive(Clone, Debug)]
struct PythonSecurityPattern {
    name: String,
    pattern: String,
    category: OwaspCategory,
    severity: SecuritySeverity,
    description: String,
}

impl PythonOwaspAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> Vec<PythonSecurityPattern> {
        vec![
            PythonSecurityPattern {
                name: "Eval Usage".to_string(),
                pattern: "eval(".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::Critical,
                description: "eval() executes arbitrary code and is highly dangerous".to_string(),
            },
            PythonSecurityPattern {
                name: "Exec Usage".to_string(),
                pattern: "exec(".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::Critical,
                description: "exec() executes arbitrary code and should be avoided".to_string(),
            },
            PythonSecurityPattern {
                name: "OS System Call".to_string(),
                pattern: "os.system(".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::High,
                description: "os.system() is vulnerable to command injection".to_string(),
            },
            PythonSecurityPattern {
                name: "SQL F-String".to_string(),
                pattern: "execute(f\"".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::Critical,
                description: "F-strings in SQL queries enable SQL injection".to_string(),
            },
            PythonSecurityPattern {
                name: "Pickle Load".to_string(),
                pattern: "pickle.load".to_string(),
                category: OwaspCategory::SoftwareDataIntegrityFailures,
                severity: SecuritySeverity::Critical,
                description: "Pickle can execute arbitrary code during deserialization".to_string(),
            },
            PythonSecurityPattern {
                name: "YAML Load".to_string(),
                pattern: "yaml.load(".to_string(),
                category: OwaspCategory::SoftwareDataIntegrityFailures,
                severity: SecuritySeverity::High,
                description: "yaml.load() is unsafe, use yaml.safe_load()".to_string(),
            },
            PythonSecurityPattern {
                name: "MD5 Hash".to_string(),
                pattern: "hashlib.md5(".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::High,
                description: "MD5 is cryptographically broken".to_string(),
            },
            PythonSecurityPattern {
                name: "Random for Crypto".to_string(),
                pattern: "random.random()".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::High,
                description: "Use secrets module for cryptographic randomness".to_string(),
            },
            PythonSecurityPattern {
                name: "Debug Mode".to_string(),
                pattern: "debug=True".to_string(),
                category: OwaspCategory::SecurityMisconfiguration,
                severity: SecuritySeverity::Medium,
                description: "Debug mode exposes sensitive information".to_string(),
            },
            PythonSecurityPattern {
                name: "Assert Statement".to_string(),
                pattern: "assert ".to_string(),
                category: OwaspCategory::InsecureDesign,
                severity: SecuritySeverity::Low,
                description: "Assertions are removed in optimized code".to_string(),
            },
        ]
    }

    pub async fn analyze(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        if file.language != SourceLanguage::Python {
            return Ok(vec![]);
        }

        let mut vulnerabilities = Vec::new();
        let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
            AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
        })?;

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if line.contains(&pattern.pattern) && !Self::is_safe_usage(line, &pattern.pattern) {
                    let location = SecurityLocation::new(
                        file.file_path.as_ref().to_path_buf(),
                        line_num as i32 + 1,
                        line_num as i32 + 1,
                    );

                    let vulnerability = OwaspVulnerability::new(
                        pattern.category,
                        SecurityIssueType::from(pattern.category),
                        pattern.name.clone(),
                        pattern.description.clone(),
                        location,
                    )
                    .with_severity(pattern.severity)
                    .with_confidence(0.75);

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        Ok(vulnerabilities)
    }

    fn is_safe_usage(line: &str, pattern: &str) -> bool {
        // Check for safe alternatives
        match pattern {
            "yaml.load(" => line.contains("yaml.safe_load("),
            "random.random()" => line.contains("secrets.") || line.contains("# nosec"),
            _ => line.contains("# nosec") || line.contains("# noqa"),
        }
    }

    /// Get Python-specific security recommendations
    pub fn get_recommendations() -> HashMap<OwaspCategory, Vec<String>> {
        let mut recommendations = HashMap::new();

        recommendations.insert(
            OwaspCategory::Injection,
            vec![
                "Use parameterized queries with psycopg2 or SQLAlchemy".to_string(),
                "Use subprocess with shell=False".to_string(),
                "Never use eval() or exec() with user input".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::CryptographicFailures,
            vec![
                "Use cryptography library for encryption".to_string(),
                "Use secrets module for secure randomness".to_string(),
                "Use bcrypt or argon2 for password hashing".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::SoftwareDataIntegrityFailures,
            vec![
                "Use JSON instead of pickle when possible".to_string(),
                "Always use yaml.safe_load() instead of yaml.load()".to_string(),
                "Validate deserialized data".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::SecurityMisconfiguration,
            vec![
                "Set DEBUG=False in production".to_string(),
                "Use python-dotenv for configuration".to_string(),
                "Enable security headers in web frameworks".to_string(),
            ],
        );

        recommendations
    }

    /// Check if a Python package is known to be vulnerable
    pub fn check_vulnerable_packages(package: &str, version: &str) -> Option<String> {
        let vulnerable_packages = vec![
            ("django", "2.2", "Django < 2.2.28 has security vulnerabilities"),
            ("flask", "1.0", "Flask < 1.0.4 has security issues"),
            ("requests", "2.5", "Requests < 2.5.2 has SSL verification issues"),
            ("pyyaml", "5.0", "PyYAML < 5.1 has unsafe load vulnerability"),
        ];

        for (name, vuln_version, message) in vulnerable_packages {
            if package == name && version.starts_with(vuln_version) {
                return Some(message.to_string());
            }
        }

        None
    }

    /// Detect framework-specific issues
    pub fn check_framework_issues(content: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Django specific
        if content.contains("django") {
            if content.contains("csrf_exempt") {
                issues.push("CSRF protection disabled".to_string());
            }
            if content.contains("ALLOWED_HOSTS = []") {
                issues.push("ALLOWED_HOSTS not configured".to_string());
            }
        }

        // Flask specific
        if content.contains("flask") {
            if content.contains("app.run(debug=True)") {
                issues.push("Flask debug mode enabled".to_string());
            }
            if !content.contains("SECRET_KEY") {
                issues.push("Flask SECRET_KEY may not be set".to_string());
            }
        }

        issues
    }
}

impl Default for PythonOwaspAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_python_patterns() {
        let analyzer = PythonOwaspAnalyzer::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: "eval(user_input)".to_string(),
            tree: None,
        };

        std::fs::write("test.py", "eval(user_input)").unwrap();
        let vulnerabilities = analyzer.analyze(&file).await.unwrap();
        std::fs::remove_file("test.py").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::Injection);
    }

    #[test]
    fn test_vulnerable_packages() {
        assert!(PythonOwaspAnalyzer::check_vulnerable_packages("django", "2.2.1").is_some());
        assert!(PythonOwaspAnalyzer::check_vulnerable_packages("django", "3.2.0").is_none());
    }
}