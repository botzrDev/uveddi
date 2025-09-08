# Plugin API Reference

## Overview

This document provides a comprehensive reference for Uveddi's Plugin API, including host functions, data structures, security model, and integration patterns. The production-ready WebAssembly plugin system enables secure, performant extensibility through capability-based security and comprehensive host function APIs.

**Plugin System Status**: ✅ **PRODUCTION READY**
- Complete CLI integration with `uveddi plugin` commands
- Full host functions API for core system access
- WASM runtime with security sandboxing and resource limits
- Seamless analysis pipeline integration
- Comprehensive development tooling and documentation

## Host Functions API

### Core Analysis Functions

#### `parse_ast(code: &str, language: &str) -> Result<AstNode, PluginError>`

Parse source code into an Abstract Syntax Tree using Uveddi's tree-sitter implementation.

**Parameters:**
- `code: &str` - Source code to parse
- `language: &str` - Programming language ("rust", "python", "javascript", "typescript")

**Returns:**
- `Result<AstNode, PluginError>` - Parsed AST root node or error

**Permissions Required:** `ConfigRead`

**Example Usage:**
```rust
let ast = parse_ast(source_code, "rust")?;
log_info(&format!("AST has {} children", ast.children.len()));
```

**Error Conditions:**
- Invalid language specified
- Syntax errors in source code (returns partial AST)
- Memory allocation failure
- Permission denied

---

#### `get_analysis_context(file_path: &str) -> Result<AnalysisContext, PluginError>`

Retrieve analysis context and metadata for a specific file.

**Parameters:**
- `file_path: &str` - Path to the file being analyzed

**Returns:**
- `Result<AnalysisContext, PluginError>` - File context information

**Permissions Required:** `ConfigRead`

**Example Usage:**
```rust
let context = get_analysis_context("/path/to/file.rs")?;
if context.ast_available {
    log_info("AST is available for this file");
}
```

**AnalysisContext Structure:**
```rust
pub struct AnalysisContext {
    pub file_path: String,
    pub language: String,
    pub ast_available: bool,
    pub line_count: u32,
    pub metadata: HashMap<String, String>,
}
```

---

#### `execute_tree_sitter_query(file_path: &str, query: &str) -> Result<Vec<QueryMatch>, PluginError>`

Execute a tree-sitter query against a cached AST.

**Parameters:**
- `file_path: &str` - Path to the file with cached AST
- `query: &str` - Tree-sitter query string

**Returns:**
- `Result<Vec<QueryMatch>, PluginError>` - Query match results

**Permissions Required:** `ConfigRead`

**Example Usage:**
```rust
let query = r#"
    (function_item
        name: (identifier) @func_name
        body: (block) @func_body
    )
"#;
let matches = execute_tree_sitter_query("/path/to/file.rs", query)?;

for query_match in matches {
    for capture in query_match.captures {
        log_info(&format!("Found {}: {}", capture.name, capture.text));
    }
}
```

**QueryMatch Structure:**
```rust
pub struct QueryMatch {
    pub pattern: u32,
    pub captures: Vec<QueryCapture>,
}

pub struct QueryCapture {
    pub index: u32,
    pub name: String,
    pub text: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_row: u32,
    pub start_column: u32,
    pub end_row: u32,
    pub end_column: u32,
}
```

### Database Functions

#### `store_plugin_results(results: &[PluginIssue], analysis_run_id: i32) -> Result<(), PluginError>`

Store plugin analysis results in Uveddi's database.

**Parameters:**
- `results: &[PluginIssue]` - Array of issues found by the plugin
- `analysis_run_id: i32` - ID of the current analysis run

**Returns:**
- `Result<(), PluginError>` - Success or error

**Permissions Required:** `TempFileCreate`

**Example Usage:**
```rust
let issues = vec![
    PluginIssue {
        anti_pattern_type_id: Some(1),
        file_path: "src/main.rs".to_string(),
        line_number: Some(42),
        message: "Function is too complex".to_string(),
        severity: "warning".to_string(),
        suggestion: "Consider breaking into smaller functions".to_string(),
        metadata: HashMap::new(),
    }
];

store_plugin_results(&issues, run_id)?;
```

**PluginIssue Structure:**
```rust
pub struct PluginIssue {
    pub anti_pattern_type_id: Option<i64>,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub message: String,
    pub severity: String,    // "error", "warning", "info"
    pub suggestion: String,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

---

#### `get_analysis_run(run_id: i32) -> Result<Option<AnalysisRun>, PluginError>`

Retrieve information about an analysis run.

**Parameters:**
- `run_id: i32` - Analysis run identifier

**Returns:**
- `Result<Option<AnalysisRun>, PluginError>` - Analysis run information or None if not found

**Permissions Required:** `ConfigRead`

**Example Usage:**
```rust
if let Some(run) = get_analysis_run(run_id)? {
    log_info(&format!("Analysis started at: {}", run.start_time));
}
```

### Configuration Functions

#### `get_project_config(key: &str) -> Result<Option<String>, PluginError>`

Retrieve a configuration value from Uveddi's configuration system.

**Parameters:**
- `key: &str` - Configuration key (e.g., "my_plugin.threshold")

**Returns:**
- `Result<Option<String>, PluginError>` - Configuration value or None if not set

**Permissions Required:** `ConfigRead`

**Example Usage:**
```rust
let threshold = get_project_config("my_plugin.complexity_threshold")?
    .map(|s| s.parse::<i32>().unwrap_or(10))
    .unwrap_or(10);

log_info(&format!("Using complexity threshold: {}", threshold));
```

---

#### `set_project_config(key: &str, value: &str) -> Result<(), PluginError>`

Set a configuration value in Uveddi's configuration system.

**Parameters:**
- `key: &str` - Configuration key
- `value: &str` - Configuration value

**Returns:**
- `Result<(), PluginError>` - Success or error

**Permissions Required:** `ConfigRead` (Note: This is a security limitation - write permissions may require higher privileges in future versions)

**Example Usage:**
```rust
set_project_config("my_plugin.last_run", "2025-01-01T12:00:00Z")?;
```

### Logging Functions

#### `log_message(level: LogLevel, message: &str) -> Result<(), PluginError>`

Log a message from the plugin for debugging and monitoring.

**Parameters:**
- `level: LogLevel` - Log level (Debug, Info, Warn, Error)
- `message: &str` - Message to log

**Returns:**
- `Result<(), PluginError>` - Success or error

**Permissions Required:** `Logging` (implicitly granted to most plugins)

**LogLevel Enumeration:**
```rust
pub enum LogLevel {
    Debug,   // Detailed debugging information
    Info,    // General information
    Warn,    // Warning conditions
    Error,   // Error conditions
}
```

**Example Usage:**
```rust
log_message(LogLevel::Info, "Starting custom analysis")?;
log_message(LogLevel::Debug, &format!("Processing {} lines", line_count))?;
log_message(LogLevel::Warn, "Unusual pattern detected")?;
log_message(LogLevel::Error, "Analysis failed for file")?;
```

## Data Structures

### AstNode

Represents a node in the Abstract Syntax Tree.

```rust
pub struct AstNode {
    pub node_type: String,        // Node type (e.g., "function_item", "identifier")
    pub text: String,            // Source text for this node
    pub start_byte: u32,         // Start position in source
    pub end_byte: u32,           // End position in source
    pub start_row: u32,          // Start line number (0-based)
    pub start_column: u32,       // Start column number (0-based)
    pub end_row: u32,            // End line number (0-based)
    pub end_column: u32,         // End column number (0-based)
    pub children: Vec<AstNode>,  // Child nodes
}
```

**Usage Examples:**
```rust
// Traverse AST
fn traverse_ast(node: &AstNode, depth: usize) {
    log_info(&format!("{}{}: {}", 
        "  ".repeat(depth), 
        node.node_type, 
        node.text.chars().take(50).collect::<String>()
    ));
    
    for child in &node.children {
        traverse_ast(child, depth + 1);
    }
}

// Find specific node types
fn find_functions(node: &AstNode) -> Vec<&AstNode> {
    let mut functions = Vec::new();
    
    if node.node_type == "function_item" || node.node_type == "function_definition" {
        functions.push(node);
    }
    
    for child in &node.children {
        functions.extend(find_functions(child));
    }
    
    functions
}
```

### AnalysisRun

Information about an analysis execution.

```rust
pub struct AnalysisRun {
    pub run_id: i32,
    pub start_time: String,      // ISO 8601 timestamp
    pub end_time: Option<String>, // ISO 8601 timestamp
    pub status: String,          // "running", "completed", "failed"
    pub project_path: String,
    pub configuration: String,   // JSON configuration
}
```

## Security Model

### Permission System

Plugins operate under a capability-based security model with explicit permissions.

#### Available Permissions

```rust
pub enum Permission {
    /// Read access to specific directory
    FileRead(PathBuf),
    
    /// Write access to specific directory  
    FileWrite(PathBuf),
    
    /// Network access to specific host:port
    NetworkConnect(String),
    
    /// Environment variable access
    EnvRead(String),
    
    /// Logging permission (usually granted)
    Logging,
    
    /// Configuration read access
    ConfigRead,
    
    /// Temporary file creation (for result storage)
    TempFileCreate,
}
```

#### Permission Configuration

In your `plugin.toml`:

```toml
[capabilities]
permissions = [
    "ConfigRead",                    # Basic configuration access
    "TempFileCreate",               # Store analysis results
    "Logging",                      # Plugin logging
    "FileRead:/specific/path",      # Read specific directory
    "NetworkConnect:api.example.com:443"  # Network access
]
```

### Resource Limits

#### Memory Limits

```toml
[capabilities]
max_memory_mb = 64              # Maximum memory usage
```

#### Execution Limits

```toml
[capabilities]
max_execution_seconds = 30      # Maximum execution time
max_fuel = 1000000             # Computation fuel limit
```

#### API Rate Limits

Host function calls are monitored and limited:

```toml
[capabilities]
max_host_calls = 10000         # Maximum host function calls
```

## Plugin Entry Points

### Required Functions

Every plugin must implement these entry points following the Uveddi plugin interface:

#### `analyze_file`

**Export Name**: `analyze_file`
**Signature**: `pub fn analyze_file(file_data: &[u8]) -> Vec<u8>`

Main analysis function called for each file by the PluginDetectorAdapter.

**Input Format:**
```rust
pub struct PluginFileInput {
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub file_size: usize,
    pub modification_time: SystemTime,
    pub analysis_run_id: String,
}
```

**Output Format:**
```rust
type AnalysisResults = Vec<PluginIssue>;
```

**Implementation Template:**
```rust
use uveddi_plugin_interface::*;

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(_) => return serde_json::to_vec(&Vec::<PluginIssue>::new()).unwrap(),
    };
    
    let mut issues = Vec::new();
    
    // Perform analysis on input.content
    // Example: detect TODO comments
    for (line_num, line) in input.content.lines().enumerate() {
        if line.contains("TODO") || line.contains("FIXME") {
            issues.push(PluginIssue {
                title: "TODO/FIXME found".to_string(),
                description: format!("Consider addressing: {}", line.trim()),
                severity: IssueSeverity::Low,
                file_path: input.file_path.clone(),
                line_number: Some(line_num + 1),
                anti_pattern_type: AntiPatternType::CodeSmell,
                suggested_fix: Some("Implement the TODO item".to_string()),
                code_snippet: Some(line.to_string()),
                metadata: std::collections::HashMap::new(),
            });
        }
    }
    
    serde_json::to_vec(&issues).unwrap()
}
```

#### `get_plugin_info`

**Export Name**: `get_plugin_info`
**Signature**: `pub fn get_plugin_info() -> Vec<u8>`

Returns plugin metadata and capabilities.

**Output Format:**
```rust
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub supported_languages: Vec<String>,
    pub anti_pattern_types: Vec<AntiPatternType>,
    pub required_permissions: Vec<Permission>,
    pub resource_requirements: ResourceRequirements,
}
```

**Implementation Template:**
```rust
#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8> {
    let info = PluginInfo {
        name: "todo-detector".to_string(),
        version: "1.0.0".to_string(),
        description: "Detects TODO and FIXME comments".to_string(),
        author: "Plugin Developer".to_string(),
        license: Some("MIT".to_string()),
        supported_languages: vec![
            "rust".to_string(),
            "python".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
        ],
        anti_pattern_types: vec![AntiPatternType::CodeSmell],
        required_permissions: vec![Permission::ConfigRead, Permission::Logging],
        resource_requirements: ResourceRequirements {
            max_memory_mb: 32,
            max_execution_seconds: 30,
            requires_network: false,
            requires_file_access: false,
        },
    };
    
    serde_json::to_vec(&info).unwrap()
}
```

### Optional Functions

#### `initialize() -> *const u8`

Called once when the plugin is loaded.

```rust
#[no_mangle]
pub extern "C" fn initialize() -> *const u8 {
    // Plugin initialization logic
    log_message(LogLevel::Info, "Plugin initialized successfully");
    
    let result = InitResult { success: true, message: "OK".to_string() };
    let serialized = serde_json::to_vec(&result).unwrap();
    Box::into_raw(serialized.into_boxed_slice()) as *const u8
}

struct InitResult {
    success: bool,
    message: String,
}
```

#### `shutdown() -> *const u8`

Called when the plugin is unloaded.

```rust
#[no_mangle]
pub extern "C" fn shutdown() -> *const u8 {
    // Cleanup logic
    log_message(LogLevel::Info, "Plugin shutting down");
    
    std::ptr::null() // No return value needed
}
```

## Error Handling

### PluginError Types

```rust
pub enum PluginError {
    /// Permission denied for requested operation
    PermissionDenied(String),
    
    /// Invalid input data or parameters
    InvalidInput(String),
    
    /// Internal plugin execution error
    Execution(String),
    
    /// Resource limit exceeded
    ResourceExhausted(String),
    
    /// Host function not available
    HostFunctionUnavailable(String),
}
```

### Error Response Format

When a plugin encounters an error, it should return an error response:

```rust
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub details: Option<String>,
}

// Example usage
let error = ErrorResponse {
    error: "Analysis failed".to_string(),
    code: "ANALYSIS_ERROR".to_string(),
    details: Some("Invalid syntax in input file".to_string()),
};

let serialized = serde_json::to_vec(&error).unwrap();
```

### Best Practices for Error Handling

```rust
// Use Result types internally
fn safe_analyze(file_data: &FileData) -> Result<Vec<PluginIssue>, PluginError> {
    // Validate input
    if file_data.source.is_empty() {
        return Err(PluginError::InvalidInput("Empty source file".to_string()));
    }
    
    // Check resource limits
    if file_data.source.len() > 1_000_000 { // 1MB limit
        return Err(PluginError::ResourceExhausted("File too large".to_string()));
    }
    
    // Perform analysis with error handling
    match perform_complex_analysis(file_data) {
        Ok(results) => Ok(results),
        Err(e) => {
            log_message(LogLevel::Error, &format!("Analysis error: {}", e));
            Err(PluginError::Execution(e.to_string()))
        }
    }
}

// Convert to response in entry point
#[no_mangle]
pub extern "C" fn analyze_file(data_ptr: *const u8, data_len: usize) -> *const u8 {
    let result = std::panic::catch_unwind(|| {
        // ... deserialize input ...
        
        match safe_analyze(&file_data) {
            Ok(results) => serde_json::to_vec(&results).unwrap(),
            Err(e) => {
                let error = ErrorResponse {
                    error: format!("{:?}", e),
                    code: "PLUGIN_ERROR".to_string(),
                    details: None,
                };
                serde_json::to_vec(&error).unwrap()
            }
        }
    });
    
    match result {
        Ok(data) => Box::into_raw(data.into_boxed_slice()) as *const u8,
        Err(_) => std::ptr::null(), // Panic occurred
    }
}
```

## Performance Guidelines

### Memory Management

```rust
// Use iterators to avoid unnecessary allocations
fn analyze_lines_efficiently(source: &str) -> Vec<PluginIssue> {
    source
        .lines()
        .enumerate()
        .filter_map(|(line_num, line)| {
            if line.trim().starts_with("TODO") {
                Some(create_todo_issue(line_num + 1, line))
            } else {
                None
            }
        })
        .collect()
}

// Pre-allocate collections when size is known
fn collect_results(estimated_size: usize) -> Vec<PluginIssue> {
    let mut results = Vec::with_capacity(estimated_size);
    // ... populate results ...
    results
}
```

### Computation Limits

```rust
// Respect fuel limits
fn analyze_with_fuel_awareness(file_data: &FileData) -> Vec<PluginIssue> {
    let start_time = std::time::Instant::now();
    let max_duration = std::time::Duration::from_secs(25); // Leave buffer
    
    let mut results = Vec::new();
    
    for (i, line) in file_data.source.lines().enumerate() {
        // Check time limit periodically
        if i % 100 == 0 && start_time.elapsed() > max_duration {
            log_message(LogLevel::Warn, "Time limit approaching, stopping analysis");
            break;
        }
        
        if let Some(issue) = analyze_line(line, i) {
            results.push(issue);
        }
    }
    
    results
}
```

### Caching Strategies

```rust
use std::collections::HashMap;

// Cache expensive computations
struct AnalysisCache {
    ast_cache: HashMap<String, AstNode>,
    pattern_cache: HashMap<String, Vec<String>>,
}

impl AnalysisCache {
    fn get_or_parse_ast(&mut self, file_path: &str, source: &str, language: &str) -> Option<&AstNode> {
        if !self.ast_cache.contains_key(file_path) {
            if let Ok(ast) = parse_ast(source, language) {
                self.ast_cache.insert(file_path.to_string(), ast);
            } else {
                return None;
            }
        }
        
        self.ast_cache.get(file_path)
    }
}
```

## Integration Examples

### Tree-sitter Query Plugin

```rust
// Plugin that uses tree-sitter queries for pattern detection
pub fn analyze_with_queries(file_data: &FileData) -> Vec<PluginIssue> {
    let mut results = Vec::new();
    
    if !file_data.has_tree {
        return results;
    }
    
    // Define language-specific queries
    let queries = match file_data.language.as_str() {
        "rust" => vec![
            ("long_functions", r#"
                (function_item
                    name: (identifier) @name
                    body: (block) @body
                ) @function
            "#),
            ("unwrap_calls", r#"
                (call_expression
                    function: (field_expression
                        field: (field_identifier) @method
                    )
                ) @call
                (#eq? @method "unwrap")
            "#),
        ],
        "python" => vec![
            ("long_functions", r#"
                (function_definition
                    name: (identifier) @name
                    body: (block) @body
                ) @function  
            "#),
        ],
        _ => return results,
    };
    
    for (pattern_name, query) in queries {
        match execute_tree_sitter_query(&file_data.file_path, query) {
            Ok(matches) => {
                for query_match in matches {
                    results.extend(process_query_match(pattern_name, &query_match, file_data));
                }
            },
            Err(e) => {
                log_message(LogLevel::Warn, &format!("Query '{}' failed: {}", pattern_name, e));
            }
        }
    }
    
    results
}

fn process_query_match(pattern_name: &str, query_match: &QueryMatch, file_data: &FileData) -> Vec<PluginIssue> {
    let mut issues = Vec::new();
    
    match pattern_name {
        "long_functions" => {
            if let Some(body_capture) = query_match.captures.iter().find(|c| c.name == "body") {
                let line_count = body_capture.end_row - body_capture.start_row + 1;
                
                if line_count > 50 {
                    let name_capture = query_match.captures.iter()
                        .find(|c| c.name == "name")
                        .map(|c| c.text.as_str())
                        .unwrap_or("unknown");
                        
                    issues.push(PluginIssue {
                        anti_pattern_type_id: Some(1), // God Object
                        file_path: file_data.file_path.clone(),
                        line_number: Some(body_capture.start_row as i32 + 1),
                        message: format!("Function '{}' is too long ({} lines)", name_capture, line_count),
                        severity: "warning".to_string(),
                        suggestion: "Consider breaking this function into smaller functions".to_string(),
                        metadata: {
                            let mut map = HashMap::new();
                            map.insert("line_count".to_string(), serde_json::Value::Number(line_count.into()));
                            map.insert("function_name".to_string(), serde_json::Value::String(name_capture.to_string()));
                            map
                        },
                    });
                }
            }
        },
        "unwrap_calls" => {
            for capture in &query_match.captures {
                if capture.name == "call" {
                    issues.push(PluginIssue {
                        anti_pattern_type_id: None,
                        file_path: file_data.file_path.clone(),
                        line_number: Some(capture.start_row as i32 + 1),
                        message: "Usage of .unwrap() can cause panics".to_string(),
                        severity: "warning".to_string(),
                        suggestion: "Consider using match, if let, or .expect() with a descriptive message".to_string(),
                        metadata: HashMap::new(),
                    });
                }
            }
        },
        _ => {}
    }
    
    issues
}
```

### Configuration-Driven Plugin

```rust
// Plugin that uses configuration for customizable behavior
#[derive(Debug)]
struct PluginConfig {
    complexity_threshold: i32,
    line_length_limit: usize,
    enabled_rules: Vec<String>,
    ignore_patterns: Vec<String>,
}

impl PluginConfig {
    fn load() -> Self {
        Self {
            complexity_threshold: get_config_int("my_plugin.complexity_threshold").unwrap_or(10),
            line_length_limit: get_config_int("my_plugin.line_length_limit").unwrap_or(120) as usize,
            enabled_rules: get_config_string_list("my_plugin.enabled_rules").unwrap_or_else(|| {
                vec!["long_lines".to_string(), "complexity".to_string()]
            }),
            ignore_patterns: get_config_string_list("my_plugin.ignore_patterns").unwrap_or_default(),
        }
    }
}

pub fn analyze_with_config(file_data: &FileData) -> Vec<PluginIssue> {
    let config = PluginConfig::load();
    let mut results = Vec::new();
    
    // Skip if file matches ignore patterns
    for pattern in &config.ignore_patterns {
        if file_data.file_path.contains(pattern) {
            log_message(LogLevel::Debug, &format!("Skipping file due to ignore pattern: {}", pattern));
            return results;
        }
    }
    
    // Apply enabled rules
    for rule in &config.enabled_rules {
        match rule.as_str() {
            "long_lines" => {
                results.extend(check_long_lines(file_data, config.line_length_limit));
            },
            "complexity" => {
                results.extend(check_complexity(file_data, config.complexity_threshold));
            },
            _ => {
                log_message(LogLevel::Warn, &format!("Unknown rule: {}", rule));
            }
        }
    }
    
    results
}

fn get_config_int(key: &str) -> Option<i32> {
    get_project_config(key).ok().flatten()?.parse().ok()
}

fn get_config_string_list(key: &str) -> Option<Vec<String>> {
    let value = get_project_config(key).ok().flatten()?;
    serde_json::from_str(&value).ok()
}
```

## Testing and Validation

### Unit Testing Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_file_data(source: &str, language: &str) -> FileData {
        FileData {
            file_path: "test.rs".to_string(),
            language: language.to_string(),
            source: source.to_string(),
            has_tree: false,
            syntax_errors: 0,
        }
    }
    
    #[test]
    fn test_empty_file() {
        let file_data = create_test_file_data("", "rust");
        let results = analyze_file_content(&file_data);
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_todo_detection() {
        let source = r#"
            fn main() {
                // TODO: implement this
                println!("Hello");
            }
        "#;
        
        let file_data = create_test_file_data(source, "rust");
        let results = analyze_file_content(&file_data);
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, "info");
        assert!(results[0].message.contains("TODO"));
    }
    
    #[test]
    fn test_performance_with_large_file() {
        let large_source = "fn test() {}\n".repeat(10000);
        let file_data = create_test_file_data(&large_source, "rust");
        
        let start = std::time::Instant::now();
        let results = analyze_file_content(&file_data);
        let duration = start.elapsed();
        
        assert!(duration.as_secs() < 5, "Analysis took too long: {:?}", duration);
        assert!(!results.is_empty(), "Should detect issues in large file");
    }
}
```

### Integration Testing

```rust
// Integration tests using actual host functions (when available)
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_host_function_integration() {
        // This would run in a test environment with mock host functions
        let source = "fn example() {}";
        
        // Test AST parsing
        if let Ok(ast) = parse_ast(source, "rust") {
            assert_eq!(ast.node_type, "source_file");
            assert!(!ast.children.is_empty());
        }
        
        // Test configuration access
        if let Ok(Some(config_value)) = get_project_config("test.key") {
            assert!(!config_value.is_empty());
        }
        
        // Test logging
        assert!(log_message(LogLevel::Info, "Test message").is_ok());
    }
}
```

## Versioning and Compatibility

### API Versioning

The Plugin API follows semantic versioning:

- **Major version**: Breaking changes to host functions or data structures
- **Minor version**: New host functions or optional fields in data structures  
- **Patch version**: Bug fixes and clarifications

### Compatibility Matrix

| Uveddi Version | Plugin API Version | Compatibility |
|---------------|-------------------|---------------|
| 1.0.x         | 1.0.x            | Full compatibility |
| 1.1.x         | 1.0.x - 1.1.x    | Backward compatible |
| 2.0.x         | 2.0.x            | Breaking changes |

### Migration Guide

When updating to new API versions:

1. **Check Compatibility**: Update `plugin.toml` with minimum version requirements
2. **Update Dependencies**: Ensure WASM runtime compatibility  
3. **Test Functions**: Verify all used host functions are available
4. **Update Error Handling**: New error types may be introduced
5. **Performance**: Check resource limit changes

## Troubleshooting

### Common Issues

#### Permission Denied Errors
```
Error: PluginError::PermissionDenied("Plugin does not have permission ConfigRead")
```

**Solution**: Add required permission to `plugin.toml`:
```toml
[capabilities]
permissions = ["ConfigRead", "TempFileCreate", "Logging"]
```

#### Memory Limit Exceeded
```  
Error: PluginError::ResourceExhausted("Memory limit exceeded")
```

**Solutions**:
1. Increase limit in `plugin.toml`:
   ```toml
   [capabilities]
   max_memory_mb = 128
   ```
2. Optimize memory usage in plugin code
3. Process data in chunks

#### Host Function Unavailable
```
Error: PluginError::HostFunctionUnavailable("parse_ast not available")
```

**Solutions**:
1. Check Uveddi version compatibility
2. Ensure required features are enabled
3. Verify plugin is running in correct environment

### Debug Techniques

```rust
// Add comprehensive logging
fn debug_analyze(file_data: &FileData) -> Vec<PluginIssue> {
    log_message(LogLevel::Debug, &format!("Starting analysis of {}", file_data.file_path));
    log_message(LogLevel::Debug, &format!("File size: {} bytes", file_data.source.len()));
    log_message(LogLevel::Debug, &format!("Language: {}", file_data.language));
    log_message(LogLevel::Debug, &format!("AST available: {}", file_data.has_tree));
    
    let start_time = std::time::Instant::now();
    let results = perform_analysis(file_data);
    let duration = start_time.elapsed();
    
    log_message(LogLevel::Debug, &format!("Analysis completed in {:?}", duration));
    log_message(LogLevel::Debug, &format!("Found {} issues", results.len()));
    
    for (i, result) in results.iter().enumerate() {
        log_message(LogLevel::Debug, &format!("Issue {}: {} at line {:?}", 
            i + 1, result.message, result.line_number));
    }
    
    results
}
```

This comprehensive API reference provides all the necessary information for developing robust, secure, and performant WASM plugins for Uveddi.