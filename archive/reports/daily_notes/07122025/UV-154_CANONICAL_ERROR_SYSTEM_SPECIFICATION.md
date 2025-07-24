# 🎯 **UV-154: Canonical Error System Specification**

## **Developer Question Response**

Your developer asked for clarification on the canonical set of variants for `UveddiError` and related error enums. Based on my analysis of the current codebase, here's the definitive specification to resolve all error variant mismatches.

---

## 📋 **1. CANONICAL UveddiError Definition**

**File**: `src/error/main.rs`

```rust
#[derive(Error, Debug)]
pub enum UveddiError {
    // === Core Analysis Errors ===
    #[error("Analysis error: {0}")]
    AnalysisError(#[from] AnalysisError),
    
    #[error("Extraction error: {0}")]
    ExtractionError(#[from] ExtractionError),
    
    // === Service & Infrastructure Errors ===
    #[error("Rendering service error: {0}")]
    RenderingServiceError(#[from] RenderingServiceError),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] RusqliteError),
    
    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),
    
    // === External System Errors ===
    #[error("Network error: {0}")]
    NetworkError(#[from] ReqwestError),
    
    #[error("Report generation error: {0}")]
    ReportError(#[from] ReportGenerationError),
    
    // === System Errors ===
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Command line error: {0}")]
    CliError(#[from] ClapError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Path error: {0}")]
    PathError(PathBuf),
    
    #[error("Generic error: {0}")]
    GenericError(#[from] anyhow::Error),
}
```

---

## 📋 **2. CANONICAL ErrorCategory Definition**

**File**: `src/error/main.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    // Core analysis categories
    Analysis,
    Extraction,
    
    // Service categories  
    Rendering,
    Database,
    Plugin,
    
    // External system categories
    Network,
    Reporting,
    
    // Infrastructure categories
    Configuration,
    Cli,
    Io,
    Serialization,
    Path,
    
    // Resilience-specific categories (for retry logic)
    ServiceCommunication,
    ResourceExhaustion,
    ServiceSpecific,
    
    // Fallback
    Generic,
}
```

---

## 📋 **3. CANONICAL ErrorSeverity Definition**

**File**: `src/error/main.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

---

## 📋 **4. CANONICAL AnalysisError Definition**

**File**: `src/analysis/errors.rs`

```rust
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("AST error: {0}")]
    AstError(#[from] AstError),
    
    #[error("Dependency extraction error: {0}")]
    DependencyExtractionError(#[from] crate::analysis::detectors::dependency::ExtractionError),
    
    #[error("Metric calculation error: {0}")]
    MetricCalculationError(String),
    
    #[error("Anti-pattern detection error: {0}")]
    AntiPatternDetectionError(String),
    
    #[error("Component extraction error: {0}")]
    ComponentExtractionError(#[from] crate::analysis::component_extractor::ComponentExtractionError),
    
    #[error("Mermaid generation error: {0}")]
    MermaidGenerationError(#[from] crate::analysis::mermaid_generator::MermaidGenerationError),
    
    #[error("Symbol resolution error: {0}")]
    SymbolResolutionError(String),
    
    #[error("Graph analysis error: {0}")]
    GraphAnalysisError(String),
}
```

---

## 🔧 **5. REQUIRED FIXES**

### **A. Update src/error/main.rs**

1. **Add missing imports**:
```rust
use crate::analysis::errors::AnalysisError;
use crate::report::errors::ReportGenerationError;
```

2. **Add missing UveddiError variants**:
```rust
#[error("Analysis error: {0}")]
AnalysisError(#[from] AnalysisError),

#[error("Report generation error: {0}")]
ReportError(#[from] ReportGenerationError),
```

3. **Add missing ErrorCategory variants**:
```rust
ServiceCommunication,
ResourceExhaustion, 
ServiceSpecific,
```

4. **Update severity() method**:
```rust
impl UveddiError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            UveddiError::AnalysisError(_)
            | UveddiError::ExtractionError(_)
            | UveddiError::DatabaseError(_)
            | UveddiError::PluginError(_) => ErrorSeverity::High,
            UveddiError::RenderingServiceError(_)
            | UveddiError::ReportError(_)
            | UveddiError::NetworkError(_) => ErrorSeverity::Medium,
            _ => ErrorSeverity::Low,
        }
    }

    pub fn category(&self) -> ErrorCategory {
        match self {
            UveddiError::AnalysisError(_) => ErrorCategory::Analysis,
            UveddiError::ExtractionError(_) => ErrorCategory::Extraction,
            UveddiError::DatabaseError(_) => ErrorCategory::Database,
            UveddiError::ConfigError(_) => ErrorCategory::Configuration,
            UveddiError::ReportError(_) => ErrorCategory::Reporting,
            UveddiError::PluginError(_) => ErrorCategory::Plugin,
            UveddiError::NetworkError(_) => ErrorCategory::Network,
            UveddiError::CliError(_) => ErrorCategory::Cli,
            UveddiError::IoError(_) => ErrorCategory::Io,
            UveddiError::SerializationError(_) => ErrorCategory::Serialization,
            UveddiError::PathError(_) => ErrorCategory::Path,
            UveddiError::RenderingServiceError(_) => ErrorCategory::Rendering,
            UveddiError::GenericError(_) => ErrorCategory::Generic,
        }
    }
}
```

### **B. Fix src/plugins/errors.rs**

**Current Issue**: Line 142 has incorrect conversion
```rust
// WRONG:
crate::error::UveddiError::PluginError(err.to_string())

// CORRECT:
crate::error::UveddiError::PluginError(err)
```

### **C. Update src/error/mod.rs**

```rust
pub mod main;
pub mod rendering;

pub use main::{UveddiError, ErrorCategory, ErrorSeverity, ExtractionError};
pub use rendering::RenderingServiceError;

// Re-export rusqlite error for convenience
pub use rusqlite::Error as RusqliteError;
```

### **D. Fix Resilience Module Errors**

**File**: `src/resilience/retry.rs`

Update retry configuration to use correct ErrorCategory variants:
```rust
impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on_categories: vec![
                ErrorCategory::ServiceCommunication,
                ErrorCategory::ResourceExhaustion,
                ErrorCategory::ServiceSpecific,
                ErrorCategory::Network,
            ],
            retry_on_severities: vec![
                ErrorSeverity::Medium,
                ErrorSeverity::High,
            ],
        }
    }
}
```

---

## 🎯 **6. IMPLEMENTATION STRATEGY**

### **Phase 1: Core Error System (Priority 1)**
1. Update `src/error/main.rs` with canonical definitions
2. Fix `src/plugins/errors.rs` conversion
3. Update `src/error/mod.rs` exports
4. Add missing imports

### **Phase 2: Analysis Integration (Priority 2)**  
1. Ensure `src/analysis/errors.rs` matches specification
2. Add missing From implementations
3. Update analysis engine error handling

### **Phase 3: Resilience Integration (Priority 3)**
1. Fix `src/resilience/retry.rs` ErrorCategory usage
2. Update circuit breaker error handling
3. Fix metrics collection error types

### **Phase 4: Cleanup (Priority 4)**
1. Remove duplicate error definitions
2. Clean up unused imports
3. Update documentation

---

## 🚨 **7. CRITICAL CONFLICTS TO RESOLVE**

### **A. Duplicate ErrorCategory/ErrorSeverity**
- **Keep**: Definitions in `src/error/main.rs`
- **Remove**: Duplicate definitions in `src/error/rendering.rs`
- **Update**: All imports to use `crate::error::{ErrorCategory, ErrorSeverity}`

### **B. Missing Module Files**
```bash
# Create these files:
touch src/analysis/tests/mod.rs
touch src/semantic_search/embedding_tests.rs
```

### **C. PluginError Conversion Conflict**
- **Issue**: Both `#[from]` derive and manual `impl From` exist
- **Solution**: Remove manual impl, keep `#[from]` derive in UveddiError

---

## ✅ **8. VALIDATION CHECKLIST**

After implementing these changes, verify:

- [ ] `cargo check` passes without errors
- [ ] All error conversions work correctly
- [ ] No duplicate error type definitions
- [ ] Resilience module uses correct ErrorCategory variants
- [ ] Plugin system integrates properly
- [ ] Analysis engine error handling works
- [ ] All imports resolve correctly

---

## 🎯 **9. RECOMMENDED IMPLEMENTATION ORDER**

1. **Start with `src/error/main.rs`** - Add missing variants and fix categories
2. **Fix `src/plugins/errors.rs`** - Correct the conversion implementation  
3. **Update `src/error/mod.rs`** - Fix exports and imports
4. **Create missing files** - Add empty module files to resolve E0583 errors
5. **Fix resilience modules** - Update ErrorCategory usage
6. **Test compilation** - Verify `cargo check` passes
7. **Clean up warnings** - Remove unused imports

This specification provides the complete canonical error system. Follow this exactly to ensure all error variant mismatches are resolved consistently across the codebase.

---

**🎯 Ready to implement? Start with Phase 1 and work systematically through each fix!**