# Input Validation Security Guide

## Overview

This guide documents the comprehensive input validation framework implemented in Uveddi to prevent security vulnerabilities such as injection attacks, path traversal, and data corruption. All user inputs, API requests, configuration files, and external data are validated using a defense-in-depth approach.

## Security Architecture

### Validation Layers

1. **Type System Validation** - Rust's type system provides the first layer of validation
2. **Input Syntax Validation** - Format and syntax validation for all inputs  
3. **Security Pattern Detection** - Detection of injection attacks and malicious patterns
4. **Business Logic Validation** - Domain-specific validation rules
5. **Sanitization** - Safe data transformation for storage and display

### Security Principles

- **Fail Secure** - Invalid inputs are rejected by default
- **Defense in Depth** - Multiple validation layers prevent bypass
- **Clear Error Messages** - Specific error information without information leakage
- **Performance Conscious** - Efficient validation algorithms
- **Comprehensive Coverage** - All input sources are validated

## Validation Functions Reference

### Core Input Validation

#### `validate_input(input: &str, field_name: &str)`

Primary validation function for general text inputs.

**Security Checks:**
- SQL injection pattern detection
- Length constraints (max 10,000 characters)
- Null byte detection
- Control character filtering

**SQL Injection Patterns Detected:**
- `;`, `--`, `/*`, `*/` (comment injection)
- `union`, `select`, `insert`, `update`, `delete`, `drop` (keyword injection)
- `exec`, `execute`, `xp_`, `sp_` (stored procedure injection)
- `script`, `javascript:` (script injection)
- `alter`, `create`, `truncate`, `grant`, `revoke` (DDL injection)

**Example:**
```rust
use uveddi::security::validate_input;

// Valid input
assert!(validate_input("user123", "username").is_ok());

// SQL injection attempt - rejected
assert!(validate_input("'; DROP TABLE users; --", "username").is_err());
```

#### `validate_url(url: &str)`

Validates URL format and security.

**Security Checks:**
- Protocol restriction (HTTP/HTTPS only)
- Path traversal detection (`..`, `\`)
- Null byte detection
- Format validation

**Example:**
```rust
use uveddi::security::validate_url;

// Valid URLs
assert!(validate_url("https://api.example.com/v1").is_ok());
assert!(validate_url("http://localhost:11434").is_ok());

// Dangerous URLs - rejected
assert!(validate_url("javascript:alert(1)").is_err());
assert!(validate_url("file:///etc/passwd").is_err());
```

### CLI Argument Validation

#### `validate_cli_argument(arg_value: &str, arg_name: &str, validation_type: CliArgumentType)`

Enhanced validation for command-line arguments.

**Validation Types:**

##### `CliArgumentType::FilePath`
- Basic input validation
- Shell injection detection (`$(`, `` ` ``)
- Path format validation

##### `CliArgumentType::Port`
- Numeric format validation
- Port range validation (1024-65535)
- System port protection

##### `CliArgumentType::Percentage`
- Float format validation
- Range validation (0.0-100.0)

##### `CliArgumentType::Count`
- Integer format validation
- Range validation (0-1,000,000)

##### `CliArgumentType::Generic`
- Standard input validation
- SQL injection detection

**Example:**
```rust
use uveddi::security::{validate_cli_argument, CliArgumentType};

// Port validation
assert!(validate_cli_argument("8080", "port", CliArgumentType::Port).is_ok());
assert!(validate_cli_argument("80", "port", CliArgumentType::Port).is_err()); // System port

// File path validation
assert!(validate_cli_argument("/valid/path", "path", CliArgumentType::FilePath).is_ok());
assert!(validate_cli_argument("$(malicious)", "path", CliArgumentType::FilePath).is_err());
```

### API Request Validation

#### `validate_api_request(content_type: Option<&str>, content_length: Option<u64>, user_agent: Option<&str>)`

Validates HTTP API requests for security compliance.

**Security Checks:**
- Content-Type allowlist
- Request size limits (10MB maximum)
- User-Agent validation
- Bot detection (logged, not blocked)

**Allowed Content Types:**
- `application/json`
- `application/x-www-form-urlencoded`
- `multipart/form-data`
- `text/plain`

**Example:**
```rust
use uveddi::security::validate_api_request;

// Valid API request
assert!(validate_api_request(
    Some("application/json"),
    Some(1024),
    Some("curl/7.68.0")
).is_ok());

// Oversized request - rejected
assert!(validate_api_request(
    Some("application/json"),
    Some(20 * 1024 * 1024), // 20MB
    None
).is_err());
```

### Configuration File Validation

#### `validate_config_file_path(config_path: &str, allowed_directories: Option<&[&str]>)`

Secure configuration file access validation.

**Security Checks:**
- File extension validation (`.toml`, `.yaml`, `.yml`, `.json`)
- Directory restriction enforcement
- File size limits (1MB maximum)
- Path traversal prevention
- Canonicalization and containment verification

**Example:**
```rust
use uveddi::security::validate_config_file_path;

// Valid config in allowed directory
let allowed_dirs = ["./config", "/etc/uveddi"];
assert!(validate_config_file_path(
    "./config/app.toml",
    Some(&allowed_dirs)
).is_ok());

// Path traversal attempt - rejected
assert!(validate_config_file_path(
    "../../../etc/passwd",
    Some(&allowed_dirs)
).is_err());
```

### Database Parameter Validation

#### `validate_db_parameter(param_value: &str, param_name: &str, param_type: DbParameterType)`

Database query parameter validation for additional SQL injection protection.

**Parameter Types:**

##### `DbParameterType::Id`
- Integer format validation
- Numeric bounds checking

##### `DbParameterType::Text`
- Standard input validation
- SQL injection detection

##### `DbParameterType::FilePath`
- File path format validation
- Length and character validation

##### `DbParameterType::CodeContent`
- Permissive validation for code analysis results
- Null byte detection
- Size limits (100KB default)

##### `DbParameterType::Timestamp`
- RFC3339 timestamp format validation

**Example:**
```rust
use uveddi::security::{validate_db_parameter, DbParameterType};

// Valid database parameters
assert!(validate_db_parameter("123", "id", DbParameterType::Id).is_ok());
assert!(validate_db_parameter("2023-01-01T12:00:00Z", "created_at", DbParameterType::Timestamp).is_ok());

// Code content (allows SQL keywords in context)
assert!(validate_db_parameter("SELECT * FROM table;", "code", DbParameterType::CodeContent).is_ok());
```

### JSON Input Validation

#### `validate_json_input(json_str: &str, max_depth: Option<usize>, max_size: Option<usize>)`

Secure JSON parsing with size and depth limits.

**Security Checks:**
- JSON format validation
- Nesting depth limits (10 levels default)
- Size limits (1MB default)
- Basic input validation of content

**Example:**
```rust
use uveddi::security::validate_json_input;

// Valid JSON
let json = r#"{"name": "test", "value": 42}"#;
assert!(validate_json_input(json, None, None).is_ok());

// Deeply nested JSON - rejected
let deep_json = "{".repeat(20) + "\"value\": true" + &"}".repeat(20);
assert!(validate_json_input(&deep_json, Some(10), None).is_err());
```

### External API Response Validation

#### `validate_external_api_response(response_data: &str, max_size: Option<usize>)`

Validates responses from external services like Ollama.

**Security Checks:**
- Size limits (1MB default)
- Null byte detection
- Control character filtering
- HTML/script tag detection
- XSS pattern detection

**Example:**
```rust
use uveddi::security::validate_external_api_response;

// Valid API response
assert!(validate_external_api_response("Normal response text", None).is_ok());

// Malicious response - rejected
assert!(validate_external_api_response("<script>alert('xss')</script>", None).is_err());
```

### Web Display Sanitization

#### `sanitize_for_web_display(content: &str) -> String`

Sanitizes content for safe HTML display.

**Transformations:**
- `&` → `&amp;`
- `<` → `&lt;`
- `>` → `&gt;`
- `"` → `&quot;`
- `'` → `&#x27;`
- `/` → `&#x2F;`

**Example:**
```rust
use uveddi::security::sanitize_for_web_display;

let malicious = "<script>alert('xss')</script>";
let safe = sanitize_for_web_display(malicious);
assert_eq!(safe, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;");
```

## API Endpoint Security

### Validation Middleware

All API endpoints are protected by validation middleware that automatically checks:

- **Request Headers**: Content-Type, Content-Length, User-Agent validation
- **Request Size**: 10MB maximum request size
- **Content Type**: Allowlist of safe content types
- **Rate Limiting**: (Planned) Per-IP and per-user limits

### Path Parameter Validation

All path parameters in API endpoints are validated:

```rust
// Example: GET /api/v1/reports/:id
async fn get_report(AxumPath(report_id): AxumPath<String>) -> Result<impl IntoResponse, StatusCode> {
    // Enhanced input validation
    if let Err(e) = security::validate_input(&report_id, "report_id") {
        error!("Invalid report ID parameter: {}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Format validation - must be numeric or "demo"
    if report_id != "demo" {
        if let Err(_) = report_id.parse::<i64>() {
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Range validation
        if let Ok(id_num) = report_id.parse::<i64>() {
            if id_num < 0 || id_num > i32::MAX as i64 {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }
    
    // Continue with business logic...
}
```

### Query Parameter Validation

Query parameters are validated using structured types:

```rust
#[derive(Debug, Deserialize)]
struct PaginationQuery {
    offset: Option<u32>,
    limit: Option<u32>,
    severity: Option<String>,
    detector: Option<String>,
}

async fn list_issues(Query(params): Query<PaginationQuery>) -> Result<impl IntoResponse, StatusCode> {
    // Validation happens automatically through deserialization
    // Additional validation can be added here
    if let Some(ref severity) = params.severity {
        if let Err(e) = security::validate_input(severity, "severity") {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Continue with business logic...
}
```

## CLI Security

### Argument Validation

All CLI arguments are validated before processing:

```rust
impl AnalyzeCommand {
    pub fn validate_inputs(&self) -> Result<(), SecurityError> {
        // Enhanced CLI argument validation
        let path_str = self.path.to_string_lossy();
        validate_cli_argument(&path_str, "path", CliArgumentType::FilePath)?;
        
        validate_cli_argument(&self.output_format, "output_format", CliArgumentType::Generic)?;
        
        // Continue with comprehensive validation...
    }
}
```

### Environment Variable Sanitization

Environment variables are validated when used:

```rust
// Ollama configuration from environment
if let Some(ref url) = env::var("OLLAMA_API_URL").ok() {
    security::validate_url(url)?;
}

if let Some(ref model) = env::var("OLLAMA_MODEL").ok() {
    security::validate_model_name(model)?;
}
```

## Configuration Security

### File Path Security

Configuration files are validated for secure access:

```rust
pub fn from_file(path: &str) -> crate::error::Result<Self> {
    // Path validation with directory restrictions
    let allowed_dirs = [".", "./config", "/etc/uveddi", "~/.config/uveddi"];
    let validated_path = security::validate_config_file_path(path, Some(&allowed_dirs))?;

    // Content validation
    let content = fs::read_to_string(&validated_path)?;
    security::validate_input(&content, "config_content")?;

    // Parse and validate configuration values
    let mut config: Config = toml::from_str(&content)?;
    config.validate()?;
    
    Ok(config)
}
```

### Value Validation

Configuration values are validated after parsing:

```rust
impl Config {
    fn validate(&self) -> Result<(), SecurityError> {
        // Model name validation
        if let Some(ref model) = self.ollama_model {
            security::validate_model_name(model)?;
        }

        // Numeric range validation
        if let Some(ref dc_config) = self.dead_code {
            if let Some(confidence) = dc_config.confidence_threshold {
                if !(0.0..=1.0).contains(&confidence) {
                    return Err(SecurityError::InvalidInput {
                        field: "dead_code.confidence_threshold".to_string(),
                        reason: "Must be between 0.0 and 1.0".to_string(),
                    });
                }
            }
        }

        // Continue with other validations...
        Ok(())
    }
}
```

## Database Security

### Parameterized Queries

All database operations use parameterized queries exclusively:

```rust
// Secure database query example
conn.prepare("SELECT project_id FROM projects WHERE path = ?")?;
stmt.query([&path_str])?;

// Never use string interpolation
// BAD: format!("SELECT * FROM table WHERE id = {}", user_id)
// GOOD: "SELECT * FROM table WHERE id = ?" with params [user_id]
```

### Input Sanitization

Database inputs are validated and sanitized:

```rust
// Validate before database storage
security::validate_db_parameter(&issue.description, "description", DbParameterType::CodeContent)?;
security::validate_file_path_for_storage(&issue.file_path, "file_path")?;

// Sanitize for safe storage
let sanitized_description = security::sanitize_description(&issue.description);
```

## Error Handling

### Secure Error Messages

Error messages provide helpful information without revealing sensitive details:

```rust
// Good: Specific but not revealing
SecurityError::InvalidInput {
    field: "username".to_string(),
    reason: "Contains invalid characters".to_string(),
}

// Bad: Too revealing
// "SQL injection detected in username: '; DROP TABLE users; --"
```

### Error Types

The `SecurityError` enum provides categorized error handling:

```rust
pub enum SecurityError {
    InvalidInput { field: String, reason: String },
    ValidationError { errors: Vec<String> },
    PathTraversalAttempt,
    SqlInjectionAttempt,
    HttpsRequired { url: String },
    RateLimitExceeded { identifier: String, current: u32, limit: u32 },
    // ... other security-related errors
}
```

### Logging Security Events

Security violations are logged for monitoring:

```rust
if let Err(e) = validate_input(user_input, "field") {
    error!("Input validation failed for field '{}': {}", field, e);
    // Additional security logging could be added here
}
```

## Testing

### Comprehensive Test Coverage

The security validation framework includes extensive tests:

- **Basic validation tests** - Standard valid/invalid input testing
- **Injection attack tests** - SQL injection, XSS, and other attack patterns
- **Boundary condition tests** - Edge cases and limit testing  
- **Performance tests** - Validation efficiency with large inputs
- **Integration tests** - End-to-end validation workflows
- **Configuration tests** - Secure configuration loading
- **API tests** - Request validation middleware testing

### Running Security Tests

```bash
# Run security-specific tests
cargo test validation_security_tests

# Run all security tests
cargo test --test security_tests

# Performance validation
cargo test test_performance_with_large_inputs
```

## Best Practices

### Input Validation Guidelines

1. **Validate Early** - Validate inputs at the earliest possible point
2. **Fail Secure** - Reject invalid inputs by default
3. **Be Specific** - Provide clear error messages without information leakage
4. **Use Allowlists** - Prefer allowlists over denylists when possible
5. **Validate Context** - Apply appropriate validation for the input context
6. **Log Security Events** - Log validation failures for security monitoring

### Common Patterns

#### Validating User Input
```rust
// Always validate user input first
security::validate_input(user_input, "field_name")?;

// Then apply context-specific validation
match input_type {
    InputType::FilePath => validate_cli_argument(user_input, "path", CliArgumentType::FilePath)?,
    InputType::Port => validate_cli_argument(user_input, "port", CliArgumentType::Port)?,
    InputType::Url => security::validate_url(user_input)?,
}
```

#### Database Parameter Validation
```rust
// Validate parameters before database operations
security::validate_db_parameter(&param_value, "param_name", DbParameterType::Text)?;

// Use parameterized queries
conn.execute("INSERT INTO table (field) VALUES (?)", [&param_value])?;
```

#### API Response Validation
```rust
// Validate external API responses
security::validate_external_api_response(&response_data, Some(1024 * 1024))?;

// Sanitize for web display if needed
let safe_content = security::sanitize_for_web_display(&response_data);
```

## Security Monitoring

### Metrics and Alerts

Consider implementing monitoring for:

- **Validation failure rates** - Unusual patterns may indicate attacks
- **Input size distributions** - Large inputs may indicate DoS attempts
- **Error pattern analysis** - Repeated injection attempts
- **Geographic request patterns** - Unusual source locations

### Logging Best Practices

- **Log security events** but not sensitive data
- **Include context** like IP addresses and timestamps
- **Use structured logging** for analysis
- **Implement log rotation** and retention policies
- **Monitor logs** for security incidents

## Conclusion

The Uveddi input validation framework provides comprehensive protection against common web application vulnerabilities through:

- **Multi-layer validation** covering all input sources
- **Security-focused design** with fail-secure defaults  
- **Performance optimization** for production use
- **Comprehensive testing** ensuring reliability
- **Clear documentation** enabling proper usage

This defense-in-depth approach significantly reduces the attack surface and provides robust protection against injection attacks, path traversal, and other input-based vulnerabilities.

For questions or contributions to the security framework, please refer to the [Security Policy](../SECURITY.md) and [Contributing Guidelines](../CONTRIBUTING.md).