//! Dependency vulnerability scanner for OWASP vulnerabilities

use super::{Scanner, UnifiedScanResult};
use crate::analysis::detectors::security::owasp::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerableDependency {
    pub package_name: String,
    pub version: String,
    pub vulnerability_id: String,
    pub severity: SecuritySeverity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyScanResult {
    pub vulnerable_dependencies: Vec<VulnerableDependency>,
    pub total_dependencies: usize,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct VulnerabilityDatabase {
    vulnerabilities: HashMap<String, Vec<VulnerableDependency>>,
}

impl VulnerabilityDatabase {
    pub fn new() -> Self {
        let mut db = VulnerabilityDatabase {
            vulnerabilities: HashMap::new(),
        };
        db.load_sample_data();
        db
    }

    fn load_sample_data(&mut self) {
        let vuln_lodash = VulnerableDependency {
            package_name: "lodash".to_string(),
            version: "4.17.0".to_string(),
            vulnerability_id: "CVE-2020-8203".to_string(),
            severity: SecuritySeverity::High,
            description: "Prototype pollution vulnerability".to_string(),
        };

        let vuln_django = VulnerableDependency {
            package_name: "django".to_string(),
            version: "2.2.0".to_string(),
            vulnerability_id: "CVE-2021-31542".to_string(),
            severity: SecuritySeverity::Medium,
            description: "Directory traversal vulnerability".to_string(),
        };

        self.vulnerabilities
            .insert("lodash".to_string(), vec![vuln_lodash]);
        self.vulnerabilities
            .insert("django".to_string(), vec![vuln_django]);
    }

    pub fn check_vulnerability(
        &self,
        package: &str,
        version: &str,
    ) -> Option<&VulnerableDependency> {
        self.vulnerabilities
            .get(package)
            .and_then(|vulns| vulns.iter().find(|v| version.starts_with(&v.version)))
    }
}

/// Dependency vulnerability scanner
pub struct DependencyScanner {
    vulnerability_db: VulnerabilityDatabase,
}

impl DependencyScanner {
    pub fn new() -> Self {
        Self {
            vulnerability_db: VulnerabilityDatabase::new(),
        }
    }

    fn parse_package_json(&self, content: &str) -> Vec<(String, String)> {
        let mut dependencies = Vec::new();

        // Try JSON parsing first for compact JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            // Check "dependencies" and "devDependencies"
            for dep_key in &["dependencies", "devDependencies"] {
                if let Some(deps) = json.get(dep_key).and_then(|v| v.as_object()) {
                    for (package, version) in deps {
                        if let Some(v) = version.as_str() {
                            // Strip semver prefixes (^, ~, >=, etc.)
                            let clean_version = v.trim_start_matches(|c| c == '^' || c == '~' || c == '>' || c == '=' || c == '<');
                            dependencies.push((package.clone(), clean_version.to_string()));
                        }
                    }
                }
            }
            return dependencies;
        }

        // Fallback to line-by-line parsing for malformed JSON
        for line in content.lines() {
            if line.contains('"') && line.contains(':') {
                if let Some(package_info) = self.extract_js_dependency(line) {
                    dependencies.push(package_info);
                }
            }
        }
        dependencies
    }

    fn parse_requirements_txt(&self, content: &str) -> Vec<(String, String)> {
        content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                if let Some(eq_pos) = line.find("==") {
                    let package = line[..eq_pos].trim().to_string();
                    let version = line[eq_pos + 2..].trim().to_string();
                    Some((package, version))
                } else {
                    None
                }
            })
            .collect()
    }

    fn parse_cargo_toml(&self, content: &str) -> Vec<(String, String)> {
        let mut dependencies = Vec::new();
        let mut in_dependencies = false;

        for line in content.lines() {
            let line = line.trim();
            if line == "[dependencies]" {
                in_dependencies = true;
                continue;
            }
            if line.starts_with('[') && line != "[dependencies]" {
                in_dependencies = false;
                continue;
            }
            if in_dependencies && line.contains('=') {
                if let Some((package, version)) = self.extract_cargo_dependency(line) {
                    dependencies.push((package, version));
                }
            }
        }
        dependencies
    }

    fn extract_js_dependency(&self, line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 {
            let package = parts[0].trim().trim_matches('"').trim();
            let version = parts[1].trim().trim_matches(['"', ',', ' ']);
            if !package.is_empty() && !version.is_empty() {
                return Some((package.to_string(), version.to_string()));
            }
        }
        None
    }

    fn extract_cargo_dependency(&self, line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() >= 2 {
            let package = parts[0].trim();
            let version = parts[1].trim().trim_matches(['"', ' ']);
            if !package.is_empty() && !version.is_empty() {
                return Some((package.to_string(), version.to_string()));
            }
        }
        None
    }

    fn check_dependencies(&self, dependencies: &[(String, String)]) -> Vec<VulnerableDependency> {
        dependencies
            .iter()
            .filter_map(|(package, version)| {
                self.vulnerability_db
                    .check_vulnerability(package, version)
                    .cloned()
            })
            .collect()
    }

    fn create_vulnerability(
        &self,
        vuln: &VulnerableDependency,
        location: SecurityLocation,
    ) -> OwaspVulnerability {
        OwaspVulnerability::new(
            OwaspCategory::VulnerableComponents,
            SecurityIssueType::VulnerableComponents,
            format!("Vulnerable dependency: {}", vuln.package_name),
            vuln.description.clone(),
            location,
        )
        .with_severity(vuln.severity)
        .with_confidence(0.9)
    }
}

#[async_trait::async_trait]
impl Scanner for DependencyScanner {
    async fn scan(&self, file: &ParsedFile) -> Result<UnifiedScanResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();

        let file_name = file
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let content = std::fs::read_to_string(&**file.file_path).map_err(|e| {
            AnalysisError::file_system_error(file.file_path.to_string_lossy().to_string(), e)
        })?;

        let dependencies = match file_name {
            "package.json" => self.parse_package_json(&content),
            "requirements.txt" => self.parse_requirements_txt(&content),
            "Cargo.toml" => self.parse_cargo_toml(&content),
            _ => Vec::new(),
        };

        let vulnerable_deps = self.check_dependencies(&dependencies);

        for vuln in &vulnerable_deps {
            let location = SecurityLocation::new(file.file_path.as_ref().to_path_buf(), 1, 1);
            vulnerabilities.push(self.create_vulnerability(vuln, location));
        }

        let result = DependencyScanResult {
            vulnerable_dependencies: vulnerable_deps,
            total_dependencies: dependencies.len(),
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        };

        Ok(UnifiedScanResult {
            vulnerabilities,
            scanner_metadata: serde_json::to_value(&result).unwrap_or_default(),
            scan_duration_ms: result.scan_duration_ms,
        })
    }

    fn name(&self) -> &'static str {
        "DependencyScanner"
    }

    fn supported_languages(&self) -> Vec<SourceLanguage> {
        vec![
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ]
    }

    fn detectable_categories(&self) -> Vec<OwaspCategory> {
        vec![OwaspCategory::VulnerableComponents]
    }
}

impl Default for DependencyScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_package_json_scan() {
        let scanner = DependencyScanner::new();
        let content = r#"{"dependencies": {"lodash": "4.17.0"}}"#;

        let file = ParsedFile {
            file_path: std::sync::Arc::new(PathBuf::from("package.json")),
            language: SourceLanguage::JavaScript,
            source: content.to_string().into(),
            tree: None,
            custom_ast: std::sync::Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        std::fs::write("package.json", content).unwrap();
        let result = scanner.scan(&file).await.unwrap();
        std::fs::remove_file("package.json").unwrap();

        assert!(!result.vulnerabilities.is_empty());
    }
}
