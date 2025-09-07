# Uveddi Plugin Development Guide

Welcome to the comprehensive guide for developing plugins for Uveddi's powerful WebAssembly plugin system. This documentation will help you create, test, and distribute custom analysis plugins that extend Uveddi's capabilities.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Plugin Architecture](#plugin-architecture)
3. [Development Setup](#development-setup)
4. [Creating Your First Plugin](#creating-your-first-plugin)
5. [Plugin API Reference](#plugin-api-reference)
6. [Testing and Validation](#testing-and-validation)
7. [Publishing to Marketplace](#publishing-to-marketplace)
8. [Best Practices](#best-practices)
9. [Troubleshooting](#troubleshooting)

## Quick Start

The fastest way to get started with plugin development is using the Uveddi plugin template generator:

```bash
# Generate a new plugin from template
uveddi plugin template generate --interactive

# Or specify parameters directly
uveddi plugin template generate \
  --template detector \
  --plugin-id my-awesome-detector \
  --name "My Awesome Detector" \
  --author "Your Name" \
  --description "Detects awesome patterns in code"

# Navigate to the generated plugin
cd my-awesome-detector

# Build the plugin
./build.sh

# Test the plugin
cargo test

# Install locally for testing
uveddi plugin install target/wasm32-wasi/release/my_awesome_detector_plugin.wasm plugin.toml
```

## Plugin Architecture

Uveddi's plugin system is built on WebAssembly (WASM) using the Component Model, providing:

### Core Features

- **Security**: Capability-based security model with WASI sandboxing
- **Performance**: Zero-copy data exchange with Apache Arrow integration
- **Portability**: WASM ensures plugins run consistently across platforms
- **Isolation**: Plugins run in isolated environments with resource limits
- **Integration**: Seamless integration with Uveddi's analysis pipeline

### Plugin Types

Uveddi supports several types of plugins:

1. **Detector Plugins**: Identify code issues and anti-patterns
2. **Security Plugins**: Scan for vulnerabilities and compliance violations  
3. **Performance Plugins**: Analyze performance characteristics and bottlenecks
4. **Documentation Plugins**: Check documentation coverage and quality
5. **Dependency Plugins**: Track and analyze project dependencies
6. **Custom Plugins**: Specialized analysis for specific domains

### WebAssembly Interface Types (WIT)

Plugins communicate with Uveddi through well-defined WIT interfaces:

```wit
// Core analysis interface (simplified)
world core-analysis {
    export initialize: func(config: plugin-config, limits: resource-limits) -> result<_, string>;
    export analyze: func(file: source-file) -> result<analysis-result, string>;
    export get-info: func() -> plugin-info;
    export cleanup: func() -> result<_, string>;

    // Host functions available to plugins
    import log: func(level: log-level, message: string);
    import read-file: func(path: string) -> result<string, string>;
    import parse-ast: func(code: string, language: string) -> result<ast-node, string>;
    // ... more host functions
}
```

## Development Setup

### Prerequisites

1. **Rust Toolchain** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-wasi
   ```

2. **Uveddi CLI** (0.9.0+)
   ```bash
   # Install from releases or build from source
   cargo install uveddi-cli
   ```

3. **Development Tools** (optional but recommended)
   ```bash
   cargo install wasm-tools
   cargo install wit-bindgen-cli
   ```

### Project Structure

A typical Uveddi plugin project follows this structure:

```
my-plugin/
├── src/
│   └── lib.rs              # Main plugin implementation
├── tests/
│   └── integration_test.rs # Integration tests
├── examples/
│   └── fixtures/           # Test files for analysis
├── docs/
│   ├── user-guide.md       # User documentation
│   └── api.md              # API documentation
├── Cargo.toml              # Rust project configuration
├── plugin.toml             # Plugin manifest
├── build.sh                # Build script
└── README.md               # Plugin description
```

## Creating Your First Plugin

### Step 1: Generate Plugin Scaffold

```bash
# Use the interactive generator
uveddi plugin template generate --interactive

# Choose from available templates:
# - detector: Basic analysis detector
# - security: Security vulnerability scanner  
# - performance: Performance analyzer
# - documentation: Documentation analyzer
# - dependency: Dependency tracker
```

### Step 2: Understand the Generated Code

The generator creates a complete plugin with:

```rust
// src/lib.rs (simplified)
wit_bindgen::generate!({
    world: "core-analysis",
    path: "../../../wit/core-analysis.wit",
});

use exports::{initialize, analyze, get_info, cleanup};

static mut PLUGIN_STATE: Option<MyPlugin> = None;

struct MyPlugin {
    config: PluginConfig,
    // Your plugin-specific state
}

// Plugin lifecycle implementation
impl initialize {
    fn call(config: PluginConfig, limits: ResourceLimits) -> Result<(), String> {
        // Initialize your plugin
        unsafe {
            PLUGIN_STATE = Some(MyPlugin::new(config));
        }
        Ok(())
    }
}

impl analyze {
    fn call(file: SourceFile) -> Result<AnalysisResult, String> {
        unsafe {
            if let Some(ref mut plugin) = PLUGIN_STATE {
                plugin.analyze_file(file)
            } else {
                Err("Plugin not initialized".to_string())
            }
        }
    }
}

// Implement get_info and cleanup similarly...
```

### Step 3: Implement Your Analysis Logic

```rust
impl MyPlugin {
    fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
        let mut issues = Vec::new();
        
        // Example: Simple text-based analysis
        let lines: Vec<&str> = file.content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if line.contains("TODO") {
                issues.push(Issue {
                    id: "TODO001".to_string(),
                    severity: SeverityLevel::Info,
                    category: IssueCategory::Quality,
                    message: "TODO comment found".to_string(),
                    description: Some("This TODO comment indicates incomplete work.".to_string()),
                    file: file.path.clone(),
                    span: Span {
                        start: Position { 
                            line: (line_num + 1) as u32, 
                            column: 1, 
                            byte_offset: 0 
                        },
                        end: Position { 
                            line: (line_num + 1) as u32, 
                            column: line.len() as u32, 
                            byte_offset: 0 
                        },
                    },
                    rule_id: Some("todo_comment".to_string()),
                    suggestion: Some("Complete the TODO or create a proper issue".to_string()),
                    fix: None,
                    metadata: vec![],
                });
            }
        }

        // Example: AST-based analysis
        if let Some(ast) = file.ast {
            let ast_issues = self.analyze_ast(&ast, &file)?;
            issues.extend(ast_issues);
        }

        // Return results
        Ok(AnalysisResult {
            issues,
            metrics: self.calculate_metrics(&file),
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0, // Set actual duration
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn analyze_ast(&self, ast: &AstNode, file: &SourceFile) -> Result<Vec<Issue>, String> {
        let mut issues = Vec::new();

        // Query for function declarations using tree-sitter syntax
        if let Ok(functions) = query_ast(ast.clone(), "(function_item) @function") {
            for func in functions {
                // Analyze each function
                if func.content.len() > 1000 { // Large function
                    issues.push(Issue {
                        id: "FUNC001".to_string(),
                        severity: SeverityLevel::Medium,
                        category: IssueCategory::Maintainability,
                        message: "Large function detected".to_string(),
                        description: Some("This function is very large and may be hard to maintain.".to_string()),
                        file: file.path.clone(),
                        span: func.span,
                        rule_id: Some("large_function".to_string()),
                        suggestion: Some("Consider breaking this function into smaller parts".to_string()),
                        fix: None,
                        metadata: vec![
                            ("function_size".to_string(), func.content.len().to_string()),
                        ],
                    });
                }
            }
        }

        Ok(issues)
    }
}
```

### Step 4: Configure Plugin Manifest

Edit `plugin.toml` to define your plugin's metadata and configuration:

```toml
[plugin]
id = "my-awesome-detector"
name = "My Awesome Detector"
version = "1.0.0"
description = "Detects awesome patterns in code"
author = "Your Name"
license = "MIT"

[plugin.capabilities]
detectors = ["todo_comment", "large_function"]
categories = ["quality", "maintainability"]
metrics = ["issues_found", "function_count"]

[plugin.configuration.options]
max_function_size = { type = "integer", default = 1000, description = "Maximum function size in characters" }
check_todos = { type = "boolean", default = true, description = "Check for TODO comments" }

[plugin.rules]
[plugin.rules.TODO001]
name = "todo_comment"
severity = "info"
message = "TODO comment found"
enabled = true

[plugin.rules.FUNC001] 
name = "large_function"
severity = "medium"
message = "Large function detected"
enabled = true
```

### Step 5: Build and Test

```bash
# Build the plugin
cargo build --target wasm32-wasi --release

# Run tests
cargo test

# Test with actual code
uveddi plugin install target/wasm32-wasi/release/my_awesome_detector_plugin.wasm plugin.toml
uveddi analyze ./src --plugin my-awesome-detector
```

## Plugin API Reference

### Core Functions

Every plugin must implement these four core functions:

#### `initialize(config: PluginConfig, limits: ResourceLimits) -> Result<(), String>`

Called when the plugin is loaded. Use this to:
- Initialize plugin state
- Parse configuration options
- Set up any required resources
- Validate plugin requirements

#### `analyze(file: SourceFile) -> Result<AnalysisResult, String>`

Main analysis function called for each file. Parameters:
- `file`: Contains file path, content, language, and optional AST

Returns `AnalysisResult` with:
- `issues`: List of detected issues
- `metrics`: Analysis metrics  
- `dependencies`: Discovered dependencies
- `exports`: Discovered exports
- `duration_ms`: Analysis duration
- `plugin_version`: Plugin version used

#### `get_info() -> PluginInfo`

Returns plugin metadata including:
- Basic information (name, version, author)
- Supported languages and capabilities
- Required permissions
- API version compatibility

#### `cleanup() -> Result<(), String>`

Called when the plugin is unloaded. Use this to:
- Clean up resources
- Save persistent state
- Log summary information

### Host Functions

Plugins can use these host functions provided by Uveddi:

#### Logging
```rust
log(LogLevel::Info, "Analysis started");
log(LogLevel::Warn, "Unusual pattern detected");
log(LogLevel::Error, "Analysis failed");
```

#### File Operations
```rust
// Read additional files
let config_content = read_file("config.toml")?;

// Check if files exist
if file_exists("package.json") {
    // Analyze package.json
}

// List files matching pattern
let rust_files = list_files("*.rs")?;
```

#### AST Operations
```rust
// Parse code to AST
let ast = parse_ast(&file.content, &file.language)?;

// Query AST using tree-sitter patterns
let functions = query_ast(ast, "(function_item) @function")?;
let variables = query_ast(ast, "(let_declaration) @var")?;
```

#### Configuration
```rust
// Get configuration values
let max_complexity = get_config("max_complexity")
    .unwrap_or("10".to_string())
    .parse::<u32>()
    .unwrap_or(10);

// Set persistent configuration
set_config("last_run", &Utc::now().to_rfc3339())?;
```

#### Network Operations (if permission granted)
```rust
// HTTP requests
let response = http_get("https://api.example.com/data", vec![
    ("User-Agent".to_string(), "MyPlugin/1.0".to_string()),
])?;

let post_response = http_post(
    "https://api.example.com/submit",
    r#"{"data": "value"}"#,
    vec![("Content-Type".to_string(), "application/json".to_string())],
)?;
```

#### Database Operations
```rust
// Cache data
db_set("analysis_cache", &serialized_data, Some(3600))?; // 1 hour TTL

// Retrieve cached data
if let Some(cached) = db_get("analysis_cache") {
    // Use cached data
}

// Remove cached data
db_delete("analysis_cache")?;
```

### Data Types

#### SourceFile
```rust
struct SourceFile {
    path: String,           // File path
    content: String,        // File content
    language: String,       // Programming language
    size: u32,             // File size in bytes
    hash: String,          // Content hash
    ast: Option<AstNode>,  // Parsed AST (if available)
}
```

#### Issue
```rust
struct Issue {
    id: String,                    // Unique issue ID
    severity: SeverityLevel,       // Critical, High, Medium, Low, Info
    category: IssueCategory,       // Security, Performance, Quality, etc.
    message: String,               // Short description
    description: Option<String>,   // Detailed description
    file: String,                  // File path
    span: Span,                   // Location in file
    rule_id: Option<String>,      // Rule identifier
    suggestion: Option<String>,    // How to fix
    fix: Option<CodeFix>,         // Automatic fix
    metadata: Vec<(String, String)>, // Additional data
}
```

#### Metrics
```rust
struct Metrics {
    lines_of_code: u32,           // Total lines of code
    lines_of_comments: u32,       // Lines of comments
    complexity: u32,              // Cyclomatic complexity
    maintainability_index: f64,   // Maintainability score (0-100)
    technical_debt_minutes: u32,  // Estimated tech debt
    custom_metrics: Vec<(String, f64)>, // Plugin-specific metrics
}
```

## Testing and Validation

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_detection() {
        let mut plugin = MyPlugin::new(PluginConfig::default());
        
        let file = SourceFile {
            path: "test.rs".to_string(),
            content: "// TODO: Implement this function\nfn placeholder() {}".to_string(),
            language: "rust".to_string(),
            size: 42,
            hash: "hash123".to_string(),
            ast: None,
        };

        let result = plugin.analyze_file(file).unwrap();
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].rule_id, Some("todo_comment".to_string()));
    }
}
```

### Integration Testing

```bash
# Create test fixtures
mkdir -p examples/fixtures

# Create test files with known patterns
cat > examples/fixtures/sample.rs << 'EOF'
// TODO: This needs implementation
pub fn complex_function() {
    // Very long function with many lines...
    // (add many lines to trigger large function detection)
}
EOF

# Test plugin against fixtures
uveddi plugin test my-awesome-detector examples/fixtures/
```

### Performance Testing

```rust
#[test]
fn test_performance_benchmark() {
    let mut plugin = MyPlugin::new(PluginConfig::default());
    
    // Large file for performance testing
    let large_content = "// TODO: test\n".repeat(10000);
    let file = SourceFile {
        path: "large.rs".to_string(),
        content: large_content,
        language: "rust".to_string(),
        size: 150000,
        hash: "hash456".to_string(),
        ast: None,
    };

    let start = std::time::Instant::now();
    let result = plugin.analyze_file(file).unwrap();
    let duration = start.elapsed();

    // Should complete within reasonable time
    assert!(duration.as_millis() < 1000);
    assert!(!result.issues.is_empty());
}
```

### Automated Testing

Use the provided test framework:

```bash
# Run comprehensive test suite
./scripts/test-plugin.sh my-awesome-detector

# Test against known patterns
uveddi plugin validate my-awesome-detector --test-suite standard

# Performance benchmarking
uveddi plugin benchmark my-awesome-detector --iterations 100
```

## Publishing to Marketplace

### Prepare for Publication

1. **Complete Documentation**
   ```bash
   # Ensure all documentation is complete
   ls docs/
   # Should contain: user-guide.md, api.md, changelog.md
   ```

2. **Validate Plugin**
   ```bash
   # Run full validation suite
   uveddi plugin validate my-awesome-detector --strict
   
   # Check security and compliance
   uveddi plugin security-audit my-awesome-detector
   ```

3. **Package Plugin**
   ```bash
   # Create distribution package
   uveddi plugin package my-awesome-detector --output my-awesome-detector-1.0.0.zip
   ```

### Submit to Marketplace

```bash
# Login to marketplace account
uveddi marketplace login

# Submit plugin for review
uveddi marketplace submit my-awesome-detector-1.0.0.zip \
  --category quality \
  --tags "todo,maintainability,code-quality" \
  --readme README.md \
  --changelog CHANGELOG.md

# Check submission status
uveddi marketplace status my-awesome-detector
```

### Marketplace Requirements

Your plugin must meet these requirements:

1. **Functionality**
   - Implements all required functions correctly
   - Handles errors gracefully
   - Provides meaningful analysis results

2. **Documentation**
   - Complete README with usage examples
   - API documentation
   - Changelog with version history

3. **Testing**
   - Comprehensive test coverage (>80%)
   - Integration tests with real code samples
   - Performance benchmarks

4. **Security**
   - Follows security best practices
   - Uses minimal required permissions
   - Handles untrusted input safely

5. **Quality**
   - Clean, well-commented code
   - Consistent coding style
   - Proper error handling

## Best Practices

### Performance Optimization

1. **Efficient Parsing**
   ```rust
   // Cache parsed ASTs when processing multiple files
   fn analyze_with_caching(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
       let ast = if let Some(ast) = file.ast {
           ast // Use provided AST
       } else {
           parse_ast(&file.content, &file.language)? // Parse if needed
       };
       
       // Use the AST for analysis...
   }
   ```

2. **Memory Management**
   ```rust
   // Use iterators to avoid collecting large datasets
   let issues: Result<Vec<_>, _> = file.content
       .lines()
       .enumerate()
       .filter_map(|(i, line)| {
           if should_analyze_line(line) {
               Some(analyze_line(i, line))
           } else {
               None
           }
       })
       .collect();
   ```

3. **Resource Limits**
   ```rust
   impl MyPlugin {
       fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
           // Respect resource limits
           if file.size > 1_000_000 { // 1MB limit
               return Err("File too large for analysis".to_string());
           }
           
           let start = std::time::Instant::now();
           // ... perform analysis ...
           
           if start.elapsed().as_secs() > 30 { // 30 second timeout
               return Err("Analysis timeout".to_string());
           }
           
           // Return results
       }
   }
   ```

### Error Handling

1. **Graceful Degradation**
   ```rust
   fn analyze_ast(&self, ast: &AstNode, file: &SourceFile) -> Result<Vec<Issue>, String> {
       let mut issues = Vec::new();
       
       // Try different analysis approaches, continue on failure
       if let Ok(functions) = query_ast(ast.clone(), "(function_item) @function") {
           issues.extend(self.analyze_functions(functions));
       } else {
           log(LogLevel::Warn, "Failed to parse functions, skipping function analysis");
       }
       
       if let Ok(classes) = query_ast(ast.clone(), "(struct_item) @struct") {
           issues.extend(self.analyze_classes(classes));
       } else {
           log(LogLevel::Warn, "Failed to parse classes, skipping class analysis");
       }
       
       Ok(issues)
   }
   ```

2. **Informative Error Messages**
   ```rust
   fn validate_configuration(&self, config: &PluginConfig) -> Result<(), String> {
       for (key, value) in &config.custom_settings {
           match key.as_str() {
               "max_complexity" => {
                   let _: u32 = value.parse()
                       .map_err(|_| format!("Invalid max_complexity value '{}': must be a positive integer", value))?;
               },
               "enable_strict_mode" => {
                   let _: bool = value.parse()
                       .map_err(|_| format!("Invalid enable_strict_mode value '{}': must be true or false", value))?;
               },
               unknown => {
                   log(LogLevel::Warn, &format!("Unknown configuration option: {}", unknown));
               }
           }
       }
       Ok(())
   }
   ```

### Security Considerations

1. **Input Validation**
   ```rust
   fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
       // Validate input parameters
       if file.path.is_empty() {
           return Err("File path cannot be empty".to_string());
       }
       
       if file.content.len() > 10_000_000 { // 10MB limit
           return Err("File too large for safe processing".to_string());
       }
       
       // Sanitize file path
       let safe_path = file.path.replace("../", "").replace("..\\", "");
       
       // Continue with analysis...
   }
   ```

2. **Resource Management**
   ```rust
   static mut ANALYSIS_COUNT: usize = 0;
   const MAX_CONCURRENT_ANALYSES: usize = 10;
   
   fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
       unsafe {
           if ANALYSIS_COUNT >= MAX_CONCURRENT_ANALYSES {
               return Err("Too many concurrent analyses".to_string());
           }
           ANALYSIS_COUNT += 1;
       }
       
       // Ensure cleanup even on error
       let result = self.perform_analysis(file);
       
       unsafe {
           ANALYSIS_COUNT -= 1;
       }
       
       result
   }
   ```

### Configuration Management

```rust
struct PluginConfig {
    max_issues_per_file: u32,
    severity_threshold: SeverityLevel,
    custom_rules: HashMap<String, RuleConfig>,
}

impl PluginConfig {
    fn from_plugin_config(config: &PluginConfig) -> Result<Self, String> {
        let mut plugin_config = Self::default();
        
        // Parse custom settings
        for (key, value) in &config.custom_settings {
            match key.as_str() {
                "max_issues" => {
                    plugin_config.max_issues_per_file = value.parse()
                        .map_err(|e| format!("Invalid max_issues value: {}", e))?;
                },
                "severity_threshold" => {
                    plugin_config.severity_threshold = match value.as_str() {
                        "critical" => SeverityLevel::Critical,
                        "high" => SeverityLevel::High,
                        "medium" => SeverityLevel::Medium,
                        "low" => SeverityLevel::Low,
                        "info" => SeverityLevel::Info,
                        _ => return Err(format!("Invalid severity threshold: {}", value)),
                    };
                },
                rule_name if rule_name.starts_with("rule.") => {
                    let rule_id = &rule_name[5..]; // Remove "rule." prefix
                    let rule_config = RuleConfig::parse(value)?;
                    plugin_config.custom_rules.insert(rule_id.to_string(), rule_config);
                },
                _ => {
                    log(LogLevel::Warn, &format!("Unknown configuration option: {}", key));
                }
            }
        }
        
        Ok(plugin_config)
    }
}
```

## Troubleshooting

### Common Issues

1. **Plugin Fails to Load**
   ```
   Error: Failed to load plugin: Invalid WASM module
   ```
   **Solution**: Ensure you're building for the correct target:
   ```bash
   cargo build --target wasm32-wasi --release
   ```

2. **Host Function Not Found**
   ```
   Error: Import `log` not found
   ```
   **Solution**: Check that your WIT file includes the required imports:
   ```wit
   import log: func(level: log-level, message: string);
   ```

3. **Plugin Initialization Fails**
   ```
   Error: Plugin initialization failed: Configuration error
   ```
   **Solution**: Validate your configuration parsing:
   ```rust
   impl initialize {
       fn call(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
           // Add detailed error checking
           log(LogLevel::Info, &format!("Initializing with config: {:?}", config));
           
           let parsed_config = MyPluginConfig::from_plugin_config(&config)
               .map_err(|e| format!("Configuration error: {}", e))?;
               
           unsafe {
               PLUGIN_STATE = Some(MyPlugin::new(parsed_config));
           }
           
           Ok(())
       }
   }
   ```

4. **AST Parsing Errors**
   ```
   Error: Failed to parse AST: Unsupported language
   ```
   **Solution**: Check language support and provide fallback:
   ```rust
   fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
       let ast = if let Some(ast) = file.ast {
           Some(ast)
       } else {
           match parse_ast(&file.content, &file.language) {
               Ok(ast) => Some(ast),
               Err(e) => {
                   log(LogLevel::Warn, &format!("AST parsing failed: {}, falling back to text analysis", e));
                   None
               }
           }
       };
       
       // Continue with analysis using available data...
   }
   ```

### Debug Logging

Enable detailed logging in your plugin:

```rust
fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
    log(LogLevel::Debug, &format!("Analyzing file: {} ({} bytes)", file.path, file.size));
    
    let start = std::time::Instant::now();
    
    // ... analysis logic ...
    
    log(LogLevel::Debug, &format!("Analysis completed in {}ms", start.elapsed().as_millis()));
    
    Ok(result)
}
```

### Testing in Isolation

Test your plugin with minimal configuration:

```bash
# Create minimal test plugin
uveddi plugin create-test-harness my-plugin \
  --config minimal.toml \
  --input examples/fixtures/simple.rs

# Run with debug output
RUST_LOG=debug uveddi analyze examples/fixtures/ --plugin my-plugin
```

### Performance Profiling

```rust
#[cfg(debug_assertions)]
fn profile_analysis(&mut self, file: &SourceFile) -> Result<AnalysisResult, String> {
    let start = std::time::Instant::now();
    
    let result = self.analyze_file_impl(file)?;
    
    let duration = start.elapsed();
    if duration.as_millis() > 100 {
        log(LogLevel::Warn, &format!("Slow analysis: {}ms for {}", duration.as_millis(), file.path));
    }
    
    Ok(result)
}
```

## Support and Community

- **Documentation**: https://docs.uveddi.dev/plugins/
- **Examples**: https://github.com/uveddi/plugin-examples
- **Community Forum**: https://community.uveddi.dev/
- **Discord**: https://discord.gg/uveddi
- **Issue Tracker**: https://github.com/uveddi/plugins/issues

## Contributing

We welcome contributions to improve the plugin system:

1. **Report Issues**: Use the GitHub issue tracker
2. **Submit PRs**: Follow our contribution guidelines
3. **Share Plugins**: Publish useful plugins to the marketplace
4. **Improve Documentation**: Help make this guide better

Happy plugin development! 🚀