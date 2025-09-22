use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::analysis::detectors::dependency::types::*;
use crate::analysis::detectors::dependency::config::VulnerabilityConfig;
use super::{VulnerabilityScanner, VulnerabilityScanOutput, ScanPriority};

#[derive(Debug, Clone)]
pub struct AdvisoryScanner {
    advisory_sources: HashMap<String, AdvisorySource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryScanResult {
    pub scanned_packages: usize,
    pub advisories_found: Vec<AdvisoryMatch>,
    pub scan_duration_ms: u64,
    pub sources_checked: Vec<String>,
    pub failed_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryMatch {
    pub advisory_id: String,
    pub source: String,
    pub package_name: String,
    pub package_version: String,
    pub severity: VulnerabilitySeverity,
    pub title: String,
    pub description: String,
    pub affected_versions: Vec<String>,
    pub patched_versions: Vec<String>,
    pub cwe_ids: Vec<String>,
    pub references: Vec<String>,
    pub published_date: String,
    pub updated_date: String,
}

#[derive(Debug, Clone)]
struct AdvisorySource {
    name: String,
    base_url: String,
    advisories: HashMap<String, Vec<Advisory>>,
    is_available: bool,
}

#[derive(Debug, Clone)]
struct Advisory {
    id: String,
    title: String,
    description: String,
    severity: VulnerabilitySeverity,
    affected_versions: Vec<String>,
    patched_versions: Vec<String>,
    cwe_ids: Vec<String>,
    references: Vec<String>,
    published_date: String,
    updated_date: String,
}

impl AdvisoryScanner {
    pub fn new() -> Self {
        let mut advisory_sources = HashMap::new();

        // GitHub Security Advisories
        advisory_sources.insert("github".to_string(), AdvisorySource {
            name: "GitHub Security Advisories".to_string(),
            base_url: "https://api.github.com/advisories".to_string(),
            advisories: Self::load_github_advisories(),
            is_available: true,
        });

        // RustSec Advisory Database
        advisory_sources.insert("rustsec".to_string(), AdvisorySource {
            name: "RustSec Advisory Database".to_string(),
            base_url: "https://rustsec.org/advisories".to_string(),
            advisories: Self::load_rustsec_advisories(),
            is_available: true,
        });

        // NPM Security Advisories
        advisory_sources.insert("npm".to_string(), AdvisorySource {
            name: "NPM Security Advisories".to_string(),
            base_url: "https://www.npmjs.com/advisories".to_string(),
            advisories: Self::load_npm_advisories(),
            is_available: true,
        });

        Self { advisory_sources }
    }

    fn load_github_advisories() -> HashMap<String, Vec<Advisory>> {
        let mut advisories = HashMap::new();

        advisories.insert("axios".to_string(), vec![
            Advisory {
                id: "GHSA-42xw-2xvc-qx8m".to_string(),
                title: "Axios SSRF Vulnerability".to_string(),
                description: "Axios contains an SSRF vulnerability in version 0.21.0".to_string(),
                severity: VulnerabilitySeverity::Medium,
                affected_versions: vec![">=0.8.1 <0.21.1".to_string()],
                patched_versions: vec!["0.21.1".to_string()],
                cwe_ids: vec!["CWE-918".to_string()],
                references: vec!["https://github.com/axios/axios/security/advisories/GHSA-42xw-2xvc-qx8m".to_string()],
                published_date: "2021-01-06".to_string(),
                updated_date: "2021-01-06".to_string(),
            }
        ]);

        advisories
    }

    fn load_rustsec_advisories() -> HashMap<String, Vec<Advisory>> {
        let mut advisories = HashMap::new();

        advisories.insert("tokio".to_string(), vec![
            Advisory {
                id: "RUSTSEC-2023-0001".to_string(),
                title: "tokio configuration corruption".to_string(),
                description: "Windows named pipe server configuration issue".to_string(),
                severity: VulnerabilitySeverity::Medium,
                affected_versions: vec![">=1.7.0 <1.18.4".to_string()],
                patched_versions: vec!["1.18.4".to_string()],
                cwe_ids: vec!["CWE-670".to_string()],
                references: vec!["https://rustsec.org/advisories/RUSTSEC-2023-0001.html".to_string()],
                published_date: "2023-01-04".to_string(),
                updated_date: "2023-01-04".to_string(),
            }
        ]);

        advisories
    }

    fn load_npm_advisories() -> HashMap<String, Vec<Advisory>> {
        let mut advisories = HashMap::new();

        advisories.insert("minimist".to_string(), vec![
            Advisory {
                id: "1179".to_string(),
                title: "minimist Prototype Pollution".to_string(),
                description: "minimist vulnerable to prototype pollution".to_string(),
                severity: VulnerabilitySeverity::Low,
                affected_versions: vec!["<0.2.1".to_string()],
                patched_versions: vec!["0.2.1".to_string()],
                cwe_ids: vec!["CWE-1321".to_string()],
                references: vec!["https://www.npmjs.com/advisories/1179".to_string()],
                published_date: "2020-03-11".to_string(),
                updated_date: "2020-03-11".to_string(),
            }
        ]);

        advisories
    }

    fn query_advisory_source(
        &self,
        source_name: &str,
        dependencies: &[DependencyInfo],
    ) -> Result<Vec<AdvisoryMatch>, String> {
        let mut matches = Vec::new();

        if let Some(source) = self.advisory_sources.get(source_name) {
            if !source.is_available {
                return Err(format!("Advisory source {} is not available", source_name));
            }

            for dependency in dependencies {
                if let Some(advisories) = source.advisories.get(&dependency.name) {
                    for advisory in advisories {
                        if let Some(version) = &dependency.version {
                            if self.is_version_affected(version, &advisory.affected_versions) {
                                matches.push(AdvisoryMatch {
                                    advisory_id: advisory.id.clone(),
                                    source: source_name.to_string(),
                                    package_name: dependency.name.clone(),
                                    package_version: version.clone(),
                                    severity: advisory.severity,
                                    title: advisory.title.clone(),
                                    description: advisory.description.clone(),
                                    affected_versions: advisory.affected_versions.clone(),
                                    patched_versions: advisory.patched_versions.clone(),
                                    cwe_ids: advisory.cwe_ids.clone(),
                                    references: advisory.references.clone(),
                                    published_date: advisory.published_date.clone(),
                                    updated_date: advisory.updated_date.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(matches)
    }

    fn is_version_affected(&self, version: &str, affected_ranges: &[String]) -> bool {
        // Simplified version range checking
        for range in affected_ranges {
            if range.contains("<=") || range.contains(">=") || range.contains("<") || range.contains(">") {
                // Simulate semantic version range checking
                if self.version_satisfies_range(version, range) {
                    return true;
                }
            } else if range == version || range.contains('*') {
                return true;
            }
        }
        false
    }

    fn version_satisfies_range(&self, version: &str, range: &str) -> bool {
        // Very basic version range satisfaction check
        // In a real implementation, this would use a proper semver library
        if range.starts_with(">=") {
            let min_version = range.trim_start_matches(">=").trim();
            return version >= min_version;
        }
        if range.starts_with("<=") {
            let max_version = range.trim_start_matches("<=").trim();
            return version <= max_version;
        }
        if range.starts_with('<') {
            let max_version = range.trim_start_matches('<').trim();
            return version < max_version;
        }
        if range.starts_with('>') {
            let min_version = range.trim_start_matches('>').trim();
            return version > min_version;
        }
        false
    }
}

impl VulnerabilityScanner for AdvisoryScanner {
    fn scan(
        &self,
        dependencies: &[DependencyInfo],
        config: &VulnerabilityConfig,
    ) -> Result<VulnerabilityScanOutput, DependencyError> {
        let start_time = std::time::Instant::now();
        let mut all_matches = Vec::new();
        let mut failed_sources = Vec::new();
        let mut checked_sources = Vec::new();

        for source_name in &config.advisory_sources {
            checked_sources.push(source_name.clone());
            match self.query_advisory_source(source_name, dependencies) {
                Ok(mut matches) => {
                    // Filter by minimum severity
                    matches.retain(|m| m.severity >= config.min_severity);
                    all_matches.extend(matches);
                }
                Err(error) => {
                    failed_sources.push(format!("{}: {}", source_name, error));
                }
            }
        }

        let scan_duration = start_time.elapsed();

        Ok(VulnerabilityScanOutput::Advisory(AdvisoryScanResult {
            scanned_packages: dependencies.len(),
            advisories_found: all_matches,
            scan_duration_ms: scan_duration.as_millis() as u64,
            sources_checked: checked_sources,
            failed_sources,
        }))
    }

    fn name(&self) -> &str {
        "Advisory Scanner"
    }

    fn priority(&self) -> ScanPriority {
        ScanPriority::High
    }
}

impl Default for AdvisoryScanner {
    fn default() -> Self {
        Self::new()
    }
}