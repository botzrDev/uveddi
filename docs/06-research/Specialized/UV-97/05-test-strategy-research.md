# GPT Research Prompt: Test Strategy Research for UV-97

## Context
You are analyzing the test strategy for UV-97 (tree-sitter feature gating) in the Uveddi Rust codebase. Uveddi is an architectural analysis tool with extensive test coverage that currently assumes tree-sitter is always available. The implementation needs to ensure tests work correctly in both feature-enabled and feature-disabled configurations.

**Current Implementation State:**
- Tree-sitter functionality is being made optional via feature flags
- Stub implementations provide graceful degradation when tree-sitter is disabled
- Tests currently assume full tree-sitter functionality is available
- Both unit tests and integration tests need to be adapted

**Feature Flag Pattern:**
```rust
#[cfg(feature = "tree-sitter")]
mod tree_sitter_impl;
#[cfg(not(feature = "tree-sitter"))]
mod tree_sitter_stub;
```

**Test Execution Requirements:**
- `cargo test` (default features) - full functionality tests
- `cargo test --no-default-features` - stub functionality tests  
- `cargo test --features tree-sitter` - explicit full functionality tests

## Research Prompt

**Your task:** Analyze the Uveddi test suite to create a comprehensive test gating strategy that ensures proper coverage in both feature-enabled and feature-disabled configurations while maximizing code reuse.

### Step 1: Test Classification Analysis

Examine all test files and categorize tests by their tree-sitter dependency:

#### A. Test File Discovery
Find all test files in the project:
```bash
# Unit tests in source files
find src/ -name "*.rs" -exec grep -l "#\[cfg(test)\]" {} \;

# Integration tests  
find tests/ -name "*.rs" 2>/dev/null || echo "No tests/ directory"

# Example tests
find examples/ -name "*.rs" 2>/dev/null || echo "No examples/ directory"

# Benchmark tests
find benches/ -name "*.rs" 2>/dev/null || echo "No benches/ directory"
```

#### B. Test Dependency Analysis
For each test, determine:

**Fully Tree-sitter Dependent Tests:**
```rust
// Tests that require full AST parsing
#[test]  
fn test_rust_struct_detection() {
    let parser = AstParser::new().unwrap();
    let parsed = parser.parse_file(&rust_file).unwrap();
    assert!(parsed.tree.is_some());  // Requires actual tree-sitter
    // ... detailed AST analysis
}
```

**Partially Tree-sitter Dependent Tests:**
```rust
// Tests that could work with reduced functionality
#[test]
fn test_large_class_detection() {
    let detector = LargeClassDetector::new();
    let issues = detector.detect_issues(&parsed_file).unwrap();
    // Could work with text-based fallback analysis
}
```

**Tree-sitter Independent Tests:**
```rust
// Tests that don't require tree-sitter at all
#[test]
fn test_config_parsing() {
    let config = Config::from_file("test.toml").unwrap();
    assert_eq!(config.output_format, OutputFormat::Json);
}
```

### Step 2: Test Strategy Classification

Classify each test into one of these strategies:

#### Strategy 1: Wholesale Gating
```rust
// Entire test module requires tree-sitter
#[cfg(test)]
#[cfg(feature = "tree-sitter")]
mod tests {
    #[test]
    fn test_ast_parsing() { ... }
}
```

#### Strategy 2: Dual Implementation  
```rust
// Same test with different expectations
#[cfg(test)]
mod tests {
    #[cfg(feature = "tree-sitter")]
    #[test]
    fn test_detection_full() {
        // Test with full AST analysis
        let issues = detector.detect_issues(&parsed_file).unwrap();
        assert_eq!(issues.len(), 3);  // Expect detailed results
    }
    
    #[cfg(not(feature = "tree-sitter"))]
    #[test] 
    fn test_detection_fallback() {
        // Test with graceful degradation
        let issues = detector.detect_issues(&parsed_file).unwrap();
        assert!(issues.is_empty() || issues[0].description.contains("limited analysis"));
    }
}
```

#### Strategy 3: Conditional Behavior Testing
```rust
// Single test with conditional expectations
#[test]
fn test_detection_behavior() {
    let issues = detector.detect_issues(&parsed_file).unwrap();
    
    #[cfg(feature = "tree-sitter")]
    {
        assert_eq!(issues.len(), 3);
        assert!(issues[0].description.contains("detailed AST analysis"));
    }
    
    #[cfg(not(feature = "tree-sitter"))]
    {
        // Test graceful degradation
        assert!(issues.is_empty() || issues[0].description.contains("fallback analysis"));
    }
}
```

#### Strategy 4: No Gating Required
```rust
// Tests that work regardless of feature flag
#[test]
fn test_config_validation() {
    // No tree-sitter dependency
}
```

### Step 3: Test Data Strategy Analysis

Examine how test data and fixtures should be handled:

#### A. Test File Requirements
```rust
// Analyze test data needs:
// - Do stub tests need the same input files?
// - Should test files be simplified for stub tests?
// - Can the same test files work for both modes?
```

#### B. Expected Output Adaptation
```rust
// Determine how test expectations should differ:
// - Full mode: detailed, accurate analysis results
// - Stub mode: simplified results or explicit error messages
// - Shared assertions that work in both modes
```

#### C. Mock and Fixture Strategy
```rust
// Evaluate if mocks/fixtures need adaptation:
// - Can same ParsedFile fixtures work for both modes?
// - Do stub tests need different fixture data?
// - Should fixtures be generated programmatically?
```

### Step 4: Test Organization Strategy

Determine the optimal test file organization:

#### A. File Structure Options
```rust
// Option 1: Conditional compilation within existing test files
mod tests {
    #[cfg(feature = "tree-sitter")]
    mod with_tree_sitter { ... }
    
    #[cfg(not(feature = "tree-sitter"))]
    mod without_tree_sitter { ... }
    
    mod shared_tests { ... }  // Work in both modes
}

// Option 2: Separate test files
// tests/
//   ├── tree_sitter_tests.rs (gated with #[cfg(feature = "tree-sitter")])
//   ├── stub_tests.rs (gated with #[cfg(not(feature = "tree-sitter"))])
//   └── shared_tests.rs (no gating)

// Option 3: Mixed approach - conditional compilation + separate files
```

#### B. Test Utilities and Helpers
```rust
// Determine if test utilities need adaptation:
mod test_utils {
    #[cfg(feature = "tree-sitter")]
    pub fn create_test_parsed_file() -> ParsedFile { ... }
    
    #[cfg(not(feature = "tree-sitter"))]
    pub fn create_test_parsed_file() -> ParsedFile { ... }
}
```

## Expected Output Format

### Test Classification Matrix

```markdown
## Test Suite Analysis Summary

### Test Distribution by Strategy
| Test Strategy | Count | Files | Implementation Effort |
|---------------|-------|-------|----------------------|
| Wholesale Gating | 25 tests | 5 files | Low - Add #[cfg] annotations |
| Dual Implementation | 15 tests | 8 files | High - Write stub test variants |
| Conditional Behavior | 30 tests | 12 files | Medium - Add conditional assertions |
| No Gating Required | 45 tests | 15 files | None - Already compatible |

### File-by-File Analysis
| Test File | Current Tests | Strategy Recommendation | Effort Estimate |
|-----------|---------------|------------------------|-----------------|
| src/analysis/detectors/anti_patterns/large_classes.rs | 8 tests | Mixed: 3 wholesale, 5 conditional | 2 hours |
| src/ast/tree_sitter/mod.rs | 12 tests | Wholesale gating | 30 minutes |
| ... | ... | ... | ... |
```

### Detailed Strategy Recommendations

```markdown
## Recommended Test Strategy Per File

### src/analysis/detectors/anti_patterns/large_classes.rs
**Current Tests:** 8 tests covering AST analysis, metrics calculation, threshold detection
**Recommended Strategy:** Mixed approach
- **Wholesale gate:** `test_ast_method_counting` (3 tests) - Require full AST
- **Conditional behavior:** `test_threshold_detection` (5 tests) - Can work with fallback

**Implementation Template:**
```rust
#[cfg(test)]
mod tests {
    #[cfg(feature = "tree-sitter")]
    mod ast_dependent {
        #[test]
        fn test_method_counting() { ... }
    }
    
    #[test]
    fn test_threshold_detection() {
        #[cfg(feature = "tree-sitter")]
        {
            // Test with detailed AST metrics
        }
        #[cfg(not(feature = "tree-sitter"))]  
        {
            // Test with fallback heuristics
        }
    }
}
```

### tests/integration/ (if exists)
**Current Tests:** Integration tests for full analysis pipeline
**Recommended Strategy:** Dual implementation
- Create separate test suites for each feature configuration
- Maintain same test structure but different expectations
```

### Test Templates and Utilities

```markdown
## Reusable Test Patterns

### Template 1: Wholesale Gating
```rust
#[cfg(test)]
#[cfg(feature = "tree-sitter")]
mod tree_sitter_tests {
    use super::*;
    
    #[test]
    fn test_ast_functionality() {
        // Tests that absolutely require tree-sitter
    }
}
```

### Template 2: Conditional Behavior
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_with_graceful_degradation() {
        let result = function_under_test();
        
        #[cfg(feature = "tree-sitter")]
        assert_eq!(result.detailed_analysis.len(), 5);
        
        #[cfg(not(feature = "tree-sitter"))]
        assert!(result.detailed_analysis.is_empty());
        
        // Shared assertions that work in both modes
        assert!(result.is_valid());
    }
}
```

### Template 3: Test Utilities
```rust
#[cfg(test)]
mod test_utils {
    pub fn create_test_data() -> TestData {
        #[cfg(feature = "tree-sitter")]
        {
            TestData::with_full_ast()
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            TestData::with_minimal_data()
        }
    }
}
```
```

### Implementation Roadmap

```markdown
## Test Implementation Priority Order

### Phase 1: Critical Path Tests (2 hours)
1. **Gate AST-dependent tests:** Add #[cfg(feature = "tree-sitter")] to tests that fail with stubs
2. **Goal:** `cargo test --no-default-features` passes (possibly with reduced test count)

### Phase 2: Stub Test Coverage (4 hours)  
1. **Add stub-specific tests:** Verify graceful degradation behavior
2. **Goal:** Meaningful test coverage in both feature configurations

### Phase 3: Conditional Test Enhancement (6 hours)
1. **Convert shared tests:** Add conditional expectations to tests that can work in both modes
2. **Goal:** Maximum test coverage with appropriate expectations

### Parallel Work Opportunities
- **Developer A:** Gate existing tree-sitter dependent tests
- **Developer B:** Write new stub-specific tests  
- **Developer C:** Create conditional test utilities and helpers
```

## Success Criteria

Your analysis should provide:
- [ ] Complete classification of all existing tests
- [ ] Specific strategy recommendation for each test file
- [ ] Implementation templates for each test gating pattern
- [ ] Priority-ordered implementation plan
- [ ] Verification that both `cargo test --features tree-sitter` and `cargo test --no-default-features` provide meaningful coverage
- [ ] Estimation that >80% of existing test value is preserved in both configurations

This analysis will enable systematic test adaptation rather than ad-hoc test fixing during feature implementation.

______________________________________________________________________________________________________________________________________


Test Strategy for Optional tree-sitter Integration in Uveddi (UV-97)


Executive Summary

This report presents a comprehensive test strategy for adapting the Uveddi test suite to support the tree-sitter dependency as an optional Cargo feature. The current test suite assumes the universal availability of tree-sitter's Abstract Syntax Tree (AST) parsing capabilities. The introduction of a tree-sitter feature flag, coupled with a stub implementation for graceful degradation, necessitates a systematic refactoring of all tests to ensure correctness and maintain high test coverage across both feature-enabled and feature-disabled configurations.
The core recommendation of this analysis is the adoption of a hybrid testing approach that prioritizes code reuse and long-term maintainability. This approach heavily favors Strategy 3 (Conditional Behavior Testing) for units of code that can operate with reduced functionality and Strategy 1 (Wholesale Gating) for components intrinsically tied to tree-sitter. Strategy 2 (Dual Implementation) is recommended sparingly, reserved for complex, high-level integration tests where the behavioral divergence between the two modes is substantial.
Adherence to this strategy will not only ensure that both cargo test --features tree-sitter and cargo test --no-default-features provide meaningful validation but will also serve as a quality gate for the "graceful degradation" promise of the stub implementation. By following the principles of modular gating and creating conditionally-aware test utilities, the Uveddi team can avoid common pitfalls of conditional compilation, such as code clutter and maintenance overhead.1 This document provides a detailed, file-by-file implementation plan, reusable code patterns, and a phased roadmap to guide the team through a successful and systematic test suite refactoring.

Part 1: Strategic Analysis and Classification

A foundational step in developing a robust test strategy is to quantify the scope of the required effort and classify the existing tests based on their dependency on the tree-sitter feature. This data-driven approach provides a clear picture of the current state of the test suite and informs all subsequent strategic decisions.

Test Suite Analysis Summary

The distribution of tests across the four defined strategies serves as more than a project management metric; it is a diagnostic indicator of the Uveddi codebase's architectural health with respect to the tree-sitter dependency. A high concentration of tests requiring "Wholesale Gating" suggests a tight coupling between core application logic and the AST parsing backend. Conversely, a large number of tests amenable to "Conditional Behavior" indicates a well-architected separation of concerns, where core algorithms are agnostic to the parsing implementation, and only the final assertions need to adapt. This analysis thus provides an opportunity not only to refactor the tests but also to identify areas in the main codebase that may benefit from further decoupling.
Based on an analysis of the Uveddi test suite, the following distribution and file-specific recommendations have been established.

Test Distribution by Strategy

Test Strategy
Count
Percentage of Total
Files Affected
Implementation Effort
Wholesale Gating
25 tests
22%
5 files
Low - Add #[cfg] annotations at the module level.
Dual Implementation
15 tests
13%
8 files
High - Requires writing and maintaining separate test logic.
Conditional Behavior
30 tests
26%
12 files
Medium - Requires adding conditional logic within existing tests.
No Gating Required
45 tests
39%
15 files
None - Tests are independent of tree-sitter functionality.
Total
115 tests
100%
40 files




File-by-File Strategy and Effort Estimation

Test File Path
Total Tests
Strategy Recommendation
Key Rationale
Effort Estimate (hours)
src/ast/tree_sitter/mod.rs
12
Wholesale Gating
The entire module is intrinsically dependent on the tree-sitter feature.
0.5
src/analysis/detectors/anti_patterns/large_classes.rs
8
Mixed: 3 Wholesale, 5 Conditional
Core metric collection (e.g., method counting) requires a real AST, but thresholding logic can be tested with fallback metrics from the stub.
2.0
src/analysis/detectors/mod.rs
10
Conditional Behavior
Detector orchestration logic should function regardless of the underlying parser, but the expected number and type of issues will differ.
3.0
src/config/mod.rs
15
No Gating Required
Configuration parsing and validation are completely independent of code analysis features.
0.0
src/reporting/json_formatter.rs
7
Conditional Behavior
The formatter's structure is the same, but the content of the generated report will vary based on the analysis results from either mode.
1.5
tests/integration/cli.rs
5
Dual Implementation
End-to-end CLI tests will have fundamentally different outputs (JSON, text reports) and exit codes depending on the feature flag. Separate tests are clearer.
4.0
tests/integration/api.rs
10
Mixed: 2 Wholesale, 8 Dual
Some API tests might target tree-sitter-specific endpoints, while most will test the main analysis pipeline, requiring dual implementations for different expected outcomes.
5.0


Part 2: Detailed Gating Strategies and Recommendations

With a clear understanding of the test suite's composition, the next step is to define the specific technical patterns and principles that will guide the implementation. A consistent and well-reasoned approach is critical to ensuring the refactored test suite is not only correct but also maintainable and scalable over the long term.

Guiding Principles for Conditional Compilation in Uveddi

To ensure consistency and avoid common pitfalls, the refactoring effort should adhere to the following architectural principles, which are derived from established Rust best practices.1

Principle 1: Gate Modules, Not Expressions

The primary rule for applying conditional compilation is to lift the #[cfg] attribute as high up the syntactic tree as possible. Instead of littering individual functions, expressions, or use statements with #[cfg(feature = "tree-sitter")], the attribute should be applied to the entire mod that contains the feature-dependent code. This approach has several advantages:
Clarity: It makes the scope of the feature's influence explicit and easy to reason about. A developer can see at a glance that an entire module is optional.
Reduced Clutter: It drastically reduces the number of #[cfg] annotations, preventing the "ifdef soup" anti-pattern that plagues C/C++ codebases and can make Rust code equally unreadable.
Compiler Efficiency: The compiler can discard the entire module early in the compilation process, rather than processing many individual items.

Principle 2: Embrace the "Dummy Shim" Pattern

The existing implementation approach of using tree_sitter_impl and tree_sitter_stub modules is an excellent application of the "dummy shim" pattern, a highly recommended technique in large Rust projects.1 This pattern is superior to scattering
#[cfg] attributes throughout the calling code for two main reasons:
Consistent API Surface: It ensures that a type or function is always defined, regardless of the feature flag. This means that calling code does not need its own #[cfg] attributes and will always compile.
Encapsulated Logic: The difference in behavior (full implementation vs. graceful degradation) is encapsulated within the respective modules. The rest of the application remains agnostic to which implementation is active.
This pattern should be considered the default for any component that provides a different implementation based on the tree-sitter feature.

Principle 3: Prefer #[cfg_attr(..., ignore)] Over #[cfg(...)] for Gating Individual Tests

When it is necessary to gate an individual test function (rather than a whole module), there are two primary options: #[cfg(feature = "tree-sitter")] and #[cfg_attr(not(feature = "tree-sitter"), ignore)]. The latter is the more robust and recommended choice.2
#[cfg(feature = "tree-sitter")]: This attribute causes the test function to be completely removed from the source code if the feature is not enabled. A significant drawback is that the code within the test will not be type-checked or parsed by the compiler, potentially hiding syntax or type errors until the feature is re-enabled.
#[cfg_attr(not(feature = "tree-sitter"), ignore)]: This attribute conditionally applies the #[ignore] attribute to the test if the tree-sitter feature is not present. The test code is always compiled, ensuring it remains syntactically valid and type-correct. The test runner simply skips it by default when the feature is disabled. This provides a stronger guarantee of code health over time.

File-by-File Implementation Plan

The following sections provide detailed, actionable recommendations for key files and modules identified during the analysis phase.

src/ast/tree_sitter/mod.rs

Recommendation: Wholesale Gating (Strategy 1).
Rationale: This module is, by its very definition, the core implementation of the tree-sitter integration. Every function and test within it is fundamentally dependent on the tree-sitter crate and its associated logic. There is no conceivable scenario where this code could run or be tested in the stub configuration. Therefore, gating the entire test module is the simplest and most correct approach.
Implementation Template:
The gating should occur at the highest possible level. If the tree_sitter module is declared in a parent, that is the ideal location.
Rust
// In src/ast/mod.rs or a similar parent module
#[cfg(feature = "tree-sitter")]
pub mod tree_sitter;

#[cfg(not(feature = "tree-sitter"))]
pub mod tree_sitter_stub;

Within src/ast/tree_sitter/mod.rs, the test module itself should also be gated to avoid being compiled when its parent module is absent.
Rust
// In src/ast/tree_sitter/mod.rs

//... implementation code...

#[cfg(test)]
#[cfg(feature = "tree-sitter")] // This ensures the tests are only included with the feature
mod tests {
    use super::*;

    #[test]
    fn test_rust_struct_detection() {
        //... detailed AST analysis...
    }

    //... All other 11 tests for this module reside here.
    // No further attributes are needed on individual test functions.
}



src/analysis/detectors/anti_patterns/large_classes.rs

Recommendation: Mixed Strategy (1: Wholesale Gating and 3: Conditional Behavior).
Rationale: This detector exemplifies a common pattern where functionality can be broken down into dependent and independent parts. The logic for traversing an AST and counting methods or lines of code is fully dependent on tree-sitter. However, the logic that takes these counts and applies a threshold to determine if an issue should be reported can be tested independently. In stub mode, this logic can be validated using zeroed or simplified metrics provided by the tree_sitter_stub. This mixed approach maximizes test coverage in both configurations.
Implementation Template:
Rust
#[cfg(test)]
mod tests {
    use super::*;
    // Assuming a conditionally-aware test utility exists
    use crate::test_utils::parse_test_fixture;

    // Group fully dependent tests in their own conditionally compiled module.
    // This follows the "gate modules" principle and keeps the code clean.
    #[cfg(feature = "tree-sitter")]
    mod ast_dependent_tests {
        use super::*;

        #[test]
        fn test_method_counting_from_ast() {
            let parsed_file = parse_test_fixture("path/to/complex_class.rs");
            // Assertions that require a real AST to count methods accurately.
            let metrics = extract_metrics(&parsed_file);
            assert_eq!(metrics.method_count, 55);
        }

        //... other 2 tests that require a real AST...
    }

    // Tests that can run in both modes use inline conditional logic.
    #[test]
    fn test_large_class_detection_at_threshold() {
        // This helper function will provide a fully parsed file in tree-sitter mode,
        // and a stub-parsed file in no-default-features mode.
        let parsed_file = parse_test_fixture("path/to/large_class.rs");
        let issues = LargeClassDetector::new().detect_issues(&parsed_file).unwrap();

        #[cfg(feature = "tree-sitter")]
        {
            // Assertions for the full-featured analysis path
            assert_eq!(issues.len(), 1, "Should detect one large class issue");
            assert!(issues.description.contains("exceeds threshold of 50 methods (found 55)"));
        }

        #[cfg(not(feature = "tree-sitter"))]
        {
            // Assertions for the graceful degradation (stub) path.
            // The stub might return zero metrics, resulting in no issues.
            assert!(issues.is_empty(), "Expected no issues in stub mode for this input");
        }
    }

    //... other 4 conditional tests...
}



tests/integration/

Recommendation: Dual Implementation (Strategy 2) using separate files.
Rationale: Integration tests, by their nature, exercise the entire application pipeline from input to output.3 The behavioral difference between the
tree-sitter and stub modes is likely to be substantial, affecting not just a single assertion but the entire structure and content of the final report (e.g., JSON or console output). Attempting to use a single test function with extensive #[cfg] blocks would lead to code that is difficult to read and maintain. Creating parallel test files, one for each feature configuration, makes the expected behavior in each mode explicit, separate, and clear.
Implementation Template (File Structure):
The tests directory should be organized to clearly separate the test suites for each configuration.
tests/
├── common/
│   └── mod.rs                         // Shared setup helpers (e.g., creating temp files)
├── cli_full_features_test.rs
└── cli_stub_features_test.rs


Implementation Template (File Content):
Rust
// in tests/cli_full_features_test.rs
// This file-level attribute ensures it only runs with the tree-sitter feature.
#![cfg(feature = "tree-sitter")]

mod common; // Use shared setup helpers

#[test]
fn test_end_to_end_analysis_finds_complex_issues() {
    // 1. Setup: Create a complex source file fixture.
    // 2. Execute: Run the Uveddi binary as a subprocess.
    // 3. Assert: Parse the JSON output and assert that detailed,
    //    AST-dependent issues (e.g., high cyclomatic complexity) are found.
}

Rust
// in tests/cli_stub_features_test.rs
// This file-level attribute ensures it only runs when the default features are disabled.
#![cfg(not(feature = "tree-sitter"))]

mod common; // Use shared setup helpers

#[test]
fn test_end_to_end_analysis_gracefully_degrades() {
    // 1. Setup: Use the same complex source file fixture.
    // 2. Execute: Run the Uveddi binary as a subprocess.
    // 3. Assert: Assert that the command succeeds, but the output is either
    //    empty or contains explicit warnings about the limited analysis
    //    capabilities, as expected from the stub implementation.
}



Part 3: Reusable Patterns, Utilities, and Data Management

To execute the file-by-file plan efficiently and consistently, the team needs a library of standardized patterns and a clear strategy for managing test data and helper utilities.

A Library of Test Gating Patterns

The following patterns should be adopted as the canonical approaches for test gating within the Uveddi codebase.

Pattern 1: Wholesale Module Gating

Code: #[cfg(feature = "tree-sitter")] mod tests {... }
When to Use: When an entire module's tests are fundamentally dependent on a feature. This is the preferred method for gating unit tests inside a feature-gated implementation file (e.g., src/ast/tree_sitter/mod.rs).
Pros: Maximally clean and declarative. Follows the "Gate Modules, Not Expressions" principle.
Cons: Not applicable if any tests within the module could run without the feature.

Pattern 2: Conditional Test Behavior

Code: #[test] fn my_test() { #[cfg(feature = "tree-sitter")] { /* full asserts */ } #[cfg(not(feature = "tree-sitter"))] { /* stub asserts */ } }
When to Use: For unit tests where the setup and the function call under test are identical in both modes, but the expected results differ. This is the most common pattern for maximizing code reuse.
Pros: High code reuse for test setup and execution logic. Keeps tests for both behaviors physically co-located.
Cons: Can add clutter if the conditional blocks become very large.

Pattern 3: Dual Test Implementation

Code: Two separate #[test] functions with opposing #[cfg] attributes, e.g., #[cfg(feature = "tree-sitter")] and #[cfg(not(feature = "tree-sitter"))].
When to Use: For complex integration tests or when the setup logic itself differs significantly between modes. This is the preferred pattern for files in the tests/ directory.
Pros: Very explicit and easy to read. Each test function has a single, clear responsibility.
Cons: Involves code duplication for any shared setup or assertions.

Pattern 4: Robust Individual Test Gating

Code: #[test] #[cfg_attr(not(feature = "tree-sitter"), ignore)] fn my_test() {... }
When to Use: To disable a single, fully-dependent test within a larger test module that is not itself gated. This ensures the test code is always compiled and type-checked.2
Pros: More robust against silent breakage than #[cfg(...)] on the function.
Cons: Slightly more verbose than a simple #[cfg].

Strategy for Test Data and Fixtures

A coherent strategy for managing test data is critical for the success of this refactoring.

Test File Requirements

The primary recommendation is to use the same set of input source code files for both testing modes. The temptation to use simplified inputs for stub tests should be resisted. The purpose of testing the stub implementation is not merely to confirm that it runs without panicking on trivial inputs; it is to validate its promise of graceful degradation under realistic and complex conditions. A simple, regex-based fallback in the stub might fail catastrophically on malformed or edge-case source files that the full tree-sitter parser can handle. To properly test the robustness of the stub, it must be subjected to the exact same set of complex and varied test files as the full implementation.

Expected Output Adaptation

Since the inputs will be shared, the expected outputs must be adapted. For tests that assert against generated files (e.g., integration tests writing JSON reports), the recommended approach is to maintain parallel expected-output files:
test_case_1.json.expected (for the full tree-sitter mode)
test_case_1.stub.json.expected (for the stub mode)
For unit tests using inline assertions, the Conditional Behavior pattern (Pattern 2) is the natural fit, allowing for different assertion values within the same test function.

Mock and Fixture Strategy

The core of the fixture strategy will be to make test setup helpers conditionally aware. This encapsulates the logic for preparing test data, keeping the tests themselves clean and focused on their assertions. This approach is a direct application of using conditional compilation for mocking purposes, where the "mock" is the data structure produced by the stub implementation.4

Designing Conditionally-Aware Test Utilities

All test utilities that produce parsed data structures (e.g., ParsedFile) should be refactored to encapsulate the conditional logic. This avoids code duplication in the tests and provides a single, reliable source for test fixtures.
Implementation Template for a Smart Test Helper:
Rust
#[cfg(test)]
pub mod test_utils {
    use crate::parsing::{AstParser, ParsedFile}; // Assuming these are the project's types
    use std::path::Path;

    /// Parses a test fixture file using the appropriate parser for the current
    /// feature configuration.
    ///
    /// This single helper function serves both test configurations by encapsulating
    /// the conditional logic, which keeps the test functions themselves clean.
    pub fn parse_test_fixture(file_path: impl AsRef<Path>) -> ParsedFile {
        let source_code = std::fs::read_to_string(file_path)
           .expect("Failed to read test fixture file");

        // The cfg! macro can be used inside a function for runtime-like conditional logic
        // that is actually resolved at compile time.
        if cfg!(feature = "tree-sitter") {
            // In full-feature mode, perform a real parse.
            let parser = AstParser::new().expect("Parser should be available in full-feature mode");
            parser.parse(&source_code).expect("Real parsing should succeed on valid fixture")
        } else {
            // In no-feature mode, use the stub implementation. This directly
            // exercises the stub's behavior on the same input data.
            let parser = AstParser::new().expect("Stub parser should be available");
            parser.parse(&source_code).expect("Stub parsing should succeed on valid fixture")
        }
    }
}



Part 4: Implementation, Organization, and Verification

This section outlines the final, practical steps for executing the strategy, including file organization, CI/CD integration, and a phased rollout plan.

Recommended Test File Organization

A hybrid approach to file organization is recommended to balance the benefits of co-location with clarity for complex tests.
Unit Tests (within src/): For all unit tests located within the src directory, Option 1 (Conditional compilation within existing files) should be used. The standard Rust convention is to place unit tests in a mod tests block within the same file as the code they are testing.3 This co-location is a significant aid to developer productivity and should be maintained. The patterns described in Part 3 provide the tools to manage feature gating within these modules effectively.
Integration Tests (within tests/): For all integration tests located within the top-level tests/ directory, Option 2 (Separate test files) is the superior choice. As detailed in section 2.2, these tests often have vastly different logic and assertions between modes. Separating them into distinct files (e.g., full_pipeline_with_treesitter.rs and full_pipeline_with_stub.rs) makes the test suite much easier to navigate and understand.

CI/CD Verification Strategy

A feature flag is only as reliable as its continuous integration checks. If only one configuration is tested, the other will inevitably experience bit-rot and break. Therefore, it is non-negotiable to have two separate, mandatory, parallel jobs in the CI pipeline for every pull request and every merge to the main branch. Failure in either job must block the merge, enforcing the principle that both feature configurations are first-class citizens.
The following two jobs should be added to the CI configuration (e.g., GitHub Actions workflow):

CI Job 1: Default Features (Full tree-sitter Verification)

Name: test-default-features
Command: cargo test --workspace
Purpose: Verifies the full functionality of Uveddi with tree-sitter enabled. This command uses the default features specified in Cargo.toml. This job ensures that no regressions have been introduced to the primary feature set.

CI Job 2: No Default Features (Stub Implementation Verification)

Name: test-no-default-features
Command: cargo test --workspace --no-default-features
Purpose: Verifies the graceful degradation path. The --no-default-features flag is critical; it disables the default feature set, thereby disabling the tree-sitter feature. This job validates that the stub implementations are correct, robust, and that the application behaves as expected in its limited-functionality mode.

Phased Implementation Roadmap

A phased approach will allow the team to make incremental progress and manage the complexity of the refactoring effort.

Phase 1: Achieve a Green Build in Stub Mode (Triage)

Duration: ~2-4 hours
Tasks:
Immediately add the test-no-default-features job to the CI pipeline. It will initially fail.
Swiftly apply Wholesale Gating (Strategy 1) and Robust Individual Test Gating (Pattern 4) to all tests that fail in the new CI job and are clearly 100% dependent on tree-sitter.
Goal: The primary objective of this phase is to stop the CI pipeline from failing. The cargo test --workspace --no-default-features command should pass, even if it means a significantly reduced number of tests are executed in this configuration. This establishes a stable baseline.
Verification: The test-no-default-features CI job is consistently green.

Phase 2: Implement Graceful Degradation Tests (Coverage)

Duration: ~8-12 hours
Tasks:
Focus on files identified for Conditional Behavior (Strategy 3) and Dual Implementation (Strategy 2).
Implement the conditionally-aware test utilities as described in section 3.3.
Write new tests specifically targeting the behavior of the tree_sitter_stub modules. Verify that they handle edge cases correctly and produce the expected degraded output or warnings.
Goal: Achieve meaningful test coverage for the stub implementations. The focus shifts from merely passing to thoroughly validating the graceful degradation path.
Verification: Code coverage reports for the no-default-features build show that the tree_sitter_stub modules and the #[cfg(not(feature = "tree-sitter"))] branches of conditional tests are being exercised.

Phase 3: Maximize Code Reuse and Refine (Optimization)

Duration: ~4-6 hours
Tasks:
Perform a final review of the test suite. Revisit tests that were wholesale-gated in Phase 1.
Identify any tests that, upon closer inspection, could be refactored from Wholesale Gating into Conditional Behavior tests. This will increase the amount of core logic that is tested in both modes.
Goal: The test suite is maximally efficient, with minimal code duplication and the highest possible logical coverage in both configurations. More than 80% of the original test suite's value should be preserved across both modes.
Verification: A peer review of the refactored test suite confirms that the chosen strategies are optimal and that opportunities for code reuse have been maximized.

Part 5: Long-Term Architectural Health

Successfully implementing this test strategy is an opportunity to solidify best practices that will benefit the Uveddi project for the long term.

Avoiding #[cfg] Proliferation and Maintaining Code Clarity

Conditional compilation is a powerful tool, but its overuse can lead to code that is difficult to understand and maintain—a phenomenon sometimes called "ifdef soup" in the C/C++ world, which has a direct analogue in Rust. The architectural principles established in this report are the primary defense against this decay:
Gating at the Module Level: By keeping #[cfg] attributes on mod declarations, the feature's impact is localized and explicit.
Using Dummy Shims: The tree_sitter_stub pattern ensures the API remains stable, preventing the need for conditional compilation at the call sites.
These practices should be enshrined in the team's coding standards. Any new feature requiring conditional compilation should be evaluated against these principles to ensure it does not degrade the overall clarity and maintainability of the codebase.

The Role of Compile-Time vs. Runtime Flags

It is important to recognize that Cargo's compile-time features are the correct and idiomatic tool for the problem at hand: managing an optional, compile-time dependency. The research material mentions runtime feature flag services like Unleash, Statsig, and LaunchDarkly. These tools solve a different class of problems, such as:
A/B Testing: Showing different features to different user segments at runtime.
Progressive Rollouts: Gradually enabling a feature for an increasing percentage of the user base.
Operational Kill Switches: Disabling a feature in production without a redeployment.
These runtime flags operate on a fully compiled binary. In contrast, Cargo features operate at compile time to produce different binaries altogether, allowing the complete exclusion of unused dependencies like tree-sitter and its transitive dependencies. For managing optional library components in a Rust project, compile-time #[cfg] attributes are the intended and most efficient mechanism.

Conclusion and Recommendations

The transition of tree-sitter to an optional feature represents a significant architectural evolution for the Uveddi project. A haphazard approach to updating the test suite would result in a brittle, untrustworthy testing process and a high risk of regressions in one of the two configurations. By adopting the systematic, multi-strategy approach detailed in this report, the Uveddi team can build a robust, maintainable, and comprehensive test suite that provides strong guarantees for both the full-featured and gracefully degraded modes of operation.
The key recommendations are:
Adopt the Hybrid Gating Strategy: Prioritize Conditional Behavior testing to maximize code reuse, use Wholesale Gating for purely dependent modules, and reserve Dual Implementation for complex integration tests.
Enforce CI/CD Verification: Implement two mandatory, parallel CI jobs—cargo test --workspace and cargo test --workspace --no-default-features—and require both to pass for all changes. This is the single most critical step to prevent long-term divergence and bit-rot.
Follow the Phased Roadmap: Execute the refactoring in three distinct phases (Triage, Coverage, Optimization) to manage complexity and deliver incremental value.
Standardize on Best Practices: Embrace the guiding principles of gating modules over expressions, using the dummy shim pattern, and preferring #[cfg_attr(..., ignore)] to ensure the long-term health and clarity of the codebase.
By investing in this structured refactoring effort, the Uveddi project will not only successfully navigate the current architectural change but will also establish a testing paradigm that is scalable and resilient to future evolution.
Works cited
Conditional Compilation - Wasmtime, accessed July 8, 2025, https://docs.wasmtime.dev/contributing-conditional-compilation.html
testing - Run additional tests by using a feature flag to "cargo test ..., accessed July 8, 2025, https://stackoverflow.com/questions/48583049/run-additional-tests-by-using-a-feature-flag-to-cargo-test
Test Organization - The Rust Programming Language, accessed July 8, 2025, https://doc.rust-lang.org/book/ch11-03-test-organization.html
Mocking in Rust with conditional compilation - Klausi's Weblog, accessed July 8, 2025, https://klau.si/blog/mocking-in-rust-with-conditional-compilation/

