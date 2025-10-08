//! Rust-specific OWASP vulnerability analysis
//!
//! This module provides Rust-specific security analysis for OWASP Top 10 vulnerabilities.

use crate::analysis::detectors::security::owasp::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// Rust-specific OWASP analyzer
pub struct RustOwaspAnalyzer {
    patterns: Vec<RustSecurityPattern>,
}

#[derive(Clone, Debug)]
struct RustSecurityPattern {
    name: String,
    pattern: String,
    category: OwaspCategory,
    severity: SecuritySeverity,
    description: String,
}

impl RustOwaspAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> Vec<RustSecurityPattern> {
        vec![
            RustSecurityPattern {
                name: "Unsafe Block".to_string(),
                pattern: "unsafe {".to_string(),
                category: OwaspCategory::InsecureDesign,
                severity: SecuritySeverity::Medium,
                description: "Use of unsafe block may bypass Rust's memory safety guarantees"
                    .to_string(),
            },
            RustSecurityPattern {
                name: "Raw Pointer Dereference".to_string(),
                pattern: "* as *const".to_string(),
                category: OwaspCategory::InsecureDesign,
                severity: SecuritySeverity::High,
                description: "Raw pointer usage requires careful handling".to_string(),
            },
            RustSecurityPattern {
                name: "SQL String Formatting".to_string(),
                pattern: "format!(\"SELECT".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::Critical,
                description: "SQL query construction with format! is vulnerable to injection"
                    .to_string(),
            },
            RustSecurityPattern {
                name: "Command Execution".to_string(),
                pattern: "Command::new(".to_string(),
                category: OwaspCategory::Injection,
                severity: SecuritySeverity::High,
                description: "Process spawning requires input validation".to_string(),
            },
            RustSecurityPattern {
                name: "MD5 Usage".to_string(),
                pattern: "md5::".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::High,
                description: "MD5 is cryptographically broken".to_string(),
            },
            RustSecurityPattern {
                name: "SHA1 Usage".to_string(),
                pattern: "sha1::".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::Medium,
                description: "SHA1 is deprecated for security purposes".to_string(),
            },
            RustSecurityPattern {
                name: "Hardcoded Secret".to_string(),
                pattern: "const SECRET".to_string(),
                category: OwaspCategory::CryptographicFailures,
                severity: SecuritySeverity::Critical,
                description: "Hardcoded secrets should use environment variables".to_string(),
            },
            RustSecurityPattern {
                name: "Panic in Production".to_string(),
                pattern: "panic!(".to_string(),
                category: OwaspCategory::SecurityMisconfiguration,
                severity: SecuritySeverity::Low,
                description: "Explicit panics can cause DoS in production".to_string(),
            },
            RustSecurityPattern {
                name: "Debug Print".to_string(),
                pattern: "dbg!(".to_string(),
                category: OwaspCategory::LoggingFailures,
                severity: SecuritySeverity::Low,
                description: "Debug macros should not be in production code".to_string(),
            },
        ]
    }

    pub async fn analyze(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        if file.language != SourceLanguage::Rust {
            return Ok(vec![]);
        }

        let mut vulnerabilities = Vec::new();
        let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
            AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
        })?;

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if line.contains(&pattern.pattern) {
                    let location = SecurityLocation::new(
                        file.file_path.as_ref().to_path_buf(),
                        line_num as i32 + 1,
                        line_num as i32 + 1,
                    );

                    let vulnerability = OwaspVulnerability::new(
                        pattern.category.clone(),
                        SecurityIssueType::from(pattern.category.clone()),
                        pattern.name.clone(),
                        pattern.description.clone(),
                        location,
                    )
                    .with_severity(pattern.severity)
                    .with_confidence(0.7);

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        Ok(vulnerabilities)
    }

    /// Get Rust-specific security recommendations
    pub fn get_recommendations() -> HashMap<OwaspCategory, Vec<String>> {
        let mut recommendations = HashMap::new();

        recommendations.insert(
            OwaspCategory::Injection,
            vec![
                "Use parameterized queries with diesel or sqlx".to_string(),
                "Validate and escape shell commands".to_string(),
                "Use std::process::Command builder pattern".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::CryptographicFailures,
            vec![
                "Use ring or rustcrypto for cryptography".to_string(),
                "Store secrets in environment variables".to_string(),
                "Use constant-time comparison for sensitive data".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::InsecureDesign,
            vec![
                "Minimize unsafe block usage".to_string(),
                "Use safe abstractions when possible".to_string(),
                "Document safety invariants".to_string(),
            ],
        );

        recommendations.insert(
            OwaspCategory::BrokenAccessControl,
            vec![
                "Implement proper authorization checks".to_string(),
                "Use type system for access control".to_string(),
                "Validate all user inputs".to_string(),
            ],
        );

        recommendations
    }

    /// Check if a crate is known to be vulnerable
    pub fn check_vulnerable_crates(crate_name: &str, version: &str) -> Option<String> {
        let vulnerable_crates = vec![
            ("rand", "0.3", "Use rand 0.8+ for cryptographic randomness"),
            (
                "openssl",
                "0.9",
                "Update to openssl 0.10+ for security fixes",
            ),
            ("time", "0.1", "time 0.1 has segfault issues, use 0.3+"),
        ];

        for (name, vuln_version, message) in vulnerable_crates {
            if crate_name == name && version.starts_with(vuln_version) {
                return Some(message.to_string());
            }
        }

        None
    }
}

impl Default for RustOwaspAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::compatibility_shim::ParsedFileCompat;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_rust_patterns() {
        let analyzer = RustOwaspAnalyzer::new();

        let file = ParsedFileCompat::new(
            PathBuf::from("test.rs"),
            SourceLanguage::Rust,
            "unsafe { *ptr }".to_string(),
        )
        .to_tree_sitter();

        std::fs::write("test.rs", "unsafe { *ptr }").unwrap();
        let vulnerabilities = analyzer.analyze(&file).await.unwrap();
        std::fs::remove_file("test.rs").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::InsecureDesign);
    }

    #[test]
    fn test_vulnerable_crates() {
        assert!(RustOwaspAnalyzer::check_vulnerable_crates("rand", "0.3.1").is_some());
        assert!(RustOwaspAnalyzer::check_vulnerable_crates("rand", "0.8.0").is_none());
    }
}
