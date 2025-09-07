//! Performance Analyzer Plugin
//! 
//! This plugin analyzes code for performance anti-patterns and provides
//! optimization suggestions. It demonstrates host function usage for AST
//! parsing and complex analysis algorithms.

use std::collections::HashMap;

// Import the WIT bindings
wit_bindgen::generate!({
    world: "core-analysis",
    path: "../../../wit/core-analysis.wit",
});

use exports::initialize;
use exports::analyze;
use exports::get_info;
use exports::cleanup;

/// Plugin state
static mut PLUGIN_STATE: Option<PerformanceAnalyzer> = None;

struct PerformanceAnalyzer {
    config: PluginConfig,
    analysis_count: u32,
    performance_patterns: Vec<PerformancePattern>,
}

struct PerformancePattern {
    name: &'static str,
    description: &'static str,
    severity: SeverityLevel,
    pattern_type: &'static str,
}

impl PerformanceAnalyzer {
    fn new() -> Self {
        Self {
            config: PluginConfig {
                severity_threshold: SeverityLevel::Medium,
                max_issues_per_file: 50,
                include_patterns: vec!["*.rs".to_string(), "*.js".to_string(), "*.py".to_string()],
                exclude_patterns: vec!["*.test.*".to_string()],
                rule_overrides: vec![],
                custom_settings: vec![
                    ("max_loop_depth".to_string(), "5".to_string()),
                    ("complexity_threshold".to_string(), "15".to_string()),
                ],
            },
            analysis_count: 0,
            performance_patterns: Self::initialize_patterns(),
        }
    }

    fn initialize_patterns() -> Vec<PerformancePattern> {
        vec![
            PerformancePattern {
                name: "nested_loops",
                description: "Deeply nested loops can cause performance issues",
                severity: SeverityLevel::High,
                pattern_type: "complexity",
            },
            PerformancePattern {
                name: "string_concatenation_loop",
                description: "String concatenation in loops should use StringBuilder/Vec",
                severity: SeverityLevel::Medium,
                pattern_type: "memory",
            },
            PerformancePattern {
                name: "synchronous_file_io",
                description: "Synchronous file I/O can block execution",
                severity: SeverityLevel::Medium,
                pattern_type: "blocking",
            },
            PerformancePattern {
                name: "unoptimized_regex",
                description: "Regex compilation in hot paths is expensive",
                severity: SeverityLevel::Low,
                pattern_type: "cpu",
            },
            PerformancePattern {
                name: "memory_allocation_loop",
                description: "Memory allocations inside loops should be avoided",
                severity: SeverityLevel::High,
                pattern_type: "memory",
            },
        ]
    }

    fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
        self.analysis_count += 1;
        
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        log(LogLevel::Info, &format!("Analyzing {} for performance issues", file.path));

        let mut issues = Vec::new();
        let mut metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: 0,
            complexity: 1,
            maintainability_index: 100.0,
            technical_debt_minutes: 0,
            custom_metrics: vec![
                ("performance_score".to_string(), 0.0),
                ("hotspot_count".to_string(), 0.0),
            ],
        };

        // Parse AST using host function
        if let Some(ast) = file.ast {
            match self.analyze_ast(&ast, &file) {
                Ok(ast_issues) => {
                    issues.extend(ast_issues);
                },
                Err(e) => {
                    log(LogLevel::Warn, &format!("AST analysis failed: {}", e));
                }
            }
        } else {
            // Parse AST using host function if not provided
            match parse_ast(&file.content, &file.language) {
                Ok(ast) => {
                    match self.analyze_ast(&ast, &file) {
                        Ok(ast_issues) => {
                            issues.extend(ast_issues);
                        },
                        Err(e) => {
                            log(LogLevel::Warn, &format!("AST analysis failed: {}", e));
                        }
                    }
                },
                Err(e) => {
                    log(LogLevel::Error, &format!("Failed to parse AST: {}", e));
                }
            }
        }

        // Analyze patterns in source text
        let text_issues = self.analyze_text_patterns(&file);
        issues.extend(text_issues);

        // Calculate performance metrics
        metrics.complexity = self.calculate_complexity(&issues);
        metrics.technical_debt_minutes = self.calculate_tech_debt(&issues);
        
        let performance_score = self.calculate_performance_score(&issues);
        let hotspot_count = issues.iter().filter(|i| i.severity == SeverityLevel::Critical || i.severity == SeverityLevel::High).count() as f64;
        
        metrics.custom_metrics = vec![
            ("performance_score".to_string(), performance_score),
            ("hotspot_count".to_string(), hotspot_count),
        ];

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        Ok(AnalysisResult {
            issues,
            metrics,
            dependencies: vec![], // Could extract import/require statements
            exports: vec![], // Could extract export statements
            duration_ms: end_time - start_time,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn analyze_ast(&self, ast: &AstNode, file: &SourceFile) -> Result<Vec<Issue>, String> {
        let mut issues = Vec::new();

        // Analyze nested loops
        if let Ok(nested_nodes) = query_ast(ast.clone(), "(for_statement (for_statement))") {
            for node in nested_nodes {
                issues.push(Issue {
                    id: "PERF001".to_string(),
                    severity: SeverityLevel::High,
                    category: IssueCategory::Performance,
                    message: "Nested loops detected - consider optimization".to_string(),
                    description: Some("Deeply nested loops can cause O(n²) or worse performance. Consider using more efficient algorithms or data structures.".to_string()),
                    file: file.path.clone(),
                    span: node.span,
                    rule_id: Some("nested_loops".to_string()),
                    suggestion: Some("Consider using iterators, hash maps, or breaking the loop into separate functions".to_string()),
                    fix: None, // Could provide automatic fixes
                    metadata: vec![
                        ("pattern_type".to_string(), "complexity".to_string()),
                        ("loop_depth".to_string(), "2+".to_string()),
                    ],
                });
            }
        }

        // Analyze memory allocations in loops
        if let Ok(alloc_nodes) = query_ast(ast.clone(), "(for_statement (call_expression (identifier) @name (#match? @name \"^(Vec::new|HashMap::new|String::new)$\")))") {
            for node in alloc_nodes {
                issues.push(Issue {
                    id: "PERF002".to_string(),
                    severity: SeverityLevel::Medium,
                    category: IssueCategory::Performance,
                    message: "Memory allocation inside loop".to_string(),
                    description: Some("Allocating memory inside loops can cause performance degradation. Consider pre-allocating or reusing objects.".to_string()),
                    file: file.path.clone(),
                    span: node.span,
                    rule_id: Some("memory_allocation_loop".to_string()),
                    suggestion: Some("Pre-allocate containers before the loop or reuse existing ones".to_string()),
                    fix: None,
                    metadata: vec![
                        ("pattern_type".to_string(), "memory".to_string()),
                        ("allocation_type".to_string(), node.content),
                    ],
                });
            }
        }

        Ok(issues)
    }

    fn analyze_text_patterns(&self, file: &SourceFile) -> Vec<Issue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = file.content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num as u32 + 1;

            // Check for string concatenation in loops
            if line.contains("for ") && line.contains("+") && (line.contains("String") || line.contains("&str")) {
                issues.push(Issue {
                    id: "PERF003".to_string(),
                    severity: SeverityLevel::Medium,
                    category: IssueCategory::Performance,
                    message: "Potential string concatenation in loop".to_string(),
                    description: Some("String concatenation in loops creates many temporary objects. Use format!, String::with_capacity(), or Vec<String> for better performance.".to_string()),
                    file: file.path.clone(),
                    span: Span {
                        start: Position { line: line_num, column: 1, byte_offset: 0 },
                        end: Position { line: line_num, column: line.len() as u32, byte_offset: 0 },
                    },
                    rule_id: Some("string_concatenation_loop".to_string()),
                    suggestion: Some("Use format!() macro or String::with_capacity()".to_string()),
                    fix: None,
                    metadata: vec![
                        ("pattern_type".to_string(), "memory".to_string()),
                        ("line_content".to_string(), line.to_string()),
                    ],
                });
            }

            // Check for synchronous file I/O
            if line.contains("std::fs::read") || line.contains("fs.readFileSync") || line.contains("open(") {
                issues.push(Issue {
                    id: "PERF004".to_string(),
                    severity: SeverityLevel::Low,
                    category: IssueCategory::Performance,
                    message: "Synchronous file I/O detected".to_string(),
                    description: Some("Synchronous file operations block the thread. Consider using async alternatives for better performance.".to_string()),
                    file: file.path.clone(),
                    span: Span {
                        start: Position { line: line_num, column: 1, byte_offset: 0 },
                        end: Position { line: line_num, column: line.len() as u32, byte_offset: 0 },
                    },
                    rule_id: Some("synchronous_file_io".to_string()),
                    suggestion: Some("Use tokio::fs or async file operations".to_string()),
                    fix: None,
                    metadata: vec![
                        ("pattern_type".to_string(), "blocking".to_string()),
                    ],
                });
            }
        }

        issues
    }

    fn calculate_complexity(&self, issues: &[Issue]) -> u32 {
        let base_complexity = 1;
        let complexity_boost = issues.iter()
            .filter(|i| i.rule_id.as_ref().map_or(false, |r| r == "nested_loops"))
            .count() as u32 * 3;
        
        base_complexity + complexity_boost
    }

    fn calculate_tech_debt(&self, issues: &[Issue]) -> u32 {
        issues.iter().map(|issue| {
            match issue.severity {
                SeverityLevel::Critical => 60, // 1 hour
                SeverityLevel::High => 30,     // 30 minutes
                SeverityLevel::Medium => 15,   // 15 minutes
                SeverityLevel::Low => 5,       // 5 minutes
                SeverityLevel::Info => 2,      // 2 minutes
            }
        }).sum()
    }

    fn calculate_performance_score(&self, issues: &[Issue]) -> f64 {
        if issues.is_empty() {
            return 100.0;
        }

        let penalty: f64 = issues.iter().map(|issue| {
            match issue.severity {
                SeverityLevel::Critical => 25.0,
                SeverityLevel::High => 15.0,
                SeverityLevel::Medium => 10.0,
                SeverityLevel::Low => 5.0,
                SeverityLevel::Info => 1.0,
            }
        }).sum();

        (100.0 - penalty).max(0.0)
    }
}

// Plugin lifecycle implementation
impl initialize {
    fn call(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Performance Analyzer Plugin");
        
        unsafe {
            PLUGIN_STATE = Some(PerformanceAnalyzer::new());
            if let Some(ref mut state) = PLUGIN_STATE {
                state.config = config;
            }
        }
        
        log(LogLevel::Info, "Performance Analyzer Plugin initialized successfully");
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
            id: "performance-analyzer".to_string(),
            name: "Performance Analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyzes code for performance anti-patterns and provides optimization suggestions".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://github.com/uveddi/plugins/performance-analyzer".to_string()),
            supported_languages: vec![
                "rust".to_string(), 
                "javascript".to_string(), 
                "typescript".to_string(),
                "python".to_string()
            ],
            detector_types: vec![IssueCategory::Performance, IssueCategory::Complexity],
            api_version: "1.0".to_string(),
            required_permissions: vec![Permission::ReadFiles.into()], // Convert to flags
        }
    }
}

impl cleanup {
    fn call() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up Performance Analyzer Plugin");
        
        unsafe {
            if let Some(ref state) = PLUGIN_STATE {
                log(LogLevel::Info, &format!("Analyzed {} files during session", state.analysis_count));
            }
            PLUGIN_STATE = None;
        }
        
        Ok(())
    }
}