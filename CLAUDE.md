
# Uveddi - AI-Powered Code Analysis Tool

## Project Overview
Uveddi is a Rust-based, AI-powered CLI tool for architectural analysis of codebases. It identifies architectural anti-patterns, prevents architectural drift, and provides AI-powered insights with privacy-focused local analysis. Uveddi is designed for developers and teams who want deep, actionable insights into their codebases while keeping all data private by default.

## Core Philosophy
1. **Privacy First**: All analysis happens locally—no data leaves your machine by default.
2. **Developer-Centric**: Built by developers, for developers, with real-world workflows in mind.
3. **Extensible by Design**: Plugin architecture allows custom detectors and integrations.
4. **Performance Focused**: Optimized for large codebases with intelligent caching and parallelism.
5. **AI-Enhanced**: Optional AI integration provides intelligent insights while preserving privacy.

## Supported Environments
| Environment | Support Level | Notes |
|-------------|---------------|-------|
| **Linux**   | ✅ Full        | Primary development platform |
| **macOS**   | ✅ Full        | Native Apple Silicon support |
| **Windows** | ✅ Full        | WSL2 recommended for best experience |
| **Docker**  | ✅ Full        | Official container images available |
| **CI/CD**   | ✅ Full        | GitHub Actions, GitLab CI, Jenkins |

## Key Features
- **Multi-language analysis**: Rust, Python, JavaScript, TypeScript, and universal anti-patterns
- **AI-powered insights**: Local AI (Ollama) integration with optional cloud providers (OpenAI, Anthropic, Gemini)
- **Privacy-focused**: All analysis happens locally by default; cloud AI is opt-in
- **Extensible architecture**: WASM-based plugin system for custom detectors
- **Terminal User Interface (TUI)**: Interactive TUI for analysis and configuration
- **Comprehensive reporting**: Markdown, JSON, HTML, and interactive outputs
- **Tree-sitter enabled**: Advanced parsing for accurate code analysis
- **Performance & scalability**: Parallel processing, intelligent caching, memory optimization
- **Security & RBAC**: Role-based access control, audit logging, rate limiting, and compliance validation
- **Chaos engineering**: Fault injection and resilience testing (optional)
- **Monitoring & observability**: Prometheus metrics, structured logging, and performance tracking

## Architecture & Main Modules
Uveddi follows a layered, extensible architecture:

```
┌───────────────────────────────┐
│        CLI Layer              │
└───────────────────────────────┘
            │
┌───────────────────────────────┐
│   Application Layer           │
└───────────────────────────────┘
            │
┌───────────────────────────────┐
│    Analysis Layer             │
└───────────────────────────────┘
            │
┌───────────────────────────────┐
│ Infrastructure Layer          │
└───────────────────────────────┘
```

### Core Modules
- **Analysis Engine (`src/analysis/`)**: Orchestrates static code analysis, runs detectors, manages AST cache, and builds dependency graphs.
- **AI Engine (`src/ai/`)**: Provides AI-powered analysis with local and cloud provider support, context building, and prompt templates.
- **Plugin System (`src/plugins/`)**: WASM-based plugin architecture for extensibility and secure custom detectors.
- **TUI (`src/tui/`)**: Interactive terminal UI for analysis and exploration.
- **Security (`src/security/`)**: RBAC, authentication, rate limiting, and audit logging.
- **Monitoring (`src/monitoring/`)**: Metrics collection, performance tracking, and reporting.

### Detector Categories
- **Universal Anti-patterns**: God Object, Cyclic Dependencies, Magic Values, Global State, Tight Coupling, Resource Leaks, Silent Failures, Code Duplication, Leaky Abstraction
- **Rust-specific**: Clone Abuse, Unwrap Abuse, Lifetime Complexity, Memory Management
- **Python-specific**: Data Structure Misuse, Exception Handling, OOP Issues, Performance Issues
- **JavaScript/TypeScript-specific**: Async Anti-patterns, Scope Issues, Type Coercion, DOM Issues

## Feature Flags & Compilation Options
Uveddi uses Cargo feature flags for modular builds:

- `default = ["local-ai", "tree-sitter", "memory-optimization"]`
- `tui` - Terminal user interface
- `wasm-plugins` - WebAssembly plugin system
- `chaos` - Chaos engineering features
- `sla-monitoring` - SLA monitoring and validation
- `enterprise` - All features enabled

**Build Examples:**
```bash
# Minimal build
cargo build --no-default-features --features="tree-sitter"
# Full-featured build
cargo build --features="full-featured"
# Enterprise build
cargo build --features="enterprise"
```

## Key Dependencies & Their Roles
- **tokio**: Async runtime for concurrency, file IO, and networking
- **serde, serde_json, bincode, toml**: Serialization and config
- **rusqlite**: Embedded SQLite database for caching and results
- **tree-sitter**: Core parser for building ASTs
- **reqwest**: HTTP client for AI and remote services
- **rayon**: Data parallelism for fast analysis
- **clap**: Command-line argument parsing
- **ratatui, crossterm**: Terminal UI (TUI feature)
- **prometheus, tracing**: Metrics and structured logging
- **ring, rustls, argon2**: Security and cryptography
- ...and more (see `Cargo.toml` and manual for full list)

## Common Commands
```bash
# Build with default features
cargo build --release
# Run tests with coverage
./scripts/run-coverage-tests.sh
# Run specific test suites
cargo test --test comprehensive_coverage
cargo test --test analysis_engine
# Performance benchmarks
cargo bench
# Lint and format
cargo clippy --all-features --all-targets
cargo fmt
```

## Documentation & Community
- Full documentation: `docs/` directory and [online docs](https://botzrdev.github.io/uveddi/)
- API docs: `cargo doc --open`
- Community: [Discord](https://discord.gg/uveddi) | [GitHub Discussions](https://github.com/botzrDev/uveddi/discussions)
- For detailed architecture, development, and usage, see `docs/02-user-guide/UVEDDI_COMPREHENSIVE_MANUAL.md`

## Contributing
- See `CONTRIBUTING.md` for guidelines
- Good first issues marked in GitHub
- Community and development guides in `docs/`

## Recent Development Focus
- Enhanced test coverage framework (UV-245)
- Memory optimization (UV-210, UV-26)
- Security and compliance testing
- Performance validation and benchmarking
- Production deployment pipeline
- Community release preparation

## Testing Strategy
- **Unit tests** in individual modules
- **Integration tests** in `tests/` directory
- **Performance tests** with criterion benchmarks
- **Security tests** for vulnerability detection
- **Coverage gates** enforcing 90%+ coverage thresholds
- **Regression prevention** with automated monitoring

## Key Directories
- `src/analysis/detectors/anti_patterns/` - Core detection logic
- `tests/analysis/` - Analysis test suites
- `docs/` - Complete documentation
- `scripts/` - Development and deployment scripts
- `frontend/` - Web UI components (React/TypeScript)
- `rendering-service/` - Node.js service for diagram generation

---
Uveddi follows enterprise-grade development practices with comprehensive testing, security, and scalable architecture. For full details, see the [comprehensive manual](docs/02-user-guide/UVEDDI_COMPREHENSIVE_MANUAL.md).