# uveddi-ai-linter

AI-powered code quality analysis with hallucination detection and verification

## Features

- 
- AI-powered code analysis with local and cloud models

- Hallucination detection and claim verification

- Pattern recognition for AI-generated code

- Confidence scoring and quality assessment

- Integration with Ollama and cloud AI providers


## Building

```bash
# Build the plugin
wasm-pack build --target web --out-dir pkg

# Optimize the WASM file (optional)
wasm-opt -Oz pkg/uveddi-ai-linter_bg.wasm -o pkg/uveddi-ai-linter_opt.wasm

# Run tests
cargo test
```

## Installation

```bash
# Install via Uveddi CLI
uveddi plugin install ./pkg/uveddi-ai-linter.wasm

# Or install from registry
uveddi plugin install registry:uveddi-ai-linter@1.0.0
```

## Configuration

Create a configuration file or pass settings via the CLI:

```toml
[plugin.uveddi-ai-linter]
severity_threshold = 0.5
max_issues_per_file = 100

[plugin.uveddi-ai-linter.custom_settings]
# Add your custom settings here
```

## Usage

The plugin will be automatically used by Uveddi when analyzing code. You can also run it specifically:

```bash
# Analyze with this plugin
uveddi analyze ./src --plugins uveddi-ai-linter

# View plugin info
uveddi plugin info uveddi-ai-linter
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