//! Cryptographic Vulnerability Detection Module
//!
//! This module provides specialized detection for cryptographic vulnerabilities,
//! including timing attacks, side-channel vulnerabilities, entropy analysis,
//! and cryptographic misuse patterns.

pub mod weak_crypto;
pub mod timing_attacks;
pub mod side_channel;
pub mod entropy_analysis;
pub mod crypto_misuse;

pub use weak_crypto::WeakCryptoDetector;
pub use timing_attacks::TimingAttackDetector;
pub use side_channel::SideChannelDetector;
pub use entropy_analysis::EntropyAnalyzer;
pub use crypto_misuse::CryptoMisuseDetector;

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;

/// Main vulnerability analyzer that coordinates all vulnerability-specific analyzers
pub struct VulnerabilityAnalyzer {
    weak_crypto: WeakCryptoDetector,
    timing_attacks: TimingAttackDetector,
    side_channel: SideChannelDetector,
    entropy: EntropyAnalyzer,
    crypto_misuse: CryptoMisuseDetector,
}

impl VulnerabilityAnalyzer {
    /// Create a new vulnerability analyzer with all sub-analyzers
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            weak_crypto: WeakCryptoDetector::new(),
            timing_attacks: TimingAttackDetector::new(),
            side_channel: SideChannelDetector::new(),
            entropy: EntropyAnalyzer::new(),
            crypto_misuse: CryptoMisuseDetector::new(),
        })
    }

    /// Analyze content for all vulnerability-related issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Run specialized vulnerability detectors
        findings.extend(self.weak_crypto.analyze(content, language)?);
        findings.extend(self.timing_attacks.analyze(content, language)?);
        findings.extend(self.side_channel.analyze(content, language)?);
        findings.extend(self.entropy.analyze(content, language)?);
        findings.extend(self.crypto_misuse.analyze(content, language)?);

        Ok(findings)
    }

    /// Get vulnerability statistics
    pub fn get_vulnerability_stats(&self, findings: &[CryptoFinding]) -> VulnerabilityStats {
        let mut stats = VulnerabilityStats::default();

        for finding in findings {
            stats.total_count += 1;

            match finding.severity {
                SecuritySeverity::Critical => stats.critical_count += 1,
                SecuritySeverity::High => stats.high_count += 1,
                SecuritySeverity::Medium => stats.medium_count += 1,
                SecuritySeverity::Low => stats.low_count += 1,
            }

            match finding.finding_type {
                CryptoFindingType::TimingVulnerability => stats.timing_attacks += 1,
                CryptoFindingType::SideChannelLeak => stats.side_channel += 1,
                CryptoFindingType::EntropyIssue => stats.entropy_issues += 1,
                CryptoFindingType::WeakAlgorithm => stats.weak_algorithms += 1,
                _ => stats.other_issues += 1,
            }
        }

        stats
    }
}

/// Statistics about detected vulnerabilities
#[derive(Debug, Default, Clone)]
pub struct VulnerabilityStats {
    pub total_count: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub timing_attacks: usize,
    pub side_channel: usize,
    pub entropy_issues: usize,
    pub weak_algorithms: usize,
    pub other_issues: usize,
}

impl VulnerabilityStats {
    /// Get the risk score based on vulnerability distribution
    pub fn calculate_risk_score(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let weighted_score = (self.critical_count * 10) +
                           (self.high_count * 7) +
                           (self.medium_count * 4) +
                           (self.low_count * 1);

        (weighted_score as f64 / self.total_count as f64).min(10.0)
    }

    /// Get security assessment based on findings
    pub fn get_security_assessment(&self) -> SecurityAssessment {
        let risk_score = self.calculate_risk_score();

        if risk_score >= 8.0 {
            SecurityAssessment::Critical
        } else if risk_score >= 6.0 {
            SecurityAssessment::High
        } else if risk_score >= 4.0 {
            SecurityAssessment::Medium
        } else if risk_score >= 2.0 {
            SecurityAssessment::Low
        } else {
            SecurityAssessment::Good
        }
    }
}

/// Overall security assessment levels
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityAssessment {
    Critical,
    High,
    Medium,
    Low,
    Good,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_analyzer_creation() {
        let analyzer = VulnerabilityAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_vulnerability_stats() {
        let findings = vec![
            CryptoFinding {
                finding_type: CryptoFindingType::TimingVulnerability,
                line_number: 1,
                column: 0,
                description: "Test".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                algorithm: None,
                recommendation: "Fix".to_string(),
                cwe_id: None,
            }
        ];

        let analyzer = VulnerabilityAnalyzer::new().unwrap();
        let stats = analyzer.get_vulnerability_stats(&findings);

        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.high_count, 1);
        assert_eq!(stats.timing_attacks, 1);
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut stats = VulnerabilityStats::default();
        stats.total_count = 2;
        stats.critical_count = 1;
        stats.medium_count = 1;

        let risk_score = stats.calculate_risk_score();
        assert!(risk_score > 5.0); // Should be weighted towards critical
    }

    #[test]
    fn test_security_assessment() {
        let mut stats = VulnerabilityStats::default();
        stats.total_count = 1;
        stats.critical_count = 1;

        assert_eq!(stats.get_security_assessment(), SecurityAssessment::Critical);
    }
}