# Uveddi Basic Concepts

## Architectural Analysis

Uveddi examines codebases for:
- **Cyclic dependencies**: Circular references between modules
- **God objects**: Classes/modules with too many responsibilities
- **Architectural drift**: Deviation from intended design
- **Microservice boundaries**: Improper service isolation

## Analysis Process

1. **Code Parsing**: Converts source code to AST
2. **Dependency Extraction**: Builds module dependency graph
3. **Pattern Detection**: Identifies architectural issues
4. **AI Enhancement**: Provides explanations and fixes
5. **Report Generation**: Creates actionable output

## Key Components

### Analysis Engine
- Language-agnostic core
- Plugin-based architecture
- Parallel processing

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
