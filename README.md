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
- ⚠️ TUI (development binary only)

## Documentation

Full documentation is available in the `docs/` directory and at [https://botzrdev.github.io/uveddi/](https://botzrdev.github.io/uveddi/). Key documents include:

- [Getting Started](./docs/01-getting-started/installation.md)
- [User Guide](./docs/02-user-guide/basic-concepts.md)
- [Developer Guide](./docs/05-development/DEVELOPER_GUIDE.md)
- [Community Guidelines](./docs/09-community/GUIDELINES.md)
- [Known Issues](./docs/known-issues.md)
- [Changelog](./CHANGELOG.md)

### Rust API Documentation

Generate and view HTML documentation for the public API:

```bash
cargo doc --open
```
The output is placed in `target/doc`. Documentation is also published to GitHub Pages.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Jira Integration & Commit Conventions

- All code changes must reference a Jira issue in the format `UV-XXX` (e.g., UV-154).
- Branches should be named with the issue key, e.g., `feature/UV-154-improve-logging`.
- Commit messages should follow:
  ```
  type(scope): description (UV-XXX)
  # Example:
  fix(detector): handle edge case in dead code analysis (UV-154)
  ```
- PR titles and descriptions must link to the relevant Jira issue.
- See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow and review checklist.

## 🤝 New Contributors Welcome!

Looking to contribute? We have plenty of [good first issues](https://github.com/botzrDev/uveddi/labels/good-first-issue) perfect for getting started!


## Testing

Uveddi uses Rust's built-in test framework and recommends [cargo-nextest](https://nexte.st/) for fast, isolated test runs.

### Running All Tests
```bash
cargo test
# Or, for faster runs:
cargo install cargo-nextest
cargo nextest run --all-features
```

### Running Stub Implementation Tests
```bash
cargo nextest run --lib --no-default-features
```

### Integration Tests
Integration tests are in the `tests/` directory. Each file is a separate crate and tests the public API.

### Test Utilities
- Property-based testing: [proptest](https://crates.io/crates/proptest)
- Code coverage: [cargo-tarpaulin](https://crates.io/crates/cargo-tarpaulin) or [cargo-llvm-cov](https://crates.io/crates/cargo-llvm-cov)

### Verifying Documentation Examples
```bash
cargo test --doc
```
All documentation examples are tested in CI to prevent documentation rot.

Check our [Good First Issues Guide](docs/09-community/GOOD_FIRST_ISSUES.md) to find the perfect task for your skill level.

## License

MIT - See [LICENSE](LICENSE) for details.

## Docker & Deployment

Uveddi provides a production-ready Dockerfile and docker-compose setup for local development and E2E testing.

### Build and Run with Docker
```bash
docker build -t uveddi:latest .
docker run --rm -it -v $(pwd):/workspace uveddi:latest
```

### Orchestrate with Docker Compose
```bash
docker-compose up --build
```

See [docker-compose.dev.yml](docker-compose.dev.yml) and [docker-entrypoint.sh](docker-entrypoint.sh) for details.

## Dependencies

Dependencies are managed in [Cargo.toml](Cargo.toml). To update dependencies:
```bash
cargo update
```
Key crates: `axum`, `sqlx`, `serde`, `jsonwebtoken`, `tracing`, `opentelemetry`, `tree-sitter`, and language grammars as optional features.

Security and license compliance are enforced with [cargo-audit](https://crates.io/crates/cargo-audit) and [cargo-deny](https://crates.io/crates/cargo-deny).

## Support & Community

- Discord: [https://discord.gg/uveddi](https://discord.gg/uveddi)
- GitHub Discussions: [https://github.com/botzrDev/uveddi/discussions](https://github.com/botzrDev/uveddi/discussions)

To report bugs, please create a GitHub issue and include:
- Uveddi version
- Debug log output
- Steps to reproduce
- Severity and frequency
- Screenshots or logs if possible

See [docs/reporting-bugs.md](docs/reporting-bugs.md) for more details.

## Project Status & Roadmap

- **Current Version:** 0.9.0 (Rust Edition 2021)
- **License:** MIT
- **Repository:** https://github.com/botzrDev/uveddi
- **Documentation:** https://botzrdev.github.io/uveddi/

Major ongoing efforts and epics are tracked in Jira (see UV-97, UV-154, etc.).
See the [Phased Implementation and Accountability Roadmap](docs/05-development/roadmap.md) for details on immediate, near-term, and long-term goals.
