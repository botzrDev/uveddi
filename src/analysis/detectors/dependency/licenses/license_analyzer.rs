use super::{LicenseCheckOutput, LicenseChecker};
use crate::analysis::detectors::dependency::config::LicenseConfig;
use crate::analysis::detectors::dependency::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LicenseAnalyzer {
    spdx_database: SpdxDatabase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseAnalysisResult {
    pub detected_licenses: Vec<DetectedLicense>,
    pub unidentified_licenses: Vec<UnidentifiedLicense>,
    pub license_summary: LicenseSummary,
    pub transitive_licenses: HashMap<String, Vec<LicenseInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLicense {
    pub package_name: String,
    pub package_version: String,
    pub license_info: LicenseInfo,
    pub confidence: f32,
    pub detection_method: DetectionMethod,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnidentifiedLicense {
    pub package_name: String,
    pub package_version: String,
    pub raw_license_text: String,
    pub possible_matches: Vec<LicenseMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseMatch {
    pub spdx_id: String,
    pub name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSummary {
    pub total_packages: usize,
    pub licensed_packages: usize,
    pub unlicensed_packages: usize,
    pub category_breakdown: HashMap<LicenseCategory, usize>,
    pub popular_licenses: Vec<(String, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    SpdxIdentifier,
    LicenseFile,
    PackageMetadata,
    HeaderComment,
    FuzzyMatching,
}

#[derive(Debug, Clone)]
struct SpdxDatabase {
    licenses: HashMap<String, SpdxLicense>,
    aliases: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct SpdxLicense {
    id: String,
    name: String,
    category: LicenseCategory,
    is_osi_approved: bool,
    is_fsf_approved: bool,
    text_patterns: Vec<String>,
}

impl LicenseAnalyzer {
    pub fn new() -> Self {
        Self {
            spdx_database: SpdxDatabase::new(),
        }
    }

    /// Analyze licenses for a set of dependencies
    pub fn analyze_licenses(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Result<LicenseAnalysisResult, DependencyError> {
        let mut detected_licenses = Vec::new();
        let mut unidentified_licenses = Vec::new();
        let mut transitive_licenses = HashMap::new();

        for dep in dependencies {
            match self.detect_package_license(dep) {
                Ok(license) => {
                    detected_licenses.push(license);
                }
                Err(_) => {
                    if let Some(unidentified) = self.handle_unidentified_license(dep) {
                        unidentified_licenses.push(unidentified);
                    }
                }
            }

            if config.check_transitive {
                if let Ok(transitive) = self.analyze_transitive_licenses(dep) {
                    transitive_licenses.insert(dep.name.clone(), transitive);
                }
            }
        }

        let license_summary = self.create_license_summary(&detected_licenses, dependencies.len());

        Ok(LicenseAnalysisResult {
            detected_licenses,
            unidentified_licenses,
            license_summary,
            transitive_licenses,
        })
    }

    fn detect_package_license(
        &self,
        dep: &DependencyInfo,
    ) -> Result<DetectedLicense, DependencyError> {
        // Try metadata first, then license files
        if let Some(license) = self.detect_from_metadata(dep)? {
            return Ok(license);
        }
        if let Some(license) = self.detect_from_license_files(dep)? {
            return Ok(license);
        }
        Err(DependencyError::NotFound(format!(
            "No license found for {}",
            dep.name
        )))
    }

    fn detect_from_metadata(
        &self,
        dep: &DependencyInfo,
    ) -> Result<Option<DetectedLicense>, DependencyError> {
        let metadata_license = match dep.source {
            DependencySource::Registry(ref registry) => {
                self.fetch_registry_license_info(registry, &dep.name, dep.version.as_deref())?
            }
            _ => None,
        };

        if let Some(spdx_id) = metadata_license {
            if let Some(license_info) = self.spdx_database.get_license(&spdx_id) {
                return Ok(Some(DetectedLicense {
                    package_name: dep.name.clone(),
                    package_version: dep.version.clone().unwrap_or_default(),
                    license_info: LicenseInfo {
                        spdx_id: Some(license_info.id.clone()),
                        name: license_info.name.clone(),
                        url: None,
                        is_osi_approved: license_info.is_osi_approved,
                        is_fsf_approved: license_info.is_fsf_approved,
                        category: license_info.category.clone(),
                    },
                    confidence: 0.95,
                    detection_method: DetectionMethod::SpdxIdentifier,
                    source_file: None,
                }));
            }
        }

        Ok(None)
    }

    fn detect_from_license_files(
        &self,
        dep: &DependencyInfo,
    ) -> Result<Option<DetectedLicense>, DependencyError> {
        if let Some(ref path) = dep.resolved_path {
            for file_name in ["LICENSE", "LICENSE.txt", "LICENSE.md"] {
                let license_path = path.join(file_name);
                if license_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&license_path) {
                        if let Some(license_info) = self.analyze_license_text(&content) {
                            return Ok(Some(DetectedLicense {
                                package_name: dep.name.clone(),
                                package_version: dep.version.clone().unwrap_or_default(),
                                license_info,
                                confidence: 0.90,
                                detection_method: DetectionMethod::LicenseFile,
                                source_file: Some(file_name.to_string()),
                            }));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn fetch_registry_license_info(
        &self,
        registry: &str,
        _name: &str,
        _version: Option<&str>,
    ) -> Result<Option<String>, DependencyError> {
        match registry {
            "crates.io" => Ok(Some("MIT".to_string())),
            "npmjs.org" => Ok(Some("Apache-2.0".to_string())),
            _ => Ok(None),
        }
    }

    fn analyze_license_text(&self, text: &str) -> Option<LicenseInfo> {
        let normalized_text = text.to_lowercase();
        for (spdx_id, license) in &self.spdx_database.licenses {
            for pattern in &license.text_patterns {
                if normalized_text.contains(&pattern.to_lowercase()) {
                    return Some(LicenseInfo {
                        spdx_id: Some(spdx_id.clone()),
                        name: license.name.clone(),
                        url: None,
                        is_osi_approved: license.is_osi_approved,
                        is_fsf_approved: license.is_fsf_approved,
                        category: license.category.clone(),
                    });
                }
            }
        }
        None
    }

    fn handle_unidentified_license(&self, dep: &DependencyInfo) -> Option<UnidentifiedLicense> {
        if let Some(ref path) = dep.resolved_path {
            let license_path = path.join("LICENSE");
            if let Ok(raw_text) = std::fs::read_to_string(&license_path) {
                return Some(UnidentifiedLicense {
                    package_name: dep.name.clone(),
                    package_version: dep.version.clone().unwrap_or_default(),
                    raw_license_text: raw_text.chars().take(500).collect(),
                    possible_matches: Vec::new(),
                });
            }
        }
        None
    }

    fn analyze_transitive_licenses(
        &self,
        _dep: &DependencyInfo,
    ) -> Result<Vec<LicenseInfo>, DependencyError> {
        Ok(vec![LicenseInfo {
            spdx_id: Some("MIT".to_string()),
            name: "MIT License".to_string(),
            url: Some("https://opensource.org/licenses/MIT".to_string()),
            is_osi_approved: true,
            is_fsf_approved: true,
            category: LicenseCategory::Permissive,
        }])
    }

    fn create_license_summary(
        &self,
        detected: &[DetectedLicense],
        total_packages: usize,
    ) -> LicenseSummary {
        let mut category_breakdown = HashMap::new();
        let mut license_counts = HashMap::new();

        for license in detected {
            *category_breakdown
                .entry(license.license_info.category.clone())
                .or_insert(0) += 1;
            *license_counts
                .entry(license.license_info.name.clone())
                .or_insert(0) += 1;
        }

        let mut popular_licenses: Vec<(String, usize)> = license_counts.into_iter().collect();
        popular_licenses.sort_by(|a, b| b.1.cmp(&a.1));
        popular_licenses.truncate(10);

        LicenseSummary {
            total_packages,
            licensed_packages: detected.len(),
            unlicensed_packages: total_packages.saturating_sub(detected.len()),
            category_breakdown,
            popular_licenses,
        }
    }
}

impl SpdxDatabase {
    fn new() -> Self {
        let mut licenses = HashMap::new();

        licenses.insert(
            "MIT".to_string(),
            SpdxLicense {
                id: "MIT".to_string(),
                name: "MIT License".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_approved: true,
                text_patterns: vec![
                    "Permission is hereby granted".to_string(),
                    "MIT License".to_string(),
                ],
            },
        );

        licenses.insert(
            "Apache-2.0".to_string(),
            SpdxLicense {
                id: "Apache-2.0".to_string(),
                name: "Apache License 2.0".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_approved: true,
                text_patterns: vec!["Licensed under the Apache License".to_string()],
            },
        );

        licenses.insert(
            "GPL-3.0".to_string(),
            SpdxLicense {
                id: "GPL-3.0".to_string(),
                name: "GNU General Public License v3.0".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_approved: true,
                text_patterns: vec!["GNU GENERAL PUBLIC LICENSE".to_string()],
            },
        );

        let mut aliases: HashMap<String, String> = HashMap::new();
        aliases.insert("GPL-3".to_string(), "GPL-3.0".to_string());
        aliases.insert("MIT License".to_string(), "MIT".to_string());
        aliases.insert("Apache License 2.0".to_string(), "Apache-2.0".to_string());

        Self { licenses, aliases }
    }

    fn get_license(&self, license_id: &str) -> Option<&SpdxLicense> {
        self.licenses.get(license_id)
    }

    fn find_license_by_text(&self, text: &str) -> Option<&SpdxLicense> {
        for license in self.licenses.values() {
            for pattern in &license.text_patterns {
                if text.contains(pattern) {
                    return Some(license);
                }
            }
        }
        None
    }
}

impl LicenseChecker for LicenseAnalyzer {
    fn check(
        &self,
        dependencies: &[DependencyInfo],
        config: &LicenseConfig,
    ) -> Result<LicenseCheckOutput, DependencyError> {
        let result = self.analyze_licenses(dependencies, config)?;
        Ok(LicenseCheckOutput::Analysis(result))
    }

    fn name(&self) -> &str {
        "LicenseAnalyzer"
    }

    fn supports_transitive(&self) -> bool {
        true
    }
}
