//! Weak Cryptography Detection

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;

pub struct WeakCryptoDetector;

impl WeakCryptoDetector {
    pub fn new() -> Self { Self }

    pub fn analyze(&self, content: &str, _language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        if content.contains("MD5") || content.contains("md5") {
            findings.push(CryptoFinding {
                finding_type: CryptoFindingType::WeakAlgorithm,
                line_number: 1,
                column: 0,
                description: "MD5 is cryptographically broken".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                algorithm: Some("MD5".to_string()),
                recommendation: "Use SHA-256 or newer".to_string(),
                cwe_id: Some(327),
            });
        }

        Ok(findings)
    }
}