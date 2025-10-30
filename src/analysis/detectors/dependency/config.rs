use super::types::{RiskLevel, UpdateUrgency, VulnerabilitySeverity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyDetectorConfig {
    pub vulnerability_scanning: VulnerabilityConfig,
    pub license_checking: LicenseConfig,
    pub outdated_checking: OutdatedConfig,
    pub supply_chain: SupplyChainConfig,
    pub analysis_depth: usize,
    pub parallel_workers: usize,
    pub cache_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityConfig {
    pub enabled: bool,
    pub min_severity: VulnerabilitySeverity,
    pub check_transitive: bool,
    pub advisory_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub enabled: bool,
    pub allowed_licenses: Vec<String>,
    pub denied_licenses: Vec<String>,
    pub check_transitive: bool,
    pub require_attribution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedConfig {
    pub enabled: bool,
    pub min_urgency: UpdateUrgency,
    pub check_major_only: bool,
    pub ignore_prerelease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainConfig {
    pub enabled: bool,
    pub min_risk_level: RiskLevel,
    pub check_maintainers: bool,
    pub check_typosquatting: bool,
    pub trusted_publishers: Vec<String>,
}

impl Default for DependencyDetectorConfig {
    fn default() -> Self {
        Self {
            vulnerability_scanning: VulnerabilityConfig::default(),
            license_checking: LicenseConfig::default(),
            outdated_checking: OutdatedConfig::default(),
            supply_chain: SupplyChainConfig::default(),
            analysis_depth: 5,
            parallel_workers: 4,
            cache_enabled: true,
        }
    }
}

impl Default for VulnerabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: VulnerabilitySeverity::Low,
            check_transitive: true,
            advisory_sources: vec![
                "github".to_string(),
                "rustsec".to_string(),
                "npm".to_string(),
            ],
        }
    }
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_licenses: vec![],
            denied_licenses: vec!["GPL-3.0".to_string()],
            check_transitive: true,
            require_attribution: true,
        }
    }
}

impl Default for OutdatedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_urgency: UpdateUrgency::Low,
            check_major_only: false,
            ignore_prerelease: true,
        }
    }
}

impl Default for SupplyChainConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_risk_level: RiskLevel::Medium,
            check_maintainers: true,
            check_typosquatting: true,
            trusted_publishers: vec![],
        }
    }
}
