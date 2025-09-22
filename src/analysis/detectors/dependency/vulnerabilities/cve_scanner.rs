use super::{ScanPriority, VulnerabilityScanOutput, VulnerabilityScanner};
use crate::analysis::detectors::dependency::config::VulnerabilityConfig;
use crate::analysis::detectors::dependency::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CveScanner {
    cve_database: CveDatabase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveScanResult {
    pub scanned_packages: usize,
    pub vulnerabilities_found: Vec<CveVulnerability>,
    pub scan_duration_ms: u64,
    pub database_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveVulnerability {
    pub cve_id: String,
    pub package_name: String,
    pub package_version: String,
    pub severity: VulnerabilitySeverity,
    pub cvss_score: Option<f32>,
    pub cvss_vector: Option<String>,
    pub description: String,
    pub published_date: String,
    pub last_modified: String,
    pub affected_versions: Vec<String>,
    pub fixed_versions: Vec<String>,
    pub references: Vec<String>,
    pub cwe_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct CveDatabase {
    entries: HashMap<String, Vec<CveEntry>>,
    version: String,
    last_updated: String,
}

#[derive(Debug, Clone)]
struct CveEntry {
    cve_id: String,
    severity: VulnerabilitySeverity,
    cvss_score: Option<f32>,
    cvss_vector: Option<String>,
    description: String,
    published_date: String,
    last_modified: String,
    affected_versions: Vec<String>,
    fixed_versions: Vec<String>,
    references: Vec<String>,
    cwe_ids: Vec<String>,
}

impl CveScanner {
    pub fn new() -> Self {
        Self {
            cve_database: CveDatabase::load_simulated(),
        }
    }

    pub fn with_custom_database(database: CveDatabase) -> Self {
        Self {
            cve_database: database,
        }
    }

    fn calculate_cvss_severity(score: f32) -> VulnerabilitySeverity {
        match score {
            s if s >= 9.0 => VulnerabilitySeverity::Critical,
            s if s >= 7.0 => VulnerabilitySeverity::High,
            s if s >= 4.0 => VulnerabilitySeverity::Medium,
            _ => VulnerabilitySeverity::Low,
        }
    }

    fn is_version_affected(&self, package_version: &str, affected_ranges: &[String]) -> bool {
        // Simulate version range checking
        for range in affected_ranges {
            if range.contains("*") || range == package_version {
                return true;
            }
            // Simple semantic version comparison simulation
            if let Some(version_num) = package_version.split('.').next() {
                if range.starts_with(&format!("{}.x", version_num)) {
                    return true;
                }
            }
        }
        false
    }

    fn query_cve_database(&self, package_name: &str, version: &str) -> Vec<CveVulnerability> {
        let mut vulnerabilities = Vec::new();

        if let Some(entries) = self.cve_database.entries.get(package_name) {
            for entry in entries {
                if self.is_version_affected(version, &entry.affected_versions) {
                    let severity = if let Some(score) = entry.cvss_score {
                        Self::calculate_cvss_severity(score)
                    } else {
                        entry.severity
                    };

                    vulnerabilities.push(CveVulnerability {
                        cve_id: entry.cve_id.clone(),
                        package_name: package_name.to_string(),
                        package_version: version.to_string(),
                        severity,
                        cvss_score: entry.cvss_score,
                        cvss_vector: entry.cvss_vector.clone(),
                        description: entry.description.clone(),
                        published_date: entry.published_date.clone(),
                        last_modified: entry.last_modified.clone(),
                        affected_versions: entry.affected_versions.clone(),
                        fixed_versions: entry.fixed_versions.clone(),
                        references: entry.references.clone(),
                        cwe_ids: entry.cwe_ids.clone(),
                    });
                }
            }
        }

        vulnerabilities
    }
}

impl VulnerabilityScanner for CveScanner {
    fn scan(
        &self,
        dependencies: &[DependencyInfo],
        config: &VulnerabilityConfig,
    ) -> Result<VulnerabilityScanOutput, DependencyError> {
        let start_time = std::time::Instant::now();
        let mut all_vulnerabilities = Vec::new();

        for dependency in dependencies {
            if let Some(version) = &dependency.version {
                let vulnerabilities = self.query_cve_database(&dependency.name, version);

                // Filter by minimum severity
                for vuln in vulnerabilities {
                    if vuln.severity >= config.min_severity {
                        all_vulnerabilities.push(vuln);
                    }
                }
            }
        }

        let scan_duration = start_time.elapsed();

        Ok(VulnerabilityScanOutput::Cve(CveScanResult {
            scanned_packages: dependencies.len(),
            vulnerabilities_found: all_vulnerabilities,
            scan_duration_ms: scan_duration.as_millis() as u64,
            database_version: self.cve_database.version.clone(),
        }))
    }

    fn name(&self) -> &str {
        "CVE Scanner"
    }

    fn priority(&self) -> ScanPriority {
        ScanPriority::High
    }
}

impl CveDatabase {
    fn load_simulated() -> Self {
        let mut entries = HashMap::new();

        // Simulate some common vulnerable packages
        entries.insert("openssl".to_string(), vec![
            CveEntry {
                cve_id: "CVE-2023-0464".to_string(),
                severity: VulnerabilitySeverity::High,
                cvss_score: Some(7.5),
                cvss_vector: Some("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H".to_string()),
                description: "A security issue was discovered in OpenSSL where invalid certificate policies in leaf certificates are silently ignored by OpenSSL.".to_string(),
                published_date: "2023-03-22".to_string(),
                last_modified: "2023-03-22".to_string(),
                affected_versions: vec!["3.0.x".to_string(), "1.1.1*".to_string()],
                fixed_versions: vec!["3.0.9".to_string(), "1.1.1u".to_string()],
                references: vec![
                    "https://www.openssl.org/news/secadv/20230322.txt".to_string(),
                    "https://nvd.nist.gov/vuln/detail/CVE-2023-0464".to_string(),
                ],
                cwe_ids: vec!["CWE-295".to_string()],
            }
        ]);

        entries.insert("lodash".to_string(), vec![
            CveEntry {
                cve_id: "CVE-2021-23337".to_string(),
                severity: VulnerabilitySeverity::High,
                cvss_score: Some(7.2),
                cvss_vector: Some("CVSS:3.1/AV:N/AC:L/PR:H/UI:N/S:U/C:H/I:H/A:H".to_string()),
                description: "Lodash versions prior to 4.17.21 are vulnerable to Command Injection via template.".to_string(),
                published_date: "2021-02-15".to_string(),
                last_modified: "2021-02-23".to_string(),
                affected_versions: vec!["<4.17.21".to_string()],
                fixed_versions: vec!["4.17.21".to_string()],
                references: vec![
                    "https://github.com/lodash/lodash/commit/3469357cff396a26c363f8c1b5a91dde28ba4b1c".to_string(),
                    "https://nvd.nist.gov/vuln/detail/CVE-2021-23337".to_string(),
                ],
                cwe_ids: vec!["CWE-94".to_string()],
            }
        ]);

        entries.insert("serde".to_string(), vec![
            CveEntry {
                cve_id: "CVE-2022-31394".to_string(),
                severity: VulnerabilitySeverity::Medium,
                cvss_score: Some(5.3),
                cvss_vector: Some("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:L/I:N/A:N".to_string()),
                description: "Serde is a framework for serializing and deserializing Rust data structures. Prior to version 1.0.144, serde_json contains a bug affecting JSON deserialization.".to_string(),
                published_date: "2022-07-04".to_string(),
                last_modified: "2022-07-12".to_string(),
                affected_versions: vec!["<1.0.144".to_string()],
                fixed_versions: vec!["1.0.144".to_string()],
                references: vec![
                    "https://github.com/serde-rs/serde/security/advisories".to_string(),
                    "https://nvd.nist.gov/vuln/detail/CVE-2022-31394".to_string(),
                ],
                cwe_ids: vec!["CWE-502".to_string()],
            }
        ]);

        Self {
            entries,
            version: "2023.1.0".to_string(),
            last_updated: "2023-09-22".to_string(),
        }
    }
}

impl Default for CveScanner {
    fn default() -> Self {
        Self::new()
    }
}
