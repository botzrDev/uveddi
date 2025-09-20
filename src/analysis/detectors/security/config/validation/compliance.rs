//! Compliance validation for configuration security
//!
//! This module validates configuration security findings against
//! industry compliance standards like OWASP, CIS, NIST, etc.

use crate::analysis::AnalysisError;
use crate::analysis::detectors::security::types::SecurityIssue;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity};
use std::collections::HashMap;

/// Compliance validator for configuration security issues
pub struct ComplianceValidator {
    standards: Vec<ComplianceStandard>,
    config: ConfigSecurityConfig,
}

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

impl ComplianceValidator {
    /// Create a new compliance validator
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let standards = Self::build_compliance_standards(config);

        Ok(Self {
            standards,
            config: config.clone(),
        })
    }

    /// Enhance a configuration issue with compliance information
    pub fn enhance_issue(&self, mut issue: ConfigIssue) -> Result<ConfigIssue, AnalysisError> {
        if !self.config.enable_compliance_validation {
            return Ok(issue);
        }

        // Find applicable compliance requirements
        let mut compliance_mappings = Vec::new();
        let mut enhanced_remediation = issue.remediation.unwrap_or_default();

        for standard in &self.standards {
            if !standard.enabled {
                continue;
            }

            for requirement in &standard.requirements {
                if self.requirement_applies_to_issue(&issue, requirement) {
                    compliance_mappings.push(format!("{}:{}", standard.name, requirement.id));

                    // Enhance remediation with compliance guidance
                    if !requirement.remediation_guidance.is_empty() {
                        enhanced_remediation.push_str(&format!(
                            "\n\nCompliance Guidance ({}): {}",
                            standard.name,
                            requirement.remediation_guidance
                        ));
                    }
                }
            }
        }

        // Add compliance tags
        for mapping in compliance_mappings {
            issue = issue.with_tag(format!("compliance:{}", mapping));
        }

        if !enhanced_remediation.is_empty() {
            issue = issue.with_remediation(enhanced_remediation);
        }

        Ok(issue)
    }

    /// Enhance a security issue with compliance information
    pub fn enhance_security_issue(&self, issue: SecurityIssue) -> Result<SecurityIssue, AnalysisError> {
        // For now, return the issue as-is since SecurityIssue enhancement
        // would require modifying the SecurityIssue struct
        Ok(issue)
    }

    /// Get the count of active compliance standards
    pub fn get_active_standards_count(&self) -> usize {
        self.standards.iter().filter(|s| s.enabled).count()
    }

    /// Get compliance coverage report
    pub fn get_compliance_coverage(&self, issues: &[ConfigIssue]) -> HashMap<String, ComplianceCoverage> {
        let mut coverage = HashMap::new();

        for standard in &self.standards {
            if !standard.enabled {
                continue;
            }

            let mut covered_requirements = 0;
            let total_requirements = standard.requirements.len();

            for requirement in &standard.requirements {
                let has_applicable_issue = issues.iter().any(|issue| {
                    self.requirement_applies_to_issue(issue, requirement)
                });

                if has_applicable_issue {
                    covered_requirements += 1;
                }
            }

            coverage.insert(standard.name.clone(), ComplianceCoverage {
                standard_name: standard.name.clone(),
                version: standard.version.clone(),
                total_requirements,
                covered_requirements,
                coverage_percentage: (covered_requirements as f64 / total_requirements as f64) * 100.0,
            });
        }

        coverage
    }

    fn build_compliance_standards(config: &ConfigSecurityConfig) -> Vec<ComplianceStandard> {
        let mut standards = Vec::new();

        // OWASP Top 10 2021
        standards.push(ComplianceStandard {
            name: "OWASP-Top-10-2021".to_string(),
            enabled: true,
            version: "2021".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "A01".to_string(),
                    title: "Broken Access Control".to_string(),
                    description: "Restrictions on what authenticated users are allowed to do are often not properly enforced".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.9), (ConfigSeverity::Medium, 0.7)]),
                    applicable_cwe_ids: vec![22, 284, 285, 639, 732, 863],
                    applicable_tags: vec!["access-control".to_string(), "authorization".to_string(), "permission".to_string()],
                    remediation_guidance: "Implement proper access controls and principle of least privilege".to_string(),
                },
                ComplianceRequirement {
                    id: "A02".to_string(),
                    title: "Cryptographic Failures".to_string(),
                    description: "Failures related to cryptography which often lead to exposure of sensitive data".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::Critical, 1.0), (ConfigSeverity::High, 0.8)]),
                    applicable_cwe_ids: vec![259, 319, 326, 327, 328, 331, 335, 798],
                    applicable_tags: vec!["crypto".to_string(), "encryption".to_string(), "credential".to_string()],
                    remediation_guidance: "Use strong encryption algorithms and proper key management".to_string(),
                },
                ComplianceRequirement {
                    id: "A03".to_string(),
                    title: "Injection".to_string(),
                    description: "Application is vulnerable to injection attacks".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::Critical, 1.0), (ConfigSeverity::High, 0.9)]),
                    applicable_cwe_ids: vec![79, 89, 73, 74, 77, 78, 94, 95, 116, 564],
                    applicable_tags: vec!["injection".to_string(), "command-injection".to_string(), "sql-injection".to_string()],
                    remediation_guidance: "Use parameterized queries and input validation".to_string(),
                },
                ComplianceRequirement {
                    id: "A05".to_string(),
                    title: "Security Misconfiguration".to_string(),
                    description: "Security misconfiguration is a consequence of insecure default configurations".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.8), (ConfigSeverity::Medium, 0.6)]),
                    applicable_cwe_ids: vec![16, 260, 315, 489, 1188],
                    applicable_tags: vec!["misconfiguration".to_string(), "default-config".to_string(), "debug".to_string()],
                    remediation_guidance: "Review and harden all security configurations".to_string(),
                },
                ComplianceRequirement {
                    id: "A07".to_string(),
                    title: "Identification and Authentication Failures".to_string(),
                    description: "Confirmation of the user's identity, authentication, and session management is critical".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.9), (ConfigSeverity::Medium, 0.7)]),
                    applicable_cwe_ids: vec![287, 288, 306, 307, 521, 522, 620, 640],
                    applicable_tags: vec!["authentication".to_string(), "session".to_string(), "password".to_string()],
                    remediation_guidance: "Implement multi-factor authentication and secure session management".to_string(),
                },
                ComplianceRequirement {
                    id: "A10".to_string(),
                    title: "Server-Side Request Forgery (SSRF)".to_string(),
                    description: "SSRF flaws occur whenever a web application is fetching a remote resource without validating the user-supplied URL".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.8), (ConfigSeverity::Medium, 0.6)]),
                    applicable_cwe_ids: vec![918],
                    applicable_tags: vec!["ssrf".to_string(), "url-injection".to_string()],
                    remediation_guidance: "Validate and whitelist URLs and network destinations".to_string(),
                },
            ],
        });

        // CIS Controls v8
        standards.push(ComplianceStandard {
            name: "CIS-Controls-v8".to_string(),
            enabled: config.enable_compliance_validation,
            version: "8.0".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "3.3".to_string(),
                    title: "Configure Data Access Control Lists".to_string(),
                    description: "Configure data access control lists based on a user's need to know".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.8), (ConfigSeverity::Medium, 0.6)]),
                    applicable_cwe_ids: vec![732, 284],
                    applicable_tags: vec!["access-control".to_string(), "file-permissions".to_string()],
                    remediation_guidance: "Implement least privilege access controls".to_string(),
                },
                ComplianceRequirement {
                    id: "3.11".to_string(),
                    title: "Encrypt Sensitive Data at Rest".to_string(),
                    description: "Encrypt sensitive data at rest".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.9), (ConfigSeverity::Medium, 0.7)]),
                    applicable_cwe_ids: vec![311, 326, 327],
                    applicable_tags: vec!["encryption".to_string(), "data-at-rest".to_string()],
                    remediation_guidance: "Use strong encryption for sensitive data storage".to_string(),
                },
                ComplianceRequirement {
                    id: "16.7".to_string(),
                    title: "Establish and Maintain a Vulnerability Management Process".to_string(),
                    description: "Establish and maintain a vulnerability management process".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::Medium, 0.6), (ConfigSeverity::Low, 0.4)]),
                    applicable_cwe_ids: vec![1188],
                    applicable_tags: vec!["vulnerability-management".to_string(), "patching".to_string()],
                    remediation_guidance: "Implement regular vulnerability scanning and remediation processes".to_string(),
                },
            ],
        });

        // NIST Cybersecurity Framework
        standards.push(ComplianceStandard {
            name: "NIST-CSF".to_string(),
            enabled: config.enable_compliance_validation,
            version: "1.1".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "PR.AC-1".to_string(),
                    title: "Identities and credentials are issued, managed, verified, revoked, and audited".to_string(),
                    description: "Identity and credential management processes".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.8), (ConfigSeverity::Medium, 0.6)]),
                    applicable_cwe_ids: vec![287, 306, 521, 798],
                    applicable_tags: vec!["credential".to_string(), "authentication".to_string()],
                    remediation_guidance: "Implement proper identity and credential management".to_string(),
                },
                ComplianceRequirement {
                    id: "PR.DS-1".to_string(),
                    title: "Data-at-rest is protected".to_string(),
                    description: "Data protection for stored information".to_string(),
                    severity_mapping: Self::create_severity_mapping(&[(ConfigSeverity::High, 0.9), (ConfigSeverity::Medium, 0.7)]),
                    applicable_cwe_ids: vec![311, 326, 327, 359],
                    applicable_tags: vec!["encryption".to_string(), "data-protection".to_string()],
                    remediation_guidance: "Encrypt sensitive data at rest using strong algorithms".to_string(),
                },
            ],
        });

        standards
    }

    fn create_severity_mapping(mappings: &[(ConfigSeverity, f64)]) -> HashMap<ConfigSeverity, f64> {
        mappings.iter().cloned().collect()
    }

    fn requirement_applies_to_issue(&self, issue: &ConfigIssue, requirement: &ComplianceRequirement) -> bool {
        // Check CWE ID match
        if let Some(cwe_id) = issue.cwe_id {
            if requirement.applicable_cwe_ids.contains(&cwe_id) {
                return true;
            }
        }

        // Check tag match
        for req_tag in &requirement.applicable_tags {
            if issue.tags.iter().any(|issue_tag| issue_tag.contains(req_tag)) {
                return true;
            }
        }

        // Check severity threshold
        if let Some(&score) = requirement.severity_mapping.get(&issue.severity) {
            return score > 0.5; // Threshold for applicability
        }

        false
    }
}

/// Compliance coverage information
#[derive(Debug, Clone)]
pub struct ComplianceCoverage {
    pub standard_name: String,
    pub version: String,
    pub total_requirements: usize,
    pub covered_requirements: usize,
    pub coverage_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_enhancement() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();

        let issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.9,
            "Hardcoded Password",
            "Password found in configuration",
        ).with_cwe(798)
         .with_tag("credential");

        let enhanced = validator.enhance_issue(issue).unwrap();

        // Should have compliance tags
        let compliance_tags: Vec<_> = enhanced.tags.iter()
            .filter(|tag| tag.starts_with("compliance:"))
            .collect();
        assert!(!compliance_tags.is_empty());

        // Should have enhanced remediation
        assert!(enhanced.remediation.is_some());
        let remediation = enhanced.remediation.unwrap();
        assert!(remediation.contains("Compliance Guidance"));
    }

    #[test]
    fn test_compliance_coverage() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();

        let issues = vec![
            ConfigIssue::new(
                ConfigSeverity::High,
                0.9,
                "Weak Crypto",
                "Weak encryption detected",
            ).with_cwe(327).with_tag("crypto"),
            ConfigIssue::new(
                ConfigSeverity::Medium,
                0.8,
                "Access Control Issue",
                "Permission issue detected",
            ).with_cwe(732).with_tag("access-control"),
        ];

        let coverage = validator.get_compliance_coverage(&issues);
        assert!(!coverage.is_empty());

        // Check OWASP coverage
        if let Some(owasp_coverage) = coverage.get("OWASP-Top-10-2021") {
            assert!(owasp_coverage.covered_requirements > 0);
            assert!(owasp_coverage.coverage_percentage > 0.0);
        }
    }

    #[test]
    fn test_requirement_applicability() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();

        let crypto_issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.9,
            "Weak Encryption",
            "MD5 hash detected",
        ).with_cwe(327).with_tag("crypto");

        // Find OWASP A02 requirement
        let owasp_standard = validator.standards.iter()
            .find(|s| s.name == "OWASP-Top-10-2021")
            .unwrap();
        let a02_requirement = owasp_standard.requirements.iter()
            .find(|r| r.id == "A02")
            .unwrap();

        assert!(validator.requirement_applies_to_issue(&crypto_issue, a02_requirement));
    }

    #[test]
    fn test_active_standards_count() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();

        let count = validator.get_active_standards_count();
        assert!(count > 0);
    }

    #[test]
    fn test_compliance_disabled() {
        let config = ConfigSecurityConfig {
            enable_compliance_validation: false,
            ..Default::default()
        };
        let validator = ComplianceValidator::new(&config).unwrap();

        let issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.9,
            "Test Issue",
            "Test description",
        );

        let enhanced = validator.enhance_issue(issue.clone()).unwrap();

        // Should be unchanged when compliance is disabled
        assert_eq!(enhanced.tags.len(), issue.tags.len());
        assert_eq!(enhanced.remediation, issue.remediation);
    }
}