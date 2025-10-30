//! Main Cryptographic Security Detector
//!
//! This module implements the main detector that coordinates all cryptographic
//! analysis components and provides a unified interface for crypto security analysis.

use crate::analysis::detectors::security::crypto::{
    algorithms::AlgorithmAnalyzer,
    implementations::ImplementationAnalyzer,
    vulnerabilities::VulnerabilityAnalyzer,
    language_support::LanguageAnalyzerCoordinator,
    config::CryptoConfig,
    types::{CryptoFinding, CryptoFindingType},
};
use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use async_trait::async_trait;

/// Main cryptographic security detector that coordinates all analysis components
pub struct CryptoDetector {
    config: CryptoConfig,
    algorithm_analyzer: AlgorithmAnalyzer,
    implementation_analyzer: ImplementationAnalyzer,
    vulnerability_analyzer: VulnerabilityAnalyzer,
    language_coordinator: LanguageAnalyzerCoordinator,
}

impl CryptoDetector {
    /// Create a new crypto detector with default configuration
    pub fn new() -> Result<Self, AnalysisError> {
        Self::with_config(CryptoConfig::default())
    }

    /// Create a new crypto detector with custom configuration
    pub fn with_config(config: CryptoConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            algorithm_analyzer: AlgorithmAnalyzer::new()?,
            implementation_analyzer: ImplementationAnalyzer::new()?,
            vulnerability_analyzer: VulnerabilityAnalyzer::new()?,
            language_coordinator: LanguageAnalyzerCoordinator::new(),
            config,
        })
    }

    /// Analyze a file for cryptographic security issues
    pub async fn analyze_file(&self, file: &ParsedFile) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        let content = std::fs::read_to_string(&**file.file_path)
            .map_err(|e| AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e))?;

        // Algorithm analysis
        if self.config.enable_algorithm_analysis {
            let algo_findings = self.algorithm_analyzer.analyze(&content, &file.language)?;
            findings.extend(algo_findings);
        }

        // Implementation security analysis
        if self.config.enable_implementation_security {
            let impl_findings = self.implementation_analyzer.analyze(&content, &file.language)?;
            findings.extend(impl_findings);
        }

        // Vulnerability detection
        if self.config.enable_vulnerability_detection {
            let vuln_findings = self.vulnerability_analyzer.analyze(&content, &file.language)?;
            findings.extend(vuln_findings);
        }

        // Language-specific analysis
        let lang_findings = self.language_coordinator.analyze(&content, &file.language)?;
        findings.extend(lang_findings);

        // Filter by confidence threshold
        let filtered_findings: Vec<CryptoFinding> = findings
            .into_iter()
            .filter(|finding| finding.confidence >= self.config.confidence_threshold)
            .collect();

        Ok(filtered_findings)
    }

    /// Convert crypto findings to OWASP vulnerabilities
    fn convert_to_owasp_vulnerabilities(
        &self,
        findings: Vec<CryptoFinding>,
        file_path: &std::path::Path,
    ) -> Vec<OwaspVulnerability> {
        findings
            .into_iter()
            .map(|finding| {
                let location = SecurityLocation::new(
                    file_path.to_path_buf(),
                    finding.line_number as i32,
                    finding.line_number as i32,
                );

                let security_issue_type = match finding.finding_type {
                    CryptoFindingType::WeakAlgorithm |
                    CryptoFindingType::DeprecatedFunction |
                    CryptoFindingType::WeakRandom |
                    CryptoFindingType::EntropyIssue => SecurityIssueType::CryptographicFailures,
                    CryptoFindingType::HardcodedKey |
                    CryptoFindingType::WeakKeyManagement => SecurityIssueType::CryptographicFailures,
                    CryptoFindingType::InsecureTls |
                    CryptoFindingType::InvalidCertValidation => SecurityIssueType::SecurityMisconfiguration,
                    CryptoFindingType::TimingVulnerability |
                    CryptoFindingType::SideChannelLeak => SecurityIssueType::CryptographicFailures,
                };

                OwaspVulnerability::new(
                    OwaspCategory::CryptographicFailures,
                    security_issue_type,
                    finding.algorithm.unwrap_or_else(|| "Cryptographic Issue".to_string()),
                    finding.description,
                    location,
                )
                .with_confidence(finding.confidence)
                .with_severity(finding.severity)
                .with_remediation(finding.recommendation)
                .with_cwe_id(finding.cwe_id)
            })
            .collect()
    }

    /// Get detector configuration
    pub fn get_config(&self) -> &CryptoConfig {
        &self.config
    }

    /// Update detector configuration
    pub fn update_config(&mut self, config: CryptoConfig) {
        self.config = config;
    }

    /// Get analysis statistics
    pub fn get_analysis_stats(&self, findings: &[CryptoFinding]) -> AnalysisStats {
        let mut stats = AnalysisStats::default();

        for finding in findings {
            stats.total_findings += 1;

            match finding.severity {
                SecuritySeverity::Critical => stats.critical_findings += 1,
                SecuritySeverity::High => stats.high_findings += 1,
                SecuritySeverity::Medium => stats.medium_findings += 1,
                SecuritySeverity::Low => stats.low_findings += 1,
            }

            match finding.finding_type {
                CryptoFindingType::WeakAlgorithm => stats.weak_algorithms += 1,
                CryptoFindingType::HardcodedKey => stats.hardcoded_keys += 1,
                CryptoFindingType::WeakRandom => stats.weak_random += 1,
                CryptoFindingType::InsecureTls => stats.tls_issues += 1,
                CryptoFindingType::TimingVulnerability => stats.timing_vulnerabilities += 1,
                _ => stats.other_issues += 1,
            }

            // Track confidence distribution
            if finding.confidence >= 0.9 {
                stats.high_confidence += 1;
            } else if finding.confidence >= 0.7 {
                stats.medium_confidence += 1;
            } else {
                stats.low_confidence += 1;
            }
        }

        stats.average_confidence = if stats.total_findings > 0 {
            findings.iter().map(|f| f.confidence).sum::<f64>() / stats.total_findings as f64
        } else {
            0.0
        };

        stats
    }

    /// Check if the detector is properly configured
    pub fn validate_config(&self) -> Result<(), AnalysisError> {
        if self.config.confidence_threshold < 0.0 || self.config.confidence_threshold > 1.0 {
            return Err(AnalysisError::configuration_error(
                "Confidence threshold must be between 0.0 and 1.0".to_string()
            ));
        }

        if !self.config.enable_algorithm_analysis &&
           !self.config.enable_implementation_security &&
           !self.config.enable_vulnerability_detection {
            return Err(AnalysisError::configuration_error(
                "At least one analysis type must be enabled".to_string()
            ));
        }

        Ok(())
    }
}

/// Statistics about the analysis results
#[derive(Debug, Default, Clone)]
pub struct AnalysisStats {
    pub total_findings: usize,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub medium_findings: usize,
    pub low_findings: usize,
    pub weak_algorithms: usize,
    pub hardcoded_keys: usize,
    pub weak_random: usize,
    pub tls_issues: usize,
    pub timing_vulnerabilities: usize,
    pub other_issues: usize,
    pub high_confidence: usize,
    pub medium_confidence: usize,
    pub low_confidence: usize,
    pub average_confidence: f64,
}

impl AnalysisStats {
    /// Calculate overall risk score
    pub fn calculate_risk_score(&self) -> f64 {
        if self.total_findings == 0 {
            return 0.0;
        }

        let weighted_score = (self.critical_findings * 10) +
                           (self.high_findings * 7) +
                           (self.medium_findings * 4) +
                           (self.low_findings * 1);

        let base_score = weighted_score as f64 / self.total_findings as f64;

        // Adjust by confidence
        let confidence_factor = self.average_confidence;

        (base_score * confidence_factor).min(10.0)
    }

    /// Get security posture assessment
    pub fn get_security_posture(&self) -> SecurityPosture {
        let risk_score = self.calculate_risk_score();

        if risk_score >= 8.0 {
            SecurityPosture::Poor
        } else if risk_score >= 6.0 {
            SecurityPosture::Weak
        } else if risk_score >= 4.0 {
            SecurityPosture::Fair
        } else if risk_score >= 2.0 {
            SecurityPosture::Good
        } else {
            SecurityPosture::Excellent
        }
    }
}

/// Overall security posture assessment
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityPosture {
    Excellent,
    Good,
    Fair,
    Weak,
    Poor,
}

#[async_trait]
impl OwaspCategoryDetector for CryptoDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        // Validate configuration first
        self.validate_config()?;

        // Analyze the file
        let findings = self.analyze_file(file).await?;

        // Convert to OWASP vulnerabilities
        let vulnerabilities = self.convert_to_owasp_vulnerabilities(findings, &file.file_path);

        Ok(vulnerabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_crypto_detector_creation() {
        let detector = CryptoDetector::new();
        assert!(detector.is_ok());
    }

    #[tokio::test]
    async fn test_crypto_detector_with_config() {
        let config = CryptoConfig::minimal();
        let detector = CryptoDetector::with_config(config);
        assert!(detector.is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = CryptoConfig::default();
        config.confidence_threshold = 1.5; // Invalid

        let detector = CryptoDetector::with_config(config).unwrap();
        assert!(detector.validate_config().is_err());
    }

    #[test]
    fn test_analysis_stats() {
        let findings = vec![
            CryptoFinding::new(
                CryptoFindingType::WeakAlgorithm,
                1,
                "Test finding".to_string(),
                SecuritySeverity::High,
                0.9,
            ),
        ];

        let detector = CryptoDetector::new().unwrap();
        let stats = detector.get_analysis_stats(&findings);

        assert_eq!(stats.total_findings, 1);
        assert_eq!(stats.high_findings, 1);
        assert_eq!(stats.weak_algorithms, 1);
        assert_eq!(stats.high_confidence, 1);
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut stats = AnalysisStats::default();
        stats.total_findings = 2;
        stats.critical_findings = 1;
        stats.medium_findings = 1;
        stats.average_confidence = 0.8;

        let risk_score = stats.calculate_risk_score();
        assert!(risk_score > 5.0);
    }

    #[test]
    fn test_security_posture_assessment() {
        let mut stats = AnalysisStats::default();
        stats.total_findings = 1;
        stats.critical_findings = 1;
        stats.average_confidence = 1.0;

        assert_eq!(stats.get_security_posture(), SecurityPosture::Poor);
    }
}