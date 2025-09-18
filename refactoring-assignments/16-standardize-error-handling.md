# Assignment 16: Standardize Error Handling

## Priority: HIGH
## Estimated Time: 3-4 hours
## Files: Throughout codebase

## Objective
Implement consistent error handling strategy across the entire codebase to improve debugging and user experience.

## Current Problem
- Multiple error handling approaches (anyhow, thiserror, custom errors)
- Inconsistent error messages and context
- Poor error propagation and debugging information
- Different error types used in different modules

## Tasks

### 1. Audit Current Error Handling

#### A. Identify Error Patterns:
```bash
# Find different error handling approaches
rg "use anyhow" src/ --type rust
rg "use thiserror" src/ --type rust
rg "Result<.*Error>" src/ --type rust
rg "panic!" src/ --type rust

# Find custom error types
rg "enum.*Error" src/ --type rust -A 5

# Find error conversions
rg "\.map_err\(" src/ --type rust
rg "From<.*> for.*Error" src/ --type rust
```

#### B. Categorize Errors:
```markdown
# Error Audit Results

## Error Libraries Used:
- anyhow: ___ files
- thiserror: ___ files
- std::error: ___ files
- Custom errors: ___ files

## Error Categories Found:
- I/O errors (file operations)
- Parse errors (AST, configuration)
- Network errors (API calls)
- Database errors (persistence)
- Validation errors (user input)
- Business logic errors (analysis failures)
```

### 2. Design Unified Error System

#### A. Define Error Hierarchy:
```rust
// src/error/mod.rs
use thiserror::Error;

/// Top-level error type for all Uveddi operations
#[derive(Error, Debug)]
pub enum UveddiError {
    #[error("I/O operation failed")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Feature not available: {feature}")]
    FeatureNotAvailable { feature: String },
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration file: {path}")]
    InvalidFile { path: String },

    #[error("Missing required configuration: {key}")]
    MissingRequired { key: String },

    #[error("Invalid value for {key}: {value}")]
    InvalidValue { key: String, value: String },

    #[error("Configuration validation failed: {details}")]
    ValidationFailed { details: String },
}

/// Analysis-related errors
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Failed to parse file: {path}")]
    ParseError { path: String, source: Box<dyn std::error::Error + Send + Sync> },

    #[error("Unsupported language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("Analysis timeout after {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },

    #[error("Memory limit exceeded: {used_mb}MB > {limit_mb}MB")]
    MemoryLimitExceeded { used_mb: u64, limit_mb: u64 },

    #[error("Detector failed: {detector_name}")]
    DetectorFailed { detector_name: String, source: Box<dyn std::error::Error + Send + Sync> },
}

/// Database-related errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {details}")]
    ConnectionFailed { details: String },

    #[error("Query failed: {query}")]
    QueryFailed { query: String, source: Box<dyn std::error::Error + Send + Sync> },

    #[error("Migration failed: {version}")]
    MigrationFailed { version: String, source: Box<dyn std::error::Error + Send + Sync> },

    #[error("Transaction failed")]
    TransactionFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Record not found: {id}")]
    NotFound { id: String },
}

/// Plugin-related errors
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {name}")]
    NotFound { name: String },

    #[error("Plugin load failed: {path}")]
    LoadFailed { path: String, source: Box<dyn std::error::Error + Send + Sync> },

    #[error("Plugin execution failed: {name}")]
    ExecutionFailed { name: String, source: Box<dyn std::error::Error + Send + Sync> },

    #[error("Plugin API version mismatch: expected {expected}, got {actual}")]
    ApiVersionMismatch { expected: String, actual: String },

    #[error("Plugin security violation: {details}")]
    SecurityViolation { details: String },
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("HTTP request failed: {url}")]
    HttpError { url: String, source: Box<dyn std::error::Error + Send + Sync> },

    #[error("Timeout connecting to {url}")]
    Timeout { url: String },

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[error("Authentication failed")]
    AuthenticationFailed,
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Field validation failed: {field}")]
    FieldError { field: String, details: String },

    #[error("Schema validation failed")]
    SchemaError { source: Box<dyn std::error::Error + Send + Sync> },

    #[error("Range error: {value} not in range {min}..{max}")]
    RangeError { value: String, min: String, max: String },
}
```

### 3. Implement Error Context System

#### A. Error Context Trait:
```rust
// src/error/context.rs
pub trait ErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T, UveddiError>
    where
        F: FnOnce() -> String;

    fn with_context_lazy<F>(self, f: F) -> Result<T, UveddiError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<UveddiError>,
{
    fn with_context<F>(self, f: F) -> Result<T, UveddiError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                UveddiError::Internal { message } => UveddiError::Internal {
                    message: format!("{}: {}", f(), message),
                },
                other => other,
            }
        })
    }

    fn with_context_lazy<F>(self, f: F) -> Result<T, UveddiError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            let base_error = e.into();
            // Add context to error
            match base_error {
                UveddiError::Analysis(AnalysisError::ParseError { path, source }) => {
                    UveddiError::Analysis(AnalysisError::ParseError {
                        path: format!("{} ({})", path, context),
                        source,
                    })
                }
                other => other,
            }
        })
    }
}

// Convenience macros
#[macro_export]
macro_rules! context {
    ($expr:expr, $msg:expr) => {
        $expr.with_context(|| $msg.to_string())
    };
    ($expr:expr, $fmt:expr, $($arg:tt)*) => {
        $expr.with_context(|| format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into())
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(UveddiError::Internal {
            message: format!($fmt, $($arg)*)
        })
    };
}
```

### 4. Implement Error Conversion Layer

#### A. From Implementations:
```rust
// src/error/conversions.rs
use sqlx::Error as SqlxError;
use reqwest::Error as ReqwestError;
use serde_json::Error as JsonError;

impl From<SqlxError> for DatabaseError {
    fn from(err: SqlxError) -> Self {
        match err {
            SqlxError::RowNotFound => DatabaseError::NotFound {
                id: "unknown".to_string(),
            },
            SqlxError::Database(db_err) => DatabaseError::QueryFailed {
                query: "unknown".to_string(),
                source: Box::new(db_err),
            },
            other => DatabaseError::ConnectionFailed {
                details: other.to_string(),
            },
        }
    }
}

impl From<ReqwestError> for NetworkError {
    fn from(err: ReqwestError) -> Self {
        if err.is_timeout() {
            NetworkError::Timeout {
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
            }
        } else {
            NetworkError::HttpError {
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                source: Box::new(err),
            }
        }
    }
}

impl From<JsonError> for ConfigError {
    fn from(err: JsonError) -> Self {
        ConfigError::ValidationFailed {
            details: err.to_string(),
        }
    }
}

impl From<tree_sitter::Error> for AnalysisError {
    fn from(err: tree_sitter::Error) -> Self {
        AnalysisError::ParseError {
            path: "unknown".to_string(),
            source: Box::new(err),
        }
    }
}
```

### 5. Create Error Reporting System

#### A. Error Reporter:
```rust
// src/error/reporter.rs
use std::fmt::Write;

pub struct ErrorReporter {
    include_backtrace: bool,
    include_context: bool,
}

impl ErrorReporter {
    pub fn new(include_backtrace: bool, include_context: bool) -> Self {
        Self {
            include_backtrace,
            include_context,
        }
    }

    pub fn format_error(&self, error: &UveddiError) -> String {
        let mut output = String::new();

        // Main error message
        writeln!(output, "Error: {}", error).unwrap();

        // Error chain
        let mut source = error.source();
        while let Some(err) = source {
            writeln!(output, "  Caused by: {}", err).unwrap();
            source = err.source();
        }

        // Context information
        if self.include_context {
            self.add_context_info(&mut output, error);
        }

        // Backtrace (if available)
        if self.include_backtrace {
            if let Some(backtrace) = std::error::request_ref::<std::backtrace::Backtrace>(error) {
                writeln!(output, "\nBacktrace:\n{}", backtrace).unwrap();
            }
        }

        output
    }

    fn add_context_info(&self, output: &mut String, error: &UveddiError) {
        match error {
            UveddiError::Analysis(AnalysisError::ParseError { path, .. }) => {
                writeln!(output, "  File: {}", path).unwrap();
            }
            UveddiError::Database(DatabaseError::QueryFailed { query, .. }) => {
                writeln!(output, "  Query: {}", query).unwrap();
            }
            UveddiError::Plugin(PluginError::LoadFailed { path, .. }) => {
                writeln!(output, "  Plugin path: {}", path).unwrap();
            }
            _ => {}
        }
    }

    pub fn report_error(&self, error: &UveddiError) {
        eprintln!("{}", self.format_error(error));
    }

    pub fn create_user_friendly_message(&self, error: &UveddiError) -> String {
        match error {
            UveddiError::Config(ConfigError::InvalidFile { path }) => {
                format!("Configuration file '{}' is invalid. Please check the file format and try again.", path)
            }
            UveddiError::Analysis(AnalysisError::UnsupportedLanguage { language }) => {
                format!("Language '{}' is not supported. Supported languages: Rust, Python, JavaScript, TypeScript.", language)
            }
            UveddiError::Database(DatabaseError::ConnectionFailed { .. }) => {
                "Could not connect to database. Please check your database configuration.".to_string()
            }
            UveddiError::FeatureNotAvailable { feature } => {
                format!("Feature '{}' is not available in this build. Please use a different build configuration.", feature)
            }
            _ => error.to_string(),
        }
    }
}
```

### 6. Update Result Type Aliases

#### A. Convenient Result Types:
```rust
// src/error/types.rs
pub type Result<T> = std::result::Result<T, UveddiError>;
pub type AnalysisResult<T> = std::result::Result<T, AnalysisError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;
pub type PluginResult<T> = std::result::Result<T, PluginError>;

// Specialized result types for common operations
pub type AnalysisOperationResult = Result<crate::types::AnalysisResult>;
pub type ConfigLoadResult = Result<crate::config::Config>;
pub type PluginLoadResult = Result<Box<dyn crate::plugins::Plugin>>;
```

### 7. Implement Error Recovery Strategies

#### A. Retry Logic:
```rust
// src/error/retry.rs
use std::time::Duration;
use tokio::time::sleep;

pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

pub async fn retry_async<F, Fut, T, E>(
    config: &RetryConfig,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = config.base_delay;
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);

                if attempt < config.max_attempts - 1 {
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * config.backoff_multiplier) as u64),
                        config.max_delay,
                    );
                }
            }
        }
    }

    Err(last_error.unwrap())
}

// Macro for easier retry usage
#[macro_export]
macro_rules! retry {
    ($config:expr, $operation:expr) => {
        retry_async($config, || async { $operation }).await
    };
}
```

### 8. Update Existing Code

#### A. Replace Error Handling Patterns:
```bash
# Create script to help with migration
cat > scripts/migrate-errors.sh << 'EOF'
#!/bin/bash

# Replace anyhow::Result with crate::error::Result
find src -name "*.rs" -exec sed -i 's/anyhow::Result/crate::error::Result/g' {} +

# Replace anyhow::Error with UveddiError
find src -name "*.rs" -exec sed -i 's/anyhow::Error/crate::error::UveddiError/g' {} +

# Replace .context() with .with_context()
find src -name "*.rs" -exec sed -i 's/\.context(/\.with_context(|| /g' {} +

echo "Error handling migration completed"
EOF

chmod +x scripts/migrate-errors.sh
```

#### B. Update Service Interfaces:
```rust
// Example: Update analysis service
// Before:
// pub async fn analyze(&self, config: &Config) -> anyhow::Result<AnalysisResult>;

// After:
pub async fn analyze(&self, config: &Config) -> crate::error::Result<AnalysisResult> {
    // Add proper error context
    let result = self.engine.parse_files(&config.target_path)
        .with_context(|| format!("Failed to parse files in {}", config.target_path.display()))?;

    let analysis = self.run_detectors(&result)
        .with_context(|| "Detector execution failed")?;

    Ok(analysis)
}
```

### 9. Create Error Testing Framework

#### A. Error Testing Utilities:
```rust
// src/error/testing.rs
#[cfg(test)]
pub mod testing {
    use super::*;

    pub fn assert_error_type<T>(result: Result<T>, expected_error: fn() -> UveddiError) {
        match result {
            Err(actual_error) => {
                assert_eq!(
                    std::mem::discriminant(&actual_error),
                    std::mem::discriminant(&expected_error())
                );
            }
            Ok(_) => panic!("Expected error, but got Ok"),
        }
    }

    pub fn create_test_io_error() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Test file not found")
    }

    pub fn create_test_analysis_error() -> AnalysisError {
        AnalysisError::UnsupportedLanguage {
            language: "test-lang".to_string(),
        }
    }

    // Test error conversion
    #[test]
    fn test_io_error_conversion() {
        let io_err = create_test_io_error();
        let uveddi_err: UveddiError = io_err.into();

        match uveddi_err {
            UveddiError::Io(_) => (),
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(AnalysisError::UnsupportedLanguage {
            language: "test".to_string(),
        }.into());

        let with_context = result.with_context(|| "Additional context");
        assert!(with_context.is_err());
    }
}
```

### 10. Create Error Documentation

#### A. Error Handling Guide:
```rust
// src/error/docs.rs
//! # Error Handling Guide
//!
//! This module provides a comprehensive error handling system for Uveddi.
//!
//! ## Usage Examples
//!
//! ### Basic Error Handling
//! ```rust
//! use crate::error::{Result, UveddiError, AnalysisError};
//!
//! fn analyze_file(path: &Path) -> Result<AnalysisResult> {
//!     let content = std::fs::read_to_string(path)
//!         .with_context(|| format!("Failed to read file: {}", path.display()))?;
//!
//!     // Analysis logic...
//!     Ok(analysis_result)
//! }
//! ```
//!
//! ### Error Creation
//! ```rust
//! // Create specific error types
//! let config_error = ConfigError::InvalidValue {
//!     key: "timeout".to_string(),
//!     value: "invalid".to_string(),
//! };
//!
//! // Convert to top-level error
//! let uveddi_error: UveddiError = config_error.into();
//! ```
//!
//! ### Error Reporting
//! ```rust
//! let reporter = ErrorReporter::new(true, true);
//! reporter.report_error(&error);
//! ```
```

## Success Criteria
- [ ] Single error type hierarchy for entire codebase
- [ ] Consistent error messages and context
- [ ] Proper error conversion from external libraries
- [ ] User-friendly error reporting
- [ ] Comprehensive error testing
- [ ] All existing tests pass with new error system

## Migration Strategy
1. **Phase 1**: Implement new error types
2. **Phase 2**: Update core modules
3. **Phase 3**: Migrate external integrations
4. **Phase 4**: Update tests and documentation
5. **Phase 5**: Remove old error handling code

## Verification Commands
```bash
# Check error consistency
rg "anyhow::" src/ --type rust  # Should find minimal usage
rg "Result<" src/ --type rust | head -20

# Test error handling
cargo test error::

# Check error conversion coverage
cargo test --test error_conversions
```

## Completion Notes
_To be filled by AI developer:_
- Error types implemented: ___
- Modules migrated: ___
- Test coverage for errors: ___
- Breaking changes: ___
- Performance impact: ___