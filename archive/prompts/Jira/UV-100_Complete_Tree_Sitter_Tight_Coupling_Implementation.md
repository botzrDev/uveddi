# 🎯 UV-100: Complete Tree-sitter Integration and Detection Logic for Tight Coupling Analysis Framework

## 📋 **Task Overview**

**Jira Issue**: UV-100  
**Priority**: High  
**Type**: Story  
**Status**: On Deck  

**Objective**: Complete the Tree-sitter integration and detection logic for the sophisticated tight coupling analysis framework in the Uveddi static analysis tool. Transform the existing 197-line foundation into a production-ready, multi-language coupling detector.

---

## 🎯 **Current State Analysis**

### ✅ **Already Implemented (Strong Foundation)**
- **Advanced coupling metrics framework** (CBO, RFC, Fan-in, Fan-out) ✅
- **Dependency graph integration** with `LocalDependencyGraph` ✅  
- **Language analyzer framework** with extensible trait-based design ✅
- **Component modeling** supporting modules, classes, functions ✅
- **Graph-based calculations** with efficient metrics computation ✅
- **Multi-language AST parser** with LRU caching for Rust, Python, JavaScript ✅

### 🔧 **Implementation Gaps (Your Tasks)**
- [ ] **Tree-sitter Query Implementation** - Complete Rust dependency extraction
- [ ] **Threshold-based Detection** - Issue generation from metrics  
- [ ] **Configuration System** - Configurable thresholds and options
- [ ] **Python Language Analyzer** - Python-specific dependency extraction
- [ ] **JavaScript Language Analyzer** - JavaScript-specific dependency extraction
- [ ] **Cross-file Analysis Engine** - Project-wide coupling analysis

---

## 🏗️ **Architecture Context**

### **Existing Infrastructure You'll Work With:**

```rust
// Current tight coupling detector structure (src/analysis/detectors/anti_patterns/tight_coupling.rs)
pub struct TightCouplingDetector {
    // Your task: Add configuration fields
}

pub trait LanguageAnalyzer {
    fn extract_dependencies(&self, file_path: &Path, parsed_file: &ParsedFile) -> Vec<Dependency>;
}

pub struct RustAnalyzer; // Currently stub - needs full implementation

// Dependency graph already exists
pub struct LocalDependencyGraph {
    graph: DiGraph<ComponentNode, DependencyEdge>,
    node_map: HashMap<ComponentNode, NodeIndex>,
}

// AST parser with multi-language support already implemented
pub struct AstParser {
    parsers: HashMap<SourceLanguage, Parser>, // Rust, Python, JavaScript ready
    cache: Mutex<LruCache<String, ParsedFile>>, // LRU caching implemented
}
```

### **Component Integration Points:**
- **Tree-sitter Queries**: Use existing `tree_sitter::{Query, QueryCursor}` patterns
- **Detector Registry**: Integrate with existing `AnalysisDetector` trait
- **Configuration**: Follow patterns from `LargeClassConfig` in large_classes.rs
- **Error Handling**: Use `AnalysisError` and `UveddiError` patterns

---

## 📚 **Research Foundation**

Based on comprehensive research in `docs/06-research/Specialized/UV-100/UV-100_Research.md`, implement:

### **1. Tree-sitter Query Patterns**

#### **Rust Dependency Extraction Queries:**
```rust
// Use declarations
const RUST_USE_QUERY: &str = r#"
(use_declaration 
  argument: (scoped_use_list 
    list: (use_list 
      (scoped_identifier 
        path: (identifier) @module
        name: (identifier) @item))) @use_decl)

(use_declaration 
  argument: (scoped_identifier 
    path: (identifier) @module
    name: (identifier) @item)) @use_decl
"#;

// Function calls
const RUST_CALL_QUERY: &str = r#"
(call_expression
  function: (scoped_identifier
    path: (identifier) @module
    name: (identifier) @function)) @call

(call_expression
  function: (field_expression
    value: (identifier) @object
    field: (field_identifier) @method)) @method_call
"#;

// Struct instantiation
const RUST_STRUCT_QUERY: &str = r#"
(struct_expression
  name: (scoped_type_identifier
    path: (identifier) @module
    name: (type_identifier) @struct)) @instantiation
"#;
```

#### **Python Dependency Extraction Queries:**
```rust
const PYTHON_IMPORT_QUERY: &str = r#"
(import_statement
  name: (dotted_name) @module) @import

(import_from_statement
  module_name: (dotted_name) @module
  name: (dotted_name) @item) @from_import

(import_from_statement
  module_name: (dotted_name) @module
  name: (import_list
    (dotted_name) @item)) @from_import_list
"#;

const PYTHON_CALL_QUERY: &str = r#"
(call
  function: (attribute
    object: (identifier) @object
    attribute: (identifier) @method)) @method_call

(call
  function: (identifier) @function) @function_call
"#;
```

#### **JavaScript Dependency Extraction Queries:**
```rust
const JAVASCRIPT_IMPORT_QUERY: &str = r#"
(import_statement
  source: (string) @module) @import

(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (identifier) @item)))
  source: (string) @module) @named_import

(variable_declaration
  (variable_declarator
    name: (identifier) @var
    value: (call_expression
      function: (identifier) @require
      arguments: (arguments (string) @module)))) @require_call
"#;
```

### **2. Multi-Factorial Severity Model**

Implement context-aware thresholds based on research findings:

```rust
#[derive(Debug, Clone)]
pub struct CouplingThresholds {
    // Base thresholds
    pub fan_out_warning: usize,
    pub fan_out_critical: usize,
    pub cbo_warning: usize,
    pub cbo_critical: usize,
    pub rfc_warning: usize,
    pub rfc_critical: usize,
    
    // Context modifiers
    pub production_multiplier: f64,
    pub test_multiplier: f64,
    pub framework_multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct TightCouplingConfig {
    pub rust_thresholds: CouplingThresholds,
    pub python_thresholds: CouplingThresholds,
    pub javascript_thresholds: CouplingThresholds,
    pub enable_cross_file_analysis: bool,
    pub exclude_patterns: Vec<String>,
    pub include_test_files: bool,
}
```

### **3. Evidence-Based Threshold Values**

From academic research, implement these baseline thresholds:

| Language | Metric | Warning | Critical | Context |
|----------|--------|---------|----------|---------|
| Rust | Fan-out | 7 | 12 | Explicit dependencies |
| Rust | CBO | 6 | 10 | Strong typing |
| Python | Fan-out | 10 | 15 | Dynamic imports |
| Python | CBO | 8 | 12 | Duck typing |
| JavaScript | Fan-out | 12 | 18 | Module ecosystem |
| JavaScript | CBO | 10 | 15 | Prototype chains |

---

## 🎯 **Implementation Tasks**

### **Phase 1: Core Tree-sitter Integration (Priority 1)**

#### **Task 1.1: Complete RustAnalyzer Implementation**
```rust
// File: src/analysis/detectors/anti_patterns/tight_coupling.rs

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_dependencies(&self, file_path: &Path, parsed_file: &ParsedFile) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        
        // Extract use declarations
        dependencies.extend(self.extract_use_declarations(parsed_file));
        
        // Extract function calls
        dependencies.extend(self.extract_function_calls(parsed_file));
        
        // Extract struct instantiations
        dependencies.extend(self.extract_struct_instantiations(parsed_file));
        
        // Extract trait implementations
        dependencies.extend(self.extract_trait_implementations(parsed_file));
        
        dependencies
    }
}

impl RustAnalyzer {
    fn extract_use_declarations(&self, parsed_file: &ParsedFile) -> Vec<Dependency> {
        // TODO: Implement Tree-sitter query execution for use declarations
        // Use RUST_USE_QUERY pattern
    }
    
    fn extract_function_calls(&self, parsed_file: &ParsedFile) -> Vec<Dependency> {
        // TODO: Implement Tree-sitter query execution for function calls
        // Use RUST_CALL_QUERY pattern
    }
    
    // Additional extraction methods...
}
```

#### **Task 1.2: Implement Configuration System**
```rust
// File: src/analysis/detectors/anti_patterns/tight_coupling.rs

impl TightCouplingDetector {
    pub fn new(config: TightCouplingConfig) -> Self {
        Self { config }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(TightCouplingConfig::default())
    }
}

impl Default for TightCouplingConfig {
    fn default() -> Self {
        Self {
            rust_thresholds: CouplingThresholds {
                fan_out_warning: 7,
                fan_out_critical: 12,
                cbo_warning: 6,
                cbo_critical: 10,
                rfc_warning: 15,
                rfc_critical: 25,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            // Similar for Python and JavaScript...
            enable_cross_file_analysis: true,
            exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
            include_test_files: false,
        }
    }
}
```

#### **Task 1.3: Implement Threshold-Based Detection**
```rust
impl TightCouplingDetector {
    fn evaluate_coupling_issues(
        &self,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        language: SourceLanguage,
    ) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();
        let thresholds = self.get_thresholds_for_language(language);
        
        for (component, metric) in metrics {
            // Evaluate fan-out
            if metric.fan_out >= thresholds.fan_out_critical {
                issues.push(self.create_coupling_issue(
                    component,
                    "Critical",
                    format!("Fan-out {} exceeds critical threshold {}", 
                           metric.fan_out, thresholds.fan_out_critical),
                    metric,
                ));
            } else if metric.fan_out >= thresholds.fan_out_warning {
                issues.push(self.create_coupling_issue(
                    component,
                    "Warning", 
                    format!("Fan-out {} exceeds warning threshold {}", 
                           metric.fan_out, thresholds.fan_out_warning),
                    metric,
                ));
            }
            
            // Similar evaluation for CBO, RFC...
        }
        
        issues
    }
}
```

### **Phase 2: Multi-Language Support (Priority 2)**

#### **Task 2.1: Implement PythonAnalyzer**
```rust
pub struct PythonAnalyzer;

impl LanguageAnalyzer for PythonAnalyzer {
    fn extract_dependencies(&self, file_path: &Path, parsed_file: &ParsedFile) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        
        // Extract import statements
        dependencies.extend(self.extract_imports(parsed_file));
        
        // Extract function/method calls
        dependencies.extend(self.extract_calls(parsed_file));
        
        // Extract class inheritance
        dependencies.extend(self.extract_inheritance(parsed_file));
        
        dependencies
    }
}
```

#### **Task 2.2: Implement JavaScriptAnalyzer**
```rust
pub struct JavaScriptAnalyzer;

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn extract_dependencies(&self, file_path: &Path, parsed_file: &ParsedFile) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        
        // Extract ES6 imports
        dependencies.extend(self.extract_es6_imports(parsed_file));
        
        // Extract CommonJS requires
        dependencies.extend(self.extract_commonjs_requires(parsed_file));
        
        // Extract function calls
        dependencies.extend(self.extract_function_calls(parsed_file));
        
        dependencies
    }
}
```

### **Phase 3: Cross-File Analysis Engine (Priority 3)**

#### **Task 3.1: Extend Detection for Project-Wide Analysis**
```rust
impl AnalysisDetector for TightCouplingDetector {
    fn detect_issues(&self, parsed_file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Single-file analysis (existing)
        self.detect_single_file_issues(parsed_file)
    }
}

// Add new trait for cross-file analysis
pub trait CrossFileAnalysisDetector {
    fn detect_cross_file_issues(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
}

impl CrossFileAnalysisDetector for TightCouplingDetector {
    fn detect_cross_file_issues(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Build project-wide dependency graph
        let graph = self.build_project_dependency_graph(files)?;
        
        // Calculate metrics across all components
        let metrics = self.calculate_metrics(&graph);
        
        // Evaluate against thresholds
        let issues = self.evaluate_project_coupling_issues(&metrics, files);
        
        Ok(issues)
    }
}
```

### **Phase 4: Performance Optimization (Priority 4)**

#### **Task 4.1: Implement Incremental Analysis**
```rust
impl TightCouplingDetector {
    pub fn analyze_incrementally(
        &self,
        changed_files: &[String],
        existing_graph: &LocalDependencyGraph,
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        // Only re-analyze changed files and their dependents
        // Reuse existing graph structure where possible
    }
}
```

#### **Task 4.2: Add Parallel Processing**
```rust
use rayon::prelude::*;

impl TightCouplingDetector {
    fn extract_dependencies_parallel(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Vec<Vec<Dependency>> {
        files
            .par_iter()
            .map(|(path, parsed)| {
                let analyzer = self.get_analyzer_for_file(path);
                analyzer.extract_dependencies(Path::new(path), parsed)
            })
            .collect()
    }
}
```

---

## 🧪 **Testing Strategy**

### **Unit Tests (Required)**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_use_declaration_extraction() {
        // Test Tree-sitter query for Rust use statements
    }
    
    #[test]
    fn test_python_import_extraction() {
        // Test Tree-sitter query for Python imports
    }
    
    #[test]
    fn test_javascript_import_extraction() {
        // Test Tree-sitter query for JavaScript imports
    }
    
    #[test]
    fn test_threshold_evaluation() {
        // Test coupling threshold evaluation logic
    }
    
    #[test]
    fn test_cross_file_analysis() {
        // Test project-wide dependency analysis
    }
}
```

### **Integration Tests (Required)**
```rust
// File: tests/tight_coupling_integration.rs

#[test]
fn test_rust_project_coupling_analysis() {
    // Test with real Rust project structure
}

#[test]
fn test_multi_language_project_analysis() {
    // Test with mixed Rust/Python/JavaScript project
}

#[test]
fn test_performance_large_codebase() {
    // Performance test with large number of files
}
```

---

## 📊 **Success Criteria**

### **Functional Requirements:**
- [ ] **Tree-sitter Integration**: All three languages (Rust, Python, JavaScript) extract dependencies accurately
- [ ] **Metrics Calculation**: CBO, RFC, Fan-in, Fan-out calculated correctly for all component types
- [ ] **Threshold Detection**: Configurable thresholds trigger appropriate warnings/errors
- [ ] **Cross-file Analysis**: Project-wide dependency analysis works correctly
- [ ] **Configuration System**: YAML/JSON configuration loading and validation
- [ ] **Performance**: Handles projects with 1000+ files efficiently

### **Technical Requirements:**
- [ ] **Code Quality**: All code passes `cargo clippy` without warnings
- [ ] **Test Coverage**: >90% test coverage for new code
- [ ] **Documentation**: All public APIs documented with examples
- [ ] **Error Handling**: Robust error handling with informative messages
- [ ] **Integration**: Seamless integration with existing detector registry

### **Performance Requirements:**
- [ ] **Speed**: Analysis completes within 30 seconds for 1000-file project
- [ ] **Memory**: Memory usage stays under 500MB for large projects
- [ ] **Caching**: Effective use of existing AST cache system
- [ ] **Incremental**: Support for incremental analysis of changed files

---

## 🔧 **Implementation Guidelines**

### **Code Style & Patterns:**
1. **Follow existing patterns** from `code_duplication.rs` and `large_classes.rs`
2. **Use Tree-sitter queries** similar to existing detector implementations
3. **Implement proper error handling** with `Result<T, AnalysisError>` pattern
4. **Add comprehensive logging** using `tracing` crate
5. **Follow Rust naming conventions** (snake_case, PascalCase)

### **Integration Points:**
1. **Detector Registry**: Register with existing `AnalysisDetector` trait
2. **Configuration**: Follow `LargeClassConfig` pattern for configuration
3. **AST Parser**: Use existing `AstParser` with LRU caching
4. **Dependency Graph**: Extend existing `LocalDependencyGraph` structure
5. **Error Types**: Use existing `AnalysisError` and `UveddiError` types

### **Performance Considerations:**
1. **Leverage AST Cache**: Use existing LRU cache for parsed files
2. **Minimize Memory**: Use references where possible, avoid cloning large structures
3. **Parallel Processing**: Use `rayon` for CPU-intensive operations
4. **Incremental Analysis**: Support for analyzing only changed files
5. **Query Optimization**: Efficient Tree-sitter query execution

---

## 📝 **Deliverables**

### **Code Files to Modify/Create:**
1. **`src/analysis/detectors/anti_patterns/tight_coupling.rs`** - Complete implementation
2. **`tests/tight_coupling_integration.rs`** - Integration tests
3. **`examples/tight_coupling_config.toml`** - Configuration example
4. **Documentation updates** - API docs and usage examples

### **Documentation Requirements:**
1. **API Documentation** - All public functions documented
2. **Configuration Guide** - How to configure thresholds
3. **Usage Examples** - Code examples for each language
4. **Performance Guide** - Optimization recommendations

---

## 🚀 **Getting Started**

### **Step 1: Environment Setup**
```bash
# Ensure Tree-sitter feature is enabled
cargo check --features tree-sitter

# Run existing tests to ensure baseline
cargo test tight_coupling

# Check current implementation
cargo doc --open
```

### **Step 2: Implementation Order**
1. **Start with RustAnalyzer** - Implement Tree-sitter queries for Rust
2. **Add Configuration System** - Implement threshold configuration
3. **Implement Threshold Detection** - Add issue generation logic
4. **Add PythonAnalyzer** - Extend to Python support
5. **Add JavaScriptAnalyzer** - Extend to JavaScript support
6. **Implement Cross-file Analysis** - Project-wide analysis
7. **Performance Optimization** - Parallel processing and caching

### **Step 3: Testing & Validation**
```bash
# Run unit tests
cargo test tight_coupling

# Run integration tests
cargo test --test tight_coupling_integration

# Performance testing
cargo bench tight_coupling

# Code quality checks
cargo clippy -- -D warnings
cargo fmt --check
```

---

## 📞 **Support & Resources**

### **Key Files to Reference:**
- **Research**: `docs/06-research/Specialized/UV-100/UV-100_Research.md`
- **Existing Detectors**: `src/analysis/detectors/anti_patterns/`
- **AST Implementation**: `src/ast/tree_sitter_impl.rs`
- **Dependency Graph**: `src/analysis/graph/dependency.rs`

### **Tree-sitter Resources:**
- **Tree-sitter Documentation**: https://tree-sitter.github.io/
- **Rust Grammar**: https://github.com/tree-sitter/tree-sitter-rust
- **Python Grammar**: https://github.com/tree-sitter/tree-sitter-python
- **JavaScript Grammar**: https://github.com/tree-sitter/tree-sitter-javascript

### **Questions & Clarifications:**
For any questions about implementation details, architecture decisions, or research interpretation, please refer to the comprehensive research document or ask for clarification.

---

**🎯 Ready to transform the tight coupling analysis framework into a production-ready, multi-language coupling detector! Let's build something amazing! 🚀**