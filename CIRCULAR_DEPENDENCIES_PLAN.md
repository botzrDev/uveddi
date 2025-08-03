# Circular Dependencies Resolution Plan (UV-105)

## Current Circular Dependency Issues
**Problem**: 2,045 bidirectional dependencies identified  
**Root Cause**: AST module at center of dependency cycles  
**Impact**: Fragile system, prevents modularization  

## Dependency Inversion Strategy

### Phase 3.1: Identify Dependency Cycles (Week 2)

#### Core Circular Dependencies Analysis
Based on the diagnostic report, major cycles involve:

1. **AST ↔ Analysis Engine**
2. **Analysis Engine ↔ Detectors** 
3. **Cache ↔ AST Provider**
4. **Plugin System ↔ Analysis Engine**

### Phase 3.2: Interface Segregation (Week 2-3)

#### 1. AST Provider Interface
**Current Issue**: Direct dependency on concrete AST implementation  
**Solution**: Interface segregation with dependency inversion

```rust
// src/analysis/traits/ast_provider.rs
pub trait AstProvider: Send + Sync {
    async fn parse_file(&self, path: &Path) -> AnalysisResult<ParsedFile>;
    async fn parse_content(&self, content: &str, language: SourceLanguage) -> AnalysisResult<ParsedFile>;
    fn supports_language(&self, language: SourceLanguage) -> bool;
}

// src/analysis/traits/ast_cache.rs  
pub trait AstCache: Send + Sync {
    async fn get_cached_ast(&self, key: &str) -> Option<ParsedFile>;
    async fn cache_ast(&self, key: String, ast: ParsedFile) -> AnalysisResult<()>;
    async fn invalidate(&self, key: &str) -> AnalysisResult<()>;
}
```

#### 2. Detector Interface Refinement
**Current Issue**: Detectors depend on concrete engine implementation  
**Solution**: Lightweight context passing

```rust
// src/analysis/traits/detector.rs
pub trait AnalysisDetector: Send + Sync {
    async fn detect_issues(&self, context: &DetectionContext) -> AnalysisResult<Vec<ArchitecturalIssue>>;
    fn get_detector_name(&self) -> &'static str;
    fn get_supported_languages(&self) -> Vec<SourceLanguage>;
}

// Context object to break circular dependencies
pub struct DetectionContext {
    pub parsed_file: ParsedFile,
    pub project_context: ProjectContext,
    pub cache_provider: Arc<dyn CacheProvider>,
}
```

#### 3. Cache Provider Interface  
**Current Issue**: Cache directly depends on AST types, creating cycles  
**Solution**: Generic cache interface with type erasure

```rust
// src/analysis/traits/cache_provider.rs
pub trait CacheProvider: Send + Sync {
    async fn get<T>(&self, key: &str) -> AnalysisResult<Option<T>>
    where T: serde::DeserializeOwned + Send + 'static;
    
    async fn set<T>(&self, key: String, value: T) -> AnalysisResult<()>
    where T: serde::Serialize + Send + 'static;
    
    async fn invalidate(&self, key: &str) -> AnalysisResult<()>;
}
```

### Phase 3.3: Dependency Graph Restructuring (Week 3)

#### Target Dependency Flow
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │    │    Analysis     │    │  Infrastructure │
│     Layer       │───▶│     Layer       │───▶│     Layer       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │                         ▲
                              └─────────────────────────┘
                                   (Dependency Inversion)
```

#### Concrete Implementation:

**1. Application Layer** (depends on Analysis Layer)
- CLI commands
- Configuration management
- Result presentation

**2. Analysis Layer** (depends on Infrastructure abstractions)
- AnalysisOrchestrator (replaces AnalysisEngine God Object)
- AnalysisService, DependencyService, PerformanceService
- Detector implementations

**3. Infrastructure Layer** (no upward dependencies)
- AST Provider implementation
- Cache implementation  
- Database operations
- Plugin runtime

### Phase 3.4: Implementation Steps (Week 3-4)

#### Step 1: Extract Interfaces (3 days)
```rust
// Create trait definitions
mkdir -p src/analysis/traits
touch src/analysis/traits/{ast_provider,cache_provider,detector,dependency_extractor}.rs
```

#### Step 2: Implement Dependency Injection (4 days)
```rust
// Enhanced service constructors with DI
impl AnalysisService {
    pub fn new(
        ast_provider: Arc<dyn AstProvider>,
        cache_provider: Arc<dyn CacheProvider>,
        detectors: Vec<Box<dyn AnalysisDetector>>,
    ) -> Self {
        // Construction with injected dependencies
    }
}
```

#### Step 3: Migrate Concrete Implementations (5 days)
```rust
// Update existing implementations to use interfaces
impl AstProvider for AstProviderImpl { /* implementation */ }
impl CacheProvider for CacheManagerImpl { /* implementation */ }
```

#### Step 4: Remove Circular References (2 days)
- Update import statements
- Remove bidirectional dependencies
- Validate dependency graph is acyclic

### Phase 3.5: Validation and Testing (Week 4)

#### Dependency Analysis Validation
```bash
# Add cargo-modules for dependency visualization
cargo install cargo-modules
cargo modules generate graph --with-types > dependency_graph.dot
dot -Tpng dependency_graph.dot -o dependency_graph.png

# Verify no cycles exist
cargo modules dependencies --package uveddi --no-dev-dependencies | grep -i cycle
```

#### Automated Dependency Checks
```toml
# Add to Cargo.toml
[dev-dependencies]
cargo-deny = "0.14"

# Create deny.toml with dependency rules
```

```toml
# deny.toml
[graph]
targets = []
all-features = false
no-default-features = false

[[graph.exclude]]
name = "uveddi"
# Prevent circular dependencies
deny-multiple-versions = "warn"
```

## Benefits of This Approach

### 1. **Eliminated Circular Dependencies**
- Clean dependency flow: Application → Analysis → Infrastructure  
- No bidirectional coupling between layers
- Easier to reason about and modify

### 2. **Enhanced Testability**
- Mock interfaces for unit testing
- Isolated component testing
- Integration testing with controlled dependencies

### 3. **Improved Modularity** 
- Clear separation of concerns
- Independent development of components
- Plugin system becomes truly isolated

### 4. **Better Performance**
- Reduced compilation times due to eliminated cycles
- More efficient linking and optimization
- Cleaner memory management

## Implementation Timeline

### Week 2: Analysis and Interface Definition
- [ ] Complete dependency cycle analysis
- [ ] Define core interfaces (AstProvider, CacheProvider, etc.)
- [ ] Create trait definitions and documentation

### Week 3: Dependency Inversion Implementation
- [ ] Implement dependency injection for services
- [ ] Update service constructors with DI
- [ ] Migrate concrete implementations to use interfaces

### Week 4: Migration and Validation
- [ ] Remove circular references
- [ ] Update all import statements  
- [ ] Comprehensive dependency validation
- [ ] Performance regression testing

## Success Metrics

### Quantitative Targets
- [ ] Reduce bidirectional dependencies from 2,045 to <50
- [ ] Achieve 100% acyclic dependency graph
- [ ] Maintain or improve compilation times
- [ ] Zero dependency-related warnings

### Qualitative Improvements
- [ ] Clear layered architecture
- [ ] Improved code organization
- [ ] Enhanced maintainability
- [ ] Better separation of concerns

## Risk Mitigation

### Technical Risks
- **Breaking changes**: Use adapter pattern during migration
- **Performance impact**: Benchmark before/after changes
- **Complex refactoring**: Incremental approach with validation points

### Process Risks
- **Development coordination**: Clear interface contracts upfront
- **Testing complexity**: Automated dependency validation in CI
- **Integration issues**: Continuous integration testing
