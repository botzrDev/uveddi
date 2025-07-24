# 🔒 **UV-273 GPT Development Prompt: Comprehensive Input Validation Implementation**

## 📋 **Issue Context**
**Jira Issue**: UV-273  
**Title**: Implement Comprehensive Input Validation  
**Type**: Story  
**Priority**: High  
**Status**: Dev & Test  
**Epic**: UV-267 (Security Infrastructure)  
**Assignee**: Phillip Austin Green

## 🎯 **Mission Objective**
You are tasked with implementing comprehensive input validation across all external input points in the Uveddi static code analysis platform to prevent injection attacks. This builds upon existing security infrastructure and requires implementing standardized validation utilities with comprehensive test coverage.

## 📁 **Target Files & Locations**
### **Primary Implementation Files:**
1. `src/database/crud.rs:203` - Add SQL injection prevention for database operations
2. `src/cli/analyze_command.rs` - Add CLI input validation for command-line interface
3. `src/tui/ui/analyze_form.rs` - Add TUI form validation for terminal user interface
4. `src/security/mod.rs` - Extend core validation utilities

### **Supporting Files:**
- `src/security/errors.rs` - Extend SecurityError types
- `tests/security/input_validation_tests.rs` - Create comprehensive test suite
- `tests/security/fuzz_input_validation.rs` - Add fuzzing tests

## 🏗️ **Architecture Context**
### **Existing Security Infrastructure:**
- ✅ SecurityError enum with comprehensive error types
- ✅ Path traversal protection (`sanitize_path` function)
- ✅ File validation utilities (`validate_file_size`, `validate_file_type`)
- ✅ Model name validation (`validate_model_name`)
- ✅ Description sanitization (`sanitize_description`)
- ✅ Comprehensive security test framework

### **Integration Points:**
- **Database Layer**: SQLite operations with parameterized queries
- **CLI Layer**: Command-line argument processing
- **TUI Layer**: Interactive form validation with real-time feedback
- **Security Layer**: Centralized validation utilities

## 🔧 **Technical Implementation Requirements**

### **Phase 1: Core Validation Utilities (Priority: High)**

#### **1.1 Extend `src/security/mod.rs`**
Add comprehensive validation functions:

```rust
/// Maximum input length for general text fields
pub const MAX_INPUT_LENGTH: usize = 10000;

/// Maximum length for file paths
pub const MAX_PATH_LENGTH: usize = 4096;

/// Maximum length for model names
pub const MAX_MODEL_NAME_LENGTH: usize = 100;

/// Comprehensive input validation function
pub fn validate_input(input: &str, field_name: &str) -> Result<(), SecurityError> {
    // Check for SQL injection patterns
    if contains_sql_injection_patterns(input) {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: "Contains potentially dangerous SQL patterns".to_string(),
        });
    }
    
    // Check length constraints
    if input.len() > MAX_INPUT_LENGTH {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!("Input length {} exceeds maximum {}", input.len(), MAX_INPUT_LENGTH),
        });
    }
    
    Ok(())
}

/// Check for SQL injection patterns
fn contains_sql_injection_patterns(input: &str) -> bool {
    let dangerous_patterns = [
        ";", "--", "/*", "*/", "xp_", "sp_", 
        "union", "select", "insert", "update", "delete", "drop",
        "exec", "execute", "script", "javascript:",
    ];
    
    let input_lower = input.to_lowercase();
    dangerous_patterns.iter().any(|pattern| input_lower.contains(pattern))
}

/// Validate URL format for API endpoints
pub fn validate_url(url: &str) -> Result<(), SecurityError> {
    if url.is_empty() {
        return Err(SecurityError::InvalidInput {
            field: "url".to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }
    
    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(SecurityError::InvalidInput {
            field: "url".to_string(),
            reason: "URL must start with http:// or https://".to_string(),
        });
    }
    
    // Check for dangerous characters
    if url.contains("..") || url.contains("\\") || url.contains("\0") {
        return Err(SecurityError::InvalidInput {
            field: "url".to_string(),
            reason: "URL contains invalid characters".to_string(),
        });
    }
    
    Ok(())
}

/// Validate numeric input ranges
pub fn validate_numeric_range(value: i32, min: i32, max: i32, field_name: &str) -> Result<(), SecurityError> {
    if value < min || value > max {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!("Value {} must be between {} and {}", value, min, max),
        });
    }
    Ok(())
}

/// Validate character set for specific fields
pub fn validate_character_set(input: &str, field_name: &str, allowed_chars: &str) -> Result<(), SecurityError> {
    for ch in input.chars() {
        if !allowed_chars.contains(ch) {
            return Err(SecurityError::InvalidInput {
                field: field_name.to_string(),
                reason: format!("Character '{}' is not allowed", ch),
            });
        }
    }
    Ok(())
}
```

#### **1.2 Extend `src/security/errors.rs`**
Add new SecurityError variants:

```rust
// Add these variants to the SecurityError enum
#[error("Invalid input for field '{field}': {reason}")]
InvalidInput { field: String, reason: String },

#[error("Input validation failed: {errors:?}")]
ValidationError { errors: Vec<String> },

#[error("Input length exceeds maximum allowed")]
InputTooLong,

#[error("SQL injection attempt detected")]
SqlInjectionAttempt,
```

### **Phase 2: Database Input Validation (Priority: High)**

#### **2.1 Secure `src/database/crud.rs`**
Implement validated database operations:

```rust
use crate::security::{validate_input, sanitize_description, SecurityError};

impl Database {
    /// Insert analysis run with input validation
    pub fn insert_analysis_run_validated(
        &self,
        project_path: &str,
        description: Option<&str>,
    ) -> Result<i64> {
        // Validate project path
        validate_input(project_path, "project_path")?;
        
        // Validate and sanitize description
        let safe_description = if let Some(desc) = description {
            validate_input(desc, "description")?;
            Some(sanitize_description(desc))
        } else {
            None
        };
        
        // Use parameterized query (already implemented, but ensure it's used)
        let mut stmt = self.conn.prepare(
            "INSERT INTO analysis_runs (project_path, description, created_at) 
             VALUES (?1, ?2, ?3)"
        )?;
        
        let id = stmt.insert(params![
            project_path,
            safe_description,
            Utc::now().naive_utc()
        ])?;
        
        Ok(id)
    }
    
    /// Validate architectural issue data before insertion
    pub fn insert_architectural_issue_validated(
        &self,
        analysis_run_id: i64,
        issue_type: &str,
        description: &str,
        file_path: &str,
        line_number: Option<i32>,
        severity: &str,
    ) -> Result<()> {
        // Validate all inputs
        validate_input(issue_type, "issue_type")?;
        validate_input(description, "description")?;
        validate_input(file_path, "file_path")?;
        validate_input(severity, "severity")?;
        
        // Validate line number range
        if let Some(line) = line_number {
            crate::security::validate_numeric_range(line, 1, 1_000_000, "line_number")?;
        }
        
        // Sanitize description
        let safe_description = sanitize_description(description);
        
        // Use parameterized query
        self.conn.execute(
            "INSERT INTO architectural_issues 
             (analysis_run_id, issue_type, description, file_path, line_number, severity, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                analysis_run_id,
                issue_type,
                safe_description,
                file_path,
                line_number,
                severity,
                Utc::now().naive_utc()
            ],
        )?;
        
        Ok(())
    }
}
```

### **Phase 3: CLI Input Validation (Priority: Medium)**

#### **3.1 Secure `src/cli/analyze_command.rs`**
Add comprehensive CLI validation:

```rust
use crate::security::{validate_input, validate_url, validate_model_name, SecurityError};

impl AnalyzeCommand {
    /// Validate all command inputs before processing
    pub fn validate_inputs(&self) -> Result<(), SecurityError> {
        // Validate path
        let path_str = self.path.to_string_lossy();
        validate_input(&path_str, "path")?;
        
        // Validate output file if specified
        if let Some(ref output) = self.output {
            let output_str = output.to_string_lossy();
            validate_input(&output_str, "output")?;
        }
        
        // Validate Ollama API URL if specified
        if let Some(ref url) = self.ollama_api_url {
            validate_url(url)?;
        }
        
        // Validate Ollama model name if specified
        if let Some(ref model) = self.ollama_model {
            validate_model_name(model)?;
        }
        
        // Validate numeric parameters
        if let Some(confidence) = self.dead_code_confidence {
            crate::security::validate_numeric_range(confidence as i32, 0, 100, "dead_code_confidence")?;
        }
        
        if let Some(max_loc) = self.large_classes_max_loc {
            crate::security::validate_numeric_range(max_loc as i32, 1, 100_000, "large_classes_max_loc")?;
        }
        
        Ok(())
    }
    
    /// Execute analysis with input validation
    pub async fn execute_validated(&self) -> Result<(), UveddiError> {
        // Validate inputs first
        self.validate_inputs()
            .map_err(|e| UveddiError::Security(e))?;
        
        // Proceed with existing execution logic
        self.execute().await
    }
}
```

### **Phase 4: TUI Form Validation (Priority: Medium)**

#### **4.1 Enhance `src/tui/ui/analyze_form.rs`**
Add real-time form validation:

```rust
use crate::security::{validate_input, validate_url, validate_model_name, validate_numeric_range, SecurityError};

impl AnalyzeForm {
    /// Validate individual form field
    pub fn validate_field(&self, field: FormField) -> ValidationResult {
        match field {
            FormField::Path => {
                let path_str = self.path_picker.current_path().to_string_lossy();
                match validate_input(&path_str, "path") {
                    Ok(_) => ValidationResult::Valid,
                    Err(e) => ValidationResult::Invalid(e.to_string()),
                }
            }
            
            FormField::OutputFile => {
                if let Some(output) = self.output_file.value() {
                    if !output.is_empty() {
                        match validate_input(output, "output_file") {
                            Ok(_) => ValidationResult::Valid,
                            Err(e) => ValidationResult::Invalid(e.to_string()),
                        }
                    } else {
                        ValidationResult::Valid // Optional field
                    }
                } else {
                    ValidationResult::Valid
                }
            }
            
            FormField::OllamaApiUrl => {
                if let Some(url) = self.ollama_api_url.value() {
                    if !url.is_empty() {
                        match validate_url(url) {
                            Ok(_) => ValidationResult::Valid,
                            Err(e) => ValidationResult::Invalid(e.to_string()),
                        }
                    } else {
                        ValidationResult::Valid // Optional field
                    }
                } else {
                    ValidationResult::Valid
                }
            }
            
            FormField::OllamaModel => {
                if let Some(model) = self.ollama_model.value() {
                    if !model.is_empty() {
                        match validate_model_name(model) {
                            Ok(_) => ValidationResult::Valid,
                            Err(e) => ValidationResult::Invalid(e.to_string()),
                        }
                    } else {
                        ValidationResult::Valid // Optional field
                    }
                } else {
                    ValidationResult::Valid
                }
            }
            
            FormField::DeadCodeConfidence => {
                let value = self.dead_code_confidence.value();
                match validate_numeric_range(value as i32, 0, 100, "dead_code_confidence") {
                    Ok(_) => ValidationResult::Valid,
                    Err(e) => ValidationResult::Invalid(e.to_string()),
                }
            }
            
            FormField::LargeClassesMaxLoc => {
                let value = self.large_classes_max_loc.value();
                match validate_numeric_range(value as i32, 1, 100_000, "large_classes_max_loc") {
                    Ok(_) => ValidationResult::Valid,
                    Err(e) => ValidationResult::Invalid(e.to_string()),
                }
            }
            
            // Add validation for other numeric fields...
            _ => ValidationResult::Valid, // Default for fields without specific validation
        }
    }
    
    /// Validate entire form before submission
    pub fn validate_form(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate all fields
        for field in FormField::all() {
            if let ValidationResult::Invalid(error) = self.validate_field(field) {
                errors.push(format!("{}: {}", field.label(), error));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### **Phase 5: Comprehensive Test Suite (Priority: High)**

#### **5.1 Create `tests/security/input_validation_tests.rs`**
Implement comprehensive validation tests:

```rust
//! Comprehensive input validation tests for UV-273

use uveddi::security::{
    validate_input, validate_url, validate_model_name, validate_numeric_range,
    validate_character_set, SecurityError
};

#[cfg(test)]
mod input_validation_tests {
    use super::*;

    #[test]
    fn test_sql_injection_prevention() {
        let malicious_inputs = vec![
            "'; DROP TABLE users; --",
            "1; DELETE FROM analysis_runs; --",
            "test' UNION SELECT * FROM users --",
            "admin'/**/OR/**/1=1/**/--",
            "'; EXEC xp_cmdshell('dir'); --",
        ];

        for input in malicious_inputs {
            let result = validate_input(input, "test_field");
            assert!(result.is_err(), "Should reject SQL injection: {}", input);
            
            if let Err(SecurityError::InvalidInput { reason, .. }) = result {
                assert!(reason.contains("dangerous SQL patterns"));
            }
        }
    }

    #[test]
    fn test_length_validation() {
        // Test normal length
        let normal_input = "a".repeat(1000);
        assert!(validate_input(&normal_input, "test").is_ok());

        // Test excessive length
        let long_input = "a".repeat(20000);
        let result = validate_input(&long_input, "test");
        assert!(result.is_err());
        
        if let Err(SecurityError::InvalidInput { reason, .. }) = result {
            assert!(reason.contains("exceeds maximum"));
        }
    }

    #[test]
    fn test_url_validation() {
        // Valid URLs
        let valid_urls = vec![
            "http://localhost:11434",
            "https://api.example.com",
            "http://127.0.0.1:8080/api",
        ];

        for url in valid_urls {
            assert!(validate_url(url).is_ok(), "Should accept valid URL: {}", url);
        }

        // Invalid URLs
        let invalid_urls = vec![
            "ftp://example.com",
            "javascript:alert(1)",
            "http://example.com/../../../etc/passwd",
            "https://example.com\0",
            "",
        ];

        for url in invalid_urls {
            assert!(validate_url(url).is_err(), "Should reject invalid URL: {}", url);
        }
    }

    #[test]
    fn test_model_name_validation() {
        // Valid model names
        let valid_names = vec![
            "deepseek-coder",
            "llama2-7b",
            "codellama",
            "mistral-7b-instruct",
        ];

        for name in valid_names {
            assert!(validate_model_name(name).is_ok(), "Should accept valid model: {}", name);
        }

        // Invalid model names
        let invalid_names = vec![
            "",
            "model/with/slashes",
            "model..with..dots",
            "model\\with\\backslashes",
            "model\0with\0nulls",
            &"a".repeat(200), // Too long
        ];

        for name in invalid_names {
            assert!(validate_model_name(name).is_err(), "Should reject invalid model: {}", name);
        }
    }

    #[test]
    fn test_numeric_range_validation() {
        // Valid ranges
        assert!(validate_numeric_range(50, 0, 100, "confidence").is_ok());
        assert!(validate_numeric_range(0, 0, 100, "confidence").is_ok());
        assert!(validate_numeric_range(100, 0, 100, "confidence").is_ok());

        // Invalid ranges
        assert!(validate_numeric_range(-1, 0, 100, "confidence").is_err());
        assert!(validate_numeric_range(101, 0, 100, "confidence").is_err());
    }

    #[test]
    fn test_character_set_validation() {
        let alphanumeric = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        
        // Valid input
        assert!(validate_character_set("abc123", "test", alphanumeric).is_ok());
        
        // Invalid input
        assert!(validate_character_set("abc@123", "test", alphanumeric).is_err());
        assert!(validate_character_set("test-input", "test", alphanumeric).is_err());
    }

    #[test]
    fn test_edge_cases() {
        // Empty strings
        assert!(validate_input("", "test").is_ok());
        
        // Unicode characters
        assert!(validate_input("测试", "test").is_ok());
        
        // Special characters that should be allowed
        assert!(validate_input("file_name.txt", "test").is_ok());
        assert!(validate_input("path/to/file", "test").is_ok());
    }

    #[test]
    fn test_performance_with_large_inputs() {
        use std::time::Instant;
        
        let large_input = "a".repeat(9999); // Just under the limit
        let start = Instant::now();
        let result = validate_input(&large_input, "test");
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100, "Validation should be fast even for large inputs");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use uveddi::database::Database;
    use tempfile::TempDir;

    #[test]
    fn test_database_validation_integration() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(Some(&db_path)).unwrap();

        // Test valid insertion
        let result = db.insert_analysis_run_validated(
            "/safe/path/to/project",
            Some("Valid description"),
        );
        assert!(result.is_ok());

        // Test SQL injection prevention
        let result = db.insert_analysis_run_validated(
            "'; DROP TABLE analysis_runs; --",
            Some("Malicious description"),
        );
        assert!(result.is_err());
    }
}
```

#### **5.2 Create `tests/security/fuzz_input_validation.rs`**
Add fuzzing tests for robustness:

```rust
//! Fuzz testing for input validation

use uveddi::security::validate_input;
use rand::{Rng, thread_rng};

#[cfg(test)]
mod fuzz_tests {
    use super::*;

    #[test]
    fn test_random_input_fuzzing() {
        let mut rng = thread_rng();
        
        for _ in 0..1000 {
            // Generate random string
            let length = rng.gen_range(0..20000);
            let random_string: String = (0..length)
                .map(|_| rng.gen_range(0..128) as u8 as char)
                .collect();
            
            // Validation should never panic
            let _ = validate_input(&random_string, "fuzz_test");
        }
    }

    #[test]
    fn test_boundary_conditions() {
        // Test exactly at the limit
        let at_limit = "a".repeat(10000);
        assert!(validate_input(&at_limit, "test").is_ok());
        
        // Test one over the limit
        let over_limit = "a".repeat(10001);
        assert!(validate_input(&over_limit, "test").is_err());
    }
}
```

## ✅ **Acceptance Criteria Validation**

### **Must Complete:**
- [ ] ✅ Create input validation utilities in `src/security/mod.rs`
- [ ] ✅ Add SQL injection prevention for database operations
- [ ] ✅ Implement length and format validation for all inputs
- [ ] ✅ Add character set validation for restricted fields
- [ ] ✅ Create comprehensive validation test suite with 100% coverage

### **Quality Gates:**
1. **Security**: No SQL injection vulnerabilities detected in security scan
2. **Performance**: Validation functions complete in <1ms for typical inputs
3. **Coverage**: 100% test coverage for all new validation code
4. **Integration**: All affected components pass integration tests
5. **Documentation**: Clear error messages and validation feedback

## 🚀 **Implementation Strategy**

### **Development Phases:**
1. **Phase 1**: Core validation utilities (2-3 hours)
2. **Phase 2**: Database security (1-2 hours)
3. **Phase 3**: CLI validation (1-2 hours)
4. **Phase 4**: TUI validation (2-3 hours)
5. **Phase 5**: Test suite (2-3 hours)

### **Testing Strategy:**
- **Unit Tests**: Individual validation function testing
- **Integration Tests**: End-to-end validation workflows
- **Fuzz Tests**: Random input robustness testing
- **Performance Tests**: Validation speed benchmarks
- **Security Tests**: Injection attack prevention validation

## 🔍 **Security Considerations**

### **Attack Vectors to Prevent:**
- **SQL Injection**: Malicious SQL code in user inputs
- **Path Traversal**: Directory traversal attempts in file paths
- **Command Injection**: OS command execution attempts
- **XSS**: Cross-site scripting in web contexts
- **Buffer Overflow**: Excessive input lengths
- **Unicode Attacks**: Unicode normalization exploits

### **Defense Strategies:**
- **Input Sanitization**: Remove/escape dangerous characters
- **Length Limits**: Enforce maximum input sizes
- **Character Set Validation**: Restrict to allowed character sets
- **Pattern Matching**: Detect known attack patterns
- **Parameterized Queries**: Use prepared statements for database operations

## 📚 **Documentation Requirements**

### **Code Documentation:**
- Comprehensive rustdoc comments for all validation functions
- Security considerations and attack prevention explanations
- Usage examples and best practices
- Error handling and recovery strategies

### **User Documentation:**
- Validation error message explanations
- Input format requirements and restrictions
- Troubleshooting guide for validation failures
- Security best practices for users

## 🎯 **Success Metrics**

### **Security Metrics:**
- Zero SQL injection vulnerabilities in security scans
- 100% coverage of input validation attack vectors
- All security tests passing with comprehensive edge case coverage

### **Performance Metrics:**
- Validation functions complete in <1ms for typical inputs
- No performance degradation in existing workflows
- Memory usage remains constant under load

### **Quality Metrics:**
- 100% test coverage for new validation code
- All integration tests passing
- Code review approval with security focus

## 🚨 **Critical Implementation Notes**

### **Security Best Practices:**
1. **Never trust user input** - Validate everything
2. **Fail securely** - Default to rejection on validation errors
3. **Log security events** - Audit all validation failures
4. **Use allowlists** - Prefer allowlists over blocklists
5. **Defense in depth** - Multiple validation layers

### **Performance Considerations:**
1. **Early validation** - Validate inputs as early as possible
2. **Efficient patterns** - Use optimized regex and string operations
3. **Caching** - Cache validation results where appropriate
4. **Async validation** - Use async for I/O-bound validation

### **Error Handling:**
1. **Secure error messages** - Don't leak sensitive information
2. **Consistent error format** - Use standardized SecurityError types
3. **Audit logging** - Log all security-relevant validation failures
4. **Graceful degradation** - Handle validation errors gracefully

---

## 🎯 **Final Implementation Checklist**

Before marking UV-273 as complete, ensure:

- [ ] All validation functions implemented and tested
- [ ] SQL injection prevention verified through security testing
- [ ] Length and format validation working across all input points
- [ ] Character set validation implemented for restricted fields
- [ ] Comprehensive test suite with 100% coverage
- [ ] Performance benchmarks meet requirements (<1ms validation)
- [ ] Security scan shows no injection vulnerabilities
- [ ] Integration tests pass for all affected components
- [ ] Documentation updated with validation requirements
- [ ] Code review completed with security focus

**Ready to implement comprehensive input validation for Uveddi! 🚀**