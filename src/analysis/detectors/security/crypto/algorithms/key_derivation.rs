//! Key Derivation Function Analysis
//!
//! This module analyzes key derivation functions (KDFs) for security issues,
//! including weak algorithms, insufficient iteration counts, and improper usage.

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use regex::Regex;
use std::collections::HashMap;

/// Analyzer for key derivation functions
pub struct KeyDerivationAnalyzer {
    weak_kdfs: HashMap<SourceLanguage, Vec<WeakKdfPattern>>,
    iteration_patterns: HashMap<SourceLanguage, Vec<IterationPattern>>,
    salt_patterns: HashMap<SourceLanguage, Vec<SaltPattern>>,
}

/// Pattern for detecting weak key derivation functions
#[derive(Debug, Clone)]
pub struct WeakKdfPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub replacement: String,
    pub kdf_type: KdfType,
}

/// Pattern for detecting insufficient iteration counts
#[derive(Debug, Clone)]
pub struct IterationPattern {
    pub name: String,
    pub pattern: String,
    pub min_iterations: u32,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
}

/// Pattern for detecting salt-related issues
#[derive(Debug, Clone)]
pub struct SaltPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub recommendation: String,
}

/// Types of key derivation functions
#[derive(Debug, Clone, PartialEq)]
pub enum KdfType {
    PBKDF2,
    Scrypt,
    Argon2,
    HKDF,
    BCrypt,
    WeakKdf,
}

impl KeyDerivationAnalyzer {
    /// Create a new key derivation analyzer
    pub fn new() -> Self {
        Self {
            weak_kdfs: Self::initialize_weak_kdfs(),
            iteration_patterns: Self::initialize_iteration_patterns(),
            salt_patterns: Self::initialize_salt_patterns(),
        }
    }

    /// Analyze content for key derivation issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Check for weak KDFs
        if let Some(weak_patterns) = self.weak_kdfs.get(language) {
            findings.extend(self.detect_weak_kdfs(content, weak_patterns)?);
        }

        // Check for insufficient iterations
        if let Some(iter_patterns) = self.iteration_patterns.get(language) {
            findings.extend(self.detect_weak_iterations(content, iter_patterns)?);
        }

        // Check for salt issues
        if let Some(salt_patterns) = self.salt_patterns.get(language) {
            findings.extend(self.detect_salt_issues(content, salt_patterns)?);
        }

        Ok(findings)
    }

    /// Initialize weak KDF patterns for each language
    fn initialize_weak_kdfs() -> HashMap<SourceLanguage, Vec<WeakKdfPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            WeakKdfPattern {
                name: "Simple Hash KDF".to_string(),
                pattern: r"(?i)sha256\s*\(\s*password.*salt".to_string(),
                description: "Using simple hash function for key derivation is insecure".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                replacement: "PBKDF2, Argon2, or scrypt".to_string(),
                kdf_type: KdfType::WeakKdf,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            WeakKdfPattern {
                name: "Simple Hash KDF".to_string(),
                pattern: r"hashlib\.(sha256|md5|sha1)\s*\(\s*password".to_string(),
                description: "Fast hash functions are inappropriate for key derivation".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                replacement: "Use PBKDF2, Argon2, or scrypt".to_string(),
                kdf_type: KdfType::WeakKdf,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            WeakKdfPattern {
                name: "Simple Hash KDF".to_string(),
                pattern: r"crypto\.createHash.*password".to_string(),
                description: "Using createHash for password derivation is insecure".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                replacement: "Use crypto.pbkdf2Sync or a dedicated library".to_string(),
                kdf_type: KdfType::WeakKdf,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Initialize iteration count patterns
    fn initialize_iteration_patterns() -> HashMap<SourceLanguage, Vec<IterationPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            IterationPattern {
                name: "Low PBKDF2 Iterations".to_string(),
                pattern: r"pbkdf2.*[,\s]([1-9]\d{0,3}|[1-9]\d{4})[,\s)]".to_string(), // < 100,000
                min_iterations: 100_000,
                description: "PBKDF2 iteration count is too low".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.8,
            },
            IterationPattern {
                name: "Low Argon2 Iterations".to_string(),
                pattern: r"argon2.*iterations[:\s=]*([1-9]|[12]\d|3[01])[,\s)]".to_string(), // < 32
                min_iterations: 32,
                description: "Argon2 iteration count should be at least 32".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.8,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            IterationPattern {
                name: "Low PBKDF2 Iterations".to_string(),
                pattern: r"pbkdf2_hmac\([^,]+,[^,]+,\s*(\d{1,4}|[1-9]\d{4})\s*[,)]".to_string(), // < 100,000
                min_iterations: 100_000,
                description: "PBKDF2 iteration count below 100,000 is insufficient".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.9,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            IterationPattern {
                name: "Low PBKDF2 Iterations".to_string(),
                pattern: r"pbkdf2Sync\([^,]+,[^,]+,\s*(\d{1,4}|[1-9]\d{4})\s*[,)]".to_string(),
                min_iterations: 100_000,
                description: "PBKDF2 iteration count is too low for security".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.9,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Initialize salt patterns
    fn initialize_salt_patterns() -> HashMap<SourceLanguage, Vec<SaltPattern>> {
        let mut patterns = HashMap::new();

        // Common patterns for all languages
        let common_patterns = vec![
            SaltPattern {
                name: "Missing Salt".to_string(),
                pattern: r"(?i)pbkdf2.*password[^,]*\)".to_string(),
                description: "Key derivation function used without salt".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.7,
                recommendation: "Always use a unique random salt".to_string(),
            },
            SaltPattern {
                name: "Short Salt".to_string(),
                pattern: r#"salt\s*[:=]\s*["'][^"']{1,7}["']"#.to_string(),
                description: "Salt is too short (minimum 16 bytes recommended)".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.8,
                recommendation: "Use salts of at least 16 random bytes".to_string(),
            },
            SaltPattern {
                name: "Hardcoded Salt".to_string(),
                pattern: r#"salt\s*[:=]\s*["'][^"']{8,}["']"#.to_string(),
                description: "Salt appears to be hardcoded".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.6,
                recommendation: "Generate unique random salts for each operation".to_string(),
            },
        ];

        patterns.insert(SourceLanguage::Rust, common_patterns.clone());
        patterns.insert(SourceLanguage::Python, common_patterns.clone());
        patterns.insert(SourceLanguage::JavaScript, common_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, common_patterns);

        patterns
    }

    /// Detect weak key derivation functions
    fn detect_weak_kdfs(&self, content: &str, patterns: &[WeakKdfPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakAlgorithm,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: format!("Replace with {}", pattern.replacement),
                            cwe_id: Some(916), // Use of Password Hash With Insufficient Computational Effort
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect weak iteration counts
    fn detect_weak_iterations(&self, content: &str, patterns: &[IterationPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakAlgorithm,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: format!("Use at least {} iterations", pattern.min_iterations),
                            cwe_id: Some(916), // Use of Password Hash With Insufficient Computational Effort
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect salt-related issues
    fn detect_salt_issues(&self, content: &str, patterns: &[SaltPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakAlgorithm,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: pattern.recommendation.clone(),
                            cwe_id: Some(916), // Use of Password Hash With Insufficient Computational Effort
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Check if KDF usage appears secure
    pub fn is_secure_usage(&self, line: &str) -> bool {
        let secure_indicators = [
            "argon2", "scrypt", "pbkdf2.*100000", "pbkdf2.*200000",
            "bcrypt", "random.*salt", "os.urandom"
        ];

        secure_indicators.iter().any(|&indicator|
            Regex::new(indicator).map_or(false, |re| re.is_match(&line.to_lowercase()))
        )
    }

    /// Get recommended secure KDF configurations
    pub fn get_secure_recommendations() -> HashMap<&'static str, Vec<&'static str>> {
        let mut recommendations = HashMap::new();

        recommendations.insert("password_hashing", vec![
            "Argon2id (memory: 64MB, iterations: 3, parallelism: 4)",
            "scrypt (N: 32768, r: 8, p: 1)",
            "bcrypt (cost: 12-15)",
            "PBKDF2-SHA256 (iterations: 600,000+)",
        ]);

        recommendations.insert("key_derivation", vec![
            "HKDF with SHA-256",
            "PBKDF2-SHA256 (iterations: 100,000+)",
            "Argon2id for password-based keys",
            "scrypt for password-based keys",
        ]);

        recommendations.insert("salt_generation", vec![
            "Use crypto-secure random number generator",
            "Minimum 16 bytes (128 bits) of entropy",
            "Unique salt per operation",
            "Store salt alongside derived key",
        ]);

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_kdf_detection() {
        let analyzer = KeyDerivationAnalyzer::new();
        let content = "hashlib.sha256(password + salt).hexdigest()";
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::WeakAlgorithm);
    }

    #[test]
    fn test_low_iteration_detection() {
        let analyzer = KeyDerivationAnalyzer::new();
        let content = "pbkdf2_hmac('sha256', password, salt, 1000)";
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert!(findings[0].description.contains("iteration"));
    }

    #[test]
    fn test_salt_issue_detection() {
        let analyzer = KeyDerivationAnalyzer::new();
        let content = r#"salt = "fixedsalt""#;
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert!(findings[0].description.contains("salt"));
    }

    #[test]
    fn test_secure_usage_detection() {
        let analyzer = KeyDerivationAnalyzer::new();
        assert!(analyzer.is_secure_usage("argon2.hash(password, salt)"));
        assert!(!analyzer.is_secure_usage("md5(password + salt)"));
    }
}