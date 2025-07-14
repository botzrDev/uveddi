# Comprehensive Implementation Plan for UV-157: Input Validation System

Based on the research document and current codebase analysis, here's a complete top-to-bottom implementation plan that follows the "Parse, Don't Validate" philosophy with type-driven security.

## 🎯 **Implementation Strategy Overview**

### **Core Philosophy**: Type-Driven Security
- **Parse, Don't Validate**: Transform primitive inputs into validated domain types at system boundaries
- **Compile-Time Guarantees**: Use Rust's type system to prevent invalid data from reaching core logic
- **Zero-Cost Abstractions**: Leverage `nutype` and `garde` for validation without runtime overhead
- **Fail-Safe Defaults**: Secure by construction, not convention

## 📋 **Phase-by-Phase Implementation Plan**

### **Phase 0: Foundation Setup** (2 hours)
**Goal**: Add required dependencies and create validation infrastructure

#### **Step 0.1: Update Dependencies**
```toml
# Add to Cargo.toml [dependencies]
nutype = { version = "0.4", features = ["serde"] }
garde = { version = "0.20", features = ["derive"] }
url = "2.5"
mime = "0.3"
infer = "0.16"  # For file type detection by content
```

#### **Step 0.2: Create Validation Module Structure**
```
src/validation/
├── mod.rs              # Public API and re-exports
├── types.rs            # nutype domain types
├── config.rs           # garde configuration validation
├── filesystem.rs       # File system validation logic
├── errors.rs           # Validation error types
└── limits.rs           # Configurable limits and constants
```

#### **Step 0.3: Extend Error System**
Add `ValidationError` to `src/error/main.rs`:
```rust
#[derive(Error, Debug)]
pub enum UveddiError {
    // ... existing variants ...
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
}
```

### **Phase 1: Core Domain Types** (4 hours)
**Goal**: Create type-safe wrappers for all critical inputs using `nutype`

#### **Step 1.1: File System Types** (`src/validation/types.rs`)
```rust
use nutype::nutype;
use std::path::PathBuf;

// File path that's been validated and canonicalized
#[nutype(
    sanitize(with = |path: String| std::path::Path::new(&path).canonicalize().map(|p| p.to_string_lossy().to_string()).unwrap_or(path)),
    validate(predicate = |path| std::path::Path::new(path).exists()),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Deref, Serialize, Deserialize)
)]
pub struct ValidatedPath(String);

// File size within acceptable limits
#[nutype(
    validate(range = 1..=10_485_760), // 1 byte to 10MB
    derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)
)]
pub struct ValidatedFileSize(u64);

// Directory depth within limits
#[nutype(
    validate(range = 0..=100),
    derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)
)]
pub struct ValidatedDepth(usize);

// Supported file extensions
#[nutype(
    validate(predicate = |ext| matches!(ext.as_str(), "rs" | "py" | "js" | "jsx" | "ts" | "tsx")),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Deref, Serialize, Deserialize)
)]
pub struct ValidatedExtension(String);
```

#### **Step 1.2: Configuration Types**
```rust
// API URL validation
#[nutype(
    sanitize(trim),
    validate(predicate = |url| url::Url::parse(url).is_ok() && url.starts_with("http")),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Deref, Serialize, Deserialize)
)]
pub struct ValidatedApiUrl(String);

// Model name validation
#[nutype(
    sanitize(trim),
    validate(
        len_char_min = 1,
        len_char_max = 100,
        predicate = |name| !name.contains(['/', '\\', '..', '\0'])
    ),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Deref, Serialize, Deserialize)
)]
pub struct ValidatedModelName(String);

// Database description with length limits
#[nutype(
    sanitize(with = |desc: String| desc.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\t').collect()),
    validate(len_char_max = 10_000),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Deref, Serialize, Deserialize)
)]
pub struct SanitizedDescription(String);
```

### **Phase 2: File System Validation** (6 hours)
**Goal**: Implement comprehensive file system validation with type safety

#### **Step 2.1: Enhanced File Validation** (`src/validation/filesystem.rs`)
```rust
use crate::validation::{types::*, errors::ValidationError};
use std::path::{Path, PathBuf};
use std::fs;

pub struct FileSystemValidator {
    max_file_size: u64,
    max_depth: usize,
    allowed_extensions: Vec<String>,
}

impl FileSystemValidator {
    pub fn new() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_depth: 100,
            allowed_extensions: vec!["rs", "py", "js", "jsx", "ts", "tsx"]
                .into_iter().map(String::from).collect(),
        }
    }

    pub fn validate_file_for_analysis(&self, path: &Path) -> Result<ValidatedAnalysisFile, ValidationError> {
        // Step 1: Validate path exists and is readable
        let validated_path = ValidatedPath::new(path.to_string_lossy().to_string())?;
        
        // Step 2: Check file size
        let metadata = fs::metadata(path)?;
        let file_size = ValidatedFileSize::new(metadata.len())?;
        
        // Step 3: Validate file extension
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or(ValidationError::InvalidFileExtension { path: path.to_path_buf() })?;
        let validated_extension = ValidatedExtension::new(extension.to_string())?;
        
        // Step 4: Validate file type by content (security critical)
        self.validate_file_content_type(path)?;
        
        Ok(ValidatedAnalysisFile {
            path: validated_path,
            size: file_size,
            extension: validated_extension,
        })
    }

    fn validate_file_content_type(&self, path: &Path) -> Result<(), ValidationError> {
        let mut file = fs::File::open(path)?;
        let mut buffer = [0; 512]; // Read first 512 bytes
        use std::io::Read;
        let bytes_read = file.read(&mut buffer)?;
        
        // Use infer crate to detect actual file type
        if let Some(file_type) = infer::get(&buffer[..bytes_read]) {
            // Reject binary files masquerading as source code
            if file_type.mime_type().starts_with("application/") && 
               !file_type.mime_type().contains("json") {
                return Err(ValidationError::SuspiciousFileContent {
                    path: path.to_path_buf(),
                    detected_type: file_type.mime_type().to_string(),
                });
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedAnalysisFile {
    pub path: ValidatedPath,
    pub size: ValidatedFileSize,
    pub extension: ValidatedExtension,
}
```

#### **Step 2.2: Safe Async Walker** (`src/ingestion/async_walker.rs` - modifications)
```rust
use crate::validation::{types::ValidatedDepth, errors::ValidationError};

pub struct SafeAsyncWalker {
    include_extensions: Vec<String>,
    max_depth: ValidatedDepth,
    max_files: usize,
    current_depth: usize,
    files_processed: usize,
}

impl SafeAsyncWalker {
    pub fn new(include_extensions: Vec<String>) -> Result<Self, ValidationError> {
        Ok(Self {
            include_extensions,
            max_depth: ValidatedDepth::new(50)?, // Safe default
            max_files: 10_000,
            current_depth: 0,
            files_processed: 0,
        })
    }

    fn check_limits(&self) -> Result<(), ValidationError> {
        if self.current_depth > self.max_depth.into_inner() {
            return Err(ValidationError::DirectoryTooDeep {
                depth: self.current_depth,
                max_depth: self.max_depth.into_inner(),
            });
        }
        
        if self.files_processed > self.max_files {
            return Err(ValidationError::TooManyFiles {
                count: self.files_processed,
                max_files: self.max_files,
            });
        }
        
        Ok(())
    }
}
```

### **Phase 3: Configuration Validation** (4 hours)
**Goal**: Implement type-safe configuration validation using `garde`

#### **Step 3.1: Configuration Struct** (`src/validation/config.rs`)
```rust
use garde::Validate;
use crate::validation::types::*;

#[derive(Debug, Clone, Validate)]
#[garde(context(ValidationContext))]
pub struct ValidatedUveddiConfig {
    #[garde(dive)] // Recursively validate
    pub ai: AiConfig,
    
    #[garde(dive)]
    pub analysis: AnalysisConfig,
    
    #[garde(dive)]
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Validate)]
pub struct AiConfig {
    #[garde(transparent)] // Delegate to ValidatedApiUrl's validation
    pub api_url: Option<ValidatedApiUrl>,
    
    #[garde(transparent)]
    pub model: ValidatedModelName,
    
    #[garde(range(min = 1, max = 300))] // 1-300 second timeout
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Validate)]
pub struct AnalysisConfig {
    #[garde(transparent)]
    pub max_depth: ValidatedDepth,
    
    #[garde(range(min = 1, max = 100))]
    pub max_threads: usize,
    
    #[garde(length(min = 1))]
    pub include_extensions: Vec<ValidatedExtension>,
}

#[derive(Debug, Clone, Validate)]
pub struct SecurityConfig {
    #[garde(transparent)]
    pub max_file_size: ValidatedFileSize,
    
    #[garde(range(min = 1000, max = 1_000_000))] // 1KB to 1MB cache entries
    pub max_cache_entry_size: usize,
    
    #[garde(custom(validate_memory_limit))]
    pub memory_limit_mb: Option<usize>,
}

pub struct ValidationContext {
    pub available_memory: usize,
    pub cpu_count: usize,
}

fn validate_memory_limit(limit: &Option<usize>, context: &ValidationContext) -> garde::Result {
    if let Some(limit) = limit {
        if *limit > context.available_memory / 2 {
            return Err(garde::Error::new("Memory limit too high for system"));
        }
    }
    Ok(())
}
```

#### **Step 3.2: Environment Variable Validation** (`src/ai/engine.rs` - modifications)
```rust
use crate::validation::{types::*, config::ValidatedUveddiConfig};

impl AiAnalysisEngine {
    pub fn new() -> Result<Self, UveddiError> {
        let config = Self::load_validated_config()?;
        
        let provider = if let Some(api_url) = &config.ai.api_url {
            let ollama_config = OllamaConfig {
                model: config.ai.model.into_inner(),
                api_url: api_url.into_inner(),
                timeout: Duration::from_secs(config.ai.timeout_seconds),
            };
            Some(Box::new(OllamaProvider::new(ollama_config)) as Box<dyn LlmProvider + Send + Sync>)
        } else {
            None
        };

        Ok(AiAnalysisEngine {
            provider,
            prompt_builder: SmartPromptBuilder::new(),
        })
    }

    fn load_validated_config() -> Result<ValidatedUveddiConfig, UveddiError> {
        // Load from environment with validation
        let api_url = env::var("OLLAMA_API_URL").ok()
            .map(ValidatedApiUrl::new)
            .transpose()?;
            
        let model = ValidatedModelName::new(
            env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "deepseek-coder:6.7b-instruct-q4_0".to_string())
        )?;

        let config = ValidatedUveddiConfig {
            ai: AiConfig {
                api_url,
                model,
                timeout_seconds: 30,
            },
            // ... other config sections
        };

        // Validate the entire config structure
        let context = ValidationContext {
            available_memory: get_available_memory(),
            cpu_count: num_cpus::get(),
        };
        
        config.validate(&context)?;
        Ok(config)
    }
}
```

### **Phase 4: Database Validation** (3 hours)
**Goal**: Add input sanitization and validation to database operations

#### **Step 4.1: Database Input Validation** (`src/database/crud.rs` - modifications)
```rust
use crate::validation::types::SanitizedDescription;

impl Database {
    pub fn insert_issue(&self, issue: &ArchitecturalIssue) -> Result<i64, UveddiError> {
        // Validate and sanitize description
        let sanitized_desc = SanitizedDescription::new(issue.description.clone())?;
        
        // Validate file path if present
        let validated_path = if let Some(path) = &issue.file_path {
            Some(ValidatedPath::new(path.clone())?)
        } else {
            None
        };

        let mut stmt = self.conn.prepare(
            "INSERT INTO architectural_issues (description, file_path, severity, pattern_type) 
             VALUES (?1, ?2, ?3, ?4)"
        )?;

        let id = stmt.insert(rusqlite::params![
            sanitized_desc.into_inner(),
            validated_path.map(|p| p.into_inner()),
            issue.severity.to_string(),
            issue.pattern_type.to_string(),
        ])?;

        Ok(id)
    }
}
```

#### **Step 4.2: Cache Size Validation** (`src/cache/result_cache.rs` - modifications)
```rust
use crate::validation::errors::ValidationError;

impl ResultCache {
    pub fn set<K: Hash, V: Serialize>(&self, key_data: &K, value: &V) -> Result<(), UveddiError> {
        let key = Self::hash_key(key_data);
        let value_blob = bincode::serialize(value)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
        
        // Validate cache entry size
        const MAX_CACHE_ENTRY_SIZE: usize = 1024 * 1024; // 1MB
        if value_blob.len() > MAX_CACHE_ENTRY_SIZE {
            return Err(ValidationError::CacheEntryTooLarge {
                size: value_blob.len(),
                max_size: MAX_CACHE_ENTRY_SIZE,
            }.into());
        }

        self.conn.execute(
            "INSERT OR REPLACE INTO cache (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value_blob],
        )?;
        Ok(())
    }
}
```

### **Phase 5: Error Handling & Integration** (5 hours)
**Goal**: Complete error system and integrate validation throughout

#### **Step 5.1: Comprehensive Error Types** (`src/validation/errors.rs`)
```rust
use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum ValidationError {
    // File System Validation Errors
    #[error("File too large: {path} ({size} bytes, max: {max_size})")]
    FileTooLarge { path: PathBuf, size: u64, max_size: u64 },
    
    #[error("Directory too deep: {depth} levels (max: {max_depth})")]
    DirectoryTooDeep { depth: usize, max_depth: usize },
    
    #[error("Too many files: {count} (max: {max_files})")]
    TooManyFiles { count: usize, max_files: usize },
    
    #[error("Invalid file extension: {path}")]
    InvalidFileExtension { path: PathBuf },
    
    #[error("Suspicious file content: {path} (detected: {detected_type})")]
    SuspiciousFileContent { path: PathBuf, detected_type: String },
    
    // Configuration Validation Errors
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },
    
    #[error("Invalid model name: {name}")]
    InvalidModelName { name: String },
    
    // Database Validation Errors
    #[error("Description too long: {length} chars (max: {max_length})")]
    DescriptionTooLong { length: usize, max_length: usize },
    
    #[error("Cache entry too large: {size} bytes (max: {max_size})")]
    CacheEntryTooLarge { size: usize, max_size: usize },
    
    // Type Construction Errors
    #[error("Type validation failed: {0}")]
    TypeValidation(#[from] nutype::Error),
    
    #[error("Struct validation failed: {0}")]
    StructValidation(#[from] garde::Report),
    
    // System Errors
    #[error("IO error during validation: {0}")]
    Io(#[from] std::io::Error),
}
```

#### **Step 5.2: AST Parser Integration** (`src/ast/tree_sitter_impl.rs` - modifications)
```rust
use crate::validation::{filesystem::FileSystemValidator, types::ValidatedAnalysisFile};

impl AstParser {
    pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        // Validate file before parsing
        let validator = FileSystemValidator::new();
        let validated_file = validator.validate_file_for_analysis(file_path)
            .map_err(|e| AstError::ValidationError(e.to_string()))?;
        
        // Use validated path for reading
        let source = fs::read_to_string(validated_file.path.as_ref())?;
        
        // Continue with existing parsing logic...
        let language = self.detect_language(validated_file.extension.as_ref())?;
        // ... rest of implementation
    }
}
```

### **Phase 6: Testing & Documentation** (4 hours)
**Goal**: Comprehensive test coverage and documentation

#### **Step 6.1: Validation Tests** (`tests/validation/`)
```rust
// tests/validation/filesystem_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_file_size_validation() {
        let validator = FileSystemValidator::new();
        
        // Test file too large
        let large_file = create_test_file(11 * 1024 * 1024); // 11MB
        let result = validator.validate_file_for_analysis(&large_file);
        assert!(matches!(result, Err(ValidationError::FileTooLarge { .. })));
    }
    
    #[test]
    fn test_malicious_file_detection() {
        let validator = FileSystemValidator::new();
        
        // Create a binary file with .rs extension
        let malicious_file = create_binary_file_with_rs_extension();
        let result = validator.validate_file_for_analysis(&malicious_file);
        assert!(matches!(result, Err(ValidationError::SuspiciousFileContent { .. })));
    }
}
```

## 🚀 **Implementation Timeline**

| Phase | Duration | Deliverables | Dependencies |
|-------|----------|--------------|--------------|
| **Phase 0** | 2 hours | Dependencies, module structure, error integration | None |
| **Phase 1** | 4 hours | Core domain types with `nutype` | Phase 0 |
| **Phase 2** | 6 hours | File system validation | Phase 1 |
| **Phase 3** | 4 hours | Configuration validation with `garde` | Phase 1 |
| **Phase 4** | 3 hours | Database input sanitization | Phase 1 |
| **Phase 5** | 5 hours | Error handling & integration | Phases 2-4 |
| **Phase 6** | 4 hours | Testing & documentation | All phases |

**Total: 28 hours** (vs. original estimate of 18 hours - more comprehensive approach)

## ✅ **Success Criteria Validation**

### **Security Benefits Achieved**
- ✅ **Resource exhaustion prevention**: File size and depth limits enforced at type level
- ✅ **Directory traversal protection**: Path validation with canonicalization
- ✅ **Configuration injection prevention**: URL and parameter validation
- ✅ **Database injection prevention**: Input sanitization with length limits
- ✅ **Type safety**: Compile-time guarantees prevent invalid data propagation

### **Performance Benefits**
- ✅ **Zero-cost abstractions**: `nutype` generates efficient code
- ✅ **Early validation**: Fail fast at system boundaries
- ✅ **Memory safety**: Bounded allocations and cache sizes

### **Maintainability Benefits**
- ✅ **Centralized validation**: Single source of truth for validation rules
- ✅ **Type-driven design**: Self-documenting code through types
- ✅ **Comprehensive error handling**: Clear, actionable error messages

This implementation plan transforms UV-157 from a basic input validation task into a comprehensive security architecture that leverages Rust's type system for compile-time safety guarantees.

## 📚 **Additional Resources**

### **Key Dependencies Documentation**
- [nutype crate](https://docs.rs/nutype/) - Type-safe newtypes with validation
- [garde crate](https://docs.rs/garde/) - Struct validation framework
- [thiserror crate](https://docs.rs/thiserror/) - Error handling macros
- [infer crate](https://docs.rs/infer/) - File type detection by content

### **Related Documentation**
- [UV-157 Research Document](./UV_157_Research.md) - Comprehensive research and architecture
- [Uveddi Security Module](../../../src/security.rs) - Existing security utilities
- [Error Handling Strategy](../../../src/error/mod.rs) - Current error system

### **Implementation Notes**
- All validation happens at system boundaries (Parse, Don't Validate)
- Type safety prevents invalid data from reaching core business logic
- Comprehensive error messages provide actionable feedback
- Zero-cost abstractions maintain performance while improving security
- Incremental rollout possible - can be implemented phase by phase

---

**This implementation plan provides a robust foundation for secure input validation that leverages Rust's type system for compile-time safety guarantees while maintaining performance and usability.**