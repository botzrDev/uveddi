//! TLS/SSL Implementation Analysis
//!
//! This module analyzes TLS/SSL implementations for security misconfigurations,
//! weak protocol versions, insecure cipher suites, and certificate validation issues.

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use regex::Regex;
use std::collections::HashMap;

/// Analyzer for TLS/SSL implementations
pub struct TlsAnalyzer {
    version_patterns: HashMap<SourceLanguage, Vec<TlsVersionPattern>>,
    cipher_patterns: HashMap<SourceLanguage, Vec<CipherSuitePattern>>,
    config_patterns: HashMap<SourceLanguage, Vec<TlsConfigPattern>>,
}

/// Pattern for detecting weak TLS/SSL versions
#[derive(Debug, Clone)]
pub struct TlsVersionPattern {
    pub name: String,
    pub pattern: String,
    pub version: TlsVersion,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
}

/// Pattern for detecting weak cipher suites
#[derive(Debug, Clone)]
pub struct CipherSuitePattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub weakness_type: CipherWeakness,
}

/// Pattern for detecting TLS configuration issues
#[derive(Debug, Clone)]
pub struct TlsConfigPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub recommendation: String,
}

/// TLS/SSL protocol versions
#[derive(Debug, Clone, PartialEq)]
pub enum TlsVersion {
    SSL20,
    SSL30,
    TLS10,
    TLS11,
    TLS12,
    TLS13,
}

/// Types of cipher suite weaknesses
#[derive(Debug, Clone, PartialEq)]
pub enum CipherWeakness {
    WeakEncryption,
    WeakHash,
    NoAuthentication,
    WeakKeyExchange,
    ExportGrade,
}

impl TlsAnalyzer {
    /// Create a new TLS analyzer
    pub fn new() -> Self {
        Self {
            version_patterns: Self::initialize_version_patterns(),
            cipher_patterns: Self::initialize_cipher_patterns(),
            config_patterns: Self::initialize_config_patterns(),
        }
    }

    /// Analyze content for TLS/SSL issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Check for weak TLS versions
        if let Some(version_patterns) = self.version_patterns.get(language) {
            findings.extend(self.detect_weak_tls_versions(content, version_patterns)?);
        }

        // Check for weak cipher suites
        if let Some(cipher_patterns) = self.cipher_patterns.get(language) {
            findings.extend(self.detect_weak_ciphers(content, cipher_patterns)?);
        }

        // Check for configuration issues
        if let Some(config_patterns) = self.config_patterns.get(language) {
            findings.extend(self.detect_config_issues(content, config_patterns)?);
        }

        Ok(findings)
    }

    /// Initialize TLS version patterns
    fn initialize_version_patterns() -> HashMap<SourceLanguage, Vec<TlsVersionPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            TlsVersionPattern {
                name: "SSL v2/v3".to_string(),
                pattern: r"(?i)ssl.*v?[23]|sslv[23]".to_string(),
                version: TlsVersion::SSL30,
                description: "SSL v2/v3 protocols are deprecated and insecure".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.9,
            },
            TlsVersionPattern {
                name: "TLS 1.0".to_string(),
                pattern: r"(?i)tls.*v?1\.0|tlsv1\.0".to_string(),
                version: TlsVersion::TLS10,
                description: "TLS 1.0 is deprecated and should not be used".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
            },
            TlsVersionPattern {
                name: "TLS 1.1".to_string(),
                pattern: r"(?i)tls.*v?1\.1|tlsv1\.1".to_string(),
                version: TlsVersion::TLS11,
                description: "TLS 1.1 is deprecated, use TLS 1.2 or higher".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.9,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            TlsVersionPattern {
                name: "SSL v2/v3".to_string(),
                pattern: r"ssl\.PROTOCOL_SSLv[23]|SSLv[23]".to_string(),
                version: TlsVersion::SSL30,
                description: "SSL v2/v3 are cryptographically broken".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
            },
            TlsVersionPattern {
                name: "TLS 1.0".to_string(),
                // Match ssl.PROTOCOL_TLSv1 at word boundary (not followed by _, ., or digit)
                pattern: r"ssl\.PROTOCOL_TLSv1\b|TLSv1\.0".to_string(),
                version: TlsVersion::TLS10,
                description: "TLS 1.0 has known vulnerabilities".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            TlsVersionPattern {
                name: "SSL/TLS Version".to_string(),
                pattern: r"(?i)(ssl|tls).*version.*['\"]?(ssl|tls)?v?[0-2]".to_string(),
                version: TlsVersion::TLS11,
                description: "Old SSL/TLS version detected".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.7,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Initialize cipher suite patterns
    fn initialize_cipher_patterns() -> HashMap<SourceLanguage, Vec<CipherSuitePattern>> {
        let mut patterns = HashMap::new();

        // Common weak cipher patterns for all languages
        let common_patterns = vec![
            CipherSuitePattern {
                name: "DES/3DES Ciphers".to_string(),
                pattern: r"(?i)(des|3des|des_ede)".to_string(),
                description: "DES and 3DES ciphers are weak".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                weakness_type: CipherWeakness::WeakEncryption,
            },
            CipherSuitePattern {
                name: "RC4 Cipher".to_string(),
                pattern: r"(?i)rc4".to_string(),
                description: "RC4 cipher has known vulnerabilities".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                weakness_type: CipherWeakness::WeakEncryption,
            },
            CipherSuitePattern {
                name: "MD5 in Cipher Suite".to_string(),
                pattern: r"(?i)md5".to_string(),
                description: "MD5 is cryptographically broken".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                weakness_type: CipherWeakness::WeakHash,
            },
            CipherSuitePattern {
                name: "NULL Cipher".to_string(),
                pattern: r"(?i)null|anon".to_string(),
                description: "NULL ciphers provide no encryption or authentication".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
                weakness_type: CipherWeakness::NoAuthentication,
            },
            CipherSuitePattern {
                name: "Export Grade Ciphers".to_string(),
                pattern: r"(?i)export|exp".to_string(),
                description: "Export-grade ciphers are intentionally weak".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.9,
                weakness_type: CipherWeakness::ExportGrade,
            },
        ];

        patterns.insert(SourceLanguage::Rust, common_patterns.clone());
        patterns.insert(SourceLanguage::Python, common_patterns.clone());
        patterns.insert(SourceLanguage::JavaScript, common_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, common_patterns);

        patterns
    }

    /// Initialize TLS configuration patterns
    fn initialize_config_patterns() -> HashMap<SourceLanguage, Vec<TlsConfigPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            TlsConfigPattern {
                name: "Disabled Certificate Verification".to_string(),
                pattern: r"(?i)danger.*accept.*invalid|verify.*false".to_string(),
                description: "Certificate verification is disabled".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.9,
                recommendation: "Enable certificate verification".to_string(),
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            TlsConfigPattern {
                name: "Unverified SSL Context".to_string(),
                pattern: r"ssl\._create_unverified_context|ssl_verify.*False".to_string(),
                description: "SSL context created without verification".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
                recommendation: "Use ssl.create_default_context() with verification".to_string(),
            },
            TlsConfigPattern {
                name: "Disabled Hostname Checking".to_string(),
                pattern: r"check_hostname.*False".to_string(),
                description: "Hostname verification is disabled".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                recommendation: "Enable hostname checking".to_string(),
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            TlsConfigPattern {
                name: "Reject Unauthorized False".to_string(),
                pattern: r"rejectUnauthorized\s*:\s*false".to_string(),
                description: "Certificate authorization is disabled".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
                recommendation: "Set rejectUnauthorized to true".to_string(),
            },
            TlsConfigPattern {
                name: "Ignore TLS Errors".to_string(),
                pattern: r"process\.env\[?['\"]NODE_TLS_REJECT_UNAUTHORIZED['\"]?\]?\s*=\s*['\"]?0".to_string(),
                description: "TLS certificate errors are globally ignored".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.95,
                recommendation: "Remove NODE_TLS_REJECT_UNAUTHORIZED=0".to_string(),
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Detect weak TLS versions
    fn detect_weak_tls_versions(&self, content: &str, patterns: &[TlsVersionPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::InsecureTls,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: "Use TLS 1.2 or TLS 1.3".to_string(),
                            cwe_id: Some(326), // Inadequate Encryption Strength
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect weak cipher suites
    fn detect_weak_ciphers(&self, content: &str, patterns: &[CipherSuitePattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) && self.is_cipher_context(line) {
                        let cwe_id = match pattern.weakness_type {
                            CipherWeakness::WeakEncryption => Some(327),
                            CipherWeakness::WeakHash => Some(328),
                            CipherWeakness::NoAuthentication => Some(287),
                            CipherWeakness::ExportGrade => Some(326),
                            _ => Some(327),
                        };

                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::InsecureTls,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: "Use strong cipher suites with AEAD".to_string(),
                            cwe_id,
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect TLS configuration issues
    fn detect_config_issues(&self, content: &str, patterns: &[TlsConfigPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::InvalidCertValidation,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: pattern.recommendation.clone(),
                            cwe_id: Some(295), // Improper Certificate Validation
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Check if line appears to be in a cipher context
    fn is_cipher_context(&self, line: &str) -> bool {
        let cipher_indicators = [
            "cipher", "suite", "ssl", "tls", "crypto",
            "encrypt", "algorithm", "protocol"
        ];

        cipher_indicators.iter().any(|&indicator|
            line.to_lowercase().contains(indicator)
        )
    }

    /// Check if TLS configuration appears secure
    pub fn is_secure_tls_config(&self, content: &str) -> bool {
        let secure_indicators = [
            "tls.*1\\.[23]", "ssl.*default.*context", "verify.*true",
            "rejectUnauthorized.*true", "check_hostname.*true"
        ];

        secure_indicators.iter().any(|&indicator|
            Regex::new(indicator).map_or(false, |re| re.is_match(&content.to_lowercase()))
        )
    }

    /// Get recommended secure TLS configurations
    pub fn get_secure_recommendations() -> HashMap<&'static str, Vec<&'static str>> {
        let mut recommendations = HashMap::new();

        recommendations.insert("tls_versions", vec![
            "TLS 1.3 (preferred)",
            "TLS 1.2 (minimum acceptable)",
            "Disable SSL v2/v3, TLS 1.0/1.1",
        ]);

        recommendations.insert("cipher_suites", vec![
            "Use AEAD cipher suites (GCM, CCM, ChaCha20-Poly1305)",
            "Prefer ECDHE for forward secrecy",
            "Use RSA-3072+ or ECDSA-P256+",
            "Disable RC4, DES, 3DES, NULL ciphers",
        ]);

        recommendations.insert("certificate_validation", vec![
            "Always verify certificate chains",
            "Check hostname against certificate",
            "Verify certificate expiration",
            "Check certificate revocation (OCSP/CRL)",
        ]);

        recommendations.insert("configuration", vec![
            "Use secure defaults",
            "Enable perfect forward secrecy",
            "Implement HSTS",
            "Use certificate pinning for critical applications",
        ]);

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_tls_version_detection() {
        let analyzer = TlsAnalyzer::new();
        let content = "ssl.PROTOCOL_SSLv3";
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::InsecureTls);
    }

    #[test]
    fn test_disabled_cert_verification() {
        let analyzer = TlsAnalyzer::new();
        let content = "rejectUnauthorized: false";
        let findings = analyzer.analyze(content, &SourceLanguage::JavaScript).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::InvalidCertValidation);
    }

    #[test]
    fn test_cipher_context_detection() {
        let analyzer = TlsAnalyzer::new();
        assert!(analyzer.is_cipher_context("cipher_suite = 'RC4'"));
        assert!(!analyzer.is_cipher_context("variable_name = 'rc4_like'"));
    }

    #[test]
    fn test_secure_tls_config() {
        let analyzer = TlsAnalyzer::new();
        assert!(analyzer.is_secure_tls_config("rejectUnauthorized: true"));
        assert!(!analyzer.is_secure_tls_config("rejectUnauthorized: false"));
    }
}