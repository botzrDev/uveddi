use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::analysis::detectors::dependency::types::*;
use crate::analysis::detectors::dependency::config::VulnerabilityConfig;
use super::{VulnerabilityScanner, VulnerabilityScanOutput, ScanPriority};

#[derive(Debug, Clone)]
pub struct SupplyChainAnalyzer {
    typosquat_detector: TyposquatDetector,
    publisher_verifier: PublisherVerifier,
    maintenance_tracker: MaintenanceTracker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainAnalysis {
    pub scanned_packages: usize,
    pub risks_identified: Vec<SupplyChainRisk>,
    pub typosquat_warnings: Vec<TyposquatWarning>,
    pub publisher_issues: Vec<PublisherIssue>,
    pub maintenance_concerns: Vec<MaintenanceConcern>,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyposquatWarning {
    pub package_name: String,
    pub similar_packages: Vec<String>,
    pub confidence: f32,
    pub risk_level: RiskLevel,
    pub edit_distance: usize,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherIssue {
    pub package_name: String,
    pub publisher: String,
    pub issue_type: PublisherIssueType,
    pub severity: VulnerabilitySeverity,
    pub description: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublisherIssueType {
    UnverifiedPublisher,
    RecentOwnershipChange,
    SuspiciousActivity,
    NoSecurityPolicy,
    InactivePublisher,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConcern {
    pub package_name: String,
    pub concern_type: MaintenanceConcernType,
    pub severity: VulnerabilitySeverity,
    pub last_activity_days: u32,
    pub contributor_count: usize,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceConcernType {
    Unmaintained,
    FewContributors,
    NoRecentReleases,
    HighIssueCount,
    NoDocumentation,
}

#[derive(Debug, Clone)]
struct TyposquatDetector {
    popular_packages: HashSet<String>,
    edit_distance_threshold: usize,
}

#[derive(Debug, Clone)]
struct PublisherVerifier {
    trusted_publishers: HashMap<String, PublisherInfo>,
    suspicious_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
struct PublisherInfo {
    name: String,
    verified: bool,
    reputation_score: f32,
    has_security_policy: bool,
}

#[derive(Debug, Clone)]
struct MaintenanceTracker {
    package_metadata: HashMap<String, PackageMetadata>,
}

#[derive(Debug, Clone)]
struct PackageMetadata {
    last_update_days: u32,
    contributor_count: usize,
    open_issues: usize,
    has_tests: bool,
}

impl SupplyChainAnalyzer {
    pub fn new() -> Self {
        Self {
            typosquat_detector: TyposquatDetector::new(),
            publisher_verifier: PublisherVerifier::new(),
            maintenance_tracker: MaintenanceTracker::new(),
        }
    }

    fn analyze_typosquatting(&self, dependencies: &[DependencyInfo]) -> Vec<TyposquatWarning> {
        let mut warnings = Vec::new();

        for dependency in dependencies {
            let similar_packages = self.typosquat_detector.find_similar_packages(&dependency.name);

            if !similar_packages.is_empty() {
                let min_edit_distance = similar_packages
                    .iter()
                    .map(|pkg| self.calculate_edit_distance(&dependency.name, pkg))
                    .min()
                    .unwrap_or(999);

                let confidence = self.calculate_typosquat_confidence(&dependency.name, &similar_packages);
                let risk_level = self.assess_typosquat_risk(confidence, min_edit_distance);

                if confidence > 0.3 {
                    warnings.push(TyposquatWarning {
                        package_name: dependency.name.clone(),
                        similar_packages,
                        confidence,
                        risk_level,
                        edit_distance: min_edit_distance,
                        recommendations: vec!["Verify package name".to_string()],
                    });
                }
            }
        }

        warnings
    }

    fn verify_publishers(&self, dependencies: &[DependencyInfo]) -> Vec<PublisherIssue> {
        let mut issues = Vec::new();

        for dependency in dependencies {
            if let Some(issue) = self.check_suspicious_patterns(dependency) {
                issues.push(issue);
            }
        }

        issues
    }

    fn check_suspicious_patterns(&self, dependency: &DependencyInfo) -> Option<PublisherIssue> {
        for pattern in &self.publisher_verifier.suspicious_patterns {
            if dependency.name.contains(pattern) {
                return Some(PublisherIssue {
                    package_name: dependency.name.clone(),
                    publisher: "unknown".to_string(),
                    issue_type: PublisherIssueType::SuspiciousActivity,
                    severity: VulnerabilitySeverity::High,
                    description: format!("Package name contains suspicious pattern: {}", pattern),
                    evidence: vec![format!("Pattern '{}' found", pattern)],
                });
            }
        }
        None
    }

    fn check_maintenance_status(&self, dependencies: &[DependencyInfo]) -> Vec<MaintenanceConcern> {
        let mut concerns = Vec::new();

        for dependency in dependencies {
            if let Some(metadata) = self.maintenance_tracker.package_metadata.get(&dependency.name) {
                if metadata.last_update_days > 365 {
                    concerns.push(MaintenanceConcern {
                        package_name: dependency.name.clone(),
                        concern_type: MaintenanceConcernType::Unmaintained,
                        severity: VulnerabilitySeverity::Medium,
                        last_activity_days: metadata.last_update_days,
                        contributor_count: metadata.contributor_count,
                        details: format!("Package has not been updated for {} days", metadata.last_update_days),
                    });
                }
            }
        }

        concerns
    }

    fn calculate_edit_distance(&self, s1: &str, s2: &str) -> usize {
        // Simplified edit distance - just count character differences
        s1.chars().zip(s2.chars()).filter(|(a, b)| a != b).count() +
        s1.len().abs_diff(s2.len())
    }

    fn calculate_typosquat_confidence(&self, package: &str, similar: &[String]) -> f32 {
        let mut max_confidence = 0.0;

        for similar_pkg in similar {
            let distance = self.calculate_edit_distance(package, similar_pkg);
            let length_ratio = (package.len() as f32 / similar_pkg.len() as f32).min(1.0);
            let confidence = (1.0 - distance as f32 / package.len().max(similar_pkg.len()) as f32) * length_ratio;
            max_confidence = max_confidence.max(confidence);
        }

        max_confidence
    }

    fn assess_typosquat_risk(&self, confidence: f32, edit_distance: usize) -> RiskLevel {
        match (confidence, edit_distance) {
            (c, d) if c > 0.8 && d <= 2 => RiskLevel::Critical,
            (c, d) if c > 0.6 && d <= 3 => RiskLevel::High,
            (c, d) if c > 0.4 && d <= 4 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }
}

impl VulnerabilityScanner for SupplyChainAnalyzer {
    fn scan(
        &self,
        dependencies: &[DependencyInfo],
        _config: &VulnerabilityConfig,
    ) -> Result<VulnerabilityScanOutput, DependencyError> {
        let start_time = std::time::Instant::now();

        let typosquat_warnings = self.analyze_typosquatting(dependencies);
        let publisher_issues = self.verify_publishers(dependencies);
        let maintenance_concerns = self.check_maintenance_status(dependencies);

        // Convert to SupplyChainRisk format
        let mut risks = Vec::new();

        for warning in &typosquat_warnings {
            risks.push(SupplyChainRisk {
                package_name: warning.package_name.clone(),
                risk_level: warning.risk_level,
                risk_factors: vec![RiskFactor::Typosquatting {
                    similar_to: warning.similar_packages.first().unwrap_or(&"unknown".to_string()).clone(),
                }],
                recommendations: warning.recommendations.clone(),
            });
        }

        let scan_duration = start_time.elapsed();

        Ok(VulnerabilityScanOutput::SupplyChain(SupplyChainAnalysis {
            scanned_packages: dependencies.len(),
            risks_identified: risks,
            typosquat_warnings,
            publisher_issues,
            maintenance_concerns,
            scan_duration_ms: scan_duration.as_millis() as u64,
        }))
    }

    fn name(&self) -> &str {
        "Supply Chain Analyzer"
    }

    fn priority(&self) -> ScanPriority {
        ScanPriority::High
    }
}

impl TyposquatDetector {
    fn new() -> Self {
        let mut popular_packages = HashSet::new();
        popular_packages.insert("serde".to_string());
        popular_packages.insert("tokio".to_string());
        popular_packages.insert("lodash".to_string());
        popular_packages.insert("react".to_string());

        Self {
            popular_packages,
            edit_distance_threshold: 3,
        }
    }

    fn find_similar_packages(&self, package_name: &str) -> Vec<String> {
        self.popular_packages
            .iter()
            .filter(|&popular| {
                let distance = self.calculate_edit_distance(package_name, popular);
                distance <= self.edit_distance_threshold && distance > 0
            })
            .map(|s| s.to_string())
            .collect()
    }

    fn calculate_edit_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[a_len][b_len]
    }
}
