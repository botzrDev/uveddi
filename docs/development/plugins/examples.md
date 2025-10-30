# Plugin Examples

This document provides comprehensive examples for developing different types of Uveddi plugins. Each example includes complete source code, configuration, and testing instructions.

## Table of Contents

1. [Basic Detector Plugin](#basic-detector-plugin)
2. [Security Scanner Plugin](#security-scanner-plugin)
3. [Performance Analyzer Plugin](#performance-analyzer-plugin)
4. [Custom Rule Engine Plugin](#custom-rule-engine-plugin)
5. [AI-Powered Code Quality Plugin](#ai-powered-code-quality-plugin)
6. [Advanced Metrics Plugin](#advanced-metrics-plugin)
7. [Framework-Specific Plugin](#framework-specific-plugin)
8. [Enterprise Compliance Plugin](#enterprise-compliance-plugin)

## Basic Detector Plugin

This example shows a simple plugin that detects TODO comments in code.

### Plugin Structure

```
todo-detector/
├── Cargo.toml
├── plugin.toml
├── src/
│   └── lib.rs
├── tests/
│   └── integration.rs
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
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.5"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
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
permissions = ["Logging"]
max_memory_mb = 16
max_execution_seconds = 10

[detection]
anti_pattern_types = ["CODE_SMELL", "MAINTAINABILITY"]
issue_categories = ["TODO_COMMENTS"]
severity_levels = ["INFO", "WARNING"]

[dependencies]
min_uveddi_version = "0.9.0"
supported_languages = ["rust", "python", "javascript", "typescript", "java", "go"]
```

### src/lib.rs

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

#[derive(Deserialize)]
pub struct PluginFileInput {
    pub file_path: String,
    pub content: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct PluginIssue {
    pub issue_type: String,
    pub severity: String,
    pub message: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub column: Option<u32>,
    pub suggestion: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Serialize)]
pub struct PluginAnalysisResult {
    pub plugin_name: String,
    pub issues: Vec<PluginIssue>,
    pub metadata: HashMap<String, String>,
}

struct TodoDetector {
    patterns: Vec<CommentPattern>,
}

struct CommentPattern {
    keyword: &'static str,
    regex: Regex,
    severity: &'static str,
    message_template: &'static str,
}

impl TodoDetector {
    fn new() -> Self {
        let patterns = vec![
            CommentPattern {
                keyword: "TODO",
                regex: Regex::new(r"(?i)(?://|#|\*|<!--)\s*TODO\b").unwrap(),
                severity: "INFO",
                message_template: "TODO comment found - consider creating a proper issue",
            },
            CommentPattern {
                keyword: "FIXME",
                regex: Regex::new(r"(?i)(?://|#|\*|<!--)\s*FIXME\b").unwrap(),
                severity: "WARNING",
                message_template: "FIXME comment found - indicates code that needs fixing",
            },
            CommentPattern {
                keyword: "HACK",
                regex: Regex::new(r"(?i)(?://|#|\*|<!--)\s*HACK\b").unwrap(),
                severity: "WARNING",
                message_template: "HACK comment found - indicates non-standard solution",
            },
            CommentPattern {
                keyword: "BUG",
                regex: Regex::new(r"(?i)(?://|#|\*|<!--)\s*BUG\b").unwrap(),
                severity: "WARNING",
                message_template: "BUG comment found - indicates known issue",
            },
        ];

        TodoDetector { patterns }
    }

    fn analyze(&self, input: &PluginFileInput) -> Vec<PluginIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in input.content.lines().enumerate() {
            for pattern in &self.patterns {
                if let Some(mat) = pattern.regex.find(line) {
                    let suggestion = match pattern.keyword {
                        "TODO" => Some("Create a GitHub issue or Jira ticket for this task".to_string()),
                        "FIXME" => Some("Schedule time to fix this issue".to_string()),
                        "HACK" => Some("Refactor to use a proper solution".to_string()),
                        "BUG" => Some("File a bug report and fix this issue".to_string()),
                        _ => None,
                    };

                    issues.push(PluginIssue {
                        issue_type: format!("{}_COMMENT", pattern.keyword),
                        severity: pattern.severity.to_string(),
                        message: pattern.message_template.to_string(),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_num as u32 + 1),
                        column: Some(mat.start() as u32),
                        suggestion,
                        confidence: Some(0.95),
                    });
                }
            }
        }

        issues
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => {
            return create_error_result(&format!("Failed to parse input: {}", e));
        }
    };

    let detector = TodoDetector::new();
    let issues = detector.analyze(&input);

    let result = PluginAnalysisResult {
        plugin_name: "todo-detector".to_string(),
        issues,
        metadata: [
            ("analyzed_lines".to_string(), input.content.lines().count().to_string()),
            ("language".to_string(), input.language.clone()),
        ].into_iter().collect(),
    };

    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Failed to serialize result")
    })
}

#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8> {
    let info = serde_json::json!({
        "name": "todo-detector",
        "version": "1.0.0",
        "description": "Detects TODO, FIXME, and HACK comments in code",
        "supported_languages": ["rust", "python", "javascript", "typescript", "java", "go"],
        "capabilities": {
            "static_analysis": true,
            "comment_analysis": true
        }
    });

    serde_json::to_vec(&info).unwrap_or_default()
}

fn create_error_result(message: &str) -> Vec<u8> {
    let result = PluginAnalysisResult {
        plugin_name: "todo-detector".to_string(),
        issues: vec![],
        metadata: [("error".to_string(), message.to_string())].into_iter().collect(),
    };
    serde_json::to_vec(&result).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_detection() {
        let input = PluginFileInput {
            file_path: "test.rs".to_string(),
            content: "// TODO: Implement this\nfn main() {\n    // FIXME: Handle error\n}".to_string(),
            language: "rust".to_string(),
        };

        let detector = TodoDetector::new();
        let issues = detector.analyze(&input);

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].issue_type, "TODO_COMMENT");
        assert_eq!(issues[1].issue_type, "FIXME_COMMENT");
    }
}
```

### Build and Test

```bash
# Build the plugin
cargo build --release --target wasm32-wasi

# Test with Uveddi
echo "// TODO: Fix this later" > test.rs
uveddi plugin test ./target/wasm32-wasi/release/todo_detector_plugin.wasm test.rs
```

## Security Scanner Plugin

A comprehensive security scanner that detects various security vulnerabilities.

### src/lib.rs

```rust
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct SecurityIssue {
    pub issue_type: String,
    pub severity: String,
    pub message: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub column: Option<u32>,
    pub suggestion: Option<String>,
    pub confidence: Option<f64>,
    pub cwe_id: Option<String>,
    pub owasp_category: Option<String>,
}

struct SecurityRule {
    id: &'static str,
    name: &'static str,
    pattern: Regex,
    severity: &'static str,
    message: &'static str,
    suggestion: &'static str,
    cwe_id: Option<&'static str>,
    owasp_category: Option<&'static str>,
    confidence: f64,
}

struct SecurityScanner {
    rules: Vec<SecurityRule>,
}

impl SecurityScanner {
    fn new() -> Self {
        let rules = vec![
            // SQL Injection patterns
            SecurityRule {
                id: "SEC001",
                name: "Potential SQL Injection",
                pattern: Regex::new(r#"(?i)(query|execute|exec)\s*\(\s*["\'][^"\']*\+[^"\']*["\']"#).unwrap(),
                severity: "HIGH",
                message: "Potential SQL injection vulnerability detected",
                suggestion: "Use parameterized queries or prepared statements",
                cwe_id: Some("CWE-89"),
                owasp_category: Some("A03:2021-Injection"),
                confidence: 0.8,
            },
            // XSS patterns
            SecurityRule {
                id: "SEC002", 
                name: "Potential XSS",
                pattern: Regex::new(r"(?i)innerHTML\s*=\s*.*\+").unwrap(),
                severity: "HIGH",
                message: "Potential XSS vulnerability through innerHTML",
                suggestion: "Use textContent or properly sanitize input",
                cwe_id: Some("CWE-79"),
                owasp_category: Some("A03:2021-Injection"),
                confidence: 0.7,
            },
            // Hardcoded secrets
            SecurityRule {
                id: "SEC003",
                name: "Hardcoded Secret",
                pattern: Regex::new(r#"(?i)(password|secret|key|token)\s*=\s*["\'][A-Za-z0-9+/=]{8,}["\']"#).unwrap(),
                severity: "CRITICAL",
                message: "Hardcoded secret detected",
                suggestion: "Use environment variables or a secure key management system",
                cwe_id: Some("CWE-798"),
                owasp_category: Some("A02:2021-Cryptographic-Failures"),
                confidence: 0.9,
            },
            // Weak crypto
            SecurityRule {
                id: "SEC004",
                name: "Weak Cryptography",
                pattern: Regex::new(r"(?i)(md5|sha1|des|rc4)").unwrap(),
                severity: "MEDIUM",
                message: "Weak cryptographic algorithm detected",
                suggestion: "Use SHA-256 or stronger algorithms",
                cwe_id: Some("CWE-327"),
                owasp_category: Some("A02:2021-Cryptographic-Failures"),
                confidence: 0.6,
            },
        ];

        SecurityScanner { rules }
    }

    fn scan(&self, input: &PluginFileInput) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in input.content.lines().enumerate() {
            for rule in &self.rules {
                if let Some(mat) = rule.pattern.find(line) {
                    issues.push(SecurityIssue {
                        issue_type: rule.id.to_string(),
                        severity: rule.severity.to_string(),
                        message: format!("{}: {}", rule.name, rule.message),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_num as u32 + 1),
                        column: Some(mat.start() as u32),
                        suggestion: Some(rule.suggestion.to_string()),
                        confidence: Some(rule.confidence),
                        cwe_id: rule.cwe_id.map(String::from),
                        owasp_category: rule.owasp_category.map(String::from),
                    });
                }
            }
        }

        issues
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => return create_error_result(&format!("Parse error: {}", e)),
    };

    let scanner = SecurityScanner::new();
    let issues = scanner.scan(&input);

    let result = PluginAnalysisResult {
        plugin_name: "security-scanner".to_string(),
        issues: issues.into_iter().map(|issue| PluginIssue {
            issue_type: issue.issue_type,
            severity: issue.severity,
            message: issue.message,
            file_path: issue.file_path,
            line_number: issue.line_number,
            column: issue.column,
            suggestion: issue.suggestion,
            confidence: issue.confidence,
        }).collect(),
        metadata: [
            ("scanned_lines".to_string(), input.content.lines().count().to_string()),
            ("rules_applied".to_string(), scanner.rules.len().to_string()),
        ].into_iter().collect(),
    };

    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Serialization failed")
    })
}
```

## Performance Analyzer Plugin

Analyzes code for performance bottlenecks and optimization opportunities.

### src/lib.rs

```rust
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct PerformanceMetrics {
    pub cyclomatic_complexity: u32,
    pub function_count: u32,
    pub loop_count: u32,
    pub nested_loop_count: u32,
    pub string_concatenations: u32,
    pub sync_io_operations: u32,
}

struct PerformanceAnalyzer {
    patterns: HashMap<&'static str, Regex>,
}

impl PerformanceAnalyzer {
    fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Function definitions
        patterns.insert("function", Regex::new(r"(?i)\b(fn|function|def|public|private)\s+\w+\s*\(").unwrap());
        
        // Loop constructs
        patterns.insert("for_loop", Regex::new(r"(?i)\b(for|foreach)\b").unwrap());
        patterns.insert("while_loop", Regex::new(r"(?i)\bwhile\b").unwrap());
        
        // String concatenation in loops (performance issue)
        patterns.insert("string_concat", Regex::new(r#"(?i)(\+\s*=\s*["\']|\bconcat\b|\bappend\b)"#).unwrap());
        
        // Synchronous I/O operations
        patterns.insert("sync_io", Regex::new(r"(?i)\b(read|write|open)(?!async)\b").unwrap());
        
        // Nested structure indicators
        patterns.insert("if_statement", Regex::new(r"(?i)\bif\b").unwrap());
        patterns.insert("switch_case", Regex::new(r"(?i)\b(switch|match|case)\b").unwrap());

        PerformanceAnalyzer { patterns }
    }

    fn analyze(&self, input: &PluginFileInput) -> (Vec<PluginIssue>, PerformanceMetrics) {
        let mut issues = Vec::new();
        let mut metrics = PerformanceMetrics {
            cyclomatic_complexity: 1, // Base complexity
            function_count: 0,
            loop_count: 0,
            nested_loop_count: 0,
            string_concatenations: 0,
            sync_io_operations: 0,
        };

        let mut loop_nesting_level = 0;
        let mut current_complexity = 1;

        for (line_num, line) in input.content.lines().enumerate() {
            let line_number = line_num as u32 + 1;

            // Count functions
            if self.patterns["function"].is_match(line) {
                metrics.function_count += 1;
            }

            // Analyze loops
            if self.patterns["for_loop"].is_match(line) || self.patterns["while_loop"].is_match(line) {
                metrics.loop_count += 1;
                loop_nesting_level += 1;
                current_complexity += 1;

                if loop_nesting_level > 1 {
                    metrics.nested_loop_count += 1;
                    issues.push(PluginIssue {
                        issue_type: "PERF001".to_string(),
                        severity: "WARNING".to_string(),
                        message: "Nested loop detected - potential performance bottleneck".to_string(),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_number),
                        column: None,
                        suggestion: Some("Consider optimizing algorithm complexity".to_string()),
                        confidence: Some(0.8),
                    });
                }

                // Check for string concatenation in loops
                if self.patterns["string_concat"].is_match(line) {
                    metrics.string_concatenations += 1;
                    issues.push(PluginIssue {
                        issue_type: "PERF003".to_string(),
                        severity: "MEDIUM".to_string(),
                        message: "String concatenation in loop - inefficient memory usage".to_string(),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_number),
                        column: None,
                        suggestion: Some("Use StringBuilder or Vec<String> for efficient concatenation".to_string()),
                        confidence: Some(0.9),
                    });
                }
            }

            // Detect end of loop constructs (simplified)
            if line.trim() == "}" && loop_nesting_level > 0 {
                loop_nesting_level = loop_nesting_level.saturating_sub(1);
            }

            // Detect synchronous I/O
            if self.patterns["sync_io"].is_match(line) && !line.contains("async") {
                metrics.sync_io_operations += 1;
                issues.push(PluginIssue {
                    issue_type: "PERF004".to_string(),
                    severity: "INFO".to_string(),
                    message: "Synchronous I/O operation detected".to_string(),
                    file_path: input.file_path.clone(),
                    line_number: Some(line_number),
                    column: None,
                    suggestion: Some("Consider using asynchronous I/O for better performance".to_string()),
                    confidence: Some(0.7),
                });
            }

            // Calculate cyclomatic complexity
            if self.patterns["if_statement"].is_match(line) || 
               self.patterns["switch_case"].is_match(line) {
                current_complexity += 1;
            }
        }

        metrics.cyclomatic_complexity = current_complexity;

        // Generate high-level performance warnings
        if metrics.cyclomatic_complexity > 15 {
            issues.push(PluginIssue {
                issue_type: "PERF_COMPLEXITY".to_string(),
                severity: "WARNING".to_string(),
                message: format!("High cyclomatic complexity: {}", metrics.cyclomatic_complexity),
                file_path: input.file_path.clone(),
                line_number: None,
                column: None,
                suggestion: Some("Consider refactoring into smaller functions".to_string()),
                confidence: Some(0.95),
            });
        }

        (issues, metrics)
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => return create_error_result(&format!("Parse error: {}", e)),
    };

    let analyzer = PerformanceAnalyzer::new();
    let (issues, metrics) = analyzer.analyze(&input);

    let mut metadata = HashMap::new();
    metadata.insert("cyclomatic_complexity".to_string(), metrics.cyclomatic_complexity.to_string());
    metadata.insert("function_count".to_string(), metrics.function_count.to_string());
    metadata.insert("loop_count".to_string(), metrics.loop_count.to_string());
    metadata.insert("nested_loops".to_string(), metrics.nested_loop_count.to_string());

    let result = PluginAnalysisResult {
        plugin_name: "performance-analyzer".to_string(),
        issues,
        metadata,
    };

    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Serialization failed")
    })
}
```

## Custom Rule Engine Plugin

A flexible rule engine that allows users to define custom detection patterns.

### Configuration Example

```toml
# custom-rules.toml
[[rules]]
id = "CUSTOM001"
name = "No Print Statements"
pattern = '\b(print|println|console\.log)\s*\('
severity = "WARNING"
message = "Print statement detected in production code"
suggestion = "Use proper logging framework"

[[rules]]
id = "CUSTOM002"
name = "Long Parameter Lists"
pattern = '\([^)]*,[^)]*,[^)]*,[^)]*,[^)]*,'
severity = "INFO"
message = "Function has too many parameters"
suggestion = "Consider using a struct or object to group parameters"
```

### src/lib.rs

```rust
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
struct CustomRule {
    id: String,
    name: String,
    pattern: String,
    severity: String,
    message: String,
    suggestion: Option<String>,
    enabled: Option<bool>,
    languages: Option<Vec<String>>,
}

struct RuleEngine {
    rules: Vec<CompiledRule>,
}

struct CompiledRule {
    rule: CustomRule,
    regex: Regex,
}

impl RuleEngine {
    fn new() -> Self {
        RuleEngine { rules: Vec::new() }
    }

    fn load_rules(&mut self, rules_config: &str) -> Result<(), String> {
        let config: RuleConfig = toml::from_str(rules_config)
            .map_err(|e| format!("Failed to parse rules config: {}", e))?;

        for rule in config.rules {
            if rule.enabled.unwrap_or(true) {
                match Regex::new(&rule.pattern) {
                    Ok(regex) => {
                        self.rules.push(CompiledRule { rule, regex });
                    }
                    Err(e) => {
                        return Err(format!("Invalid regex in rule {}: {}", rule.id, e));
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze(&self, input: &PluginFileInput) -> Vec<PluginIssue> {
        let mut issues = Vec::new();

        for compiled_rule in &self.rules {
            // Check if rule applies to this language
            if let Some(ref languages) = compiled_rule.rule.languages {
                if !languages.contains(&input.language) {
                    continue;
                }
            }

            // Apply rule to each line
            for (line_num, line) in input.content.lines().enumerate() {
                if let Some(mat) = compiled_rule.regex.find(line) {
                    issues.push(PluginIssue {
                        issue_type: compiled_rule.rule.id.clone(),
                        severity: compiled_rule.rule.severity.clone(),
                        message: format!("{}: {}", compiled_rule.rule.name, compiled_rule.rule.message),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_num as u32 + 1),
                        column: Some(mat.start() as u32),
                        suggestion: compiled_rule.rule.suggestion.clone(),
                        confidence: Some(0.8),
                    });
                }
            }
        }

        issues
    }
}

#[derive(Deserialize)]
struct RuleConfig {
    rules: Vec<CustomRule>,
}

// Configuration loading from host
extern "C" {
    fn host_get_config(key_ptr: *const u8, key_len: usize) -> u64;
}

fn get_rules_config() -> Option<String> {
    // Try to get rules configuration from host
    unsafe {
        let key = "custom_rules.config";
        let handle = host_get_config(key.as_ptr(), key.len());
        if handle != 0 {
            // Extract config string from handle (implementation specific)
            Some(r#"
[[rules]]
id = "CUSTOM001"
name = "No Print Statements"
pattern = '\b(print|println|console\.log)\s*\('
severity = "WARNING"
message = "Print statement detected"
suggestion = "Use proper logging framework"
            "#.to_string())
        } else {
            None
        }
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => return create_error_result(&format!("Parse error: {}", e)),
    };

    let mut engine = RuleEngine::new();

    // Load rules from configuration
    if let Some(config) = get_rules_config() {
        if let Err(e) = engine.load_rules(&config) {
            return create_error_result(&format!("Failed to load rules: {}", e));
        }
    } else {
        return create_error_result("No rules configuration found");
    }

    let issues = engine.analyze(&input);

    let result = PluginAnalysisResult {
        plugin_name: "custom-rule-engine".to_string(),
        issues,
        metadata: [
            ("rules_loaded".to_string(), engine.rules.len().to_string()),
            ("language".to_string(), input.language.clone()),
        ].into_iter().collect(),
    };

    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Serialization failed")
    })
}
```

## AI-Powered Code Quality Plugin

Uses AI models to analyze code quality and provide intelligent suggestions.

### src/lib.rs

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct AIAnalysisResult {
    pub quality_score: f64,
    pub maintainability_score: f64,
    pub readability_score: f64,
    pub suggestions: Vec<AISuggestion>,
    pub complexity_analysis: ComplexityAnalysis,
}

#[derive(Serialize)]
pub struct AISuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub confidence: f64,
    pub line_range: Option<(u32, u32)>,
    pub before_code: Option<String>,
    pub after_code: Option<String>,
}

#[derive(Serialize)]
pub struct ComplexityAnalysis {
    pub cognitive_complexity: u32,
    pub cyclomatic_complexity: u32,
    pub nesting_depth: u32,
    pub hotspots: Vec<ComplexityHotspot>,
}

#[derive(Serialize)]
pub struct ComplexityHotspot {
    pub line_number: u32,
    pub complexity_score: u32,
    pub description: String,
}

struct AICodeAnalyzer {
    // In a real implementation, this would include ML model integration
    patterns: HashMap<&'static str, Vec<QualityPattern>>,
}

struct QualityPattern {
    name: &'static str,
    pattern: regex::Regex,
    impact: f64,
    suggestion: &'static str,
}

impl AICodeAnalyzer {
    fn new() -> Self {
        let mut patterns = HashMap::new();

        // Readability patterns
        let readability_patterns = vec![
            QualityPattern {
                name: "Long Line",
                pattern: regex::Regex::new(r".{120,}").unwrap(),
                impact: -0.1,
                suggestion: "Consider breaking long lines for better readability",
            },
            QualityPattern {
                name: "Deep Nesting",
                pattern: regex::Regex::new(r"^\s{16,}").unwrap(), // 4+ levels of indentation
                impact: -0.2,
                suggestion: "Deep nesting detected - consider extracting methods",
            },
        ];

        // Maintainability patterns
        let maintainability_patterns = vec![
            QualityPattern {
                name: "Magic Number",
                pattern: regex::Regex::new(r"\b\d{2,}\b").unwrap(),
                impact: -0.1,
                suggestion: "Consider extracting magic numbers into named constants",
            },
            QualityPattern {
                name: "Duplicate Code",
                pattern: regex::Regex::new(r"(?m)^(.{10,})$\n(?:.*\n)*?\1").unwrap(),
                impact: -0.3,
                suggestion: "Potential code duplication detected - consider extraction",
            },
        ];

        patterns.insert("readability", readability_patterns);
        patterns.insert("maintainability", maintainability_patterns);

        AICodeAnalyzer { patterns }
    }

    fn analyze(&self, input: &PluginFileInput) -> (Vec<PluginIssue>, AIAnalysisResult) {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut quality_score = 100.0;
        let mut readability_score = 100.0;
        let mut maintainability_score = 100.0;

        // Analyze readability
        for pattern in &self.patterns["readability"] {
            for (line_num, line) in input.content.lines().enumerate() {
                if pattern.pattern.is_match(line) {
                    readability_score += pattern.impact;
                    quality_score += pattern.impact * 0.3;

                    issues.push(PluginIssue {
                        issue_type: "READABILITY".to_string(),
                        severity: "INFO".to_string(),
                        message: format!("Readability issue: {}", pattern.name),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_num as u32 + 1),
                        column: None,
                        suggestion: Some(pattern.suggestion.to_string()),
                        confidence: Some(0.7),
                    });

                    suggestions.push(AISuggestion {
                        suggestion_type: "readability".to_string(),
                        description: pattern.suggestion.to_string(),
                        confidence: 0.7,
                        line_range: Some((line_num as u32 + 1, line_num as u32 + 1)),
                        before_code: Some(line.to_string()),
                        after_code: None,
                    });
                }
            }
        }

        // Analyze maintainability
        for pattern in &self.patterns["maintainability"] {
            for (line_num, line) in input.content.lines().enumerate() {
                if pattern.pattern.is_match(line) {
                    maintainability_score += pattern.impact;
                    quality_score += pattern.impact * 0.4;

                    issues.push(PluginIssue {
                        issue_type: "MAINTAINABILITY".to_string(),
                        severity: "WARNING".to_string(),
                        message: format!("Maintainability issue: {}", pattern.name),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_num as u32 + 1),
                        column: None,
                        suggestion: Some(pattern.suggestion.to_string()),
                        confidence: Some(0.8),
                    });
                }
            }
        }

        // Calculate complexity
        let complexity = self.calculate_complexity(&input.content);

        // Generate AI-powered suggestions based on analysis
        if quality_score < 70.0 {
            suggestions.push(AISuggestion {
                suggestion_type: "refactoring".to_string(),
                description: "Consider refactoring this file to improve code quality".to_string(),
                confidence: 0.9,
                line_range: None,
                before_code: None,
                after_code: None,
            });
        }

        let ai_result = AIAnalysisResult {
            quality_score: quality_score.max(0.0),
            maintainability_score: maintainability_score.max(0.0),
            readability_score: readability_score.max(0.0),
            suggestions,
            complexity_analysis: complexity,
        };

        (issues, ai_result)
    }

    fn calculate_complexity(&self, content: &str) -> ComplexityAnalysis {
        let mut cognitive_complexity = 0;
        let mut cyclomatic_complexity = 1;
        let mut max_nesting = 0;
        let mut current_nesting = 0;
        let mut hotspots = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let indent_level = line.len() - line.trim_start().len();
            current_nesting = indent_level / 4; // Assuming 4-space indentation
            max_nesting = max_nesting.max(current_nesting);

            // Simple complexity calculation
            if line.contains("if ") || line.contains("while ") || 
               line.contains("for ") || line.contains("match ") {
                cyclomatic_complexity += 1;
                cognitive_complexity += 1 + current_nesting;

                if current_nesting > 3 {
                    hotspots.push(ComplexityHotspot {
                        line_number: line_num as u32 + 1,
                        complexity_score: current_nesting as u32,
                        description: "High nesting complexity".to_string(),
                    });
                }
            }
        }

        ComplexityAnalysis {
            cognitive_complexity,
            cyclomatic_complexity,
            nesting_depth: max_nesting as u32,
            hotspots,
        }
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => return create_error_result(&format!("Parse error: {}", e)),
    };

    let analyzer = AICodeAnalyzer::new();
    let (issues, ai_result) = analyzer.analyze(&input);

    let mut metadata = HashMap::new();
    metadata.insert("quality_score".to_string(), ai_result.quality_score.to_string());
    metadata.insert("maintainability_score".to_string(), ai_result.maintainability_score.to_string());
    metadata.insert("readability_score".to_string(), ai_result.readability_score.to_string());
    metadata.insert("suggestions_count".to_string(), ai_result.suggestions.len().to_string());

    let result = PluginAnalysisResult {
        plugin_name: "ai-code-quality".to_string(),
        issues,
        metadata,
    };

    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Serialization failed")
    })
}
```

## Framework-Specific Plugin (React)

A specialized plugin for React applications that detects React-specific patterns and issues.

### src/lib.rs

```rust
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;

struct ReactAnalyzer {
    patterns: HashMap<&'static str, Regex>,
    rules: Vec<ReactRule>,
}

struct ReactRule {
    id: &'static str,
    name: &'static str,
    pattern: &'static str,
    severity: &'static str,
    message: &'static str,
    suggestion: &'static str,
}

impl ReactAnalyzer {
    fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // React-specific patterns
        patterns.insert("component", Regex::new(r"(?:function|const)\s+([A-Z][a-zA-Z0-9]*)\s*\(").unwrap());
        patterns.insert("hook", Regex::new(r"\buse[A-Z][a-zA-Z0-9]*\(").unwrap());
        patterns.insert("jsx", Regex::new(r"<[A-Z][a-zA-Z0-9]*").unwrap());
        patterns.insert("inline_style", Regex::new(r#"style\s*=\s*\{\{.*?\}\}"#).unwrap());
        patterns.insert("direct_dom", Regex::new(r"document\.(getElementById|querySelector)").unwrap());

        let rules = vec![
            ReactRule {
                id: "REACT001",
                name: "Inline Styles",
                pattern: r#"style\s*=\s*\{\{.*?\}\}"#,
                severity: "INFO",
                message: "Inline styles detected",
                suggestion: "Consider using CSS classes or styled-components for better maintainability",
            },
            ReactRule {
                id: "REACT002",
                name: "Direct DOM Manipulation",
                pattern: r"document\.(getElementById|querySelector)",
                severity: "WARNING",
                message: "Direct DOM manipulation in React component",
                suggestion: "Use React refs instead of direct DOM manipulation",
            },
            ReactRule {
                id: "REACT003",
                name: "Missing Key Prop",
                pattern: r"\.map\([^}]*=>\s*<[^>]*(?!.*key=)",
                severity: "WARNING",
                message: "Missing key prop in list rendering",
                suggestion: "Add a unique key prop to list items for optimal rendering performance",
            },
            ReactRule {
                id: "REACT004",
                name: "Unsafe Lifecycle Method",
                pattern: r"componentWillMount|componentWillReceiveProps|componentWillUpdate",
                severity: "HIGH",
                message: "Unsafe lifecycle method detected",
                suggestion: "Replace with safe lifecycle methods or hooks",
            },
        ];

        ReactAnalyzer { patterns, rules }
    }

    fn analyze(&self, input: &PluginFileInput) -> Vec<PluginIssue> {
        let mut issues = Vec::new();

        // Only analyze JavaScript/TypeScript files that likely contain React code
        if !["javascript", "typescript"].contains(&input.language.as_str()) {
            return issues;
        }

        // Check if file contains React imports or JSX
        let has_react = input.content.contains("import React") || 
                       input.content.contains("from 'react'") ||
                       self.patterns["jsx"].is_match(&input.content);

        if !has_react {
            return issues;
        }

        // Apply React-specific rules
        for rule in &self.rules {
            let pattern = match Regex::new(rule.pattern) {
                Ok(p) => p,
                Err(_) => continue,
            };

            for (line_num, line) in input.content.lines().enumerate() {
                if pattern.is_match(line) {
                    issues.push(PluginIssue {
                        issue_type: rule.id.to_string(),
                        severity: rule.severity.to_string(),
                        message: format!("{}: {}", rule.name, rule.message),
                        file_path: input.file_path.clone(),
                        line_number: Some(line_num as u32 + 1),
                        column: None,
                        suggestion: Some(rule.suggestion.to_string()),
                        confidence: Some(0.85),
                    });
                }
            }
        }

        // Analyze component structure
        issues.extend(self.analyze_component_structure(input));
        
        // Analyze hook usage
        issues.extend(self.analyze_hook_usage(input));

        issues
    }

    fn analyze_component_structure(&self, input: &PluginFileInput) -> Vec<PluginIssue> {
        let mut issues = Vec::new();
        
        // Check for large components (simplified heuristic)
        let line_count = input.content.lines().count();
        if line_count > 200 {
            issues.push(PluginIssue {
                issue_type: "REACT_LARGE_COMPONENT".to_string(),
                severity: "INFO".to_string(),
                message: format!("Large component detected ({} lines)", line_count),
                file_path: input.file_path.clone(),
                line_number: None,
                column: None,
                suggestion: Some("Consider breaking this component into smaller, reusable components".to_string()),
                confidence: Some(0.9),
            });
        }

        issues
    }

    fn analyze_hook_usage(&self, input: &PluginFileInput) -> Vec<PluginIssue> {
        let mut issues = Vec::new();
        let mut in_component = false;
        let mut hook_calls = Vec::new();

        for (line_num, line) in input.content.lines().enumerate() {
            // Simple heuristic to detect if we're inside a component
            if self.patterns["component"].is_match(line) {
                in_component = true;
                hook_calls.clear();
            }

            // Detect hook calls
            if in_component && self.patterns["hook"].is_match(line) {
                hook_calls.push(line_num);
            }

            // Check for conditional hook usage (simplified)
            if in_component && line.contains("useState") && 
               (line.contains("if ") || line.contains("for ") || line.contains("while ")) {
                issues.push(PluginIssue {
                    issue_type: "REACT_CONDITIONAL_HOOK".to_string(),
                    severity: "HIGH".to_string(),
                    message: "Hook called conditionally".to_string(),
                    file_path: input.file_path.clone(),
                    line_number: Some(line_num as u32 + 1),
                    column: None,
                    suggestion: Some("Hooks must be called at the top level of components".to_string()),
                    confidence: Some(0.8),
                });
            }

            // Reset when leaving component (simplified)
            if line.contains("export") || line.contains("};") {
                in_component = false;
            }
        }

        issues
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => return create_error_result(&format!("Parse error: {}", e)),
    };

    let analyzer = ReactAnalyzer::new();
    let issues = analyzer.analyze(&input);

    let result = PluginAnalysisResult {
        plugin_name: "react-analyzer".to_string(),
        issues,
        metadata: [
            ("framework".to_string(), "React".to_string()),
            ("language".to_string(), input.language.clone()),
        ].into_iter().collect(),
    };

    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Serialization failed")
    })
}
```

## Testing Example

Here's a comprehensive test setup that can be used with any of the above plugins:

### tests/integration.rs

```rust
use std::process::Command;
use std::fs;

#[test]
fn test_plugin_with_uveddi() {
    // Build the plugin first
    let output = Command::new("cargo")
        .args(&["build", "--release", "--target", "wasm32-wasi"])
        .output()
        .expect("Failed to build plugin");

    assert!(output.status.success(), "Plugin build failed: {}", 
            String::from_utf8_lossy(&output.stderr));

    // Create test file
    fs::write("test_input.rs", r#"
        // TODO: Implement this function
        fn main() {
            println!("Hello, world!");
            // FIXME: Handle errors properly
        }
    "#).expect("Failed to write test file");

    // Test plugin with Uveddi CLI
    let output = Command::new("uveddi")
        .args(&[
            "plugin", "test",
            "./target/wasm32-wasi/release/todo_detector_plugin.wasm",
            "test_input.rs"
        ])
        .output()
        .expect("Failed to run plugin test");

    // Cleanup
    fs::remove_file("test_input.rs").ok();

    // Check that plugin executed successfully
    assert!(output.status.success(), "Plugin test failed: {}", 
            String::from_utf8_lossy(&output.stderr));

    // Verify output contains expected issues
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TODO_COMMENT"));
    assert!(stdout.contains("FIXME_COMMENT"));
}

#[test]
fn test_plugin_performance() {
    use std::time::Instant;

    let large_input = "// TODO: test\n".repeat(10000);
    let input = serde_json::json!({
        "file_path": "large_test.rs",
        "content": large_input,
        "language": "rust"
    });

    let start = Instant::now();
    
    // This would call your plugin's analyze_file function
    // let result = analyze_file(&serde_json::to_vec(&input).unwrap());
    
    let duration = start.elapsed();
    
    // Plugin should complete within reasonable time
    assert!(duration.as_millis() < 1000, "Plugin too slow: {}ms", duration.as_millis());
}
```

## Build Scripts

### Makefile

```makefile
.PHONY: build test install clean release package

PLUGIN_NAME ?= my-plugin
PLUGIN_VERSION ?= $(shell grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Development build
build:
	cargo build --target wasm32-wasi

# Optimized production build
release:
	cargo build --release --target wasm32-wasi
	wasm-strip target/wasm32-wasi/release/$(PLUGIN_NAME).wasm
	wasm-opt -Os target/wasm32-wasi/release/$(PLUGIN_NAME).wasm -o target/wasm32-wasi/release/$(PLUGIN_NAME).optimized.wasm

# Run tests
test:
	cargo test
	./scripts/integration-test.sh

# Install plugin locally for testing
install: release
	uveddi plugin install target/wasm32-wasi/release/$(PLUGIN_NAME).optimized.wasm plugin.toml

# Clean build artifacts
clean:
	cargo clean
	rm -f *.wasm *.zip

# Create distribution package
package: release
	mkdir -p dist
	cp target/wasm32-wasi/release/$(PLUGIN_NAME).optimized.wasm dist/$(PLUGIN_NAME).wasm
	cp plugin.toml dist/
	cp README.md dist/
	cp CHANGELOG.md dist/ 2>/dev/null || true
	cd dist && zip -r ../$(PLUGIN_NAME)-$(PLUGIN_VERSION).zip .
	rm -rf dist

# Validate plugin
validate: release
	uveddi plugin validate target/wasm32-wasi/release/$(PLUGIN_NAME).optimized.wasm plugin.toml
```

These examples provide a solid foundation for developing various types of plugins for Uveddi. Each example demonstrates different aspects of plugin development, from basic pattern matching to advanced AI-powered analysis. The key principles demonstrated across all examples include:

1. **Proper input validation and error handling**
2. **Structured output with meaningful metadata**
3. **Performance considerations and resource limits**
4. **Comprehensive testing strategies**
5. **Clear configuration and documentation**

For more advanced features and integration patterns, refer to the [API Reference](api-reference.md) and [Development Guide](development-guide.md).