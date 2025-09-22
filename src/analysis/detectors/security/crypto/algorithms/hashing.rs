//! Hash Function Algorithm Analysis
//!
//! This module analyzes hash function usage for security issues, including weak hash
//! algorithms, improper usage patterns, and insufficient security properties.

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use regex::Regex;
use std::collections::HashMap;

/// Analyzer for hash function algorithms
pub struct HashingAnalyzer {
    weak_hashes: HashMap<SourceLanguage, Vec<WeakHashPattern>>,
    usage_patterns: HashMap<SourceLanguage, Vec<HashUsagePattern>>,
}

/// Pattern for detecting weak hash algorithms
#[derive(Debug, Clone)]
pub struct WeakHashPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub replacement: String,
    pub deprecated: bool,
    pub security_level: HashSecurityLevel,
}

/// Pattern for detecting insecure hash usage
#[derive(Debug, Clone)]
pub struct HashUsagePattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub recommendation: String,
    pub usage_type: HashUsageType,
}

/// Security levels for hash functions
#[derive(Debug, Clone, PartialEq)]
pub enum HashSecurityLevel {
    Secure,
    Weak,
    Broken,
    Deprecated,
}

/// Types of hash function usage
#[derive(Debug, Clone, PartialEq)]
pub enum HashUsageType {
    PasswordHashing,
    CryptographicSigning,
    IntegrityCheck,
    KeyDerivation,
    Unknown,
}

impl HashingAnalyzer {
    /// Create a new hashing analyzer
    pub fn new() -> Self {
        Self {
            weak_hashes: Self::initialize_weak_hashes(),
            usage_patterns: Self::initialize_usage_patterns(),
        }
    }

    /// Analyze content for hash function issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Check for weak hash algorithms
        if let Some(weak_patterns) = self.weak_hashes.get(language) {
            findings.extend(self.detect_weak_hashes(content, weak_patterns)?);
        }

        // Check for insecure usage patterns
        if let Some(usage_patterns) = self.usage_patterns.get(language) {
            findings.extend(self.detect_insecure_usage(content, usage_patterns)?);
        }

        Ok(findings)
    }

    /// Initialize weak hash patterns for each language
    fn initialize_weak_hashes() -> HashMap<SourceLanguage, Vec<WeakHashPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            WeakHashPattern {
                name: "MD5 Hash".to_string(),
                pattern: r"(?i)(md5::|Md5::new|digest::Md5)".to_string(),
                description: "MD5 is cryptographically broken and vulnerable to collision attacks".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                replacement: "SHA-256, SHA-3, or BLAKE3".to_string(),
                deprecated: true,
                security_level: HashSecurityLevel::Broken,
            },
            WeakHashPattern {
                name: "SHA-1 Hash".to_string(),
                pattern: r"(?i)(sha1::|Sha1::new|digest::Sha1)".to_string(),
                description: "SHA-1 is cryptographically weak and deprecated".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.95,
                replacement: "SHA-256 or SHA-3".to_string(),
                deprecated: true,
                security_level: HashSecurityLevel::Weak,
            },
            WeakHashPattern {
                name: "CRC32 Hash".to_string(),
                pattern: r"(?i)(crc32|crc::|Crc32)".to_string(),
                description: "CRC32 is not cryptographically secure and should not be used for security".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.8,
                replacement: "SHA-256 for cryptographic purposes".to_string(),
                deprecated: false,
                security_level: HashSecurityLevel::Weak,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            WeakHashPattern {
                name: "MD5 Hash".to_string(),
                pattern: r"hashlib\.md5\s*\(|md5\.new\s*\(|MD5\.new\s*\(".to_string(),
                description: "MD5 hash algorithm is cryptographically broken".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                replacement: "hashlib.sha256() or hashlib.sha3_256()".to_string(),
                deprecated: true,
                security_level: HashSecurityLevel::Broken,
            },
            WeakHashPattern {
                name: "SHA-1 Hash".to_string(),
                pattern: r"hashlib\.sha1\s*\(|sha\.new\s*\(|SHA\.new\s*\(".to_string(),
                description: "SHA-1 is cryptographically weak".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.95,
                replacement: "hashlib.sha256() or hashlib.sha3_256()".to_string(),
                deprecated: true,
                security_level: HashSecurityLevel::Weak,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            WeakHashPattern {
                name: "MD5 Hash".to_string(),
                pattern: r"crypto\.createHash\s*\(\s*['\"]md5['\"]".to_string(),
                description: "MD5 hash is cryptographically broken".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                replacement: "sha256 or sha3-256".to_string(),
                deprecated: true,
                security_level: HashSecurityLevel::Broken,
            },
            WeakHashPattern {
                name: "SHA-1 Hash".to_string(),
                pattern: r"crypto\.createHash\s*\(\s*['\"]sha1['\"]".to_string(),
                description: "SHA-1 is cryptographically weak".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.95,
                replacement: "sha256 or sha512".to_string(),
                deprecated: true,
                security_level: HashSecurityLevel::Weak,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Initialize hash usage patterns
    fn initialize_usage_patterns() -> HashMap<SourceLanguage, Vec<HashUsagePattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            HashUsagePattern {
                name: "Password Hashing with Fast Hash".to_string(),
                pattern: r"(?i)password.*(?:sha256|md5|sha1)".to_string(),
                description: "Fast hash functions are inappropriate for password hashing".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.7,
                recommendation: "Use bcrypt, scrypt, or Argon2 for password hashing".to_string(),
                usage_type: HashUsageType::PasswordHashing,
            },
            HashUsagePattern {
                name: "Hash without Salt".to_string(),
                pattern: r"(?i)hash\s*\(\s*password\s*\)".to_string(),
                description: "Hashing passwords without salt is vulnerable to rainbow table attacks".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.6,
                recommendation: "Use a unique salt for each password".to_string(),
                usage_type: HashUsageType::PasswordHashing,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            HashUsagePattern {
                name: "Password with SHA256".to_string(),
                pattern: r"(?i)hashlib\.sha256\s*\(\s*password".to_string(),
                description: "SHA-256 is too fast for password hashing".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                recommendation: "Use bcrypt, scrypt, or Argon2 via the cryptography library".to_string(),
                usage_type: HashUsageType::PasswordHashing,
            },
            HashUsagePattern {
                name: "Hardcoded Salt".to_string(),
                pattern: r"(?i)salt\s*=\s*['\"][^'\"]{1,16}['\"]".to_string(),
                description: "Hardcoded or short salt reduces security".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.7,
                recommendation: "Generate random salts of at least 16 bytes".to_string(),
                usage_type: HashUsageType::PasswordHashing,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            HashUsagePattern {
                name: "Password with Fast Hash".to_string(),
                pattern: r"(?i)crypto\.createHash.*password".to_string(),
                description: "Built-in crypto hash functions are too fast for passwords".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                recommendation: "Use bcrypt, scrypt, or Argon2 library".to_string(),
                usage_type: HashUsageType::PasswordHashing,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Detect weak hash algorithms
    fn detect_weak_hashes(&self, content: &str, patterns: &[WeakHashPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) && !self.is_likely_safe_context(line) {
                        let cwe_id = match pattern.security_level {
                            HashSecurityLevel::Broken => Some(328), // Reversible One-Way Hash
                            HashSecurityLevel::Weak => Some(327),   // Use of a Broken or Risky Cryptographic Algorithm
                            _ => Some(327),
                        };

                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakAlgorithm,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: format!("Replace with {}", pattern.replacement),
                            cwe_id,
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect insecure hash usage patterns
    fn detect_insecure_usage(&self, content: &str, patterns: &[HashUsagePattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        let cwe_id = match pattern.usage_type {
                            HashUsageType::PasswordHashing => Some(916), // Use of Password Hash With Insufficient Computational Effort
                            HashUsageType::KeyDerivation => Some(327),   // Use of a Broken or Risky Cryptographic Algorithm
                            _ => Some(327),
                        };

                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakAlgorithm,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: pattern.recommendation.clone(),
                            cwe_id,
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Check if the hash usage appears to be in a safe context
    fn is_likely_safe_context(&self, line: &str) -> bool {
        let safe_indicators = [
            "test", "example", "demo", "checksum", "etag",
            "fingerprint", "cache", "non-crypto", "debug"
        ];

        let line_lower = line.to_lowercase();
        safe_indicators.iter().any(|&indicator|
            line_lower.contains(indicator)
        )
    }

    /// Check if hash usage appears secure
    pub fn is_secure_usage(&self, line: &str) -> bool {
        let secure_indicators = [
            "sha256", "sha512", "sha3", "blake2", "blake3",
            "bcrypt", "scrypt", "argon2", "pbkdf2"
        ];

        secure_indicators.iter().any(|&indicator|
            line.to_lowercase().contains(indicator)
        )
    }

    /// Get recommended secure hash functions by use case
    pub fn get_secure_recommendations() -> HashMap<&'static str, Vec<&'static str>> {
        let mut recommendations = HashMap::new();

        recommendations.insert("general_purpose", vec![
            "SHA-256",
            "SHA-512",
            "SHA-3 (Keccak)",
            "BLAKE3",
            "BLAKE2",
        ]);

        recommendations.insert("password_hashing", vec![
            "Argon2id",
            "bcrypt",
            "scrypt",
            "PBKDF2 with SHA-256",
        ]);

        recommendations.insert("key_derivation", vec![
            "HKDF with SHA-256",
            "PBKDF2 with SHA-256",
            "scrypt",
            "Argon2",
        ]);

        recommendations.insert("digital_signatures", vec![
            "SHA-256",
            "SHA-512",
            "SHA-3",
        ]);

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_detection() {
        let analyzer = HashingAnalyzer::new();
        let content = "use md5::Md5;\nlet digest = Md5::new();";
        let findings = analyzer.analyze(content, &SourceLanguage::Rust).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::WeakAlgorithm);
        assert!(findings[0].description.contains("MD5"));
    }

    #[test]
    fn test_password_hashing_detection() {
        let analyzer = HashingAnalyzer::new();
        let content = "hashlib.sha256(password.encode()).hexdigest()";
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert!(findings[0].description.contains("password"));
    }

    #[test]
    fn test_safe_context_detection() {
        let analyzer = HashingAnalyzer::new();
        assert!(analyzer.is_likely_safe_context("// test md5 checksum"));
        assert!(!analyzer.is_likely_safe_context("user_hash = md5(password)"));
    }

    #[test]
    fn test_secure_usage_detection() {
        let analyzer = HashingAnalyzer::new();
        assert!(analyzer.is_secure_usage("use sha2::Sha256;"));
        assert!(!analyzer.is_secure_usage("use md5::Md5;"));
    }
}