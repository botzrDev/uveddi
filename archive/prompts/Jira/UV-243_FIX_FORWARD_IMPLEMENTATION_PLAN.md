# UV-243 Fix Forward Implementation Plan
## Complete Migration to Current Dependency Versions

### Executive Summary
This document outlines a comprehensive implementation plan for migrating the Uveddi codebase to current dependency versions, addressing 50+ compilation errors caused by breaking changes in `tree-sitter` v0.25.8 and `wasmtime-wasi` v34.0.2.

**Strategic Decision**: Option B - Fix Forward Migration
**Estimated Effort**: 40-60 engineering hours over 2-3 weeks
**Risk Level**: Medium (systematic approach with incremental validation)
**Business Impact**: Resolves technical debt, improves security, enables modern features

---

## Phase 1: Core Infrastructure Migration (16-20 hours)

### 1.1 Tree-Sitter API Migration (8-10 hours)
**Scope**: Update all tree-sitter usage patterns across the codebase

**Key Changes Required**:
- Language initialization: `tree_sitter_rust::language()` → `tree_sitter_rust::LANGUAGE`
- Query iteration: `QueryMatches` → `StreamingIterator` pattern
- Parser API: Update method signatures and error handling
- AST traversal: Modernize cursor usage patterns

**Affected Files**:
- `src/ast/tree_sitter_impl.rs` - Core parser implementation
- `src/analysis/detectors/anti_patterns/` - All detector modules
- `src/analysis/cfg/mod.rs` - Control flow graph generation

**Implementation Steps**:
1. Update language constants in all detector modules (2h)
2. Migrate query execution patterns to streaming iterators (4h)
3. Fix parser initialization and configuration (2h)
4. Update error handling for new API patterns (2h)

### 1.2 WASI API Migration (6-8 hours)
**Scope**: Migrate from legacy WASI imports to modern p2 module structure

**Key Changes Required**:
- Import paths: `wasmtime_wasi::{WasiCtx, WasiView}` → `wasmtime_wasi::p2::{WasiCtx, WasiView}`
- Trait implementations: Split `WasiView` and `ResourceTableView`
- Linker configuration: Update `add_to_linker_sync` calls
- Directory preopen API: Modern permission patterns

**Affected Files**:
- `src/plugins/types.rs` - Core WASI types and traits
- `src/plugins/security.rs` - Security policy and WASI configuration
- `src/plugins/lifecycle.rs` - Plugin lifecycle management

**Implementation Steps**:
1. Update import statements across plugin modules (1h)
2. Fix trait implementations and method signatures (3h)
3. Migrate directory preopen and permission APIs (2h)
4. Update linker configuration and error handling (2h)

### 1.3 Compilation Validation (2 hours)
- Incremental compilation testing after each migration
- Feature flag validation (`--no-default-features` testing)
- Dependency resolution verification

---

## Phase 2: Feature Integration & Testing (16-20 hours)

### 2.1 AST Parser Integration (6-8 hours)
**Scope**: Ensure seamless integration between updated tree-sitter and existing AST infrastructure

**Tasks**:
- Validate `ParsedFile` type compatibility across modules
- Update AST serialization/deserialization patterns
- Fix memory management for new parser instances
- Performance validation with new streaming patterns

### 2.2 Plugin System Integration (6-8 hours)
**Scope**: Restore full WASM plugin functionality with updated WASI APIs

**Tasks**:
- Validate security policy enforcement with new APIs
- Test resource limiting and sandboxing functionality  
- Verify plugin lifecycle management (load/unload/reload)
- Integration testing with example plugins

### 2.3 Detector Module Validation (4 hours)
**Scope**: Ensure all anti-pattern detectors function correctly with new tree-sitter patterns

**Priority Detectors**:
- God Object detection (`src/analysis/detectors/anti_patterns/god_object.rs`)
- Tight Coupling detection (`src/analysis/detectors/anti_patterns/tight_coupling.rs`)
- Long Methods detection (`src/analysis/detectors/anti_patterns/long_methods.rs`)
- Code Duplication detection (`src/analysis/detectors/anti_patterns/code_duplication.rs`)

---

## Phase 3: Performance & Quality Assurance (8-12 hours)

### 3.1 Performance Validation (4-6 hours)
**Scope**: Ensure migration maintains or improves performance characteristics

**Benchmarks**:
- AST parsing performance (pre/post migration comparison)
- Memory usage patterns with new streaming iterators
- Plugin execution overhead measurements
- End-to-end analysis pipeline benchmarks

### 3.2 Test Suite Restoration (4-6 hours)
**Scope**: Restore full test coverage and address test failures

**Priority Test Suites**:
- `tests/analysis/engine.rs` - Core analysis engine tests
- `tests/comprehensive_coverage.rs` - Coverage validation
- Plugin system integration tests
- Detector-specific unit tests

---

## Risk Mitigation Strategy

### Technical Risks
1. **API Compatibility Issues**: Incremental migration with feature flags
2. **Performance Regressions**: Continuous benchmarking during migration
3. **Plugin Compatibility**: Comprehensive integration testing
4. **Test Suite Failures**: Parallel test restoration with code migration

### Mitigation Approach
- **Incremental Development**: Migrate one subsystem at a time
- **Feature Flag Protection**: Use conditional compilation during transition
- **Continuous Validation**: Run subset of tests after each major change
- **Rollback Capability**: Maintain clean git history for easy reversion

---

## Success Criteria

### Primary Objectives
- [ ] Clean `cargo check` compilation (zero errors/warnings)
- [ ] Full test suite passes (`cargo test --all-features`)
- [ ] Performance benchmarks within 5% of baseline
- [ ] All detector modules functional
- [ ] Plugin system operational with security policies

### Quality Gates
- [ ] Code coverage maintained at 90%+ threshold
- [ ] Clippy linting passes with zero warnings
- [ ] Memory safety validation (no unsafe code added)
- [ ] Documentation updated for API changes
- [ ] Integration tests pass for all supported languages

---

## Implementation Timeline

### Week 1: Core Infrastructure
- Days 1-2: Tree-sitter API migration
- Days 3-4: WASI API migration  
- Day 5: Compilation validation and integration testing

### Week 2: Feature Integration
- Days 1-2: AST parser integration and validation
- Days 3-4: Plugin system restoration
- Day 5: Detector module validation

### Week 3: Quality Assurance
- Days 1-2: Performance validation and optimization
- Days 3-4: Test suite restoration
- Day 5: Final validation and documentation

---

## Resource Requirements

### Engineering Resources
- **Primary Developer**: 40-60 hours over 2-3 weeks
- **Code Review**: 8-10 hours (senior engineer review)
- **QA Testing**: 4-6 hours (comprehensive validation)

### Infrastructure Requirements
- Development environment with latest Rust toolchain
- CI/CD pipeline access for automated testing
- Performance benchmarking environment
- Test coverage reporting tools

---

## Next Steps

1. **Stakeholder Approval**: Review and approve this implementation plan
2. **Resource Allocation**: Assign primary developer and reviewer
3. **Timeline Confirmation**: Confirm project timeline and milestones
4. **Kickoff Meeting**: Technical deep-dive session with implementation team

---

**Plan Status**: Ready for Implementation  
**Created**: January 2025  
**Owner**: Development Team  
**Stakeholders**: Engineering Leadership, Product Team