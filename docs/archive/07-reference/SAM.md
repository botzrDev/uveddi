# Uveddi Software Architecture Model (SAM)

## Overview

This document defines the Software Architecture Model (SAM) for Uveddi - the canonical, serialized source of truth for the system architecture. This model serves as the foundation for generating diagrams, validating architectural constraints, and guiding development decisions.

## Architecture Metadata

```yaml
sam_version: "1.0"
system_name: "Uveddi"
system_description: "AI-Powered Architectural Analysis CLI"
last_updated: "2025-07-01"
maintainer: "Uveddi Development Team"
validation_status: "active"
```

## System Context

### System Boundary
```yaml
system:
  name: "Uveddi"
  type: "CLI Application"
  description: "Code analysis tool with AI-powered architectural insights"
  
external_systems:
  - name: "OpenAI API"
    type: "AI Service"
    relationship: "queries for code analysis"
    protocol: "HTTPS/REST"
    
  - name: "Anthropic Claude API"
    type: "AI Service" 
    relationship: "queries for code analysis"
    protocol: "HTTPS/REST"
    
  - name: "Local Ollama"
    type: "Local AI Service"
    relationship: "queries for private analysis"
    protocol: "HTTP/REST"
    
  - name: "Target Codebase"
    type: "File System"
    relationship: "analyzes source code"
    protocol: "File I/O"
    
  - name: "CI/CD Pipeline"
    type: "External System"
    relationship: "integrates analysis into build process"
    protocol: "CLI execution"

actors:
  - name: "Developer"
    type: "Human User"
    description: "Software developer performing local analysis"
    
  - name: "Tech Lead"
    type: "Human User"
    description: "Technical leader reviewing architecture"
    
  - name: "CI System"
    type: "Automated System"
    description: "Continuous integration performing automated checks"
```

## Container Architecture

### Container Definitions
```yaml
containers:
  - id: "rust_cli"
    name: "Rust CLI Application"
    technology: "Rust"
    description: "Main analysis engine and command-line interface"
    responsibilities:
      - "Code parsing and AST generation"
      - "Anti-pattern detection"
      - "AI integration and analysis"
      - "Report generation"
      - "Local data storage"
    
  - id: "python_backend"
    name: "Python FastAPI Backend"
    technology: "Python/FastAPI"
    description: "Optional web interface for team collaboration"
    responsibilities:
      - "Web API endpoints"
      - "Team data synchronization"
      - "Analysis history tracking"
    
  - id: "sqlite_db"
    name: "SQLite Database"
    technology: "SQLite"
    description: "Local analysis results and cache"
    responsibilities:
      - "Analysis result storage"
      - "AST caching"
      - "Configuration persistence"
    
  - id: "postgres_db"
    name: "PostgreSQL Database"
    technology: "PostgreSQL"
    description: "Shared team analysis data"
    responsibilities:
      - "Team analysis results"
      - "Historical trending"
      - "Multi-user collaboration"

container_relationships:
  - from: "rust_cli"
    to: "sqlite_db"
    relationship: "stores/retrieves"
    protocol: "SQL"
    
  - from: "rust_cli"
    to: "postgres_db"
    relationship: "syncs data"
    protocol: "SQL"
    
  - from: "python_backend"
    to: "postgres_db"
    relationship: "queries"
    protocol: "SQL"
    
  - from: "rust_cli"
    to: "python_backend"
    relationship: "optional integration"
    protocol: "HTTP/REST"
```

## Component Architecture (Rust CLI)

### Layer Definitions
```yaml
layers:
  - id: "cli_layer"
    name: "CLI Layer"
    level: 1
    description: "User interface and command handling"
    constraints:
      - "Cannot directly access infrastructure components"
      - "Must delegate business logic to application layer"
      - "Handles only UI concerns and input validation"
    
  - id: "application_layer"
    name: "Application Layer" 
    level: 2
    description: "Command orchestration and workflow management"
    constraints:
      - "Cannot contain business logic"
      - "Orchestrates calls between analysis components"
      - "Handles cross-cutting concerns"
    
  - id: "analysis_layer"
    name: "Analysis Layer"
    level: 3
    description: "Core business logic and analysis engines"
    constraints:
      - "Contains all business logic"
      - "Cannot depend on higher layers"
      - "Must be testable in isolation"
    
  - id: "infrastructure_layer"
    name: "Infrastructure Layer"
    level: 4
    description: "Technical services and external integrations"
    constraints:
      - "Provides stable service interfaces"
      - "Cannot depend on business logic"
      - "Must be mockable for testing"
```

### Component Definitions
```yaml
components:
  # CLI Layer Components
  - id: "main_entry"
    name: "Main Entry Point"
    layer: "cli_layer"
    module_path: "src/main.rs"
    responsibilities:
      - "Command line parsing"
      - "Error reporting to user"
      - "Application initialization"
    interfaces:
      - name: "main"
        type: "function"
        visibility: "public"
    
  - id: "analyze_command"
    name: "Analyze Command"
    layer: "cli_layer"
    module_path: "src/cli/analyze_command.rs"
    responsibilities:
      - "Analysis command handling"
      - "Parameter validation"
      - "Output formatting"
    interfaces:
      - name: "AnalyzeCommand"
        type: "struct"
        visibility: "public"
      - name: "execute"
        type: "method"
        visibility: "public"
    
  # Application Layer Components
  - id: "analysis_orchestrator"
    name: "Analysis Orchestrator"
    layer: "application_layer"
    module_path: "src/analysis/engine.rs"
    responsibilities:
      - "Analysis workflow coordination"
      - "Multi-step process management"
      - "Error handling and recovery"
    interfaces:
      - name: "AnalysisEngine"
        type: "trait"
        visibility: "public"
      - name: "analyze"
        type: "method"
        visibility: "public"
    
  # Analysis Layer Components
  - id: "anti_pattern_detectors"
    name: "Anti-pattern Detectors"
    layer: "analysis_layer"
    module_path: "src/analysis/"
    responsibilities:
      - "God object detection"
      - "Cycle detection"
      - "Dependency analysis"
    interfaces:
      - name: "AnalysisDetector"
        type: "trait"
        visibility: "public"
      - name: "detect"
        type: "method"
        visibility: "public"
    
  # Infrastructure Layer Components
  - id: "ast_engine"
    name: "AST Engine"
    layer: "infrastructure_layer"
    module_path: "src/ast/"
    responsibilities:
      - "Multi-language parsing"
      - "AST node traversal"
      - "Syntax tree caching"
    interfaces:
      - name: "AstParser"
        type: "trait"
        visibility: "public"
      - name: "parse_file"
        type: "method"
        visibility: "public"
    
  - id: "ai_engine"
    name: "AI Engine"
    layer: "infrastructure_layer"
    module_path: "src/ai/"
    responsibilities:
      - "AI provider abstraction"
      - "Context building"
      - "Response parsing"
    interfaces:
      - name: "LlmProvider"
        type: "trait"
        visibility: "public"
      - name: "analyze_code"
        type: "method"
        visibility: "public"
    
  - id: "database_layer"
    name: "Database Layer"
    layer: "infrastructure_layer"
    module_path: "src/database/"
    responsibilities:
      - "Data persistence"
      - "Query abstraction"
      - "Migration management"
    interfaces:
      - name: "AnalysisRepository"
        type: "trait"
        visibility: "public"
      - name: "store_result"
        type: "method"
        visibility: "public"
    
  - id: "plugin_system"
    name: "Plugin System"
    layer: "infrastructure_layer"
    module_path: "src/plugin/"
    responsibilities:
      - "WASM runtime management"
      - "Plugin lifecycle"
      - "API sandboxing"
    interfaces:
      - name: "PluginManager"
        type: "struct"
        visibility: "public"
      - name: "load_plugin"
        type: "method"
        visibility: "public"
```

### Component Relationships
```yaml
component_relationships:
  # CLI → Application Layer
  - from: "analyze_command"
    to: "analysis_orchestrator"
    relationship: "calls"
    interface: "AnalysisEngine::analyze"
    constraint: "async"
    
  # Application → Analysis Layer  
  - from: "analysis_orchestrator"
    to: "anti_pattern_detectors"
    relationship: "uses"
    interface: "AnalysisDetector::detect"
    constraint: "trait object"
    
  # Analysis → Infrastructure Layer
  - from: "anti_pattern_detectors"
    to: "ast_engine"
    relationship: "depends_on"
    interface: "AstParser::parse_file"
    constraint: "async"
    
  - from: "analysis_orchestrator"
    to: "ai_engine"
    relationship: "uses"
    interface: "LlmProvider::analyze_code"
    constraint: "optional"
    
  - from: "analysis_orchestrator"
    to: "database_layer"
    relationship: "stores_to"
    interface: "AnalysisRepository::store_result"
    constraint: "async"
```

## Data Architecture

### Core Data Models
```yaml
data_models:
  - name: "AnalysisRequest"
    description: "Input parameters for analysis"
    fields:
      - name: "target_path"
        type: "PathBuf"
        required: true
      - name: "output_format"
        type: "OutputFormat"
        required: true
      - name: "ai_providers"
        type: "Vec<String>"
        required: false
    
  - name: "AnalysisResult"
    description: "Output of analysis process"
    fields:
      - name: "findings"
        type: "Vec<Finding>"
        required: true
      - name: "metadata"
        type: "AnalysisMetadata"
        required: true
      - name: "performance_metrics"
        type: "PerformanceMetrics"
        required: true
    
  - name: "Finding"
    description: "Individual architectural issue detected"
    fields:
      - name: "detector_name"
        type: "String"
        required: true
      - name: "severity"
        type: "Severity"
        required: true
      - name: "location"
        type: "SourceLocation"
        required: true
      - name: "description"
        type: "String"
        required: true
      - name: "suggestion"
        type: "Option<String>"
        required: false
```

## Error Architecture

### Error Taxonomy
```yaml
error_hierarchy:
  root: "UveddiError"
  categories:
    - name: "InputErrors"
      types:
        - "PathNotFound"
        - "InvalidInputPath"
        - "UnsupportedLanguage"
    
    - name: "ProcessingErrors"
      types:
        - "AstParsing"
        - "Analysis"
        - "AiContext"
        - "AiResponseParsing"
    
    - name: "InfrastructureErrors"
      types:
        - "Database"
        - "Network"
        - "AiApi"
        - "Plugin"
    
    - name: "ConfigurationErrors"
      types:
        - "Configuration"
        - "MissingConfiguration"
        - "Validation"

error_propagation:
  pattern: "Result<T, UveddiError>"
  context_adding: "ErrContext trait"
  recovery_strategy: "fail-fast with context"
```

## Quality Attributes

### Performance Requirements
```yaml
performance:
  response_time:
    small_project: "< 30 seconds"
    medium_project: "< 2 minutes"
    large_project: "< 10 minutes"
  
  memory_usage:
    peak_memory: "< 1GB for typical projects"
    streaming: "constant memory for file processing"
  
  scalability:
    file_count: "up to 10,000 files"
    project_size: "up to 1GB codebase"

reliability:
  error_recovery: "graceful degradation"
  ai_fallback: "multiple provider support"
  data_consistency: "ACID compliance for analysis results"

security:
  plugin_isolation: "WASM sandboxing"
  api_key_storage: "secure configuration management"
  data_privacy: "local-first with optional cloud sync"
```

## Architectural Constraints

### Dependency Rules
```yaml
constraints:
  layer_dependencies:
    - rule: "Higher layers cannot depend on lower layers"
      enforcement: "compile-time"
    
    - rule: "Infrastructure layer cannot depend on business logic"
      enforcement: "architecture tests"
    
    - rule: "All external dependencies through interfaces"
      enforcement: "code review"
  
  error_handling:
    - rule: "All functions return Result<T, UveddiError>"
      enforcement: "clippy lint"
    
    - rule: "No panic! in production code"
      enforcement: "clippy lint"
  
  plugin_system:
    - rule: "Plugins cannot access file system directly"
      enforcement: "WASM runtime"
    
    - rule: "Plugin API is stable and versioned"
      enforcement: "semantic versioning"
```

## Validation Rules

### Architecture Validation
```yaml
validation_rules:
  - name: "Layer Boundary Validation"
    description: "Ensure no upward dependencies"
    check: "dependency_analysis"
    frequency: "on_commit"
  
  - name: "Interface Stability"
    description: "Public interfaces don't break"
    check: "api_compatibility"
    frequency: "on_release"
  
  - name: "Error Propagation"
    description: "All errors go through UveddiError"
    check: "error_type_analysis"
    frequency: "on_build"
  
  - name: "Plugin API Compliance"
    description: "Plugins use only allowed APIs"
    check: "wasm_api_analysis"
    frequency: "on_plugin_load"
```

## Evolution Strategy

### Future Architecture Changes
```yaml
planned_changes:
  - description: "Add streaming analysis for large codebases"
    impact: "infrastructure_layer"
    timeline: "Q3 2025"
  
  - description: "Implement distributed analysis"
    impact: "application_layer"
    timeline: "Q4 2025"
  
  - description: "Add visual diagram generation"
    impact: "report_generation"
    timeline: "Q1 2026"

migration_strategy:
  - phase: "Interface stability"
    description: "Lock down public APIs"
  
  - phase: "Internal refactoring"
    description: "Improve implementation without API changes"
  
  - phase: "Feature addition"
    description: "Add new capabilities through existing interfaces"
```

---

This SAM serves as the authoritative specification for Uveddi's architecture. All architectural decisions, changes, and validations should reference and update this model to maintain consistency and enable automated architectural governance.
