# WASM Plugin API Reference

Complete reference for the Uveddi WASM plugin API.

## WIT Interface Specification

### World Definition

```wit
package uveddi:plugins@0.1.0;

world plugin {
    // Host functions that plugins can import
    import logging: interface {
        log: func(level: string, message: string);
    }
    
    import config: interface {
        get-value: func(key: string) -> option<string>;
    }

    // Plugin exports
    export info: func() -> plugin-info;
    export analyze: func(dependencies: list<dependency>) -> list<architectural-issue>;
}
```

## Data Types

### Core Types

#### `dependency`
```wit
record dependency {
    from-module: string,     // Source module identifier
    to-module: string,       // Target module identifier  
    dependency-type: string, // Type of dependency relationship
}
```

**Fields:**
- `from-module`: The name/path of the module that has the dependency
- `to-module`: The name/path of the module being depended upon
- `dependency-type`: The type of dependency (e.g., "import", "use", "mod", "external")

**Example:**
```json
{
  "from-module": "src/main",
  "to-module": "std::collections",
  "dependency-type": "use"
}
```

#### `architectural-issue`
```wit
record architectural-issue {
    file-path: string,              // File where issue was found
    start-line: u32,                // Starting line number
    end-line: u32,                  // Ending line number
    issue-type: string,             // Custom issue classifier
    severity: string,               // Issue severity level
    message: string,                // Human-readable description
    code-snippet: option<string>,   // Optional code context
}
```

**Fields:**
- `file-path`: Absolute or relative path to the file containing the issue
- `start-line`: Line number where the issue begins (1-indexed)
- `end-line`: Line number where the issue ends (1-indexed)
- `issue-type`: Plugin-defined issue classification
- `severity`: One of "low", "medium", "high", "critical"
- `message`: Clear, actionable description of the issue
- `code-snippet`: Optional code excerpt showing the problematic code

**Example:**
```json
{
  "file-path": "src/user_service.rs",
  "start-line": 45,
  "end-line": 45,
  "issue-type": "god_object",
  "severity": "high",
  "message": "Class has 23 dependencies, indicating it may be a God Object",
  "code-snippet": "impl UserService { ... }"
}
```

#### `plugin-info`
```wit
record plugin-info {
    name: string,        // Plugin display name
    version: string,     // Semantic version
    description: string, // What the plugin does
    author: string,      // Plugin author/maintainer
}
```

**Fields:**
- `name`: Human-readable plugin name
- `version`: Semantic version string (e.g., "1.2.3")
- `description`: Brief description of plugin functionality
- `author`: Plugin developer name or organization

## Exported Functions

### `info() -> plugin-info`

Returns metadata about the plugin.

**Purpose:** Provides information for plugin discovery and management.

**Returns:** Plugin metadata record

**Example Implementation:**
```rust
fn info() -> PluginInfo {
    PluginInfo {
        name: "Complexity Analyzer".to_string(),
        version: "1.0.0".to_string(),
        description: "Detects overly complex code structures".to_string(),
        author: "Uveddi Team".to_string(),
    }
}
```

### `analyze(dependencies: list<dependency>) -> list<architectural-issue>`

Performs analysis on the provided dependency graph.

**Purpose:** Main plugin entry point for code analysis.

**Parameters:**
- `dependencies`: List of all dependencies found in the analyzed codebase

**Returns:** List of architectural issues found by the plugin

**Example Implementation:**
```rust
fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
    let mut issues = Vec::new();
    
    // Analysis logic here
    for dep in dependencies {
        if is_problematic(&dep) {
            issues.push(ArchitecturalIssue {
                file_path: dep.from_module,
                start_line: 1,
                end_line: 1,
                issue_type: "custom_pattern".to_string(),
                severity: "medium".to_string(),
                message: "Problematic pattern detected".to_string(),
                code_snippet: None,
            });
        }
    }
    
    issues
}
```

## Imported Functions (Host API)

### Logging Interface

#### `logging::log(level: string, message: string)`

Logs a message through the host logging system.

**Purpose:** Allows plugins to emit log messages for debugging and monitoring.

**Parameters:**
- `level`: Log level ("debug", "info", "warn", "error")
- `message`: Log message content

**Example Usage:**
```rust
// Log plugin startup
logging::log("info", "Analysis plugin started");

// Log warning
logging::log("warn", "Unusual dependency pattern detected");

// Log error
logging::log("error", "Failed to process dependency graph");

// Debug information
logging::log("debug", &format!("Processing {} dependencies", deps.len()));
```

**Log Levels:**
- `"debug"`: Detailed diagnostic information
- `"info"`: General informational messages
- `"warn"`: Warning messages for unusual conditions
- `"error"`: Error messages for failures

### Configuration Interface

#### `config::get-value(key: string) -> option<string>`

Retrieves a configuration value from the host environment.

**Purpose:** Allows plugins to access configuration settings.

**Parameters:**
- `key`: Configuration key name

**Returns:** Optional configuration value as string

**Example Usage:**
```rust
// Get plugin-specific configuration
if let Some(threshold) = config::get_value("complexity_threshold") {
    let threshold: u32 = threshold.parse().unwrap_or(10);
    // Use threshold in analysis
}

// Get global settings
if let Some(debug_mode) = config::get_value("debug_mode") {
    if debug_mode == "true" {
        // Enable detailed logging
    }
}
```

**Standard Configuration Keys:**
- `"debug_mode"`: Enable debug output ("true"/"false")
- `"max_issues"`: Maximum issues to report per plugin
- `"<plugin_name>_*"`: Plugin-specific configuration

## Error Handling

### Plugin-Level Errors

Plugins should handle errors gracefully and continue processing when possible:

```rust
fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
    let mut issues = Vec::new();
    
    for dep in dependencies {
        match process_dependency(&dep) {
            Ok(Some(issue)) => issues.push(issue),
            Ok(None) => continue,
            Err(e) => {
                logging::log("error", &format!("Error processing {}: {}", dep.from_module, e));
                continue; // Don't fail entire analysis
            }
        }
    }
    
    issues
}
```

### Host Error Handling

The host will handle plugin failures:
- Plugin crashes are isolated and logged
- Resource limit violations terminate plugin execution
- Verification failures prevent plugin loading

## Resource Management

### Memory Limits

Plugins have limited memory available:
- Default: 16MB maximum
- Configurable via plugin manifest
- Host enforces limits at runtime

**Best Practices:**
```rust
// Use iterators to avoid large allocations
fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
    dependencies
        .iter()
        .filter_map(|dep| analyze_single(dep))
        .collect()
}

// Process in chunks for large datasets
fn analyze_large_graph(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
    dependencies
        .chunks(100)
        .flat_map(|chunk| analyze_chunk(chunk))
        .collect()
}
```

### Execution Limits

Plugins have limited execution time:
- Measured in "fuel" (instruction count)
- Default: 1M instructions
- Host terminates runaway plugins

**Optimization Tips:**
```rust
// Early returns save execution time
if dependencies.is_empty() {
    return Vec::new();
}

// Avoid expensive operations in loops
let patterns = precompile_patterns();
for dep in dependencies {
    if patterns.matches(&dep) {
        // Process match
    }
}
```

## Data Flow

### Input Processing

1. Host discovers dependencies in codebase
2. Host converts to WIT data format
3. Host calls `plugin::analyze(dependencies)`
4. Plugin processes dependency list
5. Plugin returns list of issues

### Output Generation

1. Plugin creates `ArchitecturalIssue` records
2. Host validates issue format
3. Host integrates with main analysis results
4. Host includes in final report

## Versioning and Compatibility

### API Versioning

The plugin API uses semantic versioning:

```wit
package uveddi:plugins@0.1.0;
```

- **0.x.y**: Pre-1.0 API, breaking changes allowed
- **Major**: Breaking changes requiring plugin updates
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, no API changes

### Plugin Compatibility

Plugins specify API compatibility:

```toml
[package.metadata.component]
package = "uveddi:plugins@0.1.0"
```

### Forward Compatibility

New API versions may add:
- New optional fields to records
- New imported functions
- New exported functions (optional)

Existing plugins continue working with new hosts.

### Breaking Changes

Major version changes may:
- Change existing function signatures
- Remove deprecated functions
- Change record field types
- Modify error handling behavior

## Security Considerations

### Input Validation

Always validate input data:

```rust
fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
    let mut issues = Vec::new();
    
    for dep in dependencies {
        // Validate dependency data
        if dep.from_module.is_empty() || dep.to_module.is_empty() {
            logging::log("warn", "Invalid dependency with empty module name");
            continue;
        }
        
        if dep.from_module.len() > 1000 {
            logging::log("warn", "Suspiciously long module name");
            continue;
        }
        
        // Process valid dependency
        if let Some(issue) = analyze_dependency(&dep) {
            issues.push(issue);
        }
    }
    
    issues
}
```

### Output Sanitization

Ensure output is safe:

```rust
fn create_issue(file_path: &str, message: &str) -> ArchitecturalIssue {
    ArchitecturalIssue {
        file_path: sanitize_path(file_path),
        message: sanitize_message(message),
        // ... other fields
    }
}

fn sanitize_path(path: &str) -> String {
    // Remove potentially dangerous characters
    path.chars()
        .filter(|c| c.is_alphanumeric() || "/._-".contains(*c))
        .collect()
}
```

### Resource Usage

Monitor resource consumption:

```rust
fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
    if dependencies.len() > 10000 {
        logging::log("warn", "Large dependency graph, may hit resource limits");
    }
    
    // Process efficiently
    process_dependencies(dependencies)
}
```

## Best Practices

### Performance

1. **Minimize Allocations**: Use iterators and references
2. **Early Filtering**: Skip irrelevant dependencies early
3. **Batch Processing**: Process similar items together
4. **Cache Results**: Store expensive computations

### Usability

1. **Clear Messages**: Write helpful issue descriptions
2. **Accurate Locations**: Provide precise line numbers when possible
3. **Consistent Severity**: Use severity levels consistently
4. **Actionable Issues**: Suggest how to fix problems

### Maintainability

1. **Modular Design**: Separate analysis logic from WIT interface
2. **Comprehensive Tests**: Test with various input patterns
3. **Documentation**: Document analysis algorithms
4. **Error Handling**: Handle edge cases gracefully

---

*This API reference covers the complete WASM plugin interface for Uveddi. For implementation examples, see the Developer Guide and example plugins.*