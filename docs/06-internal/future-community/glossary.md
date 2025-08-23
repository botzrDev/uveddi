# Uveddi Ubiquitous Language Glossary

This glossary defines the standardized terminology used throughout Uveddi's codebase, documentation, and team communication. It serves as the authoritative reference for all domain concepts and technical terms.

## Core Analysis Domain

### Analysis Engine
The main orchestrator responsible for coordinating code analysis across multiple detectors and generating comprehensive results. Central component of Uveddi's architecture.

### Anti-Pattern
A recurring problematic solution to common problems that generates negative consequences. Standardized term used consistently across codebase (hyphenated in documentation, `anti_pattern` in code structures, `AntiPattern` in type names).

### Architectural Drift
When implementation gradually diverges from intended architecture, leading to technical debt and maintainability issues.

### Architectural Issue
A specific problem identified in code architecture, classified by criticality level and containing location information and remediation suggestions.

### Detector
A component that identifies specific types of architectural issues or anti-patterns in code. All detector implementations follow the `{Purpose}Detector` naming pattern (e.g., `GodObjectDetector`).

### Analysis Result
The structured output from running detectors on code, containing identified issues, metrics, and recommendations for improvement.

### Criticality Level
Severity classification system with three levels: Critical (must fix), Warning (should fix), and Suggestion (consider fixing).

### Issue Location
Source code location information associated with an architectural issue, including file path, line numbers, and relevant code snippets.

## Anti-Pattern Types

### God Object
A class or module that knows too much or does too much, violating the Single Responsibility Principle by centralizing excessive functionality.

### Cyclic Dependency
When two or more modules depend on each other directly or indirectly, creating circular references that complicate maintenance and testing.

### Tight Coupling
Excessive interdependence between modules that makes code brittle and difficult to modify or test independently.

### Dead Code
Unreachable or unused code segments that increase maintenance burden without providing functional value.

### Long Parameter List
Functions or methods with excessive parameters that are difficult to understand, test, and maintain.

### Feature Envy
When a class or module accesses data or methods from another class more than its own, suggesting misplaced responsibilities.

## AI Integration Domain

### LLM Provider
A service interface for integrating large language models, supporting both local (Ollama) and cloud-based AI providers for code analysis enhancement.

### AI Analysis
AI-enhanced code analysis that provides natural language explanations, contextual insights, and intelligent recommendations beyond traditional static analysis.

### Prompt Template
Structured templates used for AI interactions that ensure consistent and effective communication with language models for analysis tasks.

### Context Builder
Component responsible for constructing relevant context information to send to AI providers, optimizing prompt effectiveness while respecting token limits.

### Smart Prompting
Advanced prompting techniques that leverage few-shot learning, chain-of-thought reasoning, and context optimization for improved AI analysis quality.

### Context Window
The maximum amount of text (measured in tokens) that an AI model can process in a single interaction.

### Temperature
Parameter controlling randomness in AI responses; lower values produce more deterministic outputs, higher values increase creativity.

### Few-Shot Learning
Technique of providing examples within prompts to guide AI behavior and improve response quality for specific analysis tasks.

### Embedding
Numerical vector representation of code or text that captures semantic meaning for similarity comparisons and semantic search.

## Plugin System Domain

### WASM Plugin
WebAssembly module that extends Uveddi's analysis capabilities, providing sandboxed execution environment for third-party detectors and analyzers.

### Plugin Registry
Central repository and discovery system for managing available plugins, handling installation, updates, and dependency resolution.

### Plugin Manifest
Metadata descriptor containing plugin information including capabilities, dependencies, security requirements, and configuration options.

### Plugin Lifecycle Manager
Component responsible for plugin installation, initialization, execution, and cleanup throughout the plugin's operational lifetime.

### Security Policy
Access control rules and restrictions governing plugin execution, file system access, and network permissions to ensure system security.

### Plugin Verifier
Component that validates plugin signatures, checks security compliance, and ensures plugins meet quality and safety standards before execution.

## Security & RBAC Domain

### RBAC (Role-Based Access Control)
Security framework that assigns permissions to roles rather than individual users, simplifying access management and improving security governance.

### User Role
Defined set of permissions and capabilities assigned to users, controlling access to features, data, and system operations.

### Authentication Service
Component responsible for verifying user identity through various mechanisms including OAuth2, JWT tokens, and multi-factor authentication.

### Authorization Service
Component that determines whether authenticated users have permission to perform specific actions based on their assigned roles and security policies.

### Security Policy
Rules and constraints governing system access, data handling, and operation permissions to ensure compliance and protect sensitive information.

### Audit Logger
Component that creates immutable records of security-relevant events for compliance monitoring, forensic analysis, and security incident investigation.

### Audit Trail
Immutable chronological record of system activities and security events that supports compliance requirements and incident investigation.

### Compliance Validator
Component that ensures operations and configurations adhere to regulatory requirements and organizational security standards.

## Configuration Domain

### Configuration
System settings and parameters that control Uveddi's behavior. Use full term "configuration" in documentation and user interfaces, `Config` suffix in code structures.

### Configuration Service
Component responsible for loading, validating, and managing system configuration from various sources including files, environment variables, and runtime parameters.

### Configuration Schema
Structured definition of valid configuration parameters, their types, constraints, and default values used for validation and documentation.

## Monitoring & Observability Domain

### Telemetry
Automated collection and transmission of performance metrics, usage statistics, and system health data for monitoring and analysis.

### Performance Metrics
Quantitative measurements of system performance including execution time, memory usage, throughput, and resource utilization.

### Health Check
Diagnostic endpoint or procedure that verifies system component functionality and reports operational status.

### Circuit Breaker
Resilience pattern that prevents cascading failures by temporarily disabling calls to failing services or components.

### Rate Limiting
Mechanism to control the frequency of operations or requests to prevent system overload and ensure fair resource usage.

## Technical Architecture Terms

### AST (Abstract Syntax Tree)
Tree representation of source code structure created by parsing, providing the foundation for code analysis and transformation operations.

### Dependency Graph
Visual or data representation showing relationships and dependencies between code modules, enabling architectural analysis and impact assessment.

### Tree-sitter
Multi-language parsing library used by Uveddi for creating accurate ASTs across different programming languages with incremental parsing capabilities.

### Incremental Analysis
Analysis approach that processes only changed code sections rather than entire codebases, improving performance for large projects and CI/CD integration.

### Cache Manager
Component responsible for managing cached analysis results, AST data, and computed metrics to improve performance across analysis runs.

### Memory Optimization
Techniques and components for efficient memory usage including arena allocation, zero-copy serialization, and memory-mapped file handling.

## Reporting Domain

### Analysis Report
Comprehensive document containing analysis results, identified issues, metrics, and recommendations, available in multiple formats (Markdown, JSON, HTML).

### Report Generator
Component responsible for transforming analysis results into formatted reports with customizable templates and output formats.

### Visualization
Graphical representation of analysis results including dependency graphs, complexity charts, and issue distribution diagrams.

### Report Template
Predefined format structure for generating consistent analysis reports with customizable sections and styling.

## User Interface Domain

### TUI (Terminal User Interface)
Interactive text-based interface providing rich user experience within terminal environments using cursor positioning and text formatting.

### CLI (Command Line Interface)
Text-based interface accepting commands and arguments for programmatic interaction and automation support.

### Interactive Mode
TUI operational mode allowing real-time navigation, configuration, and analysis execution with immediate visual feedback.

### Batch Mode
Non-interactive operational mode optimized for automation, CI/CD integration, and scripting environments.

## Development & Quality Terms

### Code Coverage
Metric indicating the percentage of code exercised by tests, used for assessing test suite completeness and quality assurance.

### Regression Testing
Testing approach that verifies new changes don't break existing functionality, essential for maintaining system stability.

### Performance Benchmarking
Systematic measurement and comparison of system performance across different configurations, versions, or implementations.

### Static Analysis
Code examination technique that analyzes source code without execution to identify potential issues, vulnerabilities, and quality problems.

### Dynamic Analysis
Code examination that occurs during program execution, providing runtime behavior insights and performance characteristics.

## Data & Storage Terms

### Vector Database
Specialized database optimized for storing and querying high-dimensional vector embeddings used in semantic search and AI operations.

### Semantic Search
Search technique based on meaning and context rather than exact text matches, leveraging vector embeddings for improved relevance.

### Serialization
Process of converting data structures into format suitable for storage or transmission, with zero-copy techniques used for performance optimization.

### Persistence Layer
Components responsible for data storage and retrieval operations, including databases, file systems, and caching mechanisms.

## Naming Conventions

### Code Structure Naming
- **Structs/Enums:** PascalCase (`AnalysisEngine`, `DetectorConfig`)
- **Traits:** PascalCase with descriptive names (`AnalysisDetector`, `LlmProvider`)
- **Functions:** snake_case (`detect_issues`, `get_detector_names`)
- **Constants:** SCREAMING_SNAKE_CASE
- **Module names:** snake_case

### Architectural Component Patterns
- **Detectors:** `{Purpose}Detector` (e.g., `GodObjectDetector`)
- **Services:** `{Purpose}Service` (e.g., `ConfigurationService`)
- **Managers:** `{Purpose}Manager` (e.g., `CacheManager`)
- **Engines:** `{Purpose}Engine` (e.g., `AnalysisEngine`)
- **Builders:** `{Purpose}Builder` (e.g., `AnalysisEngineBuilder`)
- **Providers:** `{Purpose}Provider` (e.g., `OllamaProvider`)

### Terminology Usage Guidelines

#### Component Type Distinctions
- **Engine:** Core orchestrators and main processing components
- **Service:** Internal business logic and domain services
- **Provider:** External service integrations and adapters
- **Manager:** Resource and lifecycle management components
- **Builder:** Object construction and configuration components

#### Spelling Standardization
- **Analysis/Analyze:** Use US spelling consistently (`analyze` for verb, `analysis` for noun)
- **Configuration:** Use full term in documentation, `Config` suffix in code
- **Anti-pattern:** Hyphenated in documentation, `anti_pattern` in code structures, `AntiPattern` in types

## Common Abbreviations

| Abbreviation | Full Term | Usage Context |
|--------------|-----------|---------------|
| AST | Abstract Syntax Tree | Code parsing and analysis |
| API | Application Programming Interface | System interfaces |
| CI/CD | Continuous Integration/Continuous Deployment | Development workflow |
| WASM | WebAssembly | Plugin system |
| CLI | Command Line Interface | User interaction |
| TUI | Terminal User Interface | Interactive terminal interface |
| RBAC | Role-Based Access Control | Security framework |
| LLM | Large Language Model | AI integration |
| JSON | JavaScript Object Notation | Data format |
| TOML | Tom's Obvious Minimal Language | Configuration format |
| HTTP | HyperText Transfer Protocol | Network communication |
| REST | Representational State Transfer | API architecture |
| JWT | JSON Web Token | Authentication |
| OAuth | Open Authorization | Authorization framework |
| SQL | Structured Query Language | Database queries |
| AI | Artificial Intelligence | Machine learning integration |
| ML | Machine Learning | AI subset |
| NLP | Natural Language Processing | Text analysis |
| RAM | Random Access Memory | System memory |
| CPU | Central Processing Unit | System processor |
| I/O | Input/Output | System operations |

---

**Note:** This glossary is a living document that should be updated as new concepts are introduced or existing terminology evolves. All team members are responsible for adhering to these standardized terms in code, documentation, and communication.
