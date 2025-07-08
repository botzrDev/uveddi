# Leaky Abstraction Detector Research Prompts

This document contains a series of focused GPT research prompts for implementing the Leaky Abstraction Detector, the 2nd most complex detector in the Uveddi analysis engine. Each prompt is designed to gather specific technical knowledge needed for implementation.

## Table of Contents

1. [Abstraction Boundary Detection](#1-abstraction-boundary-detection)
2. [Type Flow Analysis](#2-type-flow-analysis)
3. [Language-Specific Patterns](#3-language-specific-patterns)
4. [Cross-File Semantic Analysis](#4-cross-file-semantic-analysis)
5. [Detection Algorithms](#5-detection-algorithms)
6. [Performance and Scalability](#6-performance-and-scalability)
7. [Configuration and Thresholds](#7-configuration-and-thresholds)
8. [Implementation Architecture](#8-implementation-architecture)

---

## 1. Abstraction Boundary Detection

### Prompt 1.1: Module Boundary Identification
```
I'm building a static analysis tool that needs to identify abstraction boundaries in multi-language codebases (Rust, Python, JavaScript). I need to understand:

1. How to programmatically identify module/package boundaries in each language:
   - Rust: mod.rs files, pub visibility, crate boundaries
   - Python: __init__.py files, package structure, import patterns
   - JavaScript: ES6 modules, CommonJS, export patterns

2. What constitutes an "abstraction layer" in software architecture:
   - Presentation layer, business logic, data access, infrastructure
   - How these map to actual code organization patterns
   - Common architectural patterns (MVC, Clean Architecture, Hexagonal)

3. How to detect dependency direction violations:
   - What constitutes "upward" vs "downward" dependencies
   - Algorithms to detect circular dependencies between layers
   - How to handle legitimate cross-cutting concerns

Please provide concrete examples of code patterns that represent proper vs. improper abstraction boundaries, and suggest algorithms for automated detection using AST analysis.
```

### Prompt 1.2: Architectural Pattern Recognition
```
I need to automatically recognize common software architectural patterns in codebases to detect abstraction boundary violations. Help me understand:

1. How to identify these architectural patterns from code structure:
   - Layered Architecture (N-tier)
   - Clean Architecture (Uncle Bob)
   - Hexagonal Architecture (Ports and Adapters)
   - MVC/MVP/MVVM patterns
   - Domain-Driven Design boundaries

2. What directory structures and naming conventions indicate these patterns:
   - Common folder naming (controllers, services, repositories, models)
   - File organization patterns
   - Interface/implementation separation indicators

3. How to build heuristics that can classify code into architectural layers:
   - Based on import/dependency patterns
   - Based on naming conventions
   - Based on interface definitions
   - Based on data flow patterns

4. Edge cases and challenges:
   - Microservices vs monoliths
   - Framework-imposed structures
   - Legacy code with mixed patterns

Provide specific examples of how these patterns manifest in Rust, Python, and JavaScript codebases, with focus on automated detection strategies.
```

---

## 2. Type Flow Analysis

### Prompt 2.1: Cross-Module Type Tracking
```
I'm implementing a leaky abstraction detector that needs to track how types flow across module boundaries. I need detailed guidance on:

1. How to perform type flow analysis using AST parsing:
   - Tracking function return types across module boundaries
   - Identifying when internal types leak through public APIs
   - Handling generic types and type parameters
   - Dealing with type aliases and wrappers

2. Language-specific type system challenges:
   - Rust: Ownership, borrowing, trait objects vs concrete types
   - Python: Dynamic typing, type hints, duck typing
   - JavaScript: Prototype-based inheritance, TypeScript integration

3. Algorithms for building type dependency graphs:
   - How to represent type relationships
   - Efficient data structures for large codebases
   - Handling recursive and circular type dependencies

4. Detecting problematic type leakage patterns:
   - Database entities in API responses
   - Framework-specific types in business logic
   - Implementation details in public interfaces

Please provide concrete examples of leaky vs. proper type usage, and suggest algorithms for automated detection using Tree-sitter AST analysis.
```

### Prompt 2.2: Exception and Error Propagation Analysis
```
I need to detect when low-level implementation details leak through exception/error handling. Help me understand:

1. How to track exception propagation across abstraction boundaries:
   - When database exceptions should be caught and wrapped
   - When network errors should be abstracted
   - When file system errors should be hidden

2. Language-specific error handling patterns:
   - Rust: Result types, error trait implementations, error chains
   - Python: Exception hierarchy, custom exceptions, exception chaining
   - JavaScript: Error objects, Promise rejections, async/await error handling

3. Algorithms to detect problematic error propagation:
   - Tracking exception types through call chains
   - Identifying when specific implementation errors leak upward
   - Detecting missing error abstraction layers

4. Best practices for proper error abstraction:
   - When to wrap vs. when to propagate
   - How to maintain error context while abstracting details
   - Domain-specific error handling patterns

Provide examples of good vs. bad error handling patterns and suggest automated detection strategies.
```

---

## 3. Language-Specific Patterns

### Prompt 3.1: Rust Leaky Abstraction Patterns
```
I'm building a leaky abstraction detector for Rust code. I need comprehensive understanding of:

1. Rust-specific abstraction mechanisms and their potential leaks:
   - pub visibility and module boundaries
   - Trait objects vs concrete type exposure
   - Ownership and borrowing in public APIs
   - Error type propagation (Result<T, SpecificError> vs Result<T, Box<dyn Error>>)

2. Common Rust leaky abstraction anti-patterns:
   - Exposing internal struct fields instead of methods
   - Returning concrete types instead of traits
   - Leaking lifetime parameters in public APIs
   - Exposing implementation-specific error types

3. Rust ecosystem-specific patterns:
   - Serde serialization leaking internal structure
   - Database ORM types in business logic
   - Web framework types in domain models
   - Async runtime details in interfaces

4. How to detect these patterns using Tree-sitter queries:
   - AST patterns for visibility violations
   - Type signature analysis
   - Import/use statement analysis
   - Trait bound analysis

Please provide specific Rust code examples of leaky vs. proper abstractions, and suggest Tree-sitter query patterns for automated detection.
```

### Prompt 3.2: Python Leaky Abstraction Patterns
```
I need to detect leaky abstractions in Python codebases. Help me understand:

1. Python-specific abstraction challenges:
   - Dynamic typing and duck typing implications
   - Import system and module visibility
   - Class inheritance and composition patterns
   - Decorator and metaclass abstraction leaks

2. Common Python leaky abstraction patterns:
   - Direct database model usage in views/controllers
   - Framework-specific objects in business logic
   - File system paths in public interfaces
   - Third-party library types in domain models

3. Python ecosystem anti-patterns:
   - Django models in templates
   - SQLAlchemy sessions in business logic
   - Flask request objects in service layers
   - Pandas DataFrames in API responses

4. Detection strategies for Python:
   - Import analysis for layer violations
   - Type hint analysis (when available)
   - Naming convention analysis
   - AST pattern matching for common violations

5. Handling Python's dynamic nature:
   - Duck typing vs. proper interfaces
   - Runtime type checking implications
   - Monkey patching detection

Provide concrete Python examples and Tree-sitter AST analysis strategies for automated detection.
```

### Prompt 3.3: JavaScript Leaky Abstraction Patterns
```
I'm implementing leaky abstraction detection for JavaScript/TypeScript. I need detailed knowledge of:

1. JavaScript module system abstraction patterns:
   - ES6 modules vs CommonJS
   - Export patterns and interface design
   - Namespace pollution and global leaks
   - Module bundler implications

2. Common JavaScript/TypeScript leaky patterns:
   - DOM manipulation in business logic
   - Framework-specific components in data models
   - HTTP request objects in domain logic
   - Database connection details in UI components

3. Frontend-specific abstraction violations:
   - React component internals in business logic
   - Browser APIs in reusable modules
   - CSS-in-JS leaking styling concerns
   - State management library details in components

4. Node.js backend patterns:
   - Express request/response objects in business logic
   - Database driver specifics in service layers
   - File system operations in domain models
   - Third-party API client details in interfaces

5. TypeScript-specific considerations:
   - Type definition leaks
   - Interface vs implementation exposure
   - Generic type parameter pollution
   - Utility type misuse

Provide specific JavaScript/TypeScript examples and suggest AST analysis techniques for automated detection.
```

---

## 4. Cross-File Semantic Analysis

### Prompt 4.1: Building Semantic Dependency Graphs
```
I need to implement cross-file semantic analysis for detecting leaky abstractions. Help me understand:

1. How to build semantic dependency graphs from AST analysis:
   - Representing modules, types, and functions as graph nodes
   - Modeling different types of dependencies (imports, type usage, inheritance)
   - Handling circular dependencies and complex relationships
   - Efficient graph data structures for large codebases

2. Algorithms for cross-file analysis:
   - Topological sorting for dependency analysis
   - Strongly connected component detection
   - Path analysis for abstraction violation detection
   - Graph traversal strategies for type flow analysis

3. Incremental analysis strategies:
   - Detecting which files need re-analysis after changes
   - Caching semantic information efficiently
   - Handling partial analysis results
   - Maintaining graph consistency during updates

4. Memory and performance considerations:
   - Lazy loading of semantic information
   - Garbage collection of unused analysis data
   - Parallel processing strategies
   - Disk-based caching for large projects

5. Integration with Tree-sitter:
   - Extracting semantic information from ASTs
   - Handling multiple language parsers
   - Symbol resolution across files
   - Managing parser state and memory

Please provide algorithms, data structures, and implementation strategies for efficient cross-file semantic analysis.
```

<<<### Prompt 4.2: Symbol Resolution and Scope Analysis <><>
```
I need to implement symbol resolution across multiple files for leaky abstraction detection. Help me understand:

1. Symbol resolution algorithms:
   - Building symbol tables from AST analysis
   - Resolving imports and references across files
   - Handling scope chains and lexical scoping
   - Managing symbol visibility and accessibility

2. Language-specific symbol resolution:
   - Rust: Module system, use statements, visibility rules
   - Python: Import system, namespace resolution, scope rules
   - JavaScript: Module imports, hoisting, closure scoping

3. Cross-reference analysis:
   - Tracking where symbols are defined vs. used
   - Identifying abstraction boundary crossings
   - Detecting inappropriate symbol access
   - Building usage dependency graphs

4. Handling complex cases:
   - Dynamic imports and runtime symbol resolution
   - Macro expansion and code generation
   - Conditional compilation and feature flags
   - Generic/template instantiation

5. Performance optimization:
   - Efficient symbol lookup data structures
   - Caching resolved symbols
   - Incremental symbol table updates
   - Memory-efficient representation

Provide specific algorithms and data structures for implementing robust symbol resolution in a multi-language static analysis tool.
```

---

## 5. Detection Algorithms

### Prompt 5.1: Layer Violation Detection Algorithms
```
I need algorithms to detect architectural layer violations in codebases. Help me design:

1. Layer classification algorithms:
   - How to automatically classify code into architectural layers
   - Heuristics based on directory structure, naming, and imports
   - Machine learning approaches for layer detection
   - Handling ambiguous or mixed-layer code

2. Dependency direction analysis:
   - Algorithms to detect "upward" dependencies (violations)
   - Handling legitimate cross-cutting concerns
   - Detecting circular dependencies between layers
   - Measuring dependency strength and coupling

3. Violation scoring and ranking:
   - How to quantify the severity of layer violations
   - Weighting factors for different types of violations
   - Confidence scoring for detected violations
   - Prioritization algorithms for large codebases

4. Configuration and customization:
   - Allowing custom layer definitions
   - Configurable dependency rules
   - Whitelist/blacklist patterns
   - Domain-specific architectural rules

5. Edge case handling:
   - Framework-imposed violations
   - Performance optimization exceptions
   - Legacy code integration
   - Microservice boundary considerations

Please provide specific algorithms, pseudocode, and implementation strategies for robust layer violation detection.
```

### Prompt 5.2: Type Leakage Detection Algorithms
```
I need algorithms to detect when internal types leak through abstraction boundaries. Help me design:

1. Type leakage detection strategies:
   - Identifying "internal" vs "public" types
   - Tracking type usage across module boundaries
   - Detecting when implementation types appear in interfaces
   - Handling type aliases and wrapper types

2. Interface analysis algorithms:
   - Analyzing function signatures for type leaks
   - Detecting return type violations
   - Parameter type analysis
   - Generic type parameter leakage

3. Data structure analysis:
   - Detecting when internal data structures are exposed
   - Analyzing serialization boundaries
   - Database entity leakage detection
   - API response type analysis

4. Inheritance and composition analysis:
   - Detecting inappropriate inheritance relationships
   - Composition vs aggregation violations
   - Interface segregation principle violations
   - Liskov substitution principle violations

5. Scoring and confidence metrics:
   - How to score type leakage severity
   - Confidence levels for different detection methods
   - False positive reduction strategies
   - Context-aware analysis

Provide concrete algorithms and implementation approaches for detecting type leakage across different programming languages.
```

---

## 6. Performance and Scalability

### Prompt 6.1: Scalable Analysis Architecture
```
I'm building a leaky abstraction detector that needs to scale to large codebases (100k+ lines). Help me design:

1. Scalable analysis architecture:
   - Incremental analysis strategies
   - Parallel processing approaches
   - Memory-efficient data structures
   - Disk-based caching systems

2. Performance optimization techniques:
   - AST parsing optimization
   - Symbol table caching
   - Dependency graph pruning
   - Lazy evaluation strategies

3. Memory management:
   - Efficient representation of large dependency graphs
   - Garbage collection of unused analysis data
   - Memory-mapped file approaches
   - Streaming analysis for very large files

4. Caching strategies:
   - What analysis results to cache
   - Cache invalidation strategies
   - Distributed caching for team environments
   - Persistent cache storage formats

5. Benchmarking and profiling:
   - Key performance metrics to track
   - Profiling strategies for analysis bottlenecks
   - Benchmark suite design
   - Performance regression detection

Please provide specific architectural patterns, algorithms, and implementation strategies for building a high-performance static analysis tool.
```

### Prompt 6.2: Incremental Analysis Implementation
```
I need to implement incremental analysis for a leaky abstraction detector to handle large, frequently-changing codebases. Help me understand:

1. Change detection strategies:
   - File-level change detection
   - AST-level change detection
   - Semantic change impact analysis
   - Dependency change propagation

2. Incremental update algorithms:
   - Which analysis results can be reused
   - How to update dependency graphs incrementally
   - Symbol table incremental updates
   - Type information cache updates

3. Dependency tracking:
   - Tracking analysis dependencies between files
   - Invalidation cascading strategies
   - Minimizing re-analysis scope
   - Handling circular dependencies in updates

4. State management:
   - Persistent analysis state storage
   - State consistency during updates
   - Rollback strategies for failed updates
   - Concurrent access to analysis state

5. Integration with development workflows:
   - IDE integration considerations
   - CI/CD pipeline optimization
   - Watch mode implementation
   - Real-time analysis feedback

Provide specific algorithms and data structures for implementing efficient incremental analysis in a multi-language static analysis tool.
```

---

## 7. Configuration and Thresholds

### Prompt 7.1: Configurable Detection Rules
```
I need to design a flexible configuration system for a leaky abstraction detector. Help me understand:

1. Configuration schema design:
   - How to define architectural layers and rules
   - Configurable threshold parameters
   - Language-specific configuration options
   - Project-specific customization patterns

2. Rule definition languages:
   - DSL design for abstraction rules
   - YAML/JSON configuration formats
   - Code-based configuration approaches
   - Rule composition and inheritance

3. Threshold tuning strategies:
   - How to determine optimal thresholds
   - Machine learning approaches for threshold optimization
   - A/B testing for configuration validation
   - User feedback integration

4. Domain-specific configurations:
   - Web application patterns
   - Library/framework development
   - Microservice architectures
   - Legacy system integration

5. Configuration validation:
   - Validating rule consistency
   - Detecting conflicting configurations
   - Configuration testing strategies
   - Migration between configuration versions

Please provide examples of flexible configuration systems and suggest implementation approaches for a static analysis tool.
```

### Prompt 7.2: Adaptive Threshold Systems
```
I need to implement adaptive thresholds for leaky abstraction detection that can learn from codebase patterns. Help me design:

1. Adaptive threshold algorithms:
   - How to learn optimal thresholds from codebase analysis
   - Statistical approaches for threshold determination
   - Machine learning models for threshold optimization
   - Feedback loop integration

2. Codebase pattern analysis:
   - Extracting architectural patterns from existing code
   - Identifying project-specific conventions
   - Learning from developer feedback
   - Historical analysis for threshold evolution

3. False positive reduction:
   - Strategies for minimizing false positives
   - Context-aware threshold adjustment
   - Whitelist pattern learning
   - Developer annotation integration

4. Threshold validation:
   - Cross-validation approaches
   - A/B testing for threshold changes
   - Performance impact measurement
   - User satisfaction metrics

5. Implementation considerations:
   - Online vs offline learning
   - Computational overhead
   - Storage requirements
   - Update frequency strategies

Provide specific algorithms and implementation strategies for building adaptive threshold systems in static analysis tools.
```

---

## 8. Implementation Architecture

### Prompt 8.1: Detector Architecture Design
```
I'm implementing a leaky abstraction detector as part of a larger static analysis framework. Help me design:

1. Detector architecture patterns:
   - Plugin-based vs monolithic design
   - Visitor pattern for AST traversal
   - Strategy pattern for different detection algorithms
   - Observer pattern for result collection

2. Integration with existing systems:
   - Tree-sitter AST parser integration
   - Database storage for results
   - Caching layer integration
   - Reporting system integration

3. Extensibility considerations:
   - Adding new detection patterns
   - Supporting additional languages
   - Custom rule integration
   - Third-party plugin support

4. Error handling and robustness:
   - Graceful degradation strategies
   - Error recovery mechanisms
   - Logging and debugging support
   - Performance monitoring

5. Testing strategies:
   - Unit testing approaches
   - Integration testing patterns
   - Performance testing frameworks
   - Regression testing automation

Please provide architectural patterns, design principles, and implementation strategies for building a robust, extensible leaky abstraction detector.
```

### Prompt 8.2: Multi-Language Analysis Framework
```
I need to design a framework for multi-language leaky abstraction detection (Rust, Python, JavaScript). Help me understand:

1. Language abstraction strategies:
   - Common interfaces for different languages
   - Language-specific adapter patterns
   - Unified AST representation approaches
   - Cross-language analysis coordination

2. Parser integration:
   - Tree-sitter parser management
   - Language grammar handling
   - AST normalization strategies
   - Error handling across parsers

3. Semantic model unification:
   - Common semantic concepts across languages
   - Type system abstraction
   - Symbol resolution unification
   - Dependency model standardization

4. Analysis coordination:
   - Cross-language dependency analysis
   - Mixed-language project handling
   - Language boundary detection
   - Polyglot architecture analysis

5. Performance considerations:
   - Language-specific optimization
   - Parser switching overhead
   - Memory management across languages
   - Parallel analysis strategies

Provide architectural patterns and implementation strategies for building a robust multi-language static analysis framework.
```

---

## Usage Instructions

1. **Sequential Processing**: Use these prompts in order, as later prompts build on concepts from earlier ones.

2. **Customization**: Adapt prompts based on your specific implementation needs and constraints.

3. **Iteration**: Use follow-up prompts to dive deeper into specific areas that need more detail.

4. **Validation**: Cross-reference information from multiple prompts to ensure consistency.

5. **Implementation Planning**: Use the research results to create detailed implementation plans and milestones.

## Expected Outcomes

After processing these prompts, you should have:

- Comprehensive understanding of leaky abstraction patterns
- Concrete algorithms for detection
- Implementation architecture design
- Performance optimization strategies
- Configuration and customization approaches
- Language-specific detection patterns
- Testing and validation strategies

This research will provide the foundation for implementing a robust, scalable leaky abstraction detector that can handle real-world codebases effectively.