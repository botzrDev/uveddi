# AnalysisEngine Decomposition Plan (UV-104)

## Current God Object Analysis
**File**: `src/analysis/engine.rs`  
**Size**: 2,419 lines  
**Dependencies**: 35+ direct dependencies  
**Issue**: Violates Single Responsibility Principle  

## Decomposition Strategy

### Target Architecture (Component-Based)
Based on the existing `src/analysis/components/` structure, we'll complete the decomposition:

```
┌─────────────────────────────────────────────────────┐
│                AnalysisOrchestrator                 │ 
│                  (Facade Pattern)                   │
└─────────────────────┬───────────────────────────────┘
                      │
    ┌─────────────────┼─────────────────┐
    │                 │                 │
┌───▼────┐    ┌──────▼─────┐    ┌──────▼─────┐
│Analysis│    │Dependency  │    │Performance │
│Service │    │Service     │    │Service     │
└────────┘    └────────────┘    └────────────┘
```

### Phase 2.1: Core Service Extraction (Week 2-3)

#### 1. AnalysisService
**Responsibility**: Core analysis coordination  
**Location**: `src/analysis/services/analysis_service.rs`

```rust
pub struct AnalysisService {
    config_service: Arc<ConfigurationService>,
    detector_scheduler: Arc<DetectorScheduler>,
    aggregator: Arc<AnalysisAggregator>,
}

impl AnalysisService {
    pub async fn run_analysis(&self, path: &Path) -> AnalysisResult<Vec<ArchitecturalIssue>>;
    pub async fn run_file_analysis(&self, file: &Path) -> AnalysisResult<Vec<ArchitecturalIssue>>;
    pub fn get_supported_detectors(&self) -> Vec<DetectorInfo>;
}
```

#### 2. DependencyAnalysisService  
**Responsibility**: Dependency graph analysis  
**Location**: `src/analysis/services/dependency_service.rs`

```rust
pub struct DependencyAnalysisService {
    ast_provider: Arc<AstProviderImpl>,
    dependency_builder: Arc<DependencyGraphBuilderImpl>,
    cache_manager: Arc<CacheManagerImpl>,
}

impl DependencyAnalysisService {
    pub async fn build_dependency_graph(&self, path: &Path) -> AnalysisResult<LocalDependencyGraph>;
    pub async fn analyze_cycles(&self, graph: &LocalDependencyGraph) -> Vec<CycleDependency>;
    pub async fn extract_dependencies(&self, files: &[ParsedFile]) -> Vec<Dependency>;
}
```

#### 3. PerformanceAnalysisService
**Responsibility**: Memory and performance monitoring  
**Location**: `src/analysis/services/performance_service.rs`

```rust
pub struct PerformanceAnalysisService {
    metrics_collector: Arc<PerformanceMetricsCollector>,
    memory_monitor: MemoryMonitor,
}

impl PerformanceAnalysisService {
    pub fn start_monitoring(&self) -> MonitoringSession;
    pub async fn analyze_with_memory_limits(&self, analysis: impl Future) -> AnalysisResult<T>;
    pub fn get_performance_report(&self) -> PerformanceReport;
}
```

### Phase 2.2: Enhanced Facade Pattern (Week 3-4)

#### New AnalysisOrchestrator
**Purpose**: Simplified coordinator replacing the God Object  
**Location**: `src/analysis/orchestrator.rs`

```rust
pub struct AnalysisOrchestrator {
    analysis_service: Arc<AnalysisService>,
    dependency_service: Arc<DependencyAnalysisService>, 
    performance_service: Arc<PerformanceAnalysisService>,
    
    // Optional AI services (feature-gated)
    #[cfg(feature = "ai")]
    ai_service: Option<Arc<AiAnalysisService>>,
}

impl AnalysisOrchestrator {
    // Simple public API maintaining backward compatibility
    pub async fn analyze(&self, path: &Path) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        // Coordinate between services
        let (issues, graph) = tokio::try_join!(
            self.analysis_service.run_analysis(path),
            self.dependency_service.build_dependency_graph(path)
        )?;
        
        Ok((issues, graph))
    }
    
    pub async fn analyze_with_performance_monitoring(&self, path: &Path) -> EnhancedAnalysisResult {
        let _session = self.performance_service.start_monitoring();
        let result = self.analyze(path).await?;
        let performance_report = self.performance_service.get_performance_report();
        
        Ok(EnhancedAnalysisResult { result, performance_report })
    }
}
```

### Phase 2.3: Migration Strategy (Week 4-5)

#### Step 1: Maintain Backward Compatibility
```rust
// src/analysis/engine.rs (simplified)
pub struct AnalysisEngine {
    orchestrator: AnalysisOrchestrator,
}

impl AnalysisEngine {
    pub fn new() -> AnalysisResult<Self> {
        let orchestrator = AnalysisOrchestrator::new()?;
        Ok(Self { orchestrator })
    }
    
    // Delegate to orchestrator
    pub async fn analyze(&mut self, path: &Path) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        self.orchestrator.analyze(path).await
    }
}
```

#### Step 2: Gradual Component Migration
1. **Week 4**: Extract AnalysisService from current engine
2. **Week 5**: Extract DependencyAnalysisService  
3. **Week 6**: Extract PerformanceAnalysisService
4. **Week 7**: Replace engine internals with orchestrator
5. **Week 8**: Testing and performance validation

### Phase 2.4: Dependency Injection Enhancement (Week 6-7)

#### Enhanced Builder Pattern
```rust
pub struct AnalysisOrchestratorBuilder {
    analysis_service: Option<Arc<AnalysisService>>,
    dependency_service: Option<Arc<DependencyAnalysisService>>,
    performance_service: Option<Arc<PerformanceAnalysisService>>,
}

impl AnalysisOrchestratorBuilder {
    pub fn with_custom_analysis_service(mut self, service: Arc<AnalysisService>) -> Self {
        self.analysis_service = Some(service);
        self
    }
    
    pub fn build(self) -> AnalysisResult<AnalysisOrchestrator> {
        // Build with provided or default services
    }
}
```

## Benefits of This Approach

### 1. **Single Responsibility Principle**
- Each service has a clear, focused responsibility
- Easier to understand, test, and maintain

### 2. **Testability**
- Mock individual services for isolated testing
- Dependency injection enables comprehensive unit testing

### 3. **Parallel Development**
- Teams can work on different services simultaneously
- Reduced merge conflicts and development bottlenecks

### 4. **Performance**
- Services can be optimized independently
- Better resource management and caching strategies

### 5. **Backward Compatibility**
- Existing code continues to work unchanged
- Gradual migration path for consumers

## Implementation Timeline

### Week 2-3: Service Extraction
- [ ] Create service interfaces and base implementations
- [ ] Extract AnalysisService from engine.rs
- [ ] Extract DependencyAnalysisService from engine.rs
- [ ] Extract PerformanceAnalysisService from engine.rs

### Week 4-5: Orchestrator Implementation
- [ ] Implement AnalysisOrchestrator with facade pattern
- [ ] Update AnalysisEngine to delegate to orchestrator
- [ ] Comprehensive testing of new architecture

### Week 6-7: Enhancement and Optimization
- [ ] Enhanced dependency injection
- [ ] Performance optimization
- [ ] Documentation updates

### Week 8: Validation and Performance Testing
- [ ] End-to-end integration testing
- [ ] Performance regression testing
- [ ] Memory usage validation
- [ ] Production readiness assessment

## Success Metrics

### Quantitative Targets
- [ ] Reduce largest single file from 2,419 lines to <500 lines
- [ ] Achieve >90% test coverage for each service
- [ ] Maintain or improve analysis performance (<5% regression)
- [ ] Reduce coupling metrics by 60%

### Qualitative Improvements
- [ ] Clear separation of concerns
- [ ] Improved code maintainability
- [ ] Enhanced testability
- [ ] Better documentation and examples

## Risk Mitigation

### Technical Risks
- **Performance regression**: Comprehensive benchmarking before/after
- **Breaking changes**: Maintain strict backward compatibility
- **Integration issues**: Incremental migration with rollback points

### Process Risks
- **Development velocity**: Parallel work streams where possible
- **Testing complexity**: Automated testing for all migration steps
- **Documentation debt**: Continuous documentation updates
