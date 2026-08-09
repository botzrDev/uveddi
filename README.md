# Uveddi CLI

Uveddi is a modern architectural analysis CLI that combines static code analysis with AI-assisted insights to help engineering teams audit, modernize, and govern large codebases.

**Current Version**: `1.0.0` &nbsp;|&nbsp; **Status**: Stable Release

**License**: CC-BY-NC-SA-4.0 (Non-Commercial)

## What's included

- **Multi-language coverage** – Deep Rust, Python, JavaScript, and TypeScript analysis powered by Tree-sitter
- **Anti-pattern detection** – Catch God Objects, dead code, circular dependencies, large classes, and dozens of architectural smells
- **Security scanning** – Built-in security mode with SARIF export for CI/CD pipelines
- **AI insights (optional)** – Integrate with local Ollama instances to generate remediation guidance and summaries
- **Comprehensive reporting** – Markdown, JSON, and HTML outputs with severity scoring and technical debt metrics
- **Progress tracking** – Real-time spinner-based progress display with phase tracking
- **Reliable storage** – Persistent SQLite database with migration support and structured logging
- **Complete documentation** – [CLI Reference](docs/CLI_REFERENCE.md) and [Troubleshooting Guide](docs/TROUBLESHOOTING.md)

> Uveddi 1.0.0 is the first stable release, focused on core CLI functionality. WASM plugins and AI insights ship as experimental opt-in features.

## Quick start

### Easy Installation (Recommended)

```bash
# Clone and install with automatic PATH setup
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
./deploy/install.sh
```

The installer will:
- ✓ Build the release binary
- ✓ Install to `~/.local/bin/uveddi`
- ✓ Automatically add to your PATH
- ✓ Verify the installation

After installation completes:

```bash
# Restart your terminal or source your shell config, then:
uveddi --version

# Run your first analysis
uveddi analyze /path/to/project --output-format markdown

# Generate security-focused SARIF output (optional)
uveddi analyze /path/to/project --security --export-sarif --output-format json

# Enable AI guidance when Ollama is available
OLLAMA_API_URL=http://localhost:11434 \
uveddi analyze /path/to/project --enable-ai --ollama-model deepseek-coder:6.7b
```

### Install with Cargo

If you already have Rust installed, you can build and install directly from the
repository in one command — no clone required:

```bash
cargo install --git https://github.com/botzrDev/uveddi --bin uveddi --locked
```

This builds the release binary and places it on your Cargo bin path
(`~/.cargo/bin`), which is typically already on your `PATH`.

### Manual Installation

If you prefer manual installation or the script doesn't work for your environment:

```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo build --release --bin uveddi
cp target/release/uveddi ~/.local/bin/
```

Then add `~/.local/bin` to your PATH if it's not already there.

### Uninstallation

```bash
# From the uveddi repository directory
./deploy/uninstall.sh
```

## Requirements

1. **Rust 1.70+** – Install from [https://rustup.rs/](https://rustup.rs/)
2. **Git** – For cloning the repository
3. **~50MB disk space** – For the compiled binary

Optional components:
- **Ollama** – For AI-powered code explanations and remediation guidance
- **SQLite** – Bundled automatically for persistent storage

## CLI commands

- `uveddi analyze` – Full codebase audits with configurable detectors and output formats
- `uveddi config` – Manage configuration settings (show, set, validate)
- `uveddi doctor` – Environment diagnostics and system health checks
- `uveddi init` – Interactive project setup with template selection and configuration
- `uveddi hooks` – Git hooks management (install, uninstall, list, test)
- `uveddi ci` – CI/CD integration with quality gate checks
- `uveddi migrate` – Database migration management with dry-run support

Run `uveddi --help` or `uveddi <command> --help` for detailed usage. See the [CLI Reference](docs/CLI_REFERENCE.md) for comprehensive documentation.

## Build profiles

- `cli-standard` *(default)* – Recommended profile with all core features and language support
- `cli-ai` – Adds AI workflows on top of `cli-standard` for Ollama integration
- `cli-plugins` – Enables WebAssembly plugin execution for custom detectors
- `cli-full` – Everything: AI + plugins + all features

**Security scanning** is built-in and available via the `--security` CLI flag.

Build with a specific profile:
```bash
cargo build --release --features cli-ai
```

## Testing summary

All core functionality has been verified working in v0.0.2:

✅ **Installation** – `./deploy/install.sh` successfully builds and installs to `~/.local/bin`
✅ **Version info** – Shows build timestamp, commit hash, branch, and enabled features
✅ **Doctor command** – Health checks for parsers, disk space, memory, and AI features
✅ **Config management** – Show, set, and validate configuration settings
✅ **Analysis** – Multiple output formats (Markdown, JSON) with real-time progress tracking
✅ **Init command** – Interactive project setup with template selection and AI configuration
✅ **Hooks management** – Install, uninstall, list, and test git hooks
✅ **CI integration** – Quality gate checks for CI/CD pipelines

⚠️ **Migrate command** – Requires database initialization (expected behavior for fresh installs)

## Support

For issues and questions:
- Check the [Troubleshooting Guide](docs/TROUBLESHOOTING.md) first
- Open a GitHub issue at [botzrDev/uveddi](https://github.com/botzrDev/uveddi/issues)
- See [CLI Reference](docs/CLI_REFERENCE.md) for detailed command documentation

## Security & privacy

Uveddi follows security best practices and privacy-first design principles:

- **Privacy First:** No telemetry or data collection; all analysis runs locally
- **Security Scanning:** Built-in security detectors for common vulnerabilities (SQL injection, XSS, hardcoded secrets)
- **SARIF Export:** Security findings can be exported in SARIF format for CI/CD integration
- **Report Vulnerabilities:** Use [GitHub Security Advisories](https://github.com/botzrDev/uveddi/security/advisories) or open an issue

## License & attribution

The source code is distributed under the **CC-BY-NC-SA-4.0 license** (Non-Commercial). See [LICENSE](LICENSE) for details.

### Third-party licenses

Uveddi incorporates open source components from the Rust ecosystem and other projects. See:
- [THIRD_PARTY_LICENSES.txt](THIRD_PARTY_LICENSES.txt) – Complete license texts for all 441 dependencies
- [NOTICE](NOTICE) – Third-party acknowledgements and trademarks

These files are included in all distribution packages and document our compliance with open source license obligations.

---

© 2025 Uveddi. Crafted for engineering teams that need trustworthy code intelligence.
