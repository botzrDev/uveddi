# Assignment 23: Improve Test Coverage

## Priority: HIGH
## Estimated Time: 5-6 hours
## Dependencies: All previous phases (especially interfaces and patterns)

## Objective
Significantly improve test coverage across the codebase, targeting 80%+ coverage with high-quality tests.

## Current Problem
- Test coverage at 36% (163 test files / 445 source files)
- Missing tests for critical components
- Inconsistent testing patterns
- Limited integration test coverage
- No performance or load testing

## Tasks

### 1. Audit Current Test Coverage

#### A. Generate Coverage Report:
```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Generate detailed coverage report
cargo tarpaulin --out Html --output-dir coverage --skip-clean

# Generate line-by-line coverage
cargo tarpaulin --out Lcov --output-dir coverage

# Analyze coverage by module
cargo tarpaulin --out Json | jq '.files[] | {name: .name, coverage: .coverage}'
```

#### B. Identify Coverage Gaps:
```bash
# Find files with no tests
find src -name "*.rs" | while read file; do
    test_file="${file/src\//tests/}"
    test_file="${test_file/.rs/_test.rs}"
    if [ ! -f "$test_file" ]; then
        echo "No test for: $file"
    fi
done

# Find large files with low coverage
find src -name "*.rs" -exec wc -l {} + | sort -nr | head -20
```

#### C. Categorize Testing Needs:
```markdown
# Test Coverage Analysis

## Critical Components (Must have >90% coverage):
- Error handling system
- Configuration management
- Core analysis engine
- Database operations
- Security-related code

## Important Components (Must have >80% coverage):
- Report generation
- Plugin system
- Cache management
- Interface implementations

## Standard Components (Must have >70% coverage):
- Utility functions
- Helper modules
- CLI commands
- API endpoints
```

### 2. Create Comprehensive Unit Tests

#### A. Test Framework Setup:
```rust
// tests/common/mod.rs
use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

pub mod fixtures {
    use crate::types::*;
    use std::path::PathBuf;

    pub fn sample_analysis_config() -> AnalysisConfig {
        AnalysisConfig {
            target_path: PathBuf::from("./test_data"),
            languages: vec![Language::Rust],
            detectors: vec!["god-object".to_string()],
            output_format: OutputFormat::Json,
            output_path: None,
            parallel_analysis: false,
            timeout: Duration::from_secs(30),
            max_file_size: 1024 * 1024,
            exclude_patterns: vec![],
            include_patterns: vec![],
            custom_rules: vec![],
            cache_enabled: false,
        }
    }

    pub fn sample_analysis_result() -> AnalysisResult {
        AnalysisResult {
            summary: AnalysisSummary {
                files_analyzed: 10,
                lines_analyzed: 1000,
                duration: Duration::from_secs(5),
                timestamp: chrono::Utc::now(),
            },
            findings: vec![
                Finding {
                    id: "1".to_string(),
                    severity: Severity::Warning,
                    message: "Sample finding".to_string(),
                    file_path: PathBuf::from("test.rs"),
                    line_number: Some(42),
                    column_number: Some(10),
                    rule_id: "sample-rule".to_string(),
                },
            ],
            metrics: AnalysisMetrics {
                complexity_score: 5.5,
                maintainability_index: 75.0,
                technical_debt_ratio: 0.15,
                test_coverage: 80.0,
            },
        }
    }

    pub fn sample_finding() -> Finding {
        Finding {
            id: "test-finding".to_string(),
            severity: Severity::Error,
            message: "Test finding message".to_string(),
            file_path: PathBuf::from("src/test.rs"),
            line_number: Some(100),
            column_number: Some(5),
            rule_id: "test-rule".to_string(),
        }
    }

    pub fn sample_database_config() -> DatabaseConfig {
        DatabaseConfig {
            connection_string: "sqlite::memory:".to_string(),
            max_connections: 1,
            min_connections: 1,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(60),
            ssl_mode: SslMode::Disable,
            migration_timeout: Duration::from_secs(30),
        }
    }
}

pub mod assertions {
    use crate::types::*;

    pub fn assert_analysis_result_valid(result: &AnalysisResult) {
        assert!(result.summary.files_analyzed > 0);
        assert!(result.summary.duration.as_secs() >= 0);
        assert!(!result.summary.timestamp.to_string().is_empty());

        for finding in &result.findings {
            assert!(!finding.id.is_empty());
            assert!(!finding.message.is_empty());
            assert!(!finding.rule_id.is_empty());
        }

        assert!(result.metrics.complexity_score >= 0.0);
        assert!(result.metrics.maintainability_index >= 0.0 && result.metrics.maintainability_index <= 100.0);
        assert!(result.metrics.technical_debt_ratio >= 0.0 && result.metrics.technical_debt_ratio <= 1.0);
    }

    pub fn assert_finding_valid(finding: &Finding) {
        assert!(!finding.id.is_empty());
        assert!(!finding.message.is_empty());
        assert!(!finding.rule_id.is_empty());
        assert!(finding.file_path.to_string_lossy().len() > 0);

        match finding.severity {
            Severity::Info | Severity::Warning | Severity::Error | Severity::Critical => (),
        }
    }

    pub fn assert_config_valid(config: &AnalysisConfig) {
        assert!(config.target_path.exists());
        assert!(!config.languages.is_empty());
        assert!(!config.detectors.is_empty());
        assert!(config.timeout.as_secs() > 0);
        assert!(config.max_file_size > 0);
    }
}
```

#### B. Error Handling Tests:
```rust
// tests/error_handling_test.rs
use uveddi::error::*;
use uveddi::types::*;

mod error_tests {
    use super::*;

    #[test]
    fn test_error_conversion_chain() {
        // Test io::Error -> UveddiError conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let uveddi_error: UveddiError = io_error.into();

        match uveddi_error {
            UveddiError::Io(_) => (),
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_analysis_error_construction() {
        let error = AnalysisError::UnsupportedLanguage {
            language: "invalid-lang".to_string(),
        };

        assert_eq!(error.to_string(), "Unsupported language: invalid-lang");
    }

    #[test]
    fn test_database_error_construction() {
        let error = DatabaseError::NotFound {
            id: "test-id".to_string(),
        };

        assert_eq!(error.to_string(), "Record not found: test-id");
    }

    #[test]
    fn test_validation_error_construction() {
        let error = ValidationError::FieldError {
            field: "test_field".to_string(),
            details: "Invalid value".to_string(),
        };

        assert_eq!(error.to_string(), "Field validation failed: test_field");
    }

    #[test]
    fn test_error_context_macro() {
        use uveddi::{context, Result};

        fn failing_operation() -> Result<()> {
            Err(UveddiError::Internal {
                message: "Original error".to_string(),
            })
        }

        let result = context!(failing_operation(), "Additional context");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Additional context"));
    }

    #[test]
    fn test_error_source_chain() {
        let root_cause = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let analysis_error = AnalysisError::ParseError {
            path: "test.rs".to_string(),
            source: Box::new(root_cause),
        };
        let top_error = UveddiError::Analysis(analysis_error);

        // Test error source chain
        let mut source = top_error.source();
        assert!(source.is_some());

        source = source.unwrap().source();
        assert!(source.is_some());

        let io_source = source.unwrap().downcast_ref::<std::io::Error>();
        assert!(io_source.is_some());
        assert_eq!(io_source.unwrap().kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_error_reporter() {
        let error = UveddiError::Analysis(AnalysisError::Timeout {
            timeout_seconds: 300,
        });

        let reporter = ErrorReporter::new(false, true);
        let formatted = reporter.format_error(&error);

        assert!(formatted.contains("Analysis timeout"));
        assert!(formatted.contains("300"));
    }

    #[test]
    fn test_user_friendly_error_messages() {
        let error = UveddiError::Config(ConfigError::InvalidFile {
            path: "invalid.toml".to_string(),
        });

        let reporter = ErrorReporter::new(false, false);
        let user_message = reporter.create_user_friendly_message(&error);

        assert!(user_message.contains("Configuration file"));
        assert!(user_message.contains("invalid.toml"));
        assert!(user_message.contains("check the file format"));
    }
}
```

#### C. Configuration Tests:
```rust
// tests/configuration_test.rs
use uveddi::config::*;
use uveddi::builders::*;
use uveddi::error::*;

mod config_tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_analysis_config_builder() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path();

        let config = AnalysisConfigBuilder::new()
            .target_path(target_path)
            .language(Language::Rust)
            .detector("test-detector")
            .timeout_seconds(60)
            .build()
            .expect("Failed to build config");

        assert_eq!(config.target_path, target_path);
        assert!(config.languages.contains(&Language::Rust));
        assert!(config.detectors.contains(&"test-detector".to_string()));
        assert_eq!(config.timeout.as_secs(), 60);
    }

    #[test]
    fn test_config_builder_validation() {
        // Test missing required field
        let result = AnalysisConfigBuilder::new()
            .language(Language::Rust)
            .build();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Target path is required"));
    }

    #[test]
    fn test_config_builder_validation_multiple_errors() {
        let mut builder = AnalysisConfigBuilder::new();

        // Multiple validation errors
        builder.validate().ok(); // This will populate validation_errors

        let errors = builder.validation_errors();
        assert!(errors.len() > 1);
        assert!(errors.iter().any(|e| e.contains("Target path is required")));
    }

    #[test]
    fn test_database_config_builder() {
        let config = DatabaseConfigBuilder::new()
            .sqlite(":memory:")
            .build()
            .expect("Failed to build database config");

        assert!(config.connection_string.contains("sqlite"));
        assert!(config.connection_string.contains(":memory:"));
    }

    #[test]
    fn test_database_config_postgresql() {
        let config = DatabaseConfigBuilder::new()
            .postgresql()
            .host("localhost")
            .port(5432)
            .database("test_db")
            .username("test_user")
            .password("test_pass")
            .build()
            .expect("Failed to build PostgreSQL config");

        assert!(config.connection_string.starts_with("postgresql://"));
        assert!(config.connection_string.contains("localhost:5432"));
        assert!(config.connection_string.contains("test_db"));
    }

    #[test]
    fn test_config_environment_loading() {
        // Test environment variable loading
        env::set_var("UVEDDI_TIMEOUT", "120");
        env::set_var("UVEDDI_MAX_FILE_SIZE", "5242880");

        let config = ConfigurationFacade::new()
            .expect("Failed to create configuration facade");

        let settings = config.get_analysis_settings();
        assert_eq!(settings.timeout_minutes, 2); // 120 seconds = 2 minutes
        assert_eq!(settings.max_file_size_mb, 5); // 5242880 bytes = 5MB

        // Clean up
        env::remove_var("UVEDDI_TIMEOUT");
        env::remove_var("UVEDDI_MAX_FILE_SIZE");
    }

    #[test]
    fn test_config_file_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let config_content = r#"
[analysis]
timeout_seconds = 180
max_file_size_mb = 20
parallel_analysis = true

[detectors]
enabled = ["god-object", "security", "performance"]

[database]
type = "sqlite"
path = ":memory:"
"#;

        fs::write(&config_path, config_content).unwrap();

        let mut facade = ConfigurationFacade::new().unwrap();
        facade.load_configuration(ConfigurationOptions {
            config_file: Some(config_path),
            use_environment: false,
            command_line_args: None,
        }).expect("Failed to load configuration");

        let settings = facade.get_analysis_settings();
        assert_eq!(settings.timeout_minutes, 3); // 180 seconds = 3 minutes
        assert_eq!(settings.max_file_size_mb, 20);
        assert!(settings.parallel_processing);
        assert_eq!(settings.enabled_detectors.len(), 3);
    }

    #[test]
    fn test_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent");

        let result = AnalysisConfigBuilder::new()
            .target_path(&invalid_path)
            .build();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn test_config_modes() {
        let temp_dir = TempDir::new().unwrap();

        // Test CI mode
        let ci_config = AnalysisConfigBuilder::new()
            .target_path(temp_dir.path())
            .ci_mode()
            .build()
            .unwrap();

        assert_eq!(ci_config.output_format, OutputFormat::Json);
        assert!(!ci_config.cache_enabled);
        assert!(ci_config.parallel_analysis);

        // Test development mode
        let dev_config = AnalysisConfigBuilder::new()
            .target_path(temp_dir.path())
            .dev_mode()
            .build()
            .unwrap();

        assert_eq!(dev_config.output_format, OutputFormat::Html);
        assert!(dev_config.cache_enabled);
        assert_eq!(dev_config.timeout.as_secs(), 120);
    }
}
```

### 3. Create Integration Tests

#### A. End-to-End Analysis Tests:
```rust
// tests/integration/analysis_integration_test.rs
use uveddi::facades::*;
use uveddi::container::*;
use uveddi::builders::*;
use uveddi::types::*;
use tempfile::TempDir;
use std::fs;

mod analysis_integration {
    use super::*;

    #[tokio::test]
    async fn test_full_analysis_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create test Rust file
        let test_file = src_dir.join("main.rs");
        fs::write(&test_file, r#"
fn main() {
    println!("Hello, world!");
}

struct LargeStruct {
    field1: String,
    field2: i32,
    field3: Vec<String>,
    field4: HashMap<String, String>,
    field5: bool,
    // ... many more fields to trigger god object detection
}

impl LargeStruct {
    fn method1(&self) {} fn method2(&self) {} fn method3(&self) {}
    fn method4(&self) {} fn method5(&self) {} fn method6(&self) {}
    fn method7(&self) {} fn method8(&self) {} fn method9(&self) {}
    fn method10(&self) {} fn method11(&self) {} fn method12(&self) {}
}
"#).unwrap();

        // Setup service container
        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        // Create analysis facade
        let analysis_facade = AnalysisFacade::new(&container).unwrap();

        // Run analysis
        let config = AnalysisConfigBuilder::new()
            .target_path(&src_dir)
            .language(Language::Rust)
            .detector("god-object")
            .timeout_seconds(30)
            .build()
            .unwrap();

        let result = analysis_facade.analyze_advanced(config).await.unwrap();

        // Verify results
        assert!(result.summary.files_analyzed > 0);
        assert!(result.summary.duration.as_secs() < 30);
        assert!(!result.findings.is_empty()); // Should detect god object

        // Verify god object detection
        let god_object_findings: Vec<_> = result.findings
            .iter()
            .filter(|f| f.rule_id.contains("god-object"))
            .collect();
        assert!(!god_object_findings.is_empty());
    }

    #[tokio::test]
    async fn test_multi_language_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create Rust file
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        // Create Python file
        fs::write(src_dir.join("script.py"), r#"
class LargeClass:
    def __init__(self):
        self.data = {}

    def method1(self): pass
    def method2(self): pass
    def method3(self): pass
    def method4(self): pass
    def method5(self): pass
    def method6(self): pass
    def method7(self): pass
    def method8(self): pass
    def method9(self): pass
    def method10(self): pass
"#).unwrap();

        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        let analysis_facade = AnalysisFacade::new(&container).unwrap();

        let config = AnalysisConfigBuilder::new()
            .target_path(&src_dir)
            .auto_detect_languages()
            .all_detectors()
            .build()
            .unwrap();

        let result = analysis_facade.analyze_advanced(config).await.unwrap();

        assert_eq!(result.summary.files_analyzed, 2);
        assert!(result.findings.iter().any(|f| f.file_path.to_string_lossy().contains("main.rs")));
        assert!(result.findings.iter().any(|f| f.file_path.to_string_lossy().contains("script.py")));
    }

    #[tokio::test]
    async fn test_analysis_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("test.rs"), "fn test() {}").unwrap();

        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        let analysis_facade = AnalysisFacade::new(&container).unwrap();

        let config = AnalysisConfigBuilder::new()
            .target_path(&src_dir)
            .language(Language::Rust)
            .cache_enabled(true)
            .build()
            .unwrap();

        // First analysis - should populate cache
        let start = std::time::Instant::now();
        let result1 = analysis_facade.analyze_advanced(config.clone()).await.unwrap();
        let first_duration = start.elapsed();

        // Second analysis - should use cache
        let start = std::time::Instant::now();
        let result2 = analysis_facade.analyze_advanced(config).await.unwrap();
        let second_duration = start.elapsed();

        // Results should be identical
        assert_eq!(result1.summary.files_analyzed, result2.summary.files_analyzed);
        assert_eq!(result1.findings.len(), result2.findings.len());

        // Second analysis should be faster (using cache)
        assert!(second_duration < first_duration);
    }

    #[tokio::test]
    async fn test_analysis_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        let analysis_facade = AnalysisFacade::new(&container).unwrap();

        let config = AnalysisConfigBuilder::new()
            .target_path(&nonexistent_dir)
            .language(Language::Rust)
            .build();

        // Should fail due to nonexistent directory
        assert!(config.is_err());
    }

    #[tokio::test]
    async fn test_analysis_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create many files to potentially trigger timeout
        for i in 0..100 {
            fs::write(src_dir.join(format!("file_{}.rs", i)), format!(
                "fn function_{}() {{ /* file {} */ }}", i, i
            )).unwrap();
        }

        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        let analysis_facade = AnalysisFacade::new(&container).unwrap();

        let config = AnalysisConfigBuilder::new()
            .target_path(&src_dir)
            .language(Language::Rust)
            .timeout_seconds(1) // Very short timeout
            .build()
            .unwrap();

        let result = analysis_facade.analyze_advanced(config).await;

        // Should either complete quickly or timeout gracefully
        match result {
            Ok(analysis_result) => {
                assert!(analysis_result.summary.duration.as_secs() <= 2);
            }
            Err(e) => {
                assert!(e.to_string().contains("timeout") || e.to_string().contains("Timeout"));
            }
        }
    }
}
```

#### B. Database Integration Tests:
```rust
// tests/integration/database_integration_test.rs
use uveddi::adapters::database::*;
use uveddi::interfaces::*;
use uveddi::types::*;
use uveddi::error::*;

mod database_integration {
    use super::*;
    use tests::common::fixtures::*;

    #[tokio::test]
    async fn test_sqlite_adapter() {
        let adapter = SqlxAdapter::new_sqlite(":memory:").await.unwrap();

        // Test basic operations
        adapter.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();
        adapter.execute("INSERT INTO test (name) VALUES ('test1')").await.unwrap();
        adapter.execute("INSERT INTO test (name) VALUES ('test2')").await.unwrap();

        // Test query
        let results: Vec<serde_json::Value> = adapter
            .query("SELECT * FROM test")
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_database_transaction() {
        let adapter = SqlxAdapter::new_sqlite(":memory:").await.unwrap();

        adapter.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)").await.unwrap();

        // Test successful transaction
        let result = adapter.transaction(|tx| {
            Box::pin(async move {
                tx.execute("INSERT INTO test (value) VALUES ('success')").await?;
                Ok("transaction completed")
            })
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "transaction completed");

        // Verify data was committed
        let count: Vec<serde_json::Value> = adapter
            .query("SELECT COUNT(*) as count FROM test")
            .await
            .unwrap();
        assert_eq!(count[0]["count"], 1);

        // Test failed transaction (rollback)
        let result = adapter.transaction(|tx| {
            Box::pin(async move {
                tx.execute("INSERT INTO test (value) VALUES ('will_rollback')").await?;
                // Force an error
                Err(UveddiError::Internal {
                    message: "Test error".to_string(),
                })
            })
        }).await;

        assert!(result.is_err());

        // Verify data was rolled back
        let count: Vec<serde_json::Value> = adapter
            .query("SELECT COUNT(*) as count FROM test")
            .await
            .unwrap();
        assert_eq!(count[0]["count"], 1); // Still only 1 record
    }

    #[tokio::test]
    async fn test_database_error_handling() {
        let adapter = SqlxAdapter::new_sqlite(":memory:").await.unwrap();

        // Test invalid SQL
        let result = adapter.execute("INVALID SQL STATEMENT").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UveddiError::Database(DatabaseError::QueryFailed { query, .. }) => {
                assert!(query.contains("INVALID SQL"));
            }
            _ => panic!("Expected QueryFailed error"),
        }

        // Test query on non-existent table
        let result: Result<Vec<serde_json::Value>> = adapter
            .query("SELECT * FROM nonexistent_table")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analysis_repository_implementation() {
        use uveddi::repositories::SqliteAnalysisRepository;

        let config = sample_database_config();
        let repository = SqliteAnalysisRepository::new(&config).await.unwrap();

        let analysis_result = sample_analysis_result();

        // Test save and load
        let analysis_id = repository.save(&analysis_result).await.unwrap();
        assert!(!analysis_id.is_empty());

        let loaded_result = repository.load(&analysis_id).await.unwrap();
        assert_eq!(loaded_result.summary.files_analyzed, analysis_result.summary.files_analyzed);
        assert_eq!(loaded_result.findings.len(), analysis_result.findings.len());

        // Test list
        let filter = AnalysisFilter::default();
        let analyses = repository.list(&filter).await.unwrap();
        assert_eq!(analyses.len(), 1);
        assert_eq!(analyses[0].id, analysis_id);

        // Test delete
        repository.delete(&analysis_id).await.unwrap();

        let result = repository.load(&analysis_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UveddiError::Database(DatabaseError::NotFound { .. }) => (),
            _ => panic!("Expected NotFound error"),
        }
    }
}
```

### 4. Create Performance Tests

#### A. Benchmark Tests:
```rust
// benches/analysis_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use uveddi::facades::*;
use uveddi::builders::*;
use uveddi::container::*;
use std::fs;
use tempfile::TempDir;

fn create_test_project(size: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    for i in 0..size {
        let content = format!(r#"
pub struct TestStruct_{} {{
    field1: String,
    field2: i32,
    field3: Vec<String>,
}}

impl TestStruct_{} {{
    pub fn new() -> Self {{
        Self {{
            field1: String::new(),
            field2: 0,
            field3: Vec::new(),
        }}
    }}

    pub fn method1(&self) -> String {{
        self.field1.clone()
    }}

    pub fn method2(&self) -> i32 {{
        self.field2
    }}

    pub fn method3(&self) -> &Vec<String> {{
        &self.field3
    }}
}}

pub fn function_{}() {{
    let _struct = TestStruct_{}::new();
}}
"#, i, i, i, i);

        fs::write(src_dir.join(format!("module_{}.rs", i)), content).unwrap();
    }

    temp_dir
}

fn bench_analysis_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("analysis_performance");

    for size in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("file_count", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = create_test_project(size);

                    let container = ServiceContainerBuilder::new()
                        .test_environment()
                        .unwrap()
                        .build()
                        .await
                        .unwrap();

                    let facade = AnalysisFacade::new(&container).unwrap();

                    let config = AnalysisConfigBuilder::new()
                        .target_path(temp_dir.path().join("src"))
                        .language(Language::Rust)
                        .detector("god-object")
                        .cache_enabled(false) // Disable cache for consistent benchmarks
                        .build()
                        .unwrap();

                    facade.analyze_advanced(config).await.unwrap()
                });
            },
        );
    }

    group.finish();
}

fn bench_detector_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let temp_dir = create_test_project(50);
    let container = rt.block_on(async {
        ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap()
    });

    let facade = AnalysisFacade::new(&container).unwrap();

    let mut group = c.benchmark_group("detector_performance");

    let detectors = vec![
        "god-object",
        "security",
        "code-quality",
        "performance",
    ];

    for detector in detectors {
        group.bench_with_input(
            BenchmarkId::new("detector", detector),
            &detector,
            |b, &detector| {
                b.to_async(&rt).iter(|| async {
                    let config = AnalysisConfigBuilder::new()
                        .target_path(temp_dir.path().join("src"))
                        .language(Language::Rust)
                        .detector(detector)
                        .cache_enabled(false)
                        .build()
                        .unwrap();

                    facade.analyze_advanced(config).await.unwrap()
                });
            },
        );
    }

    group.finish();
}

fn bench_parallel_vs_sequential(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = create_test_project(100);

    let container = rt.block_on(async {
        ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap()
    });

    let facade = AnalysisFacade::new(&container).unwrap();

    let mut group = c.benchmark_group("parallel_vs_sequential");

    group.bench_function("parallel", |b| {
        b.to_async(&rt).iter(|| async {
            let config = AnalysisConfigBuilder::new()
                .target_path(temp_dir.path().join("src"))
                .language(Language::Rust)
                .all_detectors()
                .parallel_analysis(true)
                .cache_enabled(false)
                .build()
                .unwrap();

            facade.analyze_advanced(config).await.unwrap()
        });
    });

    group.bench_function("sequential", |b| {
        b.to_async(&rt).iter(|| async {
            let config = AnalysisConfigBuilder::new()
                .target_path(temp_dir.path().join("src"))
                .language(Language::Rust)
                .all_detectors()
                .parallel_analysis(false)
                .cache_enabled(false)
                .build()
                .unwrap();

            facade.analyze_advanced(config).await.unwrap()
        });
    });

    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = create_test_project(50);

    let container = rt.block_on(async {
        ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap()
    });

    let facade = AnalysisFacade::new(&container).unwrap();

    let mut group = c.benchmark_group("cache_performance");

    let config = AnalysisConfigBuilder::new()
        .target_path(temp_dir.path().join("src"))
        .language(Language::Rust)
        .all_detectors()
        .cache_enabled(true)
        .build()
        .unwrap();

    // Warm up cache
    rt.block_on(async {
        facade.analyze_advanced(config.clone()).await.unwrap();
    });

    group.bench_function("with_cache", |b| {
        b.to_async(&rt).iter(|| async {
            facade.analyze_advanced(config.clone()).await.unwrap()
        });
    });

    let config_no_cache = AnalysisConfigBuilder::new()
        .target_path(temp_dir.path().join("src"))
        .language(Language::Rust)
        .all_detectors()
        .cache_enabled(false)
        .build()
        .unwrap();

    group.bench_function("without_cache", |b| {
        b.to_async(&rt).iter(|| async {
            facade.analyze_advanced(config_no_cache.clone()).await.unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_analysis_performance,
    bench_detector_performance,
    bench_parallel_vs_sequential,
    bench_cache_performance
);
criterion_main!(benches);
```

### 5. Create Load Tests

#### A. Stress Testing:
```rust
// tests/load/stress_test.rs
use uveddi::facades::*;
use uveddi::builders::*;
use uveddi::container::*;
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use std::fs;

mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_analysis_load() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create test files
        for i in 0..20 {
            fs::write(src_dir.join(format!("test_{}.rs", i)), format!(
                "fn test_function_{}() {{ /* test content */ }}", i
            )).unwrap();
        }

        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        let facade = Arc::new(AnalysisFacade::new(&container).unwrap());

        // Run 10 concurrent analyses
        let semaphore = Arc::new(Semaphore::new(10));
        let mut handles = vec![];

        let start_time = Instant::now();

        for i in 0..10 {
            let facade = facade.clone();
            let semaphore = semaphore.clone();
            let src_dir = src_dir.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                let config = AnalysisConfigBuilder::new()
                    .target_path(&src_dir)
                    .language(Language::Rust)
                    .detector("code-quality")
                    .timeout_seconds(60)
                    .cache_enabled(false) // Disable cache to test real load
                    .build()
                    .unwrap();

                let result = facade.analyze_advanced(config).await;
                (i, result)
            });

            handles.push(handle);
        }

        let mut success_count = 0;
        let mut error_count = 0;

        for handle in handles {
            let (task_id, result) = handle.await.unwrap();
            match result {
                Ok(analysis_result) => {
                    success_count += 1;
                    assert!(analysis_result.summary.files_analyzed > 0);
                    println!("Task {} completed successfully", task_id);
                }
                Err(e) => {
                    error_count += 1;
                    println!("Task {} failed: {}", task_id, e);
                }
            }
        }

        let total_time = start_time.elapsed();
        println!("Concurrent load test completed in {:?}", total_time);
        println!("Success: {}, Errors: {}", success_count, error_count);

        // At least 80% should succeed
        assert!(success_count >= 8);
        assert!(total_time < Duration::from_secs(120)); // Should complete within 2 minutes
    }

    #[tokio::test]
    async fn test_memory_usage_under_load() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create larger test files
        for i in 0..100 {
            let content = (0..1000)
                .map(|j| format!("fn function_{}_{}_{}() {{}}", i, j, j * 2))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(src_dir.join(format!("large_{}.rs", i)), content).unwrap();
        }

        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        let facade = AnalysisFacade::new(&container).unwrap();

        // Monitor memory usage during analysis
        let config = AnalysisConfigBuilder::new()
            .target_path(&src_dir)
            .language(Language::Rust)
            .all_detectors()
            .parallel_analysis(true)
            .timeout_seconds(300)
            .max_file_size_mb(10)
            .build()
            .unwrap();

        let start_memory = get_memory_usage();
        let start_time = Instant::now();

        let result = facade.analyze_advanced(config).await.unwrap();

        let end_time = Instant::now();
        let end_memory = get_memory_usage();

        println!("Memory usage: {} -> {} KB", start_memory, end_memory);
        println!("Analysis time: {:?}", end_time - start_time);
        println!("Files analyzed: {}", result.summary.files_analyzed);

        // Memory usage should be reasonable (less than 1GB)
        let memory_increase = end_memory.saturating_sub(start_memory);
        assert!(memory_increase < 1024 * 1024); // Less than 1GB increase

        // Should complete in reasonable time
        assert!(end_time - start_time < Duration::from_secs(300));

        // Should analyze all files
        assert_eq!(result.summary.files_analyzed, 100);
    }

    #[tokio::test]
    async fn test_database_connection_pool_under_load() {
        let container = ServiceContainerBuilder::new()
            .test_environment()
            .unwrap()
            .build()
            .await
            .unwrap();

        let repository = container.resolve::<dyn AnalysisRepository>();

        // Create sample analysis results
        let sample_result = tests::common::fixtures::sample_analysis_result();

        // Run concurrent database operations
        let mut handles = vec![];

        for i in 0..20 {
            let repository = repository.clone();
            let mut result = sample_result.clone();
            result.summary.files_analyzed = i + 1; // Make each result unique

            let handle = tokio::spawn(async move {
                // Save analysis
                let analysis_id = repository.save(&result).await?;

                // Load it back
                let loaded = repository.load(&analysis_id).await?;

                // List analyses
                let filter = AnalysisFilter::default();
                let _analyses = repository.list(&filter).await?;

                // Delete analysis
                repository.delete(&analysis_id).await?;

                Ok::<_, uveddi::error::UveddiError>(())
            });

            handles.push(handle);
        }

        let mut success_count = 0;
        for handle in handles {
            match handle.await.unwrap() {
                Ok(_) => success_count += 1,
                Err(e) => println!("Database operation failed: {}", e),
            }
        }

        // All operations should succeed
        assert_eq!(success_count, 20);
    }

    fn get_memory_usage() -> u64 {
        // Simple memory usage check (Linux-specific)
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        return kb_str.parse().unwrap_or(0);
                    }
                }
            }
        }
        0 // Fallback for non-Linux systems
    }
}
```

### 6. Create Property-Based Tests

#### A. Property Testing with Proptest:
```rust
// tests/property/property_test.rs
use proptest::prelude::*;
use uveddi::types::*;
use uveddi::builders::*;
use uveddi::error::*;
use std::path::PathBuf;

mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_analysis_config_builder_properties(
            timeout_seconds in 10u64..3600,
            max_file_size_mb in 1usize..1000,
            parallel in any::<bool>(),
            cache_enabled in any::<bool>(),
        ) {
            let temp_dir = tempfile::TempDir::new().unwrap();

            let config = AnalysisConfigBuilder::new()
                .target_path(temp_dir.path())
                .timeout_seconds(timeout_seconds)
                .max_file_size_mb(max_file_size_mb)
                .parallel_analysis(parallel)
                .cache_enabled(cache_enabled)
                .language(Language::Rust)
                .detector("test-detector")
                .build();

            prop_assert!(config.is_ok());

            let config = config.unwrap();
            prop_assert_eq!(config.timeout.as_secs(), timeout_seconds);
            prop_assert_eq!(config.max_file_size, max_file_size_mb * 1024 * 1024);
            prop_assert_eq!(config.parallel_analysis, parallel);
            prop_assert_eq!(config.cache_enabled, cache_enabled);
        }

        #[test]
        fn test_finding_validation_properties(
            id in "[a-zA-Z0-9-_]{1,50}",
            message in "[a-zA-Z0-9 .,!?-_]{1,200}",
            rule_id in "[a-zA-Z0-9-_]{1,30}",
            line_number in 1u32..10000,
            column_number in 1u32..1000,
        ) {
            let finding = Finding {
                id,
                severity: Severity::Warning,
                message: message.clone(),
                file_path: PathBuf::from("test.rs"),
                line_number: Some(line_number),
                column_number: Some(column_number),
                rule_id: rule_id.clone(),
            };

            // Properties that should always hold
            prop_assert!(!finding.id.is_empty());
            prop_assert!(!finding.message.is_empty());
            prop_assert!(!finding.rule_id.is_empty());
            prop_assert!(finding.line_number.unwrap() > 0);
            prop_assert!(finding.column_number.unwrap() > 0);

            // Test serialization roundtrip
            let serialized = serde_json::to_string(&finding).unwrap();
            let deserialized: Finding = serde_json::from_str(&serialized).unwrap();
            prop_assert_eq!(finding.id, deserialized.id);
            prop_assert_eq!(finding.message, deserialized.message);
        }

        #[test]
        fn test_analysis_metrics_properties(
            complexity_score in 0.0f64..100.0,
            maintainability_index in 0.0f64..100.0,
            technical_debt_ratio in 0.0f64..1.0,
            test_coverage in 0.0f64..100.0,
        ) {
            let metrics = AnalysisMetrics {
                complexity_score,
                maintainability_index,
                technical_debt_ratio,
                test_coverage,
            };

            // Validate ranges
            prop_assert!(metrics.complexity_score >= 0.0 && metrics.complexity_score <= 100.0);
            prop_assert!(metrics.maintainability_index >= 0.0 && metrics.maintainability_index <= 100.0);
            prop_assert!(metrics.technical_debt_ratio >= 0.0 && metrics.technical_debt_ratio <= 1.0);
            prop_assert!(metrics.test_coverage >= 0.0 && metrics.test_coverage <= 100.0);

            // Test that metrics can be serialized and deserialized
            let serialized = serde_json::to_string(&metrics).unwrap();
            let deserialized: AnalysisMetrics = serde_json::from_str(&serialized).unwrap();

            prop_assert!((metrics.complexity_score - deserialized.complexity_score).abs() < 0.001);
            prop_assert!((metrics.maintainability_index - deserialized.maintainability_index).abs() < 0.001);
        }

        #[test]
        fn test_error_message_properties(
            field_name in "[a-zA-Z_][a-zA-Z0-9_]{0,30}",
            details in ".{1,500}",
        ) {
            let error = ValidationError::FieldError {
                field: field_name.clone(),
                details: details.clone(),
            };

            let error_string = error.to_string();

            // Error message should contain field name
            prop_assert!(error_string.contains(&field_name));

            // Error message should be non-empty
            prop_assert!(!error_string.is_empty());

            // Error should be convertible to UveddiError
            let uveddi_error = UveddiError::Validation(error);
            prop_assert!(!uveddi_error.to_string().is_empty());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))] // Fewer cases for expensive tests

        #[test]
        fn test_config_builder_validation_properties(
            timeout_seconds in 1u64..10,
            max_file_size_bytes in 1usize..1000,
        ) {
            // Test that invalid configurations are properly rejected

            let temp_dir = tempfile::TempDir::new().unwrap();

            let mut builder = AnalysisConfigBuilder::new()
                .target_path(temp_dir.path())
                .timeout_seconds(timeout_seconds)
                .max_file_size(max_file_size_bytes);

            // Very small timeout should be rejected in validation
            if timeout_seconds < 10 {
                prop_assert!(!builder.is_valid());
            }

            // Very small file size should be rejected
            if max_file_size_bytes < 1024 {
                prop_assert!(!builder.is_valid());
            }
        }
    }
}
```

## Success Criteria
- [ ] Test coverage increased to 80%+
- [ ] All critical components have comprehensive tests
- [ ] Integration tests cover end-to-end workflows
- [ ] Performance benchmarks established
- [ ] Load testing validates system stability
- [ ] Property-based tests validate invariants
- [ ] Mock implementations enable isolated testing

## Coverage Targets by Component
- **Error handling**: >95%
- **Configuration**: >90%
- **Core analysis**: >85%
- **Database operations**: >85%
- **Interface implementations**: >80%
- **Utility functions**: >75%

## Verification Commands
```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Run all tests
cargo test --all-features

# Run integration tests
cargo test --test integration

# Run benchmarks
cargo bench

# Run property tests
cargo test property

# Run load tests
cargo test --release --test stress_test
```

## Completion Notes
_To be filled by AI developer:_
- Test coverage before: ___% after: ___%
- Tests added: ___
- Integration test scenarios: ___
- Performance benchmarks: ___
- Load test results: ___