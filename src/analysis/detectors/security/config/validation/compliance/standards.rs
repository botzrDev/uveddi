//! Compliance standards definitions
//!
//! This module defines various compliance standards including OWASP, CIS, and NIST.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::ConfigSeverity;
use super::types::{ComplianceRequirement, ComplianceStandard};
use std::collections::HashMap;

/// Standards builder for compliance validation
pub struct StandardsBuilder;

impl StandardsBuilder {
    /// Build all compliance standards based on configuration
    pub fn build_compliance_standards(config: &ConfigSecurityConfig) -> Vec<ComplianceStandard> {
        let mut standards = Vec::new();

        standards.push(Self::build_owasp_top_10());
        standards.push(Self::build_cis_controls(config));
        standards.push(Self::build_nist_csf(config));

        standards
    }

    /// Build OWASP Top 10 2021 standard
    fn build_owasp_top_10() -> ComplianceStandard {
        ComplianceStandard {
            name: "OWASP-Top-10-2021".to_string(),
            enabled: true,
            version: "2021".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "A01".to_string(),
                    title: "Broken Access Control".to_string(),
                    description: "Restrictions on what authenticated users are allowed to do are often not properly enforced".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::High, 0.9), (ConfigSeverity::Medium, 0.7)]),
                    applicable_cwe_ids: vec![22, 284, 285, 639, 732, 863],
                    applicable_tags: vec!["access-control".to_string(), "authorization".to_string(), "permission".to_string()],
                    remediation_guidance: "Implement proper access controls and principle of least privilege".to_string(),
                },
                ComplianceRequirement {
                    id: "A02".to_string(),
                    title: "Cryptographic Failures".to_string(),
                    description: "Failures related to cryptography which often lead to exposure of sensitive data".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::Critical, 1.0), (ConfigSeverity::High, 0.8)]),
                    applicable_cwe_ids: vec![259, 319, 326, 327, 328, 331, 335, 798],
                    applicable_tags: vec!["crypto".to_string(), "encryption".to_string(), "credential".to_string()],
                    remediation_guidance: "Use strong encryption algorithms and proper key management".to_string(),
                },
                ComplianceRequirement {
                    id: "A03".to_string(),
                    title: "Injection".to_string(),
                    description: "Application is vulnerable to injection attacks".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::Critical, 1.0), (ConfigSeverity::High, 0.9)]),
                    applicable_cwe_ids: vec![79, 89, 73, 74, 77, 78, 94, 95, 116, 564],
                    applicable_tags: vec!["injection".to_string(), "command-injection".to_string(), "sql-injection".to_string()],
                    remediation_guidance: "Use parameterized queries and input validation".to_string(),
                },
                ComplianceRequirement {
                    id: "A05".to_string(),
                    title: "Security Misconfiguration".to_string(),
                    description: "Security misconfiguration is a consequence of insecure default configurations".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::High, 0.8), (ConfigSeverity::Medium, 0.6)]),
                    applicable_cwe_ids: vec![16, 260, 315, 489, 1188],
                    applicable_tags: vec!["misconfiguration".to_string(), "default-config".to_string(), "debug".to_string()],
                    remediation_guidance: "Review and harden all security configurations".to_string(),
                },
                ComplianceRequirement {
                    id: "A07".to_string(),
                    title: "Identification and Authentication Failures".to_string(),
                    description: "Confirmation of the user's identity, authentication, and session management is critical".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::High, 0.9), (ConfigSeverity::Medium, 0.7)]),
                    applicable_cwe_ids: vec![287, 288, 306, 307, 521, 522, 620, 640],
                    applicable_tags: vec!["authentication".to_string(), "session".to_string(), "password".to_string()],
                    remediation_guidance: "Implement multi-factor authentication and secure session management".to_string(),
                },
                ComplianceRequirement {
                    id: "A10".to_string(),
                    title: "Server-Side Request Forgery (SSRF)".to_string(),
                    description: "SSRF flaws occur whenever a web application is fetching a remote resource without validating the user-supplied URL".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::High, 0.8), (ConfigSeverity::Medium, 0.6)]),
                    applicable_cwe_ids: vec![918],
                    applicable_tags: vec!["ssrf".to_string(), "url-injection".to_string()],
                    remediation_guidance: "Validate and whitelist URLs and network destinations".to_string(),
                },
            ],
        }
    }

    /// Build CIS Controls v8 standard
    fn build_cis_controls(config: &ConfigSecurityConfig) -> ComplianceStandard {
        ComplianceStandard {
            name: "CIS-Controls-v8".to_string(),
            enabled: config.enable_compliance_validation,
            version: "8.0".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "3.3".to_string(),
                    title: "Configure Data Access Control Lists".to_string(),
                    description:
                        "Configure data access control lists based on a user's need to know"
                            .to_string(),
                    severity_mapping: create_severity_mapping(&[
                        (ConfigSeverity::High, 0.8),
                        (ConfigSeverity::Medium, 0.6),
                    ]),
                    applicable_cwe_ids: vec![732, 284],
                    applicable_tags: vec![
                        "access-control".to_string(),
                        "file-permissions".to_string(),
                    ],
                    remediation_guidance: "Implement least privilege access controls".to_string(),
                },
                ComplianceRequirement {
                    id: "3.11".to_string(),
                    title: "Encrypt Sensitive Data at Rest".to_string(),
                    description: "Encrypt sensitive data at rest".to_string(),
                    severity_mapping: create_severity_mapping(&[
                        (ConfigSeverity::High, 0.9),
                        (ConfigSeverity::Medium, 0.7),
                    ]),
                    applicable_cwe_ids: vec![311, 326, 327],
                    applicable_tags: vec!["encryption".to_string(), "data-at-rest".to_string()],
                    remediation_guidance: "Use strong encryption for sensitive data storage"
                        .to_string(),
                },
                ComplianceRequirement {
                    id: "16.7".to_string(),
                    title: "Establish and Maintain a Vulnerability Management Process".to_string(),
                    description: "Establish and maintain a vulnerability management process"
                        .to_string(),
                    severity_mapping: create_severity_mapping(&[
                        (ConfigSeverity::Medium, 0.6),
                        (ConfigSeverity::Low, 0.4),
                    ]),
                    applicable_cwe_ids: vec![1188],
                    applicable_tags: vec![
                        "vulnerability-management".to_string(),
                        "patching".to_string(),
                    ],
                    remediation_guidance:
                        "Implement regular vulnerability scanning and remediation processes"
                            .to_string(),
                },
            ],
        }
    }

    /// Build NIST Cybersecurity Framework standard
    fn build_nist_csf(config: &ConfigSecurityConfig) -> ComplianceStandard {
        ComplianceStandard {
            name: "NIST-CSF".to_string(),
            enabled: config.enable_compliance_validation,
            version: "1.1".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "PR.AC-1".to_string(),
                    title: "Identities and credentials are issued, managed, verified, revoked, and audited".to_string(),
                    description: "Identity and credential management processes".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::High, 0.8), (ConfigSeverity::Medium, 0.6)]),
                    applicable_cwe_ids: vec![287, 306, 521, 798],
                    applicable_tags: vec!["credential".to_string(), "authentication".to_string()],
                    remediation_guidance: "Implement proper identity and credential management".to_string(),
                },
                ComplianceRequirement {
                    id: "PR.DS-1".to_string(),
                    title: "Data-at-rest is protected".to_string(),
                    description: "Data protection for stored information".to_string(),
                    severity_mapping: create_severity_mapping(&[(ConfigSeverity::High, 0.9), (ConfigSeverity::Medium, 0.7)]),
                    applicable_cwe_ids: vec![311, 326, 327, 359],
                    applicable_tags: vec!["encryption".to_string(), "data-protection".to_string()],
                    remediation_guidance: "Encrypt sensitive data at rest using strong algorithms".to_string(),
                },
            ],
        }
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
    fn test_owasp_standard_creation() {
        let standard = StandardsBuilder::build_owasp_top_10();
        assert_eq!(standard.name, "OWASP-Top-10-2021");
        assert!(standard.enabled);
        assert!(!standard.requirements.is_empty());
    }

    #[test]
    fn test_cis_controls_creation() {
        let config = ConfigSecurityConfig::default();
        let standard = StandardsBuilder::build_cis_controls(&config);
        assert_eq!(standard.name, "CIS-Controls-v8");
        assert!(!standard.requirements.is_empty());
    }

    #[test]
    fn test_nist_csf_creation() {
        let config = ConfigSecurityConfig::default();
        let standard = StandardsBuilder::build_nist_csf(&config);
        assert_eq!(standard.name, "NIST-CSF");
        assert!(!standard.requirements.is_empty());
    }

    #[test]
    fn test_all_standards_building() {
        let config = ConfigSecurityConfig::default();
        let standards = StandardsBuilder::build_compliance_standards(&config);
        assert_eq!(standards.len(), 3);
        assert!(standards.iter().any(|s| s.name == "OWASP-Top-10-2021"));
        assert!(standards.iter().any(|s| s.name == "CIS-Controls-v8"));
        assert!(standards.iter().any(|s| s.name == "NIST-CSF"));
    }
}
