# GPT Dev Prompt 003: Circular Dependency Resolution

## 🔄 HIGH PRIORITY - Week 4-5 Implementation

### **Issue**: UV-105 - Critical Circular Dependency Resolution
**Reference Document**: `CIRCULAR_DEPENDENCIES_PLAN.md`  
**Jira Issue**: UV-105  
**Priority**: HIGH - Blocks Compilation & Testing  
**Estimated Time**: 2 weeks  
**Dependencies**: Complete UV-104 (Architecture Refactoring) first

---

## **TASK OVERVIEW**

The codebase has **2,045 circular dependencies** causing compilation issues and preventing clean architecture. You need to implement a systematic resolution strategy that addresses both immediate compilation blockers and underlying architectural issues.

**Current State**: 2,045 circular dependencies  
**Target State**: <50 circular dependencies (clean architecture threshold)  
**Critical Path**: Enable stable compilation and testing

---

## **CIRCULAR DEPENDENCY ANALYSIS**

### **Current Impact Assessment**
- **Total Cycles**: 2,045 circular dependencies detected
- **Compilation Failures**: Multiple crates affected
- **Testing Blocked**: Integration tests failing due to cycles
- **Architecture Debt**: Prevents clean modular design

### **Dependency Hotspots** (from diagnostic analysis)
```
High-Impact Cycles:
├── analysis/ ↔ database/ (457 cycles)
├── ast/ ↔ analysis/ (389 cycles)  
├── security/ ↔ analysis/ (324 cycles)
├── monitoring/ ↔ analysis/ (278 cycles)
└── ui/ ↔ analysis/ (267 cycles)
```

**Root Cause**: `analysis` module is central dependency hub causing star-pattern cycles

---

## **IMPLEMENTATION PHASES**

### **Phase 1: Critical Path Resolution (Week 4)**

#### **1.1 Analysis ↔ Database Cycle Resolution (457 cycles)**
**Priority**: CRITICAL - Blocks data persistence  
**Strategy**: Interface Segregation + Dependency Inversion

**Current Problem**:
```rust
// analysis/engine.rs
use crate::database::models::ArchitecturalIssue;  // analysis → database
use crate::database::Database;

// database/models.rs  
use crate::analysis::detectors::DetectorType;     // database → analysis
```

**Solution Implementation**:

1. **Create Abstract Interfaces**:
```rust
// src/core/interfaces/persistence.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Abstract persistence interface - breaks database dependency
#[async_trait]
pub trait PersistenceProvider: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    type Issue: IssueEntity;
    
    async fn save_issues(&self, issues: Vec<Self::Issue>) -> Result<(), Self::Error>;
    async fn load_issues(&self, filter: IssueFilter) -> Result<Vec<Self::Issue>, Self::Error>;
    async fn get_issue_stats(&self) -> Result<IssueStats, Self::Error>;
}

/// Domain issue entity - no database dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainIssue {
    pub id: Option<String>,
    pub detector_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub file_path: String,
    pub line_number: u32,
    pub metadata: serde_json::Value,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

2. **Implement Database Provider**:
```rust
// src/infrastructure/database/persistence_provider.rs
use crate::core::interfaces::persistence::{PersistenceProvider, DomainIssue};
use crate::database::Database;
use async_trait::async_trait;

/// Database implementation of persistence interface
pub struct DatabasePersistenceProvider {
    database: Database,
}

#[async_trait]
impl PersistenceProvider for DatabasePersistenceProvider {
    type Error = DatabaseError;
    type Issue = DomainIssue;
    
    async fn save_issues(&self, issues: Vec<DomainIssue>) -> Result<(), DatabaseError> {
        // Convert domain issues to database models
        let db_issues: Vec<ArchitecturalIssue> = issues
            .into_iter()
            .map(|issue| self.convert_to_db_model(issue))
            .collect();
            
        self.database.save_issues(db_issues).await
    }
    
    async fn load_issues(&self, filter: IssueFilter) -> Result<Vec<DomainIssue>, DatabaseError> {
        let db_issues = self.database.load_issues(filter).await?;
        let domain_issues = db_issues
            .into_iter()
            .map(|db_issue| self.convert_to_domain(db_issue))
            .collect();
        Ok(domain_issues)
    }
    
    // Private conversion methods
    fn convert_to_db_model(&self, domain_issue: DomainIssue) -> ArchitecturalIssue {
        // Map domain → database
    }
    
    fn convert_to_domain(&self, db_issue: ArchitecturalIssue) -> DomainIssue {
        // Map database → domain
    }
}
```

3. **Update Analysis Service**:
```rust
// src/analysis/services/analysis_service.rs (updated)
use crate::core::interfaces::persistence::{PersistenceProvider, DomainIssue};
use std::sync::Arc;

pub struct AnalysisService<P: PersistenceProvider> {
    persistence: Arc<P>,
    // ... other dependencies
}

impl<P: PersistenceProvider> AnalysisService<P> {
    pub fn new(persistence: Arc<P>) -> Self {
        Self { persistence }
    }
    
    pub async fn run_analysis(&self, path: &Path) -> AnalysisResult<Vec<DomainIssue>> {
        // Analysis logic - now uses domain types only
        let issues = self.detect_issues(path).await?;
        
        // Persist through interface - no direct database dependency
        self.persistence.save_issues(issues.clone()).await
            .map_err(|e| AnalysisError::PersistenceError(e.to_string()))?;
            
        Ok(issues)
    }
}
```

**Tasks for Analysis ↔ Database Resolution**:
- [ ] Create `core/interfaces/persistence.rs` with abstract interfaces
- [ ] Implement `DatabasePersistenceProvider` with proper conversion
- [ ] Update `AnalysisService` to use dependency injection
- [ ] Create comprehensive integration tests
- [ ] Verify 457 cycles reduced to <10

#### **1.2 AST ↔ Analysis Cycle Resolution (389 cycles)**
**Priority**: CRITICAL - Blocks code parsing  
**Strategy**: Observer Pattern + Event-Driven Architecture

**Current Problem**:
```rust
// ast/parser.rs
use crate::analysis::detectors::DetectorRegistry;  // ast → analysis

// analysis/detectors/base.rs
use crate::ast::ParsedFile;                       // analysis → ast
```

**Solution Implementation**:

1. **Create AST Event System**:
```rust
// src/core/events/ast_events.rs
use serde::{Deserialize, Serialize};

/// AST parsing events - breaks direct analysis dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstEvent {
    FileParsed {
        file_path: String,
        ast_data: AstData,
        parse_duration: std::time::Duration,
    },
    ParseError {
        file_path: String,
        error: String,
    },
    ParsingStarted {
        total_files: usize,
    },
    ParsingCompleted {
        successful: usize,
        failed: usize,
    },
}

/// Generic AST data representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstData {
    pub syntax_tree: serde_json::Value,
    pub metadata: AstMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstMetadata {
    pub language: String,
    pub file_size: u64,
    pub node_count: usize,
    pub depth: usize,
}
```

2. **Implement Event Bus**:
```rust
// src/core/events/event_bus.rs
use tokio::sync::broadcast;
use std::sync::Arc;

/// Central event bus for decoupling components
pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
}

#[derive(Debug, Clone)]
pub enum DomainEvent {
    Ast(AstEvent),
    Analysis(AnalysisEvent),
    // ... other event types
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }
    
    /// Publish event to all subscribers
    pub fn publish(&self, event: DomainEvent) -> Result<usize, EventError> {
        self.sender.send(event)
            .map_err(|_| EventError::NoSubscribers)
    }
    
    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }
}
```

3. **Update AST Parser**:
```rust
// src/ast/parser.rs (updated)
use crate::core::events::{EventBus, AstEvent, DomainEvent};
use std::sync::Arc;

pub struct AstParser {
    event_bus: Arc<EventBus>,
    // Remove detector registry dependency
}

impl AstParser {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }
    
    pub async fn parse_file(&self, file_path: &Path) -> Result<ParsedFile, ParseError> {
        let start = std::time::Instant::now();
        
        match self.internal_parse(file_path).await {
            Ok(parsed_file) => {
                // Publish event instead of direct analysis notification
                let event = AstEvent::FileParsed {
                    file_path: file_path.to_string_lossy().to_string(),
                    ast_data: self.convert_to_ast_data(&parsed_file),
                    parse_duration: start.elapsed(),
                };
                
                let _ = self.event_bus.publish(DomainEvent::Ast(event));
                Ok(parsed_file)
            }
            Err(error) => {
                let event = AstEvent::ParseError {
                    file_path: file_path.to_string_lossy().to_string(),
                    error: error.to_string(),
                };
                
                let _ = self.event_bus.publish(DomainEvent::Ast(event));
                Err(error)
            }
        }
    }
}
```

**Tasks for AST ↔ Analysis Resolution**:
- [ ] Create event-driven communication system
- [ ] Implement `EventBus` with proper error handling
- [ ] Update `AstParser` to publish events instead of direct calls
- [ ] Update analysis components to subscribe to AST events
- [ ] Verify 389 cycles reduced to <5

### **Phase 2: Security & Monitoring Cycles (Week 5)**

#### **2.1 Security ↔ Analysis Cycle Resolution (324 cycles)**
**Strategy**: Security Audit Event Pattern

**Solution**:
```rust
// src/core/interfaces/security.rs
pub trait SecurityAuditor: Send + Sync {
    async fn audit_analysis_results(&self, results: &[DomainIssue]) -> SecurityAuditResult;
    async fn validate_file_access(&self, path: &Path) -> AccessValidationResult;
}

// src/security/analysis_auditor.rs
pub struct AnalysisSecurityAuditor {
    // Security-specific logic without analysis dependencies
}

impl SecurityAuditor for AnalysisSecurityAuditor {
    async fn audit_analysis_results(&self, results: &[DomainIssue]) -> SecurityAuditResult {
        // Audit logic using domain types
    }
}
```

#### **2.2 Monitoring ↔ Analysis Cycle Resolution (278 cycles)**
**Strategy**: Metrics Collection Interface

**Solution**:
```rust
// src/core/interfaces/monitoring.rs
pub trait MetricsCollector: Send + Sync {
    fn record_analysis_metrics(&self, metrics: AnalysisMetrics);
    fn record_performance_metrics(&self, metrics: PerformanceMetrics);
}

// src/monitoring/analysis_metrics_collector.rs
pub struct AnalysisMetricsCollector {
    // Monitoring logic without analysis dependencies
}
```

---

## **DEPENDENCY RESOLUTION TOOLS**

### **1. Cycle Detection Script**
Create automated cycle detection:

```rust
// tools/dependency_analyzer.rs
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::kosaraju_scc;
use std::collections::HashMap;

/// Tool for detecting and analyzing circular dependencies
pub struct DependencyAnalyzer {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl DependencyAnalyzer {
    pub fn analyze_project(&mut self, project_path: &Path) -> Vec<CircularDependency> {
        self.build_dependency_graph(project_path);
        self.detect_cycles()
    }
    
    fn detect_cycles(&self) -> Vec<CircularDependency> {
        let sccs = kosaraju_scc(&self.graph);
        
        sccs.into_iter()
            .filter(|scc| scc.len() > 1)  // Only actual cycles
            .map(|scc| {
                let modules: Vec<String> = scc
                    .into_iter()
                    .map(|node_idx| self.graph[node_idx].clone())
                    .collect();
                    
                CircularDependency {
                    modules,
                    severity: self.calculate_severity(&modules),
                    suggested_fix: self.suggest_fix(&modules),
                }
            })
            .collect()
    }
}

/// Run cycle detection
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = DependencyAnalyzer::new();
    let cycles = analyzer.analyze_project(Path::new("src/"));
    
    println!("Found {} circular dependencies:", cycles.len());
    for cycle in cycles {
        println!("  Cycle: {} ({})", cycle.modules.join(" → "), cycle.severity);
        println!("    Fix: {}", cycle.suggested_fix);
    }
    
    Ok(())
}
```

### **2. Automated Refactoring Script**
```bash
#!/bin/bash
# scripts/resolve_cycles.sh

echo "🔄 Starting circular dependency resolution..."

# Phase 1: Interface extraction
echo "Phase 1: Extracting interfaces..."
cargo run --bin extract_interfaces

# Phase 2: Dependency injection
echo "Phase 2: Implementing dependency injection..."
cargo run --bin implement_di

# Phase 3: Event system setup
echo "Phase 3: Setting up event system..."
cargo run --bin setup_events

# Verify reduction
echo "Verifying cycle reduction..."
cargo run --bin dependency_analyzer > cycle_report.txt
echo "Results saved to cycle_report.txt"

echo "✅ Circular dependency resolution completed!"
```

---

## **ACCEPTANCE CRITERIA**

### **Quantitative Goals**:
- [ ] **Reduce total cycles** from 2,045 to <50 (97.5% reduction)
- [ ] **Eliminate critical cycles**: Analysis ↔ Database, AST ↔ Analysis  
- [ ] **Enable clean compilation** with no circular dependency errors
- [ ] **Maintain functionality** - all existing features work unchanged

### **Architectural Goals**:
- [ ] **Interface segregation** - Abstract interfaces break concrete dependencies
- [ ] **Dependency inversion** - High-level modules don't depend on low-level modules
- [ ] **Event-driven communication** - Loose coupling through events
- [ ] **Single responsibility** - Each module has clear, focused purpose

### **Quality Assurance**:
- [ ] **Automated cycle detection** - Tools prevent regression
- [ ] **Comprehensive testing** - All refactored code has >90% coverage
- [ ] **Performance maintenance** - No significant performance degradation
- [ ] **Documentation updates** - Architecture changes fully documented

---

## **TESTING STRATEGY**

### **1. Pre-Resolution Testing**
```bash
# Document current state
cargo run --bin dependency_analyzer > cycles_before.txt
cargo build 2> build_errors_before.txt
cargo test 2> test_errors_before.txt
```

### **2. Progressive Testing During Resolution**
```rust
// tests/integration/cycle_resolution_tests.rs
#[cfg(test)]
mod cycle_resolution_tests {
    #[test]
    fn test_analysis_database_cycle_resolved() {
        let analyzer = DependencyAnalyzer::new();
        let cycles = analyzer.find_cycles_between("analysis", "database");
        assert!(cycles.len() < 10, "Analysis-Database cycles should be <10, found: {}", cycles.len());
    }
    
    #[test]
    fn test_ast_analysis_cycle_resolved() {
        let analyzer = DependencyAnalyzer::new();
        let cycles = analyzer.find_cycles_between("ast", "analysis");
        assert!(cycles.len() < 5, "AST-Analysis cycles should be <5, found: {}", cycles.len());
    }
    
    #[tokio::test]
    async fn test_event_driven_communication() {
        let event_bus = EventBus::new();
        let mut receiver = event_bus.subscribe();
        
        // Test AST event publishing
        let ast_event = AstEvent::FileParsed { /* ... */ };
        event_bus.publish(DomainEvent::Ast(ast_event)).unwrap();
        
        // Verify event received
        let received = receiver.recv().await.unwrap();
        assert!(matches!(received, DomainEvent::Ast(_)));
    }
}
```

### **3. Post-Resolution Validation**
```bash
# Verify improvements
cargo run --bin dependency_analyzer > cycles_after.txt
diff cycles_before.txt cycles_after.txt

# Ensure compilation works
cargo build --all-features
cargo test --all-features

# Performance regression testing  
cargo bench --bench dependency_benchmarks
```

---

## **ROLLBACK STRATEGY**

### **Phase 1 Rollback**: Interface Issues
- Keep original concrete dependencies alongside interfaces
- Use feature flags to switch between implementations
- Maintain backward compatibility during transition

### **Phase 2 Rollback**: Event System Issues  
- Keep direct method calls alongside event system
- Gradual migration with fallback mechanisms
- Monitor for performance or reliability issues

### **Emergency Rollback**: Complete Reversion
```bash
git checkout main -- src/analysis/engine.rs
git checkout main -- src/database/models.rs
git checkout main -- src/ast/parser.rs
cargo build --release  # Verify working state
```

---

## **SUCCESS METRICS**

### **Immediate Success Indicators**:
```bash
# Successful compilation without circular dependency errors
cargo build --all-features 2>&1 | grep -i "circular" | wc -l  # Should be 0

# Cycle count reduction
cargo run --bin dependency_analyzer | grep "Found" | awk '{print $2}'  # Should be <50

# Test suite stability  
cargo test --all-features | grep "test result" | grep -o "[0-9]* passed"  # Should maintain count
```

### **Long-term Architecture Health**:
- ✅ **Clean module boundaries** - Clear interfaces between components
- ✅ **Loose coupling** - Components communicate through abstractions
- ✅ **High cohesion** - Related functionality grouped together
- ✅ **Testability improvement** - Mock implementations possible
- ✅ **Parallel development enabled** - Teams can work independently

### **Maintainability Improvements**:
- ✅ **Easier refactoring** - Changes isolated to specific modules
- ✅ **Better code navigation** - Clear dependency directions
- ✅ **Reduced cognitive load** - Simpler mental models
- ✅ **Enhanced debugging** - Clearer execution paths

**Upon completion, the codebase will have clean, maintainable architecture enabling rapid feature development.**
