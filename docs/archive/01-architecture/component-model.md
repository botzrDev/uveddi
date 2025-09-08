# Component Model

> **Status**: Architecture documentation in development for v0.9.0-alpha

Uveddi follows a layered architectural pattern with clear separation of concerns and dependency inversion.

## Core Layers

### CLI Layer (`src/cli/`)
- Command-line interface and argument parsing
- User interaction and command dispatch
- Entry point for all operations

### Application Layer (`src/application/`)
- Business logic orchestration
- Use case implementations
- Service coordination

### Analysis Layer (`src/analysis/`)
- Core analysis engines and algorithms
- Anti-pattern detection (God Object, Dead Code, etc.)
- AST processing and code metrics

### Infrastructure Layer
- **AST Parsing** (`src/ast/`): Tree-sitter based parsing for multiple languages
- **AI Integration** (`src/ai/`): Ollama and future AI provider integrations
- **Database** (`src/database/`): SQLite-based persistence
- **Plugins** (`src/plugins/`): WebAssembly plugin system

### Platform Layer
- Operating system abstractions
- File system operations
- Network communications

## Component Dependencies

```
CLI → Application → Analysis → Infrastructure → Platform
```

**Key Principles:**
- Dependencies flow downward only
- Each layer only knows about the layer directly below
- Shared types prevent circular dependencies (e.g., `src/api/types.rs`)
- Dependency injection used for testing and flexibility

## Service Components

### Core Services
- **Analysis Engine**: Central processing component
- **Report Generator**: Multi-format output generation
- **Configuration Manager**: Settings and preferences
- **Cache Manager**: Performance optimization

### External Services
- **API Server** (`src/api/`): REST endpoints for web integration
- **Rendering Service** (separate binary): Diagram generation with Playwright
- **Database Service**: Data persistence and querying

## Inter-Component Communication

### Synchronous Communication
- Direct function calls within layers
- Trait-based abstractions for testability

### Asynchronous Communication  
- `tokio::sync::oneshot` for service readiness
- Message passing for TUI interactions
- Event-driven architecture for plugin system

## Plugin Architecture

WebAssembly-based plugins extend analysis capabilities:
- Sandboxed execution environment
- Standardized plugin API
- Hot-loading and unloading support

## Future Enhancements

- Microservice decomposition for web deployment
- Event sourcing for analysis history
- Distributed analysis for large codebases

---

For detailed implementation examples, see the [Analysis Engine](./analysis-engine.md) and [Extensibility](./extensibility.md) documentation.