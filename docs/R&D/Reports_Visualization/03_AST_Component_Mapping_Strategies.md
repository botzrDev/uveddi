# AST-to-Component Mapping Strategies Research

**Research Request #3**  
**Priority:** MEDIUM  
**Status:** PENDING RESEARCH  
**Assigned To:** [Assistant Name]  
**Due Date:** End of Week 1-2  

## Research Objectives

Design and specify the data flow and algorithms for extracting architectural components from AST data, enabling accurate and performant diagram generation across multiple programming languages.

## Research Areas

### 1. Component Extraction Patterns

#### Current State Analysis:
```rust
// Existing ComponentNode enum (to be enhanced)
pub enum ComponentNode {
    Module { path: String },
    Class { name: String, file_path: String },
    Function { name: String, file_path: String },
}
```

#### Enhanced Component Model Requirements:
```rust
// Proposed ArchitecturalComponent structure
pub struct ArchitecturalComponent {
    pub component_id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub component_type: ComponentType,
    pub dependencies: Vec<Dependency>,
    pub metrics: ComponentMetrics,
    pub source_location: SourceLocation,
    pub visibility: Visibility,
}

pub enum ComponentType {
    // Rust-specific
    Module { is_public: bool },
    Struct { fields: Vec<FieldInfo> },
    Enum { variants: Vec<VariantInfo> },
    Trait { methods: Vec<MethodSignature> },
    Impl { target: String, trait_impl: Option<String> },
    Function { signature: FunctionSignature },
    
    // Python-specific
    Class { bases: Vec<String>, methods: Vec<MethodInfo> },
    Method { is_static: bool, is_class_method: bool },
    
    // JavaScript-specific
    EsModule { exports: Vec<ExportInfo> },
    Class { extends: Option<String> },
    Function { is_async: bool, is_generator: bool },
    
    // Universal
    Variable { data_type: Option<String> },
    Constant { value: Option<String> },
}
```

### 2. Multi-Language Component Mapping

#### Rust Component Extraction:
```rust
// Tree-sitter queries for Rust components
const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @struct_name
  body: (field_declaration_list
    (field_declaration
      name: (field_identifier) @field_name
      type: (_) @field_type)?)*) @struct_body
"#;

const RUST_IMPL_QUERY: &str = r#"
(impl_item
  type: (type_identifier) @impl_target
  body: (declaration_list
    (function_item
      name: (identifier) @method_name
      parameters: (parameters) @method_params
      return_type: (type_annotation)? @return_type)?)*) @impl_body
"#;
```

#### Python Component Extraction:
```rust
const PYTHON_CLASS_QUERY: &str = r#"
(class_definition
  name: (identifier) @class_name
  superclasses: (argument_list
    (identifier) @base_class)*
  body: (block
    (function_definition
      name: (identifier) @method_name
      parameters: (parameters) @method_params)*)) @class_body
"#;
```

#### JavaScript Component Extraction:
```rust
const JS_CLASS_QUERY: &str = r#"
(class_declaration
  name: (identifier) @class_name
  superclass: (identifier)? @extends
  body: (class_body
    (method_definition
      name: (property_identifier) @method_name
      parameters: (formal_parameters) @method_params)*)) @class_body
"#;
```

### 3. Dependency Relationship Extraction

#### Dependency Types to Extract:
```rust
pub enum DependencyRelationship {
    // Static dependencies
    Import { module: String, items: Vec<String> },
    Use { path: String, alias: Option<String> },
    Extends { parent: String },
    Implements { trait_name: String },
    
    // Dynamic dependencies
    MethodCall { target: String, method: String },
    FieldAccess { target: String, field: String },
    Instantiation { class: String },
    
    // Composition relationships
    HasField { field_type: String },
    Contains { element_type: String },
    Aggregates { component: String },
}
```

#### Cross-File Symbol Resolution:
```rust
pub struct SymbolResolver {
    symbol_table: GlobalSymbolTable,
    import_graph: ImportGraph,
    type_registry: TypeRegistry,
}

impl SymbolResolver {
    pub fn resolve_symbol(&self, symbol: &str, context: &FileContext) -> Option<ResolvedSymbol> {
        // Multi-step resolution:
        // 1. Local scope
        // 2. File-level imports
        // 3. Module-level exports
        // 4. External dependencies
    }
    
    pub fn build_dependency_graph(&self, components: &[ArchitecturalComponent]) -> DependencyGraph {
        // Build complete dependency graph with resolved symbols
    }
}
```

## Data Model Design Research

### 1. Component Hierarchy Representation

#### Tree Structure:
```rust
pub struct ComponentHierarchy {
    pub root: ComponentNode,
    pub children: HashMap<Uuid, Vec<ComponentNode>>,
    pub parent_map: HashMap<Uuid, Uuid>,
}

impl ComponentHierarchy {
    pub fn find_path(&self, from: Uuid, to: Uuid) -> Option<Vec<Uuid>> {
        // Find path between components for dependency visualization
    }
    
    pub fn get_siblings(&self, component_id: Uuid) -> Vec<&ComponentNode> {
        // Get components at same hierarchy level
    }
    
    pub fn get_descendants(&self, component_id: Uuid) -> Vec<&ComponentNode> {
        // Get all nested components
    }
}
```

### 2. Performance Optimization Strategies

#### Incremental Parsing:
```rust
pub struct IncrementalComponentExtractor {
    file_cache: HashMap<PathBuf, FileFingerprint>,
    component_cache: HashMap<PathBuf, Vec<ArchitecturalComponent>>,
    dependency_cache: HashMap<PathBuf, Vec<DependencyRelationship>>,
}

impl IncrementalComponentExtractor {
    pub fn extract_components_incremental(
        &mut self,
        files: &[PathBuf],
    ) -> Result<Vec<ArchitecturalComponent>, ExtractionError> {
        // Only re-parse changed files
        // Update dependency graph incrementally
        // Maintain cache consistency
    }
}
```

#### Memory-Efficient Storage:
```rust
pub struct CompactComponent {
    pub id: u32,              // Compact ID instead of UUID
    pub name_id: u32,         // String interning
    pub file_id: u32,         // File path interning
    pub component_type: u8,   // Enum as byte
    pub metrics: CompactMetrics,
}

pub struct StringInterner {
    strings: Vec<String>,
    string_to_id: HashMap<String, u32>,
}
```

## Integration with Existing Systems

### 1. AST Parser Integration:
```rust
// Enhanced AstParser with component extraction
impl AstParser {
    pub fn extract_components(&self, parsed_file: &ParsedFile) -> Result<Vec<ArchitecturalComponent>, ExtractionError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_components(parsed_file),
            SourceLanguage::Python => self.extract_python_components(parsed_file),
            SourceLanguage::JavaScript => self.extract_js_components(parsed_file),
        }
    }
    
    pub fn extract_dependencies(&self, components: &[ArchitecturalComponent]) -> Vec<DependencyRelationship> {
        // Extract relationships between components
    }
}
```

### 2. LocalDependencyGraph Enhancement:
```rust
// Enhanced dependency graph with component metadata
pub struct EnhancedDependencyGraph {
    pub graph: DiGraph<ArchitecturalComponent, DependencyRelationship>,
    pub component_index: HashMap<Uuid, NodeIndex>,
    pub file_to_components: HashMap<PathBuf, Vec<Uuid>>,
    pub type_hierarchy: TypeHierarchy,
}

impl EnhancedDependencyGraph {
    pub fn find_cycles(&self) -> Vec<Vec<Uuid>> {
        // Enhanced cycle detection with component context
    }
    
    pub fn calculate_coupling_metrics(&self) -> HashMap<Uuid, CouplingMetrics> {
        // Calculate afferent/efferent coupling per component
    }
    
    pub fn get_component_dependencies(&self, component_id: Uuid) -> Vec<&DependencyRelationship> {
        // Get all dependencies for a specific component
    }
}
```

## Expected Deliverables

### 1. Component Extraction Algorithm Specification
```rust
pub trait ComponentExtractor {
    fn extract_components(&self, ast: &Tree, source: &str) -> Result<Vec<ArchitecturalComponent>, ExtractionError>;
    fn extract_dependencies(&self, components: &[ArchitecturalComponent]) -> Vec<DependencyRelationship>;
    fn supported_languages(&self) -> Vec<SourceLanguage>;
}

pub struct MultiLanguageExtractor {
    rust_extractor: RustComponentExtractor,
    python_extractor: PythonComponentExtractor,
    js_extractor: JavaScriptComponentExtractor,
}
```

### 2. Enhanced Data Models
- Complete `ArchitecturalComponent` implementation
- `ComponentMetrics` for size/complexity measurement
- `DependencyRelationship` with relationship types
- `ComponentHierarchy` for nested structures

### 3. Performance Benchmarks
- Extraction time vs. file size
- Memory usage patterns
- Cache hit rates
- Incremental parsing benefits

### 4. Integration Plan
- Migration path from current `ComponentNode`
- Backward compatibility considerations
- Database schema updates
- API changes required

## Research Questions

1. How can we efficiently handle large codebases (1000+ files)?
2. What's the optimal granularity for component extraction?
3. How do we handle dynamic language features (Python duck typing, JS prototypes)?
4. What caching strategies provide the best performance?
5. How can we maintain accuracy while optimizing for speed?
6. What are the memory vs. accuracy trade-offs?

## Success Criteria

- [ ] Complete component extraction algorithm for all 3 languages
- [ ] Performance benchmarks showing <100ms per file
- [ ] Memory usage under 50MB for 1000-file codebase
- [ ] 95%+ accuracy in dependency detection
- [ ] Integration plan with existing codebase
- [ ] Comprehensive test suite with edge cases

## Notes

*This section will be populated by the research assistant with algorithm designs, performance data, and implementation recommendations.*

---

**Status Updates:**
- [ ] Research initiated
- [ ] Algorithm design completed
- [ ] Performance analysis conducted
- [ ] Data models finalized
- [ ] Integration plan documented
- [ ] Prototype implemented