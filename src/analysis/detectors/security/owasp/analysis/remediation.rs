//! Remediation guidance engine
//!
//! This module provides specific fix recommendations for OWASP vulnerabilities.

use super::super::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::SecuritySeverity;
use std::collections::HashMap;

/// Engine for generating remediation guidance for OWASP vulnerabilities
pub struct RemediationEngine {
    category_remediations: HashMap<OwaspCategory, Vec<String>>,
}

impl RemediationEngine {
    pub fn new() -> Self {
        let mut category_remediations = HashMap::new();

        // A01: Broken Access Control
        category_remediations.insert(
            OwaspCategory::BrokenAccessControl,
            vec![
                "Implement proper access control matrices and role-based permissions".to_string(),
                "Use deny-by-default approach for access control".to_string(),
                "Implement centralized authorization mechanisms".to_string(),
                "Log access control failures and alert administrators".to_string(),
                "Rate limit API calls to minimize automated attacks".to_string(),
            ],
        );

        // A02: Cryptographic Failures
        category_remediations.insert(
            OwaspCategory::CryptographicFailures,
            vec![
                "Use strong, modern cryptographic algorithms (AES-256, SHA-256)".to_string(),
                "Implement proper key management and rotation".to_string(),
                "Use secure random number generators for cryptographic operations".to_string(),
                "Encrypt sensitive data at rest and in transit".to_string(),
                "Disable weak protocols and cipher suites".to_string(),
            ],
        );

        // A03: Injection
        category_remediations.insert(
            OwaspCategory::Injection,
            vec![
                "Use parameterized queries and prepared statements".to_string(),
                "Implement input validation and sanitization".to_string(),
                "Use safe APIs that avoid interpreters entirely".to_string(),
                "Escape special characters using specific syntax for target interpreter".to_string(),
                "Implement positive server-side input validation".to_string(),
            ],
        );

        // A04: Insecure Design
        category_remediations.insert(
            OwaspCategory::InsecureDesign,
            vec![
                "Implement secure development lifecycle with threat modeling".to_string(),
                "Use established secure design patterns and reference architectures".to_string(),
                "Integrate security and privacy controls into all development stages".to_string(),
                "Implement proper error handling that doesn't leak information".to_string(),
                "Use unit and integration tests to validate all critical flows".to_string(),
            ],
        );

        // A05: Security Misconfiguration
        category_remediations.insert(
            OwaspCategory::SecurityMisconfiguration,
            vec![
                "Implement repeatable hardening processes".to_string(),
                "Remove unnecessary features, components, and services".to_string(),
                "Implement automated configuration verification".to_string(),
                "Segment application architecture for better security".to_string(),
                "Keep all components and dependencies up to date".to_string(),
            ],
        );

        // Add other categories with placeholder remediations
        for category in [
            OwaspCategory::VulnerableComponents,
            OwaspCategory::AuthenticationFailures,
            OwaspCategory::DataIntegrityFailures,
            OwaspCategory::LoggingFailures,
            OwaspCategory::ServerSideRequestForgery,
        ] {
            category_remediations.insert(
                category,
                vec!["Implement appropriate security controls for this category".to_string()],
            );
        }

        Self {
            category_remediations,
        }
    }

    /// Generate comprehensive remediation plan for a set of vulnerabilities
    pub fn generate_plan(&self, vulnerabilities: &[OwaspVulnerability]) -> Vec<String> {
        let mut remediation_plan = Vec::new();

        if vulnerabilities.is_empty() {
            return vec!["No vulnerabilities detected. Continue following security best practices.".to_string()];
        }

        // Group vulnerabilities by category
        let mut category_counts: HashMap<OwaspCategory, usize> = HashMap::new();
        let mut critical_vulns = Vec::new();

        for vuln in vulnerabilities {
            *category_counts.entry(vuln.category.clone()).or_insert(0) += 1;
            if vuln.severity == SecuritySeverity::Critical {
                critical_vulns.push(vuln);
            }
        }

        // Add urgent actions for critical vulnerabilities
        if !critical_vulns.is_empty() {
            remediation_plan.push(format!(
                "URGENT: {} critical vulnerabilities require immediate attention",
                critical_vulns.len()
            ));
        }

        // Add category-specific remediations
        for (category, count) in category_counts {
            if let Some(remediations) = self.category_remediations.get(&category) {
                remediation_plan.push(format!(
                    "{} ({} issues): {}",
                    category.identifier(),
                    count,
                    remediations.join("; ")
                ));
            }
        }

        // Add general security recommendations
        remediation_plan.extend([
            "Implement comprehensive security testing in CI/CD pipeline".to_string(),
            "Conduct regular security code reviews".to_string(),
            "Establish incident response procedures".to_string(),
            "Provide security training for development team".to_string(),
        ]);

        remediation_plan
    }

    /// Get specific remediation for a single vulnerability
    pub fn get_vulnerability_remediation(&self, vulnerability: &OwaspVulnerability) -> String {
        if let Some(remediation) = &vulnerability.remediation {
            return remediation.clone();
        }

        // Fallback to category-specific remediation
        if let Some(remediations) = self.category_remediations.get(&vulnerability.category) {
            return remediations.first().unwrap_or(&"Apply appropriate security controls".to_string()).clone();
        }

        "Review and apply appropriate security controls".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation};
    use std::path::PathBuf;

    #[test]
    fn test_remediation_plan_generation() {
        let engine = RemediationEngine::new();

        // Test empty vulnerabilities
        let plan = engine.generate_plan(&[]);
        assert_eq!(plan.len(), 1);
        assert!(plan[0].contains("No vulnerabilities"));

        // Test with vulnerabilities
        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);
        let vuln = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "Test".to_string(),
            "Test vuln".to_string(),
            location,
        ).with_severity(SecuritySeverity::Critical);

        let plan = engine.generate_plan(&[vuln]);
        assert!(plan.len() > 1);
        assert!(plan[0].contains("URGENT"));
    }

    #[test]
    fn test_vulnerability_remediation() {
        let engine = RemediationEngine::new();
        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);

        let vuln_with_remediation = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "Test".to_string(),
            "Test vuln".to_string(),
            location.clone(),
        ).with_remediation("Custom remediation".to_string());

        assert_eq!(
            engine.get_vulnerability_remediation(&vuln_with_remediation),
            "Custom remediation"
        );

        let vuln_without_remediation = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "Test".to_string(),
            "Test vuln".to_string(),
            location,
        );

        let remediation = engine.get_vulnerability_remediation(&vuln_without_remediation);
        assert!(remediation.contains("parameterized"));
    }
}