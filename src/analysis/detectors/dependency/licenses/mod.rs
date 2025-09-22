pub mod license_analyzer;
pub mod compliance_checker;
pub mod conflict_detector;
pub mod policy_enforcer;

pub use license_analyzer::{LicenseAnalyzer, LicenseAnalysisResult};
pub use compliance_checker::{ComplianceChecker, ComplianceResult};
pub use conflict_detector::{ConflictDetector, ConflictAnalysis};
pub use policy_enforcer::{PolicyEnforcer, PolicyResult};

use crate::analysis::detectors::dependency::types::*;
use crate::analysis::detectors::dependency::config::*;

pub trait LicenseChecker: Send + Sync {
    fn check(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Result<LicenseCheckOutput, DependencyError>;

    fn name(&self) -> &str;
    fn supports_transitive(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum LicenseCheckOutput {
    Analysis(LicenseAnalysisResult),
    Compliance(ComplianceResult),
    Conflicts(ConflictAnalysis),
    Policy(PolicyResult),
}

pub struct LicenseCheckerRegistry {
    checkers: Vec<Box<dyn LicenseChecker>>,
}

impl LicenseCheckerRegistry {
    pub fn new() -> Self {
        Self {
            checkers: vec![
                Box::new(LicenseAnalyzer::new()),
                Box::new(ComplianceChecker::new()),
                Box::new(ConflictDetector::new()),
                Box::new(PolicyEnforcer::new()),
            ],
        }
    }

    pub fn run_all(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Vec<Result<LicenseCheckOutput, DependencyError>> {
        let mut results = Vec::new();
        for checker in &self.checkers {
            results.push(checker.check(dependencies, config));
        }
        results
    }
}

impl Default for LicenseCheckerRegistry {
    fn default() -> Self {
        Self::new()
    }
}