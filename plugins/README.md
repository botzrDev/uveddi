# Uveddi Plugins

This directory contains the plugin system for Uveddi, supporting both native Rust plugins and secure WASM-based plugins.

## Directory Structure

- `interface/` - WIT interface definitions for WASM plugins
- `examples/` - Example plugin implementations
- `installed/` - Directory for installed WASM plugins
- `uveddi-plugin-api/` - Rust crate providing the plugin API

## Plugin Types

### Native Rust Plugins
- Built into the main Uveddi binary
- Direct access to Rust APIs
- Examples: `GodObjectDetector`, `CyclomaticComplexityDetector`

### WASM Plugins
- Sandboxed execution environment
- Capability-based security model
- Resource limits and verification
- Cross-platform compatibility

## Security Features

- **Verification Pipeline**: Hash-based integrity checks and manifest validation
- **Resource Limits**: Memory, execution time, and output size restrictions
- **Capability System**: Fine-grained permission control for filesystem and network access
- **Sandboxing**: WASI-based isolation from host system

## Installation

WASM plugins should be placed in the `installed/` directory with:
- `.wasm` file - The compiled plugin
- `.toml` file - Plugin manifest with permissions and metadata

Example:
```
plugins/installed/
├── god-object-detector.wasm
├── god-object-detector.toml
├── complexity-analyzer.wasm
└── complexity-analyzer.toml
```

## Development

See the `examples/` directory for plugin development templates and the plugin API documentation.