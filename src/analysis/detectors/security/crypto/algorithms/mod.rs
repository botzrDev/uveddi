//! Cryptographic Algorithm Analysis Module
//!
//! This module provides specialized analysis for different types of cryptographic algorithms,
//! including detection of weak, deprecated, or misused algorithms across supported languages.

pub mod symmetric;
pub mod asymmetric;
pub mod hashing;
pub mod key_derivation;
pub mod random_generation;

pub use symmetric::SymmetricAnalyzer;
pub use asymmetric::AsymmetricAnalyzer;
pub use hashing::HashingAnalyzer;
pub use key_derivation::KeyDerivationAnalyzer;
pub use random_generation::RandomGenerationAnalyzer;

use crate::analysis::detectors::security::crypto::types::{
    AlgorithmPattern, AlgorithmType, SecurityLevel, CryptoFinding, CryptoFindingType
};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;

/// Main algorithm analyzer that coordinates all algorithm-specific analyzers
pub struct AlgorithmAnalyzer {
    symmetric: SymmetricAnalyzer,
    asymmetric: AsymmetricAnalyzer,
    hashing: HashingAnalyzer,
    key_derivation: KeyDerivationAnalyzer,
    random_generation: RandomGenerationAnalyzer,
    patterns: HashMap<SourceLanguage, Vec<AlgorithmPattern>>,
}

impl AlgorithmAnalyzer {
    /// Create a new algorithm analyzer with all sub-analyzers
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            symmetric: SymmetricAnalyzer::new(),
            asymmetric: AsymmetricAnalyzer::new(),
            hashing: HashingAnalyzer::new(),
            key_derivation: KeyDerivationAnalyzer::new(),
            random_generation: RandomGenerationAnalyzer::new(),
            patterns: Self::initialize_patterns(),
        })
    }

    /// Analyze content for all algorithm-related issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Run specialized analyzers
        findings.extend(self.symmetric.analyze(content, language)?);
        findings.extend(self.asymmetric.analyze(content, language)?);
        findings.extend(self.hashing.analyze(content, language)?);
        findings.extend(self.key_derivation.analyze(content, language)?);
        findings.extend(self.random_generation.analyze(content, language)?);

        // Apply general algorithm patterns
        if let Some(patterns) = self.patterns.get(language) {
            findings.extend(self.apply_patterns(content, patterns)?);
        }

        Ok(findings)
    }

    /// Initialize language-specific algorithm patterns
    fn initialize_patterns() -> HashMap<SourceLanguage, Vec<AlgorithmPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    /// Get Rust algorithm patterns
    fn rust_patterns() -> Vec<AlgorithmPattern> {
        vec![
            AlgorithmPattern {
                name: "Generic Weak Algorithm".to_string(),
                pattern: r"(cbc|ecb|des|3des|rc4)".to_string(),
                algorithm_type: AlgorithmType::SymmetricEncryption,
                security_level: SecurityLevel::Weak,
                description: "Potentially weak cryptographic algorithm detected".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::Medium,
                deprecated: false,
                replacements: vec!["AES-GCM".to_string(), "ChaCha20-Poly1305".to_string()],
            },
        ]
    }

    /// Get Python algorithm patterns
    fn python_patterns() -> Vec<AlgorithmPattern> {
        vec![
            AlgorithmPattern {
                name: "Weak Cipher".to_string(),
                pattern: r"Crypto\.Cipher\.(DES|ARC2|ARC4|Blowfish)".to_string(),
                algorithm_type: AlgorithmType::SymmetricEncryption,
                security_level: SecurityLevel::Weak,
                description: "Weak symmetric cipher detected".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                deprecated: true,
                replacements: vec!["AES".to_string()],
            },
        ]
    }

    /// Get JavaScript algorithm patterns
    fn javascript_patterns() -> Vec<AlgorithmPattern> {
        vec![
            AlgorithmPattern {
                name: "Weak Algorithm".to_string(),
                pattern: r"createCipher\s*\(\s*['\"]des".to_string(),
                algorithm_type: AlgorithmType::SymmetricEncryption,
                security_level: SecurityLevel::Broken,
                description: "DES cipher is cryptographically broken".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                deprecated: true,
                replacements: vec!["aes-256-gcm".to_string()],
            },
        ]
    }

    /// Apply algorithm patterns to content
    fn apply_patterns(&self, content: &str, patterns: &[AlgorithmPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakAlgorithm,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: format!("Replace with: {:?}", pattern.replacements),
                            cwe_id: None,
                        });
                    }
                }
            }
        }

        Ok(findings)
    }
}