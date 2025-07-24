# Uveddi Comprehensive Architecture & Development Manual

> **Version 0.9.0** | **Last Updated**: July 2025 | **Rust Edition**: 2021

## Table of Contents

1. [Quick Start Guide](#quick-start-guide)
2. [Project Overview](#project-overview)
3. [Architecture Deep Dive](#architecture-deep-dive)
4. [Core Modules & Components](#core-modules--components)
5. [Feature Flags & Compilation Options](#feature-flags--compilation-options)
6. [Dependencies & Technology Stack](#dependencies--technology-stack)
7. [Development Patterns & Methods](#development-patterns--methods)
8. [Plugin System](#plugin-system)
9. [AI Integration](#ai-integration)
10. [Security & RBAC](#security--rbac)
11. [Resilience & Error Handling](#resilience--error-handling)
12. [Performance & Monitoring](#performance--monitoring)
13. [Terminal User Interface (TUI)](#terminal-user-interface-tui)
14. [Chaos Engineering](#chaos-engineering)
15. [Testing Strategy](#testing-strategy)
16. [Configuration Management](#configuration-management)
17. [Deployment & Operations](#deployment--operations)
18. [Development Workflow](#development-workflow)
19. [Troubleshooting & Best Practices](#troubleshooting--best-practices)
20. [API Reference](#api-reference)

---

## Quick Start Guide

### Installation

#### Quick Install (Recommended)
```bash
curl -sSL https://uveddi.dev/install.sh | bash
```

#### From Source
```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo install --path .
```

#### Using Cargo
```bash
cargo install uveddi
```

### Basic Usage

#### Analyze a Codebase
```bash
# Basic analysis
uveddi analyze /path/to/code

# With AI-powered insights (requires Ollama)
uveddi analyze /path/to/code --enable-ai

# Generate detailed report
uveddi analyze /path/to/code --output report.md --format markdown
```

#### Interactive Terminal UI
```bash
# Launch TUI for interactive analysis
uveddi tui

# TUI with specific project
uveddi tui --project /path/to/code
```

#### Configuration
```bash
# Initialize configuration
uveddi config init

# Edit configuration
uveddi config edit

# Validate configuration
uveddi config validate
```

### First Analysis Example

```bash
# 1. Analyze your Rust project
uveddi analyze src/ --output analysis.md

# 2. View results
cat analysis.md

# 3. Enable AI explanations (optional)
uveddi analyze src/ --enable-ai --ai-provider ollama
```

### Common Use Cases

| Use Case | Command | Description |
|----------|---------|-------------|
| **Quick Check** | `uveddi analyze src/` | Fast analysis without AI |
| **Detailed Report** | `uveddi analyze src/ --output report.md` | Generate comprehensive report |
| **AI Insights** | `uveddi analyze src/ --enable-ai` | Include AI-powered explanations |
| **Interactive Mode** | `uveddi tui` | Launch terminal interface |
| **CI/CD Integration** | `uveddi analyze src/ --format json` | Machine-readable output |

---

## Project Overview

**Uveddi** is a sophisticated AI-powered CLI tool for architectural analysis of codebases, designed to identify architectural anti-patterns and prevent architectural drift. It combines static code analysis with AI-powered insights to help developers understand and improve their codebases.

### Core Philosophy

1. **Privacy First**: All analysis happens locally - no data leaves your machine by default
2. **Developer-Centric**: Built by developers, for developers, with real-world workflows in mind
3. **Extensible by Design**: Plugin architecture allows custom detectors and integrations
4. **Performance Focused**: Optimized for large codebases with intelligent caching
5. **AI-Enhanced**: Optional AI integration provides intelligent insights while preserving privacy

### Key Features

#### 🔍 **Multi-Language Analysis**
- **Rust**: Advanced ownership, lifetime, and memory safety analysis
- **Python**: OOP patterns, data structure usage, exception handling
- **JavaScript/TypeScript**: Async patterns, scope issues, type safety
- **Universal Patterns**: Cross-language architectural anti-patterns

#### 🤖 **AI-Powered Insights**
- **Local AI (Ollama)**: Privacy-preserving analysis with local models
- **Cloud AI**: Optional integration with OpenAI, Anthropic, Google Gemini
- **Smart Prompting**: Context-aware AI prompts for accurate analysis
- **Self-Correction**: AI response validation and improvement

#### 🏗️ **Architectural Analysis**
- **Anti-Pattern Detection**: God objects, tight coupling, dead code, etc.
- **Dependency Analysis**: Circular dependencies, coupling metrics
- **Code Quality Metrics**: Complexity, maintainability scores
- **Visualization**: Mermaid diagrams for architectural insights

#### 🔧 **Developer Experience**
- **Terminal UI**: Interactive analysis and exploration
- **Multiple Output Formats**: Markdown, JSON, HTML reports
- **IDE Integration**: VS Code extension support
- **CI/CD Ready**: Machine-readable outputs for automation

#### 🚀 **Performance & Scalability**
- **Tree-sitter Parsing**: Fast, accurate syntax analysis
- **Intelligent Caching**: AST and result caching for speed
- **Parallel Processing**: Multi-threaded analysis for large codebases
- **Memory Optimization**: Efficient memory usage with arena allocation

#### 🔒 **Enterprise Features**
- **Security & RBAC**: Role-based access control and audit logging
- **Chaos Engineering**: Fault injection and resilience testing
- **Monitoring**: Real-time metrics and performance tracking
- **Plugin System**: WASM-based secure plugin architecture

### Supported Environments

| Environment | Support Level | Notes |
|-------------|---------------|-------|
| **Linux** | ✅ Full | Primary development platform |
| **macOS** | ✅ Full | Native Apple Silicon support |
| **Windows** | ✅ Full | WSL2 recommended for best experience |
| **Docker** | ✅ Full | Official container images available |
| **CI/CD** | ✅ Full | GitHub Actions, GitLab CI, Jenkins |

### Version & Metadata
- **Version**: 0.9.0
- **Edition**: Rust 2021
- **License**: MIT
- **Repository**: https://github.com/botzrDev/uveddi
- **Documentation**: https://botzrdev.github.io/uveddi/
- **Community**: [Discord](https://discord.gg/uveddi) | [Discussions](https://github.com/botzrDev/uveddi/discussions)

---

## Core Modules & Components

### Analysis Engine (`src/analysis/`)

The heart of Uveddi's static code analysis capabilities, implementing a pluggable detector architecture.

#### Key Components:
- **AnalysisEngine**: Core orchestrator that runs detectors and collects results
- **DetectorRegistry**: Manages and coordinates all analysis detectors
- **AST Cache**: High-performance caching layer for parsed syntax trees
- **Dependency Graph**: Graph-based analysis for architectural patterns
- **Memory Optimization**: Arena allocation and zero-copy serialization

#### Anti-Pattern Detectors:
- **God Object Detector**: Identifies classes/modules with excessive responsibilities
- **Dead Code Detector**: Finds unused code and unreachable functions
- **Code Duplication Detector**: Detects similar code blocks across the codebase
- **Tight Coupling Detector**: Analyzes component interdependencies
- **Large Classes Detector**: Identifies oversized classes and modules
- **Cyclic Dependencies Detector**: Finds circular dependency chains

#### Language-Specific Analysis:
- **Rust**: Clone abuse, unwrap abuse, lifetime complexity, memory management
- **Python**: Data structure misuse, exception handling, OOP violations
- **JavaScript**: Async anti-patterns, scope issues, type coercion problems

### AI Integration (`src/ai/`)

Provides AI-powered analysis capabilities with multiple provider support and privacy focus.

#### Components:
- **AI Engine**: Core AI analysis orchestrator with provider abstraction
- **Ollama Provider**: Local AI integration for privacy-preserving analysis
- **Context Builder**: Builds structured context for AI analysis
- **Prompt Templates**: Specialized prompts for different analysis types
- **Self Correction**: AI response validation and improvement system
- **Carbon Aware**: Energy-efficient AI usage optimization

#### Features:
- **Multi-Provider Support**: OpenAI, Anthropic, Google Gemini, Local Ollama
- **Smart Prompting**: Context-aware prompt generation for accurate results
- **Response Validation**: Automatic validation and correction of AI responses
- **Privacy Controls**: Local-first processing with optional cloud integration
- **Rate Limiting**: Intelligent request throttling and cost management

---

## Feature Flags & Compilation Options

Uveddi uses Cargo feature flags to enable modular compilation and reduce binary size for specific use cases.

### Default Features
```toml
default = ["local-ai", "tree-sitter", "memory-optimization"]
```

### Core Feature Categories

#### **Analysis Features**
```toml
# Tree-sitter parsing (enabled by default)
tree-sitter = [
    "dep:tree-sitter",
    "dep:tree-sitter-rust", 
    "dep:tree-sitter-python",
    "dep:tree-sitter-javascript",
    "dep:tree-sitter-typescript"
]

# Memory optimization features
memory-optimization = ["mimalloc", "bumpalo", "bumpalo-herd", "memmap2", "rkyv"]
```

#### **AI Features**
```toml
# Base AI functionality
ai = ["reqwest", "async-trait"]

# Local AI only (Ollama)
local-ai = ["ai"]
```

#### **User Interface Features**
```toml
# Terminal User Interface
tui = ["ratatui", "crossterm", "tui-input"]

# Image rendering service integration
image-rendering = ["reqwest"]
```

#### **Enterprise Features**
```toml
# WASM plugin system
wasm-plugins = ["wasmtime", "wasmtime-wasi", "cap-std", "wit-bindgen"]

# Chaos engineering and load testing
chaos = ["fail", "statrs", "ndarray-stats"]

# Performance regression detection
regression-detection = ["changepoint", "linfa", "influxdb2"]

# SLA monitoring and validation
sla-monitoring = ["metrics-exporter-prometheus", "tokio-metrics"]
```

### Predefined Feature Combinations

#### **Zero-Cost Option** (Minimal Dependencies)
```toml
zero-cost = ["local-ai", "tree-sitter", "memory-optimization"]
```
- Local analysis only
- No external service dependencies
- Optimized for individual developers

#### **Full-Featured Option** (Complete Experience)
```toml
full-featured = ["local-ai", "tree-sitter", "memory-optimization", "image-rendering", "tui"]
```
- All user-facing features
- Interactive terminal interface
- Diagram generation support

#### **Enterprise Option** (All Features)
```toml
enterprise = [
    "local-ai", "tree-sitter", "memory-optimization", 
    "image-rendering", "tui", "wasm-plugins", 
    "chaos", "sla-monitoring"
]
```
- Complete feature set
- Plugin system support
- Chaos engineering capabilities
- Enterprise monitoring

### Compilation Examples

```bash
# Minimal build (fastest compilation)
cargo build --no-default-features --features="tree-sitter"

# Full-featured build
cargo build --features="full-featured"

# Enterprise build with all features
cargo build --features="enterprise"

# Custom build
cargo build --features="local-ai,tui,wasm-plugins"
```

---

## Architecture Deep Dive

### Architectural Principles

1. **Privacy First**: All analysis happens locally - no data leaves the user's machine by default
2. **Separation of Concerns**: Each module has a single, well-defined responsibility  
3. **Layered Architecture**: Strict dependency flow with clear boundaries
4. **Extensibility**: Plugin architecture enables custom detectors and integrations
5. **Performance Focus**: Optimized for large codebases with intelligent caching
6. **Type Safety**: Leverages Rust's type system for reliability and performance

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


---

## All Project Dependencies & Their Roles

Below is a comprehensive list of all dependencies used in Uveddi, with a brief explanation of their purpose in the context of the project:

### Core Runtime & Async
- **tokio**: The async runtime powering all concurrent operations, file IO, and networking.
- **tokio-stream**: Utilities for working with asynchronous streams, e.g., file and network data.
- **futures**: Core futures and combinators for async programming.
- **async-stream**: Macros for easily creating async streams.
- **async-trait**: Enables async functions in traits, used for extensible async APIs.

### Serialization, Data, and Config
- **serde**: Serialization/deserialization for config, cache, and data interchange.
- **serde_json**: JSON serialization for reports, config, and AI communication.
- **bincode**: Compact binary serialization for fast cache storage.
- **toml**: Parsing and writing TOML config files.
- **chrono**: Date/time handling for timestamps, logs, and metrics.

### Database & Storage
- **rusqlite**: Embedded SQLite database for caching, state, and results.
- **rmp-serde**: MessagePack serialization for efficient cache storage.
- **flate2**: Compression for diagram/image cache and storage.

### CLI & User Interface
- **clap**: Command-line argument parsing and CLI interface.
- **color-eyre**: Enhanced error reporting for CLI and TUI.
- **log**: Logging facade for all log output.
- **env_logger**: Environment-based logger initialization.

### AST & Language Support
- **tree-sitter**: Core parser for building ASTs from source code.
- **tree-sitter-rust/python/javascript/typescript**: Language grammars for multi-language analysis.

### HTTP, Networking, and Web
- **reqwest**: HTTP client for AI, image rendering, and remote services.
- **axum**: Web server for API endpoints and TUI backend.
- **tokio-tungstenite**: WebSocket support for real-time communication.

### Performance, Concurrency, and Caching
- **rayon**: Data parallelism for fast analysis and graph traversal.
- **lru**: In-memory LRU cache for frequently accessed data.
- **num_cpus**: Detects CPU count for optimal parallelism.
- **lazy_static**: Global static initialization for caches and config.

### Monitoring, Observability, and Metrics
- **prometheus**: Metrics collection and export for monitoring.
- **tracing**: Structured, async-aware logging and diagnostics.
- **tracing-subscriber/appender**: Log formatting, filtering, and output.
- **metrics**: Metrics collection framework for custom and system metrics.
- **metrics-exporter-prometheus**: Exports metrics to Prometheus (feature).
- **tokio-metrics**: Tokio runtime metrics (feature).
- **influxdb2**: InfluxDB client for time series data (feature).

### Security, Authentication, and Cryptography
- **ring**: Cryptographic primitives for secure operations.
- **rustls**: TLS for secure HTTP and WebSocket connections.
- **sha2**: SHA-2 hashing for integrity and security.
- **md5**: MD5 hashing for legacy/compatibility.
- **argon2**: Password hashing for authentication.
- **blake3**: Fast, secure hashing for IDs and cache keys.
- **casbin**: RBAC/authorization for API and TUI.
- **oauth2/openidconnect/jsonwebtoken**: Authentication and token management.
- **vaultrs**: HashiCorp Vault integration for secret management.

### Template, Rendering, and Visualization
- **tera**: Template engine for generating diagrams and reports.
- **base64**: Encoding/decoding images and binary data.

### Utilities and Miscellaneous
- **uuid**: Unique IDs for components, runs, and cache keys.
- **url**: URL parsing and manipulation.
- **walkdir**: Recursive directory traversal for file discovery.
- **ignore**: Gitignore-style file filtering for analysis.
- **regex**: Regular expressions for pattern matching.
- **glob**: File pattern matching for filtering and config.
- **petgraph**: Graph data structures for dependency analysis.
- **color-eyre**: User-friendly error reporting.
- **ndarray/ndarray-stats**: N-dimensional arrays and statistics for performance/chaos features.
- **strum/strum_macros**: Enum utilities and macro derivations.
- **arrow/arrow-ipc/arrow-schema/arrow-array**: Columnar data and serialization for advanced reporting.
- **sysinfo**: System information for monitoring and reporting.
- **confy**: Persistent configuration for TUI and CLI.
- **persisted**: State persistence for TUI.
- **streaming-iterator**: Efficient streaming over collections.

### WASM Plugin System (Feature)
- **wasmtime/wasmtime-wasi**: WASM runtime and WASI support for plugins.
- **cap-std**: Capability-based stdlib for secure plugin execution.
- **wit-bindgen**: WebAssembly Interface Types for plugin API.

### Terminal UI (Feature)
- **ratatui**: Terminal UI framework for interactive analysis.
- **crossterm**: Terminal manipulation and input.
- **tui-input**: Input handling for TUI.

### Memory Optimization (Feature)
- **mimalloc**: High-performance memory allocator.
- **bumpalo/bumpalo-herd**: Arena allocation for temporary objects.
- **memmap2**: Memory mapping for large data.
- **rkyv**: Zero-copy serialization for fast cache.

### Chaos Engineering & Testing (Feature)
- **fail**: Failpoint injection for chaos testing.
- **statrs**: Statistical analysis for regression detection.
- **changepoint**: Change point detection for performance.
- **linfa**: Machine learning for anomaly detection.

### Dev & Test Utilities
- **tempfile**: Temporary file creation for tests.
- **tokio-test**: Async test utilities.
- **mockall/mockito**: Mocking for unit/integration tests.
- **assert_cmd/predicates**: CLI testing and assertions.
- **rstest/proptest/serial_test**: Advanced test frameworks.
- **criterion/iai-callgrind**: Benchmarking and profiling.

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

## Security & RBAC

Comprehensive security framework implementing enterprise-grade authentication, authorization, and audit capabilities.

### Security Architecture (`src/security/`)

#### Authentication Systems
- **OAuth 2.0/OIDC Integration**: Federated authentication with enterprise IdPs
- **API Key Authentication**: Secure service-to-service authentication  
- **Session Management**: Secure session handling with expiration
- **JWT Token Validation**: Standards-compliant token verification

#### Authorization (RBAC/ABAC)
- **Role-Based Access Control**: Hierarchical role system with predefined roles
- **Attribute-Based Access Control**: Fine-grained permission system
- **Permission Scoping**: Resource-level access control (own/team/all)
- **Policy Engine**: Casbin-powered authorization engine

#### Security Features
- **Audit Logging**: Comprehensive, immutable security event logging
- **Input Validation**: Path traversal prevention and input sanitization
- **Rate Limiting**: DDoS protection and resource abuse prevention
- **Secret Management**: HashiCorp Vault integration for secure credential storage
- **Compliance**: SOC2, GDPR, and enterprise compliance support

#### Configuration Example
```toml
[security]
enabled = true
audit_logging = true
rate_limiting = true

[security.authentication]
provider = "oidc"
issuer_url = "https://auth.company.com"
client_id = "uveddi-client"

[security.authorization]
policy_file = "policies/rbac.conf"
default_role = "viewer"

[security.rate_limiting]
requests_per_minute = 100
burst_size = 10
```

#### Predefined Roles
- **Admin**: Full system access and user management
- **Analyst**: Analysis execution and report generation
- **Viewer**: Read-only access to reports and results
- **Developer**: Code analysis and plugin management

---

## Resilience & Error Handling

Advanced resilience patterns for handling transient failures and maintaining system stability.

### Resilience Framework (`src/resilience/`)

#### Core Patterns
- **Circuit Breaker**: Prevents cascade failures with automatic recovery
- **Retry Logic**: Exponential backoff with jitter for transient failures
- **Fallback Strategies**: Graceful degradation when services are unavailable
- **Health Monitoring**: Continuous health checks and status reporting
- **Graceful Shutdown**: Clean resource cleanup and state preservation

#### Advanced Features
- **Alert System**: Multi-channel alerting with escalation policies
- **Analytics Dashboard**: Real-time resilience metrics and trends
- **Recovery Management**: Automated recovery procedures and manual overrides
- **Availability Detection**: Service availability monitoring and reporting

#### Error Handling Strategy
```rust
#[derive(thiserror::Error, Debug)]
pub enum UveddiError {
    #[error("Analysis failed: {0}")]
    Analysis(String),
    
    #[error("AI provider error: {provider} - {message}")]
    AiProvider { provider: String, message: String },
    
    #[error("Security violation: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Configuration error: {0}")]
    Config(String),
}
```

#### Resilience Configuration
```toml
[resilience.retry]
max_attempts = 3
initial_delay = "1s"
max_delay = "30s"
backoff_multiplier = 2.0

[resilience.circuit_breaker]
failure_threshold = 5
timeout = "60s"
half_open_max_calls = 3

[resilience.fallback]
enable_cache_fallback = true
enable_offline_mode = true
```

#### Monitoring Integration
- **Prometheus Metrics**: Comprehensive metrics collection
- **Health Endpoints**: HTTP health check endpoints
- **Alert Integration**: Slack, PagerDuty, email notifications
- **Dashboard**: Real-time resilience status visualization

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

## Terminal User Interface (TUI)

Interactive terminal interface implementing The Elm Architecture (TEA) pattern for predictable state management.

### TUI Architecture (`src/tui/`)

#### Core Components
- **App State**: Centralized application state management
- **Event Handler**: Keyboard and mouse input processing
- **UI Components**: Reusable terminal widgets and layouts
- **Terminal Manager**: Terminal initialization and cleanup
- **Themes**: Customizable color schemes and styling

#### Features
- **Interactive Analysis Form**: Replace CLI arguments with guided forms
- **Real-time Progress**: Live analysis progress and status updates
- **Report Viewer**: Multi-format report display with syntax highlighting
- **Configuration Editor**: Visual configuration management
- **Plugin Manager**: Interactive plugin installation and management

#### Key Screens
- **Main Menu**: Navigation hub with project selection
- **Analysis Form**: Interactive analysis configuration
- **Progress View**: Real-time analysis monitoring
- **Report Browser**: Results exploration and export
- **Settings**: Configuration and preferences

#### Usage Examples
```bash
# Launch TUI
uveddi tui

# TUI with specific project
uveddi tui --project /path/to/code

# TUI with custom theme
uveddi tui --theme dark
```

#### Configuration
```toml
[tui]
theme = "dark"
auto_save = true
show_help = true

[tui.keybindings]
quit = "q"
help = "?"
analyze = "a"
```

---

## Chaos Engineering

Comprehensive chaos engineering framework for testing system resilience and fault tolerance.

### Chaos Framework (`src/chaos/`)

#### Core Capabilities
- **Multi-tier Fault Injection**: Unit, service, and system level failures
- **Safe Failure Scenarios**: Automatic rollback and safety constraints
- **Comprehensive Observability**: Detailed measurement and reporting
- **CI/CD Integration**: Automated chaos testing in pipelines

#### Experiment Types
- **Network Failures**: Latency injection, packet loss, connection drops
- **Resource Exhaustion**: Memory pressure, CPU saturation, disk full
- **Service Failures**: Process crashes, dependency unavailability
- **Data Corruption**: File system errors, database inconsistencies

#### Safety Features
- **Blast Radius Control**: Limit experiment scope and impact
- **Emergency Stop**: Immediate experiment termination
- **Health Monitoring**: Continuous system health validation
- **Rollback Procedures**: Automatic recovery from failed experiments

#### Configuration Example
```toml
[chaos]
enabled = true
max_concurrent_experiments = 3
safety_checks_enabled = true

[chaos.experiments.network_latency]
enabled = true
target_services = ["analysis-engine"]
latency_ms = 100
duration = "30s"
blast_radius = "single_service"

[chaos.experiments.memory_pressure]
enabled = false
memory_percentage = 80
duration = "60s"
blast_radius = "unit_test"
```

#### Experiment Lifecycle
1. **Planning**: Define experiment parameters and safety constraints
2. **Validation**: Verify system health and readiness
3. **Execution**: Inject failures and monitor system behavior
4. **Observation**: Collect metrics and analyze system response
5. **Recovery**: Restore normal operation and validate recovery
6. **Analysis**: Generate reports and identify improvements

#### Integration with CI/CD
```yaml
# .github/workflows/chaos-testing.yml
- name: Run Chaos Experiments
  run: |
    uveddi chaos run --experiment network_latency
    uveddi chaos run --experiment memory_pressure
    uveddi chaos report --format json
```

---

## Configuration Management

Comprehensive configuration system supporting multiple formats and environments.

### Configuration Architecture

#### Configuration Sources (Priority Order)
1. **Command Line Arguments**: Highest priority, overrides all other sources
2. **Environment Variables**: `UVEDDI_*` prefixed variables
3. **Configuration Files**: TOML, JSON, YAML support
4. **Default Values**: Built-in sensible defaults

#### Configuration File Example
```toml
# ~/.uveddi/config.toml

[analysis]
max_file_size = "2MB"
parallel_jobs = 4
timeout = 300
enable_cache = true

[ai]
default_provider = "ollama"
enable_explanations = true

[ai.providers.ollama]
url = "http://localhost:11434"
model = "deepseek-coder:6.7b"
temperature = 0.7

[ai.providers.openai]
model = "gpt-4"
api_key = "${OPENAI_API_KEY}"
max_tokens = 2000

[detectors.god_object]
enabled = true
max_methods = 20
max_fields = 15

[security]
enabled = false
audit_logging = true

[tui]
theme = "dark"
auto_save = true
```

#### Environment Variables
```bash
# Core settings
export UVEDDI_ANALYSIS_PARALLEL_JOBS=8
export UVEDDI_AI_PROVIDER=ollama
export UVEDDI_CACHE_SIZE=2000

# AI provider settings
export UVEDDI_OPENAI_API_KEY="your-key-here"
export UVEDDI_OLLAMA_URL="http://localhost:11434"

# Security settings
export UVEDDI_SECURITY_ENABLED=true
export UVEDDI_VAULT_URL="https://vault.company.com"
```

---

## Troubleshooting & Best Practices

### Common Issues and Solutions

#### Performance Issues
**Problem**: Slow analysis on large codebases
**Solutions**:
- Enable parallel processing: `--parallel-jobs 8`
- Increase cache size: `--cache-size 5000`
- Use memory optimization features
- Exclude unnecessary files with `.uveddiignore`

#### Memory Issues
**Problem**: High memory usage during analysis
**Solutions**:
- Enable memory optimization feature flag
- Reduce parallel job count
- Use incremental analysis for large projects
- Configure memory limits in analysis config

#### AI Integration Issues
**Problem**: AI provider timeouts or errors
**Solutions**:
- Check provider availability and API keys
- Implement retry logic with exponential backoff
- Use fallback providers
- Enable offline mode for local-only analysis

### Best Practices

#### For Individual Developers
- Use local AI (Ollama) for privacy
- Enable caching for faster repeated analysis
- Configure IDE integration for real-time feedback
- Use TUI for interactive exploration

#### For Teams
- Standardize configuration across team members
- Use CI/CD integration for automated analysis
- Implement security and RBAC for sensitive codebases
- Share custom detectors via plugin system

#### For Enterprises
- Deploy with full security features enabled
- Use chaos engineering for resilience testing
- Implement comprehensive monitoring and alerting
- Integrate with existing observability stack

---

## API Reference

### Command Line Interface

#### Core Commands
```bash
# Analysis commands
uveddi analyze <path> [OPTIONS]
uveddi tui [OPTIONS]

# Configuration commands
uveddi config init
uveddi config edit
uveddi config validate

# Plugin commands
uveddi plugin list
uveddi plugin install <plugin>
uveddi plugin remove <plugin>

# Chaos engineering commands
uveddi chaos run <experiment>
uveddi chaos stop <experiment-id>
uveddi chaos list
```

#### Global Options
- `--config <file>`: Specify configuration file
- `--verbose`: Enable verbose logging
- `--quiet`: Suppress non-error output
- `--help`: Show help information

### Rust API

#### Core Types
```rust
// Analysis engine
pub struct AnalysisEngine { /* ... */ }
pub trait AnalysisDetector { /* ... */ }

// Configuration
pub struct AnalysisConfig { /* ... */ }
pub struct SecurityConfig { /* ... */ }

// Results
pub struct AnalysisResult { /* ... */ }
pub struct ArchitecturalIssue { /* ... */ }
```

#### Usage Examples
```rust
use uveddi::analysis::AnalysisEngine;
use std::path::Path;

// Basic analysis
let engine = AnalysisEngine::new()?;
let (issues, graph) = engine.analyze(Path::new("src/")).await?;

// With custom configuration
let config = AnalysisConfig::from_file(Path::new("config.toml"))?;
let engine = config.create_engine().await?;
```

---

*This comprehensive manual serves as the definitive guide for understanding, developing, and operating Uveddi. For specific implementation details, refer to the source code and inline documentation. For community support, join our [Discord](https://discord.gg/uveddi) or visit our [GitHub Discussions](https://github.com/botzrDev/uveddi/discussions).*