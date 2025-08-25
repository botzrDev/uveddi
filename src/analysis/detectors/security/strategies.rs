//! Detection strategies for different types of security vulnerabilities
//!
//! This module implements specialized detection strategies that work together
//! to provide comprehensive security analysis across multiple domains.

use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Deterministic pattern matcher for known vulnerability patterns
pub struct DeterministicPatternMatcher {
    patterns: HashMap<SourceLanguage, Vec<VulnerabilityPattern>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub description: String,
    pub remediation: Option<String>,
}

impl DeterministicPatternMatcher {
    pub fn new() -> Self {
        let mut matcher = Self {
            patterns: HashMap::new(),
        };
        matcher.initialize_patterns();
        matcher
    }

    fn initialize_patterns(&mut self) {
        // Rust patterns
        self.add_rust_patterns();
        // Python patterns
        self.add_python_patterns();
        // JavaScript patterns
        self.add_javascript_patterns();
    }

    fn add_rust_patterns(&mut self) {
        let rust_patterns = vec![
            VulnerabilityPattern {
                pattern: "std::ptr::read_volatile".to_string(),
                vulnerability_type: SecurityIssueType::UseAfterFree,
                severity: SecuritySeverity::High,
                confidence: 0.7,
                description: "Use of unsafe memory operation".to_string(),
                remediation: Some("Consider using safe alternatives".to_string()),
            },
            VulnerabilityPattern {
                pattern: ".unwrap()".to_string(),
                vulnerability_type: SecurityIssueType::ImproperErrorHandling,
                severity: SecuritySeverity::Medium,
                confidence: 0.4,
                description: "Potential panic from unwrap()".to_string(),
                remediation: Some("Use proper error handling with match or if let".to_string()),
            },
        ];

        self.patterns.insert(SourceLanguage::Rust, rust_patterns);
    }

    fn add_python_patterns(&mut self) {
        let python_patterns = vec![
            VulnerabilityPattern {
                pattern: "pickle.load".to_string(),
                vulnerability_type: SecurityIssueType::DeserializationVulnerabilities,
                severity: SecuritySeverity::High,
                confidence: 0.8,
                description: "Unsafe deserialization with pickle".to_string(),
                remediation: Some("Use safe serialization formats like JSON".to_string()),
            },
            VulnerabilityPattern {
                pattern: "shell=True".to_string(),
                vulnerability_type: SecurityIssueType::Injection,
                severity: SecuritySeverity::Critical,
                confidence: 0.9,
                description: "Command injection via shell=True".to_string(),
                remediation: Some("Avoid shell=True or sanitize inputs".to_string()),
            },
        ];

        self.patterns
            .insert(SourceLanguage::Python, python_patterns);
    }

    fn add_javascript_patterns(&mut self) {
        let js_patterns = vec![VulnerabilityPattern {
            pattern: "document.write(".to_string(),
            vulnerability_type: SecurityIssueType::CrossSiteScripting,
            severity: SecuritySeverity::High,
            confidence: 0.7,
            description: "Potential XSS via document.write".to_string(),
            remediation: Some("Use safe DOM manipulation methods".to_string()),
        }];

        self.patterns
            .insert(SourceLanguage::JavaScript, js_patterns.clone());
        self.patterns
            .insert(SourceLanguage::TypeScript, js_patterns);
    }

    pub async fn analyze(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!(
            "Running deterministic pattern matching on: {}",
            file.file_path.display()
        );

        let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
            AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
        })?;

        let mut issues = Vec::new();

        if let Some(patterns) = self.patterns.get(&file.language) {
            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if line.contains(&pattern.pattern) {
                        let location = SecurityLocation::new(
                            file.file_path.as_ref().to_path_buf(),
                            line_num as i32 + 1,
                            line_num as i32 + 1,
                        );

                        let issue = SecurityIssue::new(
                            pattern.vulnerability_type.clone(),
                            crate::analysis::detectors::security::types::VulnerabilityType::Static,
                            format!("{} Detected", pattern.vulnerability_type.to_string()),
                            pattern.description.clone(),
                            location,
                        )
                        .with_confidence(pattern.confidence)
                        .with_severity(pattern.severity)
                        .with_language(file.language)
                        .with_remediation(pattern.remediation.clone().unwrap_or_default())
                        .with_detector("DeterministicPatternMatcher".to_string());

                        issues.push(issue);
                    }
                }
            }
        }

        info!(
            "Deterministic pattern matching found {} issues",
            issues.len()
        );
        Ok(issues)
    }
}

/// Configuration file analyzer for security misconfigurations
pub struct ConfigFileAnalyzer {
    config_patterns: HashMap<String, Vec<ConfigSecurityPattern>>,
}

#[derive(Debug, Clone)]
pub struct ConfigSecurityPattern {
    pub key_pattern: String,
    pub dangerous_values: Vec<String>,
    pub vulnerability_type: SecurityIssueType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub remediation: String,
}

impl ConfigFileAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            config_patterns: HashMap::new(),
        };
        analyzer.initialize_config_patterns();
        analyzer
    }

    fn initialize_config_patterns(&mut self) {
        // Django settings patterns
        let django_patterns = vec![
            ConfigSecurityPattern {
                key_pattern: "DEBUG".to_string(),
                dangerous_values: vec!["True".to_string(), "true".to_string()],
                vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
                severity: SecuritySeverity::High,
                description: "DEBUG mode enabled in production".to_string(),
                remediation: "Set DEBUG = False in production".to_string(),
            },
            ConfigSecurityPattern {
                key_pattern: "SECRET_KEY".to_string(),
                dangerous_values: vec!["django-insecure".to_string(), "changeme".to_string()],
                vulnerability_type: SecurityIssueType::HardcodedSecrets,
                severity: SecuritySeverity::Critical,
                description: "Insecure or default SECRET_KEY".to_string(),
                remediation: "Use a strong, randomly generated SECRET_KEY".to_string(),
            },
        ];

        self.config_patterns
            .insert("settings.py".to_string(), django_patterns);

        // Add more configuration patterns for other frameworks
        self.add_docker_patterns();
        self.add_nginx_patterns();
    }

    fn add_docker_patterns(&mut self) {
        let docker_patterns = vec![ConfigSecurityPattern {
            key_pattern: "USER".to_string(),
            dangerous_values: vec!["root".to_string()],
            vulnerability_type: SecurityIssueType::PrivilegeEscalation,
            severity: SecuritySeverity::Medium,
            description: "Running container as root user".to_string(),
            remediation: "Create and use a non-root user".to_string(),
        }];

        self.config_patterns
            .insert("Dockerfile".to_string(), docker_patterns);
    }

    fn add_nginx_patterns(&mut self) {
        let nginx_patterns = vec![ConfigSecurityPattern {
            key_pattern: "server_tokens".to_string(),
            dangerous_values: vec!["on".to_string()],
            vulnerability_type: SecurityIssueType::SecurityMisconfiguration,
            severity: SecuritySeverity::Low,
            description: "Server tokens enabled, revealing version information".to_string(),
            remediation: "Set server_tokens off".to_string(),
        }];

        self.config_patterns
            .insert("nginx.conf".to_string(), nginx_patterns);
    }

    pub async fn analyze(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!(
            "Running config file analysis on: {}",
            file.file_path.display()
        );

        let filename = file
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
            AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
        })?;

        let mut issues = Vec::new();

        if let Some(patterns) = self.config_patterns.get(filename) {
            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if line.contains(&pattern.key_pattern) {
                        // Check if any dangerous values are present
                        for dangerous_value in &pattern.dangerous_values {
                            if line.contains(dangerous_value) {
                                let location = SecurityLocation::new(
                                    file.file_path.as_ref().to_path_buf(),
                                    line_num as i32 + 1,
                                    line_num as i32 + 1,
                                );

                                let issue = SecurityIssue::new(
                                    pattern.vulnerability_type.clone(),
                                    crate::analysis::detectors::security::types::VulnerabilityType::Configuration,
                                    format!("Configuration Security Issue: {}", pattern.key_pattern),
                                    pattern.description.clone(),
                                    location,
                                )
                                .with_confidence(0.8) // High confidence for config issues
                                .with_severity(pattern.severity)
                                .with_remediation(pattern.remediation.clone())
                                .with_detector("ConfigFileAnalyzer".to_string());

                                issues.push(issue);
                                break; // Only report once per line
                            }
                        }
                    }
                }
            }
        }

        info!("Config file analysis found {} issues", issues.len());
        Ok(issues)
    }
}

/// Software Composition Analysis (SCA) for vulnerable dependencies
pub struct SoftwareCompositionAnalyzer {
    // This would integrate with vulnerability databases
}

impl SoftwareCompositionAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn analyze(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!("Running SCA analysis on: {}", file.file_path.display());

        let filename = file
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let mut issues = Vec::new();

        match filename {
            "Cargo.toml" => {
                issues.extend(self.analyze_cargo_toml(file).await?);
            }
            "package.json" => {
                issues.extend(self.analyze_package_json(file).await?);
            }
            "requirements.txt" => {
                issues.extend(self.analyze_requirements_txt(file).await?);
            }
            _ => {
                // Not a dependency file
            }
        }

        info!("SCA analysis found {} issues", issues.len());
        Ok(issues)
    }

    async fn analyze_cargo_toml(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        // TODO: Parse Cargo.toml and check dependencies against RustSec database
        Ok(Vec::new())
    }

    async fn analyze_package_json(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        // TODO: Parse package.json and check dependencies against npm audit
        Ok(Vec::new())
    }

    async fn analyze_requirements_txt(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        // TODO: Parse requirements.txt and check dependencies against safety database
        Ok(Vec::new())
    }
}

/// Vulnerability correlation engine that maps security issues to architectural anti-patterns
pub struct VulnerabilityCorrelationEngine {
    correlation_rules: HashMap<SecurityIssueType, Vec<ArchitecturalCorrelationRule>>,
}

#[derive(Debug, Clone)]
pub struct ArchitecturalCorrelationRule {
    pub anti_pattern: String,
    pub correlation_strength: f64,
    pub amplification_factor: f64,
    pub explanation: String,
}

impl VulnerabilityCorrelationEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            correlation_rules: HashMap::new(),
        };
        engine.initialize_correlation_rules();
        engine
    }

    fn initialize_correlation_rules(&mut self) {
        // God Object correlations
        self.correlation_rules.insert(
            SecurityIssueType::BrokenAccessControl,
            vec![
                ArchitecturalCorrelationRule {
                    anti_pattern: "God Object".to_string(),
                    correlation_strength: 0.8,
                    amplification_factor: 1.5,
                    explanation: "God Objects centralize too much logic, making access control complex and error-prone".to_string(),
                },
                ArchitecturalCorrelationRule {
                    anti_pattern: "Tight Coupling".to_string(),
                    correlation_strength: 0.7,
                    amplification_factor: 1.3,
                    explanation: "Tight coupling spreads authorization logic across entangled components".to_string(),
                },
            ],
        );

        // Injection correlations
        self.correlation_rules.insert(
            SecurityIssueType::Injection,
            vec![
                ArchitecturalCorrelationRule {
                    anti_pattern: "Leaky Abstraction".to_string(),
                    correlation_strength: 0.9,
                    amplification_factor: 2.0,
                    explanation: "Leaky abstractions expose implementation details that aid attackers in crafting payloads".to_string(),
                },
            ],
        );

        // Add more correlation rules...
    }

    pub fn correlate_issue(
        &self,
        security_issue: &SecurityIssue,
        detected_anti_patterns: &[String],
    ) -> Vec<SecurityCorrelation> {
        let mut correlations = Vec::new();

        if let Some(rules) = self.correlation_rules.get(&security_issue.issue_type) {
            for rule in rules {
                if detected_anti_patterns.contains(&rule.anti_pattern) {
                    correlations.push(SecurityCorrelation {
                        security_issue_id: security_issue.id.clone().unwrap_or_default(),
                        anti_pattern: rule.anti_pattern.clone(),
                        correlation_strength: rule.correlation_strength,
                        amplification_factor: rule.amplification_factor,
                        explanation: rule.explanation.clone(),
                    });
                }
            }
        }

        correlations
    }
}

#[derive(Debug, Clone)]
pub struct SecurityCorrelation {
    pub security_issue_id: String,
    pub anti_pattern: String,
    pub correlation_strength: f64,
    pub amplification_factor: f64,
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::SourceLanguage;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_deterministic_pattern_matcher() {
        let matcher = DeterministicPatternMatcher::new();

        // Create a test file with a vulnerable pattern
        use crate::analysis::cache::wrappers::ArchivableSystemTime;
        use std::sync::Arc;

        let test_file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            tree: None,
            source: Arc::new(String::new()),
            custom_ast: Arc::new(None),
            modified_at: ArchivableSystemTime::now(),
        };

        // This would need a mock file system or actual test file
        // For now, we just test that the matcher was created successfully
        assert!(!matcher.patterns.is_empty());
    }

    #[test]
    fn test_vulnerability_correlation_engine() {
        let engine = VulnerabilityCorrelationEngine::new();

        let security_issue = SecurityIssue::new(
            SecurityIssueType::BrokenAccessControl,
            crate::analysis::detectors::security::types::VulnerabilityType::Static,
            "Access Control Issue".to_string(),
            "Test issue".to_string(),
            crate::analysis::detectors::security::types::SecurityLocation::new(
                PathBuf::from("test.rs"),
                1,
                1,
            ),
        );

        let anti_patterns = vec!["God Object".to_string()];
        let correlations = engine.correlate_issue(&security_issue, &anti_patterns);

        assert!(!correlations.is_empty());
        assert_eq!(correlations[0].anti_pattern, "God Object");
        assert_eq!(correlations[0].correlation_strength, 0.8);
    }

    #[test]
    fn test_config_file_analyzer() {
        let analyzer = ConfigFileAnalyzer::new();

        // Check that patterns were loaded
        assert!(analyzer.config_patterns.contains_key("settings.py"));
        assert!(analyzer.config_patterns.contains_key("Dockerfile"));
    }

    #[test]
    fn test_vulnerability_pattern() {
        let pattern = VulnerabilityPattern {
            pattern: "eval(".to_string(),
            vulnerability_type: SecurityIssueType::Injection,
            severity: SecuritySeverity::Critical,
            confidence: 0.9,
            description: "Code injection via eval".to_string(),
            remediation: Some("Avoid using eval with user input".to_string()),
        };

        assert_eq!(pattern.vulnerability_type, SecurityIssueType::Injection);
        assert_eq!(pattern.severity, SecuritySeverity::Critical);
    }
}
