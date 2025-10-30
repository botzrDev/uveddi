//! Cryptographic Implementation Security Analysis Module
//!
//! This module provides specialized analysis for cryptographic implementation security,
//! including TLS/SSL configuration, certificate validation, key management practices,
//! padding schemes, and encryption modes.

pub mod tls_analysis;
pub mod certificate_analysis;
pub mod key_management;
pub mod padding_analysis;
pub mod mode_analysis;

pub use tls_analysis::TlsAnalyzer;
pub use certificate_analysis::CertificateAnalyzer;
pub use key_management::KeyManagementAnalyzer;
pub use padding_analysis::PaddingAnalyzer;
pub use mode_analysis::ModeAnalyzer;

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;

/// Main implementation analyzer that coordinates all implementation-specific analyzers
pub struct ImplementationAnalyzer {
    tls: TlsAnalyzer,
    certificate: CertificateAnalyzer,
    key_management: KeyManagementAnalyzer,
    padding: PaddingAnalyzer,
    mode: ModeAnalyzer,
    general_patterns: HashMap<SourceLanguage, Vec<ImplementationPattern>>,
}

/// General implementation security pattern
#[derive(Debug, Clone)]
pub struct ImplementationPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub recommendation: String,
    pub category: ImplementationCategory,
}

/// Categories of implementation issues
#[derive(Debug, Clone, PartialEq)]
pub enum ImplementationCategory {
    TlsConfiguration,
    CertificateValidation,
    KeyManagement,
    PaddingScheme,
    EncryptionMode,
    General,
}

impl ImplementationAnalyzer {
    /// Create a new implementation analyzer with all sub-analyzers
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            tls: TlsAnalyzer::new(),
            certificate: CertificateAnalyzer::new(),
            key_management: KeyManagementAnalyzer::new(),
            padding: PaddingAnalyzer::new(),
            mode: ModeAnalyzer::new(),
            general_patterns: Self::initialize_general_patterns(),
        })
    }

    /// Analyze content for all implementation-related issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Run specialized analyzers
        findings.extend(self.tls.analyze(content, language)?);
        findings.extend(self.certificate.analyze(content, language)?);
        findings.extend(self.key_management.analyze(content, language)?);
        findings.extend(self.padding.analyze(content, language)?);
        findings.extend(self.mode.analyze(content, language)?);

        // Apply general implementation patterns
        if let Some(patterns) = self.general_patterns.get(language) {
            findings.extend(self.apply_general_patterns(content, patterns)?);
        }

        Ok(findings)
    }

    /// Initialize general implementation patterns
    fn initialize_general_patterns() -> HashMap<SourceLanguage, Vec<ImplementationPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    /// Get Rust implementation patterns
    fn rust_patterns() -> Vec<ImplementationPattern> {
        vec![
            ImplementationPattern {
                name: "Hardcoded Cryptographic Key".to_string(),
                pattern: r#"(?i)(key|secret|password)\s*[:=]\s*["'][^"']{8,}["']"#.to_string(),
                description: "Cryptographic key appears to be hardcoded in source code".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.8,
                recommendation: "Store keys in environment variables or secure key management systems".to_string(),
                category: ImplementationCategory::KeyManagement,
            },
            ImplementationPattern {
                name: "Unsafe TLS Context".to_string(),
                pattern: r"(?i)danger.*accept.*invalid".to_string(),
                description: "TLS certificate validation appears to be disabled".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                recommendation: "Enable proper TLS certificate validation".to_string(),
                category: ImplementationCategory::TlsConfiguration,
            },
        ]
    }

    /// Get Python implementation patterns
    fn python_patterns() -> Vec<ImplementationPattern> {
        vec![
            ImplementationPattern {
                name: "SSL Context with No Verification".to_string(),
                pattern: r"ssl\._create_unverified_context|ssl_verify\s*=\s*False".to_string(),
                description: "SSL certificate verification is disabled".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                recommendation: "Use ssl.create_default_context() and enable verification".to_string(),
                category: ImplementationCategory::CertificateValidation,
            },
            ImplementationPattern {
                name: "Hardcoded Secret".to_string(),
                pattern: r#"(?i)(api_key|secret_key|password|token)\s*=\s*["'][^"']{8,}["']"#.to_string(),
                description: "Secret value appears to be hardcoded".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.8,
                recommendation: "Use environment variables or secure configuration".to_string(),
                category: ImplementationCategory::KeyManagement,
            },
        ]
    }

    /// Get JavaScript implementation patterns
    fn javascript_patterns() -> Vec<ImplementationPattern> {
        vec![
            ImplementationPattern {
                name: "TLS Reject Unauthorized False".to_string(),
                pattern: r"rejectUnauthorized\s*:\s*false".to_string(),
                description: "TLS certificate validation is explicitly disabled".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                recommendation: "Enable certificate validation by removing or setting to true".to_string(),
                category: ImplementationCategory::TlsConfiguration,
            },
            ImplementationPattern {
                name: "Hardcoded API Key".to_string(),
                pattern: r#"(?i)(apikey|api_key|secret)\s*[:=]\s*["'][^"']{10,}["']"#.to_string(),
                description: "API key or secret appears to be hardcoded".to_string(),
                severity: SecuritySeverity::Critical,
                confidence: 0.8,
                recommendation: "Use environment variables or secure configuration management".to_string(),
                category: ImplementationCategory::KeyManagement,
            },
        ]
    }

    /// Apply general implementation patterns to content
    fn apply_general_patterns(&self, content: &str, patterns: &[ImplementationPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                    if regex.is_match(line) && !self.is_likely_safe_context(line) {
                        let finding_type = match pattern.category {
                            ImplementationCategory::TlsConfiguration => CryptoFindingType::InsecureTls,
                            ImplementationCategory::CertificateValidation => CryptoFindingType::InvalidCertValidation,
                            ImplementationCategory::KeyManagement => CryptoFindingType::HardcodedKey,
                            ImplementationCategory::PaddingScheme => CryptoFindingType::WeakAlgorithm,
                            ImplementationCategory::EncryptionMode => CryptoFindingType::WeakAlgorithm,
                            ImplementationCategory::General => CryptoFindingType::WeakKeyManagement,
                        };

                        findings.push(CryptoFinding {
                            finding_type,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: pattern.recommendation.clone(),
                            cwe_id: Some(327), // Use of a Broken or Risky Cryptographic Algorithm
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Check if the pattern appears to be in a safe context
    fn is_likely_safe_context(&self, line: &str) -> bool {
        let safe_indicators = [
            "test", "example", "demo", "mock", "stub",
            "todo", "fixme", "comment", "//"
        ];

        let line_lower = line.to_lowercase();
        safe_indicators.iter().any(|&indicator|
            line_lower.contains(indicator)
        )
    }

    /// Check if implementation appears secure
    pub fn is_secure_implementation(&self, content: &str) -> bool {
        let secure_indicators = [
            "ssl.create_default_context",
            "rejectUnauthorized.*true",
            "verify.*true",
            "secure.*true",
            "env\\.",
            "process\\.env",
            "os\\.environ",
        ];

        secure_indicators.iter().any(|&indicator|
            regex::Regex::new(indicator).map_or(false, |re| re.is_match(content))
        )
    }

    /// Get implementation security best practices
    pub fn get_best_practices() -> HashMap<&'static str, Vec<&'static str>> {
        let mut practices = HashMap::new();

        practices.insert("tls_configuration", vec![
            "Use TLS 1.2 or higher",
            "Enable certificate validation",
            "Use strong cipher suites",
            "Implement certificate pinning for high-security applications",
            "Verify hostname matches certificate",
        ]);

        practices.insert("key_management", vec![
            "Never hardcode keys in source code",
            "Use environment variables or secure configuration",
            "Implement key rotation policies",
            "Use hardware security modules (HSMs) for critical keys",
            "Encrypt keys at rest",
        ]);

        practices.insert("certificate_validation", vec![
            "Always validate certificate chains",
            "Check certificate expiration",
            "Verify certificate revocation status",
            "Validate hostname against certificate",
            "Use trusted certificate authorities",
        ]);

        practices.insert("encryption_modes", vec![
            "Use authenticated encryption (AEAD)",
            "Avoid ECB mode",
            "Use random IVs for CBC mode",
            "Implement proper padding",
            "Use separate keys for encryption and authentication",
        ]);

        practices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implementation_analyzer_creation() {
        let analyzer = ImplementationAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_hardcoded_key_detection() {
        let analyzer = ImplementationAnalyzer::new().unwrap();
        let content = r#"api_key = "sk-1234567890abcdef""#;
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::HardcodedKey);
    }

    #[test]
    fn test_tls_misconfiguration_detection() {
        let analyzer = ImplementationAnalyzer::new().unwrap();
        let content = "rejectUnauthorized: false";
        let findings = analyzer.analyze(content, &SourceLanguage::JavaScript).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::InsecureTls);
    }

    #[test]
    fn test_safe_context_detection() {
        let analyzer = ImplementationAnalyzer::new().unwrap();
        assert!(analyzer.is_likely_safe_context("// test api_key = 'test123'"));
        assert!(!analyzer.is_likely_safe_context("api_key = 'sk-1234567890'"));
    }

    #[test]
    fn test_secure_implementation_detection() {
        let analyzer = ImplementationAnalyzer::new().unwrap();
        assert!(analyzer.is_secure_implementation("ssl.create_default_context()"));
        assert!(!analyzer.is_secure_implementation("ssl_verify = False"));
    }
}