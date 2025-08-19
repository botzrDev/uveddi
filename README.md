# Uveddi - Architectural Analysis Tool

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-v1.0.0-blue.svg)](https://github.com/botzrDev/uveddi/releases)

> **🎉 Production Release v1.0.0** - Ready for production use with comprehensive features and stability.

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
# Download the latest community release
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
uveddi analyze . --output-format html --output report.html
```

### Advanced Usage
```bash
# Specify languages to analyze
uveddi analyze . --languages rust,python,javascript

# Include performance timing
uveddi analyze . --verbose --timing

# Custom configuration
# Note: the `analyze` subcommand does not accept a `--config` flag. Manage configuration via a `uveddi.toml` file
# or the `uveddi config` subcommands (show/set/validate). Example:
#
#   # write a config file (uveddi.toml) and then validate it
#   uveddi config validate --file ./config/my-config.toml
```

### CI/CD Integration
```bash
# Quality gate for CI/CD pipelines
uveddi ci check . --debt-threshold 50 --critical-threshold 0

# Generate JSON report with summary for CI tools
uveddi analyze . --output-format json --output report.json

# The JSON report includes a summary section:
# {
#   "summary": {
#     "issuesTotal": 23,
#     "issuesBySeverity": {
#       "critical": 0,
#       "high": 3,
#       "medium": 12,
#       "low": 8
#     },
#     "filesAnalyzed": 127,
#     "debtScore": 42.5
#   },
#   "issues": [...],
#   "timing": {...}
# }

# GitHub Actions integration example
# See docs/examples/github-actions-simple.yml for a complete workflow
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

Create a `uveddi.toml` configuration file that matches the programmatic `Config` shape.

Example `uveddi.toml`:

```toml
# Optional: model used by local AI integrations (OLLAMA)
ollama_model = "deepseek-coder:6.7b-instruct-q4_0"

[dead_code]
# Confidence threshold for dead-code detection (0.0 - 1.0)
confidence_threshold = 0.9
library_mode = true
ignore_patterns = ["test", "spec", "mock"]

[large_classes]
# Thresholds for large/class complexity
max_logical_loc = 1000
max_methods = 20
max_fields = 15
max_cyclomatic_complexity = 50
max_cognitive_complexity = 40
max_lcom_score = 0.8
max_coupling = 30
ignore_patterns = ["tests/", "examples/"]
```

You can also set configuration via environment variables (see `Config::from_env()` in the source).

## 🎯 v1.0 Community Core Features

- **TypeScript**: Some complex type definitions may not be fully analyzed
- **Large Codebases**: Projects with >10,000 files may experience timeouts
- **Memory Usage**: Analysis of very large files (>1MB) may be slow
- **Plugin System**: Custom detectors not yet supported

Note: this repository is now at v1.0 Community Core. The core features are stable and ready for production use.
recorded in the project diagnostics; some features or detectors may not compile cleanly in the
current branch. If you hit build errors, please open an issue with reproduction steps and the
output from `cargo build` so the maintainers can triage.

## 🤝 Contributing

We welcome contributions! This is the v1.0 Community Core release, so feedback and feature requests are valuable.

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
- **Community Contributors** - For valuable feedback and feature requests

---

**🎯 v1.0 Community Core**: This software is production-ready with core functionality stable and tested. Advanced features may be added in future releases.

For support, questions, or feedback: [GitHub Issues](https://github.com/botzrDev/uveddi/issues)
