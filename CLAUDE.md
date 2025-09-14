# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Uveddi is a comprehensive architectural analysis tool that combines static code analysis with AI-powered insights. It's a Rust-based CLI application that analyzes codebases in multiple languages (Rust, Python, JavaScript, TypeScript) to detect anti-patterns, architectural issues, and provide actionable recommendations.

## Essential Commands

### Building
```bash
# Development build (fast compilation)
cargo build --features dev-core

# Production build with all features
cargo build --release --features production

# Minimal build for quick iterations
cargo build --features dev-minimal
```

### Testing
```bash
# Run all tests with default features
cargo test

# Run tests with specific feature set
cargo test --features dev-core
cargo test --features production

# Run specific test category
cargo test --lib
cargo test --test integration
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

### Running the Application
```bash
# Basic analysis
cargo run -- analyze ./src

# With HTML report
cargo run -- analyze ./src --output-format html --output report.html

# With AI insights (requires Ollama)
cargo run -- analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b

# Start web dashboard
cargo run -- serve --port 8888
```

### Frontend Development
```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Run tests
npm test

# Type checking
npm run type-check
```

### API Server
```bash
# Navigate to API server
cd api-server

# Install dependencies
npm install

# Start server
npm start

# Development mode with hot reload
npm run dev
```

## Architecture Overview

### Core Module Structure

The codebase follows a layered architecture with clear separation of concerns:

- **src/cli/** - Command-line interface implementation using clap
- **src/core/** - Core analysis engine and domain logic
- **src/detectors/** - Pattern detection modules (anti-patterns, security issues, etc.)
- **src/engine/** - AST parsing and processing using tree-sitter
- **src/api/** - REST API endpoints for web dashboard
- **src/plugins/** - WebAssembly plugin system for extensibility
- **src/reporting/** - Report generation (HTML, JSON, Markdown)
- **src/ai/** - LLM integration layer (Ollama, OpenAI, Anthropic)

### Key Design Patterns

1. **Plugin Architecture**: Uses WebAssembly for secure, sandboxed plugin execution
2. **Async Processing**: Tokio-based async runtime for I/O operations
3. **Parallel Analysis**: Rayon for CPU-bound parallel processing of files
4. **Knowledge Graph**: Internal representation of code relationships and dependencies
5. **Multi-Language Support**: Tree-sitter grammars for language-agnostic AST parsing

### Critical Dependencies

- **tree-sitter**: Universal AST parser for multi-language support
- **tokio**: Async runtime for concurrent operations
- **rayon**: Data parallelism for analysis tasks
- **clap**: Command-line argument parsing
- **serde**: Serialization/deserialization
- **reqwest**: HTTP client for API integrations
- **wasmtime**: WebAssembly runtime for plugins

### Feature Flags

The project uses Cargo feature flags for modular compilation:

- `dev-core`: Core functionality for development
- `tree-sitter`: Language parsing capabilities
- `production`: All features enabled
- `security`: Security scanning and authentication features
- `wasm-plugins`: Plugin system support
- `prometheus`: Metrics collection

### Integration Points

1. **LLM Providers**:
   - Local: Ollama API (default: http://localhost:11434)
   - Cloud: OpenAI, Anthropic, Google Gemini APIs

2. **Web Dashboard**:
   - React frontend in `/frontend`
   - Express.js API server in `/api-server`
   - WebSocket for real-time updates

3. **CI/CD Pipeline**:
   - GitHub Actions workflows in `.github/workflows/`
   - Multi-stage Docker builds
   - Automated testing on multiple Rust versions

## Development Workflow

### Branch Strategy
- `main`: Stable releases
- `develop`: Integration branch
- `feature/*`: Feature development
- `release/*`: Release preparation

### Pre-commit Hooks
The project uses pre-commit hooks for code quality. Install with:
```bash
pre-commit install
```

### Environment Variables
- `OLLAMA_API_URL`: Ollama API endpoint (default: http://localhost:11434)
- `OLLAMA_MODEL`: Default AI model for analysis
- `RUST_LOG`: Logging level (error, warn, info, debug, trace)
- `LOG_FORMAT`: Log format (compact or json)

## Testing Strategy

### Test Categories
- **Unit Tests**: In-module tests for individual components
- **Integration Tests**: `tests/integration/` for system-wide testing
- **Performance Tests**: Benchmarks in `benches/`
- **Real-world Tests**: `real_world_tests/` with actual codebases

### Running Specific Test Suites
```bash
# Resource management tests
./scripts/test-resource-management.sh

# API endpoint tests
./scripts/test-api-endpoints.sh

# TUI integration tests
./scripts/test-tui-analysis.sh
```

## Common Development Tasks

### Adding a New Detector
1. Create module in `src/detectors/`
2. Implement `Detector` trait
3. Register in `src/detectors/mod.rs`
4. Add tests in module
5. Update documentation

### Adding Language Support
1. Add tree-sitter grammar dependency in `Cargo.toml`
2. Create language module in `src/engine/parsers/`
3. Implement parsing logic
4. Add feature flag for the language
5. Update language detection in `src/core/language_detection.rs`

### Creating a Plugin
1. Use `scripts/generate-plugin.py` to scaffold
2. Implement plugin logic in Rust
3. Compile to WASM with `scripts/build-plugin.sh`
4. Place in `plugins/` directory
5. Test with `scripts/test-plugin.sh`

## Performance Considerations

- The codebase uses memory pooling and caching for large file analysis
- Parallel processing is CPU-bound, be mindful of thread pool sizes
- WebAssembly plugins have overhead; use native detectors for performance-critical paths
- Database operations use connection pooling (SQLite/PostgreSQL)

## Known Issues and Workarounds

- Health monitoring server is temporarily disabled in `src/main.rs` due to hanging issues
- Some monitoring dependencies are commented out pending proper configuration
- Production builds may take significant time due to feature complexity