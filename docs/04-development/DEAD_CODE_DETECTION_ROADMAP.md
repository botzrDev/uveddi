# Dead Code Detection Enhancement Roadmap

**Date**: July 7, 2025  
**Status**: Current Implementation Working, Major Enhancements Needed  
**Priority**: High - Foundation for Advanced Static Analysis

## Executive Summary

The dead code detection system in Uveddi has been stabilized with basic functionality working correctly across Rust, Python, and JavaScript. However, extensive research and industry analysis reveals significant opportunities to evolve from our current single-file approach to a sophisticated whole-program analyzer that would position Uveddi as a leading static analysis platform.

## Current State ✅

### What's Working
- **Basic Detection**: Single-file dead code detection for unused functions, variables, and classes
- **Multi-language Support**: Rust, Python, JavaScript with language-specific handling
- **Export Detection**: Proper distinction between exported (`pub`) and private symbols
- **Confidence Scoring**: Configurable thresholds to manage false positives
- **Library Mode**: Correct handling of exported symbols in library vs application contexts
- **Test Coverage**: Comprehensive test suite with all tests passing

### Technical Foundation
- Tree-sitter AST parsing for all supported languages
- Symbol extraction and reference tracking within files
- Configurable detection parameters (`library_mode`, `min_confidence`)
- Integration with the main analysis engine

## Critical Limitations of Current Implementation ⚠️

### 1. Single-File Analysis Scope
**Problem**: The detector analyzes files in isolation, missing cross-file dependencies.
```rust
// file1.rs
pub fn exported_function() { ... }  // Flagged as unused

// file2.rs  
use crate::file1::exported_function;  // Usage not detected
```

**Impact**: High false positive rate for modular codebases, limiting practical utility.

### 2. No Call Graph Construction
**Problem**: Missing whole-program reachability analysis from entry points.
**Impact**: Cannot accurately determine if exported symbols are actually used across the project.

### 3. Limited Entry Point Detection
**Problem**: Only basic detection of `main` functions, missing framework-specific entry points.
**Impact**: Incorrect classification of framework-required exports as dead code.

## Development Roadmap

## Phase 1: Foundation Enhancement (4-6 weeks) 🏗️

### Priority 1.1: Project-Wide Symbol Collection
**Goal**: Build a unified symbol table across all project files.

**Tasks**:
- [ ] Implement `ProjectSymbolTable` struct to aggregate symbols from all files
- [ ] Create cross-file reference resolution system
- [ ] Add module/import tracking for dependency relationships
- [ ] Update `DeadCodeDetector` to operate on project-wide data

**Acceptance Criteria**:
- Detector can find usage of exported symbols across files
- Test showing `pub fn` in one file used in another is not flagged
- Performance acceptable for projects with 1000+ files

### Priority 1.2: Enhanced Entry Point Detection
**Goal**: Automatically identify all legitimate entry points.

**Tasks**:
- [ ] Rust: Detect `main()`, `#[no_mangle]`, `extern "C"`, test functions
- [ ] Python: Detect `if __name__ == "__main__"`, Flask/Django routes, pytest fixtures
- [ ] JavaScript: Detect CommonJS exports, ES6 exports, React components, test files
- [ ] Add configuration for custom entry point patterns

**Acceptance Criteria**:
- No false positives for framework-required exports
- Configurable patterns for project-specific entry points
- Documentation for common framework patterns

### Priority 1.3: Call Graph Construction (Basic)
**Goal**: Build a basic call graph for reachability analysis.

**Tasks**:
- [ ] Implement `CallGraph` data structure
- [ ] Add function call detection using Tree-sitter queries
- [ ] Implement mark-and-sweep reachability from entry points
- [ ] Handle static method calls and imports

**Acceptance Criteria**:
- Functions reachable from entry points are never flagged as dead
- Transitive dependencies are correctly tracked
- Performance scales linearly with codebase size

## Phase 2: Advanced Analysis (6-8 weeks) 🧠

### Priority 2.1: Code Property Graph (CPG) Integration
**Goal**: Implement the research-recommended CPG architecture.

**Tasks**:
- [ ] Design CPG schema (nodes: File, Module, Function, Variable; edges: CONTAINS, CALLS, IMPORTS)
- [ ] Implement graph database integration (recommend Neo4j or FalkorDB)
- [ ] Migrate symbol table to graph-based storage
- [ ] Add graph query capabilities for complex analysis

**Technical Specification**:
```rust
struct CodePropertyGraph {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
    db_connection: GraphDatabase,
}

enum Node {
    File { path: PathBuf, language: SourceLanguage },
    Function { name: String, visibility: Visibility, location: SourceLocation },
    Variable { name: String, scope: ScopeId },
    // ... other node types
}

enum Edge {
    Contains { parent: NodeId, child: NodeId },
    Calls { caller: NodeId, callee: NodeId },
    References { from: NodeId, to: NodeId },
    // ... other edge types
}
```

### Priority 2.2: Dynamic Analysis Integration
**Goal**: Reduce false positives through runtime data.

**Tasks**:
- [ ] Design code coverage integration points
- [ ] Implement test execution tracking
- [ ] Add production usage monitoring hooks
- [ ] Combine static and dynamic analysis results

**Benefits**: Industry research shows 60-80% reduction in false positives.

### Priority 2.3: Framework-Aware Analysis
**Goal**: Handle framework-specific "magic" that creates implicit usage.

**Tasks**:
- [ ] Plugin system for framework-specific rules
- [ ] Rust: Derive macros, proc macros, reflection usage
- [ ] Python: Django models, Flask decorators, pytest fixtures
- [ ] JavaScript: React hooks, Next.js file routing, Webpack dynamic imports

## Phase 3: Production Optimization (4-6 weeks) ⚡

### Priority 3.1: Incremental Analysis
**Goal**: Only reanalyze changed files and their dependencies.

**Tasks**:
- [ ] Implement file change detection
- [ ] Build dependency invalidation system
- [ ] Cache intermediate analysis results
- [ ] Optimize for CI/CD pipeline integration

### Priority 3.2: Performance & Scalability
**Goal**: Handle enterprise-scale codebases efficiently.

**Tasks**:
- [ ] Parallel analysis of independent modules
- [ ] Memory usage optimization for large projects
- [ ] Benchmark against 100k+ line codebases
- [ ] Add progress reporting for long-running analysis

### Priority 3.3: Advanced Reporting
**Goal**: Provide actionable insights for development teams.

**Tasks**:
- [ ] Dead code impact analysis (lines saved, dependencies removed)
- [ ] Integration with IDEs for real-time feedback
- [ ] Automated PR generation for safe dead code removal
- [ ] Historical tracking of code health metrics

## Phase 4: Ecosystem Integration (2-4 weeks) 🔌

### Priority 4.1: Build System Integration
**Goal**: Leverage build system knowledge for better analysis.

**Tasks**:
- [ ] Cargo.toml dependency analysis for Rust
- [ ] package.json entry point detection for JavaScript
- [ ] setup.py/pyproject.toml analysis for Python
- [ ] Integration with language servers (rust-analyzer, Pylsp, TypeScript)

### Priority 4.2: CI/CD Pipeline Enhancement
**Goal**: Seamless integration with development workflows.

**Tasks**:
- [ ] GitHub Actions workflow templates
- [ ] GitLab CI integration
- [ ] Configurable failure thresholds
- [ ] Comment-based PR feedback

## Technical Architecture Recommendations

### 1. Data Storage Strategy
```
Current: In-memory HashMap per file
Recommended: Persistent graph database with incremental updates

Benefits:
- Cross-session analysis state preservation
- Complex graph queries (Cypher)
- Scalability to large codebases
- Foundation for multiple analysis types
```

### 2. Analysis Pipeline
```
Current: File → Parse → Analyze → Report
Recommended: 
1. Project Scan → Global Symbol Table
2. Entry Point Detection → Call Graph Construction  
3. Reachability Analysis → Dead Code Identification
4. Confidence Scoring → Filtered Reporting
```

### 3. Configuration Management
```yaml
# .uveddi/dead_code.yaml
analysis:
  mode: "whole_program"  # vs "single_file"
  confidence_threshold: 0.7
  
entry_points:
  rust: ["main", "lib.rs::*pub*"]
  python: ["__main__", "**/views.py::*"]
  javascript: ["index.js", "pages/**/*.js"]
  
frameworks:
  enabled: ["django", "react", "axum"]
  
ignore_patterns:
  - "**/*_test.rs"
  - "generated/**"
  - "migrations/**"
```

## Risk Assessment & Mitigation

### High Risk Items
1. **Performance Impact**: CPG construction could be slow for large codebases
   - *Mitigation*: Incremental analysis, parallel processing
2. **False Positive Management**: Dynamic language features may confuse static analysis
   - *Mitigation*: Confidence scoring, whitelist patterns, dynamic analysis integration
3. **Framework Compatibility**: Rapid framework evolution may break analysis
   - *Mitigation*: Plugin architecture, community contributions

### Medium Risk Items
1. **Database Dependency**: Adding external database dependency
   - *Mitigation*: Optional feature, fallback to in-memory analysis
2. **Complexity Increase**: System becoming harder to maintain
   - *Mitigation*: Comprehensive test suite, modular architecture

## Success Metrics

### Phase 1 Success Criteria
- [ ] Zero false positives on Uveddi's own codebase
- [ ] 90%+ accuracy on open source Rust projects
- [ ] Analysis time under 30 seconds for 10k-line projects

### Long-term Success Criteria
- [ ] Industry-leading accuracy (95%+ precision, 90%+ recall)
- [ ] Sub-minute analysis for 100k-line enterprise codebases
- [ ] Integration with 5+ major development tools/IDEs
- [ ] Community adoption by 100+ open source projects

## Resource Requirements

### Development Team
- **Senior Rust Developer**: Lead implementation (Phase 1-2)
- **Graph Database Engineer**: CPG implementation (Phase 2)
- **DevOps Engineer**: CI/CD integration (Phase 4)
- **QA Engineer**: Test framework expansion (All phases)

### Infrastructure
- **Development**: Neo4j/FalkorDB instance for testing
- **CI/CD**: Extended pipeline for multi-language testing
- **Documentation**: Expanded user guides and API docs

## Getting Started - Next Sprint

### Immediate Actions (This Sprint)
1. [ ] Create `ProjectSymbolTable` struct and basic cross-file tests
2. [ ] Research graph database options and create POC
3. [ ] Expand test coverage to include cross-file scenarios
4. [ ] Update documentation with current limitations

### Developer Assignment Recommendations
- **Phase 1.1**: Assign to senior Rust developer familiar with Tree-sitter
- **Phase 1.2**: Can be parallelized across junior developers by language
- **Phase 1.3**: Requires algorithm expertise, assign to senior developer

## Conclusion

The current dead code detection implementation provides a solid foundation, but realizing Uveddi's potential as a leading static analysis platform requires evolution to whole-program analysis. The research-backed roadmap above provides a clear path from the current single-file approach to an industry-leading, graph-based analyzer.

The investment in Phase 1 will immediately improve accuracy and reduce false positives. Phase 2 establishes Uveddi as technically competitive with commercial static analysis tools. Phases 3-4 ensure the system can scale to enterprise usage and integrate seamlessly with developer workflows.

**Recommendation**: Begin Phase 1 immediately while researching graph database options. The foundation work will pay dividends across all future static analysis features, not just dead code detection.
