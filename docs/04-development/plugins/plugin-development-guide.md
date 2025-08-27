# Plugin Development Guide

## Overview

This guide covers developing WebAssembly plugins for Uveddi's analysis engine. Plugins enable custom analysis detectors that integrate seamlessly with Uveddi's analysis pipeline while running in a secure sandboxed environment.

## Quick Start

### 1. Generate Plugin Template

```bash
# Generate a new plugin from template
scripts/generate-plugin.py my-analyzer

# This creates:
plugins/my-analyzer/
├── Cargo.toml          # Rust project configuration
├── Makefile           # Build automation
├── README.md          # Plugin documentation
├── build.rs           # Custom build scripts
├── plugin.toml        # Plugin manifest
└── src/
    └── lib.rs         # Plugin implementation
```

### 2. Basic Plugin Implementation

```rust
// src/lib.rs
use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize)]
pub struct PluginAnalysisResult {
    pub plugin_name: String,
    pub issues: Vec<PluginIssue>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    // Deserialize input
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => {
            let error_result = PluginAnalysisResult {
                plugin_name: "my-analyzer".to_string(),
                issues: vec![],
                metadata: [("error".to_string(), e.to_string())].into(),
            };
            return serde_json::to_vec(&error_result).unwrap_or_default();
        }
    };

    // Perform analysis
    let mut issues = Vec::new();
    
    // Example: Check for TODO comments
    for (line_num, line) in input.content.lines().enumerate() {
        if line.to_lowercase().contains("todo") {
            issues.push(PluginIssue {
                issue_type: "TODO_COMMENT".to_string(),
                severity: "INFO".to_string(),
                message: "TODO comment found".to_string(),
                file_path: input.file_path.clone(),
                line_number: Some(line_num as u32 + 1),
                column: Some(line.find("TODO").unwrap_or(0) as u32),
                suggestion: Some("Consider creating a proper issue for this TODO".to_string()),
            });
        }
    }

    let result = PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues,
        metadata: [("analyzed_lines".to_string(), input.content.lines().count().to_string())].into(),
    };

    serde_json::to_vec(&result).unwrap_or_default()
}

#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8> {
    let info = serde_json::json!({
        "name": "my-analyzer",
        "version": "1.0.0",
        "description": "Example plugin for demonstration",
        "supported_languages": ["rust", "python", "javascript", "typescript"],
        "capabilities": ["static_analysis"]
    });
    
    serde_json::to_vec(&info).unwrap_or_default()
}
```

### 3. Plugin Manifest Configuration

```toml
# plugin.toml
[plugin]
name = "my-analyzer"
version = "1.0.0"
description = "Custom analysis plugin for detecting TODO comments"
author = "Your Name <your.email@example.com>"
license = "MIT"
homepage = "https://github.com/yourname/my-analyzer"
repository = "https://github.com/yourname/my-analyzer"

[capabilities]
# Security permissions required by the plugin
permissions = ["ConfigRead", "Logging"]

# Resource limits
max_memory_mb = 32
max_execution_seconds = 30
fuel_limit = 2000000

# Performance settings
enable_caching = true
cache_ttl_seconds = 3600

[dependencies]
# Minimum Uveddi version required
min_uveddi_version = "0.9.0"

# Required host functions
requires_tree_sitter = false
requires_database_access = false
requires_network_access = false

# Supported file types and languages
supported_languages = ["rust", "python", "javascript", "typescript"]
supported_extensions = [".rs", ".py", ".js", ".ts", ".jsx", ".tsx"]

[detection]
# Types of issues this plugin can detect
anti_pattern_types = ["CODE_SMELL", "MAINTAINABILITY"]
issue_categories = ["TODO_COMMENTS", "CODE_QUALITY"]
severity_levels = ["INFO", "WARNING", "ERROR"]

[integration]
# Integration settings
run_in_parallel = true
depends_on_plugins = []
conflicts_with_plugins = []
priority = 100  # Higher numbers run first
```

### 4. Build and Test

```bash
# Navigate to plugin directory
cd plugins/my-analyzer

# Build the plugin
make build

# Test the plugin
make test

# Install and test with Uveddi
cargo build --release --target wasm32-wasi
uveddi plugin install target/wasm32-wasi/release/my_analyzer.wasm

# Test plugin functionality
uveddi plugin test my-analyzer --verbose
```

## Advanced Plugin Development

### Using Host Functions

Plugins can access Uveddi's core services through host functions:

```rust
// Host function bindings (add to your plugin)
extern "C" {
    fn host_parse_ast(code_ptr: *const u8, code_len: usize, lang_ptr: *const u8, lang_len: usize) -> u64;
    fn host_log_message(level: u32, msg_ptr: *const u8, msg_len: usize);
    fn host_get_config(key_ptr: *const u8, key_len: usize) -> u64;
}

// Wrapper functions for easier use
pub fn parse_ast(code: &str, language: &str) -> Option<String> {
    unsafe {
        let handle = host_parse_ast(
            code.as_ptr(), code.len(),
            language.as_ptr(), language.len()
        );
        // Handle result extraction (simplified)
        if handle != 0 {
            // Extract AST data from host memory
            Some("parsed_ast_data".to_string())
        } else {
            None
        }
    }
}

pub fn log_info(message: &str) {
    unsafe {
        host_log_message(1, message.as_ptr(), message.len());
    }
}

pub fn get_config_value(key: &str) -> Option<String> {
    unsafe {
        let handle = host_get_config(key.as_ptr(), key.len());
        // Extract config value (simplified)
        if handle != 0 {
            Some("config_value".to_string())
        } else {
            None
        }
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = serde_json::from_slice(file_data).unwrap();
    
    // Use host functions
    log_info(&format!("Analyzing file: {}", input.file_path));
    
    if let Some(ast) = parse_ast(&input.content, &input.language) {
        // Analyze using AST
        log_info("AST parsing successful");
    }
    
    if let Some(threshold) = get_config_value("my_analyzer.threshold") {
        log_info(&format!("Using threshold: {}", threshold));
    }
    
    // Continue with analysis...
    serde_json::to_vec(&PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues: vec![],
        metadata: std::collections::HashMap::new(),
    }).unwrap_or_default()
}
```

### Tree-Sitter Integration

For advanced AST analysis, plugins can use tree-sitter through host functions:

```rust
pub fn analyze_with_tree_sitter(content: &str, language: &str) -> Vec<PluginIssue> {
    let mut issues = Vec::new();
    
    // Parse AST using host function
    if let Some(ast_json) = parse_ast(content, language) {
        // Query for specific patterns
        let query = match language {
            "rust" => "(function_item name: (identifier) @func-name)",
            "python" => "(function_definition name: (identifier) @func-name)",
            "javascript" | "typescript" => "(function_declaration name: (identifier) @func-name)",
            _ => return issues,
        };
        
        // Execute tree-sitter query through host function
        if let Some(matches) = execute_query(&ast_json, query) {
            // Process query results
            for match_data in matches {
                if is_problematic_function(&match_data) {
                    issues.push(PluginIssue {
                        issue_type: "COMPLEX_FUNCTION".to_string(),
                        severity: "WARNING".to_string(),
                        message: "Function complexity is too high".to_string(),
                        file_path: "current_file".to_string(),
                        line_number: match_data.line,
                        column: match_data.column,
                        suggestion: Some("Consider breaking this function into smaller parts".to_string()),
                    });
                }
            }
        }
    }
    
    issues
}

fn execute_query(ast_json: &str, query: &str) -> Option<Vec<QueryMatch>> {
    // Implementation would use host function for tree-sitter queries
    // Simplified for example
    None
}

struct QueryMatch {
    line: Option<u32>,
    column: Option<u32>,
    text: String,
}

fn is_problematic_function(match_data: &QueryMatch) -> bool {
    // Implement your analysis logic
    match_data.text.len() > 100
}
```

### Plugin Configuration

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct PluginConfig {
    threshold: f64,
    ignore_patterns: Vec<String>,
    max_issues_per_file: usize,
    enable_suggestions: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            ignore_patterns: vec!["test".to_string(), "vendor".to_string()],
            max_issues_per_file: 50,
            enable_suggestions: true,
        }
    }
}

pub fn load_config() -> PluginConfig {
    // Try to load from host configuration
    if let Some(config_json) = get_config_value("my_analyzer.config") {
        serde_json::from_str(&config_json).unwrap_or_default()
    } else {
        PluginConfig::default()
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let config = load_config();
    let input: PluginFileInput = serde_json::from_slice(file_data).unwrap();
    
    // Use configuration in analysis
    if should_ignore_file(&input.file_path, &config.ignore_patterns) {
        return serde_json::to_vec(&PluginAnalysisResult {
            plugin_name: "my-analyzer".to_string(),
            issues: vec![],
            metadata: [("ignored".to_string(), "true".to_string())].into(),
        }).unwrap_or_default();
    }
    
    // Continue with configurable analysis...
    let mut issues = perform_analysis(&input, &config);
    
    // Limit issues per file
    issues.truncate(config.max_issues_per_file);
    
    serde_json::to_vec(&PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues,
        metadata: std::collections::HashMap::new(),
    }).unwrap_or_default()
}

fn should_ignore_file(file_path: &str, ignore_patterns: &[String]) -> bool {
    ignore_patterns.iter().any(|pattern| file_path.contains(pattern))
}

fn perform_analysis(input: &PluginFileInput, config: &PluginConfig) -> Vec<PluginIssue> {
    // Your analysis logic with configuration
    vec![]
}
```

## Testing Plugins

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_file_with_todos() {
        let input = PluginFileInput {
            file_path: "test.rs".to_string(),
            content: "// TODO: Fix this\nfn main() {}".to_string(),
            language: "rust".to_string(),
        };
        
        let input_bytes = serde_json::to_vec(&input).unwrap();
        let result_bytes = analyze_file(&input_bytes);
        
        let result: PluginAnalysisResult = serde_json::from_slice(&result_bytes).unwrap();
        
        assert_eq!(result.plugin_name, "my-analyzer");
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].issue_type, "TODO_COMMENT");
        assert_eq!(result.issues[0].line_number, Some(1));
    }
    
    #[test]
    fn test_plugin_info() {
        let info_bytes = get_plugin_info();
        let info: serde_json::Value = serde_json::from_slice(&info_bytes).unwrap();
        
        assert_eq!(info["name"], "my-analyzer");
        assert_eq!(info["version"], "1.0.0");
        assert!(info["supported_languages"].as_array().unwrap().contains(&"rust".into()));
    }
    
    #[test]
    fn test_config_loading() {
        let config = PluginConfig::default();
        assert_eq!(config.threshold, 0.8);
        assert!(config.enable_suggestions);
    }
}
```

### Integration Testing

```bash
#!/bin/bash
# test-plugin.sh

set -e

echo "Building plugin..."
cargo build --release --target wasm32-wasi

echo "Installing plugin..."
uveddi plugin install target/wasm32-wasi/release/my_analyzer.wasm

echo "Testing plugin functionality..."
uveddi plugin test my-analyzer --test-file test-data/sample.rs --verbose

echo "Running analysis with plugin..."
uveddi analyze test-data/ --plugins my-analyzer --output-format json --output test-results.json

echo "Verifying results..."
if jq -e '.issues[] | select(.detector == "my-analyzer")' test-results.json > /dev/null; then
    echo "✅ Plugin integration successful"
else
    echo "❌ Plugin integration failed"
    exit 1
fi

echo "Cleaning up..."
rm -f test-results.json
```

### Test Data Setup

```bash
# Create test data directory
mkdir -p test-data

# Sample Rust file with issues
cat > test-data/sample.rs << 'EOF'
// TODO: Refactor this function
fn complex_function() {
    // TODO: Add error handling
    let mut data = Vec::new();
    for i in 0..100 {
        data.push(i * 2);
    }
    println!("{:?}", data);
}

fn main() {
    complex_function();
}
EOF

# Sample Python file
cat > test-data/sample.py << 'EOF'
# TODO: Add docstrings
def process_data(items):
    # TODO: Optimize this loop
    result = []
    for item in items:
        if item > 0:
            result.append(item * 2)
    return result

if __name__ == "__main__":
    data = [1, 2, 3, 4, 5]
    print(process_data(data))
EOF
```

## Performance Optimization

### Memory Management

```rust
// Efficient memory usage patterns
use std::collections::HashMap;

pub struct EfficientAnalyzer {
    issue_cache: HashMap<String, Vec<PluginIssue>>,
    pattern_cache: Vec<regex::Regex>,
}

impl EfficientAnalyzer {
    pub fn new() -> Self {
        Self {
            issue_cache: HashMap::with_capacity(100),
            pattern_cache: Vec::with_capacity(10),
        }
    }
    
    pub fn analyze_efficiently(&mut self, input: &PluginFileInput) -> Vec<PluginIssue> {
        // Check cache first
        if let Some(cached) = self.issue_cache.get(&input.file_path) {
            return cached.clone();
        }
        
        let mut issues = Vec::with_capacity(20); // Pre-allocate
        
        // Process in chunks to manage memory
        let chunk_size = 1000;
        for chunk in input.content.lines().collect::<Vec<_>>().chunks(chunk_size) {
            for (local_line_num, line) in chunk.iter().enumerate() {
                // Process line efficiently
                if let Some(issue) = self.check_line_fast(line, local_line_num) {
                    issues.push(issue);
                }
            }
        }
        
        // Cache results
        self.issue_cache.insert(input.file_path.clone(), issues.clone());
        issues
    }
    
    fn check_line_fast(&self, line: &str, line_num: usize) -> Option<PluginIssue> {
        // Fast pattern matching without allocation
        if line.contains("TODO") || line.contains("FIXME") || line.contains("HACK") {
            Some(PluginIssue {
                issue_type: "CODE_SMELL".to_string(),
                severity: "INFO".to_string(),
                message: "Code comment indicates technical debt".to_string(),
                file_path: String::new(), // Will be set by caller
                line_number: Some(line_num as u32 + 1),
                column: None,
                suggestion: Some("Consider addressing this technical debt".to_string()),
            })
        } else {
            None
        }
    }
}

// Global analyzer instance to maintain state
static mut ANALYZER: Option<EfficientAnalyzer> = None;
static INIT: std::sync::Once = std::sync::Once::new();

fn get_analyzer() -> &'static mut EfficientAnalyzer {
    unsafe {
        INIT.call_once(|| {
            ANALYZER = Some(EfficientAnalyzer::new());
        });
        ANALYZER.as_mut().unwrap()
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(_) => return Vec::new(),
    };
    
    let analyzer = get_analyzer();
    let mut issues = analyzer.analyze_efficiently(&input);
    
    // Set file path for all issues
    for issue in &mut issues {
        issue.file_path = input.file_path.clone();
    }
    
    let result = PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues,
        metadata: HashMap::new(),
    };
    
    serde_json::to_vec(&result).unwrap_or_default()
}
```

### Parallel Processing

```rust
// Note: WASM doesn't support threads, but you can process data efficiently
use std::collections::VecDeque;

pub fn process_content_in_batches(content: &str) -> Vec<PluginIssue> {
    let lines: Vec<&str> = content.lines().collect();
    let batch_size = 100;
    let mut issues = Vec::new();
    
    // Process in batches to reduce memory pressure
    for (batch_idx, batch) in lines.chunks(batch_size).enumerate() {
        let batch_issues = process_batch(batch, batch_idx * batch_size);
        issues.extend(batch_issues);
        
        // Yield control periodically (simulated)
        if batch_idx % 10 == 0 {
            log_info(&format!("Processed {} batches", batch_idx));
        }
    }
    
    issues
}

fn process_batch(lines: &[&str], base_line_num: usize) -> Vec<PluginIssue> {
    let mut issues = Vec::new();
    
    for (idx, line) in lines.iter().enumerate() {
        if let Some(issue) = analyze_line(line, base_line_num + idx) {
            issues.push(issue);
        }
    }
    
    issues
}

fn analyze_line(line: &str, line_num: usize) -> Option<PluginIssue> {
    // Your analysis logic
    None
}
```

## Security Considerations

### Input Validation

```rust
use std::collections::HashSet;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_CONTENT_LENGTH: usize = 1024 * 1024; // 1MB
const ALLOWED_LANGUAGES: &[&str] = &["rust", "python", "javascript", "typescript"];

pub fn validate_input(input: &PluginFileInput) -> Result<(), String> {
    // Check file path
    if input.file_path.is_empty() || input.file_path.len() > 1000 {
        return Err("Invalid file path".to_string());
    }
    
    // Prevent path traversal
    if input.file_path.contains("..") || input.file_path.contains("//") {
        return Err("Path traversal detected".to_string());
    }
    
    // Check content size
    if input.content.len() > MAX_CONTENT_LENGTH {
        return Err("Content too large".to_string());
    }
    
    // Validate language
    if !ALLOWED_LANGUAGES.contains(&input.language.as_str()) {
        return Err(format!("Unsupported language: {}", input.language));
    }
    
    // Check for null bytes
    if input.content.contains('\0') || input.file_path.contains('\0') {
        return Err("Null bytes not allowed".to_string());
    }
    
    Ok(())
}

pub fn sanitize_output(mut result: PluginAnalysisResult) -> PluginAnalysisResult {
    // Limit number of issues
    const MAX_ISSUES: usize = 1000;
    result.issues.truncate(MAX_ISSUES);
    
    // Sanitize issue messages
    for issue in &mut result.issues {
        issue.message = sanitize_message(&issue.message);
        if let Some(ref mut suggestion) = issue.suggestion {
            *suggestion = sanitize_message(suggestion);
        }
    }
    
    // Limit metadata
    const MAX_METADATA_ENTRIES: usize = 100;
    if result.metadata.len() > MAX_METADATA_ENTRIES {
        result.metadata.clear();
        result.metadata.insert("warning".to_string(), "Metadata truncated".to_string());
    }
    
    result
}

fn sanitize_message(message: &str) -> String {
    // Remove potentially dangerous content
    message
        .chars()
        .filter(|&c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .take(500) // Limit message length
        .collect()
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    // Validate input size
    if file_data.len() > MAX_FILE_SIZE {
        let error_result = PluginAnalysisResult {
            plugin_name: "my-analyzer".to_string(),
            issues: vec![],
            metadata: [("error".to_string(), "Input too large".to_string())].into(),
        };
        return serde_json::to_vec(&error_result).unwrap_or_default();
    }
    
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => {
            let error_result = PluginAnalysisResult {
                plugin_name: "my-analyzer".to_string(),
                issues: vec![],
                metadata: [("error".to_string(), format!("JSON parse error: {}", e))].into(),
            };
            return serde_json::to_vec(&error_result).unwrap_or_default();
        }
    };
    
    // Validate input
    if let Err(error) = validate_input(&input) {
        let error_result = PluginAnalysisResult {
            plugin_name: "my-analyzer".to_string(),
            issues: vec![],
            metadata: [("error".to_string(), error)].into(),
        };
        return serde_json::to_vec(&error_result).unwrap_or_default();
    }
    
    // Perform analysis
    let mut result = PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues: perform_secure_analysis(&input),
        metadata: std::collections::HashMap::new(),
    };
    
    // Sanitize output
    result = sanitize_output(result);
    
    serde_json::to_vec(&result).unwrap_or_default()
}

fn perform_secure_analysis(input: &PluginFileInput) -> Vec<PluginIssue> {
    // Your secure analysis implementation
    vec![]
}
```

## Debugging and Troubleshooting

### Logging and Diagnostics

```rust
// Logging utilities
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

pub fn log(level: LogLevel, message: &str) {
    unsafe {
        host_log_message(level as u32, message.as_ptr(), message.len());
    }
}

macro_rules! log_debug {
    ($($arg:tt)*) => {
        log(LogLevel::Debug, &format!($($arg)*));
    };
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        log(LogLevel::Info, &format!($($arg)*));
    };
}

macro_rules! log_warn {
    ($($arg:tt)*) => {
        log(LogLevel::Warn, &format!($($arg)*));
    };
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        log(LogLevel::Error, &format!($($arg)*));
    };
}

// Performance measurement
pub struct Timer {
    start: std::time::SystemTime,
    name: String,
}

impl Timer {
    pub fn start(name: &str) -> Self {
        log_debug!("Timer started: {}", name);
        Self {
            start: std::time::SystemTime::now(),
            name: name.to_string(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Ok(duration) = self.start.elapsed() {
            log_debug!("Timer finished: {} - {}ms", self.name, duration.as_millis());
        }
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let _timer = Timer::start("analyze_file");
    
    log_info!("Starting analysis, input size: {} bytes", file_data.len());
    
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => {
            log_info!("Parsed input for file: {}", input.file_path);
            input
        }
        Err(e) => {
            log_error!("Failed to parse input: {}", e);
            return create_error_result("JSON parse error");
        }
    };
    
    log_debug!("File content length: {} chars", input.content.len());
    log_debug!("Language: {}", input.language);
    
    let issues = match perform_analysis_with_logging(&input) {
        Ok(issues) => {
            log_info!("Analysis completed, found {} issues", issues.len());
            issues
        }
        Err(e) => {
            log_error!("Analysis failed: {}", e);
            vec![]
        }
    };
    
    let result = PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues,
        metadata: [
            ("analyzed_at".to_string(), format!("{:?}", std::time::SystemTime::now())),
            ("content_length".to_string(), input.content.len().to_string()),
        ].into(),
    };
    
    match serde_json::to_vec(&result) {
        Ok(output) => {
            log_info!("Serialized result, output size: {} bytes", output.len());
            output
        }
        Err(e) => {
            log_error!("Failed to serialize result: {}", e);
            create_error_result("Serialization error")
        }
    }
}

fn perform_analysis_with_logging(input: &PluginFileInput) -> Result<Vec<PluginIssue>, String> {
    let _timer = Timer::start("perform_analysis");
    
    // Your analysis logic with logging
    let line_count = input.content.lines().count();
    log_debug!("Processing {} lines", line_count);
    
    let mut issues = Vec::new();
    for (line_num, line) in input.content.lines().enumerate() {
        if line.contains("TODO") {
            log_debug!("Found TODO at line {}", line_num + 1);
            issues.push(PluginIssue {
                issue_type: "TODO_COMMENT".to_string(),
                severity: "INFO".to_string(),
                message: "TODO comment found".to_string(),
                file_path: input.file_path.clone(),
                line_number: Some(line_num as u32 + 1),
                column: None,
                suggestion: Some("Consider creating a proper issue".to_string()),
            });
        }
    }
    
    Ok(issues)
}

fn create_error_result(error: &str) -> Vec<u8> {
    let error_result = PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues: vec![],
        metadata: [("error".to_string(), error.to_string())].into(),
    };
    serde_json::to_vec(&error_result).unwrap_or_default()
}
```

### Common Issues and Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| Plugin fails to load | Invalid WASM binary | Check build target is `wasm32-wasi` |
| `analyze_file` not found | Missing export name | Add `#[export_name = "analyze_file"]` |
| Deserialization errors | Mismatched data structures | Ensure input/output structures match Uveddi's expectations |
| Memory allocation failures | Too much memory usage | Implement efficient memory management |
| Host function errors | Incorrect function signatures | Check host function bindings match Uveddi's API |
| Permission denied | Insufficient plugin permissions | Update plugin manifest with required permissions |

## Advanced Patterns

### State Management

```rust
use std::sync::{Mutex, Once};
use std::collections::HashMap;

// Global state management (thread-safe)
struct PluginState {
    file_cache: HashMap<String, String>,
    analysis_count: u64,
    config: PluginConfig,
}

static mut PLUGIN_STATE: Option<Mutex<PluginState>> = None;
static INIT_STATE: Once = Once::new();

fn get_state() -> &'static Mutex<PluginState> {
    unsafe {
        INIT_STATE.call_once(|| {
            PLUGIN_STATE = Some(Mutex::new(PluginState {
                file_cache: HashMap::new(),
                analysis_count: 0,
                config: PluginConfig::default(),
            }));
        });
        PLUGIN_STATE.as_ref().unwrap()
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = serde_json::from_slice(file_data).unwrap();
    
    // Access global state
    let state = get_state();
    let mut state_guard = state.lock().unwrap();
    
    // Check cache
    if let Some(cached_hash) = state_guard.file_cache.get(&input.file_path) {
        let current_hash = calculate_hash(&input.content);
        if cached_hash == &current_hash {
            log_info!("Using cached analysis for {}", input.file_path);
            // Return cached result (simplified)
        }
    }
    
    // Update analysis count
    state_guard.analysis_count += 1;
    log_info!("Analysis #{} for file: {}", state_guard.analysis_count, input.file_path);
    
    // Perform analysis using state config
    let issues = analyze_with_config(&input, &state_guard.config);
    
    // Update cache
    state_guard.file_cache.insert(input.file_path.clone(), calculate_hash(&input.content));
    
    drop(state_guard);
    
    serde_json::to_vec(&PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues,
        metadata: HashMap::new(),
    }).unwrap_or_default()
}

fn calculate_hash(content: &str) -> String {
    // Simple hash implementation
    format!("{:x}", content.len() ^ content.chars().map(|c| c as u32).sum::<u32>())
}

fn analyze_with_config(input: &PluginFileInput, config: &PluginConfig) -> Vec<PluginIssue> {
    // Analysis using configuration
    vec![]
}
```

### Plugin Composition

```rust
// Multi-stage analysis pattern
pub struct AnalysisStage {
    name: String,
    processor: fn(&str) -> Vec<PluginIssue>,
}

static ANALYSIS_STAGES: &[AnalysisStage] = &[
    AnalysisStage {
        name: "syntax_check".to_string(),
        processor: check_syntax,
    },
    AnalysisStage {
        name: "pattern_detection".to_string(),
        processor: detect_patterns,
    },
    AnalysisStage {
        name: "complexity_analysis".to_string(),
        processor: analyze_complexity,
    },
];

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = serde_json::from_slice(file_data).unwrap();
    let mut all_issues = Vec::new();
    let mut metadata = HashMap::new();
    
    // Run analysis stages
    for stage in ANALYSIS_STAGES {
        let _timer = Timer::start(&format!("stage_{}", stage.name));
        log_info!("Running analysis stage: {}", stage.name);
        
        let stage_issues = (stage.processor)(&input.content);
        log_info!("Stage {} found {} issues", stage.name, stage_issues.len());
        
        metadata.insert(
            format!("{}_issues", stage.name),
            stage_issues.len().to_string()
        );
        
        all_issues.extend(stage_issues);
    }
    
    // Post-process and deduplicate
    all_issues = deduplicate_issues(all_issues);
    
    serde_json::to_vec(&PluginAnalysisResult {
        plugin_name: "my-analyzer".to_string(),
        issues: all_issues,
        metadata,
    }).unwrap_or_default()
}

fn check_syntax(content: &str) -> Vec<PluginIssue> {
    // Syntax checking logic
    vec![]
}

fn detect_patterns(content: &str) -> Vec<PluginIssue> {
    // Pattern detection logic
    vec![]
}

fn analyze_complexity(content: &str) -> Vec<PluginIssue> {
    // Complexity analysis logic
    vec![]
}

fn deduplicate_issues(mut issues: Vec<PluginIssue>) -> Vec<PluginIssue> {
    // Sort by location
    issues.sort_by_key(|issue| (issue.line_number, issue.column));
    
    // Remove duplicates
    issues.dedup_by(|a, b| {
        a.line_number == b.line_number &&
        a.column == b.column &&
        a.issue_type == b.issue_type
    });
    
    issues
}
```

## Deployment and Distribution

### Building for Production

```makefile
# Makefile
.PHONY: build test install clean release

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
	./test-plugin.sh

# Install plugin locally
install: release
	uveddi plugin install target/wasm32-wasi/release/$(PLUGIN_NAME).optimized.wasm

# Clean build artifacts
clean:
	cargo clean
	rm -f test-results.json

# Package for distribution
package: release
	mkdir -p dist
	cp target/wasm32-wasi/release/$(PLUGIN_NAME).optimized.wasm dist/$(PLUGIN_NAME).wasm
	cp plugin.toml dist/
	cp README.md dist/
	tar -czf dist/$(PLUGIN_NAME)-$(VERSION).tar.gz -C dist .

PLUGIN_NAME := my_analyzer
VERSION := $(shell grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
```

### Distribution Package

```toml
# distribution.toml
[package]
name = "my-analyzer"
version = "1.0.0"
description = "Advanced code analysis plugin"
author = "Your Name <email@example.com>"
license = "MIT"
homepage = "https://github.com/yourname/my-analyzer"
repository = "https://github.com/yourname/my-analyzer"
keywords = ["uveddi", "plugin", "analysis", "code-quality"]

[files]
binary = "my-analyzer.wasm"
manifest = "plugin.toml"
documentation = "README.md"
changelog = "CHANGELOG.md"

[compatibility]
min_uveddi_version = "0.9.0"
tested_versions = ["0.9.0", "0.9.1"]

[verification]
checksum_sha256 = "abc123..."
signature_file = "signature.sig"
public_key = "pubkey.pem"
```

## Best Practices Summary

### Code Quality
- ✅ Use proper error handling with Result types
- ✅ Implement comprehensive input validation
- ✅ Add detailed logging and diagnostics
- ✅ Write extensive unit and integration tests
- ✅ Follow Rust idioms and conventions

### Security
- ✅ Validate all inputs thoroughly
- ✅ Sanitize all outputs
- ✅ Use minimal required permissions
- ✅ Implement resource limits
- ✅ Avoid unsafe operations when possible

### Performance
- ✅ Pre-allocate collections when size is known
- ✅ Use efficient data structures
- ✅ Implement caching for repeated operations
- ✅ Process data in batches for large files
- ✅ Monitor memory usage and optimize

### Integration
- ✅ Follow Uveddi's plugin interface exactly
- ✅ Use consistent error reporting patterns
- ✅ Provide meaningful metadata
- ✅ Implement proper plugin information
- ✅ Test with real-world codebases

### Maintenance
- ✅ Keep dependencies up to date
- ✅ Version your plugin releases
- ✅ Maintain comprehensive documentation
- ✅ Provide migration guides for updates
- ✅ Monitor plugin performance in production

This guide provides a comprehensive foundation for developing high-quality WASM plugins for Uveddi's analysis engine.