//! OWASP compliance reporting
//!
//! This module generates compliance reports showing adherence to OWASP
//! Top 10 security standards.

use super::super::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Generator for OWASP compliance reports
pub struct ComplianceReporter {
    category_weights: HashMap<OwaspCategory, f64>,
}

impl ComplianceReporter {
    pub fn new() -> Self {
        let mut category_weights = HashMap::new();

        // Equal weight for all OWASP Top 10 categories (10% each)
        let weight = 0.1;
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

    /// Generate comprehensive OWASP compliance report
    pub fn generate_report(&self, vulnerabilities: &[OwaspVulnerability]) -> Result<OwaspComplianceReport, AnalysisError> {
        let mut category_compliance = HashMap::new();
        let mut category_vulnerabilities: HashMap<OwaspCategory, Vec<&OwaspVulnerability>> = HashMap::new();

        // Group vulnerabilities by category
        for vulnerability in vulnerabilities {
            category_vulnerabilities
                .entry(vulnerability.category.clone())
                .or_default()
                .push(vulnerability);
        }

        // Calculate compliance for each category
        for (category, weight) in &self.category_weights {
            let compliance_score = if let Some(vulns) = category_vulnerabilities.get(category) {
                self.calculate_category_compliance(vulns)
            } else {
                1.0 // No vulnerabilities = full compliance
            };

            category_compliance.insert(category.clone(), CategoryCompliance {
                category: category.clone(),
                compliance_percentage: (compliance_score * 100.0).round(),
                vulnerability_count: category_vulnerabilities.get(category).map_or(0, |v| v.len()),
                severity_distribution: self.calculate_severity_distribution(
                    category_vulnerabilities.get(category).unwrap_or(&Vec::new())
                ),
                recommendations: self.get_category_recommendations(category),
            });
        }

        let overall_compliance = self.calculate_overall_compliance(&category_compliance);

        Ok(OwaspComplianceReport {
            overall_compliance_percentage: (overall_compliance * 100.0).round(),
            category_compliance,
            total_vulnerabilities: vulnerabilities.len(),
            compliance_grade: self.get_compliance_grade(overall_compliance),
            areas_for_improvement: self.identify_improvement_areas(&category_compliance),
        })
    }

    fn calculate_category_compliance(&self, vulnerabilities: &[&OwaspVulnerability]) -> f64 {
        if vulnerabilities.is_empty() {
            return 1.0;
        }

        // Calculate compliance based on severity and confidence of vulnerabilities
        let total_impact: f64 = vulnerabilities
            .iter()
            .map(|vuln| self.vulnerability_impact_score(vuln))
            .sum();

        let max_possible_impact = vulnerabilities.len() as f64;
        (max_possible_impact - total_impact) / max_possible_impact
    }

    fn vulnerability_impact_score(&self, vulnerability: &OwaspVulnerability) -> f64 {
        let severity_weight = match vulnerability.severity {
            SecuritySeverity::Critical => 1.0,
            SecuritySeverity::High => 0.8,
            SecuritySeverity::Medium => 0.5,
            SecuritySeverity::Low => 0.2,
        };

        severity_weight * vulnerability.confidence_score
    }

    fn calculate_severity_distribution(&self, vulnerabilities: &[&OwaspVulnerability]) -> SeverityDistribution {
        let mut distribution = SeverityDistribution::default();

        for vulnerability in vulnerabilities {
            match vulnerability.severity {
                SecuritySeverity::Critical => distribution.critical += 1,
                SecuritySeverity::High => distribution.high += 1,
                SecuritySeverity::Medium => distribution.medium += 1,
                SecuritySeverity::Low => distribution.low += 1,
            }
        }

        distribution
    }

    fn calculate_overall_compliance(&self, category_compliance: &HashMap<OwaspCategory, CategoryCompliance>) -> f64 {
        if category_compliance.is_empty() {
            return 1.0;
        }

        let weighted_sum: f64 = category_compliance
            .iter()
            .map(|(category, compliance)| {
                let weight = self.category_weights.get(category).unwrap_or(&0.1);
                weight * (compliance.compliance_percentage / 100.0)
            })
            .sum();

        weighted_sum
    }

    fn get_compliance_grade(&self, compliance_score: f64) -> String {
        match (compliance_score * 100.0) as u8 {
            90..=100 => "A".to_string(),
            80..=89 => "B".to_string(),
            70..=79 => "C".to_string(),
            60..=69 => "D".to_string(),
            _ => "F".to_string(),
        }
    }

    fn identify_improvement_areas(&self, category_compliance: &HashMap<OwaspCategory, CategoryCompliance>) -> Vec<String> {
        let mut areas = Vec::new();

        for (category, compliance) in category_compliance {
            if compliance.compliance_percentage < 80.0 {
                areas.push(format!(
                    "{}: {}% compliance - {} vulnerabilities found",
                    category.identifier(),
                    compliance.compliance_percentage,
                    compliance.vulnerability_count
                ));
            }
        }

        areas.sort_by(|a, b| {
            let a_percent = a.split(':').nth(1).and_then(|s| s.trim().strip_suffix("% compliance")).and_then(|s| s.parse::<f64>().ok()).unwrap_or(100.0);
            let b_percent = b.split(':').nth(1).and_then(|s| s.trim().strip_suffix("% compliance")).and_then(|s| s.parse::<f64>().ok()).unwrap_or(100.0);
            a_percent.partial_cmp(&b_percent).unwrap_or(std::cmp::Ordering::Equal)
        });

        areas
    }

    fn get_category_recommendations(&self, category: &OwaspCategory) -> Vec<String> {
        match category {
            OwaspCategory::BrokenAccessControl => vec![
                "Implement proper access control checks".to_string(),
                "Use principle of least privilege".to_string(),
                "Implement centralized authorization".to_string(),
            ],
            OwaspCategory::CryptographicFailures => vec![
                "Use strong encryption algorithms".to_string(),
                "Implement proper key management".to_string(),
                "Use secure random number generators".to_string(),
            ],
            OwaspCategory::Injection => vec![
                "Use parameterized queries".to_string(),
                "Implement input validation".to_string(),
                "Use safe APIs".to_string(),
            ],
            _ => vec!["Follow OWASP guidelines for this category".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct OwaspComplianceReport {
    pub overall_compliance_percentage: f64,
    pub category_compliance: HashMap<OwaspCategory, CategoryCompliance>,
    pub total_vulnerabilities: usize,
    pub compliance_grade: String,
    pub areas_for_improvement: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CategoryCompliance {
    pub category: OwaspCategory,
    pub compliance_percentage: f64,
    pub vulnerability_count: usize,
    pub severity_distribution: SeverityDistribution,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SeverityDistribution {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation};
    use std::path::PathBuf;

    #[test]
    fn test_compliance_report_generation() {
        let reporter = ComplianceReporter::new();

        // Test with no vulnerabilities
        let report = reporter.generate_report(&[]).unwrap();
        assert_eq!(report.overall_compliance_percentage, 100.0);
        assert_eq!(report.compliance_grade, "A");

        // Test with vulnerabilities
        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);
        let vuln = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "Test".to_string(),
            "Test vulnerability".to_string(),
            location,
        ).with_severity(SecuritySeverity::High);

        let report = reporter.generate_report(&[vuln]).unwrap();
        assert!(report.overall_compliance_percentage < 100.0);
        assert_eq!(report.total_vulnerabilities, 1);
        assert!(!report.areas_for_improvement.is_empty());
    }
}