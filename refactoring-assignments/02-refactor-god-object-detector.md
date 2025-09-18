# Assignment 02: Refactor God Object Detector

## Priority: HIGH
## Estimated Time: 3-4 hours
## File: `/src/analysis/detectors/anti_patterns/god_object.rs` (1,904 lines)

## Objective
Ironically, the God Object detector has become a God Object itself. Decompose into focused components.

## Current Problems
- Contains detection logic for multiple languages in one file
- Mixes configuration, detection, and reporting
- 1,904 lines makes it difficult to maintain
- Self-referential anti-pattern

## Tasks

### 1. Create Module Structure
```
src/analysis/detectors/anti_patterns/god_object/
├── mod.rs (<150 lines)
├── detector.rs (main detection trait)
├── config.rs (configuration structures)
├── metrics.rs (metric calculations)
├── languages/
│   ├── mod.rs
│   ├── rust.rs
│   ├── python.rs
│   ├── javascript.rs
│   └── typescript.rs
└── reports/
    ├── mod.rs
    └── formatter.rs
```

### 2. Extract Language-Specific Detection
Create separate detector for each language:
- `languages/rust.rs`: Rust-specific God Object detection (<300 lines)
- `languages/python.rs`: Python class detection (<250 lines)
- `languages/javascript.rs`: JavaScript object detection (<250 lines)
- `languages/typescript.rs`: TypeScript class detection (<250 lines)

### 3. Extract Configuration
- Move all configuration structs to `config.rs`
- Create `GodObjectConfig` struct with builder pattern
- Add configuration validation
- Target: <150 lines

### 4. Extract Metrics Calculation
- Move metric calculations to `metrics.rs`
- Create `ComplexityMetrics` struct
- Implement methods: `calculate_loc()`, `calculate_methods()`, `calculate_dependencies()`
- Target: <200 lines

### 5. Create Base Detector Trait
```rust
// In detector.rs
pub trait GodObjectDetector {
    fn detect(&self, ast: &SyntaxNode) -> Vec<GodObjectViolation>;
    fn calculate_metrics(&self, node: &SyntaxNode) -> ComplexityMetrics;
    fn applies_to_language(&self, language: &Language) -> bool;
}
```

### 6. Implement Language Registry
- Create factory pattern for language-specific detectors
- Register detectors at compile time
- Allow runtime selection based on file type

### 7. Extract Report Formatting
- Move all reporting logic to `reports/formatter.rs`
- Create `GodObjectReport` struct
- Implement multiple output formats
- Target: <200 lines

### 8. Update Integration
- Update main detector registry
- Fix imports throughout codebase
- Ensure backward compatibility

## Success Criteria
- [ ] Original file eliminated (replaced with module)
- [ ] All new files <300 lines
- [ ] Separation of concerns achieved
- [ ] All tests pass
- [ ] Performance maintained or improved
- [ ] Documentation for each module

## Verification Commands
```bash
# Check module structure
tree src/analysis/detectors/anti_patterns/god_object/

# Run specific tests
cargo test god_object::

# Benchmark performance
cargo bench god_object_detection
```

## Completion Notes
_To be filled by AI developer:_
- Files created: ___
- Lines per file: ___
- Performance comparison: ___
- Test coverage: ___