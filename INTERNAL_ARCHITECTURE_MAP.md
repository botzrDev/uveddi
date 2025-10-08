# Uveddi Internal Architecture Map

This document provides a comprehensive internal map of exactly how the Uveddi application works in absolute detail, intended for creating accurate documentation for users and developers.

## Executive Summary

Uveddi is a Rust-based CLI architectural analysis tool that combines static code analysis with optional AI-powered insights. The application follows a modular, service-oriented architecture with clear separation of concerns across multiple layers.

## Core Architecture Overview

### Application Entry Point

**File**: `src/main.rs`

The application uses the `tokio::main` async pattern and follows this initialization sequence:

1. **Error Handling Setup**: `color_eyre::install()` for enhanced error reporting
2. **Logging Configuration**: Unified logging system with configurable levels and formats
3. **CLI Parsing**: Uses `clap` for command-line argument parsing
4. **Command Dispatch**: Routes to appropriate command handlers

**Key Commands**:
- `analyze`: Core analysis functionality
- `config`: Configuration management
- `doctor`: Health diagnostics
- `help`: Help information
- `hooks`: Git hooks management
- `init`: Project initialization
- `ci`: CI/CD integration
- `plugin`: WASM plugin management (feature-gated)

### Feature Flag System

Uveddi uses Cargo feature flags for modular compilation:

**CLI Profiles**:
- `cli-core`: Minimal CLI functionality
- `cli-standard`: Standard CLI with all language support (default)
- `cli-ai`: Standard CLI + AI support
- `cli-plugins`: Standard CLI + WASM plugins
- `cli-full`: Everything (AI + plugins)

**Language Support**:
- `tree-sitter`: All language parsing
- `rust-lang`, `python-lang`, `javascript-lang`, `typescript-lang`: Individual languages
- `languages-core`: Backend languages (Rust, Python)
- `languages-web`: Frontend languages (JavaScript, TypeScript)

## Detailed Component Architecture

### 1. CLI Layer (`src/cli/`)

**Purpose**: Command-line interface and argument handling

**Key Components**:
- `analyze_command.rs`: Main analysis command with comprehensive options
- `config_command.rs`: Configuration management
- `doctor_command.rs`: Health diagnostics
- `plugin_command.rs`: WASM plugin management

**Data Flow**:
1. Parse CLI arguments using clap
2. Validate inputs and security constraints
3. Route to appropriate application services
4. Handle output formatting and error reporting

### 2. Application Layer (`src/application/`)

**Purpose**: Orchestration and business logic coordination

**Key Components**:
- `orchestrator.rs`: Main analysis orchestrator using Facade pattern
- `services/`: Specialized service implementations
- `workflows/`: Analysis workflow definitions
- `configuration.rs`: Configuration management

**Architecture Pattern**: Service-oriented with dependency injection

### 3. Analysis Engine (`src/analysis/`)

**Purpose**: Core analysis pipeline and detector coordination

#### 3.1 Analysis Orchestrator (`orchestrator.rs`)

**Main Entry Point**: `AnalysisOrchestrator`

**Services**:
- `AnalysisService`: Core analysis coordination
- `DependencyAnalysisService`: Dependency graph construction
- `PerformanceAnalysisService`: Performance metrics collection

**Data Flow**:
```
Input Path → File Discovery → AST Parsing → Detector Execution → Result Aggregation → Report Generation
```

#### 3.2 Component System (`src/analysis/components/`)

**Architecture**: Decomposed monolith using specialized components

**Core Components**:
- `ConfigurationService`: Analysis configuration management
- `AstProvider`: AST parsing and caching
- `CacheManager`: AST and analysis result caching
- `DependencyGraphBuilder`: Dependency relationship construction
- `DetectorScheduler`: Detector execution coordination
- `AnalysisAggregator`: Result collection and aggregation
- `PluginManager`: WASM plugin lifecycle (feature-gated)

#### 3.3 Detector System (`src/analysis/detectors/`)

**Base Architecture**:
- `Detector` trait: Core detector interface
- `DetectorConfig` trait: Configuration management
- `DetectorOutput` trait: Result standardization

**Anti-Pattern Detectors** (`src/analysis/detectors/anti_patterns/`):
- `GodObjectDetector`: Identifies oversized classes/structs
- `DeadCodeDetector`: Detects unused code
- `CodeDuplicationDetector`: Finds duplicate code patterns
- `LargeClassDetector`: Identifies overly complex classes
- `LongMethodsDetector`: Detects overly long methods
- `TightCouplingDetector`: Identifies tightly coupled components
- `LeakyAbstractionDetector`: Finds abstraction violations
- `MagicValuesDetector`: Identifies hardcoded values
- `CyclicDependenciesDetector`: Detects circular dependencies

**Security Detectors** (`src/analysis/detectors/security/`):
- OWASP-based vulnerability detection
- Configuration security analysis
- Dependency vulnerability scanning
- Secret detection patterns

### 4. AST Parsing System (`src/ast/`)

**Purpose**: Language-agnostic AST parsing using tree-sitter

**Core Components**:
- `AstParser`: Multi-language parser with caching
- `CustomAst`: High-level AST representation
- `SourceLanguage`: Language enumeration and detection

**Supported Languages**:
- Rust: `tree-sitter-rust`
- Python: `tree-sitter-python`
- JavaScript: `tree-sitter-javascript`
- TypeScript: `tree-sitter-typescript`

**Caching Strategy**:
- LRU cache with bounded memory usage
- File-based persistence for large codebases
- Cache invalidation on file modification

### 5. Plugin System (`src/plugins/`)

**Purpose**: WebAssembly-based extensibility framework

**Architecture**:
- **Wasmtime Runtime**: Secure WASM execution
- **Component Model**: Well-defined plugin interfaces
- **Capability Security**: WASI-based sandboxing
- **Data Plane**: Apache Arrow for zero-copy data exchange

**Key Components**:
- `WasmPluginEngine`: Main plugin runtime
- `PluginRegistry`: Plugin discovery and metadata
- `SecurityPolicy`: Capability-based security
- `DataPlane`: Efficient data serialization

**Plugin Lifecycle**:
1. Discovery: Scan plugin directories
2. Validation: Verify security policies
3. Loading: Load WASM modules
4. Execution: Run analysis with sandboxed access
5. Cleanup: Resource management and unloading

### 6. AI Integration (`src/ai/`)

**Purpose**: AI-powered analysis and explanations

**Architecture**:
- **Provider Abstraction**: Multiple LLM provider support
- **Smart Prompting**: Context-aware prompt generation
- **Knowledge Library**: Compressed pattern data
- **Memory-Aware Selection**: Model selection based on available resources

**Key Components**:
- `AiAnalysisEngine`: Main AI coordination
- `OllamaProvider`: Local Ollama integration
- `SmartPromptBuilder`: Context-aware prompt generation
- `KnowledgeLibrary`: Pattern and rule storage

**AI Flow**:
```
Analysis Results → Context Selection → Prompt Generation → LLM Processing → Insight Generation → Result Integration
```

### 7. Reporting System (`src/report/`)

**Purpose**: Multi-format report generation

**Supported Formats**:
- **Markdown**: Human-readable reports with diagrams
- **JSON**: Structured data for API consumption
- **HTML**: Interactive reports with dark/light themes (disabled in CLI release)

**Key Components**:
- `ReportGenerator`: Main report orchestration
- `MarkdownGenerator`: Markdown format output
- `HtmlGenerator`: Interactive HTML reports
- `MermaidIntegration`: Diagram generation
- `ExecutiveSummary`: Business metrics and summaries

**Report Features**:
- AI-powered explanations and recommendations
- Mermaid.js diagrams for visualization
- Severity-based issue organization
- Performance metrics and trends
- Code snippets and context

### 8. Database Layer (`src/database/`)

**Purpose**: Data persistence and querying

**Technology**: SQLite with bundled library

**Key Models**:
- `AnalysisRun`: Analysis execution metadata
- `ArchitecturalIssue`: Detected issues and findings
- `AntiPatternType`: Issue categorization
- Performance and metrics storage

**Features**:
- Connection pooling
- Query optimization
- Migration support
- In-memory mode for testing

## Data Flow Architecture

### Primary Analysis Pipeline

```
1. Input Processing
   ├── CLI Argument Parsing
   ├── Configuration Loading
   └── Security Validation

2. Discovery Phase
   ├── Workspace Detection
   ├── File Discovery
   └── Language Classification

3. Parsing Phase
   ├── AST Generation (tree-sitter)
   ├── Symbol Table Construction
   └── Cache Population

4. Analysis Phase
   ├── Detector Scheduling
   ├── Parallel Execution
   ├── Result Collection
   └── AI Enhancement (optional)

5. Post-Processing
   ├── Dependency Graph Construction
   ├── Metrics Calculation
   └── Aggregation

6. Output Generation
   ├── Report Formatting
   ├── File Writing
   └── Dashboard Notification (historical)
```

### Plugin Integration Flow

```
Plugin Discovery → Security Validation → WASM Loading → Data Exchange → Analysis Execution → Result Collection → Cleanup
```

### AI Enhancement Flow

```
Issue Collection → Context Building → Prompt Generation → LLM Processing → Insight Extraction → Result Integration
```

## Performance Characteristics

### Time Complexity
- **File Discovery**: O(n) where n is number of files
- **AST Parsing**: O(m) where m is total lines of code
- **Detector Execution**: O(d × f) where d is detectors, f is files
- **Overall**: O(n + m + d×f) with parallelization

### Memory Usage
- **AST Cache**: Bounded LRU cache (configurable size)
- **Analysis Results**: Proportional to codebase complexity
- **Plugin Runtime**: Isolated WASM memory spaces
- **AI Context**: Bounded by model context windows

### Parallelization
- **File Processing**: Parallel using Rayon
- **Detector Execution**: Concurrent detector runs
- **AI Processing**: Async LLM calls
- **Report Generation**: Streamed output processing

## Security Architecture

### Input Validation
- Path traversal protection
- File size limits
- File type validation
- Argument sanitization

### Plugin Security
- WASI capability sandboxing
- Resource limits (CPU, memory)
- File system access controls
- Network access restrictions

### AI Security
- Prompt injection protection
- Data sanitization
- API key management
- Rate limiting

## Configuration System

### Configuration Sources
1. CLI arguments (highest priority)
2. Environment variables
3. Configuration files (`uveddi.toml`)
4. Default values (lowest priority)

### Key Configuration Areas
- Analysis settings and thresholds
- Detector configurations
- AI provider settings
- Plugin security policies
- Output formatting options
- Caching parameters

## Error Handling Strategy

### Error Categories
- `ConfigurationError`: Setup and validation issues
- `AnalysisError`: Analysis execution problems
- `IoError`: File system operations
- `PluginError`: WASM plugin issues
- `AiError`: AI provider problems

### Error Propagation
- `Result<T, UveddiError>` throughout
- Context preservation with error chaining
- User-friendly error messages
- Debug information in development

## Testing Architecture

### Test Categories
- **Unit Tests**: Individual component testing
- **Integration Tests**: Service interaction testing
- **End-to-End Tests**: Full pipeline testing
- **Performance Tests**: Benchmarking and profiling

### Test Organization
- `tests/`: Integration and end-to-end tests
- `src/*/tests/`: Unit tests per module
- `benches/`: Performance benchmarks
- `fixtures/`: Test data and samples

## Build and Deployment

### Build Profiles
- `dev-ultra-fast`: Fastest compilation for development
- `dev-fast`: Memory-optimized development
- `test-fast`: CI/CD optimized testing
- `release`: Production optimization
- `release-small`: Size-constrained environments

### Feature Compilation
- Modular feature flags for reduced binary size
- Conditional compilation for optional components
- Platform-specific optimizations
- Dependency management for reproducible builds

## Web Dashboard (Historical)

**Note**: Web dashboard components were removed in CLI release but architecture documented for context.

### Previous Architecture
- **Frontend**: React SPA with TypeScript
- **API Server**: Express.js with REST endpoints
- **Real-time Updates**: WebSocket integration
- **Authentication**: JWT-based security

### Current Status
- Code references remain for compatibility
- Dashboard launch commands disabled
- Report generation maintains JSON output for potential future integration

## Extension Points

### Custom Detectors
1. Implement `Detector` trait
2. Define configuration struct
3. Register in detector factory
4. Add tests and documentation

### Custom Plugins
1. Create WASM module with WIT interface
2. Implement analysis logic
3. Create plugin manifest
4. Register with plugin engine

### Custom Report Formats
1. Implement format-specific generator
2. Add to report orchestrator
3. Support configuration options
4. Add template system integration

## Migration and Compatibility

### Legacy Support
- Backward-compatible APIs
- Configuration migration helpers
- Deprecation warnings
- Gradual migration paths

### Version Strategy
- Semantic versioning
- Breaking change documentation
- Migration guides
- Compatibility matrices

This architecture map provides the foundation for creating comprehensive user and developer documentation, ensuring accurate representation of the system's internal workings and capabilities.