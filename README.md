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
- **Comprehensive reporting**: Markdown, JSON, and interactive outputs
- **Tree-sitter enabled by default**: Advanced parsing for supported languages is now always on for improved accuracy and performance. No manual configuration required.

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

Full documentation is available in the `docs/` directory. Key documents include:

- [Getting Started](./docs/01-getting-started/installation.md)
- [User Guide](./docs/02-user-guide/basic-concepts.md)
- [Developer Guide](./docs/05-development/DEVELOPER_GUIDE.md)
- [Community Guidelines](./docs/09-community/GUIDELINES.md)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 🤝 New Contributors Welcome!

Looking to contribute? We have plenty of [good first issues](https://github.com/botzrDev/uveddi/labels/good-first-issue) perfect for getting started!

- 📚 **Documentation**: Improve guides and examples
- 🧪 **Testing**: Add test coverage and cases  
- 🎨 **Frontend**: Enhance UI components
- 🔧 **Backend**: Fix bugs and add features
- 🚀 **DevOps**: Improve CI/CD and deployment

Check our [Good First Issues Guide](docs/09-community/GOOD_FIRST_ISSUES.md) to find the perfect task for your skill level.

## License

MIT - See [LICENSE](LICENSE) for details.
