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
