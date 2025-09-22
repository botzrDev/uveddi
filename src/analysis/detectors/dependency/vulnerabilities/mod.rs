pub mod cve_scanner;
pub mod advisory_scanner;
pub mod malware_scanner;
pub mod supply_chain;
pub mod integrity_checker;

pub use cve_scanner::{CveScanner, CveScanResult};
pub use advisory_scanner::{AdvisoryScanner, AdvisoryScanResult};
pub use malware_scanner::{MalwareScanner, MalwareScanResult};
pub use supply_chain::{SupplyChainAnalyzer, SupplyChainAnalysis};
pub use integrity_checker::{IntegrityChecker, IntegrityCheckResult};

use crate::analysis::detectors::dependency::types::*;
use crate::analysis::detectors::dependency::config::*;

pub trait VulnerabilityScanner: Send + Sync {
    fn scan(
        &self,
        dependencies: &[DependencyInfo],
        config: &VulnerabilityConfig,
    ) -> Result<VulnerabilityScanOutput, DependencyError>;

    fn name(&self) -> &str;
    fn priority(&self) -> ScanPriority;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum VulnerabilityScanOutput {
    Cve(CveScanResult),
    Advisory(AdvisoryScanResult),
    Malware(MalwareScanResult),
    SupplyChain(SupplyChainAnalysis),
    Integrity(IntegrityCheckResult),
}

pub struct VulnerabilityScannerRegistry {
    scanners: Vec<Box<dyn VulnerabilityScanner>>,
}

impl VulnerabilityScannerRegistry {
    pub fn new() -> Self {
        Self {
            scanners: vec![
                Box::new(CveScanner::new()),
                Box::new(AdvisoryScanner::new()),
                Box::new(MalwareScanner::new()),
                Box::new(SupplyChainAnalyzer::new()),
                Box::new(IntegrityChecker::new()),
            ],
        }
    }

    pub fn run_all(
        &self,
        dependencies: &[DependencyInfo],
        config: &VulnerabilityConfig,
    ) -> Vec<Result<VulnerabilityScanOutput, DependencyError>> {
        let mut results = Vec::new();
        for scanner in &self.scanners {
            results.push(scanner.scan(dependencies, config));
        }
        results
    }
}

impl Default for VulnerabilityScannerRegistry {
    fn default() -> Self {
        Self::new()
    }
}