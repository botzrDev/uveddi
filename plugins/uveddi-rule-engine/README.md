# uveddi-rule-engine

Custom rule and policy engine for organizational coding standards

## Features

- 
- Custom rule definition via TOML/YAML

- Tree-sitter query-based pattern matching

- Organizational policy enforcement

- Multi-language support

- Rule inheritance and overrides


## Building

```bash
# Build the plugin
wasm-pack build --target web --out-dir pkg

# Optimize the WASM file (optional)
wasm-opt -Oz pkg/uveddi-rule-engine_bg.wasm -o pkg/uveddi-rule-engine_opt.wasm

# Run tests
cargo test
```

## Installation

```bash
# Install via Uveddi CLI
uveddi plugin install ./pkg/uveddi-rule-engine.wasm

# Or install from registry
uveddi plugin install registry:uveddi-rule-engine@1.0.0
```

## Configuration

Create a configuration file or pass settings via the CLI:

```toml
[plugin.uveddi-rule-engine]
severity_threshold = 0.5
max_issues_per_file = 100

[plugin.uveddi-rule-engine.custom_settings]
# Add your custom settings here
```

## Usage

The plugin will be automatically used by Uveddi when analyzing code. You can also run it specifically:

```bash
# Analyze with this plugin
uveddi analyze ./src --plugins uveddi-rule-engine

# View plugin info
uveddi plugin info uveddi-rule-engine
```

## Development

### Testing

```bash
# Run unit tests
cargo test

# Run with test coverage
cargo tarpaulin --out Html
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo test

# Run with specific features
cargo test --features rust-support
```

## API Reference

The plugin implements the standard Uveddi plugin interface:

- `initialize(config)` - Initialize with configuration
- `analyze(file)` - Analyze a source file
- `get_info()` - Get plugin metadata
- `cleanup()` - Cleanup resources

## License

MIT

## Author

Uveddi Team