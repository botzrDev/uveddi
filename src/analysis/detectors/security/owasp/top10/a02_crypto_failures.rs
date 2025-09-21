//! A02:2021 – Cryptographic Failures Detector
//!
//! This module detects cryptographic vulnerabilities including weak algorithms,
//! hardcoded keys, insufficient entropy, and insecure random number generation.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A02:2021 – Cryptographic Failures Detector
pub struct CryptographicFailuresDetector {
    crypto_patterns: HashMap<SourceLanguage, Vec<CryptoPattern>>,
}

#[derive(Debug, Clone)]
struct CryptoPattern {
    pattern: String,
    issue: String,
    severity: SecuritySeverity,
    confidence: f64,
    category: CryptoVulnCategory,
}

#[derive(Debug, Clone)]
enum CryptoVulnCategory {
    WeakAlgorithm,
    HardcodedSecret,
    WeakRandom,
    InsecureStorage,
    WeakKeyGeneration,
}

impl CryptographicFailuresDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Rust crypto patterns
        patterns.insert(
            SourceLanguage::Rust,
            vec![
                CryptoPattern {
                    pattern: "md5::".to_string(),
                    issue: "Use of weak MD5 hash function".to_string(),
                    severity: SecuritySeverity::High,
                    confidence: 0.9,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
                CryptoPattern {
                    pattern: "sha1::".to_string(),
                    issue: "Use of weak SHA1 hash function".to_string(),
                    severity: SecuritySeverity::Medium,
                    confidence: 0.8,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
                CryptoPattern {
                    pattern: "rand::random".to_string(),
                    issue: "Use of non-cryptographically secure random number generator".to_string(),
                    severity: SecuritySeverity::Medium,
                    confidence: 0.7,
                    category: CryptoVulnCategory::WeakRandom,
                },
                CryptoPattern {
                    pattern: "const SECRET".to_string(),
                    issue: "Potential hardcoded secret constant".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.6,
                    category: CryptoVulnCategory::HardcodedSecret,
                },
                CryptoPattern {
                    pattern: "let key = \"".to_string(),
                    issue: "Potential hardcoded encryption key".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.8,
                    category: CryptoVulnCategory::HardcodedSecret,
                },
                CryptoPattern {
                    pattern: "DES::".to_string(),
                    issue: "Use of deprecated DES encryption".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.95,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
            ],
        );

        // Python crypto patterns
        patterns.insert(
            SourceLanguage::Python,
            vec![
                CryptoPattern {
                    pattern: "hashlib.md5".to_string(),
                    issue: "Use of weak MD5 hash function".to_string(),
                    severity: SecuritySeverity::High,
                    confidence: 0.9,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
                CryptoPattern {
                    pattern: "hashlib.sha1".to_string(),
                    issue: "Use of weak SHA1 hash function".to_string(),
                    severity: SecuritySeverity::Medium,
                    confidence: 0.8,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
                CryptoPattern {
                    pattern: "random.random".to_string(),
                    issue: "Use of non-cryptographically secure random number generator".to_string(),
                    severity: SecuritySeverity::Medium,
                    confidence: 0.6,
                    category: CryptoVulnCategory::WeakRandom,
                },
                CryptoPattern {
                    pattern: "SECRET_KEY = \"".to_string(),
                    issue: "Hardcoded secret key".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.9,
                    category: CryptoVulnCategory::HardcodedSecret,
                },
                CryptoPattern {
                    pattern: "API_KEY = \"".to_string(),
                    issue: "Hardcoded API key".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.9,
                    category: CryptoVulnCategory::HardcodedSecret,
                },
                CryptoPattern {
                    pattern: "Crypto.Cipher.DES".to_string(),
                    issue: "Use of deprecated DES encryption".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.95,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
            ],
        );

        // JavaScript patterns
        patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                CryptoPattern {
                    pattern: "crypto.createHash('md5')".to_string(),
                    issue: "Use of weak MD5 hash function".to_string(),
                    severity: SecuritySeverity::High,
                    confidence: 0.9,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
                CryptoPattern {
                    pattern: "crypto.createHash('sha1')".to_string(),
                    issue: "Use of weak SHA1 hash function".to_string(),
                    severity: SecuritySeverity::Medium,
                    confidence: 0.8,
                    category: CryptoVulnCategory::WeakAlgorithm,
                },
                CryptoPattern {
                    pattern: "Math.random()".to_string(),
                    issue: "Use of non-cryptographically secure random number generator".to_string(),
                    severity: SecuritySeverity::Medium,
                    confidence: 0.5,
                    category: CryptoVulnCategory::WeakRandom,
                },
                CryptoPattern {
                    pattern: "const SECRET_KEY = \"".to_string(),
                    issue: "Hardcoded secret key".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.9,
                    category: CryptoVulnCategory::HardcodedSecret,
                },
                CryptoPattern {
                    pattern: "const API_KEY = \"".to_string(),
                    issue: "Hardcoded API key".to_string(),
                    severity: SecuritySeverity::Critical,
                    confidence: 0.9,
                    category: CryptoVulnCategory::HardcodedSecret,
                },
            ],
        );

        // TypeScript patterns (same as JavaScript)
        patterns.insert(SourceLanguage::TypeScript, patterns[&SourceLanguage::JavaScript].clone());

        Self {
            crypto_patterns: patterns,
        }
    }

    fn create_vulnerability(
        &self,
        pattern: &CryptoPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::CryptographicFailures,
            SecurityIssueType::CryptographicFailures,
            "Cryptographic Failure".to_string(),
            pattern.issue.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.category))
        .with_architectural_correlation(
            OwaspCategory::CryptographicFailures
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    fn get_remediation_advice(category: &CryptoVulnCategory) -> String {
        match category {
            CryptoVulnCategory::WeakAlgorithm => {
                "Replace with strong cryptographic algorithms: SHA-256/SHA-3 for hashing, AES-256 for encryption.".to_string()
            }
            CryptoVulnCategory::HardcodedSecret => {
                "Store secrets in environment variables or secure key management systems. Never hardcode credentials.".to_string()
            }
            CryptoVulnCategory::WeakRandom => {
                "Use cryptographically secure random number generators (CSPRNG) for security-sensitive operations.".to_string()
            }
            CryptoVulnCategory::InsecureStorage => {
                "Implement secure storage with proper encryption and access controls.".to_string()
            }
            CryptoVulnCategory::WeakKeyGeneration => {
                "Use strong key derivation functions (PBKDF2, scrypt, Argon2) with sufficient iterations.".to_string()
            }
        }
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for CryptographicFailuresDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.crypto_patterns.get(&file.language) {
            let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
                AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
            })?;

            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if line.contains(&pattern.pattern) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_crypto_failures_detection() {
        let detector = CryptographicFailuresDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: "import hashlib\nhash = hashlib.md5()".to_string(),
            tree: None,
        };

        std::fs::write("test.py", "import hashlib\nhash = hashlib.md5()").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.py").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::CryptographicFailures);
        assert_eq!(vulnerabilities[0].severity, SecuritySeverity::High);
    }

    #[test]
    fn test_remediation_advice() {
        let advice = CryptographicFailuresDetector::get_remediation_advice(&CryptoVulnCategory::WeakAlgorithm);
        assert!(advice.contains("SHA-256"));
        assert!(advice.contains("AES-256"));
    }
}