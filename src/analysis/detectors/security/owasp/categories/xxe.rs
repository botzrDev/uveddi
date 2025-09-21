//! A04:2021 – XML External Entities (XXE) Detector
//!
//! Detects XML External Entity vulnerabilities and insecure XML processing.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// A04:2021 – XML External Entities Detector
pub struct XXEDetector {
    xxe_patterns: HashMap<SourceLanguage, Vec<XXEPattern>>,
}

#[derive(Debug, Clone)]
pub struct XXEPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub xxe_type: XXEIssueType,
}

#[derive(Debug, Clone)]
pub enum XXEIssueType {
    ExternalEntityReference,
    DtdProcessing,
    XmlParserMisconfiguration,
    UnsafeXmlDeserialization,
    XmlInjection,
}

impl XXEDetector {
    pub fn new() -> Self {
        Self {
            xxe_patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<XXEPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<XXEPattern> {
        vec![
            XXEPattern {
                pattern: "quick_xml::Reader".to_string(),
                vulnerability_type: SecurityIssueType::XmlExternalEntities,
                description: "XML parser without XXE protection".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::Medium,
                xxe_type: XXEIssueType::XmlParserMisconfiguration,
            },
            XXEPattern {
                pattern: "<!ENTITY".to_string(),
                vulnerability_type: SecurityIssueType::XmlExternalEntities,
                description: "XML entity declaration detected".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                xxe_type: XXEIssueType::ExternalEntityReference,
            },
        ]
    }

    fn python_patterns() -> Vec<XXEPattern> {
        vec![
            XXEPattern {
                pattern: "xml.etree.ElementTree.parse".to_string(),
                vulnerability_type: SecurityIssueType::XmlExternalEntities,
                description: "Unsafe XML parsing with ElementTree".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                xxe_type: XXEIssueType::XmlParserMisconfiguration,
            },
            XXEPattern {
                pattern: "xml.dom.minidom.parse".to_string(),
                vulnerability_type: SecurityIssueType::XmlExternalEntities,
                description: "Unsafe XML parsing with minidom".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                xxe_type: XXEIssueType::XmlParserMisconfiguration,
            },
            XXEPattern {
                pattern: "<!DOCTYPE".to_string(),
                vulnerability_type: SecurityIssueType::XmlExternalEntities,
                description: "DOCTYPE declaration may enable XXE".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::Medium,
                xxe_type: XXEIssueType::DtdProcessing,
            },
        ]
    }

    fn javascript_patterns() -> Vec<XXEPattern> {
        vec![
            XXEPattern {
                pattern: "DOMParser()".to_string(),
                vulnerability_type: SecurityIssueType::XmlExternalEntities,
                description: "DOMParser may be vulnerable to XXE".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::Medium,
                xxe_type: XXEIssueType::XmlParserMisconfiguration,
            },
            XXEPattern {
                pattern: "parseFromString".to_string(),
                vulnerability_type: SecurityIssueType::XmlExternalEntities,
                description: "XML parsing without XXE protection".to_string(),
                confidence: 0.5,
                severity: SecuritySeverity::Medium,
                xxe_type: XXEIssueType::UnsafeXmlDeserialization,
            },
        ]
    }

    fn create_vulnerability(
        &self,
        pattern: &XXEPattern,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::InsecureDesign,
            pattern.vulnerability_type.clone(),
            "XXE Vulnerability".to_string(),
            pattern.description.clone(),
            location,
        )
        .with_confidence(pattern.confidence)
        .with_severity(pattern.severity)
        .with_remediation(Self::get_remediation_advice(&pattern.xxe_type))
        .with_architectural_correlation(
            OwaspCategory::InsecureDesign
                .related_anti_patterns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    pub fn get_remediation_advice(xxe_type: &XXEIssueType) -> String {
        match xxe_type {
            XXEIssueType::ExternalEntityReference => {
                "Disable external entity processing in XML parsers".to_string()
            }
            XXEIssueType::DtdProcessing => {
                "Disable DTD processing or use a safe XML parser configuration".to_string()
            }
            XXEIssueType::XmlParserMisconfiguration => {
                "Configure XML parsers to disable XXE vulnerabilities".to_string()
            }
            XXEIssueType::UnsafeXmlDeserialization => {
                "Use safe XML deserialization libraries and validate input".to_string()
            }
            XXEIssueType::XmlInjection => {
                "Validate and sanitize XML input before processing".to_string()
            }
        }
    }

    fn is_likely_safe(&self, line: &str) -> bool {
        let safe_indicators = [
            "disable_external_entities",
            "XMLConstants.FEATURE_SECURE_PROCESSING",
            "setFeature",
            "secure_xml",
            "defusedxml",
            "lxml_safe",
        ];

        safe_indicators
            .iter()
            .any(|&indicator| line.contains(indicator))
    }
}

#[async_trait::async_trait]
impl OwaspCategoryDetector for XXEDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();

        if let Some(patterns) = self.xxe_patterns.get(&file.language) {
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
    async fn test_xxe_detection() {
        let detector = XXEDetector::new();

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: "tree = xml.etree.ElementTree.parse('data.xml')".to_string(),
            tree: None,
        };

        std::fs::write("test.py", "tree = xml.etree.ElementTree.parse('data.xml')").unwrap();
        let vulnerabilities = detector.detect(&file).await.unwrap();
        std::fs::remove_file("test.py").unwrap();

        assert!(!vulnerabilities.is_empty());
        assert_eq!(vulnerabilities[0].category, OwaspCategory::InsecureDesign);
    }
}
