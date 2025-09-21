//! CVSS-based risk severity calculation
//!
//! This module provides CVSS-based risk calculation for OWASP vulnerabilities.

use super::super::types::OwaspVulnerability;
use crate::analysis::detectors::security::types::SecuritySeverity;

/// Calculator for determining risk severity using CVSS methodology
pub struct SeverityCalculator {
    severity_weights: SeverityWeights,
}

#[derive(Debug, Clone)]
struct SeverityWeights {
    critical: f64,
    high: f64,
    medium: f64,
    low: f64,
}

impl Default for SeverityWeights {
    fn default() -> Self {
        Self {
            critical: 10.0,
            high: 7.0,
            medium: 4.0,
            low: 1.0,
        }
    }
}

impl SeverityCalculator {
    pub fn new() -> Self {
        Self {
            severity_weights: SeverityWeights::default(),
        }
    }

    /// Calculate overall risk score for a set of vulnerabilities
    pub fn calculate_overall_risk(&self, vulnerabilities: &[OwaspVulnerability]) -> f64 {
        if vulnerabilities.is_empty() {
            return 0.0;
        }

        let total_score: f64 = vulnerabilities
            .iter()
            .map(|vuln| self.calculate_vulnerability_risk(vuln))
            .sum();

        // Normalize to 0-10 scale
        let max_possible_score = vulnerabilities.len() as f64 * self.severity_weights.critical;
        if max_possible_score > 0.0 {
            (total_score / max_possible_score) * 10.0
        } else {
            0.0
        }
    }

    /// Calculate risk score for a single vulnerability
    pub fn calculate_vulnerability_risk(&self, vulnerability: &OwaspVulnerability) -> f64 {
        let base_score = self.get_severity_score(&vulnerability.severity);
        let confidence_factor = vulnerability.confidence_score;

        base_score * confidence_factor
    }

    fn get_severity_score(&self, severity: &SecuritySeverity) -> f64 {
        match severity {
            SecuritySeverity::Critical => self.severity_weights.critical,
            SecuritySeverity::High => self.severity_weights.high,
            SecuritySeverity::Medium => self.severity_weights.medium,
            SecuritySeverity::Low => self.severity_weights.low,
        }
    }

    /// Get risk level description based on score
    pub fn get_risk_level_description(score: f64) -> &'static str {
        match score {
            s if s >= 9.0 => "Critical Risk",
            s if s >= 7.0 => "High Risk",
            s if s >= 4.0 => "Medium Risk",
            s if s >= 1.0 => "Low Risk",
            _ => "Minimal Risk",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::owasp::types::OwaspCategory;
    use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation};
    use std::path::PathBuf;

    #[test]
    fn test_risk_calculation() {
        let calculator = SeverityCalculator::new();

        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);
        let vuln = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "Test".to_string(),
            "Test vuln".to_string(),
            location,
        )
        .with_severity(SecuritySeverity::Critical)
        .with_confidence(1.0);

        let risk = calculator.calculate_vulnerability_risk(&vuln);
        assert_eq!(risk, 10.0);

        let overall_risk = calculator.calculate_overall_risk(&[vuln]);
        assert_eq!(overall_risk, 10.0);
    }

    #[test]
    fn test_risk_level_descriptions() {
        assert_eq!(SeverityCalculator::get_risk_level_description(10.0), "Critical Risk");
        assert_eq!(SeverityCalculator::get_risk_level_description(8.0), "High Risk");
        assert_eq!(SeverityCalculator::get_risk_level_description(5.0), "Medium Risk");
        assert_eq!(SeverityCalculator::get_risk_level_description(2.0), "Low Risk");
        assert_eq!(SeverityCalculator::get_risk_level_description(0.5), "Minimal Risk");
    }
}