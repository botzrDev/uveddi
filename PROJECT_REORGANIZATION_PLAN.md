# Project Reorganization Plan

## Current Issues Identified

### 1. Structural Issues
- **Scattered root-level files**: `constants.rs`, `security_stub.rs` should be in proper modules
- **Inconsistent module organization**: Some modules have deep nesting, others are flat
- **Mixed concerns**: `src/analysis` contains both core logic and implementation details
- **Duplicate functionality**: Multiple detector systems and configuration approaches
- **Legacy files**: `mod_verbose.rs`, deprecated modules still present

### 2. Module Organization Issues
- **CLI module**: Contains too many command files in flat structure
- **Analysis module**: Mixes core engine with detectors, performance, and utilities
- **Report module**: Complex nested structure with unclear separation of concerns
- **Application module**: Contains both orchestration and implementation details

### 3. Import and Dependency Issues
- **Circular dependencies**: Some modules have interdependencies
- **Deep import paths**: Some imports are unnecessarily complex
- **Feature gate inconsistencies**: Not all optional modules properly gated

## Reorganization Strategy

### Phase 1: Safe Reorganization (Low Risk)
**Goal**: Clean up obvious organizational issues without breaking APIs

#### 1.1 Root-Level Cleanup
- Move `constants.rs` → `src/core/constants.rs`
- Move `security_stub.rs` → `src/security/stub.rs` (if still needed)
- Ensure all root-level files are properly organized

#### 1.2 Module Structure Standardization
- Standardize module layouts with consistent `mod.rs` patterns
- Remove unused/deprecated files
- Consolidate duplicate functionality

#### 1.3 CLI Module Reorganization
```
src/cli/
├── mod.rs                    # Main CLI exports
├── commands/                 # All command implementations
│   ├── mod.rs
│   ├── analyze.rs           # analyze_command.rs
│   ├── config.rs            # config_command.rs
│   ├── doctor.rs            # doctor_command.rs
│   ├── help.rs              # help_command.rs
│   ├── hooks.rs             # hooks_command.rs
│   ├── init.rs              # init_command.rs
│   ├── ci.rs                # ci_command.rs
│   └── plugin.rs            # plugin_command.rs
├── utils/                    # CLI utilities
│   ├── mod.rs
│   ├── enhanced_help.rs
│   └── validation.rs
└── ui/                       # UI-related commands
    ├── mod.rs
    ├── tui.rs               # tui_command.rs
    └── web.rs               # ui_command.rs
```

### Phase 2: Analysis Module Restructuring (Medium Risk)
**Goal**: Separate concerns in the analysis module

#### 2.1 Core Analysis Separation
```
src/analysis/
├── mod.rs                    # Main analysis exports
├── core/                     # Core analysis engine
│   ├── mod.rs
│   ├── engine.rs            # Main analysis engine
│   ├── orchestrator.rs      # Analysis orchestration
│   └── config.rs            # Analysis configuration
├── detectors/                # All detection logic
│   ├── mod.rs
│   ├── base/                # Base detector traits
│   ├── anti_patterns/       # Anti-pattern detectors
│   └── security/            # Security detectors
├── parsing/                  # AST and parsing logic
│   ├── mod.rs
│   ├── ast.rs               # AST definitions
│   └── language_support.rs  # Language-specific parsing
├── cache/                    # Caching system
│   ├── mod.rs
│   └── ast_cache.rs
└── utils/                    # Analysis utilities
    ├── mod.rs
    ├── file_discovery.rs
    └── parallel.rs
```

#### 2.2 Detector System Cleanup
- Consolidate detector base classes
- Standardize detector interfaces
- Remove duplicate detector implementations
- Organize by category (anti-patterns, security, performance)

### Phase 3: Advanced Reorganization (Higher Risk)
**Goal**: Deep architectural improvements

#### 3.1 Application Layer Simplification
```
src/application/
├── mod.rs                    # Main application exports
├── orchestrator.rs          # Main application orchestrator
├── config/                  # Configuration management
│   ├── mod.rs
│   ├── analysis.rs
│   ├── ai.rs
│   └── output.rs
├── services/                # Application services
│   ├── mod.rs
│   ├── analysis.rs
│   ├── progress.rs
│   └── registry.rs
└── workflows/               # Analysis workflows
    ├── mod.rs
    ├── analysis.rs
    ├── ai.rs
    └── reporting.rs
```

#### 3.2 Report Module Restructuring
```
src/report/
├── mod.rs                    # Main report exports
├── generators/              # Report generators
│   ├── mod.rs
│   ├── markdown.rs
│   ├── json.rs
│   └── html.rs
├── formats/                 # Format-specific logic
│   ├── mod.rs
│   └── mermaid.rs
├── templates/               # Report templates
│   ├── mod.rs
│   └── html/
└── utils/                   # Report utilities
    ├── mod.rs
    ├── metrics.rs
    └── diagrams.rs
```

## Risk Assessment

### Low Risk Changes
- Moving files within same module
- Updating import statements
- Removing unused files
- Standardizing module structures

### Medium Risk Changes
- Restructuring large modules (analysis, application)
- Consolidating detector systems
- Updating feature gates
- Changing public API organization

### High Risk Changes
- Breaking existing import paths
- Changing core module interfaces
- Major architectural refactoring
- Database schema changes

## Implementation Strategy

### 1. Incremental Approach
- Make changes in small, testable increments
- Run tests after each change
- Maintain backward compatibility where possible
- Use deprecation warnings for breaking changes

### 2. Testing Strategy
- Run full test suite after each phase
- Add integration tests for reorganized modules
- Test all CLI commands still work
- Verify build processes still function

### 3. Documentation Updates
- Update internal documentation
- Update import examples in docs
- Update architecture diagrams
- Update migration guides

## Pre-Implementation Checklist

### Backup Strategy
- [ ] Create git branch for reorganization
- [ ] Tag current stable state
- [ ] Document current working state
- [ ] Prepare rollback plan

### Testing Preparation
- [ ] Ensure all tests pass currently
- [ ] Document current test coverage
- [ ] Identify critical integration points
- [ ] Prepare test data for validation

### Dependencies Check
- [ ] Map all external dependencies
- [ ] Identify internal module dependencies
- [ ] Document public API contracts
- [ ] Check for circular dependencies

## Implementation Order

### Phase 1: Immediate (Day 1)
1. Root-level file cleanup
2. CLI module reorganization
3. Module declaration standardization
4. Import path updates

### Phase 2: Short-term (Day 2-3)
1. Analysis module restructuring
2. Detector system consolidation
3. Cache system organization
4. Feature gate cleanup

### Phase 3: Medium-term (Day 4-5)
1. Application layer simplification
2. Report module restructuring
3. Plugin system organization
4. Documentation updates

### Phase 4: Validation (Day 6)
1. Full test suite execution
2. Integration testing
3. Performance validation
4. Documentation verification

## Success Criteria

### Functional Requirements
- [ ] All tests pass
- [ ] All CLI commands work
- [ ] Build processes succeed
- [ ] No regressions in functionality

### Quality Requirements
- [ ] Improved code organization
- [ ] Reduced module complexity
- [ ] Clearer separation of concerns
- [ ] Better maintainability

### Documentation Requirements
- [ ] Updated architecture documentation
- [ ] Clear module boundaries
- [ ] Updated import examples
- [ ] Migration guide for users

## Rollback Plan

If critical issues arise:
1. Revert to tagged stable state
2. Identify specific breaking changes
3. Create targeted fixes
4. Re-apply changes incrementally
5. Test thoroughly at each step

This reorganization plan prioritizes stability while improving the project structure for long-term maintainability.