//! OWASP compliance scanner
//!
//! This module provides scanning functionality for OWASP compliance assessment.

use super::super::types::{OwaspCategory, OwaspVulnerability};
use std::collections::HashMap;

/// OWASP compliance scanner for calculating compliance percentages
pub struct OwaspComplianceScanner {
    category_weights: HashMap<OwaspCategory, f64>,
}

impl OwaspComplianceScanner {
    pub fn new() -> Self {
        let mut category_weights = HashMap::new();

        // Equal weight for all OWASP Top 10 categories
        let weight = 1.0 / 10.0;
        category_weights.insert(OwaspCategory::BrokenAccessControl, weight);
        category_weights.insert(OwaspCategory::CryptographicFailures, weight);
        category_weights.insert(OwaspCategory::Injection, weight);
        category_weights.insert(OwaspCategory::InsecureDesign, weight);
        category_weights.insert(OwaspCategory::SecurityMisconfiguration, weight);
        category_weights.insert(OwaspCategory::VulnerableComponents, weight);
        category_weights.insert(OwaspCategory::AuthenticationFailures, weight);
        category_weights.insert(OwaspCategory::DataIntegrityFailures, weight);
        category_weights.insert(OwaspCategory::LoggingFailures, weight);
        category_weights.insert(OwaspCategory::ServerSideRequestForgery, weight);

        Self { category_weights }
    }

    /// Calculate OWASP compliance score (0.0 = non-compliant, 1.0 = fully compliant)
    pub fn calculate_compliance_score(&self, vulnerabilities: &[OwaspVulnerability]) -> f64 {
        if vulnerabilities.is_empty() {
            return 1.0; // No vulnerabilities = full compliance
        }

        let mut category_issues: HashMap<OwaspCategory, Vec<&OwaspVulnerability>> = HashMap::new();

        // Group vulnerabilities by category
        for vuln in vulnerabilities {
            category_issues.entry(vuln.category.clone()).or_default().push(vuln);
        }

        let mut compliance_score = 1.0;

        // Reduce compliance score based on vulnerabilities in each category
        for (category, weight) in &self.category_weights {
            if let Some(issues) = category_issues.get(category) {
                let category_penalty = self.calculate_category_penalty(issues);
                compliance_score -= weight * category_penalty;
            }
        }

        compliance_score.max(0.0)
    }

    fn calculate_category_penalty(&self, vulnerabilities: &[&OwaspVulnerability]) -> f64 {
        if vulnerabilities.is_empty() {
            return 0.0;
        }

        // Calculate penalty based on number and severity of vulnerabilities
        let total_severity_score: f64 = vulnerabilities
            .iter()
            .map(|vuln| self.severity_to_score(&vuln.severity))
            .sum();

        let avg_severity = total_severity_score / vulnerabilities.len() as f64;
        let count_factor = (vulnerabilities.len() as f64).min(10.0) / 10.0;

        (avg_severity * count_factor).min(1.0)
    }

    fn severity_to_score(&self, severity: &crate::analysis::detectors::security::types::SecuritySeverity) -> f64 {
        use crate::analysis::detectors::security::types::SecuritySeverity;
        match severity {
            SecuritySeverity::Critical => 1.0,
            SecuritySeverity::High => 0.8,
            SecuritySeverity::Medium => 0.5,
            SecuritySeverity::Low => 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, SecuritySeverity};
    use std::path::PathBuf;

    #[test]
    fn test_compliance_score_calculation() {
        let scanner = OwaspComplianceScanner::new();

        // No vulnerabilities = full compliance
        assert_eq!(scanner.calculate_compliance_score(&[]), 1.0);

        // Test with some vulnerabilities
        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);
        let vuln = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "Test".to_string(),
            "Test vuln".to_string(),
            location,
        ).with_severity(SecuritySeverity::High);

        let score = scanner.calculate_compliance_score(&[vuln]);
        assert!(score < 1.0);
        assert!(score > 0.0);
    }
}