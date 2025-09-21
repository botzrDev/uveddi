//! OWASP compliance and risk reporting modules
//!
//! This module provides comprehensive reporting functionality for OWASP
//! security analysis including compliance reports and risk matrices.

pub mod compliance_report;
pub mod risk_matrix;

// Re-exports
pub use compliance_report::{ComplianceReporter, OwaspComplianceReport};
pub use risk_matrix::{RiskMatrixGenerator, SecurityRiskMatrix};

use super::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::AnalysisError;

/// OWASP reporting coordinator
pub struct OwaspReportingCoordinator {
    compliance_reporter: ComplianceReporter,
    risk_matrix_generator: RiskMatrixGenerator,
}

impl OwaspReportingCoordinator {
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            compliance_reporter: ComplianceReporter::new(),
            risk_matrix_generator: RiskMatrixGenerator::new(),
        })
    }

    /// Generate comprehensive OWASP analysis report
    pub fn generate_comprehensive_report(
        &self,
        vulnerabilities: &[OwaspVulnerability],
    ) -> Result<OwaspAnalysisReport, AnalysisError> {
        let compliance_report = self.compliance_reporter.generate_report(vulnerabilities)?;
        let risk_matrix = self.risk_matrix_generator.generate_matrix(vulnerabilities)?;

        Ok(OwaspAnalysisReport {
            compliance_report,
            risk_matrix,
            summary: self.generate_executive_summary(vulnerabilities),
        })
    }

    fn generate_executive_summary(&self, vulnerabilities: &[OwaspVulnerability]) -> ExecutiveSummary {
        let total_vulnerabilities = vulnerabilities.len();
        let critical_count = vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, crate::analysis::detectors::security::types::SecuritySeverity::Critical))
            .count();
        let high_count = vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, crate::analysis::detectors::security::types::SecuritySeverity::High))
            .count();

        let affected_categories = vulnerabilities
            .iter()
            .map(|v| v.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        ExecutiveSummary {
            total_vulnerabilities,
            critical_count,
            high_count,
            affected_owasp_categories: affected_categories,
            overall_risk_level: self.calculate_overall_risk_level(vulnerabilities),
        }
    }

    fn calculate_overall_risk_level(&self, vulnerabilities: &[OwaspVulnerability]) -> String {
        if vulnerabilities.is_empty() {
            return "Low".to_string();
        }

        let critical_count = vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, crate::analysis::detectors::security::types::SecuritySeverity::Critical))
            .count();
        let high_count = vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, crate::analysis::detectors::security::types::SecuritySeverity::High))
            .count();

        if critical_count > 0 {
            "Critical".to_string()
        } else if high_count > 5 {
            "High".to_string()
        } else if high_count > 0 || vulnerabilities.len() > 10 {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct OwaspAnalysisReport {
    pub compliance_report: OwaspComplianceReport,
    pub risk_matrix: SecurityRiskMatrix,
    pub summary: ExecutiveSummary,
}

#[derive(Debug, Clone)]
pub struct ExecutiveSummary {
    pub total_vulnerabilities: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub affected_owasp_categories: usize,
    pub overall_risk_level: String,
}