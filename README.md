# Uveddi - Architectural Analysis Tool

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Alpha Release](https://img.shields.io/badge/version-v0.9.0--alpha-red.svg)](https://github.com/botzrDev/uveddi/releases)

> **🚧 Alpha Release** - This is a pre-release version for testing and feedback.

Uveddi is a powerful static analysis tool designed to detect architectural anti-patterns and code quality issues across multiple programming languages. Built in Rust for performance and reliability, Uveddi helps development teams maintain clean, maintainable codebases.

## 🚀 Features

### Core Analysis Capabilities
- **God Object Detection** - Identify overly complex classes and modules
- **Dead Code Analysis** - Find unused functions, variables, and imports
- **Circular Dependency Detection** - Detect problematic dependency cycles
- **Tight Coupling Analysis** - Identify components with excessive dependencies
- **Magic Values Detection** - Find hardcoded constants that should be configurable

### Language Support
- ✅ **Rust** - Full AST-based analysis
- ✅ **Python** - Comprehensive pattern detection
- ✅ **JavaScript** - ES6+ support with modern syntax
- ⚠️ **TypeScript** - Beta support (some limitations)
- 🔄 **More languages** - Planned for future releases

### Reporting & Visualization
- **HTML Reports** - Interactive, styled reports with issue details
- **JSON Output** - Machine-readable format for CI/CD integration
- **Multiple Templates** - Various report formats available
- **Performance Metrics** - Analysis timing and statistics

## 📦 Installation

### From Release Binary
```bash
# Download the latest alpha release
curl -L https://github.com/botzrDev/uveddi/releases/latest/download/uveddi-linux-x86_64.tar.gz | tar xz
sudo mv uveddi /usr/local/bin/
```

### From Source
```bash
# Requires Rust 1.70+
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo build --release
sudo cp target/release/uveddi /usr/local/bin/
```

## 🔧 Quick Start

### Basic Analysis
```bash
# Analyze current directory
uveddi analyze .

# Analyze specific project
uveddi analyze /path/to/project

# Generate HTML report
uveddi analyze . --output-format html --output-file report.html
```

### Advanced Usage
```bash
# Specify languages to analyze
uveddi analyze . --languages rust,python,javascript

# Include performance timing
uveddi analyze . --verbose --timing

# Custom configuration
uveddi analyze . --config ./config/my-config.toml
```

## 📊 Sample Output

```bash
$ uveddi analyze ./my-project

🔍 Analyzing project: ./my-project
📁 Languages detected: Rust, Python
🔄 Processing 127 files...

✅ Analysis completed in 2.3s

📋 Issues Summary:
  🚨 Critical: 3 issues
  ⚠️  Warning: 12 issues
  ℹ️  Info: 8 issues

🎯 Top Issues:
  • God Object: UserService.py (complexity: 156)
  • Dead Code: 5 unused functions in utils.py
  • Circular Deps: auth_module ↔ user_module

📄 Report saved: ./uveddi-report.html
```

## ⚙️ Configuration

Create a `uveddi.toml` configuration file:

```toml
[analysis]
# Languages to analyze
languages = ["rust", "python", "javascript"]

# Analysis depth
max_depth = 10
timeout_seconds = 300

[thresholds]
# God object complexity threshold
god_object_threshold = 100

# Maximum function length
max_function_lines = 50

[output]
# Default output format
format = "html"

# Include performance metrics
include_timing = true
```

## 🐛 Known Limitations (Alpha)

- **TypeScript**: Some complex type definitions may not be fully analyzed
- **Large Codebases**: Projects with >10,000 files may experience timeouts
- **Memory Usage**: Analysis of very large files (>1MB) may be slow
- **Plugin System**: Custom detectors not yet supported

## 🤝 Contributing

We welcome contributions! This is an alpha release, so feedback and bug reports are especially valuable.

### Reporting Issues
- Use our [Issue Tracker](https://github.com/botzrDev/uveddi/issues)
- Include system info: `uveddi --version`
- Provide sample code that reproduces the issue

### Development Setup
```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo build
cargo test
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

## 📚 Documentation

- **[User Guide](./docs/README.md)** - Comprehensive usage documentation
- **[Configuration](./docs/03-user-guide/configuration.md)** - All configuration options
- **[API Reference](./docs/08-api/)** - For programmatic usage
- **[Examples](./docs/05-examples/)** - Real-world usage examples

## 🚦 Roadmap

### Beta Release (Q3 2025)
- [ ] Full TypeScript support
- [ ] Real-time analysis mode
- [ ] VS Code extension
- [ ] Plugin system for custom detectors

### v1.0 Release (Q4 2025)
- [ ] Support for Java, C#, Go
- [ ] Team collaboration features
- [ ] CI/CD integrations (GitHub Actions, GitLab CI)
- [ ] Performance optimizations

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Tree-sitter** - For excellent parsing infrastructure
- **Rust Community** - For amazing ecosystem and tooling
- **Alpha Testers** - For valuable feedback and bug reports

---

**⚠️ Alpha Notice**: This software is in alpha testing. While functional, it may contain bugs and the API may change. Not recommended for production use without thorough testing.

For support, questions, or feedback: [GitHub Issues](https://github.com/botzrDev/uveddi/issues)
