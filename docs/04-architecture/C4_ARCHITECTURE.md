# Uveddi C4 Architecture Model

## C4 Model Overview

The C4 model provides a hierarchical view of Uveddi's architecture through four levels:
1. **Context** - How Uveddi fits into the overall environment
2. **Container** - High-level technology choices and communication
3. **Component** - Internal structure of the Rust CLI application
4. **Code** - Implementation details (classes, functions)

## Level 1: System Context Diagram

```mermaid
graph TB
    subgraph "Software Development Environment"
        DEV[Developer/Tech Lead]
        CI[CI/CD Pipeline]
        IDE[IDE/VS Code]
        REPO[Code Repository]
    end
    
    subgraph "AI Services"
        OPENAI[OpenAI GPT-4]
        ANTHROPIC[Anthropic Claude]
        OLLAMA[Local Ollama]
    end
    
    subgraph "Data Storage"
        SQLITE[Local SQLite DB]
        POSTGRES[Cloud PostgreSQL]
    end
    
    UVEDDI[Uveddi CLI Tool]
    
    DEV -->|runs analysis| UVEDDI
    CI -->|automated analysis| UVEDDI
    IDE -->|integrates with| UVEDDI
    UVEDDI -->|analyzes| REPO
    UVEDDI -->|queries| OPENAI
    UVEDDI -->|queries| ANTHROPIC  
    UVEDDI -->|queries| OLLAMA
    UVEDDI -->|stores results| SQLITE
    UVEDDI -->|syncs data| POSTGRES
    UVEDDI -->|generates reports| DEV
```

**Key Relationships:**
- **Developers** use Uveddi to analyze codebases and receive architectural insights
- **CI/CD Pipelines** integrate Uveddi for automated architecture validation
- **AI Services** provide code analysis capabilities (hybrid local/cloud approach)
- **Databases** store analysis results and metadata for tracking over time

## Level 2: Container Diagram

```mermaid
graph TB
    subgraph "Uveddi System"
        CLI[Rust CLI Application<br/>Main analysis engine]
        BACKEND[Python FastAPI Backend<br/>Optional web interface]
    end
    
    subgraph "External AI Services"
        OPENAI[OpenAI API]
        ANTHROPIC[Anthropic API]
        GEMINI[Google Gemini API]
    end
    
    subgraph "Local AI"
        OLLAMA[Ollama Server<br/>Local LLM hosting]
    end
    
    subgraph "Data Layer"
        SQLITE[(SQLite Database<br/>Local analysis cache)]
        POSTGRES[(PostgreSQL<br/>Team collaboration)]
    end
    
    subgraph "File System"
        CODEBASE[Target Codebase<br/>Source code to analyze]
        CACHE[AST Cache<br/>Parsed syntax trees]
        REPORTS[Generated Reports<br/>Markdown/JSON output]
    end
    
    CLI -->|HTTP requests| OPENAI
    CLI -->|HTTP requests| ANTHROPIC
    CLI -->|HTTP requests| GEMINI
    CLI -->|HTTP requests| OLLAMA
    CLI -->|SQL queries| SQLITE
    CLI -->|SQL queries| POSTGRES
    CLI -->|reads/parses| CODEBASE
    CLI -->|reads/writes| CACHE
    CLI -->|generates| REPORTS
    BACKEND -->|SQL queries| POSTGRES
    BACKEND -->|serves| CLI
```

**Technology Choices:**
- **Rust CLI**: High-performance analysis with excellent error handling
- **Python Backend**: Optional web interface for team collaboration  
- **SQLite**: Fast local caching and development
- **PostgreSQL**: Production-grade team data storage
- **Tree-sitter**: Multi-language AST parsing
- **WASM**: Secure plugin sandboxing

## Level 3: Component Diagram - Rust CLI Application

```mermaid
graph TB
    subgraph "CLI Layer"
        MAIN[Main Entry Point]
        ANALYZE[Analyze Command]
        CONFIG[Config Command]
        PLUGIN[Plugin Command]
    end
    
    subgraph "Application Layer"
        ORCHESTRATOR[Analysis Orchestrator]
        WORKFLOW[Workflow Manager]
    end
    
    subgraph "Analysis Layer (Core Business Logic)"
        ENGINE[Analysis Engine]
        DETECTORS[Anti-pattern Detectors]
        EXTRACTOR[Dependency Extractor]
        CYCLE[Cycle Detector]
    end
    
    subgraph "Infrastructure Layer"
        subgraph "AST Subsystem"
            AST_ENGINE[AST Engine]
            TS_PARSER[Tree-sitter Parser]
            LANG_SUPPORT[Language Support]
        end
        
        subgraph "AI Subsystem"
            AI_ENGINE[AI Analysis Engine]
            PROVIDERS[LLM Providers]
            CONTEXT[AI Context Builder]
        end
        
        subgraph "Data Subsystem"
            DB_CONN[Database Connection]
            MODELS[Data Models]
            MIGRATIONS[Schema Migrations]
        end
        
        subgraph "Plugin Subsystem"
            PLUGIN_RUNTIME[WASM Runtime]
            PLUGIN_API[Plugin API]
            PLUGIN_LOADER[Plugin Loader]
        end
        
        subgraph "Support Services"
            CACHE[Cache Manager]
            CONFIG_MGR[Config Manager]
            REPORTER[Report Generator]
            ERROR[Error Handler]
        end
    end
    
    MAIN --> ANALYZE
    MAIN --> CONFIG
    MAIN --> PLUGIN
    
    ANALYZE --> ORCHESTRATOR
    CONFIG --> CONFIG_MGR
    PLUGIN --> PLUGIN_RUNTIME
    
    ORCHESTRATOR --> WORKFLOW
    WORKFLOW --> ENGINE
    
    ENGINE --> DETECTORS
    ENGINE --> EXTRACTOR
    ENGINE --> CYCLE
    ENGINE --> AI_ENGINE
    
    DETECTORS --> AST_ENGINE
    EXTRACTOR --> AST_ENGINE
    CYCLE --> AST_ENGINE
    
    AST_ENGINE --> TS_PARSER
    TS_PARSER --> LANG_SUPPORT
    
    AI_ENGINE --> PROVIDERS
    AI_ENGINE --> CONTEXT
    
    ENGINE --> DB_CONN
    DB_CONN --> MODELS
    
    ENGINE --> CACHE
    ENGINE --> REPORTER
    
    %% Error handling flows through all components
    ERROR -.-> ENGINE
    ERROR -.-> AI_ENGINE
    ERROR -.-> AST_ENGINE
    ERROR -.-> DB_CONN
```

**Component Responsibilities:**

### CLI Layer Components
- **Main Entry Point**: Command parsing and delegation
- **Commands**: Individual command implementations (analyze, config, plugin)

### Application Layer Components  
- **Analysis Orchestrator**: Coordinates high-level analysis workflows
- **Workflow Manager**: Manages multi-step analysis processes

### Analysis Layer Components
- **Analysis Engine**: Core business logic coordinator
- **Anti-pattern Detectors**: Specialized detectors for architectural smells
- **Dependency Extractor**: Extracts and analyzes code dependencies
- **Cycle Detector**: Identifies circular dependencies

### Infrastructure Components
- **AST Engine**: Manages syntax tree parsing and analysis
- **AI Engine**: Coordinates AI-powered analysis
- **Database Components**: Handle data persistence and querying
- **Plugin System**: Manages WASM-based plugin execution
- **Support Services**: Cross-cutting concerns (caching, config, reporting)

## Level 4: Code Diagram - Key Interfaces

```mermaid
classDiagram
    class AnalysisEngine {
        +analyze(request: AnalysisRequest) Result~AnalysisResult~
        +get_detectors() Vec~Box~dyn AnalysisDetector~~
    }
    
    class AnalysisDetector {
        <<interface>>
        +detect(ast: &AstNode) Result~Vec~Finding~~
        +name() &str
        +description() &str
    }
    
    class LlmProvider {
        <<interface>>
        +analyze_code(context: &AiContext) Result~AiResponse~
        +get_provider_name() &str
        +is_available() bool
    }
    
    class AstParser {
        <<interface>>
        +parse_file(path: &Path) Result~AstNode~
        +supported_languages() Vec~Language~
    }
    
    class AnalysisRepository {
        <<interface>>
        +store_result(result: &AnalysisResult) Result~()~
        +get_results(query: &Query) Result~Vec~AnalysisResult~~
    }
    
    class UveddiError {
        <<enumeration>>
        Analysis(String)
        AstParsing(String)
        AiApi{provider: String, message: String}
        Database(rusqlite::Error)
        Configuration(String)
        Plugin(String)
    }
    
    AnalysisEngine --> AnalysisDetector : uses
    AnalysisEngine --> LlmProvider : uses
    AnalysisEngine --> AstParser : uses
    AnalysisEngine --> AnalysisRepository : uses
    AnalysisEngine --> UveddiError : returns
    
    AnalysisDetector --> UveddiError : returns
    LlmProvider --> UveddiError : returns
    AstParser --> UveddiError : returns
    AnalysisRepository --> UveddiError : returns
```

## Data Flow Patterns

### Analysis Request Flow
```mermaid
sequenceDiagram
    participant CLI as CLI Command
    participant Orch as Orchestrator
    participant Engine as Analysis Engine
    participant AST as AST Parser
    participant AI as AI Engine
    participant DB as Database
    
    CLI->>Orch: execute_analysis(path)
    Orch->>Engine: analyze(request)
    Engine->>AST: parse_codebase(path)
    AST-->>Engine: ast_results
    Engine->>AI: analyze_with_ai(ast_context)
    AI-->>Engine: ai_insights
    Engine->>DB: store_results(findings)
    Engine-->>Orch: analysis_result
    Orch-->>CLI: formatted_report
```

### Plugin Loading Flow
```mermaid
sequenceDiagram
    participant CLI as Plugin Command
    participant Runtime as WASM Runtime
    participant Loader as Plugin Loader
    participant FS as File System
    
    CLI->>Runtime: load_plugin(path)
    Runtime->>Loader: discover_plugins(directory)
    Loader->>FS: read_plugin_files()
    FS-->>Loader: wasm_modules
    Loader->>Runtime: validate_and_load(modules)
    Runtime-->>CLI: loaded_plugins
```

## Architectural Decision Records

### ADR-001: Layered Architecture Choice
**Decision**: Implement a strict layered architecture with dependency inversion
**Rationale**: Provides clear separation of concerns and testability
**Consequences**: More initial complexity but better long-term maintainability

### ADR-002: Unified Error Handling
**Decision**: Use a single `UveddiError` enum for all error types
**Rationale**: Simplifies error propagation and handling across layers
**Consequences**: Requires careful error categorization but improves consistency

### ADR-003: WASM Plugin System
**Decision**: Use WebAssembly for plugin sandboxing
**Rationale**: Provides security isolation while maintaining performance
**Consequences**: Additional complexity but enables safe extensibility

### ADR-004: Hybrid AI Architecture
**Decision**: Support both local and cloud AI providers
**Rationale**: Balances privacy concerns with analysis quality
**Consequences**: More complex provider management but flexible deployment

---

This C4 model provides a comprehensive view of Uveddi's architecture, from high-level system context down to implementation details. Use these diagrams to communicate architectural decisions and guide development work.
