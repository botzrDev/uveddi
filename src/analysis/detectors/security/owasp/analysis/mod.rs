//! OWASP analysis orchestration modules
//!
//! This module provides orchestration for OWASP security analysis including
//! scanning coordination, severity calculation, and remediation guidance.

pub mod scanner;
pub mod severity_calculator;
pub mod remediation;

// Re-exports
pub use scanner::OwaspComplianceScanner;
pub use severity_calculator::SeverityCalculator;
pub use remediation::RemediationEngine;

use super::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::AnalysisError;

/// OWASP analysis orchestrator
pub struct OwaspAnalysisOrchestrator {
    scanner: OwaspComplianceScanner,
    severity_calculator: SeverityCalculator,
    remediation_engine: RemediationEngine,
}

impl OwaspAnalysisOrchestrator {
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            scanner: OwaspComplianceScanner::new(),
            severity_calculator: SeverityCalculator::new(),
            remediation_engine: RemediationEngine::new(),
        })
    }

    pub async fn analyze_vulnerabilities(
        &self,
        vulnerabilities: Vec<OwaspVulnerability>,
    ) -> Result<OwaspAnalysisResult, AnalysisError> {
        let compliance_score = self.scanner.calculate_compliance_score(&vulnerabilities);
        let risk_score = self.severity_calculator.calculate_overall_risk(&vulnerabilities);
        let remediation_plan = self.remediation_engine.generate_plan(&vulnerabilities);

        Ok(OwaspAnalysisResult {
            vulnerabilities,
            compliance_score,
            risk_score,
            remediation_plan,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OwaspAnalysisResult {
    pub vulnerabilities: Vec<OwaspVulnerability>,
    pub compliance_score: f64,
    pub risk_score: f64,
    pub remediation_plan: Vec<String>,
}