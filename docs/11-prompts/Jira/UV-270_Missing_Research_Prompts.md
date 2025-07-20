# UV-270 Architecture Refactoring - Missing Research Prompts

This document contains individual GPT research prompts for the missing research areas identified in Epic UV-270. Each prompt is designed to gather comprehensive implementation guidance for the specific architectural refactoring tasks.

## 📋 **Research Priority Matrix**

| Research Area | Jira Issue | Priority | Estimated Research Time | Implementation Impact |
|---------------|------------|----------|------------------------|----------------------|
| AnalysisEngine Refactoring Strategy | UV-289 | **HIGH** | 8-10 hours | **CRITICAL** - Affects entire system |
| Async Pattern Consistency | UV-294 | **HIGH** | 6-8 hours | **HIGH** - Multiple modules affected |
| TUI Code Duplication Elimination | UV-286 | **MEDIUM** | 4-6 hours | **MEDIUM** - Localized to TUI |
| Method Complexity Reduction | UV-295 | **MEDIUM** | 4-6 hours | **MEDIUM** - Code quality improvement |
| Comprehensive Unit Testing | UV-296 | **MEDIUM** | 4-6 hours | **HIGH** - Quality assurance |

---

## 🎯 **Research Prompt 1: AnalysisEngine God Object Refactoring Strategy**

**Jira Issue**: UV-289  
**Priority**: HIGH  
**Estimated Research Time**: 8-10 hours  
**Implementation Impact**: CRITICAL

### Research Prompt

```markdown
# AnalysisEngine God Object Refactoring - Comprehensive Implementation Strategy

## Context
You are a senior Rust architect tasked with refactoring a 780+ line AnalysisEngine god object in the Uveddi static code analysis tool. The engine currently violates Single Responsibility Principle and needs to be decomposed into focused, testable components while maintaining backward compatibility and performance.

## Current State Analysis
The AnalysisEngine currently handles:
- Detector lifecycle management (24 methods)
- AST parsing and caching
- Dependency graph construction
- Plugin management (WASM)
- Result aggregation and reporting
- Configuration management
- Error handling across all operations

## Existing Architecture Constraints
- Must integrate with existing dependency injection system (UV-156 architecture)
- Must maintain existing public API for backward compatibility
- Must support async operations with tokio
- Must integrate with tree-sitter AST parsing
- Must support WASM plugin system
- Must maintain performance characteristics

## Research Objectives

### 1. Component Separation Strategy
Design a comprehensive component separation strategy that:
- Identifies 4-6 focused components from the current god object
- Defines clear responsibilities for each component
- Establishes component interfaces and communication patterns
- Maintains cohesion within components while reducing coupling between them

### 2. Interface Design
Create detailed interface designs for:
- Component-to-component communication patterns
- Dependency injection integration points
- Error handling propagation between components
- Async operation coordination
- State management across components

### 3. Migration Strategy
Develop a step-by-step migration plan that:
- Allows incremental refactoring without breaking existing functionality
- Defines intermediate states during migration
- Identifies testing strategies for each migration step
- Minimizes risk during the refactoring process

### 4. Performance Impact Analysis
Analyze potential performance implications:
- Component communication overhead
- Memory allocation patterns
- Async operation coordination costs
- Caching strategies across components

## Deliverables Required

### A. Component Architecture Design
```rust
// Provide detailed component structure like:
pub struct AnalysisEngine {
    detector_manager: DetectorManager,
    dependency_analyzer: DependencyAnalyzer,
    cache_manager: CacheManager,
    plugin_manager: PluginManager,
    // ... other components
}

// Define traits for each component
pub trait ComponentManager {
    // Define interface methods
}
```

### B. Component Responsibility Matrix
Create a detailed matrix showing:
- What each component is responsible for
- What each component is NOT responsible for
- How components interact with each other
- Dependencies between components

### C. Migration Roadmap
Provide a detailed step-by-step migration plan:
1. Phase 1: Extract Component A (estimated effort, risks, testing strategy)
2. Phase 2: Extract Component B (dependencies on Phase 1, integration points)
3. Phase N: Final integration and cleanup

### D. Testing Strategy
Design comprehensive testing approaches for:
- Individual component testing with mocks
- Integration testing between components
- Performance regression testing
- Backward compatibility validation

### E. Error Handling Strategy
Define how errors propagate between components:
- Component-specific error types
- Error conversion at component boundaries
- Context preservation during error propagation

## Success Criteria
- Each component has a single, well-defined responsibility
- Component interfaces are clean and minimal
- Migration can be performed incrementally
- Performance characteristics are maintained or improved
- Testing strategy ensures reliability during refactoring
- Backward compatibility is preserved

## Implementation Context
- Language: Rust 2021 edition
- Async Runtime: tokio
- Error Handling: thiserror + anyhow
- Testing: Standard Rust testing with mockall for mocking
- Architecture Pattern: Dependency injection with builder pattern

Please provide a comprehensive research document with detailed implementation guidance, code examples, and migration strategies.
```

---

## 🎯 **Research Prompt 2: Async Pattern Consistency Standardization**

**Jira Issue**: UV-294  
**Priority**: HIGH  
**Estimated Research Time**: 6-8 hours  
**Implementation Impact**: HIGH

### Research Prompt

```markdown
# Async Pattern Consistency - Comprehensive Standardization Guide

## Context
You are a Rust async systems expert tasked with standardizing async/await patterns across the Uveddi codebase. The system currently has mixed async/sync patterns that create inconsistent APIs and potential performance issues.

## Current State Analysis
The codebase exhibits:
- Mixed async/sync patterns in similar operations
- Inconsistent error handling in async contexts
- Unclear boundaries between sync and async code
- Potential blocking operations in async contexts
- Inconsistent tokio integration patterns

## Existing Architecture Constraints
- Built on tokio runtime
- Must integrate with tree-sitter (sync) AST parsing
- Must support file I/O operations
- Must coordinate with WASM plugin system
- Must maintain CLI and TUI interfaces
- Must support concurrent analysis operations

## Research Objectives

### 1. Async Pattern Standardization
Define comprehensive standards for:
- When to use async vs sync patterns
- Consistent async function signatures
- Error handling in async contexts
- Resource management in async operations
- Cancellation and timeout patterns

### 2. Sync/Async Boundary Design
Establish clear patterns for:
- Where sync/async boundaries should exist
- How to handle sync operations in async contexts
- Blocking operation management
- Thread pool usage for CPU-intensive work

### 3. Performance Optimization Patterns
Design patterns for:
- Efficient async operation composition
- Resource pooling in async contexts
- Concurrent operation coordination
- Memory management in async operations

### 4. Error Handling in Async Contexts
Standardize:
- Async-specific error types and propagation
- Context preservation across await points
- Error recovery patterns in async operations
- Timeout and cancellation error handling

## Deliverables Required

### A. Async Pattern Standards
```rust
// Provide standardized patterns like:

// Standard async function signature
pub async fn analyze_file(&self, path: &Path) -> Result<AnalysisResult, AnalysisError> {
    // Implementation pattern
}

// Error handling pattern
async fn operation_with_context(&self) -> Result<T, AnalysisError> {
    some_async_operation()
        .await
        .with_context(|| format!("Operation failed for: {}", context))?;
}

// Resource management pattern
async fn with_managed_resource<T>(&self, operation: impl FnOnce() -> T) -> Result<T, Error> {
    // Resource acquisition and cleanup pattern
}
```

### B. Sync/Async Boundary Guidelines
Define clear rules for:
- Which operations should be async vs sync
- How to wrap sync operations for async contexts
- Thread pool usage patterns
- Blocking operation handling

### C. Concurrent Operation Patterns
Provide patterns for:
- Parallel file processing
- Concurrent detector execution
- Resource sharing between async operations
- Coordination between async tasks

### D. Integration Patterns
Design integration approaches for:
- CLI command execution (sync entry points)
- TUI event handling (async event loops)
- File system operations (async I/O)
- Plugin system coordination

### E. Testing Patterns
Define testing approaches for:
- Async function testing
- Concurrent operation testing
- Timeout and cancellation testing
- Mock async dependencies

### F. Performance Guidelines
Establish guidelines for:
- Avoiding async overhead where unnecessary
- Efficient async operation composition
- Resource pooling strategies
- Memory allocation patterns

## Success Criteria
- Clear, consistent async patterns across all modules
- Optimal performance characteristics
- Proper error handling in all async contexts
- Clean sync/async boundaries
- Comprehensive testing coverage for async operations

## Implementation Context
- Runtime: tokio with multi-threaded scheduler
- Error Handling: thiserror + anyhow with async context
- File I/O: tokio::fs for async file operations
- Concurrency: tokio::spawn for task spawning
- Testing: tokio::test for async testing

Please provide a comprehensive standardization guide with detailed patterns, examples, and migration strategies for achieving async consistency across the codebase.
```

---

## 🎯 **Research Prompt 3: TUI Code Duplication Elimination**

**Jira Issue**: UV-286  
**Priority**: MEDIUM  
**Estimated Research Time**: 4-6 hours  
**Implementation Impact**: MEDIUM

### Research Prompt

```markdown
# TUI Code Duplication Elimination - Trait-Based Focus Management

## Context
You are a Rust TUI expert tasked with eliminating code duplication in the Uveddi TUI analyze form. The current implementation has 41 lines of repetitive `set_focused(false)` calls that need to be refactored using a trait-based approach.

## Current State Analysis
The TUI analyze form currently exhibits:
- 41 lines of repetitive focus management code
- Manual focus state management across multiple input fields
- Error-prone focus coordination
- Difficult maintenance when adding new input fields

## Existing Architecture Constraints
- Built using ratatui (formerly tui-rs)
- Must maintain existing keyboard navigation behavior
- Must support multiple input field types (text, selection, checkbox)
- Must integrate with existing event handling system
- Must maintain accessibility and usability standards

## Current Implementation Pattern
```rust
// Current repetitive pattern in src/tui/ui/analyze_form.rs:461-502
self.project_path_input.set_focused(false);
self.output_format_input.set_focused(false);
self.detector_selection_input.set_focused(false);
// ... 38 more similar lines
```

## Research Objectives

### 1. Trait-Based Focus Management Design
Design a trait-based system that:
- Eliminates repetitive focus management code
- Provides consistent focus behavior across input types
- Supports easy addition of new focusable components
- Maintains type safety and compile-time guarantees

### 2. Reusable Focus Management Utilities
Create utility patterns for:
- Focus state coordination across multiple components
- Keyboard navigation between focusable elements
- Focus transition animations and visual feedback
- Focus state persistence and restoration

### 3. Component Architecture Patterns
Design patterns for:
- Composable TUI components with focus support
- Event delegation for focus management
- State management for complex forms
- Validation integration with focus management

### 4. Integration with Existing Event System
Ensure seamless integration with:
- Current keyboard event handling
- Mouse interaction support
- Screen refresh and rendering cycles
- Error handling and validation feedback

## Deliverables Required

### A. FocusableInput Trait Design
```rust
// Provide comprehensive trait design like:
pub trait FocusableInput {
    fn set_focused(&mut self, focused: bool);
    fn is_focused(&self) -> bool;
    fn can_receive_focus(&self) -> bool;
    fn focus_priority(&self) -> u32; // For tab order
}

// Implementation for different input types
impl FocusableInput for TextInput { /* ... */ }
impl FocusableInput for SelectionInput { /* ... */ }
impl FocusableInput for CheckboxInput { /* ... */ }
```

### B. Focus Manager Implementation
```rust
// Provide focus management utility like:
pub struct FocusManager {
    focusable_inputs: Vec<Box<dyn FocusableInput>>,
    current_focus_index: Option<usize>,
}

impl FocusManager {
    pub fn new() -> Self { /* ... */ }
    pub fn add_input(&mut self, input: Box<dyn FocusableInput>) { /* ... */ }
    pub fn next_focus(&mut self) { /* ... */ }
    pub fn previous_focus(&mut self) { /* ... */ }
    pub fn set_focus_by_index(&mut self, index: usize) { /* ... */ }
    pub fn clear_all_focus(&mut self) { /* ... */ }
}
```

### C. Form Integration Patterns
Design patterns for:
- Integrating FocusManager with existing AnalyzeForm
- Event handling delegation
- State synchronization between form and focus manager
- Validation integration

### D. Component Registration System
Create patterns for:
- Automatic registration of focusable components
- Dynamic addition/removal of focusable elements
- Focus order management
- Component lifecycle integration

### E. Testing Strategy
Define testing approaches for:
- Focus transition behavior
- Keyboard navigation testing
- Component registration testing
- Integration testing with existing form logic

### F. Migration Strategy
Provide step-by-step migration plan:
1. Implement trait and basic utilities
2. Migrate existing input components
3. Integrate with form logic
4. Remove duplicated code
5. Add comprehensive tests

## Success Criteria
- Eliminate all 41 lines of repetitive focus management code
- Maintain existing keyboard navigation behavior
- Support easy addition of new focusable components
- Improve code maintainability and reduce error potential
- Provide comprehensive test coverage

## Implementation Context
- TUI Framework: ratatui
- Event Handling: crossterm for keyboard/mouse events
- State Management: Component-based state with message passing
- Testing: Standard Rust testing with TUI-specific test utilities

Please provide a comprehensive design document with trait definitions, implementation patterns, and migration strategies for eliminating TUI code duplication through trait-based focus management.
```

---

## 🎯 **Research Prompt 4: Method Complexity Reduction Patterns**

**Jira Issue**: UV-295  
**Priority**: MEDIUM  
**Estimated Research Time**: 4-6 hours  
**Implementation Impact**: MEDIUM

### Research Prompt

```markdown
# Method Complexity Reduction - Advanced Refactoring Patterns

## Context
You are a Rust code quality expert tasked with developing comprehensive patterns for reducing method complexity in the Uveddi static code analysis tool. The goal is to extract long methods into smaller, focused methods while maintaining performance and readability.

## Current State Analysis
The codebase contains methods with:
- Excessive cyclomatic complexity (>10)
- Long method bodies (>50 lines)
- Multiple responsibilities within single methods
- Nested control structures
- Complex parameter lists

## Existing Architecture Constraints
- Must maintain existing public APIs
- Must preserve performance characteristics
- Must integrate with tree-sitter AST parsing
- Must support async operations
- Must maintain error handling patterns

## Research Objectives

### 1. Method Extraction Patterns
Develop comprehensive patterns for:
- Identifying extraction opportunities in complex methods
- Determining optimal extraction boundaries
- Maintaining cohesion in extracted methods
- Preserving performance during extraction

### 2. Complexity Reduction Strategies
Design strategies for:
- Reducing cyclomatic complexity through pattern matching
- Simplifying nested control structures
- Breaking down complex conditional logic
- Reducing parameter complexity

### 3. Refactoring Safety Patterns
Create patterns for:
- Safe refactoring without breaking existing functionality
- Maintaining test coverage during refactoring
- Preserving error handling behavior
- Ensuring performance regression detection

### 4. Code Organization Principles
Establish principles for:
- Logical grouping of extracted methods
- Naming conventions for extracted methods
- Documentation patterns for complex operations
- Module organization for related functionality

## Deliverables Required

### A. Method Extraction Patterns
```rust
// Provide detailed extraction patterns like:

// Before: Complex method with multiple responsibilities
impl LongMethodsDetector {
    fn extract_rust_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<MethodMetrics>, AnalysisError> {
        // 60+ lines of complex logic
    }
}

// After: Extracted into focused methods
impl LongMethodsDetector {
    fn extract_rust_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<MethodMetrics>, AnalysisError> {
        let source = self.prepare_source(parsed_file)?;
        let tree = self.parse_ast_tree(parsed_file)?;
        let query = self.compile_language_query(&tree)?;
        self.process_query_matches(&query, &tree, &source, parsed_file)
    }
    
    fn prepare_source(&self, parsed_file: &ParsedFile) -> Result<&[u8], AnalysisError> { /* ... */ }
    fn parse_ast_tree(&self, parsed_file: &ParsedFile) -> Result<Tree, AnalysisError> { /* ... */ }
    fn compile_language_query(&self, tree: &Tree) -> Result<Query, AnalysisError> { /* ... */ }
    fn process_query_matches(&self, query: &Query, tree: &Tree, source: &[u8], file: &ParsedFile) -> Result<Vec<MethodMetrics>, AnalysisError> { /* ... */ }
}
```

### B. Complexity Reduction Techniques
Provide techniques for:
- Pattern matching simplification
- Early return patterns
- Guard clause implementation
- State machine patterns for complex logic

### C. Parameter Simplification Patterns
Design patterns for:
- Parameter object creation
- Builder pattern for complex parameters
- Context object patterns
- Configuration parameter grouping

### D. Error Handling Preservation
Ensure patterns maintain:
- Consistent error propagation
- Context preservation during extraction
- Error message clarity
- Recovery behavior consistency

### E. Performance Optimization
Address performance considerations:
- Avoiding unnecessary allocations during extraction
- Maintaining hot path efficiency
- Preserving inlining opportunities
- Minimizing function call overhead

### F. Testing Strategy for Extracted Methods
Define testing approaches for:
- Unit testing extracted methods in isolation
- Integration testing for method coordination
- Regression testing for complex refactoring
- Performance testing for extracted operations

### G. Refactoring Workflow
Provide step-by-step workflow:
1. Complexity analysis and identification
2. Extraction boundary determination
3. Safe extraction with tests
4. Integration and validation
5. Performance verification

## Success Criteria
- Reduce method cyclomatic complexity to <10
- Limit method length to <50 lines
- Maintain or improve performance
- Preserve all existing functionality
- Achieve comprehensive test coverage for extracted methods

## Implementation Context
- Language: Rust 2021 edition
- Testing: Standard Rust testing with criterion for performance
- AST Processing: tree-sitter integration
- Error Handling: thiserror + anyhow patterns
- Performance: Zero-cost abstractions preferred

Please provide a comprehensive guide with detailed refactoring patterns, extraction techniques, and safety strategies for reducing method complexity while maintaining code quality and performance.
```

---

## 🎯 **Research Prompt 5: Comprehensive Unit Testing Strategy**

**Jira Issue**: UV-296  
**Priority**: MEDIUM  
**Estimated Research Time**: 4-6 hours  
**Implementation Impact**: HIGH

### Research Prompt

```markdown
# Comprehensive Unit Testing Strategy - Refactored Components

## Context
You are a Rust testing expert tasked with developing a comprehensive unit testing strategy for the refactored Uveddi components. The goal is to achieve >80% test coverage while ensuring reliability, maintainability, and performance of the testing suite.

## Current State Analysis
The codebase currently has:
- Limited unit test coverage for core components
- Missing tests for AnalysisEngine and related components
- Inconsistent testing patterns across modules
- Limited mock usage for complex dependencies
- No performance testing for critical paths

## Existing Architecture Constraints
- Must integrate with dependency injection system
- Must support async testing with tokio
- Must handle complex AST parsing operations
- Must test WASM plugin integration
- Must validate error handling across all components

## Research Objectives

### 1. Component Testing Strategy
Develop comprehensive testing approaches for:
- Individual component testing with controlled dependencies
- Component interaction testing
- State management testing
- Configuration testing

### 2. Mock and Test Double Patterns
Design patterns for:
- Mocking complex dependencies (AST parsers, file systems)
- Test doubles for external services
- Fake implementations for testing
- Stub patterns for controlled behavior

### 3. Integration Testing Patterns
Create patterns for:
- Testing component coordination
- End-to-end workflow testing
- Error propagation testing
- Performance regression testing

### 4. Test Data Management
Establish patterns for:
- Test fixture generation and management
- Realistic test data creation
- Test data isolation and cleanup
- Performance test data sets

## Deliverables Required

### A. Component Testing Patterns
```rust
// Provide comprehensive testing patterns like:

#[cfg(test)]
mod analysis_engine_tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        TestDetectorManager {}
        impl DetectorManager for TestDetectorManager {
            fn initialize(&mut self) -> Result<(), UveddiError>;
            fn run_detectors(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, UveddiError>;
        }
    }
    
    #[tokio::test]
    async fn test_engine_with_mock_components() {
        let mut mock_detector_manager = MockTestDetectorManager::new();
        mock_detector_manager
            .expect_run_detectors()
            .returning(|_| Ok(vec![]));
            
        let engine = AnalysisEngine::new(
            mock_detector_manager,
            // ... other mocked components
        );
        
        let result = engine.analyze(test_file_path()).await;
        assert!(result.is_ok());
    }
}
```

### B. Mock Strategy Framework
Design comprehensive mocking for:
- File system operations (using tempfile)
- AST parsing operations
- Network operations (for plugin downloads)
- Database operations
- Configuration loading

### C. Test Fixture Management
Create patterns for:
- Realistic code samples for analysis testing
- Configuration file fixtures
- Expected output fixtures
- Performance benchmark data

### D. Async Testing Patterns
Provide patterns for:
- Testing async component coordination
- Timeout and cancellation testing
- Concurrent operation testing
- Resource cleanup in async tests

### E. Error Condition Testing
Design comprehensive error testing for:
- Component failure scenarios
- Error propagation between components
- Recovery behavior testing
- Error message validation

### F. Performance Testing Integration
Create patterns for:
- Benchmark integration with unit tests
- Performance regression detection
- Memory usage testing
- Concurrent performance testing

### G. Test Organization Structure
Define organization patterns for:
- Test module structure
- Test naming conventions
- Test categorization (unit, integration, performance)
- Test documentation standards

### H. Continuous Integration Integration
Provide patterns for:
- Test execution in CI/CD pipelines
- Test result reporting
- Coverage reporting integration
- Performance regression detection in CI

## Success Criteria
- Achieve >80% test coverage for all refactored components
- Comprehensive error condition testing
- Fast test execution (<5 minutes for full suite)
- Reliable and deterministic test results
- Clear test failure diagnostics

## Implementation Context
- Testing Framework: Standard Rust testing with tokio::test
- Mocking: mockall for trait mocking
- Fixtures: tempfile for file system testing
- Performance: criterion for benchmarking
- Coverage: tarpaulin for coverage reporting

Please provide a comprehensive testing strategy with detailed patterns, mock implementations, and organizational guidelines for achieving comprehensive test coverage of refactored components.
```

---

## 📋 **Research Execution Plan**

### Recommended Research Order

1. **AnalysisEngine Refactoring Strategy (UV-289)** - 8-10 hours
   - **Priority**: CRITICAL - Affects entire system architecture
   - **Dependencies**: None - Can start immediately
   - **Impact**: Enables all other refactoring work

2. **Async Pattern Consistency (UV-294)** - 6-8 hours
   - **Priority**: HIGH - Affects multiple modules
   - **Dependencies**: Should coordinate with AnalysisEngine refactoring
   - **Impact**: System-wide consistency improvement

3. **Comprehensive Unit Testing (UV-296)** - 4-6 hours
   - **Priority**: HIGH for quality assurance
   - **Dependencies**: Should follow component refactoring design
   - **Impact**: Quality assurance for all refactoring work

4. **TUI Code Duplication (UV-286)** - 4-6 hours
   - **Priority**: MEDIUM - Localized impact
   - **Dependencies**: None - Can be done in parallel
   - **Impact**: TUI maintainability improvement

5. **Method Complexity Reduction (UV-295)** - 4-6 hours
   - **Priority**: MEDIUM - Code quality improvement
   - **Dependencies**: Can leverage AnalysisEngine refactoring patterns
   - **Impact**: Overall code quality improvement

### Total Research Effort: 26-36 hours

### Research Output Format
Each research prompt should produce:
- Comprehensive implementation guide (markdown document)
- Code examples and patterns
- Migration strategies
- Testing approaches
- Performance considerations
- Integration guidelines

### Research Documentation Location
Store completed research in:
- `docs/06-research/Specialized/UV-270/`
- Individual files for each research area
- Cross-references to related UV-270 issues

---

## 🎯 **Next Steps**

1. **Execute research prompts** in recommended priority order
2. **Document findings** in the specified location structure
3. **Update Jira issues** with completed research references
4. **Begin implementation** of well-researched components
5. **Iterate on research** as implementation reveals additional needs

This research foundation will enable confident implementation of the UV-270 architecture refactoring with minimal risk and maximum quality.