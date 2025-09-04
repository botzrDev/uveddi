# Uveddi - Architectural Analysis Tool

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-v0.9.0--alpha-orange.svg)](https://github.com/botzrDev/uveddi/releases)

> **⚠️ Alpha Release v0.9.0-alpha** - Currently in alpha testing. Core features are functional but may have limitations. Use with caution in production environments.

Uveddi is a powerful static analysis tool designed to detect architectural anti-patterns and code quality issues across multiple programming languages. Built in Rust for performance and reliability, Uveddi helps development teams maintain clean, maintainable codebases.

## 🚀 Features


### Core Analysis Capabilities
- **God Object Detection** - Identify overly complex classes and modules
- **Dead Code Analysis** - Find unused functions, variables, and imports
- **Circular Dependency Detection** - Detect problematic dependency cycles
- **Tight Coupling Analysis** - Identify components with excessive dependencies
- **Magic Values Detection** - Find hardcoded constants that should be configurable

## ⚠️ Alpha Release Limitations

This is an alpha release with the following current limitations:

- **Core Analysis Engine**: Some analysis features may not be fully implemented yet
- **Test Suite**: 18 out of 679 tests currently failing (97.3% pass rate)
- **Production Use**: Not recommended for critical production environments without thorough testing
- **API Stability**: Breaking changes may occur before v1.0 release
- **Documentation**: Some features may be documented but not yet implemented

For detailed alpha testing guidance, see our [Alpha Testing Guide](ALPHA_TESTING_GUIDE.md) and [Installation Documentation](docs/02-getting-started/installation.md).

## 🛠️ Available CLI Commands

Uveddi exposes two main CLI commands:

- `analyze` — Analyze a codebase for quality issues and architectural problems
- `config` — Manage Uveddi configuration settings (show, set, validate)

> **Note:** Other commands (such as `ci` and `ui`) may exist in the codebase but are not currently available in the main CLI.

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
# Download the latest alpha 0.9
curl -L https://github.com/botzrDev/uveddi/releases/latest/download/uveddi-linux-x86_64.tar.gz | tar xz
sudo mv uveddi /usr/local/bin/
```

### From Source
```bash
# Requires Rust 1.70+
git clone https://github.com/botzrDev/uveddi.git
cd uveddi

# Fast development build (60-80% faster, recommended for contributing)
cargo build --features=dev-core --profile=dev-fast

# Production build (full feature set)
cargo build --release --features=production
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

### Full List of `analyze` Options

```
uveddi analyze <path> [OPTIONS]

--output-format [text|json|markdown|html]   Output format (default: markdown)
--output <file>                             Output file path
--enable-ai                                 Enable AI-powered analysis
--ollama-api-url <url>                      Ollama API URL for local AI
--ollama-model <model>                      Ollama model name
--dead-code-confidence <threshold>          Confidence threshold for dead code
--dead-code-library-mode                    Enable library mode for dead code
--dead-code-ignore-patterns <patterns>      Ignore patterns for dead code
--dead-code-keep-alive <symbols>            Symbols to always keep alive
--large-classes-max-loc <lines>             Max lines for large classes
--large-classes-max-methods <count>         Max methods for large classes
--large-classes-max-fields <count>          Max fields for large classes
--large-classes-max-complexity <complexity> Max complexity for large classes
--large-classes-max-lcom <score>            Max LCOM score for large classes
--large-classes-ignore-patterns <patterns>  Ignore patterns for large classes
--large-classes-min-severity <score>        Min severity for large classes
--disable-memory-optimization               Disable memory optimization
--memory-limit-gb <GB>                      Memory limit in GB
--memory-profile [small|default|large]      Memory profile
--enable-image-rendering                    Enable image rendering
--mermaid-only                              Mermaid-only mode
--rendering-service-url <url>               Rendering service URL
--no-fallback                               Disable fallback to Mermaid-only
--check-rendering-service                   Check rendering service availability
--no-diagrams                               Disable diagram generation
--max-diagrams <n>                          Max diagrams per report
--diagram-output-dir <dir>                  Output directory for diagrams
--timeout <seconds>                         Analysis timeout (default: 300)
--verbose                                   Show detailed error information
--open-dashboard                            Open dashboard after analysis
```
```

### Performance-Optimized Development
```bash
# For fastest development builds (60-80% faster)
cargo run --features=dev-minimal --profile=dev-fast -- analyze .

# Single-language analysis (70-85% faster)
cargo run --features=dev-rust-only --profile=dev-fast -- analyze ./src
cargo run --features=dev-python-only --profile=dev-fast -- analyze ./src

# Balanced development with core features
cargo run --features=dev-core --profile=dev-fast -- analyze . --output-format html
```

### Advanced Usage
```bash
# Full production analysis
cargo run --release --features=production -- analyze . --output-format html --output report.html

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
# Fast CI builds for development branches (60-80% faster)
cargo run --features=dev-minimal --profile=dev-fast -- analyze . --output-format json --output report.json


# Production builds for release pipelines
cargo run --release --features=production -- analyze . --output-format json --output report.json

<!--
## ⚠️ Note on CI Command

The `ci` command is present in the codebase but is not currently available in the main CLI. Please use `analyze` for all analysis tasks. Future releases may expose additional commands.
-->

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

### Code Formatting
- We use rustfmt for consistent formatting across the Rust codebase.
- Before committing, run: `cargo fmt --all` to automatically format the code.
- The CI pipeline enforces formatting with: `cargo fmt --all -- --check`. PRs will fail if code is not formatted.
- Optional: Install pre-commit hooks to auto-format before commits:
  - `pre-commit install`
  - Our `.pre-commit-config.yaml` runs `cargo fmt`, `cargo clippy`, and `cargo check`.



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

### User Documentation
- **[User Guide](./docs/README.md)** - Comprehensive usage documentation
- **[Configuration](./docs/03-user-guide/configuration.md)** - All configuration options
- **[Examples](./docs/05-examples/)** - Real-world usage examples

### API Reference
- **[API Reference](./docs/08-api/)** - For programmatic usage
- **[Core Library API](https://docs.rs/uveddi/latest/uveddi/)** - Complete Rust API documentation
- **Key APIs**:
  - [`uveddi::analysis::AnalysisEngine`](https://docs.rs/uveddi/latest/uveddi/analysis/struct.AnalysisEngine.html) - Main analysis orchestration
  - [`uveddi::analysis::DetectorFactory`](https://docs.rs/uveddi/latest/uveddi/analysis/struct.DetectorFactory.html) - Detector creation and management
  - [`uveddi::analysis::AnalysisConfig`](https://docs.rs/uveddi/latest/uveddi/analysis/struct.AnalysisConfig.html) - Configuration management
  - [`uveddi::sla`](https://docs.rs/uveddi/latest/uveddi/sla/) - SLA monitoring framework
  - [`uveddi::security::models::UserRole`](https://docs.rs/uveddi/latest/uveddi/security/models/enum.UserRole.html) - Security and authorization
  - [`uveddi::plugins`](https://docs.rs/uveddi/latest/uveddi/plugins/) - WebAssembly plugin system
  - [`uveddi::server`](https://docs.rs/uveddi/latest/uveddi/server/) - HTTP API endpoints

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
