# Uveddi Comprehensive Architecture & Development Manual

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture Deep Dive](#architecture-deep-dive)
3. [Core Modules & Components](#core-modules--components)
4. [Dependencies & Technology Stack](#dependencies--technology-stack)
5. [Development Patterns & Methods](#development-patterns--methods)
6. [Plugin System](#plugin-system)
7. [AI Integration](#ai-integration)
8. [Security & Resilience](#security--resilience)
9. [Performance & Monitoring](#performance--monitoring)
10. [Testing Strategy](#testing-strategy)
11. [Deployment & Operations](#deployment--operations)
12. [Development Workflow](#development-workflow)

---

## Project Overview

**Uveddi** is a sophisticated AI-powered CLI tool for architectural analysis of codebases, designed to identify architectural anti-patterns and prevent architectural drift. It combines static code analysis with AI-powered insights to help developers understand and improve their codebases.

### Key Features
- **Multi-language analysis**: Rust, Python, JavaScript, TypeScript
- **AI-powered insights**: Local (Ollama) and cloud AI integration
- **Privacy-focused**: All analysis happens locally by default
- **Extensible architecture**: WASM-based plugin system for custom detectors
- **Comprehensive reporting**: Markdown, JSON, and interactive outputs
- **Terminal UI**: Interactive terminal interface for analysis and exploration
- **Tree-sitter enabled**: Advanced parsing for improved accuracy and performance

### Version & Metadata
- **Version**: 0.9.0
- **Edition**: Rust 2021
- **License**: MIT
- **Repository**: https://github.com/botzrDev/uveddi

---

## Architecture Deep Dive

### Architectural Principles

1. **Privacy First**: All analysis happens locally - no data leaves the user's machine
2. **Separation of Concerns**: Each module has a single, well-defined responsibility
3. **Community Extensibility**: Core functionality can be extended through open source contributions
4. **Simplicity**: Minimal dependencies and straightforward architecture
5. **Local AI Integration**: Seamless integration with Ollama for private AI analysis

### Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                            │
│                  (User Interface)                          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Application Layer                         │
│              (Command Orchestration)                       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Analysis Layer                           │
│           (Core Analysis Engine)                           │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                      │
│        (AST, Caching, Database, Monitoring)               │
└─────────────────────────────────────────────────────────────┘
```

### C4 Architecture Model

#### Level 1: System Context
- **Developer/Tech Lead**: Primary users who analyze codebases
- **CI/CD Pipeline**: Automated analysis integration
- **AI Services**: OpenAI GPT-4, Anthropic Claude, Local Ollama
- **Data Storage**: Local SQLite DB, Cloud PostgreSQL (optional)

#### Level 2: Container Diagram
- **Rust CLI Application**: Core analysis engine
- **Node.js Rendering Service**: Diagram generation service
- **TypeScript Frontend**: Web-based dashboard (optional)
- **Local Database**: SQLite for caching and results
- **AI Provider**: Ollama for local AI analysis

#### Level 3: Component Diagram
Key components within the Rust CLI application:
- **Analysis Engine**: Core orchestrator
- **AST Parser**: Tree-sitter based parsing
- **Detector Registry**: Pluggable detector system
- **Dependency Graph**: Graph-based analysis
- **Cache Manager**: Result and AST caching
- **Plugin Engine**: WASM plugin execution
- **AI Engine**: AI provider abstraction
- **TUI System**: Terminal user interface

---

## Core Modules & Components

### Analysis Module (`src/analysis/`)

The heart of Uveddi's functionality, providing comprehensive framework for detecting anti-patterns and architectural issues.

#### Key Components:
- **AnalysisEngine**: Core orchestrator that runs detectors and collects results
- **AnalysisDetector**: Trait that all detectors must implement
- **Anti-pattern Detectors**: Specialized detectors for various code quality issues
- **Dependency Graph**: Graph-based analysis for architectural patterns
- **AST Cache**: Caching layer for parsed syntax trees

#### Detector Categories:

**Universal Anti-patterns** (Multi-language):
- God Object: Classes/structs with too many responsibilities
- Cyclic Dependencies: Circular dependencies between modules
- Magic Values: Hardcoded constants without explanation
- Global State: Excessive use of global variables
- Tight Coupling: High interdependence between components
- Resource Leaks: Improper resource management
- Silent Failures: Errors that fail without notification
- Code Duplication: Repeated code patterns
- Leaky Abstraction: Implementation details exposed through interfaces

**Language-Specific Detectors**:

*Rust*:
- Clone Abuse: Excessive use of `.clone()`
- Unwrap Abuse: Overuse of `.unwrap()` without error handling
- Lifetime Complexity: Overly complex lifetime annotations
- Memory Management: Improper memory handling patterns

*Python*:
- Data Structure Misuse: Inefficient data structure usage
- Exception Handling: Poor exception handling patterns
- OOP Issues: Object-oriented programming violations
- Performance Issues: Performance anti-patterns

*JavaScript*:
- Async Anti-patterns: Improper async/await usage
- Scope Issues: Variable scoping problems
- Type Coercion: Problematic type conversion patterns
- DOM Issues: DOM manipulation anti-patterns

### AI Module (`src/ai/`)

Provides AI-powered analysis capabilities with multiple provider support.

#### Components:
- **AI Engine**: Core AI analysis orchestrator
- **Ollama Provider**: Local AI integration
- **Prompt Templates**: Structured prompts for different analysis types
- **Context Builder**: Builds context for AI analysis
- **Self Correction**: AI response validation and improvement

#### Features:
- Local AI analysis via Ollama
- Structured prompt generation
- Response validation and correction
- Context-aware analysis
- Privacy-focused design

### Monitoring Module (`src/monitoring/`)

Real-time test execution monitoring, failure categorization, and performance tracking.

#### Components:
- **Metrics Collection**: Prometheus-based metrics
- **Performance Metrics Collector**: System performance tracking
- **Carbon Metrics**: Energy consumption tracking
- **Baseline Collector**: Performance baseline management
- **Reporting Engine**: Automated report generation

### Observability Module (`src/observability/`)

Enterprise-grade observability and resilience framework implementing UV-86 specifications.

#### Five Core Pillars:

1. **Structured Logging**: Built on `tracing` ecosystem with JSON output
2. **Metrics Collection**: Prometheus-based Four Golden Signals
3. **Graceful Degradation**: Circuit breaker patterns for external dependencies
4. **Error Recovery**: Intelligent retry mechanisms with exponential backoff
5. **Unified Observability**: Cross-system correlation via trace_id

### Resilience Module (`src/resilience/`)

Provides resilience patterns for handling transient failures in distributed systems.

#### Components:
- **Circuit Breaker**: Prevents cascade failures
- **Retry Logic**: Exponential backoff with jitter
- **Fallback Strategies**: Graceful degradation mechanisms
- **Health Monitoring**: System health tracking
- **Alert System**: Advanced alerting with escalation

### Security Module (`src/security/`)

Comprehensive security framework with RBAC and enterprise features.

#### Components:
- **Authentication**: OAuth2, OpenID Connect, JWT
- **Authorization**: Casbin-based RBAC
- **Rate Limiting**: Request rate limiting
- **Audit System**: Security event logging
- **Secret Management**: HashiCorp Vault integration

### Plugin System (`src/plugins/`)

WASM-based plugin architecture for extensibility.

#### Components:
- **Plugin Engine**: WASM runtime management
- **Plugin Registry**: Plugin discovery and management
- **Security Sandbox**: Secure plugin execution
- **Lifecycle Management**: Plugin loading/unloading
- **Data Plane**: Plugin data exchange

### TUI Module (`src/tui/`)

Interactive terminal user interface for analysis and exploration.

#### Components:
- **App State**: Application state management
- **Event System**: User input handling
- **UI Components**: Reusable UI elements
- **Terminal Management**: Terminal initialization and cleanup
- **Theme System**: Customizable themes

---

## Dependencies & Technology Stack

### Core Dependencies

#### Runtime & Async
- **tokio** (1.37.0): Async runtime with full features
- **tokio-stream** (0.1.15): Async stream utilities
- **futures** (0.3): Future combinators
- **async-stream** (0.3): Async stream macros
- **async-trait** (0.1.88): Async traits

#### Serialization & Data
- **serde** (1.0.203): Serialization framework with derive
- **serde_json** (1.0.117): JSON serialization
- **bincode** (1.3.3): Binary serialization
- **toml** (0.8.0): TOML configuration parsing
- **chrono** (0.4.39): Date and time handling

#### Database & Storage
- **rusqlite** (0.31.0): SQLite database with bundled features
- **rmp-serde** (1.1): MessagePack serialization for cache optimization
- **flate2** (1.0): Compression for diagram caching

#### CLI & User Interface
- **clap** (4.5.4): Command-line argument parsing with derive
- **color-eyre** (0.6): Enhanced error reporting
- **log** (0.4.21): Logging facade
- **env_logger** (0.11.3): Environment-based logger

#### AST & Language Support
- **tree-sitter** (0.25.8): Parser generator for syntax trees
- **tree-sitter-rust** (0.24.0): Rust grammar
- **tree-sitter-python** (0.23.6): Python grammar
- **tree-sitter-javascript** (0.23.1): JavaScript grammar
- **tree-sitter-typescript** (0.23.2): TypeScript grammar

#### HTTP & Networking
- **reqwest** (0.12.22): HTTP client with JSON support
- **axum** (0.7): Web framework with WebSocket support
- **tokio-tungstenite** (0.20): WebSocket implementation

#### Performance & Concurrency
- **rayon** (1.8.0): Data parallelism
- **lru** (0.12.0): LRU cache for bounded memory management
- **num_cpus** (1.16): CPU count detection
- **lazy_static** (1.4): Lazy static initialization

#### Monitoring & Observability
- **prometheus** (0.13.4): Metrics collection
- **tracing** (0.1): Structured logging and tracing
- **tracing-subscriber** (0.3): Tracing subscriber implementations
- **metrics** (0.23): Metrics collection framework

#### Security & Cryptography
- **ring** (0.17.12): Cryptographic operations
- **rustls** (0.23.28): TLS implementation
- **sha2** (0.10.0): SHA-2 hash functions
- **md5** (0.7): MD5 hashing
- **argon2** (0.5): Password hashing
- **blake3** (1.5): BLAKE3 hash function

#### Template & Rendering
- **tera** (1.19.1): Template engine for diagram generation
- **base64** (0.22.0): Base64 encoding/decoding

#### Utilities
- **uuid** (1.8.0): UUID generation with v4, v5, and serde
- **url** (2.5): URL parsing
- **walkdir** (2.5.0): Directory traversal
- **ignore** (0.4.22): Gitignore-style file filtering
- **regex** (1.10): Regular expressions
- **glob** (0.3.1): Pattern matching for file filtering

### Optional Feature Dependencies

#### WASM Plugin System
- **wasmtime** (34.0.2): WASM runtime with component model
- **wasmtime-wasi** (34.0.2): WASI support
- **cap-std** (3.0): Capability-based standard library
- **wit-bindgen** (0.30.0): WebAssembly Interface Types

#### Terminal UI
- **ratatui** (0.29): Terminal UI framework
- **crossterm** (0.29): Cross-platform terminal manipulation
- **tui-input** (0.14): Terminal input handling

#### Memory Optimization
- **mimalloc** (0.1): High-performance allocator
- **bumpalo** (3.19): Arena allocation with collections
- **bumpalo-herd** (0.1): Herd allocation
- **memmap2** (0.9): Memory mapping
- **rkyv** (0.7): Zero-copy serialization

#### Chaos Engineering & Testing
- **fail** (0.5): Failpoint injection
- **statrs** (0.16.1): Statistical analysis
- **ndarray-stats** (0.5): N-dimensional array statistics
- **changepoint** (0.14.2): Change point detection
- **criterion** (0.5): Benchmarking framework

#### Security & Enterprise
- **casbin** (2.1): Authorization library
- **oauth2** (4.4): OAuth2 client
- **openidconnect** (3.5): OpenID Connect
- **jsonwebtoken** (9.3): JWT handling
- **vaultrs** (0.7): HashiCorp Vault client

#### Monitoring & Metrics
- **metrics-exporter-prometheus** (0.15): Prometheus exporter
- **tokio-metrics** (0.3): Tokio runtime metrics
- **influxdb2** (0.5): InfluxDB client for time series

---

## Development Patterns & Methods

### Trait-Based Architecture

Uveddi uses extensive trait-based design for modularity and testability:

```rust
pub trait AnalysisDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn detect_graph_issues(&self, graph: &LocalDependencyGraph, run_id: i64) -> Vec<ArchitecturalIssue>;
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    fn get_detector_name(&self) -> &'static str;
}
```

### Builder Pattern

Complex objects use the builder pattern for configuration:

```rust
let engine = AnalysisEngine::builder()
    .enable_plugins(true)
    .build_async()
    .await?;
```

### Facade Pattern

The analysis engine implements the facade pattern to simplify complex subsystem interactions:

```rust
pub struct AnalysisEngine {
    pub config_service: Arc<ConfigurationService>,
    pub ast_provider: Arc<AstProviderImpl>,
    pub cache_manager: Arc<CacheManagerImpl>,
    pub detector_scheduler: Arc<DetectorScheduler>,
    pub dependency_graph_builder: Arc<DependencyGraphBuilderImpl>,
    pub analysis_aggregator: Arc<AnalysisAggregator>,
    pub plugin_manager: Arc<PluginManagerHandle>,
}
```

### Error Handling Strategy

Comprehensive error handling using `thiserror` and `color-eyre`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
    #[error("Anti-pattern detection failed: {0}")]
    AntiPatternDetection(String),
    #[error("Query execution failed: {0}")]
    QueryError(String),
    #[error("AST error: {0}")]
    AstError(#[from] AstError),
}
```

### Async/Await Patterns

Consistent async patterns throughout the codebase:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let engine = AnalysisEngine::new_async().await?;
    let (issues, graph) = engine.analyze(Path::new("src/")).await?;
    Ok(())
}
```

### Memory Management

Advanced memory optimization techniques:

1. **Arena Allocation**: Using `bumpalo` for temporary objects
2. **Zero-Copy Serialization**: Using `rkyv` for AST cache
3. **Memory Mapping**: Using `memmap2` for large data structures
4. **Custom Allocator**: Using `mimalloc` for performance

### Caching Strategy

Multi-layer caching system:

1. **AST Cache**: Parsed syntax trees cached in SQLite
2. **Result Cache**: Analysis results cached with file modification tracking
3. **Diagram Cache**: Generated diagrams cached with compression
4. **Memory Cache**: In-memory LRU cache for frequently accessed data

---

## Plugin System

### WASM-Based Architecture

Uveddi uses WebAssembly for secure, sandboxed plugin execution:

```rust
pub struct WasmPluginEngine {
    engine: wasmtime::Engine,
    store: wasmtime::Store<PluginState>,
    instances: HashMap<String, wasmtime::Instance>,
}
```

### Plugin Interface

Plugins implement the WebAssembly Interface Types (WIT) specification:

```wit
// wit/plugin.wit
interface plugin {
    record analysis-result {
        issues: list<architectural-issue>,
        metrics: plugin-metrics,
    }
    
    analyze: func(code: string, language: string) -> analysis-result
}
```

### Plugin Lifecycle

1. **Discovery**: Plugins discovered via manifest files
2. **Loading**: WASM modules loaded into runtime
3. **Initialization**: Plugin state initialized
4. **Execution**: Plugin functions called during analysis
5. **Cleanup**: Resources cleaned up on unload

### Security Model

- **Sandboxed Execution**: WASM provides memory isolation
- **Capability-Based Security**: Limited system access via WASI
- **Resource Limits**: CPU and memory limits enforced
- **Audit Logging**: All plugin actions logged

---

## AI Integration

### Provider Abstraction

AI functionality abstracted through provider interface:

```rust
#[async_trait]
pub trait LlmProvider {
    async fn generate_explanation(&self, context: &AnalysisContext) -> Result<String>;
    async fn suggest_refactoring(&self, issue: &ArchitecturalIssue) -> Result<String>;
}
```

### Ollama Integration

Local AI via Ollama for privacy:

```rust
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}
```

### Prompt Engineering

Structured prompts for consistent AI responses:

```rust
pub struct PromptTemplate {
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub context_variables: Vec<String>,
}
```

### Context Building

Rich context provided to AI models:

- Code snippets with syntax highlighting
- Architectural issue descriptions
- Related code patterns
- Project metadata
- Language-specific idioms

---

## Security & Resilience

### Authentication & Authorization

Multi-layer security approach:

1. **Authentication**: OAuth2, OpenID Connect, JWT tokens
2. **Authorization**: Role-Based Access Control (RBAC) via Casbin
3. **Rate Limiting**: Request throttling to prevent abuse
4. **Audit Logging**: Comprehensive security event logging

### Circuit Breaker Pattern

Prevents cascade failures in distributed systems:

```rust
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_threshold: usize,
    timeout: Duration,
    failure_count: AtomicUsize,
}
```

### Retry Logic

Intelligent retry with exponential backoff:

```rust
pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}
```

### Secret Management

Integration with HashiCorp Vault for secure credential storage:

```rust
pub struct VaultSecretManager {
    client: vaultrs::client::VaultClient,
    mount_path: String,
}
```

---

## Performance & Monitoring

### Metrics Collection

Comprehensive metrics using Prometheus:

- **Four Golden Signals**: Latency, traffic, errors, saturation
- **Application Metrics**: Analysis pipeline performance
- **System Metrics**: CPU, memory, disk usage
- **Custom Metrics**: Domain-specific measurements

### Performance Optimization

Multiple optimization strategies:

1. **Async Processing**: Non-blocking file analysis
2. **Parallel Execution**: CPU-bound tasks distributed across cores
3. **Smart Caching**: Multi-layer caching strategy
4. **Memory Optimization**: Arena allocation and zero-copy serialization
5. **Lazy Loading**: Resources loaded on demand

### Benchmarking

Comprehensive benchmarking suite using Criterion:

```rust
fn benchmark_analysis_engine(c: &mut Criterion) {
    c.bench_function("analyze_large_codebase", |b| {
        b.iter(|| {
            // Benchmark code
        })
    });
}
```

### Carbon Awareness

Energy consumption tracking for sustainable computing:

```rust
pub struct CarbonAwarenessCollector {
    energy_monitor: EnergyMonitor,
    carbon_intensity_api: CarbonIntensityApi,
}
```

---

## Testing Strategy

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Component interaction testing
3. **End-to-End Tests**: Complete workflow testing
4. **Performance Tests**: Scalability and performance validation
5. **Security Tests**: Vulnerability assessment

### Test Infrastructure

- **Test Fixtures**: Reusable test data and mocks
- **Test Utilities**: Helper functions for common test patterns
- **Coverage Tracking**: Code coverage measurement and reporting
- **Regression Testing**: Automated regression detection

### Quality Gates

- **Code Coverage**: Minimum 90% coverage requirement
- **Performance Benchmarks**: All benchmarks must pass
- **Security Audit**: No critical vulnerabilities
- **Documentation**: All public APIs documented

---

## Deployment & Operations

### Deployment Options

1. **Standalone Binary**: Single executable with embedded dependencies
2. **Docker Container**: Containerized deployment with health checks
3. **Kubernetes**: Scalable deployment with monitoring
4. **CI/CD Integration**: Automated analysis in build pipelines

### Configuration Management

Hierarchical configuration system:

1. **Default Configuration**: Built-in defaults
2. **System Configuration**: System-wide settings
3. **User Configuration**: User-specific preferences
4. **Project Configuration**: Project-specific overrides
5. **Environment Variables**: Runtime configuration

### Monitoring & Alerting

Production monitoring setup:

- **Health Checks**: Application health endpoints
- **Metrics Dashboard**: Real-time metrics visualization
- **Alert Rules**: Automated alerting for critical issues
- **Log Aggregation**: Centralized log collection and analysis

### Disaster Recovery

- **Backup Strategy**: Regular data backups
- **Recovery Procedures**: Documented recovery steps
- **Failover Mechanisms**: Automatic failover capabilities
- **Data Integrity**: Checksums and validation

---

## Development Workflow

### Getting Started

1. **Environment Setup**:
   ```bash
   git clone https://github.com/botzrDev/uveddi.git
   cd uveddi
   cargo install --path .
   ```

2. **Development Tools**:
   ```bash
   cargo install cargo-nextest cargo-llvm-cov
   ```

3. **Running Tests**:
   ```bash
   cargo nextest run --all-features
   ```

### Code Organization

- **Module Structure**: Clear separation of concerns
- **Naming Conventions**: Consistent naming throughout
- **Documentation**: Comprehensive inline documentation
- **Error Handling**: Consistent error handling patterns

### Contribution Guidelines

1. **Code Style**: Follow Rust standard conventions
2. **Testing**: All changes must include tests
3. **Documentation**: Update documentation for public APIs
4. **Performance**: Consider performance implications
5. **Security**: Follow security best practices

### Release Process

1. **Version Bumping**: Semantic versioning
2. **Changelog**: Detailed change documentation
3. **Testing**: Comprehensive test suite execution
4. **Documentation**: Documentation updates
5. **Deployment**: Automated deployment pipeline

---

## Conclusion

Uveddi represents a sophisticated approach to static code analysis, combining modern Rust development practices with AI-powered insights and enterprise-grade observability. Its modular architecture, comprehensive testing strategy, and focus on performance and security make it suitable for both individual developers and large-scale enterprise deployments.

The system's extensibility through WASM plugins, privacy-focused AI integration, and comprehensive monitoring capabilities position it as a powerful tool for maintaining code quality and preventing architectural drift in modern software development.

---

*This manual serves as a comprehensive guide for understanding, developing, and operating Uveddi. For specific implementation details, refer to the source code and inline documentation.*