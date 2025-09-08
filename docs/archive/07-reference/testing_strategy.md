# Uveddi Testing Strategy

## Overview

This document outlines the comprehensive testing strategy for Uveddi, covering all aspects from unit tests to end-to-end integration testing. Our testing approach ensures reliability, maintainability, and confidence in the codebase as it scales with a larger development team.

## Testing Philosophy

### Core Principles

1. **Test Pyramid**: More unit tests, fewer integration tests, minimal E2E tests
2. **Fast Feedback**: Tests should run quickly to enable rapid development
3. **Reliability**: Tests should be deterministic and not flaky
4. **Maintainability**: Tests should be easy to understand and modify
5. **Coverage**: Focus on meaningful coverage, not just percentage targets

### Quality Gates

**Minimum Requirements**:
- All tests must pass before merging
- Code coverage: Minimum 15%, target 25%+
- No test flakiness tolerated
- Performance tests must not regress

## Test Categories

### 1. Unit Tests

**Purpose**: Test individual functions and modules in isolation

**Location**: `src/` directory with `#[cfg(test)]` modules

**Scope**:
- Pure functions and algorithms
- Data structure operations
- Error handling logic
- Business logic validation

**Example Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_god_object_detection_basic() {
        // Arrange
        let ast = create_test_ast();
        let detector = GodObjectDetector::new();
        
        // Act
        let result = detector.analyze(&ast);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
```

**Guidelines**:
- Test both happy path and error cases
- Use descriptive test names
- Follow Arrange-Act-Assert pattern
- Mock external dependencies
- Keep tests focused and independent

### 2. Integration Tests

**Purpose**: Test interaction between multiple modules

**Location**: `tests/` directory

**Scope**:
- Module integration
- API contract testing
- Database operations
- File system interactions

**Current Integration Tests**:
```
tests/
├── ai_integration.rs          # AI provider integration
├── analysis_pipeline.rs      # Full analysis workflow
├── ast_multilang.rs          # Multi-language AST parsing
├── cli_integration.rs        # CLI command testing
├── database_operations.rs    # Database CRUD operations
└── plugin_system.rs          # Plugin loading and execution
```

**Example**:
```rust
#[tokio::test]
async fn test_full_analysis_pipeline() {
    let temp_dir = create_test_project();
    let config = AnalysisConfig::default();
    
    let result = run_analysis(&temp_dir, config).await;
    
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(!report.issues.is_empty());
    assert!(report.summary.total_files > 0);
}
```

### 3. End-to-End Tests

**Purpose**: Test complete user workflows

**Location**: `tests/e2e/` and `frontend/cypress/`

**Scope**:
- CLI command execution
- Frontend user journeys
- API endpoint testing
- Cross-platform compatibility

**CLI E2E Tests**:
```bash
# Test basic analysis command
./target/release/uveddi analyze ./test-project --output report.md

# Test AI integration
OPENAI_API_KEY=test ./target/release/uveddi analyze ./test-project --enable-ai

# Test configuration management
./target/release/uveddi config set analysis.threshold 0.8
```

### 4. Performance Tests

**Purpose**: Ensure performance requirements are met

**Location**: `benches/` directory

**Scope**:
- Analysis engine throughput
- Memory usage profiling
- AI provider response times
- Large codebase handling

**Benchmark Structure**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_analysis_engine(c: &mut Criterion) {
    let test_project = setup_large_test_project();
    
    c.bench_function("analyze_large_project", |b| {
        b.iter(|| {
            let engine = AnalysisEngine::new();
            black_box(engine.analyze(&test_project))
        })
    });
}

criterion_group!(benches, benchmark_analysis_engine);
criterion_main!(benches);
```

**Performance Targets**:
- 100-file Rust project: <30 seconds
- 1000-file project: <5 minutes
- Memory usage: <2GB for large projects
- AI response time: <10 seconds local, <5 seconds cloud

### 5. Security Tests

**Purpose**: Validate security measures and identify vulnerabilities

**Scope**:
- Input validation testing
- API key handling
- Plugin sandboxing
- Dependency vulnerability scanning

**Security Test Examples**:
```rust
#[test]
fn test_malicious_input_handling() {
    let malicious_code = include_str!("fixtures/malicious.rs");
    let result = parse_and_analyze(malicious_code);
    
    // Should handle gracefully without panicking
    assert!(result.is_ok() || result.is_err());
    // Should not expose sensitive information
    assert!(!format!("{:?}", result).contains("secret"));
}

#[test]
fn test_api_key_sanitization() {
    let config = Config::new();
    config.set_api_key("sk-test123");
    
    let debug_output = format!("{:?}", config);
    assert!(!debug_output.contains("sk-test123"));
    assert!(debug_output.contains("***"));
}
```

## Test Infrastructure

### Test Fixtures and Data

**Location**: `tests/fixtures/`

**Structure**:
```
tests/fixtures/
├── rust/
│   ├── god_object.rs          # Large class example
│   ├── cyclic_deps/           # Circular dependency example
│   └── clean_code.rs          # Well-structured code
├── python/
│   ├── anti_patterns.py       # Various anti-patterns
│   └── good_practices.py      # Clean Python code
├── javascript/
│   ├── callback_hell.js       # Async anti-patterns
│   └── modern_async.js        # Clean async code
└── projects/
    ├── small_rust_project/    # <50 files
    ├── medium_project/        # 100-500 files
    └── large_project/         # 1000+ files
```

**Fixture Guidelines**:
- Use realistic code examples
- Cover both positive and negative cases
- Include edge cases and boundary conditions
- Maintain fixtures as code evolves
- Document the purpose of each fixture

### Mock and Stub Infrastructure

**AI Provider Mocking**:
```rust
#[cfg(test)]
pub struct MockAiProvider {
    responses: HashMap<String, String>,
}

impl MockAiProvider {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        responses.insert(
            "god_object".to_string(),
            "This class has too many responsibilities...".to_string()
        );
        Self { responses }
    }
}

impl LlmProvider for MockAiProvider {
    async fn generate_explanation(&self, prompt: &str) -> Result<String, AiError> {
        Ok(self.responses.get("god_object").unwrap().clone())
    }
}
```

**Database Mocking**:
```rust
#[cfg(test)]
pub struct InMemoryDatabase {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl Database for InMemoryDatabase {
    async fn store(&self, key: &str, value: &[u8]) -> Result<(), DatabaseError> {
        self.data.lock().unwrap().insert(key.to_string(), value.to_vec());
        Ok(())
    }
}
```

### Test Utilities

**Common Test Helpers**:
```rust
// tests/test_utils.rs
pub fn create_temp_project(files: &[(&str, &str)]) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    for (path, content) in files {
        let file_path = temp_dir.path().join(path);
        std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        std::fs::write(file_path, content).unwrap();
    }
    temp_dir
}

pub fn assert_contains_issue(report: &AnalysisReport, issue_type: &str) {
    assert!(
        report.issues.iter().any(|issue| issue.issue_type == issue_type),
        "Report should contain {} issue", issue_type
    );
}

pub async fn setup_test_database() -> TestDatabase {
    let db = TestDatabase::new().await;
    db.run_migrations().await.unwrap();
    db
}
```

## Test Execution Strategy

### Local Development

**Fast Test Subset**:
```bash
# Unit tests only (fastest)
cargo test --lib

# Core integration tests
cargo test --test analysis_pipeline

# Specific module tests
cargo test analysis::detectors::god_object
```

**Full Test Suite**:
```bash
# All tests with coverage
cargo llvm-cov --all-features --workspace

# Performance tests
cargo bench

# Security audit
cargo audit
```

### Continuous Integration

**PR Validation Pipeline**:
1. **Fast Checks** (2-3 minutes)
   - Formatting and linting
   - Unit tests
   - Compilation check

2. **Comprehensive Testing** (8-12 minutes)
   - Integration tests
   - Multi-platform testing
   - Security audit

3. **Extended Validation** (15-20 minutes)
   - Performance benchmarks
   - E2E tests
   - Coverage analysis

**Matrix Testing**:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust: [stable, beta]
    features: [
      "default",
      "minimal",
      "analysis,local-ai",
      "all-features"
    ]
```

### Test Data Management

**Synthetic Data Generation**:
```rust
// src/bin/generate_benchmark_data.rs
pub fn generate_rust_project(size: ProjectSize) -> Vec<(PathBuf, String)> {
    match size {
        ProjectSize::Small => generate_files(10, 50),   // 10 files, 50 LOC each
        ProjectSize::Medium => generate_files(100, 100), // 100 files, 100 LOC each
        ProjectSize::Large => generate_files(1000, 200), // 1000 files, 200 LOC each
    }
}

fn generate_god_object(complexity: usize) -> String {
    let mut code = String::new();
    code.push_str("pub struct GodObject {\n");
    
    // Generate many fields
    for i in 0..complexity {
        code.push_str(&format!("    field_{}: i32,\n", i));
    }
    
    code.push_str("}\n\nimpl GodObject {\n");
    
    // Generate many methods
    for i in 0..complexity {
        code.push_str(&format!(
            "    pub fn method_{}(&self) -> i32 {{ self.field_{} }}\n", 
            i, i
        ));
    }
    
    code.push_str("}\n");
    code
}
```

## Test Coverage Strategy

### Coverage Targets

**Module-Level Targets**:
- Core analysis engine: 80%+
- CLI interface: 70%+
- AI integration: 60%+
- Database layer: 75%+
- Plugin system: 50%+ (experimental)

**Overall Project Target**: 25% minimum, 35% goal

### Coverage Analysis

**Tools**:
- `cargo-llvm-cov`: Primary coverage tool
- `codecov.io`: Coverage reporting and tracking
- `grcov`: Alternative coverage analysis

**Coverage Commands**:
```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# HTML report for local viewing
cargo llvm-cov --all-features --workspace --html

# Coverage for specific module
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info -- analysis::detectors
```

### Coverage Quality

**Focus Areas**:
- Critical path coverage (error handling, main workflows)
- Edge case coverage (boundary conditions, error states)
- Integration point coverage (module interfaces)

**Exclusions**:
- Generated code
- Test utilities
- Debug/development code
- Experimental features

## Testing Best Practices

### Test Design Principles

**FIRST Principles**:
- **Fast**: Tests should run quickly
- **Independent**: Tests should not depend on each other
- **Repeatable**: Tests should produce consistent results
- **Self-Validating**: Tests should have clear pass/fail criteria
- **Timely**: Tests should be written close to the code they test

### Common Patterns

**Arrange-Act-Assert**:
```rust
#[test]
fn test_dependency_cycle_detection() {
    // Arrange
    let graph = DependencyGraph::new();
    graph.add_edge("A", "B");
    graph.add_edge("B", "C");
    graph.add_edge("C", "A");  // Creates cycle
    
    // Act
    let cycles = detect_cycles(&graph);
    
    // Assert
    assert_eq!(cycles.len(), 1);
    assert!(cycles[0].contains(&"A".to_string()));
}
```

**Given-When-Then** (for integration tests):
```rust
#[tokio::test]
async fn test_analysis_with_ai_explanation() {
    // Given a project with god object
    let project = create_test_project(&[
        ("src/main.rs", GOD_OBJECT_CODE),
    ]);
    
    // When analysis is run with AI enabled
    let config = AnalysisConfig {
        enable_ai: true,
        ai_provider: Some("mock".to_string()),
    };
    let result = analyze_project(&project, config).await;
    
    // Then the result should include AI explanation
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.issues[0].ai_explanation.is_some());
}
```

### Error Testing

**Error Condition Testing**:
```rust
#[test]
fn test_malformed_rust_file_handling() {
    let malformed_code = "fn incomplete_function(";
    let result = parse_rust_code(malformed_code);
    
    match result {
        Err(ParseError::SyntaxError { line, column, .. }) => {
            assert!(line > 0);
            assert!(column > 0);
        }
        _ => panic!("Expected syntax error"),
    }
}

#[test]
fn test_network_timeout_handling() {
    let slow_provider = SlowMockProvider::new(Duration::from_secs(30));
    let config = AiConfig {
        timeout: Duration::from_secs(5),
        ..Default::default()
    };
    
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        slow_provider.generate_explanation("test")
    ).await;
    
    assert!(result.is_err()); // Should timeout
}
```

### Async Testing

**Async Test Patterns**:
```rust
#[tokio::test]
async fn test_concurrent_analysis() {
    let projects = vec![
        create_test_project(&[("main.rs", "fn main() {}")]),
        create_test_project(&[("lib.rs", "pub fn test() {}")]),
    ];
    
    let futures: Vec<_> = projects
        .into_iter()
        .map(|project| analyze_project_async(project))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    for result in results {
        assert!(result.is_ok());
    }
}
```

## Debugging and Troubleshooting Tests

### Test Debugging

**Debug Output**:
```rust
#[test]
fn test_with_debug_output() {
    env_logger::init();
    
    let result = complex_operation();
    
    // Use debug output for troubleshooting
    eprintln!("Debug: result = {:?}", result);
    
    assert!(result.is_ok());
}
```

**Conditional Testing**:
```rust
#[test]
#[cfg(feature = "expensive-tests")]
fn test_large_project_analysis() {
    // Only run with: cargo test --features expensive-tests
}

#[test]
#[ignore = "requires network access"]
fn test_real_ai_provider() {
    // Run with: cargo test -- --ignored
}
```

### Test Maintenance

**Regular Maintenance Tasks**:
- Review and update test fixtures
- Remove obsolete tests
- Refactor duplicated test code
- Update test documentation
- Monitor test execution times

**Test Metrics Monitoring**:
- Test execution time trends
- Test failure rates
- Coverage trends
- Flaky test identification

---

*This testing strategy ensures comprehensive validation of Uveddi's functionality while maintaining development velocity and code quality.*