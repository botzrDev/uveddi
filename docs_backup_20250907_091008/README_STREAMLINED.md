# Uveddi

A comprehensive architectural analysis tool that combines static code analysis with AI-powered insights to help developers understand and improve their codebases.

## Features

- **Multi-language Support**: Analyze Rust, Python, JavaScript, and TypeScript codebases
- **Anti-pattern Detection**: Identify God Objects, Dead Code, Circular Dependencies, and more
- **AI-Powered Insights**: Optional integration with Ollama for intelligent code explanations
- **Multiple Output Formats**: Generate HTML, JSON, and Markdown reports with interactive diagrams
- **WASM Plugin System**: Extend functionality with secure WebAssembly plugins
- **Web Dashboard**: Interactive analysis exploration through web UI
- **Performance Optimized**: Memory-efficient caching and parallel processing for large codebases

## Quick Start

```bash
# Install from source
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo build --release --features=production

# Basic analysis
uveddi analyze ./src

# Generate HTML report
uveddi analyze ./src --output-format html --output report.html

# With AI insights (requires Ollama)
uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b

# Start web dashboard
uveddi serve --port 8888
```

## Installation

### From Source
```bash
cargo install --path . --features=production
```

### Prerequisites
- Rust 1.70+ 
- Optional: Ollama for AI features
- Optional: Node.js 18+ for web dashboard development

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