# Test Compilation and Stability Plan (UV-106)

## Current Test Issues Analysis
**Problem**: Tests fail to compile due to architectural drift  
**Root Causes**:
1. Missing AI feature gates in test files
2. Outdated API calls in test code
3. Architectural changes not reflected in tests
4. Complex dependency tree causing timeouts

## Test Stability Resolution Strategy

### Phase 4.1: Immediate Test Compilation Fixes (Week 2)

#### Issue 1: AI Feature Gate Problems
**Current Error**:
```
error[E0432]: unresolved import `uveddi::ai::prompts`
 --> tests/ai_explanations.rs:12:21
   |
12 |     use uveddi::ai::prompts::prompt_templates;
   |                     ^^^^^^^ could not find `prompts` in `ai`
```

**Solution**: Feature-gate test files properly

```rust
// tests/ai_explanations.rs
#[cfg(feature = "ai")]
mod ai_tests {
    use uveddi::ai::prompts::prompt_templates;
    use uveddi::ai::engine::AiAnalysisEngine;
    
    // Test implementation here
}

#[cfg(not(feature = "ai"))]
mod ai_tests {
    // Stub tests when AI feature is disabled
    #[test]
    fn ai_features_disabled() {
        println!("AI features are disabled in this build");
    }
}
```

#### Issue 2: Architectural Drift in Tests
**Problem**: Tests use deprecated APIs from old AnalysisEngine  
**Solution**: Update test files to use new component-based architecture

### Phase 4.2: Test Architecture Modernization (Week 2-3)

#### 1. Component-Based Test Structure
**New Test Organization**:
```
tests/
├── integration/
│   ├── analysis_orchestrator_tests.rs    # End-to-end analysis tests
│   ├── dependency_service_tests.rs       # Dependency analysis tests
│   └── performance_service_tests.rs      # Performance monitoring tests
├── unit/
│   ├── analysis_service_tests.rs         # Core analysis unit tests
│   ├── detector_tests.rs                 # Individual detector tests
│   └── cache_provider_tests.rs           # Cache functionality tests
└── security/
    ├── authentication_tests.rs           # Security-focused tests
    ├── secret_management_tests.rs        # Secret handling tests
    └── vulnerability_tests.rs            # Security regression tests
```

#### 2. Modern Test Utilities
**Create Test Helpers**:
```rust
// tests/common/mod.rs
pub struct TestEnvironment {
    temp_dir: TempDir,
    orchestrator: AnalysisOrchestrator,
    test_config: AnalysisConfig,
}

impl TestEnvironment {
    pub async fn new() -> TestResult<Self> {
        let temp_dir = TempDir::new()?;
        let test_config = AnalysisConfig::for_testing();
        
        let orchestrator = AnalysisOrchestratorBuilder::new()
            .with_test_configuration(&test_config)
            .with_memory_cache()
            .build().await?;
            
        Ok(Self { temp_dir, orchestrator, test_config })
    }
    
    pub async fn analyze_test_project(&self, project_name: &str) -> TestResult<AnalysisResults> {
        let project_path = self.temp_dir.path().join(project_name);
        self.orchestrator.analyze(&project_path).await
    }
}
```

### Phase 4.3: Test Performance Optimization (Week 3)

#### 1. Parallel Test Execution
**Problem**: Complex dependency tree causes test timeouts  
**Solution**: Optimize test execution strategy

```toml
# Cargo.toml test configuration
[profile.test]
opt-level = 1                    # Some optimization for faster test execution
debug = true                     # Keep debug info for better error messages
overflow-checks = true           # Safety checks
lto = false                     # No LTO for faster compilation
codegen-units = 256             # Faster parallel compilation
```

#### 2. Test Data Management
**Strategy**: Use lightweight test fixtures

```rust
// tests/fixtures/mod.rs
pub struct TestProjects;

impl TestProjects {
    /// Create minimal Rust project for testing
    pub fn minimal_rust_project() -> TestProject {
        TestProject::builder()
            .with_file("src/main.rs", r#"
                fn main() {
                    println!("Hello, world!");
                }
            "#)
            .with_file("Cargo.toml", r#"
                [package]
                name = "test-project"
                version = "0.1.0"
                edition = "2021"
            "#)
            .build()
    }
    
    /// Create project with God Object for detector testing
    pub fn god_object_project() -> TestProject {
        TestProject::builder()
            .with_file("src/lib.rs", include_str!("fixtures/god_object.rs"))
            .build()
    }
}
```

### Phase 4.4: Comprehensive Test Coverage (Week 3-4)

#### 1. Security Test Enhancement
**Focus**: Test the security fixes from Phase 1

```rust
// tests/security/timing_attack_tests.rs
#[cfg(test)]
mod timing_attack_tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    #[tokio::test]
    async fn test_jwt_timing_consistency() {
        let auth_service = create_test_auth_service().await;
        let valid_token = create_valid_jwt();
        let invalid_token = "invalid.jwt.token";
        
        // Measure timing for valid token
        let start = Instant::now();
        let _ = auth_service.authenticate_jwt(&valid_token).await;
        let valid_duration = start.elapsed();
        
        // Measure timing for invalid token  
        let start = Instant::now();
        let _ = auth_service.authenticate_jwt(&invalid_token).await;
        let invalid_duration = start.elapsed();
        
        // Timing should be consistent (within 10ms tolerance)
        let timing_diff = valid_duration.abs_diff(invalid_duration);
        assert!(timing_diff < Duration::from_millis(10), 
               "JWT validation timing variance too high: {:?}", timing_diff);
    }
}
```

#### 2. Architecture Test Coverage
**Focus**: Test the new component-based architecture

```rust  
// tests/integration/orchestrator_tests.rs
#[cfg(test)]
mod orchestrator_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_orchestrator_service_isolation() {
        let orchestrator = create_test_orchestrator().await;
        
        // Test that services can be mocked independently
        let mock_analysis = MockAnalysisService::new();
        let mock_dependency = MockDependencyService::new();
        
        let custom_orchestrator = AnalysisOrchestratorBuilder::new()
            .with_analysis_service(Arc::new(mock_analysis))
            .with_dependency_service(Arc::new(mock_dependency))
            .build().await.unwrap();
            
        // Verify isolated behavior
        let result = custom_orchestrator.analyze(Path::new("test")).await;
        assert!(result.is_ok());
    }
}
```

#### 3. Performance Regression Tests
**Focus**: Ensure refactoring doesn't impact performance

```rust
// tests/performance/regression_tests.rs
#[cfg(test)]
mod performance_tests {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_analysis_performance(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let orchestrator = rt.block_on(create_test_orchestrator());
        
        c.bench_function("analyze_medium_project", |b| {
            b.iter(|| {
                rt.block_on(orchestrator.analyze(black_box(test_project_path())))
            })
        });
    }
    
    criterion_group!(benches, benchmark_analysis_performance);
    criterion_main!(benches);
}
```

### Phase 4.5: Continuous Integration Enhancements (Week 4)

#### 1. Multi-Stage Test Pipeline
```yaml
# .github/workflows/comprehensive-testing.yml
name: Comprehensive Testing

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run unit tests
        run: cargo test --lib --bins

  integration-tests:
    needs: unit-tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run integration tests
        run: cargo test --test '*' --no-default-features

  security-tests:
    needs: unit-tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Security audit
        run: cargo audit
      - name: Run security tests
        run: cargo test --test security_tests

  performance-tests:
    needs: [unit-tests, integration-tests]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run performance benchmarks
        run: cargo bench --bench regression_tests
```

#### 2. Test Coverage Reporting
```toml
# Add to Cargo.toml
[dev-dependencies]
tarpaulin = "0.27"

# Coverage configuration
[package.metadata.tarpaulin]
exclude = ["tests/*", "benches/*", "build.rs"]
target_dir = "target/tarpaulin"
```

## Implementation Timeline

### Week 2: Immediate Test Fixes
- [ ] Fix AI feature gate issues in test files
- [ ] Update deprecated API calls in tests
- [ ] Create basic test environment utilities
- [ ] Resolve compilation errors

### Week 3: Test Architecture Modernization  
- [ ] Implement component-based test structure
- [ ] Create comprehensive test utilities
- [ ] Add security regression tests
- [ ] Optimize test performance

### Week 4: Enhanced Coverage and CI
- [ ] Achieve >90% test coverage for critical components
- [ ] Implement performance regression tests
- [ ] Set up multi-stage CI/CD pipeline
- [ ] Document testing best practices

## Success Metrics

### Quantitative Targets
- [ ] 100% test compilation success rate
- [ ] >90% code coverage for core components
- [ ] <30 second average test suite execution time
- [ ] Zero test timeouts or hangs

### Qualitative Improvements
- [ ] Clear test organization and structure
- [ ] Comprehensive security test coverage
- [ ] Reliable performance regression detection
- [ ] Maintainable and well-documented tests

## Risk Mitigation

### Technical Risks
- **Test complexity**: Start with simple cases, add complexity gradually
- **Performance impact**: Benchmark test execution times
- **Feature flag issues**: Comprehensive CI matrix testing

### Process Risks  
- **Development velocity**: Parallel test development with architecture changes
- **Maintenance burden**: Automated test generation where possible
- **Coverage gaps**: Mandatory coverage checks in CI/CD
