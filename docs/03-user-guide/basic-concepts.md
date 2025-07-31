# Uveddi Basic Concepts

## Architectural Analysis

Uveddi examines codebases for:
- **Cyclic dependencies**: Circular references between modules
- **God objects**: Classes/modules with too many responsibilities
- **Architectural drift**: Deviation from intended design
- **Microservice boundaries**: Improper service isolation

## Analysis Process

1. **System Detection**: Automatically detects system memory and configures optimization
2. **Code Parsing**: Converts source code to AST using tree-sitter
3. **Memory Optimization**: Applies object pooling and arena allocation for performance
4. **Dependency Extraction**: Builds module dependency graph with caching
5. **Pattern Detection**: Identifies architectural issues using optimized algorithms
6. **AI Enhancement**: Provides explanations and fixes (optional)
7. **Report Generation**: Creates actionable output in your preferred format

## Key Components

### Analysis Engine
- Language-agnostic core
- Plugin-based architecture
- Parallel processing
- **Memory optimization by default** with automatic system detection
- Zero-copy AST caching for improved performance

### AI Integration
- Local (Ollama) and cloud providers
- Context-aware suggestions
- Explanation generation

### Reporting System
- Multiple output formats
- Severity classification
- Visual diagrams

## Terminology

| Term | Definition |
|------|------------|
| Module | Logical code unit (file/package) |
| Dependency | Relationship between modules |
| Smell | Architectural anti-pattern |
| Criticality | Issue severity (critical/warning/suggestion) |
