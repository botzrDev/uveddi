# WASM Plugin Developer Guide

This guide explains how to develop secure WASM plugins for the Uveddi code analysis tool.

## Overview

Uveddi supports two types of plugins:
- **Native Rust plugins** - Built into the main binary with direct API access
- **WASM plugins** - Sandboxed plugins with security isolation and resource limits

This guide focuses on WASM plugin development.

## Prerequisites

### Development Environment
```bash
# Install Rust with WASM target
rustup target add wasm32-wasi

# Install WASM Component Model tools (optional, for advanced use)
cargo install wit-bindgen-cli
cargo install wasm-tools
```

### Project Structure
```
my-plugin/
├── Cargo.toml          # Rust project configuration
├── src/lib.rs          # Plugin implementation
├── plugin.toml         # Plugin manifest
└── build.sh           # Build script (optional)
```

## Quick Start

### 1. Create Plugin Project

```bash
cargo new --lib my-uveddi-plugin
cd my-uveddi-plugin
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-uveddi-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.25.0"
serde = { version = "1.0", features = ["derive"] }

[package.metadata.component]
package = "uveddi:plugins"

[package.metadata.component.target]
world = "plugin"
```

### 3. Implement Plugin

```rust
// src/lib.rs
wit_bindgen::generate!({
    world: "plugin",
    path: "../interface/plugin.wit", // Adjust path as needed
});

use exports::uveddi::plugins::plugin::Guest;
use uveddi::plugins::plugin::*;
use std::collections::HashMap;

struct MyPlugin;

impl Guest for MyPlugin {
    fn info() -> PluginInfo {
        PluginInfo {
            name: "My Analysis Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Custom analysis plugin".to_string(),
            author: "Your Name".to_string(),
        }
    }
    
    fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();
        
        // Your analysis logic here
        for dep in dependencies {
            if should_flag_dependency(&dep) {
                issues.push(ArchitecturalIssue {
                    file_path: dep.from_module,
                    start_line: 1,
                    end_line: 1,
                    issue_type: "custom_issue".to_string(),
                    severity: "medium".to_string(),
                    message: "Custom issue detected".to_string(),
                    code_snippet: None,
                });
            }
        }
        
        issues
    }
}

fn should_flag_dependency(dep: &Dependency) -> bool {
    // Implement your custom analysis logic
    dep.from_module.contains("test") && dep.dependency_type == "import"
}

export!(MyPlugin);
```

### 4. Create Plugin Manifest

```toml
# plugin.toml
[plugin]
name = "my-uveddi-plugin"
version = "0.1.0"
description = "Custom analysis plugin for detecting test imports"
author = "Your Name"

# Permissions (be restrictive!)
[[permissions]]
type = "env:read"
var = "UVEDDI_PLUGIN_CONFIG"

# Resource limits
[resource_limits]
max_memory_mb = 8
max_execution_fuel = 500_000
max_output_size_kb = 8
```

### 5. Build Plugin

```bash
# Build for WASM target
cargo build --target wasm32-wasi --release

# Copy files for installation
cp target/wasm32-wasi/release/my_uveddi_plugin.wasm plugins/installed/
cp plugin.toml plugins/installed/my_uveddi_plugin.toml
```

## Plugin Interface Reference

### Data Types

#### Dependency
```rust
struct Dependency {
    from_module: String,    // Source module name
    to_module: String,      // Target module name  
    dependency_type: String, // "import", "use", "mod", etc.
}
```

#### ArchitecturalIssue
```rust
struct ArchitecturalIssue {
    file_path: String,
    start_line: u32,
    end_line: u32,
    issue_type: String,     // Custom issue identifier
    severity: String,       // "low", "medium", "high", "critical"
    message: String,        // Human-readable description
    code_snippet: Option<String>, // Optional code context
}
```

#### PluginInfo
```rust
struct PluginInfo {
    name: String,
    version: String,
    description: String,
    author: String,
}
```

### Host Functions

#### Logging
```rust
// Available via WIT interface
logging::log("info", "Plugin message");
logging::log("warn", "Warning message");
logging::log("error", "Error message");
```

#### Configuration
```rust
// Access plugin configuration
if let Some(value) = config::get_value("my_setting") {
    // Use configuration value
}
```

## Security Model

### Capability-Based Permissions

WASM plugins run in a sandboxed environment with capability-based permissions:

#### Filesystem Access
```toml
[[permissions]]
type = "fs:read"
path = "./cache/"  # Only allow reading from cache directory
```

#### Environment Variables
```toml
[[permissions]]
type = "env:read"
var = "UVEDDI_PLUGIN_CONFIG"  # Only specific variables
```

#### Network Access (Not Yet Implemented)
```toml
[[permissions]]
type = "net:connect"
host = "api.example.com"  # Would allow specific host connections
```

### Resource Limits

All plugins are subject to resource limits:

```toml
[resource_limits]
max_memory_mb = 16        # Maximum memory usage
max_execution_fuel = 1_000_000  # Maximum instructions
max_output_size_kb = 16   # Maximum output size
```

### Security Best Practices

1. **Principle of Least Privilege**: Request only the minimum permissions needed
2. **Validate Inputs**: Always validate dependency data before processing
3. **Handle Errors Gracefully**: Use proper error handling to avoid panics
4. **Limit Resource Usage**: Be mindful of memory and CPU usage
5. **No Secrets**: Never include API keys or secrets in plugin code

## Testing and Debugging

### Local Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_analysis() {
        let deps = vec![
            Dependency {
                from_module: "test_module".to_string(),
                to_module: "external_lib".to_string(),
                dependency_type: "import".to_string(),
            }
        ];
        
        let issues = MyPlugin::analyze(deps);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, "medium");
    }
}
```

### Debug Logging

```rust
// Use host logging functions for debugging
logging::log("debug", &format!("Processing {} dependencies", dependencies.len()));
```

### Integration Testing

Place your compiled plugin in the `plugins/installed/` directory and run Uveddi:

```bash
# Build plugin
cargo build --target wasm32-wasi --release

# Install plugin
cp target/wasm32-wasi/release/my_plugin.wasm plugins/installed/
cp plugin.toml plugins/installed/my_plugin.toml

# Test with Uveddi
cargo run -- analyze ./test_project --output-file report.md
```

## Advanced Topics

### Performance Optimization

1. **Minimize Allocations**: Use iterators and avoid unnecessary cloning
2. **Batch Processing**: Process dependencies in batches when possible
3. **Early Returns**: Return early when no issues are found
4. **Cache Computed Values**: Store expensive calculations

### Error Handling

```rust
fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
    let mut issues = Vec::new();
    
    for dep in dependencies {
        match process_dependency(&dep) {
            Ok(Some(issue)) => issues.push(issue),
            Ok(None) => continue,
            Err(e) => {
                logging::log("error", &format!("Failed to process dependency: {}", e));
                continue; // Don't fail entire analysis for one bad dependency
            }
        }
    }
    
    issues
}
```

### Custom Issue Types

Define meaningful issue types for your domain:

```rust
const ISSUE_TYPES: &[&str] = &[
    "circular_dependency",
    "missing_documentation", 
    "performance_antipattern",
    "security_vulnerability",
    "architecture_violation",
];
```

## Troubleshooting

### Common Build Issues

#### Missing WASM Target
```bash
rustup target add wasm32-wasi
```

#### WIT Interface Errors
```bash
# Ensure wit-bindgen version matches
cargo update wit-bindgen
```

#### Component Model Issues
```bash
# Install wasm-tools for debugging
cargo install wasm-tools
wasm-tools validate your_plugin.wasm
```

### Runtime Issues

#### Plugin Not Loading
- Check file permissions on `.wasm` and `.toml` files
- Verify manifest format is correct TOML
- Check logs for specific error messages

#### Permission Denied
- Review requested permissions in manifest
- Ensure paths use forward slashes
- Verify environment variables exist

#### Resource Limit Exceeded
- Reduce memory usage in plugin code
- Lower resource limits in manifest
- Optimize algorithms for better performance

## Distribution and Installation

### Plugin Package Structure

```
my-plugin-v1.0.0/
├── my-plugin.wasm      # Compiled plugin
├── my-plugin.toml      # Manifest
├── README.md           # Documentation
└── LICENSE             # License file
```

### Installation

Users install plugins by copying files to the `plugins/installed/` directory:

```bash
# Manual installation
cp my-plugin.wasm $UVEDDI_HOME/plugins/installed/
cp my-plugin.toml $UVEDDI_HOME/plugins/installed/

# Or use a plugin manager (future enhancement)
uveddi plugin install my-plugin-v1.0.0.tar.gz
```

## Example Plugins

See the `plugins/examples/` directory for complete example implementations:

- `wasm-god-object-detector/` - Detects overly complex modules
- `wasm-cycle-detector/` - Finds circular dependencies  
- `wasm-security-checker/` - Identifies security anti-patterns

## API Compatibility

The plugin API follows semantic versioning:

- **Major version changes**: Breaking API changes requiring plugin updates
- **Minor version changes**: New features, backward compatible
- **Patch version changes**: Bug fixes, no API changes

Always specify the exact API version in your `Cargo.toml`:

```toml
[package.metadata.component]
package = "uveddi:plugins@0.1.0"  # Pin to specific version
```

## Support and Resources

- **Documentation**: This guide and inline code comments
- **Examples**: Reference implementations in `plugins/examples/`
- **Issues**: Report bugs and request features on GitHub
- **Community**: Join discussions in project forums

---

*For questions or support, please refer to the main Uveddi documentation or open an issue on the project repository.*