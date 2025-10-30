//! Symmetric Encryption Algorithm Analysis
//!
//! This module detects and analyzes symmetric encryption algorithms for security issues,
//! including weak ciphers, deprecated algorithms, and insecure modes of operation.

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use regex::Regex;
use std::collections::HashMap;

/// Analyzer for symmetric encryption algorithms
pub struct SymmetricAnalyzer {
    weak_algorithms: HashMap<SourceLanguage, Vec<WeakSymmetricPattern>>,
    insecure_modes: HashMap<SourceLanguage, Vec<InsecureModePattern>>,
}

/// Pattern for detecting weak symmetric algorithms
#[derive(Debug, Clone)]
pub struct WeakSymmetricPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub replacement: String,
    pub deprecated: bool,
}

/// Pattern for detecting insecure encryption modes
#[derive(Debug, Clone)]
pub struct InsecureModePattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub secure_alternative: String,
}

impl SymmetricAnalyzer {
    /// Create a new symmetric analyzer
    pub fn new() -> Self {
        Self {
            weak_algorithms: Self::initialize_weak_algorithms(),
            insecure_modes: Self::initialize_insecure_modes(),
        }
    }

    /// Analyze content for symmetric encryption issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Check for weak algorithms
        if let Some(weak_patterns) = self.weak_algorithms.get(language) {
            findings.extend(self.detect_weak_algorithms(content, weak_patterns)?);
        }

        // Check for insecure modes
        if let Some(mode_patterns) = self.insecure_modes.get(language) {
            findings.extend(self.detect_insecure_modes(content, mode_patterns)?);
        }

        Ok(findings)
    }

    /// Initialize weak algorithm patterns for each language
    fn initialize_weak_algorithms() -> HashMap<SourceLanguage, Vec<WeakSymmetricPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            WeakSymmetricPattern {
                name: "DES Encryption".to_string(),
                pattern: r"(?i)(des::|Des::new|des_ede)".to_string(),
                description: "DES encryption is cryptographically broken (56-bit key)".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
                replacement: "AES-256 or ChaCha20".to_string(),
                deprecated: true,
            },
            WeakSymmetricPattern {
                name: "RC4 Stream Cipher".to_string(),
                pattern: r"(?i)(rc4::|Rc4::new)".to_string(),
                description: "RC4 stream cipher has known vulnerabilities".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                replacement: "ChaCha20 or AES-CTR".to_string(),
                deprecated: true,
            },
            WeakSymmetricPattern {
                name: "Blowfish Cipher".to_string(),
                pattern: r"(?i)(blowfish::|Blowfish::new)".to_string(),
                description: "Blowfish has a small block size (64-bit) and is vulnerable to attacks".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.9,
                replacement: "AES-256".to_string(),
                deprecated: false,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            WeakSymmetricPattern {
                name: "DES Encryption".to_string(),
                pattern: r"Crypto\.Cipher\.DES".to_string(),
                description: "DES encryption is cryptographically broken".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
                replacement: "Crypto.Cipher.AES".to_string(),
                deprecated: true,
            },
            WeakSymmetricPattern {
                name: "ARC2 Cipher".to_string(),
                pattern: r"Crypto\.Cipher\.ARC2".to_string(),
                description: "ARC2 cipher is weak and deprecated".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                replacement: "Crypto.Cipher.AES".to_string(),
                deprecated: true,
            },
            WeakSymmetricPattern {
                name: "ARC4/RC4 Cipher".to_string(),
                pattern: r"Crypto\.Cipher\.(ARC4|RC4)".to_string(),
                description: "RC4/ARC4 stream cipher has known vulnerabilities".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                replacement: "Crypto.Cipher.ChaCha20".to_string(),
                deprecated: true,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            WeakSymmetricPattern {
                name: "DES Cipher".to_string(),
                pattern: r"crypto\.createCipher\s*\(\s*['\"]des".to_string(),
                description: "DES cipher is cryptographically broken".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
                replacement: "aes-256-gcm".to_string(),
                deprecated: true,
            },
            WeakSymmetricPattern {
                name: "RC4 Cipher".to_string(),
                pattern: r"crypto\.createCipher\s*\(\s*['\"]rc4".to_string(),
                description: "RC4 cipher has known vulnerabilities".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                replacement: "aes-256-gcm".to_string(),
                deprecated: true,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Initialize insecure mode patterns
    fn initialize_insecure_modes() -> HashMap<SourceLanguage, Vec<InsecureModePattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            InsecureModePattern {
                name: "ECB Mode".to_string(),
                pattern: r"(?i)(ecb|Ecb)".to_string(),
                description: "ECB mode is insecure for most use cases (patterns in ciphertext)".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                secure_alternative: "GCM or CBC with HMAC".to_string(),
            },
            InsecureModePattern {
                name: "CBC without MAC".to_string(),
                pattern: r"(?i)cbc(?!.*(?:hmac|tag|auth))".to_string(),
                description: "CBC mode without authentication is vulnerable to padding oracle attacks".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.6,
                secure_alternative: "GCM mode or CBC with HMAC".to_string(),
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            InsecureModePattern {
                name: "ECB Mode".to_string(),
                pattern: r"MODE_ECB".to_string(),
                description: "ECB mode reveals patterns in plaintext".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                secure_alternative: "MODE_GCM or MODE_CBC with HMAC".to_string(),
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            InsecureModePattern {
                name: "ECB Mode".to_string(),
                pattern: r"['\"]aes-\d+-ecb['\"]".to_string(),
                description: "ECB mode is deterministic and reveals patterns".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                secure_alternative: "aes-256-gcm".to_string(),
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Detect weak symmetric algorithms
    fn detect_weak_algorithms(&self, content: &str, patterns: &[WeakSymmetricPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
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

    /// Detect insecure encryption modes
    fn detect_insecure_modes(&self, content: &str, patterns: &[InsecureModePattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
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
                            recommendation: format!("Use {}", pattern.secure_alternative),
                            cwe_id: Some(327), // Use of a Broken or Risky Cryptographic Algorithm
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Check if a line appears to be using symmetric encryption securely
    pub fn is_secure_usage(&self, line: &str) -> bool {
        let secure_indicators = [
            "gcm", "ccm", "chacha20", "poly1305", "aes-256",
            "authenticated", "hmac", "tag", "auth"
        ];

        secure_indicators.iter().any(|&indicator|
            line.to_lowercase().contains(indicator)
        )
    }

    /// Get recommended secure symmetric algorithms
    pub fn get_secure_recommendations() -> Vec<&'static str> {
        vec![
            "AES-256-GCM",
            "ChaCha20-Poly1305",
            "AES-256-CBC with HMAC-SHA256",
            "AES-128-GCM (for performance-critical applications)",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_des_detection_rust() {
        let analyzer = SymmetricAnalyzer::new();
        let content = "use des::Des;\nlet cipher = Des::new(&key);";
        let findings = analyzer.analyze(content, &SourceLanguage::Rust).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::WeakAlgorithm);
        assert_eq!(findings[0].severity, SecuritySeverity::Critical);
    }

    #[test]
    fn test_ecb_mode_detection() {
        let analyzer = SymmetricAnalyzer::new();
        let content = "cipher = AES.new(key, AES.MODE_ECB)";
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert!(findings[0].description.contains("ECB"));
    }

    #[test]
    fn test_secure_usage_detection() {
        let analyzer = SymmetricAnalyzer::new();
        assert!(analyzer.is_secure_usage("use aes_gcm::AesGcm;"));
        assert!(!analyzer.is_secure_usage("use des::Des;"));
    }
}