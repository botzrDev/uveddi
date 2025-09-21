//! Pattern-based scanner for OWASP vulnerabilities
//!
//! This scanner uses regex patterns and string matching to identify common
//! vulnerability patterns in source code across multiple languages.

use super::{Scanner, UnifiedScanResult};
use crate::analysis::detectors::security::owasp::types::{
    OwaspAnalysisPattern, OwaspCategory, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use regex::Regex;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};

/// A pattern match found in the code
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub matched_text: String,
    pub context_line: String,
    pub confidence: f64,
}

/// Result from pattern-based scanning
#[derive(Debug, Clone)]
pub struct PatternScanResult {
    pub matches: Vec<PatternMatch>,
    pub patterns_tested: usize,
    pub scan_duration_ms: u64,
}

/// Pattern-based vulnerability scanner
pub struct PatternScanner {
    patterns: HashMap<SourceLanguage, Vec<VulnerabilityPattern>>,
    compiled_patterns: HashMap<String, Regex>,
}

/// Internal representation of a vulnerability pattern
#[derive(Debug, Clone)]
struct VulnerabilityPattern {
    id: String,
    regex: String,
    category: OwaspCategory,
    issue_type: SecurityIssueType,
    severity: SecuritySeverity,
    confidence: f64,
    description: String,
    languages: Vec<SourceLanguage>,
}

impl PatternScanner {
    pub fn new() -> Result<Self, AnalysisError> {
        let mut scanner = Self {
            patterns: HashMap::new(),
            compiled_patterns: HashMap::new(),
        };

        scanner.initialize_patterns()?;
        scanner.compile_patterns()?;

        Ok(scanner)
    }

    /// Initialize vulnerability patterns for different languages
    fn initialize_patterns(&mut self) -> Result<(), AnalysisError> {
        let patterns = vec![
            // SQL Injection patterns
            VulnerabilityPattern {
                id: "sql_injection_concatenation".to_string(),
                regex: r#"(?i)(query|execute|sql)\s*\+.*["'][^"']*["']"#.to_string(),
                category: OwaspCategory::Injection,
                issue_type: SecurityIssueType::Injection,
                severity: SecuritySeverity::Critical,
                confidence: 0.8,
                description: "Potential SQL injection via string concatenation".to_string(),
                languages: vec![SourceLanguage::Python, SourceLanguage::JavaScript, SourceLanguage::TypeScript],
            },

            // Command Injection patterns
            VulnerabilityPattern {
                id: "command_injection".to_string(),
                regex: r#"(?i)(exec|system|popen|subprocess|os\.system)\s*\([^)]*\$"#.to_string(),
                category: OwaspCategory::Injection,
                issue_type: SecurityIssueType::Injection,
                severity: SecuritySeverity::Critical,
                confidence: 0.9,
                description: "Potential command injection vulnerability".to_string(),
                languages: vec![SourceLanguage::Python, SourceLanguage::JavaScript, SourceLanguage::TypeScript],
            },

            // XSS patterns
            VulnerabilityPattern {
                id: "xss_innerHTML".to_string(),
                regex: r#"\.innerHTML\s*=\s*[^;]*[+]"#.to_string(),
                category: OwaspCategory::Injection,
                issue_type: SecurityIssueType::CrossSiteScripting,
                severity: SecuritySeverity::High,
                confidence: 0.7,
                description: "Potential XSS via innerHTML concatenation".to_string(),
                languages: vec![SourceLanguage::JavaScript, SourceLanguage::TypeScript],
            },

            // Hardcoded secrets patterns
            VulnerabilityPattern {
                id: "hardcoded_api_key".to_string(),
                regex: r#"(?i)(api_key|apikey|secret|password|token)\s*[:=]\s*["'][a-zA-Z0-9+/]{20,}["']"#.to_string(),
                category: OwaspCategory::CryptographicFailures,
                issue_type: SecurityIssueType::HardcodedSecrets,
                severity: SecuritySeverity::High,
                confidence: 0.9,
                description: "Potential hardcoded API key or secret".to_string(),
                languages: vec![
                    SourceLanguage::Python,
                    SourceLanguage::JavaScript,
                    SourceLanguage::TypeScript,
                    SourceLanguage::Rust,
                ],
            },

            // Insecure randomness patterns
            VulnerabilityPattern {
                id: "weak_random".to_string(),
                regex: r#"(?i)(random\.random|math\.random|rand\(\))"#.to_string(),
                category: OwaspCategory::CryptographicFailures,
                issue_type: SecurityIssueType::InsecureRandomness,
                severity: SecuritySeverity::Medium,
                confidence: 0.6,
                description: "Use of weak random number generator".to_string(),
                languages: vec![
                    SourceLanguage::Python,
                    SourceLanguage::JavaScript,
                    SourceLanguage::TypeScript,
                ],
            },

            // Path traversal patterns
            VulnerabilityPattern {
                id: "path_traversal".to_string(),
                regex: r#"\.\./|\.\.\\|%2e%2e%2f|%2e%2e%5c"#.to_string(),
                category: OwaspCategory::BrokenAccessControl,
                issue_type: SecurityIssueType::PathTraversal,
                severity: SecuritySeverity::High,
                confidence: 0.8,
                description: "Potential path traversal vulnerability".to_string(),
                languages: vec![
                    SourceLanguage::Python,
                    SourceLanguage::JavaScript,
                    SourceLanguage::TypeScript,
                    SourceLanguage::Rust,
                ],
            },

            // Rust-specific patterns
            VulnerabilityPattern {
                id: "rust_unsafe_transmute".to_string(),
                regex: r#"std::mem::transmute|transmute\s*\("#.to_string(),
                category: OwaspCategory::SecurityMisconfiguration,
                issue_type: SecurityIssueType::SecurityMisconfiguration,
                severity: SecuritySeverity::High,
                confidence: 0.7,
                description: "Use of unsafe transmute operation".to_string(),
                languages: vec![SourceLanguage::Rust],
            },

            // Authentication bypass patterns
            VulnerabilityPattern {
                id: "auth_bypass".to_string(),
                regex: r#"(?i)(authenticated|authorized|logged_in)\s*=\s*(true|1|"true")"#.to_string(),
                category: OwaspCategory::AuthenticationFailures,
                issue_type: SecurityIssueType::AuthenticationFailures,
                severity: SecuritySeverity::Critical,
                confidence: 0.8,
                description: "Potential authentication bypass".to_string(),
                languages: vec![
                    SourceLanguage::Python,
                    SourceLanguage::JavaScript,
                    SourceLanguage::TypeScript,
                ],
            },
        ];

        // Group patterns by language
        for pattern in patterns {
            for language in &pattern.languages {
                self.patterns
                    .entry(*language)
                    .or_insert_with(Vec::new)
                    .push(pattern.clone());
            }
        }

        Ok(())
    }

    /// Compile all regex patterns for performance
    fn compile_patterns(&mut self) -> Result<(), AnalysisError> {
        for patterns in self.patterns.values() {
            for pattern in patterns {
                if !self.compiled_patterns.contains_key(&pattern.id) {
                    match Regex::new(&pattern.regex) {
                        Ok(compiled) => {
                            self.compiled_patterns.insert(pattern.id.clone(), compiled);
                        }
                        Err(e) => {
                            debug!("Failed to compile pattern {}: {}", pattern.id, e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Scan file content for pattern matches
    fn scan_patterns(&self, file: &ParsedFile) -> Result<PatternScanResult, AnalysisError> {
        let start_time = Instant::now();
        let mut matches = Vec::new();
        let mut patterns_tested = 0;

        if let Some(language_patterns) = self.patterns.get(&file.language) {
            for pattern in language_patterns {
                patterns_tested += 1;

                if let Some(regex) = self.compiled_patterns.get(&pattern.id) {
                    for (line_num, line) in file.source.lines().enumerate() {
                        for regex_match in regex.find_iter(line) {
                            matches.push(PatternMatch {
                                pattern_id: pattern.id.clone(),
                                line_number: line_num + 1,
                                column_start: regex_match.start(),
                                column_end: regex_match.end(),
                                matched_text: regex_match.as_str().to_string(),
                                context_line: line.to_string(),
                                confidence: pattern.confidence,
                            });
                        }
                    }
                }
            }
        }

        Ok(PatternScanResult {
            matches,
            patterns_tested,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Convert pattern matches to OWASP vulnerabilities
    fn convert_matches_to_vulnerabilities(
        &self,
        matches: Vec<PatternMatch>,
        file: &ParsedFile,
    ) -> Vec<OwaspVulnerability> {
        let mut vulnerabilities = Vec::new();

        for pattern_match in matches {
            // Find the pattern definition
            if let Some(language_patterns) = self.patterns.get(&file.language) {
                if let Some(pattern) = language_patterns
                    .iter()
                    .find(|p| p.id == pattern_match.pattern_id)
                {
                    let location = SecurityLocation::new(
                        file.file_path.as_ref().clone(),
                        pattern_match.line_number as i32,
                        pattern_match.line_number as i32,
                    )
                    .with_columns(
                        pattern_match.column_start as i32,
                        pattern_match.column_end as i32,
                    );

                    let vulnerability = OwaspVulnerability::new(
                        pattern.category.clone(),
                        pattern.issue_type.clone(),
                        format!("Pattern Match: {}", pattern.description),
                        format!(
                            "{}\nMatched text: '{}'",
                            pattern.description, pattern_match.matched_text
                        ),
                        location,
                    )
                    .with_confidence(pattern_match.confidence)
                    .with_severity(pattern.severity)
                    .with_metadata(
                        VulnerabilityMetadata::new()
                            .with_tags(vec![
                                "pattern-match".to_string(),
                                pattern.id.clone(),
                            ])
                    );

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        vulnerabilities
    }
}

#[async_trait::async_trait]
impl Scanner for PatternScanner {
    async fn scan(&self, file: &ParsedFile) -> Result<UnifiedScanResult, AnalysisError> {
        let start_time = Instant::now();
        info!("Starting pattern-based scan for: {}", file.file_path.display());

        let scan_result = self.scan_patterns(file)?;
        let vulnerabilities = self.convert_matches_to_vulnerabilities(scan_result.matches, file);

        let scan_duration = start_time.elapsed().as_millis() as u64;

        let mut metadata = HashMap::new();
        metadata.insert("scanner_type".to_string(), serde_json::Value::String("pattern".to_string()));
        metadata.insert("patterns_tested".to_string(), serde_json::Value::Number(scan_result.patterns_tested.into()));
        metadata.insert("language".to_string(), serde_json::Value::String(file.language.to_string()));

        info!(
            "Pattern scan completed: {} vulnerabilities found from {} patterns in {}ms",
            vulnerabilities.len(),
            scan_result.patterns_tested,
            scan_duration
        );

        Ok(UnifiedScanResult {
            vulnerabilities,
            scanner_metadata: serde_json::to_value(&metadata).unwrap_or_default(),
            scan_duration_ms: scan_duration,
        })
    }

    fn name(&self) -> &'static str {
        "pattern_scanner"
    }

    fn supported_languages(&self) -> Vec<SourceLanguage> {
        self.patterns.keys().cloned().collect()
    }

    fn detectable_categories(&self) -> Vec<OwaspCategory> {
        vec![
            OwaspCategory::Injection,
            OwaspCategory::CryptographicFailures,
            OwaspCategory::BrokenAccessControl,
            OwaspCategory::AuthenticationFailures,
            OwaspCategory::SecurityMisconfiguration,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_pattern_scanner_creation() {
        let scanner = PatternScanner::new();
        assert!(scanner.is_ok());

        let scanner = scanner.unwrap();
        assert_eq!(scanner.name(), "pattern_scanner");
        assert!(!scanner.supported_languages().is_empty());
    }

    #[tokio::test]
    async fn test_sql_injection_detection() {
        let scanner = PatternScanner::new().unwrap();
        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            source: Arc::new("query = \"SELECT * FROM users WHERE id = \" + user_input".to_string()),
            tree: None,
            custom_ast: Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        let result = scanner.scan(&file).await.unwrap();
        assert!(!result.vulnerabilities.is_empty());
        assert_eq!(result.vulnerabilities[0].category, OwaspCategory::Injection);
    }

    #[tokio::test]
    async fn test_hardcoded_secret_detection() {
        let scanner = PatternScanner::new().unwrap();
        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            source: Arc::new("const API_KEY = \"sk_12345abcdef67890ghijklmnop\";".to_string()),
            tree: None,
            custom_ast: Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        let result = scanner.scan(&file).await.unwrap();
        assert!(!result.vulnerabilities.is_empty());
        assert_eq!(result.vulnerabilities[0].category, OwaspCategory::CryptographicFailures);
    }

    #[test]
    fn test_pattern_compilation() {
        let scanner = PatternScanner::new().unwrap();
        assert!(!scanner.compiled_patterns.is_empty());
    }
}
