//! OWASP scanning engine modules

pub mod dependency_scanner;
pub mod flow_scanner;
pub mod pattern_scanner;
pub mod static_scanner;

// Re-exports
pub use dependency_scanner::DependencyScanner;
pub use flow_scanner::DataFlowScanner;
pub use pattern_scanner::PatternScanner;
pub use static_scanner::StaticAnalysisScanner;

use crate::analysis::detectors::security::owasp::types::OwaspVulnerability;
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use serde_json::Value;
use std::time::Instant;

/// Unified scan result structure
#[derive(Debug, Clone)]
pub struct UnifiedScanResult {
    pub vulnerabilities: Vec<OwaspVulnerability>,
    /// Arbitrary metadata from the scanner (JSON object)
    pub scanner_metadata: Value,
    pub scan_duration_ms: u64,
}

/// Common scanner trait
#[async_trait::async_trait]
pub trait Scanner: Send + Sync {
    async fn scan(&self, file: &ParsedFile) -> Result<UnifiedScanResult, AnalysisError>;
    fn name(&self) -> &'static str;
    fn supported_languages(&self) -> Vec<SourceLanguage>;
    /// The OWASP categories this scanner can detect
    fn detectable_categories(
        &self,
    ) -> Vec<crate::analysis::detectors::security::owasp::types::OwaspCategory>;
}

/// Scanner orchestrator for coordinating multiple scanners
pub struct ScannerOrchestrator {
    scanners: Vec<Box<dyn Scanner>>,
}

impl ScannerOrchestrator {
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            scanners: vec![
                Box::new(StaticAnalysisScanner::new()?),
                Box::new(PatternScanner::new()?),
                Box::new(DataFlowScanner::new()?),
                Box::new(DependencyScanner::new()),
            ],
        })
    }

    pub async fn scan_all(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<UnifiedScanResult>, AnalysisError> {
        let mut results = Vec::new();
        for scanner in &self.scanners {
            if scanner.supported_languages().contains(&file.language) {
                let result = scanner.scan(file).await?;
                results.push(result);
            }
        }
        Ok(results)
    }

    pub fn scanner_count(&self) -> usize {
        self.scanners.len()
    }
}

impl Default for ScannerOrchestrator {
    fn default() -> Self {
        Self::new().expect("failed to create scanner orchestrator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator() {
        let orchestrator = ScannerOrchestrator::new().unwrap();
        assert_eq!(orchestrator.scanner_count(), 4);
    }
}
