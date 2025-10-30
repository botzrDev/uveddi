//! Asymmetric Encryption Algorithm Analysis
//!
//! This module analyzes asymmetric (public-key) cryptography usage for security issues,
//! including weak key sizes, deprecated algorithms, and insecure implementations.

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use regex::Regex;
use std::collections::HashMap;

/// Analyzer for asymmetric encryption algorithms
pub struct AsymmetricAnalyzer {
    weak_algorithms: HashMap<SourceLanguage, Vec<WeakAsymmetricPattern>>,
    key_size_patterns: HashMap<SourceLanguage, Vec<KeySizePattern>>,
}

/// Pattern for detecting weak asymmetric algorithms
#[derive(Debug, Clone)]
pub struct WeakAsymmetricPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub replacement: String,
    pub deprecated: bool,
}

/// Pattern for detecting insufficient key sizes
#[derive(Debug, Clone)]
pub struct KeySizePattern {
    pub algorithm: String,
    pub pattern: String,
    pub min_secure_size: u32,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
}

impl AsymmetricAnalyzer {
    /// Create a new asymmetric analyzer
    pub fn new() -> Self {
        Self {
            weak_algorithms: Self::initialize_weak_algorithms(),
            key_size_patterns: Self::initialize_key_size_patterns(),
        }
    }

    /// Analyze content for asymmetric encryption issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Check for weak algorithms
        if let Some(weak_patterns) = self.weak_algorithms.get(language) {
            findings.extend(self.detect_weak_algorithms(content, weak_patterns)?);
        }

        // Check for insufficient key sizes
        if let Some(key_patterns) = self.key_size_patterns.get(language) {
            findings.extend(self.detect_weak_key_sizes(content, key_patterns)?);
        }

        Ok(findings)
    }

    /// Initialize weak algorithm patterns for each language
    fn initialize_weak_algorithms() -> HashMap<SourceLanguage, Vec<WeakAsymmetricPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            WeakAsymmetricPattern {
                name: "DSA Algorithm".to_string(),
                pattern: r"(?i)(dsa::|Dsa::)".to_string(),
                description: "DSA is deprecated and should be avoided".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.9,
                replacement: "Ed25519 or ECDSA with P-256".to_string(),
                deprecated: true,
            },
            WeakAsymmetricPattern {
                name: "RSA PKCS#1 v1.5".to_string(),
                pattern: r"(?i)pkcs1.*v1[._]5".to_string(),
                description: "PKCS#1 v1.5 padding is vulnerable to padding oracle attacks".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                replacement: "RSA-OAEP or PSS padding".to_string(),
                deprecated: true,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            WeakAsymmetricPattern {
                name: "DSA Algorithm".to_string(),
                pattern: r"cryptography\.hazmat\.primitives\.asymmetric\.dsa".to_string(),
                description: "DSA is deprecated in favor of ECDSA or Ed25519".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.9,
                replacement: "Ed25519 or ECDSA".to_string(),
                deprecated: true,
            },
            WeakAsymmetricPattern {
                name: "PKCS1v15 Padding".to_string(),
                pattern: r"PKCS1v15\(\)".to_string(),
                description: "PKCS#1 v1.5 padding is vulnerable to padding oracle attacks".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                replacement: "OAEP padding".to_string(),
                deprecated: true,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            WeakAsymmetricPattern {
                name: "RSA PKCS1 Padding".to_string(),
                pattern: r"['\"]RSA_PKCS1_PADDING['\"]".to_string(),
                description: "PKCS#1 v1.5 padding is vulnerable".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                replacement: "RSA_PKCS1_OAEP_PADDING".to_string(),
                deprecated: true,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Initialize key size patterns
    fn initialize_key_size_patterns() -> HashMap<SourceLanguage, Vec<KeySizePattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            KeySizePattern {
                algorithm: "RSA".to_string(),
                pattern: r"rsa.*(?:512|768|1024)".to_string(),
                min_secure_size: 2048,
                description: "RSA key size is too small (minimum 2048 bits recommended)".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
            },
            KeySizePattern {
                algorithm: "ECC".to_string(),
                pattern: r"ecc.*(?:112|128|160)".to_string(),
                min_secure_size: 256,
                description: "ECC key size is too small (minimum 256 bits recommended)".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.8,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            KeySizePattern {
                algorithm: "RSA".to_string(),
                pattern: r"key_size=(?:512|768|1024)".to_string(),
                min_secure_size: 2048,
                description: "RSA key size below 2048 bits is insecure".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            KeySizePattern {
                algorithm: "RSA".to_string(),
                pattern: r"modulusLength:\s*(?:512|768|1024)".to_string(),
                min_secure_size: 2048,
                description: "RSA modulus length is too small".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Detect weak asymmetric algorithms
    fn detect_weak_algorithms(&self, content: &str, patterns: &[WeakAsymmetricPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
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
                            cwe_id: Some(327), // Use of a Broken or Risky Cryptographic Algorithm
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect weak key sizes
    fn detect_weak_key_sizes(&self, content: &str, patterns: &[KeySizePattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
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
                            algorithm: Some(pattern.algorithm.clone()),
                            recommendation: format!("Use key size of at least {} bits", pattern.min_secure_size),
                            cwe_id: Some(326), // Inadequate Encryption Strength
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Check if asymmetric crypto usage appears secure
    pub fn is_secure_usage(&self, line: &str) -> bool {
        let secure_indicators = [
            "ed25519", "ecdsa", "p-256", "p-384", "p-521",
            "rsa-oaep", "rsa-pss", "2048", "3072", "4096"
        ];

        secure_indicators.iter().any(|&indicator|
            line.to_lowercase().contains(indicator)
        )
    }

    /// Get recommended secure asymmetric algorithms
    pub fn get_secure_recommendations() -> Vec<&'static str> {
        vec![
            "Ed25519 (for signing)",
            "X25519 (for key exchange)",
            "ECDSA with P-256/P-384 (for signing)",
            "ECDH with P-256/P-384 (for key exchange)",
            "RSA-3072 with OAEP/PSS (legacy compatibility)",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_rsa_key_detection() {
        let analyzer = AsymmetricAnalyzer::new();
        let content = "generate_rsa_key(key_size=1024)";
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::WeakAlgorithm);
        assert!(findings[0].description.contains("RSA"));
    }

    #[test]
    fn test_dsa_detection() {
        let analyzer = AsymmetricAnalyzer::new();
        let content = "use dsa::Dsa;";
        let findings = analyzer.analyze(content, &SourceLanguage::Rust).unwrap();

        assert!(!findings.is_empty());
        assert!(findings[0].description.contains("DSA"));
    }

    #[test]
    fn test_secure_usage_detection() {
        let analyzer = AsymmetricAnalyzer::new();
        assert!(analyzer.is_secure_usage("use ed25519_dalek;"));
        assert!(!analyzer.is_secure_usage("use dsa::Dsa;"));
    }
}