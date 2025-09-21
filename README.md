# Uveddi

A comprehensive architectural analysis tool that combines static code analysis with AI-powered insights to help developers understand and improve their codebases.

**Current Version**: `0.9.0-alpha` - Ready for community testing and feedback

## Features

- **Multi-language Support**: Analyze Rust, Python, JavaScript, and TypeScript codebases
- **Anti-pattern Detection**: Identify God Objects, Dead Code, Circular Dependencies, and more
- **AI-Powered Insights**: Optional integration with Ollama for intelligent code explanations
- **Multiple Output Formats**: Generate HTML, JSON, and Markdown reports with interactive diagrams
- **WASM Plugin System**: Extend functionality with secure WebAssembly plugins
- **Web Dashboard**: Interactive analysis exploration through web UI
- **Performance Optimized**: Memory-efficient caching and parallel processing for large codebases
- **Security Hardened**: Zero critical vulnerabilities, production-ready security posture

## Quick Start

```bash
# Clone the repository
git clone https://github.com/botzrDev/uveddi.git
cd uveddi

# Fast development build (recommended)
cargo build --features standard

# Basic analysis
cargo run -- analyze ./src

# Generate JSON report
cargo run -- analyze ./src --output-format json

# Generate HTML report
cargo run -- analyze ./src --output-format html --output report.html

# With AI insights (requires Ollama)
cargo run -- analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b

# Start web dashboard
cargo run -- serve --port 8888
```

## Installation

### Development Builds (Faster)
```bash
# Quick development build
cargo build --features standard

# Run directly
cargo run --features dev-core -- analyze ./src
```

### Production Builds (Full Features)
```bash
# Full feature build (slower but complete)
cargo build --release --features full

# Install globally
cargo install --path . --features production
```

### Prerequisites
- Rust 1.70+ (required for core functionality)
- Optional: Ollama for AI features
- Optional: Node.js 18+ for web dashboard development

### Feature Flags
- `minimal`: Essential functionality, fastest builds
- `standard`: Recommended development profile (parsing + monitoring)
- `full`: Production-ready (all capabilities)
- `security`: Advanced security features and authentication
- `wasm-plugins`: WebAssembly plugin system support

## Documentation

- [Getting Started Guide](docs/getting-started/) - Installation and first analysis
- [User Guide](docs/user-guide/) - Complete usage documentation
- [CLI Reference](docs/reference/cli-reference.md) - All commands and options
- [Plugin Development](docs/development/plugins/) - Create custom plugins
- [API Reference](docs/reference/api-reference.md) - REST API documentation

## Project Status

Current version: **v0.9.0-alpha**

This is an alpha release. Core functionality is stable, but some features are still in development. See [known issues](docs/release-notes/known-issues.md) for details.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## Support

- [Report Issues](https://github.com/botzrDev/uveddi/issues)
- [Documentation](https://uveddi.dev/docs)
- [Community Discord](https://discord.gg/uveddi)

---

Built with 🦀 by the Uveddi Team
