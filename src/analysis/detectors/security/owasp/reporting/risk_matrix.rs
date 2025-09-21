//! Risk assessment matrix for OWASP vulnerabilities
//!
//! This module generates risk matrices for prioritizing security remediation
//! efforts based on vulnerability severity and likelihood.

use super::super::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Generator for security risk assessment matrices
pub struct RiskMatrixGenerator;

impl RiskMatrixGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate comprehensive risk matrix for vulnerabilities
    pub fn generate_matrix(&self, vulnerabilities: &[OwaspVulnerability]) -> Result<SecurityRiskMatrix, AnalysisError> {
        let mut risk_matrix = SecurityRiskMatrix {
            critical_high: Vec::new(),
            critical_medium: Vec::new(),
            critical_low: Vec::new(),
            high_high: Vec::new(),
            high_medium: Vec::new(),
            high_low: Vec::new(),
            medium_high: Vec::new(),
            medium_medium: Vec::new(),
            medium_low: Vec::new(),
            low_high: Vec::new(),
            low_medium: Vec::new(),
            low_low: Vec::new(),
            priority_rankings: Vec::new(),
        };

        // Categorize vulnerabilities by impact and likelihood
        for vulnerability in vulnerabilities {
            let impact = self.calculate_impact(&vulnerability.severity);
            let likelihood = self.calculate_likelihood(vulnerability);
            let risk_item = RiskMatrixItem {
                vulnerability: vulnerability.clone(),
                impact_level: impact,
                likelihood_level: likelihood,
                risk_score: self.calculate_risk_score(impact, likelihood),
            };

            self.place_in_matrix(&mut risk_matrix, risk_item);
        }

        // Generate priority rankings
        risk_matrix.priority_rankings = self.generate_priority_rankings(vulnerabilities);

        Ok(risk_matrix)
    }

    fn calculate_impact(&self, severity: &SecuritySeverity) -> RiskLevel {
        match severity {
            SecuritySeverity::Critical => RiskLevel::High,
            SecuritySeverity::High => RiskLevel::High,
            SecuritySeverity::Medium => RiskLevel::Medium,
            SecuritySeverity::Low => RiskLevel::Low,
        }
    }

    fn calculate_likelihood(&self, vulnerability: &OwaspVulnerability) -> RiskLevel {
        // Base likelihood on confidence score and vulnerability category
        let base_likelihood = match vulnerability.category {
            OwaspCategory::Injection => 0.8, // High likelihood
            OwaspCategory::BrokenAccessControl => 0.7,
            OwaspCategory::CryptographicFailures => 0.6,
            OwaspCategory::SecurityMisconfiguration => 0.7,
            OwaspCategory::InsecureDesign => 0.5,
            _ => 0.4, // Medium likelihood for other categories
        };

        let adjusted_likelihood = base_likelihood * vulnerability.confidence_score;

        if adjusted_likelihood >= 0.7 {
            RiskLevel::High
        } else if adjusted_likelihood >= 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn calculate_risk_score(&self, impact: RiskLevel, likelihood: RiskLevel) -> f64 {
        let impact_score = match impact {
            RiskLevel::High => 3.0,
            RiskLevel::Medium => 2.0,
            RiskLevel::Low => 1.0,
        };

        let likelihood_score = match likelihood {
            RiskLevel::High => 3.0,
            RiskLevel::Medium => 2.0,
            RiskLevel::Low => 1.0,
        };

        impact_score * likelihood_score
    }

    fn place_in_matrix(&self, matrix: &mut SecurityRiskMatrix, item: RiskMatrixItem) {
        match (item.impact_level, item.likelihood_level) {
            (RiskLevel::High, RiskLevel::High) => matrix.critical_high.push(item),
            (RiskLevel::High, RiskLevel::Medium) => matrix.critical_medium.push(item),
            (RiskLevel::High, RiskLevel::Low) => matrix.critical_low.push(item),
            (RiskLevel::Medium, RiskLevel::High) => matrix.high_high.push(item),
            (RiskLevel::Medium, RiskLevel::Medium) => matrix.high_medium.push(item),
            (RiskLevel::Medium, RiskLevel::Low) => matrix.high_low.push(item),
            (RiskLevel::Low, RiskLevel::High) => matrix.medium_high.push(item),
            (RiskLevel::Low, RiskLevel::Medium) => matrix.medium_medium.push(item),
            (RiskLevel::Low, RiskLevel::Low) => matrix.low_low.push(item),
        }
    }

    fn generate_priority_rankings(&self, vulnerabilities: &[OwaspVulnerability]) -> Vec<PriorityItem> {
        let mut priority_items: Vec<PriorityItem> = vulnerabilities
            .iter()
            .map(|vuln| {
                let impact = self.calculate_impact(&vuln.severity);
                let likelihood = self.calculate_likelihood(vuln);
                let risk_score = self.calculate_risk_score(impact, likelihood);

                PriorityItem {
                    vulnerability: vuln.clone(),
                    priority_score: risk_score,
                    remediation_effort: self.estimate_remediation_effort(vuln),
                    business_impact: self.estimate_business_impact(vuln),
                }
            })
            .collect();

        // Sort by priority score (highest first)
        priority_items.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());

        priority_items
    }

    fn estimate_remediation_effort(&self, vulnerability: &OwaspVulnerability) -> RemediationEffort {
        match vulnerability.category {
            OwaspCategory::SecurityMisconfiguration => RemediationEffort::Low,
            OwaspCategory::CryptographicFailures => RemediationEffort::Medium,
            OwaspCategory::Injection => RemediationEffort::Medium,
            OwaspCategory::BrokenAccessControl => RemediationEffort::High,
            OwaspCategory::InsecureDesign => RemediationEffort::High,
            _ => RemediationEffort::Medium,
        }
    }

    fn estimate_business_impact(&self, vulnerability: &OwaspVulnerability) -> BusinessImpact {
        match vulnerability.category {
            OwaspCategory::Injection => BusinessImpact::Critical,
            OwaspCategory::BrokenAccessControl => BusinessImpact::Critical,
            OwaspCategory::CryptographicFailures => BusinessImpact::High,
            OwaspCategory::AuthenticationFailures => BusinessImpact::High,
            OwaspCategory::SecurityMisconfiguration => BusinessImpact::Medium,
            _ => BusinessImpact::Medium,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityRiskMatrix {
    // Impact: High, Likelihood: High/Medium/Low
    pub critical_high: Vec<RiskMatrixItem>,
    pub critical_medium: Vec<RiskMatrixItem>,
    pub critical_low: Vec<RiskMatrixItem>,

    // Impact: Medium, Likelihood: High/Medium/Low
    pub high_high: Vec<RiskMatrixItem>,
    pub high_medium: Vec<RiskMatrixItem>,
    pub high_low: Vec<RiskMatrixItem>,

    // Impact: Low, Likelihood: High/Medium/Low
    pub medium_high: Vec<RiskMatrixItem>,
    pub medium_medium: Vec<RiskMatrixItem>,
    pub low_low: Vec<RiskMatrixItem>,

    pub priority_rankings: Vec<PriorityItem>,
}

#[derive(Debug, Clone)]
pub struct RiskMatrixItem {
    pub vulnerability: OwaspVulnerability,
    pub impact_level: RiskLevel,
    pub likelihood_level: RiskLevel,
    pub risk_score: f64,
}

#[derive(Debug, Clone)]
pub struct PriorityItem {
    pub vulnerability: OwaspVulnerability,
    pub priority_score: f64,
    pub remediation_effort: RemediationEffort,
    pub business_impact: BusinessImpact,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy)]
pub enum RemediationEffort {
    Low,    // Configuration changes, simple fixes
    Medium, // Code changes, testing required
    High,   // Architectural changes, significant refactoring
}

#[derive(Debug, Clone, Copy)]
pub enum BusinessImpact {
    Critical, // Data breach, system compromise
    High,     // Service disruption, compliance issues
    Medium,   // Performance impact, user experience
    Low,      // Minor issues, informational
}

impl SecurityRiskMatrix {
    /// Get all critical priority items (high impact, high likelihood)
    pub fn get_critical_items(&self) -> &Vec<RiskMatrixItem> {
        &self.critical_high
    }

    /// Get top N priority vulnerabilities for immediate attention
    pub fn get_top_priority(&self, n: usize) -> Vec<&PriorityItem> {
        self.priority_rankings.iter().take(n).collect()
    }

    /// Calculate overall risk statistics
    pub fn get_risk_statistics(&self) -> RiskStatistics {
        let total_items = self.critical_high.len() + self.critical_medium.len() + self.critical_low.len()
            + self.high_high.len() + self.high_medium.len() + self.high_low.len()
            + self.medium_high.len() + self.medium_medium.len() + self.low_low.len();

        RiskStatistics {
            total_vulnerabilities: total_items,
            critical_risk_count: self.critical_high.len(),
            high_risk_count: self.critical_medium.len() + self.high_high.len(),
            medium_risk_count: self.critical_low.len() + self.high_medium.len() + self.medium_high.len(),
            low_risk_count: self.high_low.len() + self.medium_medium.len() + self.low_low.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskStatistics {
    pub total_vulnerabilities: usize,
    pub critical_risk_count: usize,
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation};
    use std::path::PathBuf;

    #[test]
    fn test_risk_matrix_generation() {
        let generator = RiskMatrixGenerator::new();

        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);
        let vuln = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "SQL Injection".to_string(),
            "Critical SQL injection vulnerability".to_string(),
            location,
        )
        .with_severity(SecuritySeverity::Critical)
        .with_confidence(0.9);

        let matrix = generator.generate_matrix(&[vuln]).unwrap();

        assert_eq!(matrix.critical_high.len(), 1);
        assert_eq!(matrix.priority_rankings.len(), 1);

        let stats = matrix.get_risk_statistics();
        assert_eq!(stats.total_vulnerabilities, 1);
        assert_eq!(stats.critical_risk_count, 1);
    }

    #[test]
    fn test_priority_ranking() {
        let generator = RiskMatrixGenerator::new();

        let location = SecurityLocation::new(PathBuf::from("test.rs"), 1, 1);
        let high_vuln = OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            "SQL Injection".to_string(),
            "High priority vulnerability".to_string(),
            location.clone(),
        )
        .with_severity(SecuritySeverity::Critical)
        .with_confidence(0.9);

        let low_vuln = OwaspVulnerability::new(
            OwaspCategory::LoggingFailures,
            SecurityIssueType::Other,
            "Missing logs".to_string(),
            "Low priority vulnerability".to_string(),
            location,
        )
        .with_severity(SecuritySeverity::Low)
        .with_confidence(0.3);

        let matrix = generator.generate_matrix(&[high_vuln, low_vuln]).unwrap();

        assert_eq!(matrix.priority_rankings.len(), 2);
        assert!(matrix.priority_rankings[0].priority_score > matrix.priority_rankings[1].priority_score);
    }
}