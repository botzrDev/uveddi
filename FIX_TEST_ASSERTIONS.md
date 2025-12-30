# Developer Prompt: Fix Test Assertion Failures

## Context

After completing the WASM plugin system implementation, all code compiles successfully but **78 unit tests** are failing with assertion errors. These are runtime failures, not compilation errors.

**Test Summary:**
- 712 tests passing
- 78 tests failing
- All failures are assertion errors (not compilation errors)

## Task

Fix the 78 failing test assertions. The failures fall into several categories that should be addressed systematically.

## Failing Tests by Category

### 1. Cache System Tests (6 tests)
```
analysis::cache::eviction::tests::test_lru_eviction
analysis::cache::file_watcher::tests::test_glob_pattern_matching
analysis::cache::file_watcher::tests::test_pattern_matching
analysis::cache::file_watcher::tests::test_file_watcher_configuration
analysis::components::cache_manager::tests::test_cache_manager_ast_operations
analysis::detectors::cache_integration::tests::test_cache_report_generation
```

**Likely Issues:**
- Cache eviction logic may have changed
- File watcher pattern matching behavior updates
- AST cache operations returning different structures

### 2. Security Config Detectors (40+ tests)
```
analysis::detectors::security::config::analysis::credentials::detector::tests::*
analysis::detectors::security::config::analysis::defaults::password_defaults::tests::*
analysis::detectors::security::config::analysis::misconfigurations::*
analysis::detectors::security::config::analysis::permissions::*
analysis::detectors::security::config::detector::tests::*
analysis::detectors::security::config::language_support::env::*
analysis::detectors::security::config::language_support::toml::*
analysis::detectors::security::config::patterns::*
```

**Likely Issues:**
- Detection thresholds or confidence scores changed
- Pattern matching logic updated
- Expected issue counts don't match actual detection results
- Test fixtures may not match current detection rules

### 3. OWASP Security Detectors (10 tests)
```
analysis::detectors::security::owasp::detector::tests::*
analysis::detectors::security::owasp::language_support::web_frameworks::tests::*
analysis::detectors::security::owasp::scanners::dependency_scanner::tests::*
analysis::detectors::security::owasp::scanners::pattern_scanner::tests::*
analysis::detectors::security::owasp::vulnerabilities::csrf::tests::*
analysis::detectors::security::owasp::vulnerabilities::path_traversal::tests::*
analysis::detectors::security::owasp::vulnerabilities::session_management::tests::*
```

**Likely Issues:**
- Scanner detection logic changes
- Framework detection patterns updated
- Vulnerability detection threshold changes

### 4. Database Tests (5 tests)
```
database::connection::pool::tests::test_concurrent_connections
database::connection::pool::tests::test_connection_execution
database::monitoring::tests::test_metrics_history_cleanup
database::tests::test_database_initialization
database::tests::test_database_with_monitoring
```

**Likely Issues:**
- Connection pool behavior changes
- Monitoring metrics format changes
- Database initialization sequence changes

### 5. Other Component Tests (15+ tests)
```
analysis::components::ast_provider::tests::test_language_detection
analysis::detectors::anti_patterns::god_object::context_detector::tests::*
analysis::detectors::security::agents::tests::test_agent_analysis
analysis::detectors::security::core::tests::test_confidence_score_calculation
analysis::detectors::security::taint_analysis::tests::test_sanitizer_detector
analysis::detectors::security::validation::tests::*
analysis::file_discovery::tests::test_gitignore_respect
cli::enhanced_help::tests::test_command_suggestions
config::validation::tests::test_empty_config_validation
engine::cache::tests::cache_execution_tests::test_cache_hit_miss_recording
engine::test_integration::tests::test_context_god_object_detector
progress::tests::test_progress_tracking
```

## Approach

### Step 1: Run Tests with Output
```bash
cargo test --features wasm-plugins -- --nocapture 2>&1 | tee test_output.txt
```

### Step 2: Analyze Failure Patterns
For each failing test, determine if the failure is due to:
1. **Changed return values** - Update expected values in assertions
2. **Changed data structures** - Update test fixtures and assertions
3. **Changed behavior** - Update test logic to match new behavior
4. **Actual bugs** - Fix the implementation code

### Step 3: Fix by Category (Priority Order)

#### Priority 1: Database Tests
These are foundational - fix first:
- `src/database/connection/pool.rs` - connection pool tests
- `src/database/monitoring.rs` - metrics tests
- `src/database/mod.rs` or `src/database/lib.rs` - initialization tests

#### Priority 2: Cache System Tests
- `src/analysis/cache/eviction.rs`
- `src/analysis/cache/file_watcher.rs`
- `src/analysis/components/cache_manager.rs`

#### Priority 3: Core Component Tests
- `src/analysis/components/ast_provider.rs`
- `src/config/validation.rs`
- `src/cli/enhanced_help.rs`
- `src/progress/mod.rs`

#### Priority 4: Security Detector Tests (largest group)
Work through systematically:
- `src/analysis/detectors/security/config/` - config analysis
- `src/analysis/detectors/security/owasp/` - OWASP detectors
- `src/analysis/detectors/security/core.rs` - core security
- `src/analysis/detectors/security/validation.rs` - validation

### Step 4: Verify Each Fix
After fixing each test:
```bash
cargo test --features wasm-plugins <test_name> -- --nocapture
```

## Common Fix Patterns

### Pattern 1: Updated Expected Values
```rust
// Before
assert_eq!(result.count, 5);

// After - if actual count changed to 3
assert_eq!(result.count, 3);
```

### Pattern 2: Updated Struct Fields
```rust
// Before
assert!(result.issues.is_empty());

// After - if field was renamed
assert!(result.findings.is_empty());
```

### Pattern 3: Updated Return Types
```rust
// Before
let result = analyzer.analyze();
assert!(result.is_valid);

// After - if now returns Result
let result = analyzer.analyze().unwrap();
assert!(result.is_valid);
```

### Pattern 4: Updated Thresholds
```rust
// Before
assert!(confidence >= 0.8);

// After - if threshold changed
assert!(confidence >= 0.7);
```

## Commands

Run all failing tests:
```bash
cargo test --features wasm-plugins 2>&1 | grep FAILED
```

Run specific test with output:
```bash
cargo test --features wasm-plugins <test_path> -- --nocapture
```

Run tests in a specific module:
```bash
cargo test --features wasm-plugins analysis::detectors::security::config -- --nocapture
```

## Success Criteria

- All 790 tests pass (712 currently passing + 78 fixed)
- No test logic changes that hide actual bugs
- Tests accurately reflect current system behavior

## Notes

- The tests are in the same files as the implementation code (inline `#[cfg(test)]` modules)
- Some tests may need updated fixtures or mock data
- Focus on making tests reflect actual correct behavior, not just making them pass
- If a test failure reveals an actual bug, fix the implementation, not the test
