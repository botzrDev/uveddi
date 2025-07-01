# Uveddi Architecture Documentation

## Overview

Uveddi follows a layered, modular architecture designed for maintainability, testability, and extensibility. This document defines the architectural layers, their responsibilities, and the boundaries between them.

## Architectural Principles

1. **Separation of Concerns**: Each module has a single, well-defined responsibility
2. **Dependency Inversion**: Higher-level modules do not depend on lower-level modules
3. **Interface Segregation**: Modules expose minimal, focused interfaces
4. **Fail-Fast Design**: Errors are caught early and propagated cleanly
5. **Plugin Extensibility**: Core functionality can be extended without modifying the main codebase

## Layer Architecture

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
│                     Analysis Layer                          │
│           (Core Business Logic & Engines)                  │
└─────────────────────────────────────────────────────────────┘
                             │
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                      │
│          (AST, AI, Database, Plugin System)                │
└─────────────────────────────────────────────────────────────┘
                             │
┌─────────────────────────────────────────────────────────────┐
│                     Platform Layer                          │
│              (OS, File System, Network)                    │
└─────────────────────────────────────────────────────────────┘
```

## Layer Definitions

### 1. CLI Layer (`src/cli/`)
**Responsibility**: User interface and command parsing
**Dependencies**: Application Layer only
**Public Interface**: CLI commands and arguments

**Modules**:
- `analyze_command.rs` - Handles the analyze subcommand
- `config_command.rs` - Configuration management
- `init_local_ai_command.rs` - Local AI setup
- `plugin_command.rs` - Plugin management

**Key Constraints**:
- ❌ Must NOT directly access infrastructure layers (AST, Database, AI)
- ✅ Should delegate all business logic to Application Layer
- ✅ Should handle only user input validation and formatting

### 2. Application Layer (`src/main.rs`, orchestration logic)
**Responsibility**: Command orchestration and workflow management
**Dependencies**: Analysis Layer, Configuration
**Public Interface**: Command execution workflows

**Key Functions**:
- Coordinate analysis workflows
- Manage configuration loading
- Handle error reporting to users
- Orchestrate multi-step operations

**Key Constraints**:
- ❌ Must NOT contain business logic
- ✅ Should orchestrate calls between Analysis Layer components
- ✅ Should handle cross-cutting concerns (logging, metrics)

### 3. Analysis Layer (`src/analysis/`)
**Responsibility**: Core business logic and analysis engines
**Dependencies**: Infrastructure Layer (AST, AI, Models)
**Public Interface**: Analysis engines and detectors

**Modules**:
- `engine.rs` - Main analysis orchestration
- `dependency_extractor.rs` - Dependency analysis
- `god_object_detector.rs` - Anti-pattern detection
- `cycle_detector.rs` - Circular dependency detection

**Key Constraints**:
- ✅ Contains all core business logic
- ✅ Should be testable in isolation
- ❌ Must NOT depend on CLI or Application layers
- ✅ Should use Infrastructure Layer through well-defined interfaces

### 4. Infrastructure Layer
**Responsibility**: Technical services and external integrations
**Dependencies**: Platform Layer only
**Public Interface**: Service abstractions and traits

#### 4.1 AST Subsystem (`src/ast/`)
- `tree_sitter/` - Tree-sitter integration
- Language-specific parsers (JavaScript, Python, Rust, etc.)

#### 4.2 AI Subsystem (`src/ai/`)
- `providers/` - AI provider implementations
- `engine.rs` - AI analysis orchestration
- Provider abstractions (`LlmProvider` trait)

#### 4.3 Data Layer (`src/database/`, `src/models/`)
- Database connection management
- Model definitions and migrations
- Query abstractions

#### 4.4 Plugin System (`src/plugin/`)
- WASM runtime management
- Plugin loading and lifecycle
- Plugin API definitions

#### 4.5 Supporting Infrastructure
- `src/cache/` - Caching layer
- `src/config/` - Configuration management
- `src/error.rs` - Unified error handling
- `src/report/` - Report generation

**Key Constraints**:
- ✅ Should provide stable, well-defined interfaces
- ❌ Must NOT depend on higher layers
- ✅ Should handle all external system integration
- ✅ Should be mockable for testing

### 5. Platform Layer
**Responsibility**: Operating system and external system interfaces
**Dependencies**: None (standard library, OS APIs)
**Public Interface**: File system, network, process interfaces

## Interface Boundaries

### Public APIs
Each layer exposes a minimal public API through traits and structs:

```rust
// Analysis Layer Public Interface
pub trait AnalysisEngine {
    async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResult, UveddiError>;
}

// AI Infrastructure Interface  
pub trait LlmProvider {
    async fn analyze_code(&self, context: &AiContext) -> Result<AiResponse, UveddiError>;
}

// Database Infrastructure Interface
pub trait AnalysisRepository {
    async fn store_result(&self, result: &AnalysisResult) -> Result<(), UveddiError>;
}
```

### Dependency Flow Rules

1. **Downward Dependencies Only**: Higher layers depend on lower layers, never vice versa
2. **Interface Abstraction**: Dependencies are through traits, not concrete types
3. **Error Propagation**: All errors flow upward through the unified `UveddiError` type
4. **No Skip-Layer Dependencies**: Each layer only depends on the layer immediately below

## Error Handling Strategy

Uveddi uses a unified error handling approach with the `UveddiError` enum:

```rust
// All errors flow through the unified error type
pub enum UveddiError {
    // Layer-specific error categories
    Analysis(String),
    AstParsing(String), 
    AiApi { provider: String, message: String },
    Database(#[from] rusqlite::Error),
    // ... etc
}
```

**Error Flow Principles**:
- Errors originate in Infrastructure Layer
- Each layer adds context using `ErrContext` trait
- CLI Layer formats errors for user consumption
- Business logic errors are separated from technical errors

## Configuration Management

Configuration follows the layered approach:

1. **CLI Layer**: Command-line arguments and flags
2. **Application Layer**: Configuration file loading and merging
3. **Infrastructure Layer**: Service-specific configuration

```rust
// Hierarchical configuration structure
pub struct UveddiConfig {
    pub ai: AiConfig,
    pub database: DatabaseConfig,
    pub analysis: AnalysisConfig,
    pub plugins: PluginConfig,
}
```

## Plugin Architecture

The plugin system follows a strict isolation model:

```
┌─────────────┐    WASM     ┌─────────────┐
│   Core      │ ←--------→  │   Plugin    │
│   System    │  Boundary   │    (WASM)   │
└─────────────┘             └─────────────┘
```

**Plugin Constraints**:
- Plugins run in WASM sandbox for security
- Plugins communicate only through defined API
- Plugins cannot access file system or network directly
- Plugin lifecycle managed by Infrastructure Layer

## Testing Strategy by Layer

### CLI Layer Testing
- Integration tests for command parsing
- Mock Application Layer dependencies
- Test error message formatting

### Application Layer Testing  
- Workflow integration tests
- Mock Analysis and Infrastructure layers
- Test orchestration logic

### Analysis Layer Testing
- Unit tests for business logic
- Mock Infrastructure dependencies
- Property-based testing for complex algorithms

### Infrastructure Layer Testing
- Component tests for each subsystem
- Integration tests with external systems
- Performance tests for critical paths

## Performance Considerations

### Async Boundaries
- CLI Layer: Synchronous (user interaction)
- Application Layer: Async orchestration
- Analysis Layer: Async analysis operations
- Infrastructure Layer: Async I/O operations

### Caching Strategy
- AST parsing results cached at Infrastructure Layer
- Analysis results cached at Analysis Layer
- Configuration cached at Application Layer

### Memory Management
- Stream processing for large codebases
- Lazy loading of language parsers
- Plugin lifecycle management to prevent memory leaks

## Development Guidelines

### Adding New Features
1. Identify the appropriate layer for the feature
2. Define interfaces before implementation
3. Add error handling through `UveddiError`
4. Write tests at the appropriate layer
5. Update this documentation

### Refactoring Guidelines
1. Maintain layer boundaries during refactoring
2. Update interfaces before changing implementations
3. Ensure error propagation still works correctly
4. Update tests to reflect architectural changes

### Code Review Checklist
- [ ] Does this change respect layer boundaries?
- [ ] Are dependencies flowing in the correct direction?
- [ ] Is error handling following the unified pattern?
- [ ] Are interfaces minimal and well-defined?
- [ ] Is the change testable in isolation?

---

This architecture provides a solid foundation for long-term maintainability while supporting the complex requirements of AI-powered code analysis.
