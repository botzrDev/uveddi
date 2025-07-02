# Master Anti-Patterns List for Code Analysis

## Overview

This document provides a comprehensive catalog of anti-patterns and code quality issues that affect software development across multiple programming languages. It serves as the foundation for developing detection modules in the Uveddi static analysis tool.

The list is organized into two main sections:
1. **Universal Anti-Patterns** - Issues that affect all programming languages
2. **Language-Specific Anti-Patterns** - Issues unique to particular languages

---

## Universal Anti-Patterns

These anti-patterns transcend language boundaries and represent fundamental software engineering problems that manifest across all programming paradigms.

### 1. Architectural & Design Issues

#### 1.1 God Object / Monolithic Classes
- **Description**: Classes that accumulate too many responsibilities, violating Single Responsibility Principle
- **Symptoms**: Excessive line count, multiple unrelated methods, high coupling
- **Detection**: Class size metrics, responsibility analysis, coupling measurements

#### 1.2 Tight Coupling
- **Description**: Components with excessive dependencies on each other's internal implementation
- **Symptoms**: Changes cascade across multiple modules, difficult to test in isolation
- **Detection**: Dependency analysis, fan-in/fan-out metrics, interface analysis

#### 1.3 Leaky Abstraction
- **Description**: Higher-level modules exposing lower-level implementation details
- **Symptoms**: Internal types exposed in public APIs, framework-specific exceptions bubbling up
- **Detection**: Type flow analysis, exception propagation tracking

#### 1.4 Modularity Violations
- **Description**: Structurally independent components that exhibit hidden dependencies
- **Symptoms**: Modules changing together despite no apparent relationship
- **Detection**: Community detection algorithms, coupling anomaly analysis

#### 1.5 Insufficient Access Control
- **Description**: Security-sensitive operations accessible without proper authorization
- **Symptoms**: Direct object references, missing permission checks
- **Detection**: Data flow analysis, security annotation tracking

### 2. State Management Issues

#### 2.1 Global State Pollution
- **Description**: Overuse of global variables or shared mutable state
- **Symptoms**: Hidden dependencies, testing difficulties, concurrency issues
- **Detection**: Global variable usage analysis, state mutation tracking

#### 2.2 Mutable Default Arguments / Shared Mutable State
- **Description**: Unexpected state persistence between function calls or object instances
- **Symptoms**: Side effects, non-deterministic behavior
- **Detection**: Default parameter analysis, mutability tracking

#### 2.3 State Synchronization Issues
- **Description**: Multiple copies of data becoming desynchronized
- **Symptoms**: Inconsistent application state, data integrity problems
- **Detection**: Data flow analysis, mutation point tracking

### 3. Resource Management

#### 3.1 Resource Leaks
- **Description**: Failure to properly release system resources (files, connections, memory)
- **Symptoms**: Memory leaks, file handle exhaustion, connection pool depletion
- **Detection**: Resource acquisition/release pattern analysis

#### 3.2 Premature Optimization
- **Description**: Adding complexity without evidence of performance benefits
- **Symptoms**: Overly complex code, micro-optimizations, speculative features
- **Detection**: Complexity metrics, performance impact analysis

### 4. Error Handling Anti-Patterns

#### 4.1 Silent Failures / Exception Swallowing
- **Description**: Errors caught and ignored without proper handling or logging
- **Symptoms**: Difficult debugging, hidden failures, system instability
- **Detection**: Empty catch blocks, generic exception handling

#### 4.2 Error Information Loss
- **Description**: Original error context lost during exception handling or propagation
- **Symptoms**: Stack trace truncation, generic error messages
- **Detection**: Exception handling flow analysis, stack trace preservation

#### 4.3 Inappropriate Exception Types
- **Description**: Using wrong exception types or catching overly broad exceptions
- **Symptoms**: Misleading error messages, inappropriate error handling
- **Detection**: Exception type analysis, catch clause specificity

### 5. Code Organization & Maintainability

#### 5.1 Magic Numbers and Strings
- **Description**: Hardcoded values without explanation or centralization
- **Symptoms**: Difficult maintenance, unclear intent, duplication
- **Detection**: Literal value analysis, duplication detection

#### 5.2 Inconsistent Naming Conventions
- **Description**: Lack of consistent naming patterns across codebase
- **Symptoms**: Reduced readability, confusion about purpose
- **Detection**: Naming pattern analysis, convention compliance checking

#### 5.3 Excessive Complexity
- **Description**: Functions or classes that are too complex to understand or maintain
- **Symptoms**: High cyclomatic complexity, deep nesting, long parameter lists
- **Detection**: Complexity metrics, nesting depth analysis

#### 5.4 Code Duplication
- **Description**: Repeated code blocks that should be extracted into reusable components
- **Symptoms**: Maintenance burden, inconsistent changes, bug propagation
- **Detection**: Clone detection algorithms, similarity analysis

### 6. Performance Anti-Patterns

#### 6.1 Inefficient Algorithms
- **Description**: Using algorithms with poor time or space complexity
- **Symptoms**: Performance bottlenecks, scalability issues
- **Detection**: Algorithm complexity analysis, performance pattern recognition

#### 6.2 Unnecessary Computations
- **Description**: Performing redundant calculations or processing
- **Symptoms**: CPU waste, slow response times
- **Detection**: Computation flow analysis, redundancy detection

---

## Language-Specific Anti-Patterns

### JavaScript Anti-Patterns

#### JS.1 Scope and Context Issues
- **JS.1.1 Global Namespace Pollution**: Variables and functions declared globally without encapsulation
- **JS.1.2 Var Hoisting Confusion**: Using `var` instead of `let`/`const`, leading to unexpected scoping
- **JS.1.3 This Context Loss**: Arrow functions vs regular functions context binding issues

#### JS.2 Type Coercion Problems
- **JS.2.1 Loose Equality Usage**: Using `==` instead of `===`, causing unexpected type coercion
- **JS.2.2 Implicit Type Conversion**: Relying on JavaScript's automatic type conversion
- **JS.2.3 Truthy/Falsy Confusion**: Misunderstanding JavaScript's truthiness evaluation

#### JS.3 Asynchronous Code Issues
- **JS.3.1 Callback Hell**: Deeply nested callbacks creating pyramid of doom
- **JS.3.2 Promise Anti-patterns**: Unnecessary Promise wrapping, not returning promises
- **JS.3.3 Async/Await Misuse**: Not properly handling errors in async functions

#### JS.4 DOM Manipulation Problems
- **JS.4.1 Inefficient DOM Updates**: Multiple DOM manipulations causing reflows/repaints
- **JS.4.2 Memory Leaks**: Event listeners not properly removed, circular references
- **JS.4.3 Blocking Operations**: Synchronous operations blocking the event loop

#### JS.5 Module and Dependency Issues
- **JS.5.1 Circular Dependencies**: Modules depending on each other cyclically
- **JS.5.2 Global Dependency**: Not using proper module systems (ES6 modules, CommonJS)

### Python Anti-Patterns

#### PY.1 Data Structure Misuse
- **PY.1.1 Mutable Default Arguments**: Using mutable objects as default function parameters
- **PY.1.2 List Comprehension Abuse**: Overly complex list comprehensions reducing readability
- **PY.1.3 Dictionary Key Anti-patterns**: Not checking key existence, inappropriate key types

#### PY.2 Object-Oriented Issues
- **PY.2.1 Inappropriate Inheritance**: Using inheritance for code reuse instead of composition
- **PY.2.2 Missing `__init__`**: Not properly initializing object attributes
- **PY.2.3 Monkey Patching**: Modifying built-in types or third-party libraries at runtime

#### PY.3 Exception Handling Problems
- **PY.3.1 Bare Except Clauses**: Using `except:` without specifying exception types
- **PY.3.2 Exception for Control Flow**: Using exceptions for normal program flow
- **PY.3.3 Not Using Context Managers**: Manual resource management instead of `with` statements

#### PY.4 Performance Issues
- **PY.4.1 String Concatenation**: Using `+` for repeated string concatenation
- **PY.4.2 Global Lookups**: Excessive global variable access in loops
- **PY.4.3 Late Binding Closures**: Unexpected behavior in lambda functions and closures

#### PY.5 Import and Module Issues
- **PY.5.1 Star Imports**: Using `from module import *`
- **PY.5.2 Circular Imports**: Modules importing each other causing initialization issues
- **PY.5.3 Import Placement**: Imports not at module level causing performance issues

### Java Anti-Patterns

#### JV.1 Object-Oriented Design Issues
- **JV.1.1 Inheritance for Code Reuse**: Using inheritance instead of composition
- **JV.1.2 Fragile Base Class**: Changes to parent class breaking subclasses
- **JV.1.3 Interface Pollution**: Overly large interfaces violating Interface Segregation Principle

#### JV.2 Resource Management Problems
- **JV.2.1 Resource Leaks**: Not properly closing streams, connections, or other resources
- **JV.2.2 Memory Leaks**: Holding references to objects preventing garbage collection
- **JV.2.3 Static Collection Growth**: Static collections growing unbounded

#### JV.3 Concurrency Issues
- **JV.3.1 Naive Synchronization**: Overuse of synchronized blocks causing performance issues
- **JV.3.2 Race Conditions**: Inadequate synchronization of shared state
- **JV.3.3 Deadlock Prone Code**: Acquiring locks in inconsistent order

#### JV.4 Exception Handling Anti-patterns
- **JV.4.1 Generic Exception Catching**: Catching `Exception` instead of specific types
- **JV.4.2 Exception Swallowing**: Empty catch blocks or logging without re-throwing
- **JV.4.3 Checked Exception Abuse**: Overuse of checked exceptions for control flow

#### JV.5 Type System Misuse
- **JV.5.1 Raw Types**: Using collections without generics (e.g., `List` instead of `List<String>`)
- **JV.5.2 Null Pointer Issues**: Not using Optional for nullable return types
- **JV.5.3 String Concatenation**: Using `+` for multiple string concatenations

#### JV.6 Design Pattern Misuse
- **JV.6.1 Singleton Abuse**: Overusing singleton pattern creating hidden global state
- **JV.6.2 Factory Overengineering**: Complex factory patterns for simple object creation
- **JV.6.3 Observer Pattern Issues**: Not properly managing observer lifecycles

### Rust Anti-Patterns

#### RS.1 Ownership and Borrowing Issues
- **RS.1.1 Clone to Appease Compiler**: Excessive use of `.clone()` to avoid borrow checker errors
- **RS.1.2 Fighting the Borrow Checker**: Architectural issues manifesting as compiler errors
- **RS.1.3 Lifetime Annotation Proliferation**: Overuse of explicit lifetime parameters

#### RS.2 Memory Management Anti-patterns
- **RS.2.1 Premature Reference Counting**: Using `Rc`/`Arc` when ownership transfer would suffice
- **RS.2.2 Interior Mutability Overuse**: Excessive use of `RefCell`/`Mutex` avoiding borrow checker
- **RS.2.3 Memory Leaks via Reference Cycles**: Creating circular references with smart pointers

#### RS.3 Error Handling Problems
- **RS.3.1 Panic for Recoverable Errors**: Using `panic!` instead of proper error types
- **RS.3.2 Unwrap Abuse**: Excessive use of `.unwrap()` instead of proper error handling
- **RS.3.3 String-based Errors**: Using `String` for error types instead of structured errors

#### RS.4 Type System Misuse
- **RS.4.1 Inappropriate Trait Design**: Traits that don't represent coherent abstractions
- **RS.4.2 Overuse of Generic Parameters**: Making code unnecessarily complex with generics
- **RS.4.3 Enum Misuse**: Using enums where simple types would suffice

#### RS.5 Concurrency and Async Issues
- **RS.5.1 Blocking in Async**: Using blocking operations in async contexts
- **RS.5.2 Arc<Mutex<T>> Overuse**: Using heavy synchronization primitives unnecessarily
- **RS.5.3 Send/Sync Bound Issues**: Incorrect trait bounds for concurrent code

#### RS.6 Unsafe Code Misuse
- **RS.6.1 Unnecessary Unsafe**: Using `unsafe` when safe alternatives exist
- **RS.6.2 Unsafe Without Invariants**: Not documenting and maintaining safety invariants
- **RS.6.3 FFI Boundary Issues**: Incorrect handling of Foreign Function Interface

---

## Detection Strategy Categories

### Static Analysis Techniques
1. **Syntactic Pattern Matching**: Detecting specific code patterns and structures
2. **Semantic Analysis**: Understanding meaning and relationships in code
3. **Data Flow Analysis**: Tracking how data moves through the program
4. **Control Flow Analysis**: Understanding execution paths and branches
5. **Type System Analysis**: Leveraging language type information
6. **Dependency Analysis**: Understanding module and component relationships

### Metrics and Heuristics
1. **Complexity Metrics**: Cyclomatic complexity, nesting depth, function length
2. **Coupling Metrics**: Afferent/efferent coupling, instability metrics
3. **Cohesion Metrics**: LCOM (Lack of Cohesion of Methods), functional cohesion
4. **Size Metrics**: Lines of code, number of methods, parameter counts

### Graph-Based Analysis
1. **Call Graph Analysis**: Function and method invocation relationships
2. **Dependency Graph Analysis**: Module and package dependencies
3. **Inheritance Hierarchies**: Class relationship analysis
4. **Data Flow Graphs**: Variable usage and mutation tracking

---

## Implementation Priorities

### Phase 1: High-Impact Universal Patterns
1. God Object detection
2. Resource leak detection
3. Exception swallowing
4. Magic number detection

### Phase 2: Language-Specific Core Issues
1. JavaScript: Loose equality, callback hell, global pollution
2. Python: Mutable defaults, bare except, string concatenation
3. Java: Resource leaks, raw types, generic exception catching
4. Rust: Clone abuse, unwrap overuse, lifetime complexity

### Phase 3: Advanced Architectural Patterns
1. Leaky abstraction detection
2. Modularity violation analysis
3. Insufficient access control
4. Complex design pattern misuse

---

## Validation and Testing Strategy

Each anti-pattern detector should include:
1. **Positive Test Cases**: Code examples that should trigger detection
2. **Negative Test Cases**: Similar code that should not trigger false positives
3. **Edge Cases**: Boundary conditions and corner cases
4. **Performance Benchmarks**: Ensure detectors scale to large codebases
5. **Precision/Recall Metrics**: Quantitative assessment of detector accuracy

---

## Integration with Uveddi Architecture

This master list directly informs the development of:
1. **Analysis Engine Modules**: Individual detectors for each anti-pattern
2. **Language Parsers**: AST analysis for language-specific patterns
3. **Reporting System**: Structured output for detected issues
4. **Configuration System**: Allowing users to enable/disable specific detectors
5. **Plugin Architecture**: Extensible system for custom anti-pattern detectors
