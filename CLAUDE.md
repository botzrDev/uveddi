# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Rust CLI Development
```bash
# Build the project
cargo build --release

# Run all tests  
cargo test

# Run specific test
cargo test test_name

# Run the CLI tool
./target/release/codeatlas analyze . --output-file report.md

# Run with AI enhancement
./target/release/codeatlas analyze . --enable-ai --output-file report.md

# Check formatting and linting
cargo fmt --check
cargo clippy -- -D warnings
```

### Backend Development (FastAPI)
```bash
cd backend

# Setup and run backend
make setup migrate run

# Run tests
make test
pytest

# Docker deployment
make docker-build docker-run

# Create new database migration
make new-migration
```

## Architecture Overview

CodeAtlas is a dual-architecture system with a Rust CLI and Python FastAPI backend for AI-powered code analysis.

### Core Components

**Rust CLI (`src/`):**
- `main.rs` - Entry point with clap CLI parsing
- `cli/` - Command implementations (analyze, init-local-ai, config, plugin)
- `analysis/` - Core analysis engine and anti-pattern detectors
- `ai/` - Multi-provider AI integration (OpenAI, Ollama, Anthropic, Gemini)
- `database/` - SQLite local storage with ERD-compliant schema
- `ast/` - Tree-sitter based multi-language parsing
- `models/` - Data models and serialization

**Python Backend (`backend/`):**
- FastAPI service for cloud PostgreSQL storage
- Alembic migrations for schema management
- Docker deployment with Google Cloud Run support

### AI Provider Architecture

The system uses a pluggable AI provider pattern:
- `LlmProvider` trait defines common interface
- `AiAnalysisEngine` manages multiple providers with fallback
- Supported: OpenAI (full), Ollama (local), Anthropic/Gemini (stubs)
- Configuration via environment variables or TOML files

### Analysis Pipeline

1. **File Discovery** - Walk directory for source files
2. **AST Parsing** - Tree-sitter analysis for Rust/Python/JavaScript
3. **Anti-pattern Detection** - Pluggable detectors (God Object, Cycles, etc.)
4. **Dependency Graph** - Build and analyze structural relationships
5. **AI Enhancement** - Optional AI explanations and suggestions
6. **Report Generation** - Output in JSON, Markdown, or Text formats

### Database Schema

**Local SQLite:**
- `projects` - Project metadata
- `analysis_runs` - Analysis session tracking
- `architectural_issues` - Detected problems with severity
- `anti_pattern_types` - Issue categorization

**Cloud PostgreSQL:**
- Same schema with additional cloud-specific features
- Accessible via FastAPI backend

## Configuration

**Environment Variables:**
- `OPENAI_API_KEY` - OpenAI API authentication
- `OLLAMA_MODEL` - Local model name (default: mistral:7b-instruct)
- `OLLAMA_API_URL` - Ollama server URL (default: http://localhost:11434)

**Config Files:**
- `codeatlas.toml` - Main configuration file
- Command-line args override config file values

## Docker Integration

**Local AI Development:**
```bash
# Build container with Ollama and DeepSeek-Coder
docker build -t codeatlas-local-ai .
docker run -it --rm codeatlas-local-ai

# Run analysis with local AI
cargo run --release -- analyze . --enable-ai
```

## Testing Strategy

- Unit tests in `src/` modules with `#[cfg(test)]`
- Integration tests in `tests/` directory
- AI provider mocking with `mockall` crate
- Fixture-based testing for analysis engine
- Backend tests use pytest with async support

## Plugin System

Framework exists for extending analysis capabilities:
- `AnalysisDetector` trait for custom anti-pattern detectors
- Plugin API crate at `plugins/codeatlas-plugin-api/`
- Discovery and lifecycle management (stub implementation)

## Important Implementation Notes

- All AI operations are async for non-blocking execution
- Error handling uses `thiserror` with structured error types
- Database operations support transactions for consistency
- Tree-sitter queries are cached for performance
- AI responses include hallucination mitigation strategies