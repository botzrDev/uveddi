//! Compliance types and structures
//!
//! This module defines the core types for compliance validation.

use super::super::super::types::ConfigSeverity;
use std::collections::HashMap;

/// Compliance standard definition
#[derive(Debug, Clone)]
pub struct ComplianceStandard {
    pub name: String,
    pub enabled: bool,
    pub version: String,
    pub requirements: Vec<ComplianceRequirement>,
}

/// Individual compliance requirement
#[derive(Debug, Clone)]
pub struct ComplianceRequirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity_mapping: HashMap<ConfigSeverity, f64>, // Severity to compliance score mapping
    pub applicable_cwe_ids: Vec<u32>,
    pub applicable_tags: Vec<String>,
    pub remediation_guidance: String,
}

impl ComplianceStandard {
    /// Create a new compliance standard
    pub fn new(name: String, version: String, enabled: bool) -> Self {
        Self {
            name,
            enabled,
            version,
            requirements: Vec::new(),
        }
    }

    /// Add a requirement to this standard
    pub fn add_requirement(mut self, requirement: ComplianceRequirement) -> Self {
        self.requirements.push(requirement);
        self
    }

    /// Get all enabled requirements
    pub fn get_enabled_requirements(&self) -> Vec<&ComplianceRequirement> {
        if self.enabled {
            self.requirements.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Check if this standard is applicable for a given set of tags
    pub fn is_applicable_for_tags(&self, tags: &[String]) -> bool {
        if !self.enabled {
            return false;
        }

        self.requirements.iter().any(|req| {
            req.applicable_tags
                .iter()
                .any(|tag| tags.iter().any(|issue_tag| issue_tag.contains(tag)))
        })
    }
}

impl ComplianceRequirement {
    /// Create a new compliance requirement
    pub fn new(id: String, title: String, description: String) -> Self {
        Self {
            id,
            title,
            description,
            severity_mapping: HashMap::new(),
            applicable_cwe_ids: Vec::new(),
            applicable_tags: Vec::new(),
            remediation_guidance: String::new(),
        }
    }

    /// Add severity mapping
    pub fn with_severity_mapping(mut self, mappings: &[(ConfigSeverity, f64)]) -> Self {
        self.severity_mapping = create_severity_mapping(mappings);
        self
    }

    /// Add CWE IDs
    pub fn with_cwe_ids(mut self, cwe_ids: Vec<u32>) -> Self {
        self.applicable_cwe_ids = cwe_ids;
        self
    }

    /// Add applicable tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.applicable_tags = tags;
        self
    }

    /// Add remediation guidance
    pub fn with_remediation(mut self, guidance: String) -> Self {
        self.remediation_guidance = guidance;
        self
    }

    /// Get compliance score for a severity
    pub fn get_score_for_severity(&self, severity: &ConfigSeverity) -> f64 {
        self.severity_mapping.get(severity).copied().unwrap_or(0.0)
    }

    /// Check if this requirement applies to a CWE ID
    pub fn applies_to_cwe(&self, cwe_id: u32) -> bool {
        self.applicable_cwe_ids.contains(&cwe_id)
    }

    /// Check if this requirement applies to any of the given tags
    pub fn applies_to_tags(&self, tags: &[String]) -> bool {
        self.applicable_tags.iter().any(|req_tag| {
            tags.iter().any(|issue_tag| {
                issue_tag.to_lowercase().contains(&req_tag.to_lowercase())
                    || req_tag.to_lowercase().contains(&issue_tag.to_lowercase())
            })
        })
    }
}

/// Helper function to create severity mapping
fn create_severity_mapping(mappings: &[(ConfigSeverity, f64)]) -> HashMap<ConfigSeverity, f64> {
    mappings.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_standard_creation() {
        let standard = ComplianceStandard::new("OWASP".to_string(), "2021".to_string(), true);

        assert_eq!(standard.name, "OWASP");
        assert_eq!(standard.version, "2021");
        assert!(standard.enabled);
        assert!(standard.requirements.is_empty());
    }

    #[test]
    fn test_compliance_requirement_creation() {
        let requirement = ComplianceRequirement::new(
            "A01".to_string(),
            "Injection".to_string(),
            "Test requirement".to_string(),
        )
        .with_severity_mapping(&[(ConfigSeverity::High, 9.0), (ConfigSeverity::Medium, 6.0)])
        .with_cwe_ids(vec![79, 89])
        .with_tags(vec!["injection".to_string()]);

        assert_eq!(requirement.id, "A01");
        assert_eq!(
            requirement.get_score_for_severity(&ConfigSeverity::High),
            9.0
        );
        assert!(requirement.applies_to_cwe(79));
        assert!(requirement.applies_to_tags(&["sql-injection".to_string()]));
    }

    #[test]
    fn test_standard_with_requirements() {
        let requirement = ComplianceRequirement::new(
            "A01".to_string(),
            "Test".to_string(),
            "Test requirement".to_string(),
        );

        let standard =
            ComplianceStandard::new("Test Standard".to_string(), "1.0".to_string(), true)
                .add_requirement(requirement);

        assert_eq!(standard.requirements.len(), 1);
        assert_eq!(standard.get_enabled_requirements().len(), 1);
    }

    #[test]
    fn test_disabled_standard() {
        let standard =
            ComplianceStandard::new("Disabled Standard".to_string(), "1.0".to_string(), false);

        assert!(standard.get_enabled_requirements().is_empty());
        assert!(!standard.is_applicable_for_tags(&["any-tag".to_string()]));
    }
}
