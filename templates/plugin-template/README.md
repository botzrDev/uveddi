# {{plugin_name}}

{{description}}

## Overview

This is an Uveddi analysis plugin built using the WebAssembly Component Model. It integrates with Uveddi's analysis engine to provide custom code analysis capabilities.

## Requirements

- Rust 1.70+
- `wasm32-wasi` target: `rustup target add wasm32-wasi`

## Building

```bash
# Development build
cargo build --target wasm32-wasi

# Optimized release build
cargo build --release --target wasm32-wasi

# Using make
make release
```

The compiled plugin will be at `target/wasm32-wasi/release/{{plugin_name}}.wasm`.

## Installation

```bash
# Install from local file
uveddi plugin install target/wasm32-wasi/release/{{plugin_name}}.wasm

# Or install from registry
uveddi plugin install registry:{{plugin_name}}@{{version}}
```

## Configuration

Configure the plugin in your `uveddi.toml`:

```toml
[plugins.{{plugin_name}}]
enabled = true

[plugins.{{plugin_name}}.settings]
# Add your plugin-specific settings here
```

## Plugin Structure

```
{{plugin_name}}/
├── Cargo.toml              # Rust project configuration
├── plugin.toml             # Plugin manifest (metadata, permissions, limits)
├── src/
│   └── lib.rs              # Plugin implementation
├── wit/
│   └── core-analysis.wit   # WIT interface definition
├── Makefile                # Build automation
└── README.md               # This file
```

## Development

### Plugin Interface

This plugin implements the `core-analysis` WIT interface:

- `initialize(config, limits)` - Called when plugin loads
- `analyze(file)` - Called for each file to analyze
- `get_info()` - Returns plugin metadata
- `cleanup()` - Called before plugin unloads

### Host Functions

The plugin can use these host-provided functions:

- `log(level, message)` - Log messages to the host
- `parse_ast(code, language)` - Parse code to AST
- `query_ast(node, query)` - Query AST with tree-sitter patterns
- `read_file(path)` / `write_file(path, content)` - File operations
- `get_config(key)` / `set_config(key, value)` - Configuration access
- `db_get(key)` / `db_set(key, value, ttl)` - Database operations (for caching)
- `http_get(url, headers)` / `http_post(url, body, headers)` - Network requests
- `calculate_hash(algorithm, content)` - Cryptographic hashing

### Testing

```bash
# Run unit tests
cargo test

# Test plugin integration with Uveddi
uveddi plugin test {{plugin_name}} --test-file sample.rs --verbose
```

## Usage

```bash
# Analyze with this plugin
uveddi analyze ./src --plugins {{plugin_name}}

# View plugin info
uveddi plugin info {{plugin_name}}

# List all installed plugins
uveddi plugin list
```

## License

{{license}}

## Author

{{author}}
