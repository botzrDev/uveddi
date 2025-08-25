# Plugin System Troubleshooting and FAQ

## Overview

This document provides comprehensive troubleshooting guidance and frequently asked questions for Uveddi's WASM Plugin System. It covers common issues, error messages, performance problems, and best practices for plugin development and usage.

## Troubleshooting Guide

### Plugin Installation Issues

#### Issue: Plugin fails to install with "Invalid WASM binary"

**Error Message:**
```
Error: Failed to install plugin: Invalid WASM binary format
```

**Causes & Solutions:**

1. **Wrong compilation target**
   ```bash
   # ❌ Wrong target
   cargo build --release
   
   # ✅ Correct target
   cargo build --release --target wasm32-wasi
   ```

2. **Missing WASI target**
   ```bash
   # Install WASI target
   rustup target add wasm32-wasi
   
   # Verify installation
   rustup target list --installed | grep wasm32-wasi
   ```

3. **Corrupted binary file**
   ```bash
   # Verify WASM binary format
   file target/wasm32-wasi/release/your_plugin.wasm
   # Should output: WebAssembly (wasm) binary module version 0x1 (MVP)
   
   # Check file size (shouldn't be 0 bytes)
   ls -la target/wasm32-wasi/release/your_plugin.wasm
   ```

#### Issue: Plugin manifest validation fails

**Error Message:**
```
Error: Plugin manifest validation failed: missing required field 'name'
```

**Solution:**
Ensure your `plugin.toml` has all required fields:

```toml
[plugin]
name = "your-plugin-name"          # Required
version = "1.0.0"                  # Required  
description = "Plugin description" # Required
author = "Your Name"               # Required
license = "MIT"                    # Required

[capabilities]
permissions = ["ConfigRead", "Logging"] # Required
max_memory_mb = 32                      # Required
max_execution_seconds = 30              # Required
```

#### Issue: Permission denied during installation

**Error Message:**
```
Error: Permission denied: Cannot write to plugin directory
```

**Solutions:**

1. **Check plugin directory permissions**
   ```bash
   # Check current permissions
   ls -la ~/.uveddi/plugins
   
   # Fix permissions if needed
   chmod 755 ~/.uveddi/plugins
   chown $USER ~/.uveddi/plugins
   ```

2. **Use custom plugin directory**
   ```bash
   # Set custom directory
   export UVEDDI_PLUGIN_DIR=/path/to/writable/directory
   
   # Or use command line option
   uveddi plugin install plugin.wasm --plugin-dir /custom/path
   ```

3. **Install with explicit permissions**
   ```bash
   # Create directory with correct permissions
   mkdir -p ~/.uveddi/plugins
   chmod 755 ~/.uveddi/plugins
   
   # Install plugin
   uveddi plugin install plugin.wasm
   ```

### Plugin Runtime Issues

#### Issue: Plugin execution times out

**Error Message:**
```
Error: Plugin execution timeout after 30 seconds
```

**Causes & Solutions:**

1. **Increase timeout limit**
   ```toml
   # In plugin.toml
   [capabilities]
   max_execution_seconds = 120  # Increase from 30 to 120
   ```

2. **Optimize plugin performance**
   ```rust
   // Use efficient data structures
   use std::collections::HashMap;
   
   // Pre-allocate vectors when size is known
   let mut issues = Vec::with_capacity(expected_count);
   
   // Process in batches for large inputs
   for chunk in input.lines().chunks(1000) {
       // Process chunk
   }
   ```

3. **Enable fuel optimization**
   ```toml
   # In plugin.toml
   [capabilities]
   fuel_limit = 5000000  # Increase fuel limit
   ```

#### Issue: Plugin crashes with "Memory allocation failed"

**Error Message:**
```
Error: Plugin execution failed: Memory allocation failed
```

**Solutions:**

1. **Increase memory limit**
   ```toml
   # In plugin.toml
   [capabilities]
   max_memory_mb = 64  # Increase from default 32MB
   ```

2. **Optimize memory usage**
   ```rust
   // ❌ Memory-inefficient
   let mut all_lines: Vec<String> = input.lines()
       .map(|s| s.to_string())
       .collect();
   
   // ✅ Memory-efficient
   for line in input.lines() {
       // Process line immediately without storing
       analyze_line(line);
   }
   ```

3. **Use streaming processing**
   ```rust
   // Process data in chunks
   const CHUNK_SIZE: usize = 1000;
   for chunk in input.lines().collect::<Vec<_>>().chunks(CHUNK_SIZE) {
       let chunk_result = process_chunk(chunk);
       // Process and release memory immediately
   }
   ```

#### Issue: Host function calls fail

**Error Message:**
```
Error: Host function 'host_parse_ast' not available
```

**Solutions:**

1. **Check plugin permissions**
   ```toml
   # In plugin.toml - ensure required permissions
   [capabilities]
   permissions = ["ConfigRead"]  # Required for most host functions
   ```

2. **Verify host function signature**
   ```rust
   // ✅ Correct signature
   extern "C" {
       fn host_parse_ast(code_ptr: *const u8, code_len: usize, 
                        lang_ptr: *const u8, lang_len: usize) -> u64;
   }
   
   // ❌ Wrong signature
   extern "C" {
       fn host_parse_ast(code: *const str) -> *const str;  // Invalid
   }
   ```

3. **Check Uveddi version compatibility**
   ```toml
   # In plugin.toml
   [dependencies]
   min_uveddi_version = "0.9.0"  # Ensure compatible version
   ```

### Plugin Development Issues

#### Issue: Serialization/deserialization errors

**Error Message:**
```
Error: JSON deserialization failed: missing field 'content'
```

**Solutions:**

1. **Verify data structure compatibility**
   ```rust
   // Ensure structures match Uveddi's expectations
   #[derive(Deserialize)]
   pub struct PluginFileInput {
       pub file_path: String,  // Required field
       pub content: String,    // Required field  
       pub language: String,   // Required field
   }
   ```

2. **Add error handling**
   ```rust
   // ✅ Robust error handling
   #[export_name = "analyze_file"]
   pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
       let input: PluginFileInput = match serde_json::from_slice(file_data) {
           Ok(input) => input,
           Err(e) => {
               return create_error_result(&format!("Parse error: {}", e));
           }
       };
       
       // ... rest of function
   }
   ```

3. **Test with real data**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_with_real_data() {
           let test_input = PluginFileInput {
               file_path: "test.rs".to_string(),
               content: "fn main() {}".to_string(),
               language: "rust".to_string(),
           };
           
           let input_bytes = serde_json::to_vec(&test_input).unwrap();
           let result = analyze_file(&input_bytes);
           
           assert!(!result.is_empty());
       }
   }
   ```

#### Issue: Export functions not found

**Error Message:**
```
Error: Plugin does not export required function 'analyze_file'
```

**Solutions:**

1. **Verify export names**
   ```rust
   // ✅ Correct export names
   #[export_name = "analyze_file"]
   pub fn analyze_file(file_data: &[u8]) -> Vec<u8> { /* ... */ }
   
   #[export_name = "get_plugin_info"] 
   pub fn get_plugin_info() -> Vec<u8> { /* ... */ }
   ```

2. **Check function visibility**
   ```rust
   // ✅ Functions must be public
   #[export_name = "analyze_file"]
   pub fn analyze_file(file_data: &[u8]) -> Vec<u8> { /* ... */ }
   
   // ❌ Private functions won't be exported
   #[export_name = "analyze_file"]
   fn analyze_file(file_data: &[u8]) -> Vec<u8> { /* ... */ }
   ```

3. **Verify compilation settings**
   ```toml
   # In Cargo.toml
   [lib]
   crate-type = ["cdylib"]  # Required for WASM exports
   ```

### Performance Issues

#### Issue: Plugin runs slowly

**Symptoms:**
- Analysis takes much longer than expected
- High CPU usage during plugin execution
- Memory usage grows continuously

**Solutions:**

1. **Profile plugin performance**
   ```rust
   // Add timing measurements
   let start = std::time::Instant::now();
   
   // Your analysis code here
   
   let duration = start.elapsed();
   log_info(&format!("Analysis took: {:?}", duration));
   ```

2. **Optimize algorithms**
   ```rust
   // ❌ Inefficient O(n²) algorithm
   for line1 in &lines {
       for line2 in &lines {
           if lines_similar(line1, line2) {
               // Process similarity
           }
       }
   }
   
   // ✅ More efficient approach
   use std::collections::HashMap;
   let mut line_groups: HashMap<String, Vec<usize>> = HashMap::new();
   
   for (i, line) in lines.iter().enumerate() {
       let normalized = normalize_line(line);
       line_groups.entry(normalized).or_default().push(i);
   }
   ```

3. **Use caching effectively**
   ```rust
   // Cache expensive computations
   static mut REGEX_CACHE: Option<HashMap<String, Regex>> = None;
   
   fn get_cached_regex(pattern: &str) -> &'static Regex {
       unsafe {
           if REGEX_CACHE.is_none() {
               REGEX_CACHE = Some(HashMap::new());
           }
           
           let cache = REGEX_CACHE.as_mut().unwrap();
           cache.entry(pattern.to_string())
                .or_insert_with(|| Regex::new(pattern).unwrap())
       }
   }
   ```

#### Issue: Memory usage grows excessively

**Solutions:**

1. **Monitor memory allocation**
   ```rust
   // Track memory usage
   pub struct MemoryTracker {
       allocations: usize,
       peak_usage: usize,
   }
   
   impl MemoryTracker {
       pub fn allocate(&mut self, size: usize) {
           self.allocations += size;
           self.peak_usage = self.peak_usage.max(self.allocations);
       }
       
       pub fn deallocate(&mut self, size: usize) {
           self.allocations = self.allocations.saturating_sub(size);
       }
   }
   ```

2. **Use efficient data structures**
   ```rust
   // ❌ Memory-inefficient
   let mut all_data: Vec<String> = Vec::new();
   for line in input.lines() {
       all_data.push(line.to_string()); // Stores all lines in memory
   }
   
   // ✅ Memory-efficient streaming
   for line in input.lines() {
       let result = process_line(line); // Process immediately
       if let Some(issue) = result {
           issues.push(issue);
       }
       // Line is automatically dropped here
   }
   ```

### Integration Issues

#### Issue: Plugin not detected by analysis

**Error Message:**
```
Warning: Plugin 'my-plugin' not found, skipping
```

**Solutions:**

1. **Verify plugin is installed and enabled**
   ```bash
   # List all installed plugins
   uveddi plugin list
   
   # Check specific plugin status
   uveddi plugin info my-plugin
   
   # Enable plugin if disabled
   uveddi plugin enable my-plugin
   ```

2. **Check plugin naming**
   ```bash
   # Use exact plugin name from installation
   uveddi plugin list --format table
   
   # Use correct name in analysis
   uveddi analyze ./src --plugins correct-plugin-name
   ```

3. **Verify plugin compatibility**
   ```bash
   # Test plugin functionality
   uveddi plugin test my-plugin --verbose
   
   # Check supported languages
   uveddi plugin info my-plugin --show-capabilities
   ```

#### Issue: Analysis results missing plugin data

**Solutions:**

1. **Check plugin output format**
   ```rust
   // Ensure proper result structure
   let result = PluginAnalysisResult {
       plugin_name: "my-plugin".to_string(), // Must match installed name
       issues,
       metadata: HashMap::new(),
   };
   
   serde_json::to_vec(&result).unwrap_or_default()
   ```

2. **Verify issue format compatibility**
   ```rust
   // Ensure issues have required fields
   PluginIssue {
       issue_type: "ISSUE_TYPE".to_string(),     // Required
       severity: "WARNING".to_string(),          // Required  
       message: "Issue description".to_string(), // Required
       file_path: input.file_path.clone(),       // Required
       line_number: Some(line_num + 1),         // Optional but recommended
       column: Some(column),                     // Optional
       suggestion: Some("Fix suggestion".to_string()), // Optional
   }
   ```

## Frequently Asked Questions (FAQ)

### General Questions

#### Q: What is the difference between a plugin and a built-in detector?

**A:** Built-in detectors are compiled directly into Uveddi and run with full system access. Plugins are WebAssembly modules that run in a sandboxed environment with limited permissions. Plugins offer:

- **Security**: Sandboxed execution prevents system access
- **Extensibility**: Add custom analysis without modifying Uveddi core
- **Language agnostic**: Can be written in any language that compiles to WASM
- **Hot-reloading**: Can be updated without restarting Uveddi
- **Community contributions**: Easy to share and distribute

#### Q: Can plugins access the file system?

**A:** Plugins cannot directly access the file system. They can:

- Receive file content through the `analyze_file` function parameter
- Request file operations through host functions (with appropriate permissions)
- Create temporary files in a sandboxed directory (with `TempFileCreate` permission)
- Access configuration files through host functions (with `ConfigRead` permission)

#### Q: How do I debug plugin issues?

**A:** Several debugging approaches are available:

1. **Use logging extensively**
   ```rust
   log_info(&format!("Processing file: {}", file_path));
   log_debug(&format!("Found {} issues", issues.len()));
   ```

2. **Test plugin independently**
   ```bash
   # Test with verbose output
   uveddi plugin test my-plugin --verbose --test-file sample.rs
   ```

3. **Enable debug mode**
   ```bash
   # Run with debug logging
   RUST_LOG=debug uveddi analyze ./src --plugins my-plugin
   ```

4. **Use unit tests**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn debug_plugin_behavior() {
           // Test with known inputs
           let result = analyze_file(&test_input);
           println!("Debug result: {:?}", result);
       }
   }
   ```

### Development Questions

#### Q: What languages can I use to write plugins?

**A:** Any language that can compile to WebAssembly with WASI support:

- **Rust** (recommended, best tooling and documentation)
- **C/C++** (with Emscripten or Clang)
- **Go** (with TinyGo)
- **JavaScript/TypeScript** (with Deno or Node.js WASI)
- **Python** (experimental support with Pyodide)
- **C#** (with Blazor WebAssembly)
- **AssemblyScript**

**Recommendation:** Use Rust for the best development experience and performance.

#### Q: How do I handle large files efficiently in plugins?

**A:** Use streaming and chunking approaches:

```rust
// Process file in chunks
const CHUNK_SIZE: usize = 1000;

pub fn analyze_large_file(content: &str) -> Vec<PluginIssue> {
    let mut issues = Vec::new();
    
    // Process lines in chunks to manage memory
    let lines: Vec<&str> = content.lines().collect();
    
    for chunk in lines.chunks(CHUNK_SIZE) {
        for (local_idx, line) in chunk.iter().enumerate() {
            if let Some(issue) = analyze_line(line) {
                issues.push(issue);
            }
        }
        
        // Yield control periodically
        if issues.len() % 100 == 0 {
            log_debug(&format!("Processed {} issues so far", issues.len()));
        }
    }
    
    issues
}
```

#### Q: Can plugins communicate with each other?

**A:** Direct plugin-to-plugin communication is not currently supported for security reasons. However, plugins can:

- Share data through the host system (database, configuration)
- Use dependency ordering to run in sequence
- Exchange data through temporary files (with appropriate permissions)

Future versions may support controlled inter-plugin communication through a message-passing interface.

#### Q: How do I distribute plugins?

**A:** Several distribution options are available:

1. **Direct WASM file distribution**
   ```bash
   # Users install directly
   uveddi plugin install https://releases.example.com/plugin.wasm
   ```

2. **Package with manifest**
   ```bash
   # Create distribution package
   tar -czf my-plugin-v1.0.0.tar.gz plugin.wasm plugin.toml README.md
   ```

3. **GitHub releases**
   ```yaml
   # GitHub Actions workflow for release
   - name: Create Release
     uses: actions/create-release@v1
     with:
       tag_name: v1.0.0
       release_name: My Plugin v1.0.0
       body: |
         ## Installation
         ```bash
         uveddi plugin install https://github.com/user/plugin/releases/download/v1.0.0/plugin.wasm
         ```
   ```

4. **Plugin registry** (future feature)
   ```bash
   # Future plugin registry support
   uveddi plugin install my-plugin --from-registry
   ```

### Performance Questions

#### Q: How can I optimize plugin startup time?

**A:** Several optimization strategies:

1. **Minimize initialization work**
   ```rust
   // ❌ Heavy initialization
   static EXPENSIVE_DATA: Lazy<Vec<ComplexStruct>> = Lazy::new(|| {
       load_complex_data() // This runs on first call
   });
   
   // ✅ Light initialization
   static SIMPLE_CONFIG: Lazy<Config> = Lazy::new(|| {
       Config::default() // Fast initialization
   });
   ```

2. **Use efficient data structures**
   ```rust
   // Use HashMap for O(1) lookups instead of Vec for O(n)
   let mut quick_lookup = HashMap::with_capacity(expected_size);
   ```

3. **Compile with optimizations**
   ```bash
   # Use release mode with size optimizations
   cargo build --release --target wasm32-wasi
   
   # Optional: Further optimize with wasm-opt
   wasm-opt -Os plugin.wasm -o plugin.optimized.wasm
   ```

#### Q: What are the resource limits for plugins?

**A:** Default resource limits (configurable in plugin.toml):

- **Memory**: 32MB default, up to 256MB maximum
- **Execution time**: 30 seconds default, up to 300 seconds maximum  
- **Fuel**: 2,000,000 instructions default, up to 10,000,000 maximum
- **File operations**: 100 operations default
- **Network requests**: 10 requests default (if permitted)

These limits prevent runaway plugins from consuming excessive resources.

### Security Questions

#### Q: Are plugins safe to run?

**A:** Yes, plugins run in a secure WebAssembly sandbox with multiple security layers:

- **WASM sandbox**: Cannot access host system directly
- **Capability-based security**: Must explicitly request permissions
- **Resource limits**: CPU, memory, and time limits prevent DoS
- **Permission validation**: All operations validated against declared permissions
- **Input validation**: All data sanitized before passing to plugins

#### Q: What permissions should I request for my plugin?

**A:** Request minimal permissions needed:

```toml
# Minimal permissions for basic analysis
permissions = ["ConfigRead", "Logging"]

# Add only if needed:
# "TempFileCreate" - for caching analysis results
# "DatabaseRead" - for accessing analysis history
# "NetworkConnect" - for external API calls (rare)
```

**Security best practice:** Start with minimal permissions and add only what's necessary.

#### Q: Can plugins access sensitive data?

**A:** Plugins have limited access:

- **File content**: Only through the `analyze_file` function parameter
- **Configuration**: Only explicitly allowed configuration keys
- **Network**: Only to explicitly allowed domains (if permitted)
- **File system**: Only sandboxed temporary directory (if permitted)
- **No access**: Environment variables, system files, other processes

Sensitive data should never be passed to untrusted plugins.

### Troubleshooting Commands

#### Diagnostic Commands

```bash
# Check plugin system status
uveddi plugin list --verbose

# Test specific plugin
uveddi plugin test <plugin-name> --verbose

# Check plugin information and permissions
uveddi plugin info <plugin-name> --show-permissions --show-stats

# Run analysis with debug output
RUST_LOG=debug uveddi analyze ./src --plugins <plugin-name>

# Validate plugin binary
file plugin.wasm
wasm-validate plugin.wasm

# Check plugin manifest
uveddi config validate --file plugin.toml
```

#### Log Analysis

```bash
# Enable comprehensive logging
export RUST_LOG=uveddi::plugins=debug,uveddi::analysis=debug

# Check recent plugin execution logs
journalctl -u uveddi --since "1 hour ago" | grep -i plugin

# Monitor plugin performance
uveddi analyze ./src --plugins <plugin-name> --output-format json | \
  jq '.metadata."plugin_performance"'
```

#### Recovery Commands

```bash
# Remove problematic plugin
uveddi plugin remove <plugin-name> --force

# Reinstall plugin with fresh configuration
rm -rf ~/.uveddi/plugins/<plugin-name>
uveddi plugin install plugin.wasm

# Reset plugin configuration
uveddi config set plugins.enabled false
uveddi config set plugins.enabled true

# Clear plugin cache
rm -rf ~/.uveddi/cache/plugins/
```

This troubleshooting guide should help resolve most common issues with Uveddi's plugin system. For additional support, check the project's GitHub issues or community forums.