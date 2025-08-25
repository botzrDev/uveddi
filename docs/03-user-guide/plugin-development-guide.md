# Plugin Development Guide

## Overview

This guide covers everything you need to know to develop, build, test, and deploy WASM plugins for Uveddi. The plugin system enables you to extend Uveddi's analysis capabilities with custom detectors, rules, and integrations.

## Prerequisites

### Required Tools

```bash
# Rust toolchain with WASM support
rustup target add wasm32-wasi

# Uveddi development environment
cargo install --path . --features=wasm-plugins

# Plugin development tools (optional)
cargo install wasm-pack
cargo install wasmtime-cli
```

### Development Environment

```bash
# Clone Uveddi repository
git clone https://github.com/your-org/uveddi.git
cd uveddi

# Set up development environment
./scripts/setup-dev-environment.sh

# Verify plugin system is available
cargo run --features=wasm-plugins -- plugin list
```

## Quick Start

### 1. Generate Plugin Template

```bash
# Generate a new plugin from template
python scripts/generate-plugin.py my-analyzer

# Navigate to plugin directory
cd plugins/my-analyzer
```

This creates a complete plugin structure:

```
my-analyzer/
├── Cargo.toml          # Rust project configuration
├── Makefile           # Build automation scripts
├── README.md          # Plugin documentation
├── build.rs           # Custom build configuration
├── plugin.toml        # Plugin manifest and metadata
└── src/
    └── lib.rs         # Main plugin implementation
```

### 2. Configure Plugin Manifest

Edit `plugin.toml` to define your plugin's capabilities:

```toml
[plugin]
name = "my-analyzer"
version = "1.0.0"
description = "Custom code analysis plugin"
author = "Your Name <email@example.com>"
license = "MIT"
homepage = "https://github.com/your-org/my-analyzer"

[compatibility]
min_uveddi_version = "1.0.0"
supported_languages = ["rust", "python", "javascript", "typescript"]
requires_tree_sitter = true
requires_ai = false

[capabilities]
# Security permissions
permissions = [
    "ConfigRead",        # Read configuration values
    "TempFileCreate",    # Store analysis results  
    "Logging"           # Plugin logging
]

# Resource limits
max_memory_mb = 32              # Maximum memory usage
max_execution_seconds = 30      # Maximum execution time
max_fuel = 1000000             # Computation limit

[anti_patterns]
# Define what anti-patterns this plugin detects
detects = [
    { name = "CustomIssue", severity = "warning", category = "maintainability" },
    { name = "MyAntiPattern", severity = "error", category = "design" }
]
```

### 3. Implement Plugin Logic

Edit `src/lib.rs` with your plugin implementation:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Data structures for file analysis
#[derive(Deserialize)]
pub struct FileData {
    pub file_path: String,
    pub language: String,
    pub source: String,
    pub has_tree: bool,
    pub syntax_errors: usize,
}

// Plugin analysis results
#[derive(Serialize)]
pub struct PluginResult {
    pub anti_pattern_type_id: Option<i64>,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub message: String,
    pub severity: String,
    pub suggestion: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

// Main plugin entry point
#[no_mangle]
pub extern "C" fn analyze_file(data_ptr: *const u8, data_len: usize) -> *const u8 {
    // Deserialize input data from Uveddi
    let input_slice = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
    let file_data: FileData = match serde_json::from_slice(input_slice) {
        Ok(data) => data,
        Err(_) => return std::ptr::null(),
    };
    
    // Perform analysis
    let results = analyze_file_content(&file_data);
    
    // Serialize results
    let serialized = match serde_json::to_vec(&results) {
        Ok(data) => data,
        Err(_) => return std::ptr::null(),
    };
    
    // Return results pointer (simplified - real implementation needs proper memory management)
    serialized.as_ptr()
}

// Your custom analysis logic
fn analyze_file_content(file_data: &FileData) -> Vec<PluginResult> {
    let mut results = Vec::new();
    
    // Example: Detect long functions (simplified)
    if file_data.source.lines().count() > 100 {
        results.push(PluginResult {
            anti_pattern_type_id: None,
            file_path: file_data.file_path.clone(),
            line_number: Some(1),
            message: "File is too long and may need refactoring".to_string(),
            severity: "warning".to_string(),
            suggestion: "Consider breaking this file into smaller modules".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("line_count".to_string(), 
                          serde_json::Value::Number(
                              file_data.source.lines().count().into()
                          ));
                map
            },
        });
    }
    
    // Example: Detect TODO comments
    for (line_num, line) in file_data.source.lines().enumerate() {
        if line.contains("TODO") || line.contains("FIXME") {
            results.push(PluginResult {
                anti_pattern_type_id: None,
                file_path: file_data.file_path.clone(),
                line_number: Some(line_num as i32 + 1),
                message: "Found TODO or FIXME comment".to_string(),
                severity: "info".to_string(),
                suggestion: "Consider addressing this TODO or creating a tracked issue".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("comment_text".to_string(), 
                              serde_json::Value::String(line.trim().to_string()));
                    map
                },
            });
        }
    }
    
    results
}

// Plugin metadata (called by Uveddi for plugin information)
#[no_mangle]
pub extern "C" fn get_plugin_info() -> *const u8 {
    let info = serde_json::json!({
        "name": "my-analyzer",
        "version": "1.0.0",
        "capabilities": ["file-analysis", "syntax-analysis"]
    });
    
    let serialized = serde_json::to_vec(&info).unwrap();
    serialized.as_ptr()
}
```

### 4. Build and Test Plugin

```bash
# Build the plugin
make build

# Test the plugin locally
make test

# Install in Uveddi for testing
uveddi plugin install my-analyzer.wasm

# Test with Uveddi
uveddi plugin test my-analyzer

# Run analysis with the plugin
cargo run --features=wasm-plugins -- analyze ./test-code --plugins my-analyzer
```

## Advanced Plugin Development

### Using Host Functions

Plugins can access Uveddi's core functionality through host functions:

```rust
// Example: Using host functions (pseudocode - actual implementation varies)
extern "C" {
    fn parse_ast(code_ptr: *const u8, code_len: usize, language_ptr: *const u8, language_len: usize) -> *const u8;
    fn get_config(key_ptr: *const u8, key_len: usize) -> *const u8;
    fn log_message(level: u32, message_ptr: *const u8, message_len: usize);
}

fn advanced_analysis(file_data: &FileData) -> Vec<PluginResult> {
    let mut results = Vec::new();
    
    // Log analysis start
    log_info("Starting advanced analysis");
    
    // Get configuration
    let threshold = get_config_value("complexity_threshold").unwrap_or(10);
    
    // Parse AST for deeper analysis
    if file_data.has_tree {
        let ast = parse_ast_from_host(&file_data.source, &file_data.language);
        // Analyze AST structure...
    }
    
    results
}

fn log_info(message: &str) {
    let message_bytes = message.as_bytes();
    unsafe {
        log_message(1, message_bytes.as_ptr(), message_bytes.len()); // 1 = INFO level
    }
}

fn get_config_value(key: &str) -> Option<i32> {
    let key_bytes = key.as_bytes();
    let result_ptr = unsafe { get_config(key_bytes.as_ptr(), key_bytes.len()) };
    
    if result_ptr.is_null() {
        return None;
    }
    
    // Deserialize config value (implementation details vary)
    // ...
    Some(10) // Placeholder
}
```

### Tree-sitter Integration

For advanced AST-based analysis:

```rust
use serde_json::Value;

fn analyze_with_tree_sitter(file_data: &FileData) -> Vec<PluginResult> {
    let mut results = Vec::new();
    
    if !file_data.has_tree {
        return results; // No AST available
    }
    
    // Query for specific patterns (example: find long functions)
    let query = match file_data.language.as_str() {
        "rust" => r#"
            (function_item
                name: (identifier) @func_name
                body: (block) @func_body
            )
        "#,
        "python" => r#"
            (function_definition
                name: (identifier) @func_name
                body: (block) @func_body
            )
        "#,
        _ => return results,
    };
    
    // Execute tree-sitter query through host function
    let matches = execute_tree_sitter_query(&file_data.file_path, query);
    
    for query_match in matches {
        if let Some(body_capture) = query_match.captures.iter()
            .find(|c| c.name == "func_body") {
            
            let line_count = body_capture.end_row - body_capture.start_row + 1;
            
            if line_count > 50 { // Long function threshold
                results.push(PluginResult {
                    anti_pattern_type_id: Some(1), // God Object type ID
                    file_path: file_data.file_path.clone(),
                    line_number: Some(body_capture.start_row as i32 + 1),
                    message: format!("Function is too long ({} lines)", line_count),
                    severity: "warning".to_string(),
                    suggestion: "Consider breaking this function into smaller functions".to_string(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("line_count".to_string(), Value::Number(line_count.into()));
                        map.insert("function_name".to_string(), 
                                  Value::String("function_name".to_string())); // Would extract from capture
                        map
                    },
                });
            }
        }
    }
    
    results
}

// Placeholder for tree-sitter query execution (actual implementation through host functions)
fn execute_tree_sitter_query(file_path: &str, query: &str) -> Vec<QueryMatch> {
    // Implementation would call host function
    vec![] // Placeholder
}

#[derive(Debug, Clone)]
struct QueryMatch {
    pub pattern: u32,
    pub captures: Vec<QueryCapture>,
}

#[derive(Debug, Clone)]
struct QueryCapture {
    pub index: u32,
    pub name: String,
    pub text: String,
    pub start_row: u32,
    pub end_row: u32,
}
```

### Configuration Integration

Access Uveddi configuration in your plugin:

```rust
fn load_plugin_config() -> PluginConfig {
    PluginConfig {
        complexity_threshold: get_config_int("my_analyzer.complexity_threshold").unwrap_or(10),
        enable_suggestions: get_config_bool("my_analyzer.suggestions").unwrap_or(true),
        exclude_patterns: get_config_string_list("my_analyzer.exclude").unwrap_or_default(),
    }
}

#[derive(Debug)]
struct PluginConfig {
    complexity_threshold: i32,
    enable_suggestions: bool,
    exclude_patterns: Vec<String>,
}

fn get_config_int(key: &str) -> Option<i32> {
    // Implementation using host function
    None // Placeholder
}

fn get_config_bool(key: &str) -> Option<bool> {
    // Implementation using host function  
    None // Placeholder
}

fn get_config_string_list(key: &str) -> Option<Vec<String>> {
    // Implementation using host function
    None // Placeholder
}
```

## Testing and Debugging

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_file_detection() {
        let file_data = FileData {
            file_path: "test.rs".to_string(),
            language: "rust".to_string(),
            source: "fn main() {\n".repeat(150), // 150 lines
            has_tree: false,
            syntax_errors: 0,
        };

        let results = analyze_file_content(&file_data);
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, "warning");
        assert!(results[0].message.contains("too long"));
    }

    #[test]
    fn test_todo_detection() {
        let file_data = FileData {
            file_path: "test.rs".to_string(),
            language: "rust".to_string(),
            source: "fn main() {\n    // TODO: implement this\n}".to_string(),
            has_tree: false,
            syntax_errors: 0,
        };

        let results = analyze_file_content(&file_data);
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, "info");
        assert_eq!(results[0].line_number, Some(2));
    }
}
```

### Integration Testing

```bash
# Test plugin installation
uveddi plugin install my-analyzer.wasm
uveddi plugin list | grep my-analyzer

# Test plugin execution
mkdir test-project
echo "fn very_long_function() { /* ... 200 lines ... */ }" > test-project/test.rs
uveddi analyze test-project --plugins my-analyzer --output-format json

# Verify results contain your plugin's findings
cat analysis-results.json | jq '.issues[] | select(.detector == "my-analyzer")'
```

### Debugging

Enable debug logging for plugin development:

```bash
# Enable debug logging
export RUST_LOG=debug
export UVEDDI_PLUGIN_DEBUG=1

# Run with verbose output
uveddi --verbose plugin test my-analyzer

# Check plugin logs
tail -f ~/.uveddi/logs/plugin-debug.log
```

Plugin debugging techniques:

```rust
// Add debug logging in your plugin
fn analyze_file_content(file_data: &FileData) -> Vec<PluginResult> {
    log_debug(&format!("Analyzing file: {}", file_data.file_path));
    log_debug(&format!("File has {} lines", file_data.source.lines().count()));
    
    let results = perform_analysis(file_data);
    
    log_debug(&format!("Found {} issues", results.len()));
    results
}

fn log_debug(message: &str) {
    let message_bytes = message.as_bytes();
    unsafe {
        log_message(0, message_bytes.as_ptr(), message_bytes.len()); // 0 = DEBUG level
    }
}
```

## Performance Optimization

### Memory Management

```rust
// Efficient string handling
fn analyze_large_files(file_data: &FileData) -> Vec<PluginResult> {
    let mut results = Vec::with_capacity(100); // Pre-allocate capacity
    
    // Process file in chunks to manage memory
    const CHUNK_SIZE: usize = 1000;
    let lines: Vec<&str> = file_data.source.lines().collect();
    
    for chunk in lines.chunks(CHUNK_SIZE) {
        // Process chunk
        let chunk_results = analyze_chunk(chunk);
        results.extend(chunk_results);
        
        // Optional: yield control back to host
        if results.len() > 10000 {
            break; // Prevent excessive result sets
        }
    }
    
    results
}

fn analyze_chunk(lines: &[&str]) -> Vec<PluginResult> {
    // Chunk-specific analysis
    vec![]
}
```

### Caching and Memoization

```rust
use std::collections::HashMap;

// Simple cache for repeated computations
static mut ANALYSIS_CACHE: Option<HashMap<String, Vec<PluginResult>>> = None;

fn analyze_with_caching(file_data: &FileData) -> Vec<PluginResult> {
    let cache_key = format!("{}:{}", file_data.file_path, hash_content(&file_data.source));
    
    // Check cache (unsafe block needed for static mut - consider better alternatives)
    unsafe {
        if ANALYSIS_CACHE.is_none() {
            ANALYSIS_CACHE = Some(HashMap::new());
        }
        
        if let Some(cached_result) = ANALYSIS_CACHE.as_ref().unwrap().get(&cache_key) {
            return cached_result.clone();
        }
    }
    
    // Perform analysis
    let results = analyze_file_content(file_data);
    
    // Cache results
    unsafe {
        ANALYSIS_CACHE.as_mut().unwrap().insert(cache_key, results.clone());
    }
    
    results
}

fn hash_content(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}
```

## Best Practices

### Error Handling

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct PluginError {
    message: String,
    code: String,
    context: Option<String>,
}

fn safe_analyze_file(data_ptr: *const u8, data_len: usize) -> *const u8 {
    let result = std::panic::catch_unwind(|| {
        analyze_file_impl(data_ptr, data_len)
    });
    
    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            let error = PluginError {
                message: "Plugin analysis failed due to internal error".to_string(),
                code: "INTERNAL_ERROR".to_string(),
                context: None,
            };
            
            let serialized = serde_json::to_vec(&error).unwrap_or_default();
            serialized.as_ptr()
        }
    }
}

fn analyze_file_impl(data_ptr: *const u8, data_len: usize) -> *const u8 {
    // Validate input
    if data_ptr.is_null() || data_len == 0 {
        return std::ptr::null();
    }
    
    let input_slice = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
    
    let file_data: FileData = match serde_json::from_slice(input_slice) {
        Ok(data) => data,
        Err(e) => {
            let error = PluginError {
                message: "Failed to deserialize input data".to_string(),
                code: "DESERIALIZATION_ERROR".to_string(),
                context: Some(e.to_string()),
            };
            
            let serialized = serde_json::to_vec(&error).unwrap_or_default();
            return serialized.as_ptr();
        }
    };
    
    // Perform safe analysis
    let results = analyze_file_content(&file_data);
    
    match serde_json::to_vec(&results) {
        Ok(serialized) => serialized.as_ptr(),
        Err(_) => std::ptr::null(),
    }
}
```

### Resource Management

```rust
// Respect resource limits
fn analyze_with_limits(file_data: &FileData) -> Vec<PluginResult> {
    let mut results = Vec::new();
    
    // Respect memory limits
    if file_data.source.len() > 10_000_000 { // 10MB limit
        return vec![PluginResult {
            anti_pattern_type_id: None,
            file_path: file_data.file_path.clone(),
            line_number: None,
            message: "File too large for plugin analysis".to_string(),
            severity: "info".to_string(),
            suggestion: "Consider analyzing smaller files or increasing memory limits".to_string(),
            metadata: HashMap::new(),
        }];
    }
    
    // Respect execution time limits
    let start_time = std::time::Instant::now();
    let max_duration = std::time::Duration::from_secs(25); // Leave buffer for cleanup
    
    for (line_num, line) in file_data.source.lines().enumerate() {
        if start_time.elapsed() > max_duration {
            log_warning("Plugin execution time limit approached, stopping analysis");
            break;
        }
        
        // Perform line analysis
        if let Some(issue) = analyze_line(line, line_num, &file_data.file_path) {
            results.push(issue);
        }
    }
    
    results
}

fn analyze_line(line: &str, line_num: usize, file_path: &str) -> Option<PluginResult> {
    // Implement line-level analysis
    None
}

fn log_warning(message: &str) {
    let message_bytes = message.as_bytes();
    unsafe {
        log_message(2, message_bytes.as_ptr(), message_bytes.len()); // 2 = WARNING level
    }
}
```

## Distribution and Publishing

### Plugin Packaging

```bash
# Build optimized release version
make release

# Package plugin with metadata
tar -czf my-analyzer-v1.0.0.tar.gz my-analyzer.wasm plugin.toml README.md

# Generate plugin hash for verification
sha256sum my-analyzer.wasm > my-analyzer.wasm.sha256
```

### Version Management

Update `plugin.toml` for new releases:

```toml
[plugin]
name = "my-analyzer"
version = "1.1.0"  # Update version
changelog = "Added new detection rules, improved performance"

[compatibility]
min_uveddi_version = "1.0.0"
max_uveddi_version = "2.0.0"  # Optional maximum version
```

### Documentation

Create comprehensive plugin documentation:

```markdown
# My Analyzer Plugin

## Description
Custom analysis plugin for detecting specific code patterns.

## Installation
```bash
uveddi plugin install my-analyzer.wasm
```

## Configuration
Add to your `uveddi.toml`:
```toml
[plugins.my-analyzer]
complexity_threshold = 15
enable_suggestions = true
```

## Detected Issues
- **Long Functions**: Functions exceeding complexity threshold
- **TODO Comments**: Unaddressed TODO/FIXME comments

## Examples
[Include example outputs and configurations]
```

## Troubleshooting

### Common Issues

#### Plugin Won't Load
```bash
# Check plugin format
file my-analyzer.wasm

# Verify plugin manifest
cat plugin.toml

# Check Uveddi logs
uveddi --verbose plugin install my-analyzer.wasm
```

#### Performance Issues
```bash
# Monitor plugin resource usage
uveddi plugin info my-analyzer --stats

# Profile plugin execution
UVEDDI_PLUGIN_PROFILE=1 uveddi analyze ./code --plugins my-analyzer
```

#### Permission Errors
```toml
# Update plugin.toml permissions
[capabilities]
permissions = [
    "ConfigRead",        # Add missing permissions
    "TempFileCreate",    
    "Logging"
]
```

### Debug Mode

Enable comprehensive debugging:

```bash
# Set environment variables
export RUST_LOG=trace
export UVEDDI_PLUGIN_DEBUG=1
export UVEDDI_PLUGIN_TRACE=1

# Run with full debugging
uveddi --verbose analyze ./code --plugins my-analyzer 2>&1 | tee debug.log
```

## Next Steps

1. **Explore Examples**: Check `/plugins/` directory for example implementations
2. **Join Community**: Participate in plugin development discussions
3. **Contribute**: Submit plugins to the community registry
4. **Advanced Features**: Explore AI integration, multi-language support
5. **Performance**: Optimize plugins for large codebases

## Resources

- [Plugin API Reference](../07-reference/plugin-api-reference.md)
- [Security Guidelines](../11-security/plugin-security.md)  
- [Performance Best Practices](../05-examples/plugin-performance.md)
- [Community Plugins](https://github.com/your-org/uveddi-plugins)