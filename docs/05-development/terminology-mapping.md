# Uveddi Terminology Mapping

This document maps standardized ubiquitous language terms to actual module boundaries, architectural components, and code structures in the Uveddi codebase. It serves as a bridge between conceptual domain language and implementation details.

## Core Analysis Domain Mapping

### Analysis Engine (`src/analysis/engine.rs`)
- **Ubiquitous Language Term:** Analysis Engine
- **Implementation:** `AnalysisEngine` struct
- **Module Path:** `src/analysis/engine.rs`
- **Key Components:**
  - `AnalysisEngineBuilder` - Builder pattern implementation
  - `AnalysisConfig` - Configuration container
  - `run_analysis()` - Main orchestration method
- **Dependencies:** Detector registry, configuration service, report generator

### Detectors (`src/analysis/detectors/`)
- **Ubiquitous Language Term:** Detector
- **Implementation Pattern:** `{Purpose}Detector` structs implementing `AnalysisDetector` trait
- **Module Structure:**
  ```
  src/analysis/detectors/
  ├── anti_patterns/           # Anti-pattern specific detectors
  │   ├── god_object.rs       # GodObjectDetector
  │   ├── cyclic_dependency.rs # CyclicDependencyDetector
  │   └── tight_coupling.rs   # TightCouplingDetector
  ├── mod.rs                  # Detector registry and factory
  └── traits.rs               # AnalysisDetector trait definition
  ```
- **Key Traits:**
  - `AnalysisDetector` - Core detector interface
  - `DetectorConfig` - Configuration interface
- **Registration:** `DetectorRegistry` in `mod.rs`

### Anti-Pattern Types (`src/analysis/detectors/anti_patterns/`)
- **Ubiquitous Language Terms:** God Object, Cyclic Dependency, Tight Coupling, etc.
- **Implementation Mapping:**
  - **God Object** → `src/analysis/detectors/anti_patterns/god_object.rs`
  - **Cyclic Dependency** → `src/analysis/detectors/anti_patterns/cyclic_dependency.rs`
  - **Tight Coupling** → `src/analysis/detectors/anti_patterns/tight_coupling.rs`
  - **Dead Code** → `src/analysis/detectors/anti_patterns/dead_code.rs`
  - **Long Parameter List** → `src/analysis/detectors/anti_patterns/long_parameter_list.rs`
  - **Feature Envy** → `src/analysis/detectors/anti_patterns/feature_envy.rs`

### Analysis Results (`src/analysis/results.rs`)
- **Ubiquitous Language Terms:** Analysis Result, Architectural Issue, Criticality Level
- **Implementation:**
  - `AnalysisResult` struct - Container for all analysis outputs
  - `ArchitecturalIssue` struct - Individual issue representation
  - `CriticalityLevel` enum - Severity classification (Critical/Warning/Suggestion)
  - `IssueLocation` struct - Source location information
- **Usage:** Returned by detectors and aggregated by analysis engine

## AI Integration Domain Mapping

### LLM Providers (`src/ai/`)
- **Ubiquitous Language Term:** LLM Provider
- **Implementation Structure:**
  ```
  src/ai/
  ├── mod.rs                  # Provider registry and factory
  ├── traits.rs              # LlmProvider trait
  ├── ollama_provider.rs      # OllamaProvider implementation
  ├── openai_provider.rs      # OpenAI provider (if enabled)
  └── context_builder.rs      # Context construction utilities
  ```
- **Key Traits:**
  - `LlmProvider` - Provider interface
  - `ContextBuilder` - Context construction interface

### AI Analysis Components (`src/ai/analysis.rs`)
- **Ubiquitous Language Term:** AI Analysis
- **Implementation:** `AiAnalysisEngine` struct
- **Integration Point:** Used by main `AnalysisEngine` when AI features enabled
- **Key Methods:**
  - `enhance_analysis()` - Add AI insights to analysis results
  - `explain_issue()` - Generate natural language explanations

### Prompt System (`src/ai/prompts/`)
- **Ubiquitous Language Terms:** Prompt Template, Smart Prompting
- **Implementation Structure:**
  ```
  src/ai/prompts/
  ├── mod.rs                  # Prompt registry and utilities
  ├── analysis_prompts.rs     # Analysis-specific templates
  ├── explanation_prompts.rs  # Issue explanation templates
  └── context_templates.rs    # Context construction templates
  ```
- **Key Components:**
  - `PromptTemplate` struct - Template definition
  - `PromptBuilder` - Dynamic prompt construction
  - Template files in TOML format for easy customization

## Plugin System Domain Mapping

### WASM Plugin Engine (`src/plugins/`)
- **Ubiquitous Language Terms:** WASM Plugin, Plugin Registry, Plugin Lifecycle Manager
- **Implementation Structure:**
  ```
  src/plugins/
  ├── mod.rs                  # Plugin system exports
  ├── engine.rs               # WasmPluginEngine
  ├── registry.rs             # PluginRegistry
  ├── lifecycle.rs            # PluginLifecycleManager
  ├── manifest.rs             # PluginManifest parsing
  ├── security.rs             # SecurityPolicy enforcement
  └── verifier.rs             # PluginVerifier implementation
  ```

### Plugin Architecture Components
- **Plugin Manifest** → `PluginManifest` struct in `manifest.rs`
- **Security Policy** → `SecurityPolicy` struct in `security.rs`
- **Plugin Verifier** → `PluginVerifier` trait and implementations in `verifier.rs`
- **WASM Runtime** → Integration with wasmtime in `engine.rs`

## Security & RBAC Domain Mapping

### Security Framework (`src/security/`)
- **Ubiquitous Language Terms:** RBAC, User Role, Authentication Service, Authorization Service
- **Implementation Structure:**
  ```
  src/security/
  ├── mod.rs                  # Security module exports
  ├── rbac/                   # Role-Based Access Control
  │   ├── roles.rs           # UserRole definitions
  │   ├── permissions.rs     # Permission system
  │   └── policies.rs        # SecurityPolicy implementation
  ├── auth/                   # Authentication & Authorization
  │   ├── authentication.rs  # AuthenticationService
  │   ├── authorization.rs   # AuthorizationService
  │   └── tokens.rs          # JWT token handling
  ├── audit/                  # Audit and Compliance
  │   ├── logger.rs          # AuditLogger implementation
  │   ├── trail.rs           # AuditTrail data structures
  │   └── compliance.rs      # ComplianceValidator
  └── middleware.rs           # Security middleware
  ```

### Security Components Mapping
- **RBAC** → `src/security/rbac/` module with role and permission management
- **Authentication Service** → `AuthenticationService` in `src/security/auth/authentication.rs`
- **Authorization Service** → `AuthorizationService` in `src/security/auth/authorization.rs`
- **Audit Logger** → `AuditLogger` in `src/security/audit/logger.rs`
- **Compliance Validator** → `ComplianceValidator` in `src/security/audit/compliance.rs`

## Configuration Domain Mapping

### Configuration System (`src/config/`)
- **Ubiquitous Language Terms:** Configuration, Configuration Service, Configuration Schema
- **Implementation Structure:**
  ```
  src/config/
  ├── mod.rs                  # Configuration exports
  ├── service.rs              # ConfigurationService
  ├── schema.rs               # ConfigurationSchema definitions
  ├── loader.rs              # Configuration loading utilities
  └── validation.rs          # Configuration validation
  ```
- **Configuration Files:**
  - `config/default.toml` - Default configuration
  - `config/security.toml` - Security-specific settings
  - `config/ai.toml` - AI provider configurations

### Configuration Component Mapping
- **Configuration Service** → `ConfigurationService` in `src/config/service.rs`
- **Configuration Schema** → Schema definitions in `src/config/schema.rs`
- **Configuration Loading** → Multi-source loading in `src/config/loader.rs`

## Monitoring & Observability Domain Mapping

### Monitoring System (`src/monitoring/`)
- **Ubiquitous Language Terms:** Telemetry, Performance Metrics, Health Check
- **Implementation Structure:**
  ```
  src/monitoring/
  ├── mod.rs                  # Monitoring exports
  ├── telemetry.rs           # Telemetry collection
  ├── metrics.rs             # Performance metrics
  ├── health.rs              # Health check endpoints
  ├── circuit_breaker.rs     # Circuit breaker implementation
  └── rate_limiter.rs        # Rate limiting utilities
  ```

### Observability Components (`src/observability/`)
- **Advanced Monitoring:** `src/observability/` for complex monitoring scenarios
- **Distributed Tracing:** OpenTelemetry integration
- **Custom Metrics:** Application-specific performance indicators

## Technical Architecture Mapping

### Parsing and AST (`src/parsing/`)
- **Ubiquitous Language Terms:** AST, Tree-sitter, Incremental Analysis
- **Implementation:**
  - Tree-sitter integration for multi-language parsing
  - AST caching and optimization
  - Incremental parsing for performance

### Memory Management (`src/analysis/memory/`)
- **Ubiquitous Language Term:** Memory Optimization
- **Implementation:**
  - Arena allocation patterns
  - Zero-copy serialization with rkyv
  - Memory-mapped file handling
  - Custom allocator integration

### Caching System (`src/cache/`)
- **Ubiquitous Language Term:** Cache Manager
- **Implementation:**
  - `CacheManager` for analysis result caching
  - AST cache management
  - Distributed cache support for enterprise features

## Reporting Domain Mapping

### Report Generation (`src/report/`)
- **Ubiquitous Language Terms:** Analysis Report, Report Generator, Visualization
- **Implementation Structure:**
  ```
  src/report/
  ├── mod.rs                  # Report system exports
  ├── generator.rs           # ReportGenerator implementation
  ├── templates/             # Report templates
  │   ├── markdown.rs       # Markdown report template
  │   ├── json.rs           # JSON report template
  │   └── html.rs           # HTML report template
  ├── visualization.rs       # Chart and graph generation
  └── formats.rs             # Output format definitions
  ```

### Visualization Components
- **Report Generator** → `ReportGenerator` in `src/report/generator.rs`
- **Report Templates** → Template implementations in `src/report/templates/`
- **Visualization** → Chart generation in `src/report/visualization.rs`

## User Interface Domain Mapping

### Terminal UI (`src/tui/`)
- **Ubiquitous Language Terms:** TUI, Interactive Mode
- **Implementation:** Full TUI implementation using ratatui
- **Key Components:**
  - Event handling system
  - Screen management
  - Interactive analysis workflows

### Command Line Interface (`src/cli/`)
- **Ubiquitous Language Terms:** CLI, Batch Mode
- **Implementation:** Clap-based CLI with comprehensive command structure
- **Commands:**
  - `analyze` - Run analysis on codebase
  - `config` - Manage configuration
  - `plugin` - Plugin management
  - `tui` - Launch terminal interface

## Module Boundary Alignment

### Domain Service Boundaries
The codebase follows Domain-Driven Design principles with clear boundaries:

1. **Analysis Domain** (`src/analysis/`) - Core analysis functionality
2. **AI Domain** (`src/ai/`) - AI integration and enhancement
3. **Plugin Domain** (`src/plugins/`) - Plugin system and WASM runtime
4. **Security Domain** (`src/security/`) - Authentication, authorization, audit
5. **Configuration Domain** (`src/config/`) - Configuration management
6. **Monitoring Domain** (`src/monitoring/`, `src/observability/`) - Observability
7. **Reporting Domain** (`src/report/`) - Output generation and visualization
8. **Interface Domain** (`src/cli/`, `src/tui/`) - User interaction layers

### Cross-Cutting Concerns
Some components span multiple domains:
- **Error Handling** - Standardized across all domains
- **Logging** - Integrated into all major components
- **Testing** - Domain-specific test suites in `tests/`

## Implementation Guidelines

### Naming Consistency
All implementations follow the standardized naming patterns:
- Struct names use PascalCase matching ubiquitous language terms
- Module names use snake_case reflecting domain boundaries
- Function names use snake_case following Rust conventions
- Trait names clearly indicate interfaces (e.g., `AnalysisDetector`, `LlmProvider`)

### Documentation Alignment
- All public APIs documented with standardized terminology
- Code comments use ubiquitous language consistently
- Examples in documentation follow naming conventions
- Error messages use standardized terminology

### Future Evolution
This mapping document should be updated when:
- New domain concepts are introduced
- Module boundaries change
- Architectural components are refactored
- New integration points are added

The goal is maintaining perfect alignment between conceptual domain language and implementation structure, ensuring clarity for all team members and contributors.