# Plugin Development Guide

This comprehensive guide covers developing WebAssembly plugins for Uveddi's analysis engine using the WebAssembly Component Model. Plugins enable custom analysis detectors that integrate seamlessly with Uveddi's analysis pipeline while running in a secure sandboxed environment.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Plugin Architecture](#plugin-architecture)
3. [Development Setup](#development-setup)
4. [Writing Your First Plugin](#writing-your-first-plugin)
5. [Advanced Plugin Development](#advanced-plugin-development)
6. [Security and Permissions](#security-and-permissions)
7. [Performance Optimization](#performance-optimization)
8. [Testing Plugins](#testing-plugins)
9. [Building and Packaging](#building-and-packaging)
10. [Distribution](#distribution)
11. [Best Practices](#best-practices)
12. [Troubleshooting](#troubleshooting)

## Getting Started

### Prerequisites

- **Rust** (1.70+) - Install from [rustup.rs](https://rustup.rs/)
- **wasm32-wasi target** - Install with:
  ```bash
  rustup target add wasm32-wasi
  ```
- **wit-bindgen** - For generating bindings from WIT files
- **Optional**: `wasm-tools` for component model tooling:
  ```bash
  cargo install wasm-tools
  ```

### Quick Start

#### 1. Generate Plugin from Template

```bash
# Clone or copy the plugin template
cp -r templates/plugin-template plugins/my-analyzer
cd plugins/my-analyzer

# Update plugin metadata in plugin.toml and Cargo.toml
```

#### 2. Plugin Structure

```
my-analyzer/
├── Cargo.toml          # Rust project configuration
├── plugin.toml         # Plugin manifest
├── build.rs            # Build script (optional)
├── src/
│   └── lib.rs          # Plugin implementation
└── README.md           # Plugin documentation
```

#### 3. Plugin Manifest Configuration

```toml
# plugin.toml
[plugin]
name = "my-analyzer"
version = "1.0.0"
description = "Custom analysis plugin for detecting code issues"
author = "Your Name <your.email@example.com>"
license = "MIT"
homepage = "https://github.com/yourname/my-analyzer"
repository = "https://github.com/yourname/my-analyzer"

[capabilities]
# Security permissions required by the plugin
permissions = ["ReadFiles", "Logging"]

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
requires_tree_sitter = true
requires_database_access = false
requires_network_access = false

# Supported file types and languages
supported_languages = ["rust", "python", "javascript", "typescript"]
supported_extensions = [".rs", ".py", ".js", ".ts", ".jsx", ".tsx"]

[detection]
# Types of issues this plugin can detect
issue_categories = ["complexity", "maintainability", "quality"]
severity_levels = ["info", "low", "medium", "high", "critical"]

[integration]
# Integration settings
run_in_parallel = true
depends_on_plugins = []
conflicts_with_plugins = []
priority = 100  # Higher numbers run first
```

## Plugin Architecture

### WebAssembly Component Model

Uveddi plugins use the WebAssembly Component Model with WIT (WebAssembly Interface Types) for type-safe communication between the host and plugin.

```
┌─────────────────────────────────────────────────────────────┐
│                      Uveddi Host                            │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Plugin Engine │  │ AST Provider │  │ Analysis Engine  │  │
│  └───────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│          │                 │                    │            │
│          └─────────────────┼────────────────────┘            │
│                            │                                 │
│                   ┌────────▼────────┐                        │
│                   │  Host Functions │                        │
│                   │  (WIT Imports)  │                        │
│                   └────────┬────────┘                        │
└────────────────────────────┼────────────────────────────────┘
                             │
              ┌──────────────▼──────────────┐
              │     WASM Component Model    │
              │  ┌───────────────────────┐  │
              │  │   Plugin (WASM)       │  │
              │  │  - initialize()       │  │
              │  │  - analyze()          │  │
              │  │  - get_info()         │  │
              │  │  - cleanup()          │  │
              │  └───────────────────────┘  │
              └─────────────────────────────┘
```

### Plugin Types

1. **Detector Plugins**: Custom code analysis and pattern detection
2. **Rule Engine Plugins**: Organization-specific rules and policies
3. **Metrics Plugins**: Advanced code metrics and visualization
4. **Knowledge Plugins**: Domain-specific knowledge and anti-patterns

## Development Setup

### Project Structure

```
my-plugin/
├── Cargo.toml          # Rust project configuration
├── plugin.toml         # Plugin manifest
├── src/
│   └── lib.rs          # Main plugin implementation
├── tests/
│   └── integration.rs  # Integration tests
├── README.md           # Documentation
└── Makefile            # Build scripts (optional)
```

### Cargo Configuration

```toml
# Cargo.toml
[package]
name = "my-uveddi-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.16"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true
codegen-units = 1
```

## Writing Your First Plugin

### Basic Plugin Implementation

```rust
// src/lib.rs

// Generate bindings from the WIT interface
wit_bindgen::generate!({
    world: "core-analysis",
    path: "../../../wit/core-analysis.wit",
});

use std::cell::RefCell;

// Plugin state container
struct MyAnalyzerPlugin {
    state: RefCell<Option<PluginState>>,
}

struct PluginState {
    config: PluginConfig,
    analysis_count: u32,
}

impl Default for MyAnalyzerPlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

// Export the plugin
export!(MyAnalyzerPlugin);

// Implement the Guest trait (generated by wit_bindgen)
impl Guest for MyAnalyzerPlugin {
    fn initialize(config: PluginConfig, limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing My Analyzer Plugin");

        let state = PluginState {
            config,
            analysis_count: 0,
        };

        // Store state for later use
        *MY_STATE.borrow_mut() = Some(state);

        log(LogLevel::Info, "My Analyzer Plugin initialized successfully");
        Ok(())
    }

    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        log(LogLevel::Info, &format!("Analyzing {}", file.path));

        let mut issues = Vec::new();

        // Example: Check for TODO comments
        for (line_num, line) in file.content.lines().enumerate() {
            if let Some(col) = line.to_lowercase().find("todo") {
                issues.push(Issue {
                    id: format!("TODO-{}-{}", line_num, col),
                    severity: SeverityLevel::Info,
                    category: IssueCategory::Documentation,
                    message: "TODO comment found".to_string(),
                    description: Some("Consider creating an issue for this TODO".to_string()),
                    file: file.path.clone(),
                    span: Span {
                        start: Position {
                            line: line_num as u32 + 1,
                            column: col as u32,
                            byte_offset: 0,
                        },
                        end: Position {
                            line: line_num as u32 + 1,
                            column: (col + 4) as u32,
                            byte_offset: 0,
                        },
                    },
                    rule_id: Some("todo-comment".to_string()),
                    suggestion: Some("Create an issue tracker entry for this TODO".to_string()),
                    fix: None,
                    metadata: vec![],
                });
            }
        }

        // Calculate basic metrics
        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: count_comment_lines(&file.content),
            complexity: 1,
            maintainability_index: 100.0,
            technical_debt_minutes: issues.len() as u32 * 5,
            custom_metrics: vec![
                ("todo_count".to_string(), issues.len() as f64),
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
            id: "my-analyzer".to_string(),
            name: "My Analyzer Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Example plugin for demonstration".to_string(),
            author: "Your Name".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://github.com/yourname/my-analyzer".to_string()),
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
            ],
            detector_types: vec![IssueCategory::Documentation, IssueCategory::Quality],
            api_version: "1.0".to_string(),
            required_permissions: vec![],
        }
    }

    fn cleanup() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up My Analyzer Plugin");

        if let Some(state) = MY_STATE.borrow().as_ref() {
            log(LogLevel::Info, &format!(
                "Analyzed {} files during session",
                state.analysis_count
            ));
        }

        *MY_STATE.borrow_mut() = None;
        Ok(())
    }
}

// Thread-local state
thread_local! {
    static MY_STATE: RefCell<Option<PluginState>> = RefCell::new(None);
}

// Helper function to count comment lines
fn count_comment_lines(content: &str) -> u32 {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//")
                || trimmed.starts_with("#")
                || trimmed.starts_with("/*")
                || trimmed.starts_with("*")
        })
        .count() as u32
}
```

## Advanced Plugin Development

### Using Host Functions

Plugins can access Uveddi's core services through host functions defined in the WIT interface:

```rust
impl Guest for MyPlugin {
    fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
        // Log messages to host
        log(LogLevel::Info, &format!("Processing {}", file.path));

        // Parse AST using host function
        let ast = if let Some(ast) = file.ast {
            ast
        } else {
            parse_ast(&file.content, &file.language)
                .map_err(|e| format!("AST parsing failed: {}", e))?
        };

        // Query AST for function definitions
        let functions = query_ast(&ast, "(function_item) @function")
            .unwrap_or_default();

        log(LogLevel::Debug, &format!("Found {} functions", functions.len()));

        // Read configuration
        let threshold = get_config("complexity_threshold".to_string())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(10);

        // Analyze each function
        let mut issues = Vec::new();
        for func in functions {
            let complexity = calculate_complexity(&func);
            if complexity > threshold {
                issues.push(create_complexity_issue(&func, complexity, &file));
            }
        }

        Ok(AnalysisResult {
            issues,
            metrics: calculate_metrics(&file, &functions),
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0,
            plugin_version: "1.0.0".to_string(),
        })
    }
}

fn calculate_complexity(func: &AstNode) -> u32 {
    // Count decision points for cyclomatic complexity
    let mut complexity = 1;

    // Query for control flow statements
    let control_flow = [
        "(if_statement)",
        "(while_statement)",
        "(for_statement)",
        "(match_expression)",
    ];

    for query in control_flow {
        if let Ok(nodes) = query_ast(func, query) {
            complexity += nodes.len() as u32;
        }
    }

    complexity
}

fn create_complexity_issue(func: &AstNode, complexity: u32, file: &SourceFile) -> Issue {
    Issue {
        id: format!("COMP-{}", func.span.start.line),
        severity: if complexity > 20 {
            SeverityLevel::High
        } else {
            SeverityLevel::Medium
        },
        category: IssueCategory::Complexity,
        message: format!("High cyclomatic complexity: {}", complexity),
        description: Some(format!(
            "Function has complexity of {}, which exceeds the recommended maximum.",
            complexity
        )),
        file: file.path.clone(),
        span: func.span.clone(),
        rule_id: Some("high-complexity".to_string()),
        suggestion: Some("Consider breaking this function into smaller functions".to_string()),
        fix: None,
        metadata: vec![
            ("complexity".to_string(), complexity.to_string()),
        ],
    }
}
```

### Using Database for Caching

```rust
fn analyze_with_cache(file: &SourceFile) -> Result<AnalysisResult, String> {
    // Check cache first
    let cache_key = format!("analysis:{}", file.hash);

    if let Some(cached) = db_get(cache_key.clone()) {
        log(LogLevel::Debug, "Using cached analysis results");
        // In a real implementation, you'd deserialize the cached result
        // For now, we'll just skip re-analysis
    }

    // Perform analysis
    let result = perform_analysis(file)?;

    // Cache results with 1-hour TTL
    let serialized = format!("{:?}", result); // Use proper serialization
    db_set(cache_key, serialized, Some(3600))?;

    Ok(result)
}
```

### Multi-Language Support

```rust
fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
    // Dispatch to language-specific analyzer
    match file.language.as_str() {
        "rust" => analyze_rust(&file),
        "python" => analyze_python(&file),
        "javascript" | "typescript" => analyze_javascript(&file),
        _ => {
            log(LogLevel::Warn, &format!(
                "Unsupported language: {}",
                file.language
            ));
            Ok(AnalysisResult::default())
        }
    }
}

fn analyze_rust(file: &SourceFile) -> Result<AnalysisResult, String> {
    let ast = parse_ast(&file.content, "rust")?;

    // Rust-specific queries
    let unsafe_blocks = query_ast(&ast, "(unsafe_block) @unsafe")?;
    let unwraps = query_ast(&ast, "(call_expression function: (field_expression field: (field_identifier) @method (#eq? @method \"unwrap\"))) @call")?;

    let mut issues = Vec::new();

    for block in unsafe_blocks {
        issues.push(Issue {
            id: format!("RUST-UNSAFE-{}", block.span.start.line),
            severity: SeverityLevel::Medium,
            category: IssueCategory::Security,
            message: "Unsafe block detected".to_string(),
            description: Some("Consider if this unsafe block is necessary".to_string()),
            file: file.path.clone(),
            span: block.span,
            rule_id: Some("unsafe-block".to_string()),
            suggestion: Some("Document why unsafe is required here".to_string()),
            fix: None,
            metadata: vec![],
        });
    }

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

## Security and Permissions

### Capability-Based Security Model

Uveddi uses a capability-based security model where plugins must explicitly declare required permissions.

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

```toml
# plugin.toml
[capabilities]
permissions = ["ReadFiles", "Logging"]

[capabilities.limits]
max_memory_mb = 64
max_execution_seconds = 30
fuel_limit = 2000000
```

### Input Validation Best Practices

```rust
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_LINE_LENGTH: usize = 10_000;

fn validate_input(file: &SourceFile) -> Result<(), String> {
    // Check file size
    if file.content.len() > MAX_FILE_SIZE {
        return Err("File too large for analysis".to_string());
    }

    // Check for extremely long lines (potential DoS)
    for (i, line) in file.content.lines().enumerate() {
        if line.len() > MAX_LINE_LENGTH {
            log(LogLevel::Warn, &format!(
                "Line {} exceeds maximum length, truncating",
                i + 1
            ));
        }
    }

    // Validate language
    let supported = ["rust", "python", "javascript", "typescript"];
    if !supported.contains(&file.language.as_str()) {
        return Err(format!("Unsupported language: {}", file.language));
    }

    Ok(())
}
```

## Performance Optimization

### Memory Management

```rust
fn analyze_large_file(file: &SourceFile) -> Result<AnalysisResult, String> {
    let mut issues = Vec::with_capacity(100); // Pre-allocate

    // Process in chunks for large files
    let chunk_size = 1000;
    let lines: Vec<&str> = file.content.lines().collect();

    for (chunk_idx, chunk) in lines.chunks(chunk_size).enumerate() {
        let base_line = chunk_idx * chunk_size;

        for (line_idx, line) in chunk.iter().enumerate() {
            let line_num = base_line + line_idx;

            if let Some(issue) = check_line(line, line_num, file) {
                issues.push(issue);

                // Respect max issues limit
                if issues.len() >= 100 {
                    log(LogLevel::Warn, "Maximum issues reached, stopping analysis");
                    break;
                }
            }
        }
    }

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

### Caching Strategies

```rust
use std::collections::HashMap;

struct PluginState {
    config: PluginConfig,
    // Cache parsed patterns
    pattern_cache: HashMap<String, CompiledPattern>,
    // Cache file hashes to avoid re-analysis
    analyzed_files: HashMap<String, String>,
}

impl PluginState {
    fn should_analyze(&self, file: &SourceFile) -> bool {
        match self.analyzed_files.get(&file.path) {
            Some(cached_hash) => cached_hash != &file.hash,
            None => true,
        }
    }

    fn mark_analyzed(&mut self, file: &SourceFile) {
        self.analyzed_files.insert(file.path.clone(), file.hash.clone());
    }
}
```

### Performance Requirements

- **Initialization**: < 1000ms
- **Analysis**: < 100ms per file (for typical file sizes)
- **Memory Usage**: < 100MB per plugin
- **Integration Impact**: < 10% overhead on core system

## Testing Plugins

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(content: &str, language: &str) -> SourceFile {
        SourceFile {
            path: "test.rs".to_string(),
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
            "fn main() {\n    // TODO: implement\n    println!(\"Hello\");\n}",
            "rust",
        );

        // Note: In tests, you'd mock the host functions
        // This is a simplified example
        let issues = find_todos(&file.content, &file.path);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, Some("todo-comment".to_string()));
        assert_eq!(issues[0].span.start.line, 2);
    }

    #[test]
    fn test_empty_file() {
        let file = create_test_file("", "rust");
        let issues = find_todos(&file.content, &file.path);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_metrics_calculation() {
        let file = create_test_file(
            "// Comment\nfn main() {\n    println!(\"Hello\");\n}\n",
            "rust",
        );

        let comment_lines = count_comment_lines(&file.content);
        assert_eq!(comment_lines, 1);
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
    echo "Plugin integration successful"
else
    echo "Plugin integration failed"
    exit 1
fi
```

## Building and Packaging

### Build Configuration

```toml
# Cargo.toml
[package]
name = "my-uveddi-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.16"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true
codegen-units = 1
strip = true
```

### Build Process

```bash
# Build the plugin
cargo build --release --target wasm32-wasi

# The output will be at:
# target/wasm32-wasi/release/my_plugin.wasm

# Optionally optimize further with wasm-tools
wasm-tools strip target/wasm32-wasi/release/my_plugin.wasm -o my_plugin.wasm
```

### Makefile

```makefile
.PHONY: build test install clean release

PLUGIN_NAME := my_analyzer
TARGET := wasm32-wasi

# Development build
build:
	cargo build --target $(TARGET)

# Optimized production build
release:
	cargo build --release --target $(TARGET)
	wasm-tools strip target/$(TARGET)/release/$(PLUGIN_NAME).wasm -o $(PLUGIN_NAME).wasm

# Run tests
test:
	cargo test
	./test-plugin.sh

# Install plugin locally
install: release
	uveddi plugin install $(PLUGIN_NAME).wasm

# Clean build artifacts
clean:
	cargo clean
	rm -f $(PLUGIN_NAME).wasm
	rm -f test-results.json
```

## Distribution

### Publishing to Registry

```bash
# Login to plugin registry
uveddi plugin login

# Publish plugin
uveddi plugin publish ./my-plugin.wasm

# Plugin will be available at:
# https://plugins.uveddi.io/my-plugin
```

### Manual Installation

```bash
# Install from local file
uveddi plugin install ./my-plugin.wasm

# Install from URL
uveddi plugin install https://example.com/my-plugin.wasm

# Install from registry
uveddi plugin install registry:my-plugin@1.0.0
```

## Best Practices

### Code Quality

- Use proper error handling with `Result` types
- Implement comprehensive input validation
- Add detailed logging at appropriate levels
- Write extensive unit and integration tests
- Follow Rust idioms and conventions

### Security

- Validate all inputs thoroughly
- Request only minimum required permissions
- Implement resource limits
- Avoid unsafe operations when possible
- Never log sensitive information

### Performance

- Pre-allocate collections when size is known
- Use efficient data structures
- Implement caching for repeated operations
- Process data in batches for large files
- Monitor memory usage and optimize

### Integration

- Follow Uveddi's plugin interface exactly
- Use consistent error reporting patterns
- Provide meaningful metadata
- Test with real-world codebases
- Document plugin behavior clearly

## Troubleshooting

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Plugin fails to load | Invalid WASM binary | Check build target is `wasm32-wasi` |
| `Guest` trait not found | Missing wit_bindgen | Add `wit_bindgen::generate!` macro |
| Host function errors | Incorrect function usage | Check WIT interface definitions |
| Memory allocation failures | Exceeding limits | Optimize memory usage, increase limits |
| Permission denied | Missing permissions | Update plugin manifest with required permissions |

### Debug Logging

```rust
fn analyze(file: SourceFile) -> Result<AnalysisResult, String> {
    log(LogLevel::Debug, &format!("Starting analysis of {} bytes", file.content.len()));

    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let result = perform_analysis(&file)?;

    let end = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    log(LogLevel::Info, &format!(
        "Analysis completed in {}ms, found {} issues",
        end - start,
        result.issues.len()
    ));

    Ok(result)
}
```

### Performance Profiling

```bash
# Profile plugin performance
uveddi plugin profile my-plugin ./test_data

# Monitor memory usage
uveddi plugin monitor --duration 60s
```

## Resources

- [Plugin API Reference](api-reference.md)
- [Plugin Examples](examples.md)
- [WIT Interface Definition](../../../wit/core-analysis.wit)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [wit-bindgen Documentation](https://github.com/bytecodealliance/wit-bindgen)

## Support

### Getting Help
- GitHub Issues: [github.com/botzrDev/uveddi/issues](https://github.com/botzrDev/uveddi/issues)
- Discord Community: [discord.gg/uveddi](https://discord.gg/uveddi)
- Documentation: [docs.uveddi.io](https://docs.uveddi.io)

### Contributing
- Submit plugins to the community registry
- Help improve plugin documentation
- Contribute to plugin development tools
- Share examples and tutorials

This guide provides a comprehensive foundation for developing high-quality WASM plugins for Uveddi's analysis engine using the WebAssembly Component Model.
