# Assignment 06: Refactor Analysis Engine

## Priority: HIGH
## Estimated Time: 5-6 hours
## Directory: `/src/engine/`

## Objective
Refactor the analysis engine to improve modularity and reduce coupling between AST parsing and business logic.

## Current Problems
- Mixing high-level business logic with low-level AST parsing
- Large files with multiple responsibilities
- Tight coupling between parsing and analysis
- Inconsistent abstraction levels

## Tasks

### 1. Audit Analysis Engine Structure
```bash
find src/engine -name "*.rs" -exec wc -l {} + | sort -nr
find src/analysis -name "*.rs" -exec wc -l {} + | sort -nr
```

### 2. Create Clean Engine Architecture
```
src/engine/
├── mod.rs (public API)
├── parsing/
│   ├── mod.rs
│   ├── ast_builder.rs
│   ├── language_detection.rs
│   └── parsers/
│       ├── mod.rs
│       ├── rust_parser.rs
│       ├── python_parser.rs
│       ├── javascript_parser.rs
│       └── typescript_parser.rs
├── analysis/
│   ├── mod.rs
│   ├── context.rs (analysis context)
│   ├── visitor.rs (visitor pattern)
│   └── pipeline.rs (analysis pipeline)
├── knowledge_graph/
│   ├── mod.rs
│   ├── builder.rs
│   ├── query.rs
│   └── relations.rs
└── cache/
    ├── mod.rs
    ├── ast_cache.rs
    └── analysis_cache.rs
```

### 3. Separate Parsing from Analysis
Create clear boundary between parsing and analysis:

```rust
// In parsing/ast_builder.rs
pub struct AstBuilder {
    language_parsers: HashMap<Language, Box<dyn LanguageParser>>,
}

impl AstBuilder {
    pub fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        // Pure parsing logic, no business rules
    }
}

// In analysis/pipeline.rs
pub struct AnalysisPipeline {
    detectors: Vec<Box<dyn Detector>>,
    context_builder: ContextBuilder,
}

impl AnalysisPipeline {
    pub fn analyze(&self, parse_result: ParseResult) -> Result<AnalysisResult> {
        // Business logic and rule application
    }
}
```

### 4. Implement Language Parser Abstraction
Create consistent interface for all language parsers:

```rust
pub trait LanguageParser {
    fn language(&self) -> Language;
    fn parse(&self, source: &str) -> Result<SyntaxTree>;
    fn extract_symbols(&self, tree: &SyntaxTree) -> Vec<Symbol>;
    fn build_relations(&self, tree: &SyntaxTree) -> Vec<Relation>;
}
```

### 5. Extract Knowledge Graph Builder
- Move knowledge graph logic to dedicated module
- Create incremental graph building
- Implement efficient querying
- Target: <400 lines total

### 6. Implement Analysis Context
Create rich context for detectors:
```rust
pub struct AnalysisContext {
    pub file_info: FileInfo,
    pub syntax_tree: SyntaxTree,
    pub symbols: Vec<Symbol>,
    pub relations: Vec<Relation>,
    pub project_context: ProjectContext,
}
```

### 7. Create Visitor Pattern for AST Traversal
Implement visitor pattern for consistent AST traversal:
```rust
pub trait AstVisitor {
    fn visit_function(&mut self, node: &FunctionNode) -> VisitResult;
    fn visit_class(&mut self, node: &ClassNode) -> VisitResult;
    fn visit_module(&mut self, node: &ModuleNode) -> VisitResult;
}
```

### 8. Implement Caching Layer
- Create AST caching for parsed files
- Implement incremental analysis caching
- Use file modification time for cache invalidation
- Target: <300 lines total

### 9. Extract Configuration
- Move all engine configuration to dedicated structs
- Support runtime configuration updates
- Implement configuration validation
- Target: <200 lines

### 10. Update Integration Points
- Update detector interfaces to use new context
- Fix imports throughout codebase
- Ensure performance is maintained

## Performance Requirements
- Parsing performance must not degrade
- Memory usage should be optimized
- Support for incremental analysis
- Parallel processing capability maintained

## Success Criteria
- [ ] Clear separation between parsing and analysis
- [ ] All engine files <500 lines
- [ ] Language parser abstraction working
- [ ] Knowledge graph efficiently queryable
- [ ] Cache system functional
- [ ] All tests pass
- [ ] Performance maintained or improved

## Language Support Priority
1. Rust (primary language)
2. Python (common analysis target)
3. JavaScript/TypeScript (web projects)
4. Future language extensions

## Verification Commands
```bash
# Check file sizes
find src/engine -name "*.rs" -exec wc -l {} +

# Test parsing performance
cargo bench parse_large_files

# Test analysis pipeline
cargo test engine::analysis::

# Memory usage profiling
cargo run --release -- analyze large_project --profile-memory
```

## Completion Notes
_To be filled by AI developer:_
- Parsing/analysis separation achieved: ___
- Language parsers refactored: ___
- Performance impact: ___
- Cache hit rates: ___
- Memory usage change: ___