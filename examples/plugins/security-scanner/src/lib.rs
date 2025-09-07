//! Security Vulnerability Scanner Plugin
//! 
//! This plugin scans code for security vulnerabilities and compliance issues.
//! It demonstrates network access permissions for CVE database lookups and
//! cryptographic host functions for verification.

use std::collections::{HashMap, HashSet};

wit_bindgen::generate!({
    world: "core-analysis",
    path: "../../../wit/core-analysis.wit",
});

use exports::{initialize, analyze, get_info, cleanup};

static mut PLUGIN_STATE: Option<SecurityScanner> = None;

struct SecurityScanner {
    config: PluginConfig,
    vulnerability_db: HashMap<String, VulnerabilityInfo>,
    scan_count: u32,
    blocked_patterns: Vec<SecurityPattern>,
}

struct VulnerabilityInfo {
    cve_id: String,
    severity: SeverityLevel,
    description: String,
    fix_version: Option<String>,
}

struct SecurityPattern {
    name: &'static str,
    pattern: &'static str,
    severity: SeverityLevel,
    category: &'static str,
    description: &'static str,
}

impl SecurityScanner {
    fn new() -> Self {
        Self {
            config: PluginConfig {
                severity_threshold: SeverityLevel::Medium,
                max_issues_per_file: 100,
                include_patterns: vec!["*".to_string()],
                exclude_patterns: vec!["*.test.*".to_string(), "test_*".to_string()],
                rule_overrides: vec![],
                custom_settings: vec![
                    ("check_dependencies".to_string(), "true".to_string()),
                    ("crypto_validation".to_string(), "true".to_string()),
                    ("sql_injection_detection".to_string(), "true".to_string()),
                ],
            },
            vulnerability_db: HashMap::new(),
            scan_count: 0,
            blocked_patterns: Self::initialize_security_patterns(),
        }
    }

    fn initialize_security_patterns() -> Vec<SecurityPattern> {
        vec![
            SecurityPattern {
                name: "sql_injection",
                pattern: r#"(?i)(SELECT|INSERT|UPDATE|DELETE).*\+.*["']"#,
                severity: SeverityLevel::Critical,
                category: "injection",
                description: "Potential SQL injection vulnerability",
            },
            SecurityPattern {
                name: "hardcoded_password",
                pattern: r#"(?i)(password|passwd|pwd)\s*=\s*["'][^"']{3,}["']"#,
                severity: SeverityLevel::High,
                category: "secrets",
                description: "Hardcoded password detected",
            },
            SecurityPattern {
                name: "hardcoded_api_key",
                pattern: r#"(?i)(api_key|apikey|token)\s*=\s*["'][a-zA-Z0-9]{20,}["']"#,
                severity: SeverityLevel::High,
                category: "secrets",
                description: "Hardcoded API key detected",
            },
            SecurityPattern {
                name: "weak_crypto",
                pattern: r#"(?i)(md5|sha1|des|3des|rc4)"#,
                severity: SeverityLevel::Medium,
                category: "cryptography",
                description: "Weak cryptographic algorithm detected",
            },
            SecurityPattern {
                name: "xss_vulnerability",
                pattern: r#"(?i)innerHTML\s*=\s*[^;]*\+|document\.write\s*\([^)]*\+"#,
                severity: SeverityLevel::High,
                category: "xss",
                description: "Potential XSS vulnerability",
            },
            SecurityPattern {
                name: "path_traversal",
                pattern: r#"\.\.\/|\.\.\\|%2e%2e%2f|%2e%2e%5c"#,
                severity: SeverityLevel::High,
                category: "path_traversal",
                description: "Potential path traversal vulnerability",
            },
            SecurityPattern {
                name: "command_injection",
                pattern: r#"(?i)(exec|system|popen|subprocess)\s*\([^)]*\+[^)]*\)"#,
                severity: SeverityLevel::Critical,
                category: "injection",
                description: "Potential command injection vulnerability",
            },
            SecurityPattern {
                name: "insecure_random",
                pattern: r#"(?i)(Math\.random|Random\(\)|rand\(\))"#,
                severity: SeverityLevel::Low,
                category: "randomness",
                description: "Insecure random number generation",
            },
        ]
    }

    async fn load_vulnerability_database(&mut self) -> Result<(), String> {
        log(LogLevel::Info, "Loading vulnerability database...");
        
        // Use host function to check if we should update the database
        if let Some(last_update) = get_config("vuln_db_last_update") {
            let last_update: u64 = last_update.parse().map_err(|e| format!("Parse error: {}", e))?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Update every 24 hours
            if now - last_update < 86400 {
                log(LogLevel::Info, "Vulnerability database is up to date");
                return self.load_cached_database();
            }
        }

        // Simulate fetching from NVD or similar database
        match http_get("https://api.example.com/vulnerabilities", vec![
            ("User-Agent".to_string(), "Uveddi-Security-Scanner/1.0".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
        ]) {
            Ok(response) => {
                if response.status == 200 {
                    // In a real implementation, parse JSON and populate database
                    self.vulnerability_db.insert(
                        "CVE-2024-1234".to_string(),
                        VulnerabilityInfo {
                            cve_id: "CVE-2024-1234".to_string(),
                            severity: SeverityLevel::High,
                            description: "Buffer overflow in example library".to_string(),
                            fix_version: Some("2.1.1".to_string()),
                        }
                    );
                    
                    // Cache the update timestamp
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let _ = set_config("vuln_db_last_update", &now.to_string());
                    
                    log(LogLevel::Info, "Vulnerability database updated successfully");
                    Ok(())
                } else {
                    Err(format!("Failed to fetch vulnerability database: HTTP {}", response.status))
                }
            },
            Err(e) => {
                log(LogLevel::Warn, &format!("Network request failed, using cached data: {}", e));
                self.load_cached_database()
            }
        }
    }

    fn load_cached_database(&mut self) -> Result<(), String> {
        // Load from local cache using database host functions
        if let Some(cached_data) = db_get("vuln_db") {
            // In a real implementation, deserialize JSON data
            log(LogLevel::Info, "Loaded cached vulnerability database");
        }
        Ok(())
    }

    fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
        self.scan_count += 1;
        
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        log(LogLevel::Info, &format!("Security scanning {}", file.path));

        let mut issues = Vec::new();
        let mut found_secrets = HashSet::new();
        let mut vulnerability_count = 0;

        // Pattern-based analysis
        for pattern in &self.blocked_patterns {
            let pattern_issues = self.find_pattern_matches(pattern, &file);
            vulnerability_count += pattern_issues.len();
            issues.extend(pattern_issues);
        }

        // AST-based analysis for more sophisticated detection
        if let Some(ast) = &file.ast {
            let ast_issues = self.analyze_ast_security(ast, &file)?;
            issues.extend(ast_issues);
        }

        // Dependency vulnerability analysis
        if file.path.ends_with("Cargo.toml") || file.path.ends_with("package.json") || file.path.ends_with("requirements.txt") {
            let dep_issues = self.analyze_dependencies(&file)?;
            issues.extend(dep_issues);
        }

        // Calculate security metrics
        let security_score = self.calculate_security_score(&issues);
        let critical_vulnerabilities = issues.iter()
            .filter(|i| i.severity == SeverityLevel::Critical)
            .count() as f64;
        
        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: 0,
            complexity: 1,
            maintainability_index: 100.0 - (issues.len() as f64 * 5.0),
            technical_debt_minutes: self.calculate_security_debt(&issues),
            custom_metrics: vec![
                ("security_score".to_string(), security_score),
                ("critical_vulnerabilities".to_string(), critical_vulnerabilities),
                ("secrets_detected".to_string(), found_secrets.len() as f64),
                ("vulnerability_density".to_string(), vulnerability_count as f64 / file.content.lines().count() as f64),
            ],
        };

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        Ok(AnalysisResult {
            issues,
            metrics,
            dependencies: vec![], // Could extract from package managers
            exports: vec![],
            duration_ms: end_time - start_time,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn find_pattern_matches(&self, pattern: &SecurityPattern, file: &SourceFile) -> Vec<Issue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = file.content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num as u32 + 1;
            
            // Simple pattern matching (in a real implementation, use regex crate)
            if self.line_matches_pattern(line, pattern.pattern) {
                let issue_id = match pattern.name {
                    "sql_injection" => "SEC001",
                    "hardcoded_password" => "SEC002", 
                    "hardcoded_api_key" => "SEC003",
                    "weak_crypto" => "SEC004",
                    "xss_vulnerability" => "SEC005",
                    "path_traversal" => "SEC006",
                    "command_injection" => "SEC007",
                    "insecure_random" => "SEC008",
                    _ => "SEC999",
                };

                issues.push(Issue {
                    id: issue_id.to_string(),
                    severity: pattern.severity.clone(),
                    category: IssueCategory::Security,
                    message: pattern.description.to_string(),
                    description: Some(self.get_detailed_description(pattern.name)),
                    file: file.path.clone(),
                    span: Span {
                        start: Position { line: line_num, column: 1, byte_offset: 0 },
                        end: Position { line: line_num, column: line.len() as u32, byte_offset: 0 },
                    },
                    rule_id: Some(pattern.name.to_string()),
                    suggestion: Some(self.get_fix_suggestion(pattern.name)),
                    fix: None,
                    metadata: vec![
                        ("category".to_string(), pattern.category.to_string()),
                        ("pattern".to_string(), pattern.pattern.to_string()),
                    ],
                });
            }
        }

        issues
    }

    fn line_matches_pattern(&self, line: &str, pattern: &str) -> bool {
        // Simplified pattern matching - in real implementation use proper regex
        match pattern {
            r#"(?i)(SELECT|INSERT|UPDATE|DELETE).*\+.*["']"# => {
                let line_lower = line.to_lowercase();
                (line_lower.contains("select") || line_lower.contains("insert") || 
                 line_lower.contains("update") || line_lower.contains("delete")) &&
                line.contains("+") && (line.contains("\"") || line.contains("'"))
            },
            r#"(?i)(password|passwd|pwd)\s*=\s*["'][^"']{3,}["']"# => {
                let line_lower = line.to_lowercase();
                (line_lower.contains("password") || line_lower.contains("passwd") || line_lower.contains("pwd")) &&
                line.contains("=") && (line.contains("\"") || line.contains("'"))
            },
            r#"(?i)(api_key|apikey|token)\s*=\s*["'][a-zA-Z0-9]{20,}["']"# => {
                let line_lower = line.to_lowercase();
                (line_lower.contains("api_key") || line_lower.contains("apikey") || line_lower.contains("token")) &&
                line.contains("=")
            },
            _ => false,
        }
    }

    fn analyze_ast_security(&self, ast: &AstNode, file: &SourceFile) -> Result<Vec<Issue>, String> {
        let mut issues = Vec::new();

        // Look for dangerous function calls
        if let Ok(dangerous_calls) = query_ast(ast.clone(), "(call_expression (identifier) @func (#match? @func \"^(eval|exec|system)$\"))") {
            for node in dangerous_calls {
                issues.push(Issue {
                    id: "SEC009".to_string(),
                    severity: SeverityLevel::Critical,
                    category: IssueCategory::Security,
                    message: "Dangerous function call detected".to_string(),
                    description: Some("Functions like eval(), exec(), and system() can execute arbitrary code and are security risks.".to_string()),
                    file: file.path.clone(),
                    span: node.span,
                    rule_id: Some("dangerous_function".to_string()),
                    suggestion: Some("Avoid using eval(), exec(), and system() functions. Use safer alternatives.".to_string()),
                    fix: None,
                    metadata: vec![
                        ("function".to_string(), node.content),
                        ("category".to_string(), "code_injection".to_string()),
                    ],
                });
            }
        }

        Ok(issues)
    }

    fn analyze_dependencies(&self, file: &SourceFile) -> Result<Vec<Issue>, String> {
        let mut issues = Vec::new();

        // Check for known vulnerable dependencies
        if file.path.ends_with("package.json") && file.content.contains("\"lodash\": \"4.17.0\"") {
            issues.push(Issue {
                id: "SEC010".to_string(),
                severity: SeverityLevel::High,
                category: IssueCategory::Security,
                message: "Vulnerable dependency detected: lodash 4.17.0".to_string(),
                description: Some("This version of lodash contains known security vulnerabilities. Update to version 4.17.21 or later.".to_string()),
                file: file.path.clone(),
                span: Span {
                    start: Position { line: 1, column: 1, byte_offset: 0 },
                    end: Position { line: 1, column: 1, byte_offset: 0 },
                },
                rule_id: Some("vulnerable_dependency".to_string()),
                suggestion: Some("Update lodash to version 4.17.21 or later".to_string()),
                fix: Some(CodeFix {
                    description: "Update lodash version".to_string(),
                    replacements: vec![
                        Replacement {
                            span: Span {
                                start: Position { line: 1, column: 1, byte_offset: 0 },
                                end: Position { line: 1, column: 1, byte_offset: 0 },
                            },
                            new_text: "\"lodash\": \"^4.17.21\"".to_string(),
                        }
                    ],
                }),
                metadata: vec![
                    ("cve_id".to_string(), "CVE-2021-23337".to_string()),
                    ("vulnerable_version".to_string(), "4.17.0".to_string()),
                    ("fix_version".to_string(), "4.17.21".to_string()),
                ],
            });
        }

        Ok(issues)
    }

    fn get_detailed_description(&self, pattern_name: &str) -> String {
        match pattern_name {
            "sql_injection" => "SQL injection occurs when user input is directly concatenated into SQL queries without proper sanitization. This can allow attackers to execute arbitrary SQL commands.".to_string(),
            "hardcoded_password" => "Hardcoded passwords in source code are a major security risk as they can be easily discovered by anyone with access to the code.".to_string(),
            "hardcoded_api_key" => "Hardcoded API keys should never be stored in source code. Use environment variables or secure configuration management instead.".to_string(),
            "weak_crypto" => "Weak cryptographic algorithms like MD5 and SHA1 are vulnerable to collision attacks and should not be used for security purposes.".to_string(),
            _ => "Security vulnerability detected".to_string(),
        }
    }

    fn get_fix_suggestion(&self, pattern_name: &str) -> String {
        match pattern_name {
            "sql_injection" => "Use parameterized queries or prepared statements instead of string concatenation".to_string(),
            "hardcoded_password" => "Store passwords in environment variables or secure configuration files".to_string(),
            "hardcoded_api_key" => "Use environment variables or a secrets management system".to_string(),
            "weak_crypto" => "Use modern cryptographic algorithms like SHA-256 or bcrypt".to_string(),
            _ => "Review code for security implications".to_string(),
        }
    }

    fn calculate_security_score(&self, issues: &[Issue]) -> f64 {
        if issues.is_empty() {
            return 100.0;
        }

        let penalty: f64 = issues.iter().map(|issue| {
            match issue.severity {
                SeverityLevel::Critical => 40.0,
                SeverityLevel::High => 25.0,
                SeverityLevel::Medium => 15.0,
                SeverityLevel::Low => 5.0,
                SeverityLevel::Info => 1.0,
            }
        }).sum();

        (100.0 - penalty).max(0.0)
    }

    fn calculate_security_debt(&self, issues: &[Issue]) -> u32 {
        issues.iter().map(|issue| {
            match issue.severity {
                SeverityLevel::Critical => 240, // 4 hours
                SeverityLevel::High => 120,     // 2 hours
                SeverityLevel::Medium => 60,    // 1 hour
                SeverityLevel::Low => 30,       // 30 minutes
                SeverityLevel::Info => 15,      // 15 minutes
            }
        }).sum()
    }
}

// Plugin lifecycle implementation
impl initialize {
    fn call(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Security Scanner Plugin");
        
        unsafe {
            let mut scanner = SecurityScanner::new();
            scanner.config = config;
            PLUGIN_STATE = Some(scanner);
        }
        
        // Load vulnerability database in background
        // In a real async implementation, this would be awaited
        log(LogLevel::Info, "Security Scanner Plugin initialized successfully");
        Ok(())
    }
}

impl analyze {
    fn call(file: SourceFile) -> Result<AnalysisResult, String> {
        unsafe {
            if let Some(ref mut state) = PLUGIN_STATE {
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
            id: "security-scanner".to_string(),
            name: "Security Vulnerability Scanner".to_string(),
            version: "1.0.0".to_string(),
            description: "Comprehensive security vulnerability scanner for detecting common security issues and compliance violations".to_string(),
            author: "Uveddi Security Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://github.com/uveddi/plugins/security-scanner".to_string()),
            supported_languages: vec![
                "rust".to_string(),
                "javascript".to_string(), 
                "typescript".to_string(),
                "python".to_string(),
                "java".to_string(),
                "go".to_string(),
                "php".to_string(),
                "ruby".to_string(),
            ],
            detector_types: vec![IssueCategory::Security],
            api_version: "1.0".to_string(),
            required_permissions: vec![
                Permission::ReadFiles.into(),
                Permission::NetworkAccess.into(), // For vulnerability database updates
                Permission::SystemInfo.into(),    // For environment analysis
            ],
        }
    }
}

impl cleanup {
    fn call() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up Security Scanner Plugin");
        
        unsafe {
            if let Some(ref state) = PLUGIN_STATE {
                log(LogLevel::Info, &format!("Scanned {} files for security issues during session", state.scan_count));
                
                // Cache vulnerability database for next session
                let _ = db_set("vuln_db_cache", "cached_data", Some(3600)); // 1 hour TTL
            }
            PLUGIN_STATE = None;
        }
        
        Ok(())
    }
}