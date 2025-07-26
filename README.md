# Uveddi

[![Crates.io](https://img.shields.io/crates/v/uveddi)](https://crates.io/crates/uveddi)
[![Docs](https://docs.rs/uveddi/badge.svg)](https://docs.rs/uveddi)
[![Build Status](https://github.com/botzrDev/uveddi/workflows/CI/badge.svg)](https://github.com/botzrDev/uveddi/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Uveddi is an AI-powered CLI tool for architectural analysis of codebases, designed to identify architectural anti-patterns and prevent architectural drift.

## Features

- **Multi-language analysis**: Rust, Python, JavaScript, TypeScript
- **AI-powered insights**: Local (Ollama) and cloud AI integration
- **Privacy-focused**: All analysis happens locally by default
- **Terminal User Interface (TUI)**: Interactive analysis and configuration
- **Comprehensive reporting**: Markdown, JSON, and interactive outputs
- **Advanced detection**: Dead code, large classes, tight coupling, cyclic dependencies
- **Tree-sitter enabled**: Advanced parsing for accurate code analysis
- **Memory optimization**: High-performance analysis for large codebases
- **Automated testing**: Comprehensive test suite with CI/CD integration

## Installation

### Alpha Release (Current)
```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo build --release --features="alpha"
```

### Quick Install (Not Available Yet)
```bash
# Install script not yet available - use source build below
```

### From Source (Development)
```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo install --path . --features="alpha"
```

## Alpha Quick Start

**⚠️ Alpha Release Status**: The CLI interface is fully functional with comprehensive argument parsing and help system. The analysis engine is in development and will show "Analysis execution failed" errors - this is expected alpha behavior.

### Check Available Commands
```bash
# See all available commands
./target/release/uveddi --help

# See analysis options
./target/release/uveddi analyze --help

# See configuration options  
./target/release/uveddi config --help
```

### Test CLI Interface (Working)
```bash
# These commands demonstrate the CLI but will show expected analysis errors
./target/release/uveddi analyze /path/to/code --output-format=markdown
./target/release/uveddi analyze /path/to/code --output-format=json --output=report.json
./target/release/uveddi analyze /path/to/code --dead-code-confidence=0.8
```

### Interactive TUI Mode (Development Binary)
```bash
# TUI testing binary - separate from main CLI
cargo run --bin tui_test --features="tui"
```

### What Works in Alpha
- ✅ Full CLI argument parsing and validation
- ✅ Comprehensive help system
- ✅ Configuration commands
- ✅ All output format options
- ✅ Error handling and user-friendly messages
- ⚠️ Analysis engine (shows expected "execution failed" errors)
- ⚠️ TUI (development binary only)

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
