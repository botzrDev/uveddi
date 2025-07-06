# Uveddi

[![Crates.io](https://img.shields.io/crates/v/uveddi)](https://crates.io/crates/uveddi)
[![Docs](https://docs.rs/uveddi/badge.svg)](https://docs.rs/uveddi)
[![Build Status](https://github.com/botzrDev/uveddi/workflows/CI/badge.svg)](https://github.com/botzrDev/uveddi/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Uveddi is an AI-powered CLI tool for architectural analysis of codebases, designed to identify architectural anti-patterns and prevent architectural drift.

## Features

- **Multi-language analysis**: Rust, Python, JavaScript
- **AI-powered insights**: Local and cloud AI integration
- **Privacy-focused**: All analysis happens locally
- **Extensible architecture**: Plugin system for custom detectors
- **Comprehensive reporting**: Markdown, JSON, and interactive outputs

## Installation

### Quick Install
```bash
curl -sSL https://uveddi.dev/install.sh | bash
```

### From Source
```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo install --path .
```

## Quick Start

Analyze a codebase with AI-powered insights:
```bash
uveddi analyze /path/to/code --enable-ai
```

Generate a detailed architectural report:
```bash
uveddi analyze /path/to/code --output report.md
```

## Documentation

Full documentation is available in our [documentation website](https://botzrdev.github.io/uveddi/) or locally:

- [Getting Started](docs/getting-started/installation.md) - Installation and setup
- [Configuration Guide](docs/getting-started/configuration.md) - Customizing Uveddi
- [First Steps](docs/getting-started/first-steps.md) - Running your first analysis
- [User Guide](docs/user-guide/common-use-cases.md) - Common workflows
- [API Reference](docs/api/overview.md) - Integration options
- [Development Guide](docs/development/architecture.md) - Contributing to Uveddi

Build the documentation locally:
```bash
cd docs && mdbook build
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT - See [LICENSE](LICENSE) for details.
