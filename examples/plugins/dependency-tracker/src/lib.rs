//! Dependency Vulnerability Tracker Plugin
//! 
//! This plugin tracks dependencies across multiple package managers and
//! identifies vulnerabilities, licensing issues, and outdated packages.
//! Demonstrates advanced network usage and database caching.

use std::collections::{HashMap, HashSet};

wit_bindgen::generate!({
    world: "core-analysis",
    path: "../../../wit/core-analysis.wit",
});

use exports::{initialize, analyze, get_info, cleanup};

static mut PLUGIN_STATE: Option<DependencyTracker> = None;

struct DependencyTracker {
    config: PluginConfig,
    vulnerability_cache: HashMap<String, VulnerabilityInfo>,
    license_cache: HashMap<String, LicenseInfo>,
    dependency_graph: HashMap<String, DependencyNode>,
    scan_count: u32,
    package_managers: Vec<PackageManager>,
}

#[derive(Debug, Clone)]
struct DependencyNode {
    name: String,
    version: String,
    package_manager: String,
    direct_dependency: bool,
    dependencies: Vec<String>,
    vulnerabilities: Vec<VulnerabilityInfo>,
    license: Option<LicenseInfo>,
    last_updated: Option<String>,
    deprecated: bool,
}

#[derive(Debug, Clone)]
struct VulnerabilityInfo {
    id: String,
    severity: SeverityLevel,
    title: String,
    description: String,
    affected_versions: String,
    patched_versions: Option<String>,
    published_date: String,
    cve_id: Option<String>,
    cvss_score: Option<f64>,
}

#[derive(Debug, Clone)]
struct LicenseInfo {
    spdx_id: String,
    name: String,
    approved: bool,
    copyleft: bool,
    commercial_use: bool,
    attribution_required: bool,
    notice_required: bool,
}

#[derive(Debug, Clone)]
struct PackageManager {
    name: String,
    manifest_files: Vec<String>,
    lock_files: Vec<String>,
    registry_api: String,
}

impl DependencyTracker {
    fn new() -> Self {
        Self {
            config: PluginConfig {
                severity_threshold: SeverityLevel::Low,
                max_issues_per_file: 200,
                include_patterns: vec![
                    "Cargo.toml".to_string(),
                    "Cargo.lock".to_string(),
                    "package.json".to_string(),
                    "package-lock.json".to_string(),
                    "yarn.lock".to_string(),
                    "requirements.txt".to_string(),
                    "Pipfile".to_string(),
                    "Pipfile.lock".to_string(),
                    "go.mod".to_string(),
                    "go.sum".to_string(),
                ],
                exclude_patterns: vec![],
                rule_overrides: vec![],
                custom_settings: vec![
                    ("check_vulnerabilities".to_string(), "true".to_string()),
                    ("check_licenses".to_string(), "true".to_string()),
                    ("check_outdated".to_string(), "true".to_string()),
                    ("max_vulnerability_age_days".to_string(), "30".to_string()),
                ],
            },
            vulnerability_cache: HashMap::new(),
            license_cache: HashMap::new(),
            dependency_graph: HashMap::new(),
            scan_count: 0,
            package_managers: Self::initialize_package_managers(),
        }
    }

    fn initialize_package_managers() -> Vec<PackageManager> {
        vec![
            PackageManager {
                name: "cargo".to_string(),
                manifest_files: vec!["Cargo.toml".to_string()],
                lock_files: vec!["Cargo.lock".to_string()],
                registry_api: "https://crates.io/api/v1".to_string(),
            },
            PackageManager {
                name: "npm".to_string(),
                manifest_files: vec!["package.json".to_string()],
                lock_files: vec!["package-lock.json".to_string(), "yarn.lock".to_string()],
                registry_api: "https://registry.npmjs.org".to_string(),
            },
            PackageManager {
                name: "pip".to_string(),
                manifest_files: vec!["requirements.txt".to_string(), "Pipfile".to_string()],
                lock_files: vec!["Pipfile.lock".to_string()],
                registry_api: "https://pypi.org/pypi".to_string(),
            },
            PackageManager {
                name: "go".to_string(),
                manifest_files: vec!["go.mod".to_string()],
                lock_files: vec!["go.sum".to_string()],
                registry_api: "https://proxy.golang.org".to_string(),
            },
        ]
    }

    fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
        self.scan_count += 1;
        
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        log(LogLevel::Info, &format!("Analyzing dependencies in {}", file.path));

        let mut issues = Vec::new();
        let mut total_dependencies = 0;
        let mut vulnerable_dependencies = 0;
        let mut outdated_dependencies = 0;

        // Determine package manager
        let package_manager = self.detect_package_manager(&file.path);
        if package_manager.is_none() {
            return Ok(AnalysisResult {
                issues: vec![],
                metrics: self.create_empty_metrics(),
                dependencies: vec![],
                exports: vec![],
                duration_ms: 0,
                plugin_version: "1.0.0".to_string(),
            });
        }

        let pm = package_manager.unwrap();
        log(LogLevel::Info, &format!("Detected package manager: {}", pm.name));

        // Parse dependencies from file
        let dependencies = self.parse_dependencies(&file, &pm)?;
        total_dependencies = dependencies.len();

        // Analyze each dependency
        for dep in dependencies {
            // Check vulnerabilities
            let vulnerabilities = self.check_vulnerabilities(&dep).await?;
            if !vulnerabilities.is_empty() {
                vulnerable_dependencies += 1;
                let vuln_issues = self.create_vulnerability_issues(&dep, &vulnerabilities, &file);
                issues.extend(vuln_issues);
            }

            // Check for outdated versions
            if self.is_outdated(&dep).await? {
                outdated_dependencies += 1;
                let outdated_issue = self.create_outdated_issue(&dep, &file);
                issues.push(outdated_issue);
            }

            // Check license compliance
            if let Some(license_issue) = self.check_license_compliance(&dep, &file).await? {
                issues.push(license_issue);
            }

            // Store in dependency graph
            let mut node = DependencyNode {
                name: dep.name.clone(),
                version: dep.version.clone(),
                package_manager: pm.name.clone(),
                direct_dependency: dep.direct_dependency,
                dependencies: dep.dependencies,
                vulnerabilities,
                license: self.license_cache.get(&dep.name).cloned(),
                last_updated: None,
                deprecated: false,
            };

            // Check if deprecated
            if self.is_deprecated(&dep).await? {
                node.deprecated = true;
                let deprecated_issue = self.create_deprecated_issue(&dep, &file);
                issues.push(deprecated_issue);
            }

            self.dependency_graph.insert(dep.name.clone(), node);
        }

        // Calculate dependency metrics
        let dependency_depth = self.calculate_dependency_depth();
        let license_diversity = self.calculate_license_diversity();
        let security_score = self.calculate_security_score(vulnerable_dependencies, total_dependencies);
        let freshness_score = self.calculate_freshness_score(outdated_dependencies, total_dependencies);

        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: 0,
            complexity: 1,
            maintainability_index: 100.0,
            technical_debt_minutes: self.calculate_dependency_debt(&issues),
            custom_metrics: vec![
                ("total_dependencies".to_string(), total_dependencies as f64),
                ("vulnerable_dependencies".to_string(), vulnerable_dependencies as f64),
                ("outdated_dependencies".to_string(), outdated_dependencies as f64),
                ("dependency_depth".to_string(), dependency_depth),
                ("license_diversity".to_string(), license_diversity),
                ("security_score".to_string(), security_score),
                ("freshness_score".to_string(), freshness_score),
            ],
        };

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        // Cache results for future use
        self.cache_results().await?;

        Ok(AnalysisResult {
            issues,
            metrics,
            dependencies: self.dependency_graph.keys().cloned().collect(),
            exports: vec![],
            duration_ms: end_time - start_time,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn detect_package_manager(&self, file_path: &str) -> Option<&PackageManager> {
        let filename = file_path.split('/').last().unwrap_or(file_path);
        
        for pm in &self.package_managers {
            if pm.manifest_files.contains(&filename.to_string()) || 
               pm.lock_files.contains(&filename.to_string()) {
                return Some(pm);
            }
        }
        None
    }

    fn parse_dependencies(&self, file: &SourceFile, pm: &PackageManager) -> Result<Vec<ParsedDependency>, String> {
        let mut dependencies = Vec::new();
        let filename = file.path.split('/').last().unwrap_or(&file.path);

        match pm.name.as_str() {
            "cargo" if filename == "Cargo.toml" => {
                dependencies.extend(self.parse_cargo_toml(&file.content)?);
            },
            "npm" if filename == "package.json" => {
                dependencies.extend(self.parse_package_json(&file.content)?);
            },
            "pip" if filename == "requirements.txt" => {
                dependencies.extend(self.parse_requirements_txt(&file.content)?);
            },
            "go" if filename == "go.mod" => {
                dependencies.extend(self.parse_go_mod(&file.content)?);
            },
            _ => {
                log(LogLevel::Debug, &format!("Unsupported file type for parsing: {}", filename));
            }
        }

        Ok(dependencies)
    }

    fn parse_cargo_toml(&self, content: &str) -> Result<Vec<ParsedDependency>, String> {
        let mut dependencies = Vec::new();
        let mut in_dependencies_section = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line == "[dependencies]" || line == "[dev-dependencies]" {
                in_dependencies_section = true;
                continue;
            }
            
            if line.starts_with('[') && line != "[dependencies]" && line != "[dev-dependencies]" {
                in_dependencies_section = false;
                continue;
            }
            
            if in_dependencies_section && !line.is_empty() && !line.starts_with('#') {
                if let Some(eq_pos) = line.find('=') {
                    let name = line[..eq_pos].trim().to_string();
                    let version_part = line[eq_pos + 1..].trim();
                    
                    // Simple version extraction (real implementation would use TOML parser)
                    let version = if version_part.starts_with('"') {
                        version_part.trim_matches('"').to_string()
                    } else {
                        "unknown".to_string()
                    };
                    
                    dependencies.push(ParsedDependency {
                        name,
                        version,
                        direct_dependency: true,
                        dependencies: vec![],
                    });
                }
            }
        }
        
        Ok(dependencies)
    }

    fn parse_package_json(&self, content: &str) -> Result<Vec<ParsedDependency>, String> {
        let mut dependencies = Vec::new();
        let mut in_dependencies = false;
        let mut brace_count = 0;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.contains("\"dependencies\"") || line.contains("\"devDependencies\"") {
                in_dependencies = true;
                continue;
            }
            
            if in_dependencies {
                for ch in line.chars() {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count < 0 {
                                in_dependencies = false;
                                break;
                            }
                        },
                        _ => {}
                    }
                }
                
                if line.contains(':') && !line.contains("dependencies") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let name = parts[0].trim_matches([' ', '"', ',']).to_string();
                        let version = parts[1].trim_matches([' ', '"', ',']).to_string();
                        
                        if !name.is_empty() && !version.is_empty() {
                            dependencies.push(ParsedDependency {
                                name,
                                version,
                                direct_dependency: true,
                                dependencies: vec![],
                            });
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }

    fn parse_requirements_txt(&self, content: &str) -> Result<Vec<ParsedDependency>, String> {
        let mut dependencies = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse requirement specifiers (simplified)
            let parts: Vec<&str> = if line.contains("==") {
                line.split("==").collect()
            } else if line.contains(">=") {
                line.split(">=").collect() 
            } else if line.contains("<=") {
                line.split("<=").collect()
            } else {
                vec![line, "unknown"]
            };
            
            if parts.len() >= 2 {
                dependencies.push(ParsedDependency {
                    name: parts[0].trim().to_string(),
                    version: parts[1].trim().to_string(),
                    direct_dependency: true,
                    dependencies: vec![],
                });
            }
        }
        
        Ok(dependencies)
    }

    fn parse_go_mod(&self, content: &str) -> Result<Vec<ParsedDependency>, String> {
        let mut dependencies = Vec::new();
        let mut in_require_block = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line == "require (" {
                in_require_block = true;
                continue;
            }
            
            if line == ")" && in_require_block {
                in_require_block = false;
                continue;
            }
            
            if line.starts_with("require ") && !line.contains("(") {
                // Single require statement
                let parts: Vec<&str> = line[8..].split_whitespace().collect();
                if parts.len() >= 2 {
                    dependencies.push(ParsedDependency {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        direct_dependency: true,
                        dependencies: vec![],
                    });
                }
            } else if in_require_block && !line.is_empty() && !line.starts_with("//") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    dependencies.push(ParsedDependency {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        direct_dependency: true,
                        dependencies: vec![],
                    });
                }
            }
        }
        
        Ok(dependencies)
    }

    async fn check_vulnerabilities(&mut self, dep: &ParsedDependency) -> Result<Vec<VulnerabilityInfo>, String> {
        // Check cache first
        let cache_key = format!("{}:{}", dep.name, dep.version);
        if let Some(cached) = self.vulnerability_cache.get(&cache_key) {
            return Ok(vec![cached.clone()]);
        }

        // Check database cache
        if let Some(cached_data) = db_get(&format!("vuln:{}", cache_key)) {
            // In real implementation, deserialize JSON
            log(LogLevel::Debug, "Using cached vulnerability data");
            return Ok(vec![]);
        }

        // Fetch from vulnerability database
        let vulnerabilities = match self.fetch_vulnerabilities(dep).await {
            Ok(vulns) => vulns,
            Err(e) => {
                log(LogLevel::Warn, &format!("Failed to fetch vulnerabilities for {}: {}", dep.name, e));
                vec![]
            }
        };

        // Cache the results
        for vuln in &vulnerabilities {
            self.vulnerability_cache.insert(cache_key.clone(), vuln.clone());
            let _ = db_set(&format!("vuln:{}", cache_key), "cached_data", Some(3600));
        }

        Ok(vulnerabilities)
    }

    async fn fetch_vulnerabilities(&self, dep: &ParsedDependency) -> Result<Vec<VulnerabilityInfo>, String> {
        // Simulate fetching from OSV, GitHub Advisory, or similar database
        let url = format!("https://api.osv.dev/v1/query");
        let payload = format!(r#"{{"package": {{"name": "{}", "ecosystem": "crates.io"}}, "version": "{}"}}"#, 
                              dep.name, dep.version);

        match http_post(url, payload, vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("User-Agent".to_string(), "Uveddi-Dependency-Tracker/1.0".to_string()),
        ]) {
            Ok(response) => {
                if response.status == 200 {
                    // In real implementation, parse JSON response
                    // For demo, return sample vulnerability if name contains "vulnerable"
                    if dep.name.contains("vulnerable") || dep.name == "lodash" {
                        Ok(vec![VulnerabilityInfo {
                            id: "OSV-2024-1234".to_string(),
                            severity: SeverityLevel::High,
                            title: format!("Vulnerability in {}", dep.name),
                            description: "Sample vulnerability for demonstration".to_string(),
                            affected_versions: dep.version.clone(),
                            patched_versions: Some(">=1.2.3".to_string()),
                            published_date: "2024-01-01".to_string(),
                            cve_id: Some("CVE-2024-1234".to_string()),
                            cvss_score: Some(7.5),
                        }])
                    } else {
                        Ok(vec![])
                    }
                } else {
                    Err(format!("HTTP error: {}", response.status))
                }
            },
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    async fn is_outdated(&self, dep: &ParsedDependency) -> Result<bool, String> {
        // Check if package has newer versions available
        // This is a simplified check
        Ok(dep.version.starts_with("0.") || dep.version.starts_with("1.0."))
    }

    async fn check_license_compliance(&mut self, dep: &ParsedDependency, file: &SourceFile) -> Result<Option<Issue>, String> {
        // Fetch license information
        let license_info = match self.get_license_info(dep).await {
            Ok(Some(info)) => info,
            Ok(None) => return Ok(None),
            Err(e) => {
                log(LogLevel::Warn, &format!("Failed to get license info for {}: {}", dep.name, e));
                return Ok(None);
            }
        };

        // Check if license is approved for use
        if !license_info.approved {
            return Ok(Some(Issue {
                id: "DEP004".to_string(),
                severity: SeverityLevel::Medium,
                category: IssueCategory::Security,
                message: format!("Unapproved license: {}", license_info.name),
                description: Some(format!("Dependency '{}' uses license '{}' which may not be approved for use in this project.", dep.name, license_info.name)),
                file: file.path.clone(),
                span: Span {
                    start: Position { line: 1, column: 1, byte_offset: 0 },
                    end: Position { line: 1, column: 1, byte_offset: 0 },
                },
                rule_id: Some("unapproved_license".to_string()),
                suggestion: Some("Review license compatibility with project requirements".to_string()),
                fix: None,
                metadata: vec![
                    ("dependency".to_string(), dep.name.clone()),
                    ("license".to_string(), license_info.spdx_id),
                    ("copyleft".to_string(), license_info.copyleft.to_string()),
                ],
            }));
        }

        Ok(None)
    }

    async fn get_license_info(&mut self, dep: &ParsedDependency) -> Result<Option<LicenseInfo>, String> {
        // Check cache first
        if let Some(cached) = self.license_cache.get(&dep.name) {
            return Ok(Some(cached.clone()));
        }

        // Simulate fetching license info
        let license_info = LicenseInfo {
            spdx_id: "MIT".to_string(),
            name: "MIT License".to_string(),
            approved: true,
            copyleft: false,
            commercial_use: true,
            attribution_required: true,
            notice_required: false,
        };

        self.license_cache.insert(dep.name.clone(), license_info.clone());
        Ok(Some(license_info))
    }

    async fn is_deprecated(&self, dep: &ParsedDependency) -> Result<bool, String> {
        // Check if package is marked as deprecated
        Ok(dep.name.contains("deprecated") || dep.name == "bower")
    }

    fn create_vulnerability_issues(
        &self,
        dep: &ParsedDependency,
        vulnerabilities: &[VulnerabilityInfo],
        file: &SourceFile,
    ) -> Vec<Issue> {
        vulnerabilities.iter().map(|vuln| Issue {
            id: "DEP001".to_string(),
            severity: vuln.severity.clone(),
            category: IssueCategory::Security,
            message: format!("Vulnerable dependency: {} ({})", dep.name, vuln.id),
            description: Some(format!("{}\n\nAffected versions: {}\nPatched versions: {}", 
                                      vuln.description, vuln.affected_versions,
                                      vuln.patched_versions.as_ref().unwrap_or(&"Unknown".to_string()))),
            file: file.path.clone(),
            span: Span {
                start: Position { line: 1, column: 1, byte_offset: 0 },
                end: Position { line: 1, column: 1, byte_offset: 0 },
            },
            rule_id: Some("vulnerable_dependency".to_string()),
            suggestion: Some(format!("Update {} to a patched version", dep.name)),
            fix: None,
            metadata: vec![
                ("dependency".to_string(), dep.name.clone()),
                ("vulnerability_id".to_string(), vuln.id.clone()),
                ("cvss_score".to_string(), vuln.cvss_score.map_or("Unknown".to_string(), |s| s.to_string())),
                ("cve_id".to_string(), vuln.cve_id.as_ref().unwrap_or(&"Unknown".to_string()).clone()),
            ],
        }).collect()
    }

    fn create_outdated_issue(&self, dep: &ParsedDependency, file: &SourceFile) -> Issue {
        Issue {
            id: "DEP002".to_string(),
            severity: SeverityLevel::Low,
            category: IssueCategory::Maintainability,
            message: format!("Outdated dependency: {}", dep.name),
            description: Some(format!("Dependency '{}' version '{}' is outdated. Consider updating to the latest version for bug fixes and security patches.", dep.name, dep.version)),
            file: file.path.clone(),
            span: Span {
                start: Position { line: 1, column: 1, byte_offset: 0 },
                end: Position { line: 1, column: 1, byte_offset: 0 },
            },
            rule_id: Some("outdated_dependency".to_string()),
            suggestion: Some("Update to the latest version".to_string()),
            fix: None,
            metadata: vec![
                ("dependency".to_string(), dep.name.clone()),
                ("current_version".to_string(), dep.version.clone()),
            ],
        }
    }

    fn create_deprecated_issue(&self, dep: &ParsedDependency, file: &SourceFile) -> Issue {
        Issue {
            id: "DEP003".to_string(),
            severity: SeverityLevel::Medium,
            category: IssueCategory::Maintainability,
            message: format!("Deprecated dependency: {}", dep.name),
            description: Some(format!("Dependency '{}' is deprecated and no longer maintained. Consider migrating to an alternative package.", dep.name)),
            file: file.path.clone(),
            span: Span {
                start: Position { line: 1, column: 1, byte_offset: 0 },
                end: Position { line: 1, column: 1, byte_offset: 0 },
            },
            rule_id: Some("deprecated_dependency".to_string()),
            suggestion: Some("Find an alternative maintained package".to_string()),
            fix: None,
            metadata: vec![
                ("dependency".to_string(), dep.name.clone()),
                ("version".to_string(), dep.version.clone()),
            ],
        }
    }

    fn calculate_dependency_depth(&self) -> f64 {
        // Calculate the maximum depth of the dependency tree
        // Simplified calculation
        if self.dependency_graph.is_empty() {
            return 0.0;
        }
        
        let max_deps = self.dependency_graph.values()
            .map(|node| node.dependencies.len())
            .max()
            .unwrap_or(0);
        
        max_deps as f64
    }

    fn calculate_license_diversity(&self) -> f64 {
        let unique_licenses: HashSet<_> = self.dependency_graph.values()
            .filter_map(|node| node.license.as_ref())
            .map(|license| &license.spdx_id)
            .collect();
        
        unique_licenses.len() as f64
    }

    fn calculate_security_score(&self, vulnerable: usize, total: usize) -> f64 {
        if total == 0 {
            return 100.0;
        }
        
        let vulnerability_ratio = vulnerable as f64 / total as f64;
        (100.0 - (vulnerability_ratio * 100.0)).max(0.0)
    }

    fn calculate_freshness_score(&self, outdated: usize, total: usize) -> f64 {
        if total == 0 {
            return 100.0;
        }
        
        let outdated_ratio = outdated as f64 / total as f64;
        (100.0 - (outdated_ratio * 50.0)).max(0.0)
    }

    fn calculate_dependency_debt(&self, issues: &[Issue]) -> u32 {
        issues.iter().map(|issue| {
            match issue.id.as_str() {
                "DEP001" => match issue.severity {
                    SeverityLevel::Critical => 480, // 8 hours
                    SeverityLevel::High => 240,     // 4 hours  
                    SeverityLevel::Medium => 120,   // 2 hours
                    SeverityLevel::Low => 60,       // 1 hour
                    SeverityLevel::Info => 30,      // 30 minutes
                },
                "DEP002" => 30, // Outdated dependency = 30 minutes
                "DEP003" => 120, // Deprecated dependency = 2 hours
                "DEP004" => 60, // License issue = 1 hour
                _ => 15,
            }
        }).sum()
    }

    fn create_empty_metrics(&self) -> Metrics {
        Metrics {
            lines_of_code: 0,
            lines_of_comments: 0,
            complexity: 1,
            maintainability_index: 100.0,
            technical_debt_minutes: 0,
            custom_metrics: vec![],
        }
    }

    async fn cache_results(&self) -> Result<(), String> {
        // Cache dependency graph and analysis results
        let _ = db_set("dependency_graph", "cached_data", Some(3600));
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ParsedDependency {
    name: String,
    version: String,
    direct_dependency: bool,
    dependencies: Vec<String>,
}

// Plugin lifecycle implementation
impl initialize {
    fn call(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Dependency Vulnerability Tracker Plugin");
        
        unsafe {
            let mut tracker = DependencyTracker::new();
            tracker.config = config;
            PLUGIN_STATE = Some(tracker);
        }
        
        log(LogLevel::Info, "Dependency Tracker Plugin initialized successfully");
        Ok(())
    }
}

impl analyze {
    fn call(file: SourceFile) -> Result<AnalysisResult, String> {
        unsafe {
            if let Some(ref mut state) = PLUGIN_STATE {
                // Note: This would normally be async, but the WIT interface is sync
                // In a real implementation, use tokio::Runtime or similar
                state.analyze_file(file)
            } else {
                Err("Plugin not initialized".to_string())
            }
        }
    }
}

impl get_info {
    fn call() -> PluginInfo {
        PluginInfo {
            id: "dependency-tracker".to_string(),
            name: "Dependency Vulnerability Tracker".to_string(),
            version: "1.0.0".to_string(),
            description: "Tracks dependencies across multiple package managers and identifies vulnerabilities, licensing issues, and outdated packages".to_string(),
            author: "Uveddi Security Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://github.com/uveddi/plugins/dependency-tracker".to_string()),
            supported_languages: vec![
                "rust".to_string(),
                "javascript".to_string(),
                "typescript".to_string(), 
                "python".to_string(),
                "go".to_string(),
                "java".to_string(),
            ],
            detector_types: vec![IssueCategory::Security, IssueCategory::Maintainability],
            api_version: "1.0".to_string(),
            required_permissions: vec![
                Permission::ReadFiles.into(),
                Permission::NetworkAccess.into(), // For vulnerability and package data
            ],
        }
    }
}

impl cleanup {
    fn call() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up Dependency Tracker Plugin");
        
        unsafe {
            if let Some(ref state) = PLUGIN_STATE {
                log(LogLevel::Info, &format!("Scanned {} dependency files during session", state.scan_count));
                log(LogLevel::Info, &format!("Tracked {} unique dependencies", state.dependency_graph.len()));
            }
            PLUGIN_STATE = None;
        }
        
        Ok(())
    }
}