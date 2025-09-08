# Plugin API Reference

This document provides the complete API reference for developing Uveddi plugins. It covers all available types, functions, and interfaces that plugins can use.

## Table of Contents

1. [Core Types](#core-types)
2. [Plugin Interface](#plugin-interface)
3. [Host Functions](#host-functions)
4. [Data Structures](#data-structures)
5. [Plugin Lifecycle](#plugin-lifecycle)
6. [Permission System](#permission-system)
7. [Configuration API](#configuration-api)
8. [Error Handling](#error-handling)

## Core Types

### Plugin Metadata

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub api_version: String,
}
```

### Plugin Input

```rust
#[derive(Deserialize)]
pub struct PluginFileInput {
    /// Path to the file being analyzed
    pub file_path: String,
    /// Content of the file
    pub content: String,
    /// Programming language of the file
    pub language: String,
    /// Additional context information
    pub context: Option<AnalysisContext>,
}

#[derive(Deserialize)]
pub struct AnalysisContext {
    pub project_root: String,
    pub file_type: String,
    pub encoding: String,
    pub line_count: usize,
    pub size_bytes: usize,
}
```

### Plugin Output

```rust
#[derive(Serialize)]
pub struct PluginAnalysisResult {
    /// Name of the plugin that generated this result
    pub plugin_name: String,
    /// List of issues found
    pub issues: Vec<PluginIssue>,
    /// Analysis metrics
    pub metrics: Option<PluginMetrics>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct PluginIssue {
    /// Unique identifier for the issue type
    pub issue_type: String,
    /// Severity level (INFO, WARNING, ERROR, CRITICAL)
    pub severity: String,
    /// Human-readable message
    pub message: String,
    /// File path where the issue was found
    pub file_path: String,
    /// Line number (1-based)
    pub line_number: Option<u32>,
    /// Column number (0-based)
    pub column: Option<u32>,
    /// End line number for multi-line issues
    pub end_line: Option<u32>,
    /// End column for range issues
    pub end_column: Option<u32>,
    /// Suggested fix or improvement
    pub suggestion: Option<String>,
    /// Additional context or explanation
    pub context: Option<String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: Option<f64>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct PluginMetrics {
    /// Lines of code analyzed
    pub lines_of_code: usize,
    /// Cyclomatic complexity
    pub complexity: Option<usize>,
    /// Maintainability index (0-100)
    pub maintainability_index: Option<f64>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}
```

### Severity Levels

```rust
pub enum Severity {
    Critical,  // Security vulnerabilities, critical bugs
    High,      // Serious issues that should be fixed
    Medium,    // Issues that should be addressed
    Low,       // Minor improvements
    Info,      // Informational findings
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH", 
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
            Severity::Info => "INFO",
        }
    }
}
```

## Plugin Interface

### Required Exports

Every plugin must export these functions:

#### analyze_file

```rust
#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8>
```

**Description**: Main analysis function that processes a single file.

**Parameters**:
- `file_data`: JSON-encoded `PluginFileInput` as bytes

**Returns**: JSON-encoded `PluginAnalysisResult` as bytes

**Example**:
```rust
#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    // Parse input
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => return create_error_result(&format!("Parse error: {}", e)),
    };
    
    // Perform analysis
    let issues = perform_analysis(&input);
    
    // Create result
    let result = PluginAnalysisResult {
        plugin_name: "my-plugin".to_string(),
        issues,
        metrics: Some(calculate_metrics(&input)),
        metadata: HashMap::new(),
    };
    
    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Serialization failed")
    })
}
```

#### get_plugin_info

```rust
#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8>
```

**Description**: Returns plugin metadata and capabilities.

**Returns**: JSON-encoded plugin information

**Example**:
```rust
#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8> {
    let info = serde_json::json!({
        "id": "my-plugin",
        "name": "My Custom Plugin",
        "version": "1.0.0",
        "description": "Custom analysis plugin",
        "author": "Your Name",
        "license": "MIT",
        "api_version": "1.0",
        "supported_languages": ["rust", "python", "javascript"],
        "capabilities": {
            "static_analysis": true,
            "security_scanning": false,
            "performance_analysis": true,
            "metrics_calculation": true
        }
    });
    
    serde_json::to_vec(&info).unwrap_or_default()
}
```

### Optional Exports

#### initialize

```rust
#[export_name = "initialize"]
pub fn initialize(config_data: &[u8]) -> i32
```

**Description**: Initialize plugin with configuration.

**Parameters**:
- `config_data`: JSON-encoded configuration

**Returns**: 0 for success, non-zero for error

#### cleanup

```rust
#[export_name = "cleanup"]
pub fn cleanup() -> i32
```

**Description**: Cleanup resources before plugin unload.

**Returns**: 0 for success, non-zero for error

#### validate_config

```rust
#[export_name = "validate_config"]
pub fn validate_config(config_data: &[u8]) -> Vec<u8>
```

**Description**: Validate plugin configuration.

**Parameters**:
- `config_data`: JSON-encoded configuration

**Returns**: JSON-encoded validation result

## Host Functions

Plugins can call these host-provided functions:

### Logging Functions

```rust
extern "C" {
    /// Log debug message
    pub fn host_log_debug(msg_ptr: *const u8, msg_len: usize);
    
    /// Log info message
    pub fn host_log_info(msg_ptr: *const u8, msg_len: usize);
    
    /// Log warning message
    pub fn host_log_warn(msg_ptr: *const u8, msg_len: usize);
    
    /// Log error message
    pub fn host_log_error(msg_ptr: *const u8, msg_len: usize);
}

// Convenience wrappers
pub fn log_debug(message: &str) {
    unsafe {
        host_log_debug(message.as_ptr(), message.len());
    }
}

pub fn log_info(message: &str) {
    unsafe {
        host_log_info(message.as_ptr(), message.len());
    }
}

pub fn log_warn(message: &str) {
    unsafe {
        host_log_warn(message.as_ptr(), message.len());
    }
}

pub fn log_error(message: &str) {
    unsafe {
        host_log_error(message.as_ptr(), message.len());
    }
}
```

### AST Parsing Functions

```rust
extern "C" {
    /// Parse code to AST using Tree-sitter
    pub fn host_parse_ast(
        code_ptr: *const u8, 
        code_len: usize, 
        lang_ptr: *const u8, 
        lang_len: usize
    ) -> u64;
    
    /// Execute Tree-sitter query on AST
    pub fn host_query_ast(
        ast_handle: u64,
        query_ptr: *const u8,
        query_len: usize
    ) -> u64;
    
    /// Get AST node information
    pub fn host_get_ast_node(
        ast_handle: u64,
        node_id: u32
    ) -> u64;
    
    /// Free AST handle
    pub fn host_free_ast(ast_handle: u64);
}

// Convenience wrapper
pub fn parse_ast(code: &str, language: &str) -> Option<u64> {
    unsafe {
        let handle = host_parse_ast(
            code.as_ptr(), code.len(),
            language.as_ptr(), language.len()
        );
        if handle != 0 {
            Some(handle)
        } else {
            None
        }
    }
}
```

### Configuration Functions

```rust
extern "C" {
    /// Get configuration value by key
    pub fn host_get_config(key_ptr: *const u8, key_len: usize) -> u64;
    
    /// Set configuration value
    pub fn host_set_config(
        key_ptr: *const u8, 
        key_len: usize,
        value_ptr: *const u8,
        value_len: usize
    ) -> i32;
}

pub fn get_config_value(key: &str) -> Option<String> {
    unsafe {
        let handle = host_get_config(key.as_ptr(), key.len());
        if handle != 0 {
            // Extract string from handle (simplified)
            Some("config_value".to_string())
        } else {
            None
        }
    }
}
```

### File System Functions

```rust
extern "C" {
    /// Read file contents (if permitted)
    pub fn host_read_file(path_ptr: *const u8, path_len: usize) -> u64;
    
    /// Write file contents (if permitted)
    pub fn host_write_file(
        path_ptr: *const u8,
        path_len: usize,
        content_ptr: *const u8,
        content_len: usize
    ) -> i32;
    
    /// Check if file exists
    pub fn host_file_exists(path_ptr: *const u8, path_len: usize) -> i32;
    
    /// Get file metadata
    pub fn host_get_file_metadata(path_ptr: *const u8, path_len: usize) -> u64;
}
```

### Network Functions (if permitted)

```rust
extern "C" {
    /// Make HTTP request
    pub fn host_http_request(
        method_ptr: *const u8,
        method_len: usize,
        url_ptr: *const u8,
        url_len: usize,
        body_ptr: *const u8,
        body_len: usize,
        headers_ptr: *const u8,
        headers_len: usize
    ) -> u64;
}
```

## Data Structures

### AST Node Information

```rust
#[derive(Deserialize)]
pub struct AstNodeInfo {
    pub node_id: u32,
    pub node_type: String,
    pub start_position: Position,
    pub end_position: Position,
    pub text: String,
    pub children: Vec<u32>,
    pub parent: Option<u32>,
}

#[derive(Deserialize)]
pub struct Position {
    pub row: u32,
    pub column: u32,
    pub byte_offset: u32,
}
```

### Query Match

```rust
#[derive(Deserialize)]
pub struct QueryMatch {
    pub pattern_index: u32,
    pub captures: Vec<QueryCapture>,
}

#[derive(Deserialize)]
pub struct QueryCapture {
    pub node_id: u32,
    pub capture_name: String,
    pub capture_index: u32,
}
```

### File Metadata

```rust
#[derive(Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub modified_time: u64,
    pub created_time: u64,
    pub is_directory: bool,
    pub permissions: u32,
    pub mime_type: Option<String>,
}
```

## Plugin Lifecycle

### Lifecycle States

```rust
pub enum PluginState {
    /// Plugin is being loaded
    Loading,
    /// Plugin is initialized and ready
    Ready,
    /// Plugin is currently analyzing
    Active,
    /// Plugin encountered an error
    Error(String),
    /// Plugin is being unloaded
    Unloading,
}
```

### Lifecycle Hooks

```rust
// Optional lifecycle hooks that plugins can implement

/// Called when plugin is first loaded
#[export_name = "on_load"]
pub fn on_load() -> i32 {
    // Initialize global state
    // Return 0 for success
    0
}

/// Called before analysis batch starts
#[export_name = "on_analysis_start"]  
pub fn on_analysis_start(context_data: &[u8]) -> i32 {
    // Prepare for analysis
    0
}

/// Called after analysis batch completes
#[export_name = "on_analysis_complete"]
pub fn on_analysis_complete(results_data: &[u8]) -> i32 {
    // Post-process results
    0
}

/// Called when plugin is being unloaded
#[export_name = "on_unload"]
pub fn on_unload() -> i32 {
    // Cleanup resources
    0
}
```

## Permission System

### Permission Types

```rust
pub enum Permission {
    /// Read files matching pattern
    FileRead(String),
    /// Write files matching pattern  
    FileWrite(String),
    /// Create temporary files
    TempFileCreate,
    /// Make network requests to domain
    NetworkConnect(String),
    /// Read configuration values
    ConfigRead,
    /// Write configuration values
    ConfigWrite,
    /// Read environment variables
    EnvRead(String),
    /// Execute external processes
    ProcessSpawn(String),
    /// Write to logs
    Logging,
    /// Access analysis database
    DatabaseAccess,
    /// Communicate with other plugins
    PluginCommunication,
}
```

### Permission Declaration

```toml
# plugin.toml
[permissions]
# Basic permissions
logging = true
config_read = true

# File system permissions
file_read_patterns = ["src/**/*.rs", "*.toml"]
file_write_patterns = ["target/cache/**"]
temp_file_create = true

# Network permissions  
network_domains = ["api.example.com"]

# System permissions
env_read_vars = ["CI", "BUILD_*"]
process_spawn = ["rustc", "cargo"]

# Resource limits
max_memory_mb = 64
max_execution_seconds = 60
max_file_operations = 100
max_network_requests = 10
```

### Permission Checking

```rust
// Host function to check if permission is granted
extern "C" {
    pub fn host_check_permission(
        permission_ptr: *const u8,
        permission_len: usize
    ) -> i32;  // 1 if granted, 0 if denied
}

pub fn has_permission(permission: &str) -> bool {
    unsafe {
        host_check_permission(permission.as_ptr(), permission.len()) != 0
    }
}

// Usage in plugin
pub fn read_config_file(path: &str) -> Result<String, &'static str> {
    if !has_permission(&format!("file_read:{}", path)) {
        return Err("Permission denied");
    }
    
    // Proceed with file reading
    Ok("file_content".to_string())
}
```

## Configuration API

### Configuration Schema

```rust
#[derive(Deserialize, Serialize)]
pub struct PluginConfig {
    /// Plugin-specific settings
    pub settings: HashMap<String, ConfigValue>,
    /// Analysis thresholds
    pub thresholds: HashMap<String, f64>,
    /// Enabled rules
    pub rules: HashMap<String, RuleConfig>,
    /// Ignore patterns
    pub ignore_patterns: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

#[derive(Deserialize, Serialize)]
pub struct RuleConfig {
    pub enabled: bool,
    pub severity: String,
    pub parameters: HashMap<String, ConfigValue>,
}
```

### Configuration Loading

```rust
pub fn load_plugin_config() -> Result<PluginConfig, String> {
    // Get raw config from host
    let config_json = get_config_value("plugin.config")
        .ok_or("No configuration found")?;
        
    // Parse configuration
    serde_json::from_str(&config_json)
        .map_err(|e| format!("Invalid configuration: {}", e))
}

pub fn get_rule_config(rule_id: &str) -> Option<RuleConfig> {
    let config = load_plugin_config().ok()?;
    config.rules.get(rule_id).cloned()
}

pub fn is_rule_enabled(rule_id: &str) -> bool {
    get_rule_config(rule_id)
        .map(|config| config.enabled)
        .unwrap_or(false)
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, Serialize)]
pub enum PluginError {
    /// Invalid input provided
    InvalidInput(String),
    /// Configuration error
    ConfigError(String), 
    /// Permission denied
    PermissionDenied(String),
    /// Resource limit exceeded
    ResourceLimit(String),
    /// Analysis failed
    AnalysisError(String),
    /// Internal plugin error
    InternalError(String),
}

impl PluginError {
    pub fn as_json(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}
```

### Error Response Format

```rust
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
    pub suggestion: Option<String>,
}

pub fn create_error_result(message: &str) -> Vec<u8> {
    let error = ErrorResponse {
        error_type: "PLUGIN_ERROR".to_string(),
        message: message.to_string(),
        details: None,
        suggestion: None,
    };
    
    serde_json::to_vec(&error).unwrap_or_default()
}
```

### Error Handling Best Practices

```rust
#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    // Validate input size
    if file_data.len() > MAX_INPUT_SIZE {
        return create_error_result("Input too large");
    }
    
    // Parse input with error handling
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => {
            log_error(&format!("Failed to parse input: {}", e));
            return create_error_result("Invalid input format");
        }
    };
    
    // Validate input content
    if let Err(e) = validate_input(&input) {
        log_warn(&format!("Input validation failed: {}", e));
        return create_error_result(&e);
    }
    
    // Perform analysis with error handling
    let issues = match perform_analysis(&input) {
        Ok(issues) => issues,
        Err(e) => {
            log_error(&format!("Analysis failed: {}", e));
            return create_error_result("Analysis failed");
        }
    };
    
    // Create successful result
    let result = PluginAnalysisResult {
        plugin_name: "my-plugin".to_string(),
        issues,
        metrics: None,
        metadata: HashMap::new(),
    };
    
    serde_json::to_vec(&result).unwrap_or_else(|e| {
        log_error(&format!("Failed to serialize result: {}", e));
        create_error_result("Serialization failed")
    })
}
```

## Memory Management

### Memory Allocation

```rust
// Host functions for memory management
extern "C" {
    /// Allocate memory in host
    pub fn host_malloc(size: usize) -> *mut u8;
    
    /// Free memory in host
    pub fn host_free(ptr: *mut u8);
    
    /// Get memory usage statistics
    pub fn host_get_memory_stats() -> u64;
}

// Memory-safe wrappers
pub struct HostMemory {
    ptr: *mut u8,
    size: usize,
}

impl HostMemory {
    pub fn allocate(size: usize) -> Option<Self> {
        unsafe {
            let ptr = host_malloc(size);
            if ptr.is_null() {
                None
            } else {
                Some(HostMemory { ptr, size })
            }
        }
    }
    
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.size)
        }
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.size)
        }
    }
}

impl Drop for HostMemory {
    fn drop(&mut self) {
        unsafe {
            host_free(self.ptr);
        }
    }
}
```

This API reference provides comprehensive documentation for developing plugins with Uveddi. For additional examples and tutorials, see the [Plugin Examples](examples.md) and [Development Guide](development-guide.md).