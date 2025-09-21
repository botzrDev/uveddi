//! A03:2021 – Sensitive Data Exposure Detector
//!
//! Detects vulnerabilities related to sensitive data exposure and cryptographic failures.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A03:2021 – Sensitive Data Exposure Detector
pub struct SensitiveDataDetector {
    data_patterns: HashMap<SourceLanguage, Vec<DataPattern>>,
}

#[derive(Debug, Clone)]
pub struct DataPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub data_type: SensitiveDataType,
}

#[derive(Debug, Clone)]
pub enum SensitiveDataType {
    CreditCard,
    SocialSecurityNumber,
    ApiKeys,
    DatabaseCredentials,
    PersonalInformation,
    WeakEncryption,
    UnencryptedStorage,
}

impl SensitiveDataDetector {
    pub fn new() -> Self {
        Self {
            data_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<DataPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<DataPattern> {
        vec![
            DataPattern {
                pattern: "api_key = \"".to_string(),
                vulnerability_type: SecurityIssueType::CryptographicFailures,
                description: "API key hardcoded in source code".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                data_type: SensitiveDataType::ApiKeys,
            },
            DataPattern {
                pattern: "MD5".to_string(),
                vulnerability_type: SecurityIssueType::CryptographicFailures,
                description: "Use of weak MD5 hash algorithm".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                data_type: SensitiveDataType::WeakEncryption,
            },
        ]
    }

    fn python_patterns() -> Vec<DataPattern> {
        vec![
            DataPattern {
                pattern: "SSN".to_string(),
                vulnerability_type: SecurityIssueType::CryptographicFailures,
                description: "Potential social security number exposure".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::High,
                data_type: SensitiveDataType::SocialSecurityNumber,
            },
            DataPattern {
                pattern: "credit_card".to_string(),
                vulnerability_type: SecurityIssueType::CryptographicFailures,
                description: "Credit card information handling detected".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::Critical,
                data_type: SensitiveDataType::CreditCard,
            },
        ]
    }

    fn javascript_patterns() -> Vec<DataPattern> {
        vec![
            DataPattern {
                pattern: "localStorage.setItem".to_string(),
                vulnerability_type: SecurityIssueType::CryptographicFailures,
                description: "Sensitive data stored in localStorage".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                data_type: SensitiveDataType::UnencryptedStorage,
            },
            DataPattern {
                pattern: "console.log".to_string(),
                vulnerability_type: SecurityIssueType::CryptographicFailures,
                description: "Potential sensitive data logging".to_string(),
                confidence: 0.3,
                severity: SecuritySeverity::Low,
                data_type: SensitiveDataType::PersonalInformation,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &DataPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::CryptographicFailures,
            pattern.vulnerability_type.clone(),
            "Sensitive Data Exposure".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.data_type))
        .with_architectural_correlation(
            OwaspCategory::CryptographicFailures
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(data_type: &SensitiveDataType) -> String {
        match data_type {
            SensitiveDataType::CreditCard => {
                "Implement PCI DSS compliance and encrypt credit card data".to_string()
            }
            SensitiveDataType::SocialSecurityNumber => {
                "Encrypt SSN data and limit access to authorized personnel".to_string()
            }
            SensitiveDataType::ApiKeys => {
                "Store API keys in environment variables or secure key management systems"
                    .to_string()
            }
            SensitiveDataType::DatabaseCredentials => {
                "Use connection pooling and secure credential storage".to_string()
            }
            SensitiveDataType::PersonalInformation => {
                "Implement data classification and encryption for PII".to_string()
            }
            SensitiveDataType::WeakEncryption => {
                "Use strong encryption algorithms like AES-256 or modern alternatives".to_string()
            }
            SensitiveDataType::UnencryptedStorage => {
                "Encrypt sensitive data at rest and in transit".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "encrypt", "hash", "secure", "aes", "sha256", "bcrypt", "vault", "keystore", "env",
            "config",
        ];

        safe_indicators
            .iter()
            .any(|&indicator| line.to_lowercase().contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for SensitiveDataDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.data_patterns.get(&file.language) {
            let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
                AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
            })?;

            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if line.contains(&pattern.pattern) && !self.is_likely_safe(line) {
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
    async fn test_api_key_detection() {
        let detector = SensitiveDataDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            content: "let api_key = \"sk-1234567890abcdef\";".to_string(),
            tree: None,
        };

        std::fs::write("test.rs", "let api_key = \"sk-1234567890abcdef\";").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.rs").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(
            vulnerabilities[0].category,
            OwaspCategory::CryptographicFailures
        );
    }
}
