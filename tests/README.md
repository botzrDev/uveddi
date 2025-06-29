# CodeAtlas Test Suite Implementation Guide

This document provides guidance for the next developer on how to implement comprehensive tests for all Sprint 2 features in the CodeAtlas project. The test files are scaffolded and ready for detailed test logic and fixture expansion.

## General Guidelines
- Follow Rust best practices and idioms for all tests.
- Use descriptive test names and clear comments.
- Prefer real-world and edge-case fixtures for all supported languages (Rust, Python, JS/TS).
- Use the built-in `#[test]` attribute and avoid in-module tests; keep all tests in the `tests/` directory.
- Use `unimplemented!()` or `todo!()` only as temporary placeholders.
- Ensure all error handling paths are tested, including malformed files, unsupported languages, and missing configuration/API keys.

## Test File Overview
- `ast_multilang.rs`: Test AST parsing and caching for Rust, Python, and JS/TS. Cover cache hits, language detection, and error cases.
- `dependency_extraction.rs`: Test dependency extraction for all supported languages. Include edge cases and malformed code.
- `god_object_detection.rs`: Test God Object anti-pattern detection, severity scoring, and code snippet extraction for all languages.
- `cycle_detection.rs`: Test cycle detection, severity scoring, and snippet extraction for all languages and complex graphs.
- `ai_explanations.rs`: Test AI explanation generation, prompt template rendering, and fallback logic for missing API keys.
- `reporting.rs`: Test Markdown/JSON report output, summary statistics, and inclusion of code/AI context.
- `cli_integration.rs`: Test CLI output for all formats, error handling, and AI integration toggling.
- `error_handling.rs`: Test all error handling paths, including malformed files, unsupported languages, timeouts, and missing config.

## How to Implement
1. **Import the necessary modules** at the top of each test file (see TODOs in each file).
2. **Write tests** for each major feature and edge case. Use temporary files and fixtures as needed.
3. **Expand fixtures** in `tests/fixtures/` for real-world and edge-case code samples.
4. **Replace all `unimplemented!()`/`todo!()`** with real test logic.
5. **Run `cargo test`** frequently to ensure all tests pass and error handling is robust.
6. **Document any tricky cases** or design decisions in this README.

## Example Test Structure
```rust
#[test]
fn test_feature_x_handles_edge_case() {
    // Arrange: set up input/fixture
    // Act: call the function/module under test
    // Assert: check the result, including error handling
}
```

## Additional Notes
- If you add new features, create a new test file or expand the relevant one.
- Keep this README updated with new testing strategies or requirements.

---
Happy testing!
