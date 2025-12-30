# Plugin API Reference

This document provides the complete API reference for developing Uveddi plugins using the WebAssembly Component Model. Plugins are defined using WIT (WebAssembly Interface Types) and compiled to WASM components.

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

All types are defined in the WIT interface file (`wit/core-analysis.wit`). Plugins use `wit_bindgen` to generate Rust bindings for these types.

### Position

Represents a position in source code.

```wit
record position {
    line: u32,
    column: u32,
    byte-offset: u32,
}
```

### Span

Represents a range in source code.

```wit
record span {
    start: position,
    end: position,
}
```

### Source File

Input file provided to the plugin for analysis.

```wit
record source-file {
    path: string,
    content: string,
    language: string,
    size: u32,
    hash: string,
    ast: option<ast-node>,
}
```

### AST Node

Represents a node in the abstract syntax tree.

```wit
record ast-node {
    node-type: string,
    content: string,
    span: span,
    language: string,
    attributes: list<tuple<string, string>>,
}
```

### Severity Levels

```wit
enum severity-level {
    critical,  // Security vulnerabilities, critical bugs
    high,      // Serious issues that should be fixed
    medium,    // Issues that should be addressed
    low,       // Minor improvements
    info,      // Informational findings
}
```

### Issue Categories

```wit
enum issue-category {
    security,
    performance,
    quality,
    maintainability,
    style,
    complexity,
    duplication,
    architecture,
    documentation,
    testing,
}
```

### Issue

Represents an issue detected by the plugin.

```wit
record issue {
    id: string,
    severity: severity-level,
    category: issue-category,
    message: string,
    description: option<string>,
    file: string,
    span: span,
    rule-id: option<string>,
    suggestion: option<string>,
    fix: option<code-fix>,
    metadata: list<tuple<string, string>>,
}
```

### Code Fix

Suggested fix for an issue.

```wit
record code-fix {
    description: string,
    replacements: list<replacement>,
}

record replacement {
    span: span,
    new-text: string,
}
```

### Metrics

Code metrics calculated during analysis.

```wit
record metrics {
    lines-of-code: u32,
    lines-of-comments: u32,
    complexity: u32,
    maintainability-index: f64,
    technical-debt-minutes: u32,
    custom-metrics: list<tuple<string, f64>>,
}
```

### Analysis Result

Result returned by the plugin's analyze function.

```wit
record analysis-result {
    issues: list<issue>,
    metrics: metrics,
    dependencies: list<string>,
    exports: list<string>,
    duration-ms: u32,
    plugin-version: string,
}
```

## Plugin Interface

### Required Exports

Every plugin must export these four functions:

#### initialize

```wit
export initialize: func(config: plugin-config, limits: resource-limits) -> result<_, string>;
```

**Description**: Initialize the plugin with configuration and resource limits.

**Parameters**:
- `config`: Plugin configuration settings
- `limits`: Resource limits for execution

**Returns**: `Ok(())` for success, `Err(message)` for failure

**Example**:
```rust
fn initialize(config: PluginConfig, limits: ResourceLimits) -> Result<(), String> {
    log(LogLevel::Info, "Initializing plugin");

    // Store configuration for later use
    let mut state = STATE.borrow_mut();
    *state = Some(PluginState::new(config, limits));

    Ok(())
}
```

#### analyze

```wit
export analyze: func(file: source-file) -> result<analysis-result, string>;
```

**Description**: Analyze a source file and return issues and metrics.

**Parameters**:
- `file`: Source file to analyze (includes path, content, language, and optional pre-parsed AST)

**Returns**: `Ok(analysis-result)` with issues and metrics, or `Err(message)` on failure

**Example**:
```rust
fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
    let mut issues = Vec::new();

    // Parse AST if not provided
    let ast = if let Some(ast) = file.ast {
        ast
    } else {
        parse_ast(&file.content, &file.language)?
    };

    // Perform analysis
    issues.extend(check_complexity(&ast, &file)?);
    issues.extend(check_style(&ast, &file)?);

    Ok(AnalysisResult {
        issues,
        metrics: calculate_metrics(&file),
        dependencies: vec![],
        exports: vec![],
        duration_ms: 0,
        plugin_version: "1.0.0".to_string(),
    })
}
```

#### get-info

```wit
export get-info: func() -> plugin-info;
```

**Description**: Return plugin metadata and capabilities.

**Returns**: Plugin information structure

**Example**:
```rust
fn get_info() -> PluginInfo {
    PluginInfo {
        id: "my-plugin".to_string(),
        name: "My Custom Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Custom analysis plugin".to_string(),
        author: "Your Name".to_string(),
        license: "MIT".to_string(),
        homepage: Some("https://github.com/yourname/my-plugin".to_string()),
        supported_languages: vec![
            "rust".to_string(),
            "python".to_string(),
            "javascript".to_string(),
        ],
        detector_types: vec![IssueCategory::Quality, IssueCategory::Complexity],
        api_version: "1.0".to_string(),
        required_permissions: vec![Permission::ReadFiles],
    }
}
```

#### cleanup

```wit
export cleanup: func() -> result<_, string>;
```

**Description**: Clean up resources before plugin unload.

**Returns**: `Ok(())` for success, `Err(message)` for failure

**Example**:
```rust
fn cleanup() -> Result<(), String> {
    log(LogLevel::Info, "Cleaning up plugin resources");

    // Clear any cached state
    let mut state = STATE.borrow_mut();
    *state = None;

    Ok(())
}
```

## Host Functions

Plugins can call these host-provided functions. All functions are type-safe and use the WIT interface.

### Logging

```wit
enum log-level {
    trace,
    debug,
    info,
    warn,
    error,
}

import log: func(level: log-level, message: string);
```

**Example**:
```rust
log(LogLevel::Info, "Starting analysis");
log(LogLevel::Warn, &format!("High complexity detected: {}", value));
log(LogLevel::Error, "Failed to parse file");
```

### File System Operations

```wit
import read-file: func(path: string) -> result<string, string>;
import write-file: func(path: string, content: string) -> result<_, string>;
import file-exists: func(path: string) -> bool;
import list-files: func(pattern: string) -> result<list<string>, string>;
import get-file-metadata: func(path: string) -> result<file-metadata, string>;

record file-metadata {
    size: u64,
    modified: u64,  // Unix timestamp
    is-directory: bool,
    permissions: u32,
}
```

**Example**:
```rust
// Read a configuration file
if file_exists("config.toml".to_string()) {
    match read_file("config.toml".to_string()) {
        Ok(content) => log(LogLevel::Info, &format!("Config: {}", content)),
        Err(e) => log(LogLevel::Error, &format!("Failed to read config: {}", e)),
    }
}

// List all Rust files
match list_files("src/**/*.rs".to_string()) {
    Ok(files) => {
        for file in files {
            log(LogLevel::Debug, &format!("Found: {}", file));
        }
    }
    Err(e) => log(LogLevel::Error, &e),
}
```

### Configuration Access

```wit
import get-config: func(key: string) -> option<string>;
import set-config: func(key: string, value: string) -> result<_, string>;
```

**Example**:
```rust
// Get a configuration value
if let Some(threshold) = get_config("complexity_threshold".to_string()) {
    let value: u32 = threshold.parse().unwrap_or(10);
    log(LogLevel::Info, &format!("Using threshold: {}", value));
}

// Set a configuration value
set_config("last_run".to_string(), "2024-01-15".to_string())?;
```

### AST Parsing and Querying

```wit
import parse-ast: func(code: string, language: string) -> result<ast-node, string>;
import query-ast: func(node: ast-node, query: string) -> result<list<ast-node>, string>;
```

**Example**:
```rust
// Parse code to AST
let ast = parse_ast(&file.content, &file.language)?;

// Query for function definitions (tree-sitter query syntax)
let functions = query_ast(&ast, "(function_item) @function")?;

for func in functions {
    log(LogLevel::Debug, &format!("Found function: {}", func.content));
}
```

### Cryptographic Utilities

```wit
import calculate-hash: func(algorithm: string, content: string) -> result<string, string>;
import verify-signature: func(content: string, signature: string, public-key: string) -> result<bool, string>;
```

**Example**:
```rust
// Calculate SHA-256 hash
let hash = calculate_hash("sha256".to_string(), file.content.clone())?;
log(LogLevel::Info, &format!("File hash: {}", hash));
```

### Network Operations

Requires `network-access` permission.

```wit
import http-get: func(url: string, headers: list<tuple<string, string>>) -> result<http-response, string>;
import http-post: func(url: string, body: string, headers: list<tuple<string, string>>) -> result<http-response, string>;

record http-response {
    status: u16,
    headers: list<tuple<string, string>>,
    body: string,
}
```

**Example**:
```rust
// Make an HTTP GET request
let headers = vec![("Accept".to_string(), "application/json".to_string())];
match http_get("https://api.example.com/rules".to_string(), headers) {
    Ok(response) => {
        if response.status == 200 {
            log(LogLevel::Info, &format!("Rules: {}", response.body));
        }
    }
    Err(e) => log(LogLevel::Error, &format!("Request failed: {}", e)),
}
```

### Database Operations

For caching and persistence.

```wit
import db-get: func(key: string) -> option<string>;
import db-set: func(key: string, value: string, ttl-seconds: option<u32>) -> result<_, string>;
import db-delete: func(key: string) -> result<bool, string>;
```

**Example**:
```rust
// Cache analysis results
let cache_key = format!("analysis:{}", file.hash);
if let Some(cached) = db_get(cache_key.clone()) {
    log(LogLevel::Info, "Using cached results");
    return Ok(serde_json::from_str(&cached).unwrap());
}

// Store results with 1-hour TTL
let result = perform_analysis(&file)?;
db_set(cache_key, serde_json::to_string(&result).unwrap(), Some(3600))?;
```

### Process Information

```wit
import get-process-info: func() -> result<process-info, string>;

record process-info {
    pid: u32,
    memory-usage-mb: u32,
    cpu-usage-percent: f32,
    uptime-seconds: u32,
}
```

## Data Structures

### Plugin Configuration

```wit
record plugin-config {
    severity-threshold: severity-level,
    max-issues-per-file: u32,
    include-patterns: list<string>,
    exclude-patterns: list<string>,
    rule-overrides: list<tuple<string, bool>>,
    custom-settings: list<tuple<string, string>>,
}
```

### Resource Limits

```wit
record resource-limits {
    max-memory-mb: u32,
    max-cpu-percent: u32,
    timeout-seconds: u32,
    max-file-handles: u32,
}
```

### Plugin Info

```wit
record plugin-info {
    id: string,
    name: string,
    version: string,
    description: string,
    author: string,
    license: string,
    homepage: option<string>,
    supported-languages: list<string>,
    detector-types: list<issue-category>,
    api-version: string,
    required-permissions: list<permission>,
}
```

## Plugin Lifecycle

### Lifecycle Flow

```
1. Loading    -> Plugin WASM binary loaded into runtime
2. Initialize -> initialize(config, limits) called
3. Ready      -> Plugin ready to process files
4. Analysis   -> analyze(file) called for each file
5. Cleanup    -> cleanup() called before unload
6. Unloaded   -> Plugin removed from runtime
```

### State Management

Plugins should use thread-local or RefCell-based state:

```rust
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<Option<PluginState>> = RefCell::new(None);
}

struct PluginState {
    config: PluginConfig,
    limits: ResourceLimits,
    analysis_count: u32,
}

impl PluginState {
    fn new(config: PluginConfig, limits: ResourceLimits) -> Self {
        Self {
            config,
            limits,
            analysis_count: 0,
        }
    }
}
```

## Permission System

### Permission Types

```wit
flags permission {
    read-files,       // Read files from the file system
    write-files,      // Write files to the file system
    network-access,   // Make network requests
    system-info,      // Access system information
    environment-vars, // Read environment variables
    spawn-processes,  // Execute external processes
}
```

### Declaring Permissions

Permissions are declared in the plugin manifest (`plugin.toml`):

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"

[capabilities]
permissions = ["ReadFiles", "Logging"]

[capabilities.limits]
max_memory_mb = 64
max_execution_seconds = 30
fuel_limit = 2000000
```

### Permission Checking

The host automatically checks permissions before allowing operations. If a plugin attempts an operation without the required permission, it receives an error:

```rust
// This will fail if plugin lacks read-files permission
match read_file("sensitive.txt".to_string()) {
    Ok(content) => { /* use content */ }
    Err(e) => {
        // e = "Plugin does not have permission read-files"
        log(LogLevel::Error, &e);
    }
}
```

## Configuration API

### Plugin Configuration Schema

Configuration is passed during initialization:

```rust
fn initialize(config: PluginConfig, limits: ResourceLimits) -> Result<(), String> {
    // Access severity threshold
    let threshold = config.severity_threshold; // SeverityLevel enum

    // Access max issues
    let max_issues = config.max_issues_per_file;

    // Access include/exclude patterns
    for pattern in &config.include_patterns {
        log(LogLevel::Debug, &format!("Include: {}", pattern));
    }

    // Access custom settings
    for (key, value) in &config.custom_settings {
        log(LogLevel::Debug, &format!("Setting: {} = {}", key, value));
    }

    Ok(())
}
```

### Runtime Configuration

Use `get_config` and `set_config` for runtime configuration:

```rust
// Read configuration
let complexity_threshold = get_config("my_plugin.complexity_threshold".to_string())
    .and_then(|s| s.parse::<u32>().ok())
    .unwrap_or(10);

// Write configuration (persists for session)
set_config("my_plugin.last_file".to_string(), file.path.clone())?;
```

## Error Handling

### Error Response Format

All fallible functions return `Result<T, String>`. The error string should be descriptive:

```rust
fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
    // Validate input
    if file.content.is_empty() {
        return Err("Cannot analyze empty file".to_string());
    }

    // Parse AST with error handling
    let ast = parse_ast(&file.content, &file.language)
        .map_err(|e| format!("AST parsing failed: {}", e))?;

    // Perform analysis
    let issues = analyze_ast(&ast, &file)
        .map_err(|e| format!("Analysis failed: {}", e))?;

    Ok(AnalysisResult {
        issues,
        metrics: Metrics::default(),
        dependencies: vec![],
        exports: vec![],
        duration_ms: 0,
        plugin_version: "1.0.0".to_string(),
    })
}
```

### Best Practices

1. **Validate inputs early**: Check for empty content, invalid languages, etc.
2. **Use descriptive errors**: Include context about what failed and why
3. **Log errors**: Use `log(LogLevel::Error, ...)` before returning errors
4. **Handle host function failures**: All host functions can fail, handle errors appropriately
5. **Don't panic**: Use `Result` types instead of panicking

```rust
fn safe_analysis(file: &SourceFile) -> Result<Vec<Issue>, String> {
    // Validate
    if file.content.len() > 10_000_000 {
        log(LogLevel::Warn, "File too large, skipping detailed analysis");
        return Ok(vec![]);
    }

    // Parse with error handling
    let ast = match parse_ast(&file.content, &file.language) {
        Ok(ast) => ast,
        Err(e) => {
            log(LogLevel::Error, &format!("Parse error: {}", e));
            return Err(format!("Failed to parse {}: {}", file.path, e));
        }
    };

    // Analyze
    Ok(find_issues(&ast))
}
```

## Complete Plugin Example

Here's a complete minimal plugin implementation:

```rust
wit_bindgen::generate!({
    world: "core-analysis",
    path: "wit/core-analysis.wit",
});

use std::cell::RefCell;

struct MyPlugin {
    state: RefCell<Option<PluginState>>,
}

struct PluginState {
    config: PluginConfig,
    analysis_count: u32,
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

export!(MyPlugin);

impl Guest for MyPlugin {
    fn initialize(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing MyPlugin");

        let state = PluginState {
            config,
            analysis_count: 0,
        };

        *STATE.borrow_mut() = Some(state);
        Ok(())
    }

    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        log(LogLevel::Info, &format!("Analyzing {}", file.path));

        let mut issues = Vec::new();

        // Example: Check for TODO comments
        for (line_num, line) in file.content.lines().enumerate() {
            if line.contains("TODO") {
                issues.push(Issue {
                    id: format!("TODO-{}", line_num),
                    severity: SeverityLevel::Info,
                    category: IssueCategory::Documentation,
                    message: "TODO comment found".to_string(),
                    description: Some("Consider creating an issue for this TODO".to_string()),
                    file: file.path.clone(),
                    span: Span {
                        start: Position { line: line_num as u32, column: 0, byte_offset: 0 },
                        end: Position { line: line_num as u32, column: line.len() as u32, byte_offset: 0 },
                    },
                    rule_id: Some("todo-comment".to_string()),
                    suggestion: Some("Create an issue tracker entry".to_string()),
                    fix: None,
                    metadata: vec![],
                });
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
            id: "my-plugin".to_string(),
            name: "My Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Example plugin for demonstration".to_string(),
            author: "Your Name".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            supported_languages: vec!["rust".to_string(), "python".to_string()],
            detector_types: vec![IssueCategory::Documentation],
            api_version: "1.0".to_string(),
            required_permissions: vec![],
        }
    }

    fn cleanup() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up MyPlugin");
        *STATE.borrow_mut() = None;
        Ok(())
    }
}

thread_local! {
    static STATE: RefCell<Option<PluginState>> = RefCell::new(None);
}
```

This API reference provides comprehensive documentation for developing plugins with Uveddi using the WebAssembly Component Model. For tutorials and examples, see the [Plugin Development Guide](development-guide.md) and [Plugin Examples](examples.md).
