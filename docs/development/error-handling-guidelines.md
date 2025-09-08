# Error Handling Guidelines for UV-276

This document provides comprehensive guidelines for error handling in Uveddi, specifically addressing the replacement of unsafe `unwrap()` operations.

## 🚫 Prohibited Patterns

### Never Use These in Production Code

```rust
// ❌ NEVER: Raw unwrap() calls
let result = some_operation().unwrap();

// ❌ NEVER: expect() without detailed context
let value = parser.parse(code, None).expect("failed");

// ❌ NEVER: Panic-prone operations without recovery
let index = collection.len() - 1; // Could underflow
let item = collection[index]; // Could panic
```

## ✅ Recommended Patterns

### 1. Parser Operations

Replace tree-sitter and other parser unwraps with proper error handling:

```rust
// ❌ Before
let tree = parser.parse(code, None).unwrap();

// ✅ After
let tree = parser.parse(code, None)
    .ok_or_else(|| AnalysisError::parse_error("Failed to parse source code"))?;
```

### 2. Collection Access

Replace unsafe indexing with safe access patterns:

```rust
// ❌ Before
let item = collection.get(0).unwrap();

// ✅ After
let item = collection.get(0)
    .ok_or_else(|| AnalysisError::collection_access_error(
        format!("Index 0 out of bounds for collection of size {}", collection.len())
    ))?;
```

### 3. Type Conversions

Handle conversion failures gracefully:

```rust
// ❌ Before
let line_num: u32 = large_value.try_into().unwrap();

// ✅ After
let line_num: u32 = large_value.try_into()
    .map_err(|_| AnalysisError::conversion_error(
        format!("Line number {} exceeds u32 range", large_value)
    ))?;
```

### 4. Mutex and Lock Operations

Use safe lock acquisition patterns:

```rust
// ❌ Before
let guard = mutex.lock().unwrap();

// ✅ After - Option 1: Propagate error
let guard = mutex.lock()
    .map_err(|_| AnalysisError::lock_error("Mutex poisoned"))?;

// ✅ After - Option 2: Handle gracefully
let guard = match mutex.lock() {
    Ok(guard) => guard,
    Err(_) => {
        error!("Mutex poisoned, operation aborted");
        return None; // or appropriate fallback
    }
};
```

### 5. Query Captures

Handle tree-sitter query captures safely:

```rust
// ❌ Before
let name_node = query_match.captures.get(0).unwrap().node;

// ✅ After
let captures = &query_match.captures;
let name_node = captures.get(0)
    .ok_or_else(|| AnalysisError::query_error("Missing name capture in query"))?
    .node;
```

## 🛠️ Error Type Guidelines

### Use Specific Error Types

Always use the most specific error type available:

```rust
// ✅ Specific error types
AnalysisError::parse_error("Parser failed on malformed syntax")
AnalysisError::query_error("Tree-sitter query missing capture group 'function'")  
AnalysisError::collection_access_error("Index 5 out of bounds")
AnalysisError::conversion_error("Value exceeds target type range")
AnalysisError::data_not_found_error("Expected AST node not found")
AnalysisError::lock_error("Mutex poisoned by panic")
```

### Provide Actionable Error Messages

Error messages should be:
- **Descriptive**: Explain what went wrong
- **Contextual**: Include relevant values and state
- **Actionable**: Suggest what might fix the issue

```rust
// ✅ Good error messages
AnalysisError::parse_error("Failed to parse Rust code at line 42: unexpected token '{'")
AnalysisError::collection_access_error("Attempted to access element 5 in collection of size 3")
AnalysisError::conversion_error("Line number 4294967296 exceeds u32::MAX (4294967295)")
```

## 🧪 Testing Error Conditions

### Always Test Error Paths

For every unwrap replacement, add corresponding tests:

```rust
#[test]
fn test_parse_error_handling() {
    let mut parser = Parser::new();
    // Test with invalid syntax
    let result = safe_parse_code(&mut parser, "invalid rust code {{{");
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AnalysisError::ParseError { .. }));
}

#[test]
fn test_collection_access_error_handling() {
    let empty_vec: Vec<String> = vec![];
    let result = empty_vec.get(0)
        .ok_or_else(|| AnalysisError::collection_access_error("Empty collection"));
    
    assert!(result.is_err());
}
```

## 📝 Migration Checklist

For each file with unwrap() calls:

### Analysis Phase
- [ ] Identify all `.unwrap()` calls in the file
- [ ] Categorize by risk level (Critical/High/Medium/Low)
- [ ] Document the expected error conditions

### Implementation Phase
- [ ] Replace with appropriate error handling pattern
- [ ] Add comprehensive error context
- [ ] Ensure error types are specific and meaningful
- [ ] Update function signatures to return `Result<T, AnalysisError>`

### Testing Phase
- [ ] Write tests for each error condition
- [ ] Verify error messages are actionable
- [ ] Test error propagation through call stack
- [ ] Ensure graceful degradation where appropriate

### Quality Gates
- [ ] No `.unwrap()` calls remaining in production code
- [ ] All error paths tested with >90% coverage
- [ ] Error messages provide actionable context
- [ ] Performance impact measured and acceptable
- [ ] CI enforcement preventing future unwrap introduction

## 🎯 Performance Considerations

### Minimize Error Handling Overhead

```rust
// ✅ Efficient error handling
let item = collection.get(index)
    .ok_or_else(|| AnalysisError::collection_access_error("Index out of bounds"))?;

// ❌ Avoid expensive error message construction in hot paths
let item = collection.get(index)
    .ok_or_else(|| AnalysisError::collection_access_error(
        format!("Index {} out of bounds in collection {:?}", index, collection) // Expensive!
    ))?;
```

### Use Lazy Error Message Construction

```rust
// ✅ Lazy evaluation
.ok_or_else(|| AnalysisError::collection_access_error("Index out of bounds"))

// ❌ Eager evaluation (always constructs error even on success)
.ok_or(AnalysisError::collection_access_error("Index out of bounds"))
```

## 🔍 CI Integration

The repository includes automated checks in `.github/workflows/unwrap-detection.yml`:

- **Unwrap Detection**: Fails CI if unwrap() found in production code
- **Expect Validation**: Warns about insufficient expect() contexts
- **Statistics Reporting**: Tracks progress toward UV-276 compliance

## 📚 Additional Resources

- See `tests/error_handling/unwrap_replacements.rs` for comprehensive examples
- Review existing error handling in `src/analysis/errors.rs`
- Check the main error module in `src/error/mod.rs` for unified error types

---

**Remember**: The goal is not just to remove unwrap() calls, but to create robust, maintainable error handling that provides clear feedback and graceful failure modes.