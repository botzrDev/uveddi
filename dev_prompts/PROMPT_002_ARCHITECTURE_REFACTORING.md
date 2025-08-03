# GPT Dev Prompt 002: AnalysisEngine God Object Decomposition

## 🏗️ HIGH PRIORITY - Week 2-3 Implementation

### **Issue**: UV-104 - AnalysisEngine God Object Refactoring
**Reference Document**: `ARCHITECTURE_REFACTORING_PLAN.md`  
**Jira Issue**: UV-104  
**Priority**: HIGH - Major Technical Debt  
**Estimated Time**: 2 weeks  
**Dependencies**: Complete UV-103 (Security Fixes) first

---

## **TASK OVERVIEW**

The current `AnalysisEngine` in `src/analysis/engine.rs` is a God Object with 2,419 lines and 35+ dependencies. You need to decompose it into specialized services following the Component-Based Architecture pattern.

**Current State**: Monolithic God Object  
**Target State**: Component-based services with Facade pattern  

---

## **DETAILED REQUIREMENTS**

### **Current God Object Analysis**
- **File**: `src/analysis/engine.rs` (2,419 lines)
- **Dependencies**: 35+ direct dependencies
- **Issue**: Violates Single Responsibility Principle
- **Impact**: Blocks parallel development, difficult to test/maintain

### **Target Architecture**

```
┌─────────────────────────────────────────────────────┐
│                AnalysisOrchestrator                 │ 
│                  (Facade Pattern)                   │
│                     (~200 lines)                    │
└─────────────────────┬───────────────────────────────┘
                      │
    ┌─────────────────┼─────────────────┐
    │                 │                 │
┌───▼────┐    ┌──────▼─────┐    ┌──────▼─────┐
│Analysis│    │Dependency  │    │Performance │
│Service │    │Service     │    │Service     │
│~600 L  │    │~500 L      │    │~400 L      │
└────────┘    └────────────┘    └────────────┘
```

---

## **IMPLEMENTATION PHASES**

### **Phase 1: Service Extraction (Week 2)**

#### **1.1 Create AnalysisService (~600 lines)**
**Location**: `src/analysis/services/analysis_service.rs`  
**Responsibility**: Core analysis coordination

```rust
// src/analysis/services/analysis_service.rs
use crate::analysis::components::{ConfigurationService, DetectorScheduler, AnalysisAggregator};
use crate::database::models::ArchitecturalIssue;
use std::path::Path;
use std::sync::Arc;

/// Core analysis service responsible for coordinating analysis operations
pub struct AnalysisService {
    config_service: Arc<ConfigurationService>,
    detector_scheduler: Arc<DetectorScheduler>,
    aggregator: Arc<AnalysisAggregator>,
}

impl AnalysisService {
    /// Create new analysis service with injected dependencies
    pub fn new(
        config_service: Arc<ConfigurationService>,
        detector_scheduler: Arc<DetectorScheduler>,
        aggregator: Arc<AnalysisAggregator>,
    ) -> Self {
        Self {
            config_service,
            detector_scheduler,
            aggregator,
        }
    }
    
    /// Run comprehensive analysis on a path
    pub async fn run_analysis(&self, path: &Path) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        log::info!("Starting analysis for path: {}", path.display());
        
        // Coordinate detector execution
        let issues = if path.is_file() {
            self.detector_scheduler.schedule_file(path).await?
        } else {
            self.detector_scheduler.schedule_directory(path).await?
        };
        
        // Record findings
        self.aggregator.record_findings(issues.clone());
        
        log::info!("Analysis completed: {} issues found", issues.len());
        Ok(issues)
    }
    
    /// Run analysis on a single file
    pub async fn run_file_analysis(&self, file: &Path) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        self.detector_scheduler.schedule_file(file).await
    }
    
    /// Get supported detector information
    pub fn get_supported_detectors(&self) -> Vec<DetectorInfo> {
        self.detector_scheduler.get_detector_info()
    }
    
    /// Get analysis statistics
    pub fn get_analysis_stats(&self) -> AnalysisStats {
        self.aggregator.get_stats()
    }
}
```

**Tasks for AnalysisService**:
- [ ] Extract analysis coordination logic from `engine.rs`
- [ ] Implement file vs directory analysis routing  
- [ ] Add detector scheduling and result aggregation
- [ ] Create comprehensive unit tests
- [ ] Document service API and usage

#### **1.2 Create DependencyAnalysisService (~500 lines)**
**Location**: `src/analysis/services/dependency_service.rs`  
**Responsibility**: Dependency graph analysis

```rust
// src/analysis/services/dependency_service.rs
use crate::analysis::components::{AstProviderImpl, DependencyGraphBuilderImpl, CacheManagerImpl};
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::detectors::dependency::Dependency;
use crate::ast::ParsedFile;
use std::path::Path;
use std::sync::Arc;

/// Service responsible for dependency analysis and graph construction
pub struct DependencyAnalysisService {
    ast_provider: Arc<AstProviderImpl>,
    dependency_builder: Arc<DependencyGraphBuilderImpl>,
    cache_manager: Arc<CacheManagerImpl>,
}

impl DependencyAnalysisService {
    /// Create new dependency analysis service
    pub fn new(
        ast_provider: Arc<AstProviderImpl>,
        dependency_builder: Arc<DependencyGraphBuilderImpl>,
        cache_manager: Arc<CacheManagerImpl>,
    ) -> Self {
        Self {
            ast_provider,
            dependency_builder,
            cache_manager,
        }
    }
    
    /// Build comprehensive dependency graph for a path
    pub async fn build_dependency_graph(&self, path: &Path) -> AnalysisResult<LocalDependencyGraph> {
        log::info!("Building dependency graph for: {}", path.display());
        
        // Check cache first
        let cache_key = format!("dep_graph_{}", path.display());
        if let Some(cached_graph) = self.cache_manager.get_dependency_graph(&cache_key).await? {
            log::debug!("Using cached dependency graph");
            return Ok(cached_graph);
        }
        
        // Build new graph
        let graph = self.dependency_builder.build_graph(path).await?;
        
        // Cache result
        self.cache_manager.cache_dependency_graph(cache_key, &graph).await?;
        
        log::info!("Dependency graph built: {} nodes", graph.node_count());
        Ok(graph)
    }
    
    /// Analyze circular dependencies in the graph
    pub async fn analyze_cycles(&self, graph: &LocalDependencyGraph) -> Vec<CycleDependency> {
        // Extract from existing cycle detection logic in engine.rs
        self.dependency_builder.detect_cycles(graph).await
    }
    
    /// Extract dependencies from parsed files
    pub async fn extract_dependencies(&self, files: &[ParsedFile]) -> Vec<Dependency> {
        // Extract from existing dependency extraction logic
        self.dependency_builder.extract_from_files(files).await
    }
}
```

**Tasks for DependencyAnalysisService**:
- [ ] Extract dependency graph building logic from `engine.rs`
- [ ] Implement cycle detection capabilities
- [ ] Add dependency caching mechanisms
- [ ] Create comprehensive unit tests
- [ ] Document dependency analysis APIs

#### **1.3 Create PerformanceAnalysisService (~400 lines)**
**Location**: `src/analysis/services/performance_service.rs`  
**Responsibility**: Memory and performance monitoring

```rust
// src/analysis/services/performance_service.rs
use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;
use crate::analysis::memory_report::MemoryAnalysisReport;
use std::sync::Arc;
use std::future::Future;

/// Service responsible for performance monitoring and memory management
pub struct PerformanceAnalysisService {
    metrics_collector: Arc<PerformanceMetricsCollector>,
    memory_monitor: MemoryMonitor,
}

/// Monitoring session for tracking performance metrics
pub struct MonitoringSession {
    start_time: std::time::Instant,
    session_id: String,
    metrics_collector: Arc<PerformanceMetricsCollector>,
}

impl PerformanceAnalysisService {
    /// Create new performance analysis service
    pub fn new(metrics_collector: Arc<PerformanceMetricsCollector>) -> Self {
        Self {
            metrics_collector,
            memory_monitor: MemoryMonitor::new(),
        }
    }
    
    /// Start monitoring session
    pub fn start_monitoring(&self) -> MonitoringSession {
        let session_id = uuid::Uuid::new_v4().to_string();
        log::debug!("Starting monitoring session: {}", session_id);
        
        MonitoringSession {
            start_time: std::time::Instant::now(),
            session_id,
            metrics_collector: self.metrics_collector.clone(),
        }
    }
    
    /// Analyze with memory limits and monitoring
    pub async fn analyze_with_memory_limits<T, F>(&self, analysis: F) -> AnalysisResult<T>
    where
        F: Future<Output = AnalysisResult<T>>,
    {
        let _session = self.start_monitoring();
        
        // Monitor memory usage during analysis
        self.memory_monitor.start_monitoring().await;
        
        let result = analysis.await;
        
        // Check if memory limits were exceeded
        if self.memory_monitor.memory_limit_exceeded() {
            log::warn!("Memory limit exceeded during analysis");
            return Err(AnalysisError::MemoryLimitExceeded);
        }
        
        result
    }
    
    /// Get comprehensive performance report
    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            memory_usage: self.memory_monitor.get_peak_usage(),
            execution_metrics: self.metrics_collector.get_metrics(),
            recommendations: self.generate_recommendations(),
        }
    }
}
```

**Tasks for PerformanceAnalysisService**:
- [ ] Extract memory monitoring logic from `engine.rs`
- [ ] Implement performance session management
- [ ] Add memory limit enforcement
- [ ] Create performance reporting capabilities
- [ ] Create comprehensive unit tests

### **Phase 2: Orchestrator Implementation (Week 3)**

#### **2.1 Create AnalysisOrchestrator (~200 lines)**
**Location**: `src/analysis/orchestrator.rs`  
**Purpose**: Lightweight coordinator implementing Facade pattern

```rust
// src/analysis/orchestrator.rs
use crate::analysis::services::{AnalysisService, DependencyAnalysisService, PerformanceAnalysisService};
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::database::models::ArchitecturalIssue;
use std::path::Path;
use std::sync::Arc;

/// Main orchestrator coordinating analysis services (Facade Pattern)
pub struct AnalysisOrchestrator {
    analysis_service: Arc<AnalysisService>,
    dependency_service: Arc<DependencyAnalysisService>,
    performance_service: Arc<PerformanceAnalysisService>,
    
    // Optional AI services (feature-gated)
    #[cfg(feature = "ai")]
    ai_service: Option<Arc<AiAnalysisService>>,
}

/// Enhanced analysis result with performance metrics
pub struct EnhancedAnalysisResult {
    pub issues: Vec<ArchitecturalIssue>,
    pub dependency_graph: LocalDependencyGraph,
    pub performance_report: PerformanceReport,
}

impl AnalysisOrchestrator {
    /// Create new orchestrator with service dependencies
    pub fn new(
        analysis_service: Arc<AnalysisService>,
        dependency_service: Arc<DependencyAnalysisService>,
        performance_service: Arc<PerformanceAnalysisService>,
    ) -> Self {
        Self {
            analysis_service,
            dependency_service,
            performance_service,
            #[cfg(feature = "ai")]
            ai_service: None,
        }
    }
    
    /// Main analysis method - maintains backward compatibility
    pub async fn analyze(&self, path: &Path) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        log::info!("Starting orchestrated analysis for: {}", path.display());
        
        // Coordinate services concurrently
        let (issues, graph) = tokio::try_join!(
            self.analysis_service.run_analysis(path),
            self.dependency_service.build_dependency_graph(path)
        )?;
        
        log::info!("Orchestrated analysis completed successfully");
        Ok((issues, graph))
    }
    
    /// Enhanced analysis with performance monitoring
    pub async fn analyze_with_performance_monitoring(&self, path: &Path) -> AnalysisResult<EnhancedAnalysisResult> {
        let _session = self.performance_service.start_monitoring();
        
        let (issues, dependency_graph) = self.analyze(path).await?;
        let performance_report = self.performance_service.get_performance_report();
        
        Ok(EnhancedAnalysisResult {
            issues,
            dependency_graph,
            performance_report,
        })
    }
    
    /// Memory-aware analysis with adaptive behavior
    pub async fn analyze_with_memory_limits(&self, path: &Path) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        self.performance_service.analyze_with_memory_limits(
            self.analyze(path)
        ).await
    }
}
```

#### **2.2 Update AnalysisEngine for Backward Compatibility**
**Location**: `src/analysis/engine.rs` (reduce from 2,419 to ~150 lines)

```rust
// src/analysis/engine.rs (simplified - maintain backward compatibility)
use crate::analysis::orchestrator::AnalysisOrchestrator;
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::database::models::ArchitecturalIssue;
use std::path::Path;

/// Legacy AnalysisEngine - now delegates to AnalysisOrchestrator
/// 
/// This maintains backward compatibility while using the new component architecture
pub struct AnalysisEngine {
    orchestrator: AnalysisOrchestrator,
}

impl AnalysisEngine {
    /// Create new analysis engine (backward compatible)
    pub fn new() -> AnalysisResult<Self> {
        let orchestrator = AnalysisOrchestrator::builder()
            .with_default_configuration()
            .build()?;
            
        Ok(Self { orchestrator })
    }
    
    /// Main analysis method - delegates to orchestrator
    pub async fn analyze(&mut self, path: &Path) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        self.orchestrator.analyze(path).await
    }
    
    /// Enhanced analysis - delegates to orchestrator
    pub async fn analyze_with_performance_monitoring(&mut self, path: &Path) -> AnalysisResult<EnhancedAnalysisResult> {
        self.orchestrator.analyze_with_performance_monitoring(path).await
    }
    
    // ... other legacy methods that delegate to orchestrator
}
```

---

## **IMPLEMENTATION STEPS**

### **Week 2 Implementation Plan**

#### **Days 1-2: AnalysisService Extraction**
1. **Create Service Structure**:
   ```bash
   mkdir -p src/analysis/services
   touch src/analysis/services/{mod.rs,analysis_service.rs}
   ```

2. **Extract Core Logic**:
   - Move analysis coordination logic from `engine.rs`
   - Extract detector scheduling functionality
   - Move result aggregation logic

3. **Create Unit Tests**:
   ```rust
   #[cfg(test)]
   mod tests {
       #[tokio::test]
       async fn test_analysis_service_file_analysis() {
           // Test file analysis functionality
       }
       
       #[tokio::test]
       async fn test_analysis_service_directory_analysis() {
           // Test directory analysis functionality
       }
   }
   ```

#### **Days 3-4: DependencyAnalysisService Extraction**
1. **Extract Dependency Logic**:
   - Move dependency graph building from `engine.rs`
   - Extract cycle detection functionality
   - Move dependency caching logic

2. **Implement Service Interface**:
   - Create clean API for dependency operations
   - Add proper error handling
   - Implement caching strategies

#### **Days 5: PerformanceAnalysisService Extraction**
1. **Extract Performance Logic**:
   - Move memory monitoring from `engine.rs`
   - Extract performance metrics collection
   - Move memory limit enforcement

2. **Create Monitoring Framework**:
   - Implement monitoring sessions
   - Add performance reporting
   - Create recommendation engine

### **Week 3 Implementation Plan**

#### **Days 1-2: AnalysisOrchestrator Implementation**
1. **Create Orchestrator**:
   - Implement Facade pattern coordination
   - Add service dependency injection
   - Maintain backward compatibility

2. **Service Integration**:
   - Coordinate service interactions
   - Handle concurrent service execution
   - Implement error aggregation

#### **Days 3-4: AnalysisEngine Refactoring**
1. **Simplify Engine**:
   - Reduce `engine.rs` from 2,419 to ~150 lines
   - Implement delegation to orchestrator
   - Maintain all existing APIs

2. **Migration Testing**:
   - Comprehensive integration testing
   - Backward compatibility validation
   - Performance regression testing

#### **Day 5: Enhanced Builder Pattern**
1. **Create Builder**:
   ```rust
   pub struct AnalysisOrchestratorBuilder {
       analysis_service: Option<Arc<AnalysisService>>,
       dependency_service: Option<Arc<DependencyAnalysisService>>,
       performance_service: Option<Arc<PerformanceAnalysisService>>,
   }
   ```

2. **Dependency Injection Enhancement**:
   - Support custom service implementations
   - Enable testing with mock services
   - Add configuration-based construction

---

## **ACCEPTANCE CRITERIA**

### **Architecture Requirements**:
- [ ] **AnalysisEngine reduced** from 2,419 lines to <500 lines total
- [ ] **Clear separation of concerns** - each service has single responsibility
- [ ] **Backward compatibility maintained** - all existing APIs work unchanged
- [ ] **Enhanced testability** - each service can be tested in isolation

### **Service Requirements**:
- [ ] **AnalysisService** (<600 lines) - Core analysis coordination
- [ ] **DependencyAnalysisService** (<500 lines) - Dependency graph analysis
- [ ] **PerformanceAnalysisService** (<400 lines) - Memory/performance monitoring
- [ ] **AnalysisOrchestrator** (<200 lines) - Facade pattern coordinator

### **Quality Requirements**:
- [ ] **>90% test coverage** for each service
- [ ] **No performance regression** (<5% tolerance)
- [ ] **Dependency injection support** for testing and customization
- [ ] **Comprehensive documentation** for new architecture

---

## **TESTING REQUIREMENTS**

### **Unit Tests for Each Service**:

```rust
// tests/unit/analysis_service_tests.rs
#[cfg(test)]
mod analysis_service_tests {
    #[tokio::test]
    async fn test_file_analysis() {
        let service = create_test_analysis_service().await;
        let result = service.run_file_analysis(Path::new("test.rs")).await;
        assert!(result.is_ok());
    }
}

// tests/unit/dependency_service_tests.rs
#[cfg(test)]
mod dependency_service_tests {
    #[tokio::test]
    async fn test_dependency_graph_building() {
        let service = create_test_dependency_service().await;
        let graph = service.build_dependency_graph(Path::new("src/")).await;
        assert!(graph.is_ok());
    }
}

// tests/integration/orchestrator_tests.rs
#[cfg(test)]
mod orchestrator_tests {
    #[tokio::test]
    async fn test_orchestrator_coordination() {
        let orchestrator = create_test_orchestrator().await;
        let result = orchestrator.analyze(Path::new("test_project/")).await;
        assert!(result.is_ok());
    }
}
```

### **Performance Regression Tests**:
```rust
// tests/performance/architecture_performance.rs
#[tokio::test]
async fn test_no_performance_regression() {
    let old_engine = OldAnalysisEngine::new().unwrap();
    let new_orchestrator = AnalysisOrchestrator::new(...);
    
    let start = std::time::Instant::now();
    let _old_result = old_engine.analyze(test_path).await;
    let old_duration = start.elapsed();
    
    let start = std::time::Instant::now();
    let _new_result = new_orchestrator.analyze(test_path).await;
    let new_duration = start.elapsed();
    
    // New implementation should be within 5% of old performance
    assert!(new_duration <= old_duration * 105 / 100);
}
```

---

## **MIGRATION STRATEGY**

### **Phase 1: Parallel Implementation**
- Build new services alongside existing `engine.rs`
- No changes to existing code initially
- Comprehensive testing of new components

### **Phase 2: Gradual Integration**
- Update `AnalysisEngine` to delegate to orchestrator
- Maintain all existing APIs for backward compatibility
- Run both old and new implementations in tests

### **Phase 3: Complete Migration**
- Remove old implementation code from `engine.rs`
- Keep only delegation logic
- Update documentation and examples

---

## **ROLLBACK PLAN**

If issues arise:
1. **Immediate**: Revert `engine.rs` changes, keep new services as unused code
2. **Partial**: Use feature flags to switch between old/new implementations
3. **Emergency**: Complete rollback to pre-refactoring state

---

## **POST-IMPLEMENTATION VERIFICATION**

```bash
# Verify architecture improvements
cargo build --release
cargo test --all-features
cargo bench --bench architecture_benchmarks

# Check line count reduction
wc -l src/analysis/engine.rs
find src/analysis/services -name "*.rs" -exec wc -l {} +

# Verify backward compatibility
cargo test --test integration_tests -- --nocapture
```

---

## **SUCCESS METRICS**

Upon completion:
- ✅ **God Object eliminated** - Largest file <500 lines
- ✅ **Clear architecture** - Single responsibility services
- ✅ **Enhanced testability** - >90% coverage per service
- ✅ **Backward compatibility** - All existing APIs work
- ✅ **No performance regression** - <5% performance impact
- ✅ **Improved maintainability** - Parallel development enabled

**This refactoring unblocks parallel development and significantly improves code maintainability.**
