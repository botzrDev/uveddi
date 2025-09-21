//! A08:2021 – Insecure Deserialization Detector
//!
//! Detects insecure deserialization vulnerabilities and data integrity failures.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A08:2021 – Insecure Deserialization Detector
pub struct InsecureDeserializationDetector {
    deserialization_patterns: HashMap<SourceLanguage, Vec<DeserializationPattern>>,
}

#[derive(Debug, Clone)]
pub struct DeserializationPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub deserialization_type: DeserializationIssue,
}

#[derive(Debug, Clone)]
pub enum DeserializationIssue {
    UnsafeDeserialization,
    RemoteCodeExecution,
    ObjectInjection,
    DataTampering,
    IntegrityViolation,
    UnvalidatedInput,
}

impl InsecureDeserializationDetector {
    pub fn new() -> Self {
        Self {
            deserialization_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<DeserializationPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<DeserializationPattern> {
        vec![
            DeserializationPattern {
                pattern: "serde::from_str".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "Unsafe deserialization without validation".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                deserialization_type: DeserializationIssue::UnvalidatedInput,
            },
            DeserializationPattern {
                pattern: "bincode::deserialize".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "Binary deserialization that may be unsafe".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::High,
                deserialization_type: DeserializationIssue::UnsafeDeserialization,
            },
        ]
    }

    fn python_patterns() -> Vec<DeserializationPattern> {
        vec![
            DeserializationPattern {
                pattern: "pickle.loads(".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "Unsafe pickle deserialization allowing code execution".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                deserialization_type: DeserializationIssue::RemoteCodeExecution,
            },
            DeserializationPattern {
                pattern: "yaml.load(".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "Unsafe YAML loading allowing code execution".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Critical,
                deserialization_type: DeserializationIssue::RemoteCodeExecution,
            },
            DeserializationPattern {
                pattern: "marshal.loads(".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "Unsafe marshal deserialization".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                deserialization_type: DeserializationIssue::UnsafeDeserialization,
            },
        ]
    }

    fn javascript_patterns() -> Vec<DeserializationPattern> {
        vec![
            DeserializationPattern {
                pattern: "JSON.parse(".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "JSON parsing without validation".to_string(),
                confidence: 0.4,
                severity: SecuritySeverity::Low,
                deserialization_type: DeserializationIssue::UnvalidatedInput,
            },
            DeserializationPattern {
                pattern: "eval(".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "Dangerous eval() usage for deserialization".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                deserialization_type: DeserializationIssue::RemoteCodeExecution,
            },
            DeserializationPattern {
                pattern: "Function(".to_string(),
                vulnerability_type: SecurityIssueType::InsecureDeserialization,
                description: "Dynamic function creation from untrusted data".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                deserialization_type: DeserializationIssue::RemoteCodeExecution,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &DeserializationPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::DataIntegrityFailures,
            pattern.vulnerability_type.clone(),
            "Insecure Deserialization Vulnerability".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.deserialization_type))
        .with_architectural_correlation(
            OwaspCategory::DataIntegrityFailures
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(deserialization_type: &DeserializationIssue) -> String {
        match deserialization_type {
            DeserializationIssue::UnsafeDeserialization => {
                "Use safe deserialization libraries and validate data structure".to_string()
            }
            DeserializationIssue::RemoteCodeExecution => {
                "Avoid deserializing untrusted data; use safe data formats like JSON".to_string()
            }
            DeserializationIssue::ObjectInjection => {
                "Implement strict type checking and object validation".to_string()
            }
            DeserializationIssue::DataTampering => {
                "Use digital signatures to verify data integrity before deserialization".to_string()
            }
            DeserializationIssue::IntegrityViolation => {
                "Implement checksums and integrity validation for serialized data".to_string()
            }
            DeserializationIssue::UnvalidatedInput => {
                "Validate and sanitize all input before deserialization".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "safe_load",
            "yaml.safe_load",
            "validate",
            "schema",
            "whitelist",
            "allowlist",
            "verify",
            "signature",
        ];

        safe_indicators
            .iter()
            .any(|&indicator| line.to_lowercase().contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for InsecureDeserializationDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.deserialization_patterns.get(&file.language) {
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
    async fn test_pickle_detection() {
        let detector = InsecureDeserializationDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: "data = pickle.loads(user_input)".to_string(),
            tree: None,
        };

        std::fs::write("test.py", "data = pickle.loads(user_input)").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.py").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(
            vulnerabilities[0].category,
            OwaspCategory::DataIntegrityFailures
        );
    }
}
