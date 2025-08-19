# Uveddi - Architectural Analysis Tool

## Project Overview

Uveddi is a comprehensive architectural analysis tool built in Rust that combines static code analysis with AI-powered insights to help developers understand and improve their codebases. The tool focuses on detecting anti-patterns, analyzing architectural issues, and providing intelligent explanations for code quality problems.

### Key Features
- **Multi-language AST parsing**: Support for Rust, Python, JavaScript, and TypeScript
- **AI-powered analysis**: Integration with Ollama for intelligent code insights
- **Anti-pattern detection**: God Object, Dead Code, Circular Dependencies, Tight Coupling, Magic Values
- **Dependency analysis**: Graph-based dependency tracking and visualization
- **Report generation**: HTML, JSON, and Markdown output formats with interactive diagrams
- **Terminal UI**: Interactive TUI for analysis exploration
- **Plugin system**: WebAssembly-based extensibility
- **Performance optimization**: Memory-efficient caching and parallel processing
- **Service orchestration**: Integrated web services with automatic health monitoring

### Service Orchestration
Uveddi includes a comprehensive service orchestration system that automatically manages multiple services:

#### Available Services
- **API Server** (`src/api/`): REST API endpoints for interactive reporting and dashboard integration
- **Rendering Service** (`rendering-service/`): Mermaid diagram rendering with Playwright-based browser automation
- **Frontend Dev Server** (development mode): React-based interactive dashboard

#### Service Management
- **Automatic startup**: All services start automatically with intelligent readiness detection
- **Health monitoring**: Exponential backoff retry logic with detailed error reporting
- **Graceful shutdown**: Proper cleanup of all background processes and resources
- **Port configuration**: Flexible port assignment for different environments

#### Usage
```bash
# Basic service startup
uveddi serve --port 8888 --rendering-port 3333

# Development mode with frontend
uveddi serve --port 8888 --rendering-port 3333 --frontend-port 3000 --development

# Services available at:
# - Dashboard: http://localhost:8888
# - API: http://localhost:8888/api/v1  
# - Health: http://localhost:8888/health
# - Rendering: http://localhost:3333/health
```

## Architecture

The project follows a layered architecture:

```
CLI Layer (src/cli/) → Application Layer (src/application/) → Analysis Layer (src/analysis/) → Infrastructure Layer (AST, AI, Database, Plugins) → Platform Layer
```

### Core Modules
- **`src/analysis/`**: Core analysis engines, detectors, and algorithms
- **`src/ai/`**: AI provider integrations (Ollama) for intelligent analysis
- **`src/api/`**: REST API server with shared types architecture and health monitoring
- **`src/ast/`**: Abstract Syntax Tree parsing using tree-sitter
- **`src/cli/`**: Command-line interface and argument parsing
- **`src/database/`**: SQLite-based data persistence
- **`src/report/`**: Report generation in multiple formats
- **`src/plugins/`**: WebAssembly plugin system
- **`src/service_orchestration/`**: Multi-service lifecycle management with readiness detection
- **`src/tui/`**: Terminal user interface
- **`src/cache/`**: Performance optimization through caching

## Development Workflow

### Building and Testing

```bash
# Build with default features (includes TUI, AI, tree-sitter)
cargo build --features alpha

# Run comprehensive tests
./scripts/comprehensive_test_runner.sh

# Run automated tests for TUI & CLI
./scripts/run-automated-tests.sh

# Quick coverage test
./scripts/quick-coverage.sh

# Performance validation
./scripts/performance-validation.sh
```

### Feature Flags

The project uses extensive feature flags for modular compilation:

- **`default`**: Full feature set including tree-sitter, security, memory-optimization
- **`alpha`**: TUI, tree-sitter, local-ai, security, memory-optimization
- **`tree-sitter`**: AST parsing (aggregates rust-lang, python-lang, javascript-lang, typescript-lang)
- **`ai`**: Base AI functionality
- **`local-ai`**: Ollama integration
- **`tui`**: Terminal user interface
- **`wasm-plugins`**: WebAssembly plugin system
- **`memory-optimization`**: Memory-efficient processing
- **`security`**: Security analysis features

### Testing Strategy

The project has comprehensive testing infrastructure:

1. **Unit Tests**: Core functionality testing (`tests/unit/`)
2. **Integration Tests**: End-to-end workflow testing (`tests/integration/`)
3. **TUI Tests**: Terminal interface automation (`tests/tui_*.rs`)
4. **Security Tests**: Vulnerability and compliance testing (`tests/security/`)
5. **Performance Tests**: Benchmarking and regression detection (`tests/performance/`)
6. **Coverage Tests**: Code coverage validation (`tests/coverage/`)

### Common Commands

```bash
# Analyze a project
cargo run --features alpha -- analyze ./src --output-format html --output reports/analysis.html

# Run with AI explanations (requires Ollama)
cargo run --features alpha -- analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b

# Start web services (API server + rendering service)
cargo run --features alpha -- serve --port 8888 --rendering-port 3333

# Start web services in development mode (includes frontend dev server)
cargo run --features alpha -- serve --port 8888 --rendering-port 3333 --frontend-port 3000 --development

# TUI interface
cargo run --features alpha --bin tui_test

# Run specific test suite
cargo test --features alpha --test tui_integration

# Build documentation
mdbook build docs/

# Lint and format
cargo clippy --features alpha
cargo fmt
```

### Development Environment Setup

```bash
# Clone and setup
git clone https://github.com/botzrDev/uveddi.git
cd uveddi

# Install dependencies and setup environment
./scripts/setup-dev-environment.sh

# Install development tools
cargo install mdbook
cargo install cargo-tarpaulin  # For coverage
cargo install cargo-audit     # For security audits
```

### Architectural Best Practices

#### Preventing Circular Dependencies
Uveddi uses a shared types pattern to prevent circular dependencies between modules:

```rust
// src/api/types.rs - Shared types module
use tokio::sync::oneshot;

pub struct RestApiConfig { /* ... */ }

pub trait ApiServer {
    fn start(self, database: Arc<Database>) -> impl Future<Output = Result<()>>;
    fn start_with_readiness(self, database: Arc<Database>, 
                          ready_tx: oneshot::Sender<Result<()>>) -> impl Future<Output = Result<()>>;
}
```

**Key Principles:**
- Extract shared types into dedicated modules (e.g., `src/api/types.rs`)
- Use trait-based abstractions for cross-module communication
- Implement readiness signaling for service coordination
- Avoid direct module-to-module imports that create cycles

#### Service Orchestration Patterns
- Use `tokio::sync::oneshot` channels for readiness notification
- Implement exponential backoff in health check systems
- Provide detailed error context in service failure scenarios
- Design for graceful shutdown and resource cleanup

### AI Integration

The project integrates with Ollama for AI-powered analysis:

```bash
# Start Ollama (if using AI features)
ollama serve

# Pull recommended model
ollama pull deepseek-coder:6.7b

# Set environment variables
export OLLAMA_API_URL="http://localhost:11434"
export OLLAMA_MODEL="deepseek-coder:6.7b"
```

### Configuration

Configuration can be managed through:
- Command-line arguments
- Environment variables
- Configuration files (`uveddi.toml`)
- Per-user settings (`~/.config/uveddi/`)

Example configuration:
```toml
[analysis]
languages = ["rust", "python", "javascript", "typescript"]
max_depth = 10
timeout_seconds = 300

[thresholds]
god_object_threshold = 100
max_function_lines = 50

[ai]
provider = "ollama"
model = "deepseek-coder:6.7b"
api_url = "http://localhost:11434"

[output]
format = "html"
include_timing = true
```

### Performance Considerations

- **Memory optimization**: Uses arena allocation and memory mapping for large codebases
- **Parallel processing**: Multi-threaded analysis for improved performance
- **Caching**: AST parsing results cached for repeated analysis
- **Incremental analysis**: Only re-analyze changed files
- **Feature gating**: Compile only needed functionality

### Documentation

Comprehensive documentation is available:
- **User Guide**: `docs/03-user-guide/`
- **Developer Guide**: `docs/04-development/`
- **API Reference**: `docs/08-api/`
- **Architecture**: `docs/01-architecture/`
- **Examples**: `docs/05-examples/`

### Security

The project includes security features:
- Input validation and sanitization
- Secure dependency management
- Vulnerability scanning in CI/CD
- Security-focused testing suite
- RBAC and authentication systems

### Contributing

1. Read the [Contributing Guide](docs/06-community/CONTRIBUTING.md)
2. Check [Good First Issues](docs/06-community/GOOD_FIRST_ISSUES.md)
3. Follow the [Code of Conduct](docs/06-community/CODE_OF_CONDUCT.md)
4. Use the comprehensive testing framework before submitting PRs

### Troubleshooting

Common issues and solutions:
- **Build timeouts**: Use `scripts/debug-build-timeouts.sh`
- **Memory issues**: Enable memory-optimization feature
- **TUI problems**: Check terminal compatibility and run TUI tests
- **AI integration**: Verify Ollama installation and model availability
- **Service startup failures**: Check port availability and install Playwright dependencies
- **Rendering service issues**: Run `npx playwright install` and `npx playwright install-deps`
- **Health check timeouts**: Services use exponential backoff retry logic with detailed error reporting
- **Circular dependencies**: Use shared types pattern in `src/api/types.rs` for cross-module communication

### Release Information

Current version: **v0.9.0-alpha**
- This is an alpha release for testing and feedback
- Full TypeScript support and plugin system coming in beta
- Production readiness targeted for v1.0

The project maintains high code quality standards with extensive testing, comprehensive documentation, and a focus on performance and security.

## Specialized Claude Code Agents

This project includes a comprehensive suite of specialized agents located in `.claude/agents/` that provide expert-level assistance for different aspects of the Uveddi codebase. These agents can be invoked through Claude Code to perform specific analysis and development tasks.

### Available Agents

#### 🔧 **code-parser-engine** (Foundation Agent)
**Purpose**: Foundational code parsing and structural analysis
**When to use**: Start of any analysis workflow, when you need AST parsing, CFGs, or dependency graphs
**Expertise**: Tree-sitter parsing, AST generation, control flow analysis, program dependence graphs
```bash
# Use when: Beginning any code analysis, after major code changes
# Example: "Parse the TypeScript modules to understand the new structure"
```

#### 🏗️ **architecture-analyzer** 
**Purpose**: Detect architectural anti-patterns and design violations
**When to use**: After refactoring, when suspecting architectural issues, code quality reviews
**Expertise**: SOLID principles, God Objects, Tight Coupling, Cyclic Dependencies, design patterns
```bash
# Use when: "Check if there are any architectural issues after refactoring"
# Detects: Anti-patterns, design violations, architectural health scoring
```

#### 🕸️ **dependency-graph-analyzer**
**Purpose**: Understand dependency structure and coupling issues
**When to use**: Planning refactoring, investigating build issues, modularization efforts
**Expertise**: Dependency mapping, coupling assessment, circular dependency detection, impact analysis
```bash
# Use when: "Analyze what depends on the authentication module"
# Provides: Change impact analysis, modularization opportunities, coupling metrics
```

#### 🔒 **security-vulnerability-scanner**
**Purpose**: Comprehensive security analysis and vulnerability detection
**When to use**: Before production deployment, after implementing security features, regular audits
**Expertise**: OWASP Top Ten, hardcoded secrets, insecure patterns, dependency vulnerabilities
```bash
# Use when: "Scan for security vulnerabilities before deployment"
# Detects: SQL injection, XSS, insecure crypto, vulnerable dependencies
```

#### 🧪 **test-analysis-evaluator**
**Purpose**: Evaluate test coverage and quality
**When to use**: Before releases, when improving test suites, assessing test effectiveness
**Expertise**: Coverage analysis, test quality assessment, mutation testing principles, brittle test detection
```bash
# Use when: "Analyze test coverage for the authentication module"
# Provides: Coverage metrics, test gap identification, quality recommendations
```

#### 📚 **documentation-analyzer**
**Purpose**: Analyze and generate documentation quality assessments
**When to use**: After major changes, documentation audits, preparing for reviews
**Expertise**: Documentation gap analysis, code-docs alignment, ADR generation, architectural documentation
```bash
# Use when: "Check if documentation matches the refactored analysis engine"
# Provides: Documentation coverage, alignment issues, generation templates
```

#### 🔄 **refactoring-strategist**
**Purpose**: Create comprehensive refactoring plans
**When to use**: After identifying technical debt, planning major code improvements
**Expertise**: Refactoring patterns, migration strategies, risk assessment, implementation roadmaps
```bash
# Use when: "Create a refactoring plan for the god objects found"
# Provides: Prioritized refactoring plans, step-by-step implementation, risk mitigation
```

#### ⚖️ **quality-assurance-validator**
**Purpose**: Verify and validate findings from other agents
**When to use**: When multiple agents have analyzed code, resolving contradictory findings
**Expertise**: Cross-validation, confidence scoring, hallucination detection, consistency checking
```bash
# Use when: "Validate the security and architecture findings for accuracy"
# Provides: Verified results, confidence scores, consistency resolution
```

#### 🔗 **integration-manager**
**Purpose**: Configure and optimize external system integrations
**When to use**: Setting up CI/CD, troubleshooting monitoring, database optimization
**Expertise**: GitHub Actions, Prometheus/Grafana, database tuning, webhook configuration
```bash
# Use when: "Set up GitHub Actions for Uveddi analysis on PRs"
# Handles: CI/CD setup, monitoring integration, performance optimization
```

#### 🎭 **workflow-orchestrator**
**Purpose**: Coordinate complex multi-step analysis workflows
**When to use**: For comprehensive analysis requests covering multiple domains
**Expertise**: Workflow decomposition, parallel execution planning, result synthesis
```bash
# Use when: "Analyze for technical debt, security issues, and performance problems"
# Provides: Coordinated multi-agent analysis, synthesized reports, prioritized findings
```

### Agent Usage Patterns

#### 🚀 **Quick Start Analysis Workflow**
```
1. code-parser-engine → Parse codebase structure
2. architecture-analyzer → Identify architectural issues  
3. security-vulnerability-scanner → Check for vulnerabilities
4. quality-assurance-validator → Verify all findings
```

#### 🔍 **Deep Dive Analysis Workflow**
```
1. workflow-orchestrator → Plan comprehensive analysis
2. code-parser-engine → Foundation parsing
3. Multiple specialized agents → Parallel analysis
4. quality-assurance-validator → Final verification
5. documentation-analyzer → Ensure docs are current
```

#### 🛠️ **Refactoring Workflow**
```
1. architecture-analyzer → Identify problems
2. dependency-graph-analyzer → Understand impact
3. test-analysis-evaluator → Ensure test coverage
4. refactoring-strategist → Create implementation plan
5. integration-manager → Update CI/CD if needed
```

### Best Practices for Agent Usage

1. **Start with Foundation**: Always begin with `code-parser-engine` for structural analysis
2. **Use Validation**: Run `quality-assurance-validator` to verify findings from multiple agents
3. **Coordinate Complex Tasks**: Use `workflow-orchestrator` for multi-domain analysis requests
4. **Consider Context**: Each agent understands Uveddi's architecture and privacy-first philosophy
5. **Progressive Analysis**: Start with broad analysis, then dive deeper into specific areas

### Agent Selection Guide

| **Task Type** | **Primary Agent** | **Supporting Agents** |
|---------------|------------------|----------------------|
| Code Structure Analysis | code-parser-engine | architecture-analyzer |
| Security Audit | security-vulnerability-scanner | quality-assurance-validator |
| Refactoring Planning | refactoring-strategist | dependency-graph-analyzer |
| Test Improvement | test-analysis-evaluator | quality-assurance-validator |
| Documentation Review | documentation-analyzer | architecture-analyzer |
| CI/CD Setup | integration-manager | workflow-orchestrator |
| Comprehensive Review | workflow-orchestrator | All specialized agents |

These agents are designed to work together seamlessly, providing expert-level assistance tailored to Uveddi's specific architecture, security requirements, and development workflows.