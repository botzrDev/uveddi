# Error Handling Documentation

## Overview

Uveddi uses idiomatic Rust error handling with `Result<T, E>` types throughout the codebase. All errors are centralized through the `UveddiError` enum, which provides comprehensive error context, suggestions, and proper error propagation using the `?` operator.

## Core Error Types

### UveddiError

The main error type that encompasses all possible errors in the application:

```rust
use uveddi::error::{Result, UveddiError};

pub fn analyze_file(path: &Path) -> Result<AnalysisResult> {
    // Function returns Result<AnalysisResult, UveddiError>
    let content = std::fs::read_to_string(path)
        .map_err(|e| UveddiError::io_error("read", path.to_str().unwrap_or("unknown"), e))?;
    
    // Process content...
    Ok(result)
}
```

### Error Categories

| Category | Description | Severity | Example |
|----------|-------------|----------|---------|
| `Analysis` | Code analysis failures | High | AST parsing errors, detector failures |
| `Extraction` | File/data extraction issues | High | Unable to read source files |
| `Database` | Database operations | High | Connection failures, query errors |
| `Network` | Network requests | Medium | API timeouts, connection refused |
| `Configuration` | Config file issues | Low | Missing settings, invalid values |
| `IO` | File system operations | Low | Permission denied, file not found |
| `Security` | Security violations | High | SQL injection, access control |
| `Plugin` | Plugin system errors | High | WASM module failures |
| `Serialization` | JSON/data conversion | Low | Invalid JSON format |

## Error Flows

### 1. Analysis Flow

```rust
AnalysisOrchestrator::analyze()
    ├── FileExtractor::extract() -> Result<Vec<File>>
    │   └── Returns ExtractionError on failure
    ├── AnalysisEngine::analyze() -> Result<Vec<Issue>>
    │   ├── AstParser::parse() -> Result<AST>
    │   │   └── Returns AstError on parse failure
    │   └── Detector::detect() -> Result<Vec<Issue>>
    │       └── Returns AnalysisError on detection failure
    └── Database::store() -> Result<()>
        └── Returns DatabaseError on storage failure
```

### 2. API Request Flow

```rust
REST API Handler
    ├── Parse request -> Result<Request>
    │   └── Returns SerializationError on invalid JSON
    ├── Validate auth -> Result<()>
    │   └── Returns SecurityError on auth failure
    ├── Process request -> Result<Response>
    │   └── Returns appropriate error based on operation
    └── Send response -> Result<()>
        └── Returns NetworkError on send failure
```

### 3. CLI Command Flow

```rust
CLI Command
    ├── Parse arguments -> Result<Args>
    │   └── Returns CliError on invalid args
    ├── Load config -> Result<Config>
    │   └── Returns ConfigError on invalid config
    ├── Execute command -> Result<()>
    │   └── Propagates errors from underlying operations
    └── Report results -> Result<()>
        └── Returns ReportError on generation failure
```

## Error Propagation

### Using the ? Operator

```rust
fn complex_operation() -> Result<String> {
    // Automatically converts io::Error to UveddiError
    let content = std::fs::read_to_string("file.txt")?;
    
    // Propagates existing UveddiError
    let parsed = parse_content(&content)?;
    
    // Create custom error with context
    let result = process_data(parsed)
        .map_err(|e| UveddiError::analysis_error(
            "file.txt",
            42,
            &e.to_string(),
            "processing data"
        ))?;
    
    Ok(result)
}
```

### Error Context Enhancement

```rust
fn database_operation() -> Result<()> {
    database.execute(query)
        .map_err(|e| {
            // Add context to the error
            error!("Database operation failed: {}", e);
            UveddiError::database_error(
                "INSERT",
                "analysis.db",
                e
            )
        })?;
    Ok(())
}
```

## Best Practices

### 1. Never Use `unwrap()` or `expect()` in Production Code

**Bad:**
```rust
let config = load_config().unwrap();  // Will panic on error
```

**Good:**
```rust
let config = load_config()?;  // Propagates error properly
```

### 2. Provide Context When Creating Errors

**Bad:**
```rust
Err(UveddiError::GenericError { 
    message: "Failed".to_string(),
    context: "".to_string(),
    suggestion: "".to_string(),
    source: None
})
```

**Good:**
```rust
Err(UveddiError::analysis_error(
    "src/main.rs",
    142,
    "Failed to parse function declaration",
    "parsing Rust syntax tree"
))
```

### 3. Use Helper Functions for Common Errors

```rust
// For IO errors
UveddiError::io_error("read", "/path/to/file", io_error)

// For database errors
UveddiError::database_error("SELECT", "uveddi.db", db_error)

// For network errors
UveddiError::network_error("GET", "https://api.example.com", req_error)
```

### 4. Log Errors Before Propagating

```rust
fn risky_operation() -> Result<()> {
    match dangerous_call() {
        Ok(result) => process(result),
        Err(e) => {
            error!("Operation failed: {:#}", e);  // Log with context
            Err(e)  // Propagate the error
        }
    }
}
```

## Testing Error Conditions

### Unit Tests for Error Cases

```rust
#[test]
fn test_handles_missing_file() {
    let result = analyze_file(Path::new("/nonexistent/file"));
    assert!(result.is_err());
    
    match result.unwrap_err() {
        UveddiError::IoError { path, .. } => {
            assert_eq!(path, "/nonexistent/file");
        }
        _ => panic!("Expected IoError"),
    }
}

#[test]
fn test_handles_invalid_json() {
    let invalid_json = r#"{"incomplete": "#;
    let result = parse_json(invalid_json);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Serialization);
}
```

### Integration Tests for Error Flows

```rust
#[tokio::test]
async fn test_database_failure_recovery() {
    let orchestrator = AnalysisOrchestrator::new().unwrap();
    
    // Simulate database failure
    let result = orchestrator.analyze_with_failing_db().await;
    assert!(result.is_err());
    
    // Verify proper error type and recovery hint
    match result.unwrap_err() {
        UveddiError::DatabaseError { recovery_hint, .. } => {
            assert!(recovery_hint.contains("retry"));
        }
        _ => panic!("Expected DatabaseError"),
    }
}
```

## Error Recovery Strategies

### 1. Retry with Backoff

```rust
async fn retry_operation<F, T>(
    mut operation: F,
    max_retries: u32
) -> Result<T>
where
    F: FnMut() -> Result<T>
{
    let mut retries = 0;
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                warn!("Operation failed, retrying ({}/{}): {}", 
                      retries + 1, max_retries, e);
                tokio::time::sleep(Duration::from_secs(2_u64.pow(retries))).await;
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 2. Fallback Mechanisms

```rust
fn load_config() -> Result<Config> {
    // Try to load from file
    match Config::from_file("uveddi.toml") {
        Ok(config) => Ok(config),
        Err(e) => {
            warn!("Failed to load config file: {}, using defaults", e);
            Ok(Config::default())
        }
    }
}
```

### 3. Graceful Degradation

```rust
async fn analyze_with_ai(code: &str) -> Result<Analysis> {
    match ai_service.analyze(code).await {
        Ok(analysis) => Ok(analysis),
        Err(e) => {
            warn!("AI service unavailable: {}, falling back to basic analysis", e);
            basic_analyzer.analyze(code)
        }
    }
}
```

## Migration Guide

### Replacing `unwrap()` Calls

**Before:**
```rust
let conn = self.conn.lock().unwrap();
let result = conn.execute(query, params).unwrap();
```

**After:**
```rust
let conn = self.conn.lock()
    .map_err(|e| UveddiError::database_error_msg(
        &format!("Failed to acquire lock: {}", e)
    ))?;
let result = conn.execute(query, params)?;
```

### Replacing `expect()` Calls

**Before:**
```rust
let runtime = tokio::runtime::Runtime::new()
    .expect("Failed to create runtime");
```

**After:**
```rust
let runtime = tokio::runtime::Runtime::new()
    .map_err(|e| UveddiError::config_error(
        &format!("Failed to create tokio runtime: {}", e),
        "runtime configuration"
    ))?;
```

### Handling Panics in Default Implementations

For traits like `Default` that don't return `Result`:

```rust
impl Default for AnalysisEngine {
    /// Creates a default instance.
    /// 
    /// # Panics
    /// Panics if the engine cannot be created with default configuration.
    /// For production, use `AnalysisEngine::new()` which returns `Result`.
    fn default() -> Self {
        Self::new().expect(
            "Failed to create default AnalysisEngine - \
             indicates configuration or resource issue"
        )
    }
}
```

## Common Pitfalls

1. **Silent Error Swallowing**: Never ignore errors with `let _ = operation();`
2. **Generic Error Messages**: Always provide specific context
3. **Missing Error Logs**: Log errors before propagating
4. **Incorrect Error Types**: Use the appropriate `UveddiError` variant
5. **No Recovery Strategy**: Consider retry or fallback mechanisms

## Future Improvements

- [ ] Add error telemetry and monitoring
- [ ] Implement automatic error reporting
- [ ] Add error recovery middleware
- [ ] Create error dashboard for production monitoring
- [ ] Add distributed tracing for error flows