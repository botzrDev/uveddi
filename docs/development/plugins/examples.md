# Plugin Examples

This document provides comprehensive examples for developing different types of Uveddi plugins using the WebAssembly Component Model. Each example includes complete source code, configuration, and testing instructions.

## Table of Contents

1. [Basic Detector Plugin](#basic-detector-plugin)
2. [Security Scanner Plugin](#security-scanner-plugin)
3. [Performance Analyzer Plugin](#performance-analyzer-plugin)
4. [Custom Rule Engine Plugin](#custom-rule-engine-plugin)
5. [Multi-Language Plugin](#multi-language-plugin)
6. [Testing Strategies](#testing-strategies)

## Basic Detector Plugin

This example shows a simple plugin that detects TODO comments in code using the WIT Component Model.

### Plugin Structure

```
todo-detector/
├── Cargo.toml
├── plugin.toml
├── wit/
│   └── core-analysis.wit
├── src/
│   └── lib.rs
└── test-data/
    └── sample.rs
```

### Cargo.toml

```toml
[package]
name = "todo-detector-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.16"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

### plugin.toml

```toml
[plugin]
name = "todo-detector"
version = "1.0.0"
description = "Detects TODO, FIXME, and HACK comments in code"
author = "Example Developer <dev@example.com>"
license = "MIT"

[capabilities]
permissions = ["ReadFiles", "Logging"]

[capabilities.limits]
max_memory_mb = 16
max_execution_seconds = 10

[dependencies]
min_uveddi_version = "0.9.0"
supported_languages = ["rust", "python", "javascript", "typescript", "java", "go"]

[detection]
issue_categories = ["documentation", "maintainability"]
severity_levels = ["info", "low", "medium"]
```

### src/lib.rs

```rust
//! TODO Detector Plugin
//!
//! Detects TODO, FIXME, HACK, and BUG comments in source code.

wit_bindgen::generate!({
    world: "core-analysis",
    path: "wit/core-analysis.wit",
});

use std::cell::RefCell;

struct TodoDetectorPlugin {
    state: RefCell<Option<PluginState>>,
}

struct PluginState {
    config: PluginConfig,
    patterns: Vec<CommentPattern>,
    analysis_count: u32,
}

struct CommentPattern {
    keyword: &'static str,
    severity: SeverityLevel,
    message: &'static str,
}

impl Default for TodoDetectorPlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

export!(TodoDetectorPlugin);

impl Guest for TodoDetectorPlugin {
    fn initialize(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing TODO Detector Plugin");

        let patterns = vec![
            CommentPattern {
                keyword: "TODO",
                severity: SeverityLevel::Info,
                message: "TODO comment found - consider creating an issue",
            },
            CommentPattern {
                keyword: "FIXME",
                severity: SeverityLevel::Medium,
                message: "FIXME comment found - indicates code needing fixes",
            },
            CommentPattern {
                keyword: "HACK",
                severity: SeverityLevel::Medium,
                message: "HACK comment found - indicates non-standard solution",
            },
            CommentPattern {
                keyword: "BUG",
                severity: SeverityLevel::High,
                message: "BUG comment found - indicates known issue",
            },
        ];

        let state = PluginState {
            config,
            patterns,
            analysis_count: 0,
        };

        *PLUGIN_STATE.borrow_mut() = Some(state);

        log(LogLevel::Info, "TODO Detector Plugin initialized");
        Ok(())
    }

    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        log(LogLevel::Debug, &format!("Analyzing {}", file.path));

        let mut issues = Vec::new();

        // Get patterns from state
        let patterns = PLUGIN_STATE.borrow();
        let state = patterns.as_ref().ok_or("Plugin not initialized")?;

        for (line_num, line) in file.content.lines().enumerate() {
            let line_upper = line.to_uppercase();

            for pattern in &state.patterns {
                if let Some(col) = line_upper.find(pattern.keyword) {
                    // Verify it's in a comment
                    let before = &line[..col];
                    let is_comment = before.contains("//")
                        || before.contains("#")
                        || before.contains("/*")
                        || before.contains("*");

                    if is_comment || line.trim().starts_with("//") || line.trim().starts_with("#") {
                        issues.push(Issue {
                            id: format!("{}-{}-{}", pattern.keyword, line_num + 1, col),
                            severity: pattern.severity.clone(),
                            category: IssueCategory::Documentation,
                            message: pattern.message.to_string(),
                            description: Some(format!(
                                "Found {} comment at line {}",
                                pattern.keyword, line_num + 1
                            )),
                            file: file.path.clone(),
                            span: Span {
                                start: Position {
                                    line: line_num as u32 + 1,
                                    column: col as u32,
                                    byte_offset: 0,
                                },
                                end: Position {
                                    line: line_num as u32 + 1,
                                    column: (col + pattern.keyword.len()) as u32,
                                    byte_offset: 0,
                                },
                            },
                            rule_id: Some(format!("{}-comment", pattern.keyword.to_lowercase())),
                            suggestion: Some("Consider creating an issue tracker entry".to_string()),
                            fix: None,
                            metadata: vec![
                                ("keyword".to_string(), pattern.keyword.to_string()),
                                ("context".to_string(), line.trim().to_string()),
                            ],
                        });
                    }
                }
            }
        }

        // Calculate metrics
        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: count_comment_lines(&file.content),
            complexity: 1,
            maintainability_index: 100.0 - (issues.len() as f64 * 2.0),
            technical_debt_minutes: issues.len() as u32 * 5,
            custom_metrics: vec![
                ("todo_count".to_string(), issues.iter().filter(|i| i.id.starts_with("TODO")).count() as f64),
                ("fixme_count".to_string(), issues.iter().filter(|i| i.id.starts_with("FIXME")).count() as f64),
            ],
        };

        log(LogLevel::Info, &format!("Found {} issues in {}", issues.len(), file.path));

        Ok(AnalysisResult {
            issues,
            metrics,
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn get_info() -> PluginInfo {
        PluginInfo {
            id: "todo-detector".to_string(),
            name: "TODO Comment Detector".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects TODO, FIXME, HACK, and BUG comments".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            supported_languages: vec![
                "rust".to_string(), "python".to_string(),
                "javascript".to_string(), "typescript".to_string(),
                "java".to_string(), "go".to_string(),
            ],
            detector_types: vec![IssueCategory::Documentation, IssueCategory::Maintainability],
            api_version: "1.0".to_string(),
            required_permissions: vec![],
        }
    }

    fn cleanup() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up TODO Detector Plugin");
        *PLUGIN_STATE.borrow_mut() = None;
        Ok(())
    }
}

thread_local! {
    static PLUGIN_STATE: RefCell<Option<PluginState>> = RefCell::new(None);
}

fn count_comment_lines(content: &str) -> u32 {
    content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("#") ||
            trimmed.starts_with("/*") || trimmed.starts_with("*")
        })
        .count() as u32
}
```

### Build and Test

```bash
# Build the plugin
cargo build --release --target wasm32-wasi

# Install and test
uveddi plugin install target/wasm32-wasi/release/todo_detector_plugin.wasm
uveddi plugin test todo-detector --test-file sample.rs
```

## Security Scanner Plugin

A comprehensive security scanner that detects various security vulnerabilities.

### src/lib.rs

```rust
//! Security Scanner Plugin
//!
//! Detects security vulnerabilities including SQL injection, XSS, and hardcoded secrets.

wit_bindgen::generate!({
    world: "core-analysis",
    path: "wit/core-analysis.wit",
});

use std::cell::RefCell;

struct SecurityScannerPlugin {
    state: RefCell<Option<ScannerState>>,
}

struct ScannerState {
    config: PluginConfig,
    rules: Vec<SecurityRule>,
}

struct SecurityRule {
    id: &'static str,
    name: &'static str,
    pattern: &'static str,
    severity: SeverityLevel,
    message: &'static str,
    suggestion: &'static str,
    cwe_id: Option<&'static str>,
}

impl Default for SecurityScannerPlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

export!(SecurityScannerPlugin);

impl Guest for SecurityScannerPlugin {
    fn initialize(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Security Scanner Plugin");

        let rules = vec![
            SecurityRule {
                id: "SEC001",
                name: "Potential SQL Injection",
                pattern: "query|execute|exec",
                severity: SeverityLevel::High,
                message: "Potential SQL injection vulnerability",
                suggestion: "Use parameterized queries or prepared statements",
                cwe_id: Some("CWE-89"),
            },
            SecurityRule {
                id: "SEC002",
                name: "Hardcoded Secret",
                pattern: "password|secret|api_key|token",
                severity: SeverityLevel::Critical,
                message: "Hardcoded secret detected",
                suggestion: "Use environment variables or a secret management system",
                cwe_id: Some("CWE-798"),
            },
            SecurityRule {
                id: "SEC003",
                name: "Potential XSS",
                pattern: "innerHTML|outerHTML|document.write",
                severity: SeverityLevel::High,
                message: "Potential XSS vulnerability",
                suggestion: "Use textContent or properly sanitize input",
                cwe_id: Some("CWE-79"),
            },
            SecurityRule {
                id: "SEC004",
                name: "Unsafe Deserialization",
                pattern: "pickle.loads|yaml.load|unserialize",
                severity: SeverityLevel::High,
                message: "Unsafe deserialization detected",
                suggestion: "Use safe loading methods like yaml.safe_load",
                cwe_id: Some("CWE-502"),
            },
            SecurityRule {
                id: "SEC005",
                name: "Command Injection",
                pattern: "system\\(|exec\\(|eval\\(|subprocess.call",
                severity: SeverityLevel::Critical,
                message: "Potential command injection",
                suggestion: "Avoid dynamic command execution; use safe APIs",
                cwe_id: Some("CWE-78"),
            },
        ];

        let state = ScannerState { config, rules };
        *SCANNER_STATE.borrow_mut() = Some(state);

        log(LogLevel::Info, "Security Scanner initialized with {} rules", );
        Ok(())
    }

    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        log(LogLevel::Info, &format!("Scanning {} for security issues", file.path));

        let mut issues = Vec::new();
        let state_ref = SCANNER_STATE.borrow();
        let state = state_ref.as_ref().ok_or("Plugin not initialized")?;

        for (line_num, line) in file.content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            for rule in &state.rules {
                // Simple pattern matching (in production, use proper regex)
                let patterns: Vec<&str> = rule.pattern.split('|').collect();

                for pattern in patterns {
                    if line_lower.contains(pattern) {
                        // Additional context checks
                        let is_likely_issue = match rule.id {
                            "SEC002" => {
                                // Check for assignment with string literal
                                line.contains("=") && (line.contains("\"") || line.contains("'"))
                            }
                            _ => true,
                        };

                        if is_likely_issue {
                            issues.push(Issue {
                                id: format!("{}-{}", rule.id, line_num + 1),
                                severity: rule.severity.clone(),
                                category: IssueCategory::Security,
                                message: rule.message.to_string(),
                                description: Some(format!(
                                    "{}: {}",
                                    rule.name,
                                    rule.cwe_id.unwrap_or("No CWE")
                                )),
                                file: file.path.clone(),
                                span: Span {
                                    start: Position {
                                        line: line_num as u32 + 1,
                                        column: 0,
                                        byte_offset: 0,
                                    },
                                    end: Position {
                                        line: line_num as u32 + 1,
                                        column: line.len() as u32,
                                        byte_offset: 0,
                                    },
                                },
                                rule_id: Some(rule.id.to_string()),
                                suggestion: Some(rule.suggestion.to_string()),
                                fix: None,
                                metadata: vec![
                                    ("cwe".to_string(), rule.cwe_id.unwrap_or("unknown").to_string()),
                                    ("pattern".to_string(), pattern.to_string()),
                                ],
                            });
                            break; // Only report once per line per rule
                        }
                    }
                }
            }
        }

        // Calculate security score
        let critical_count = issues.iter()
            .filter(|i| matches!(i.severity, SeverityLevel::Critical))
            .count();
        let high_count = issues.iter()
            .filter(|i| matches!(i.severity, SeverityLevel::High))
            .count();

        let security_score = 100.0 - (critical_count as f64 * 20.0) - (high_count as f64 * 10.0);

        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: 0,
            complexity: 0,
            maintainability_index: security_score.max(0.0),
            technical_debt_minutes: (critical_count * 60 + high_count * 30) as u32,
            custom_metrics: vec![
                ("security_score".to_string(), security_score.max(0.0)),
                ("critical_issues".to_string(), critical_count as f64),
                ("high_issues".to_string(), high_count as f64),
            ],
        };

        log(LogLevel::Info, &format!(
            "Found {} security issues (Score: {:.1})",
            issues.len(), security_score
        ));

        Ok(AnalysisResult {
            issues,
            metrics,
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn get_info() -> PluginInfo {
        PluginInfo {
            id: "security-scanner".to_string(),
            name: "Security Scanner".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects security vulnerabilities and provides remediation guidance".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            supported_languages: vec![
                "rust".to_string(), "python".to_string(),
                "javascript".to_string(), "typescript".to_string(),
                "java".to_string(), "go".to_string(), "php".to_string(),
            ],
            detector_types: vec![IssueCategory::Security],
            api_version: "1.0".to_string(),
            required_permissions: vec![Permission::ReadFiles],
        }
    }

    fn cleanup() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up Security Scanner");
        *SCANNER_STATE.borrow_mut() = None;
        Ok(())
    }
}

thread_local! {
    static SCANNER_STATE: RefCell<Option<ScannerState>> = RefCell::new(None);
}
```

## Performance Analyzer Plugin

Analyzes code for performance bottlenecks and optimization opportunities.

### src/lib.rs

```rust
//! Performance Analyzer Plugin
//!
//! Detects performance issues like nested loops, inefficient algorithms, and memory issues.

wit_bindgen::generate!({
    world: "core-analysis",
    path: "wit/core-analysis.wit",
});

use std::cell::RefCell;

struct PerformanceAnalyzerPlugin {
    state: RefCell<Option<AnalyzerState>>,
}

struct AnalyzerState {
    config: PluginConfig,
    complexity_threshold: u32,
}

impl Default for PerformanceAnalyzerPlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

export!(PerformanceAnalyzerPlugin);

impl Guest for PerformanceAnalyzerPlugin {
    fn initialize(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Performance Analyzer Plugin");

        // Get custom threshold from config
        let complexity_threshold = config.custom_settings.iter()
            .find(|(k, _)| k == "complexity_threshold")
            .and_then(|(_, v)| v.parse().ok())
            .unwrap_or(10);

        let state = AnalyzerState {
            config,
            complexity_threshold,
        };

        *ANALYZER_STATE.borrow_mut() = Some(state);

        log(LogLevel::Info, &format!(
            "Performance Analyzer initialized (threshold: {})",
            complexity_threshold
        ));
        Ok(())
    }

    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        log(LogLevel::Debug, &format!("Analyzing performance of {}", file.path));

        let state_ref = ANALYZER_STATE.borrow();
        let state = state_ref.as_ref().ok_or("Plugin not initialized")?;

        let mut issues = Vec::new();
        let mut loop_nesting = 0;
        let mut max_nesting = 0;
        let mut cyclomatic_complexity = 1;
        let mut function_count = 0;

        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();

            // Track loop nesting
            if trimmed.contains("for ") || trimmed.contains("while ") {
                loop_nesting += 1;
                cyclomatic_complexity += 1;
                max_nesting = max_nesting.max(loop_nesting);

                if loop_nesting > 2 {
                    issues.push(Issue {
                        id: format!("PERF001-{}", line_num + 1),
                        severity: SeverityLevel::Medium,
                        category: IssueCategory::Performance,
                        message: format!("Deeply nested loop (depth: {})", loop_nesting),
                        description: Some("Deep loop nesting can lead to O(n^k) complexity".to_string()),
                        file: file.path.clone(),
                        span: create_span(line_num as u32 + 1, 0, line.len() as u32),
                        rule_id: Some("nested-loops".to_string()),
                        suggestion: Some("Consider refactoring to reduce nesting depth".to_string()),
                        fix: None,
                        metadata: vec![("nesting_depth".to_string(), loop_nesting.to_string())],
                    });
                }
            }

            // Track loop end
            if trimmed == "}" && loop_nesting > 0 {
                loop_nesting = loop_nesting.saturating_sub(1);
            }

            // Track functions
            if trimmed.contains("fn ") || trimmed.contains("function ") || trimmed.contains("def ") {
                function_count += 1;
            }

            // Track complexity
            if trimmed.contains("if ") || trimmed.contains("match ") || trimmed.contains("case ") {
                cyclomatic_complexity += 1;
            }

            // Detect inefficient patterns
            if (trimmed.contains("+ \"") || trimmed.contains("+ '")) && loop_nesting > 0 {
                issues.push(Issue {
                    id: format!("PERF002-{}", line_num + 1),
                    severity: SeverityLevel::Medium,
                    category: IssueCategory::Performance,
                    message: "String concatenation in loop".to_string(),
                    description: Some("String concatenation in loops is inefficient".to_string()),
                    file: file.path.clone(),
                    span: create_span(line_num as u32 + 1, 0, line.len() as u32),
                    rule_id: Some("string-concat-loop".to_string()),
                    suggestion: Some("Use StringBuilder, Vec, or join()".to_string()),
                    fix: None,
                    metadata: vec![],
                });
            }
        }

        // Check overall complexity
        if cyclomatic_complexity > state.complexity_threshold {
            issues.push(Issue {
                id: "PERF003".to_string(),
                severity: SeverityLevel::Medium,
                category: IssueCategory::Complexity,
                message: format!("High cyclomatic complexity: {}", cyclomatic_complexity),
                description: Some(format!(
                    "Complexity {} exceeds threshold {}",
                    cyclomatic_complexity, state.complexity_threshold
                )),
                file: file.path.clone(),
                span: create_span(1, 0, 0),
                rule_id: Some("high-complexity".to_string()),
                suggestion: Some("Consider breaking into smaller functions".to_string()),
                fix: None,
                metadata: vec![("complexity".to_string(), cyclomatic_complexity.to_string())],
            });
        }

        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: 0,
            complexity: cyclomatic_complexity,
            maintainability_index: calculate_maintainability(cyclomatic_complexity, file.content.lines().count()),
            technical_debt_minutes: issues.len() as u32 * 15,
            custom_metrics: vec![
                ("cyclomatic_complexity".to_string(), cyclomatic_complexity as f64),
                ("max_nesting_depth".to_string(), max_nesting as f64),
                ("function_count".to_string(), function_count as f64),
            ],
        };

        Ok(AnalysisResult {
            issues,
            metrics,
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn get_info() -> PluginInfo {
        PluginInfo {
            id: "performance-analyzer".to_string(),
            name: "Performance Analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyzes code for performance bottlenecks".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            supported_languages: vec![
                "rust".to_string(), "python".to_string(),
                "javascript".to_string(), "typescript".to_string(),
            ],
            detector_types: vec![IssueCategory::Performance, IssueCategory::Complexity],
            api_version: "1.0".to_string(),
            required_permissions: vec![],
        }
    }

    fn cleanup() -> Result<(), String> {
        *ANALYZER_STATE.borrow_mut() = None;
        Ok(())
    }
}

thread_local! {
    static ANALYZER_STATE: RefCell<Option<AnalyzerState>> = RefCell::new(None);
}

fn create_span(line: u32, start_col: u32, end_col: u32) -> Span {
    Span {
        start: Position { line, column: start_col, byte_offset: 0 },
        end: Position { line, column: end_col, byte_offset: 0 },
    }
}

fn calculate_maintainability(complexity: u32, lines: usize) -> f64 {
    // Simplified maintainability index
    let base = 100.0;
    let complexity_penalty = complexity as f64 * 2.0;
    let size_penalty = (lines as f64 / 100.0) * 5.0;
    (base - complexity_penalty - size_penalty).max(0.0)
}
```

## Custom Rule Engine Plugin

A flexible plugin that allows custom rule definitions from configuration.

### src/lib.rs

```rust
//! Custom Rule Engine Plugin
//!
//! Allows users to define custom detection rules via configuration.

wit_bindgen::generate!({
    world: "core-analysis",
    path: "wit/core-analysis.wit",
});

use std::cell::RefCell;

struct RuleEnginePlugin {
    state: RefCell<Option<RuleEngineState>>,
}

struct RuleEngineState {
    rules: Vec<CustomRule>,
}

struct CustomRule {
    id: String,
    name: String,
    pattern: String,
    severity: SeverityLevel,
    message: String,
    suggestion: String,
    languages: Vec<String>,
}

impl Default for RuleEnginePlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

export!(RuleEnginePlugin);

impl Guest for RuleEnginePlugin {
    fn initialize(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Custom Rule Engine");

        // Parse rules from custom_settings
        let mut rules = Vec::new();

        // Look for rules in config
        // Format: rule.<id>.pattern, rule.<id>.severity, etc.
        let rule_ids: Vec<String> = config.custom_settings.iter()
            .filter_map(|(k, _)| {
                if k.starts_with("rule.") {
                    k.split('.').nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for rule_id in rule_ids {
            let get_setting = |suffix: &str| -> Option<String> {
                config.custom_settings.iter()
                    .find(|(k, _)| k == &format!("rule.{}.{}", rule_id, suffix))
                    .map(|(_, v)| v.clone())
            };

            if let (Some(pattern), Some(message)) = (get_setting("pattern"), get_setting("message")) {
                rules.push(CustomRule {
                    id: rule_id.clone(),
                    name: get_setting("name").unwrap_or(rule_id.clone()),
                    pattern,
                    severity: match get_setting("severity").as_deref() {
                        Some("critical") => SeverityLevel::Critical,
                        Some("high") => SeverityLevel::High,
                        Some("medium") => SeverityLevel::Medium,
                        Some("low") => SeverityLevel::Low,
                        _ => SeverityLevel::Info,
                    },
                    message,
                    suggestion: get_setting("suggestion").unwrap_or_default(),
                    languages: get_setting("languages")
                        .map(|s| s.split(',').map(|l| l.trim().to_string()).collect())
                        .unwrap_or_default(),
                });
            }
        }

        // Add default rules if none configured
        if rules.is_empty() {
            rules.push(CustomRule {
                id: "DEFAULT001".to_string(),
                name: "Print Statement".to_string(),
                pattern: "print|println|console.log".to_string(),
                severity: SeverityLevel::Info,
                message: "Debug print statement found".to_string(),
                suggestion: "Consider using a logging framework".to_string(),
                languages: vec![],
            });
        }

        log(LogLevel::Info, &format!("Loaded {} custom rules", rules.len()));

        *RULE_STATE.borrow_mut() = Some(RuleEngineState { rules });
        Ok(())
    }

    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        let state_ref = RULE_STATE.borrow();
        let state = state_ref.as_ref().ok_or("Plugin not initialized")?;

        let mut issues = Vec::new();

        for rule in &state.rules {
            // Check language filter
            if !rule.languages.is_empty() && !rule.languages.contains(&file.language) {
                continue;
            }

            // Apply pattern to each line
            let patterns: Vec<&str> = rule.pattern.split('|').collect();

            for (line_num, line) in file.content.lines().enumerate() {
                for pattern in &patterns {
                    if line.to_lowercase().contains(&pattern.to_lowercase()) {
                        issues.push(Issue {
                            id: format!("{}-{}", rule.id, line_num + 1),
                            severity: rule.severity.clone(),
                            category: IssueCategory::Quality,
                            message: rule.message.clone(),
                            description: Some(format!("Rule: {}", rule.name)),
                            file: file.path.clone(),
                            span: Span {
                                start: Position {
                                    line: line_num as u32 + 1,
                                    column: 0,
                                    byte_offset: 0,
                                },
                                end: Position {
                                    line: line_num as u32 + 1,
                                    column: line.len() as u32,
                                    byte_offset: 0,
                                },
                            },
                            rule_id: Some(rule.id.clone()),
                            suggestion: if rule.suggestion.is_empty() {
                                None
                            } else {
                                Some(rule.suggestion.clone())
                            },
                            fix: None,
                            metadata: vec![("pattern".to_string(), pattern.to_string())],
                        });
                        break;
                    }
                }
            }
        }

        Ok(AnalysisResult {
            issues,
            metrics: Metrics {
                lines_of_code: file.content.lines().count() as u32,
                lines_of_comments: 0,
                complexity: 0,
                maintainability_index: 100.0,
                technical_debt_minutes: 0,
                custom_metrics: vec![],
            },
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn get_info() -> PluginInfo {
        PluginInfo {
            id: "custom-rule-engine".to_string(),
            name: "Custom Rule Engine".to_string(),
            version: "1.0.0".to_string(),
            description: "Applies user-defined detection rules".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            supported_languages: vec!["*".to_string()],
            detector_types: vec![IssueCategory::Quality],
            api_version: "1.0".to_string(),
            required_permissions: vec![],
        }
    }

    fn cleanup() -> Result<(), String> {
        *RULE_STATE.borrow_mut() = None;
        Ok(())
    }
}

thread_local! {
    static RULE_STATE: RefCell<Option<RuleEngineState>> = RefCell::new(None);
}
```

## Multi-Language Plugin

A plugin that provides different analysis strategies for different languages.

### src/lib.rs

```rust
//! Multi-Language Analyzer Plugin
//!
//! Provides language-specific analysis for multiple programming languages.

wit_bindgen::generate!({
    world: "core-analysis",
    path: "wit/core-analysis.wit",
});

use std::cell::RefCell;

struct MultiLanguagePlugin {
    state: RefCell<Option<MultiLangState>>,
}

struct MultiLangState {
    config: PluginConfig,
}

impl Default for MultiLanguagePlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

export!(MultiLanguagePlugin);

impl Guest for MultiLanguagePlugin {
    fn initialize(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Multi-Language Analyzer");
        *MULTI_STATE.borrow_mut() = Some(MultiLangState { config });
        Ok(())
    }

    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        log(LogLevel::Info, &format!("Analyzing {} ({})", file.path, file.language));

        // Dispatch to language-specific analyzer
        let issues = match file.language.as_str() {
            "rust" => analyze_rust(&file),
            "python" => analyze_python(&file),
            "javascript" | "typescript" => analyze_javascript(&file),
            _ => {
                log(LogLevel::Warn, &format!("No specific analyzer for {}", file.language));
                vec![]
            }
        };

        Ok(AnalysisResult {
            issues,
            metrics: Metrics {
                lines_of_code: file.content.lines().count() as u32,
                lines_of_comments: 0,
                complexity: 0,
                maintainability_index: 100.0,
                technical_debt_minutes: 0,
                custom_metrics: vec![],
            },
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn get_info() -> PluginInfo {
        PluginInfo {
            id: "multi-language".to_string(),
            name: "Multi-Language Analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "Language-specific analysis for multiple languages".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            supported_languages: vec![
                "rust".to_string(), "python".to_string(),
                "javascript".to_string(), "typescript".to_string(),
            ],
            detector_types: vec![IssueCategory::Quality, IssueCategory::Security],
            api_version: "1.0".to_string(),
            required_permissions: vec![],
        }
    }

    fn cleanup() -> Result<(), String> {
        *MULTI_STATE.borrow_mut() = None;
        Ok(())
    }
}

thread_local! {
    static MULTI_STATE: RefCell<Option<MultiLangState>> = RefCell::new(None);
}

fn analyze_rust(file: &SourceFile) -> Vec<Issue> {
    let mut issues = Vec::new();

    for (line_num, line) in file.content.lines().enumerate() {
        // Check for unsafe blocks
        if line.contains("unsafe {") || line.contains("unsafe{") {
            issues.push(create_issue(
                "RUST001", SeverityLevel::Medium, IssueCategory::Security,
                "Unsafe block detected",
                "Document why unsafe is necessary here",
                file, line_num,
            ));
        }

        // Check for unwrap without handling
        if line.contains(".unwrap()") && !line.contains("expect") {
            issues.push(create_issue(
                "RUST002", SeverityLevel::Low, IssueCategory::Quality,
                "Using unwrap() without error context",
                "Consider using expect() with a message or proper error handling",
                file, line_num,
            ));
        }
    }

    issues
}

fn analyze_python(file: &SourceFile) -> Vec<Issue> {
    let mut issues = Vec::new();

    for (line_num, line) in file.content.lines().enumerate() {
        // Check for bare except
        if line.contains("except:") && !line.contains("except ") {
            issues.push(create_issue(
                "PY001", SeverityLevel::Medium, IssueCategory::Quality,
                "Bare except clause",
                "Specify the exception type to catch",
                file, line_num,
            ));
        }

        // Check for mutable default arguments
        if line.contains("def ") && (line.contains("=[]") || line.contains("={}")) {
            issues.push(create_issue(
                "PY002", SeverityLevel::High, IssueCategory::Quality,
                "Mutable default argument",
                "Use None as default and initialize in function body",
                file, line_num,
            ));
        }
    }

    issues
}

fn analyze_javascript(file: &SourceFile) -> Vec<Issue> {
    let mut issues = Vec::new();

    for (line_num, line) in file.content.lines().enumerate() {
        // Check for var usage
        if line.contains("var ") {
            issues.push(create_issue(
                "JS001", SeverityLevel::Low, IssueCategory::Style,
                "Using 'var' instead of 'let' or 'const'",
                "Prefer 'let' or 'const' for block-scoped variables",
                file, line_num,
            ));
        }

        // Check for == instead of ===
        if line.contains(" == ") && !line.contains(" === ") {
            issues.push(create_issue(
                "JS002", SeverityLevel::Medium, IssueCategory::Quality,
                "Using == instead of ===",
                "Use strict equality (===) to avoid type coercion",
                file, line_num,
            ));
        }
    }

    issues
}

fn create_issue(
    id: &str,
    severity: SeverityLevel,
    category: IssueCategory,
    message: &str,
    suggestion: &str,
    file: &SourceFile,
    line_num: usize,
) -> Issue {
    Issue {
        id: format!("{}-{}", id, line_num + 1),
        severity,
        category,
        message: message.to_string(),
        description: None,
        file: file.path.clone(),
        span: Span {
            start: Position { line: line_num as u32 + 1, column: 0, byte_offset: 0 },
            end: Position { line: line_num as u32 + 1, column: 0, byte_offset: 0 },
        },
        rule_id: Some(id.to_string()),
        suggestion: Some(suggestion.to_string()),
        fix: None,
        metadata: vec![],
    }
}
```

## Testing Strategies

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(content: &str, language: &str) -> SourceFile {
        SourceFile {
            path: format!("test.{}", match language {
                "rust" => "rs",
                "python" => "py",
                "javascript" => "js",
                _ => "txt",
            }),
            content: content.to_string(),
            language: language.to_string(),
            size: content.len() as u32,
            hash: format!("{:x}", content.len()),
            ast: None,
        }
    }

    #[test]
    fn test_todo_detection() {
        let file = create_test_file(
            "// TODO: implement\nfn main() {}\n// FIXME: bug",
            "rust"
        );

        // Initialize plugin
        let config = PluginConfig {
            severity_threshold: SeverityLevel::Info,
            max_issues_per_file: 100,
            include_patterns: vec![],
            exclude_patterns: vec![],
            rule_overrides: vec![],
            custom_settings: vec![],
        };

        TodoDetectorPlugin::initialize(config, ResourceLimits::default()).unwrap();

        let result = TodoDetectorPlugin::analyze(file).unwrap();

        assert_eq!(result.issues.len(), 2);
        assert!(result.issues.iter().any(|i| i.id.contains("TODO")));
        assert!(result.issues.iter().any(|i| i.id.contains("FIXME")));
    }

    #[test]
    fn test_empty_file() {
        let file = create_test_file("", "rust");
        let result = TodoDetectorPlugin::analyze(file).unwrap();
        assert!(result.issues.is_empty());
    }
}
```

### Integration Testing Script

```bash
#!/bin/bash
# test-plugin.sh

set -e

echo "Building plugin..."
cargo build --release --target wasm32-wasi

echo "Installing plugin..."
WASM_PATH="target/wasm32-wasi/release/my_plugin.wasm"
uveddi plugin install "$WASM_PATH" --force

echo "Creating test files..."
mkdir -p test-data

cat > test-data/sample.rs << 'EOF'
// TODO: implement proper error handling
fn main() {
    // FIXME: this is broken
    println!("Hello");
}
EOF

echo "Running plugin tests..."
uveddi plugin test my-plugin --test-file test-data/sample.rs --verbose

echo "Running full analysis..."
uveddi analyze test-data/ --plugins my-plugin --output json > results.json

echo "Verifying results..."
if jq -e '.issues | length > 0' results.json > /dev/null; then
    echo "SUCCESS: Plugin detected issues"
    jq '.issues[] | {id, message, severity}' results.json
else
    echo "FAILURE: No issues detected"
    exit 1
fi

echo "Cleanup..."
rm -rf test-data results.json

echo "All tests passed!"
```

## Best Practices Summary

1. **Use WIT Component Model**: Always use `wit_bindgen::generate!` for type-safe host communication
2. **Thread-local State**: Use `thread_local!` with `RefCell` for plugin state
3. **Proper Error Handling**: Return descriptive `Result<T, String>` errors
4. **Comprehensive Logging**: Use appropriate log levels for debugging
5. **Validate Inputs**: Check file size and language before processing
6. **Performance**: Pre-allocate vectors, process incrementally for large files
7. **Testing**: Write both unit tests and integration tests

For more details, see the [API Reference](api-reference.md) and [Development Guide](development-guide.md).
