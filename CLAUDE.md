# Uveddi - AI-Powered Code Analysis Tool

## Project Overview
Uveddi is a Rust-based AI-powered CLI tool for architectural analysis of codebases. It identifies architectural anti-patterns, prevents architectural drift, and provides AI-powered insights with privacy-focused local analysis.

## Key Features
- **Multi-language analysis**: Rust, Python, JavaScript, TypeScript
- **AI-powered insights**: Local AI (Ollama) integration with optional cloud providers
- **Privacy-focused**: All analysis happens locally by default
- **Extensible architecture**: Plugin system for custom detectors
- **Terminal User Interface**: Interactive TUI for analysis and configuration
- **Comprehensive reporting**: Markdown, JSON, and interactive outputs
- **Tree-sitter enabled**: Advanced parsing for accurate code analysis

## Project Structure

### Core Source Code (`src/`)
- **`analysis/`** - Core analysis engine with anti-pattern detection
  - `detectors/anti_patterns/` - Detection implementations (god objects, tight coupling, etc.)
  - `engine.rs` - Main analysis orchestration
  - `memory/` - Memory optimization features (UV-210, UV-26)
  - `parallel/` - Parallel processing capabilities
- **`ai/`** - AI integration and prompt engineering
  - `ollama_provider.rs` - Local AI provider implementation
  - `prompts/` - Template system for AI interactions
- **`cli/`** - Command-line interface
- **`tui/`** - Terminal user interface implementation
- **`security/`** - Security and RBAC framework
- **`monitoring/`** - Observability and metrics collection
- **`plugins/`** - Plugin system for extensibility

### Configuration
- **`Cargo.toml`** - Main package configuration with extensive feature flags
- **`config/`** - Configuration files for benchmarks and security

### Testing (`tests/`)
- **`coverage/`** - Comprehensive test coverage framework
- **`analysis/`** - Analysis engine tests
- **`security/`** - Security and compliance testing
- **`performance/`** - Performance and load testing

## Key Dependencies
- **Tree-sitter** - Multi-language parsing (enabled by default)
- **Tokio** - Async runtime
- **Reqwest** - HTTP client for AI services
- **Ratatui + Crossterm** - Terminal UI (optional via `tui` feature)
- **Rusqlite** - Local database storage
- **Memory optimization**: mimalloc, bumpalo, rkyv for zero-copy serialization

## Feature Flags
- `default = ["local-ai", "tree-sitter", "memory-optimization"]`
- `tui` - Terminal user interface
- `wasm-plugins` - WebAssembly plugin system
- `chaos` - Chaos engineering features
- `sla-monitoring` - SLA monitoring and validation
- `enterprise` - All features enabled

## Common Commands

### Build and Test
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

# Clippy linting
cargo clippy --all-features --all-targets

# Format code
cargo fmt
```

### Development Scripts
- **`./scripts/setup-dev-environment.sh`** - Development environment setup
- **`./scripts/run-coverage-tests.sh`** - Comprehensive test coverage validation
- **`./scripts/pre_release_test.sh`** - Pre-release validation
- **`./scripts/run_tui_tests.sh`** - TUI-specific testing

### Analysis Usage
```bash
# Basic analysis
cargo run -- analyze /path/to/code

# AI-powered analysis
cargo run -- analyze /path/to/code --enable-ai

# Generate report
cargo run -- analyze /path/to/code --output report.md

# TUI mode (requires 'tui' feature)
cargo run --features tui -- tui
```

### Plugin Development
Example plugin structure in `examples/plugins/excessive-comments/`:
- `Cargo.toml` - Plugin manifest
- `plugin.toml` - Plugin configuration
- `src/lib.rs` - Plugin implementation

## Architecture Notes

### Memory Optimization
- Uses custom allocators (mimalloc) for performance
- Arena allocation for temporary objects
- Zero-copy serialization with rkyv for AST caching
- Memory-mapped files for large data structures

### AI Integration
- Primarily supports local AI via Ollama for privacy
- Extensible provider system for other AI services
- Structured prompting with context building
- AI explanations for detected anti-patterns

### Security Features
- RBAC (Role-Based Access Control) framework
- OAuth2 and JWT authentication support
- Rate limiting and security middleware
- Compliance validation and audit trails

### Performance
- Parallel processing with Rayon
- Incremental analysis to avoid full re-processing
- Statistical performance regression detection
- Comprehensive benchmarking suite

## CI/CD
- **GitHub Actions** workflows for Rust CI, frontend E2E, performance testing
- **Coverage validation** with 90%+ target thresholds
- **Production deployment** pipeline with blue-green deployment
- **Dependency updates** automated via Dependabot

## Documentation
- Full documentation available at `docs/` directory
- Built with mdbook: `cd docs && mdbook build`
- API documentation: `cargo doc --open`

## Contributing
- See `CONTRIBUTING.md` for contribution guidelines
- Good first issues marked in GitHub issues
- Community guidelines in `docs/09-community/`
- Development guide in `docs/05-development/`

## Recent Development Focus
Based on git history, recent work has focused on:
- Enhanced test coverage framework (UV-245)
- Memory optimization implementation (UV-210, UV-26) 
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

## Key Directories to Know
- `src/analysis/detectors/anti_patterns/` - Core detection logic
- `tests/analysis/` - Analysis test suites  
- `docs/` - Complete documentation
- `scripts/` - Development and deployment scripts
- `frontend/` - Web UI components (React/TypeScript)
- `rendering-service/` - Node.js service for diagram generation

This project follows enterprise-grade development practices with comprehensive testing, security measures, and scalable architecture patterns.