# {{plugin_name}}

{{description}}

## Features

- {{#each features}}
- {{this}}
{{/each}}

## Building

```bash
# Build the plugin
wasm-pack build --target web --out-dir pkg

# Optimize the WASM file (optional)
wasm-opt -Oz pkg/{{plugin_name}}_bg.wasm -o pkg/{{plugin_name}}_opt.wasm

# Run tests
cargo test
```

## Installation

```bash
# Install via Uveddi CLI
uveddi plugin install ./pkg/{{plugin_name}}.wasm

# Or install from registry
uveddi plugin install registry:{{plugin_name}}@{{version}}
```

## Configuration

Create a configuration file or pass settings via the CLI:

```toml
[plugin.{{plugin_name}}]
severity_threshold = 0.5
max_issues_per_file = 100

[plugin.{{plugin_name}}.custom_settings]
# Add your custom settings here
```

## Usage

The plugin will be automatically used by Uveddi when analyzing code. You can also run it specifically:

```bash
# Analyze with this plugin
uveddi analyze ./src --plugins {{plugin_name}}

# View plugin info
uveddi plugin info {{plugin_name}}
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

{{license}}

## Author

{{author}}