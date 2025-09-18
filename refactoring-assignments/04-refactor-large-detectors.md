# Assignment 04: Refactor Large Detector Files

## Priority: HIGH
## Estimated Time: 6-8 hours
## Files: Multiple detector files >800 lines

## Objective
Refactor remaining large detector files to follow single responsibility principle.

## Target Files (based on analysis)
- `/src/analysis/detectors/complexity/` (various files >800 lines)
- `/src/analysis/detectors/security/` (large security detector files)
- `/src/analysis/detectors/patterns/` (pattern detection files)

## Tasks

### 1. Audit Large Detector Files
First, identify all detector files >500 lines:
```bash
find src/analysis/detectors -name "*.rs" -exec wc -l {} + | sort -nr | head -20
```

### 2. For Each Large Detector File:

#### Standard Refactoring Pattern:
```
src/analysis/detectors/[category]/[detector_name]/
├── mod.rs (public API, <100 lines)
├── detector.rs (main trait implementation)
├── config.rs (configuration)
├── rules/
│   ├── mod.rs
│   ├── rule1.rs (specific detection rules)
│   └── rule2.rs
└── language_support/
    ├── mod.rs
    ├── rust.rs (if language-specific)
    └── python.rs
```

### 3. Create Base Detector Framework
Create shared framework for all detectors:

```rust
// In src/analysis/detectors/base/mod.rs
pub trait Detector {
    type Config: DetectorConfig;
    type Output: DetectorOutput;

    fn name(&self) -> &'static str;
    fn detect(&self, context: &AnalysisContext) -> Result<Self::Output>;
    fn supported_languages(&self) -> &[Language];
}

pub trait DetectorConfig {
    fn validate(&self) -> Result<()>;
    fn merge(&mut self, other: Self);
}

pub trait DetectorOutput {
    fn severity(&self) -> Severity;
    fn message(&self) -> String;
    fn location(&self) -> Location;
}
```

### 4. Refactor Security Detectors
Focus on security detector files:
- Extract vulnerability-specific detection into separate modules
- Create common security scanning framework
- Implement rule-based detection system
- Target: <400 lines per detector

### 5. Refactor Complexity Detectors
Break down complexity analysis:
- Separate cyclomatic complexity calculation
- Extract halstead metrics to separate module
- Create maintainability index calculator
- Target: <300 lines per detector

### 6. Refactor Pattern Detectors
Decompose pattern detection:
- Separate design pattern detection
- Extract anti-pattern detection (already started in Assignment 02)
- Create code smell detection
- Target: <350 lines per detector

### 7. Create Detector Registry
Implement detector discovery and registration:
```rust
pub struct DetectorRegistry {
    detectors: HashMap<String, Box<dyn Detector>>,
}

impl DetectorRegistry {
    pub fn register<T: Detector + 'static>(&mut self, detector: T) {
        self.detectors.insert(detector.name().to_string(), Box::new(detector));
    }

    pub fn run_all(&self, context: &AnalysisContext) -> DetectionResults {
        // Run all registered detectors
    }
}
```

### 8. Update Integration Points
- Update main analysis engine to use new detector structure
- Fix all import statements
- Update configuration system
- Ensure backward compatibility where possible

## Priority Order
1. Security detectors (highest risk)
2. Complexity detectors (most used)
3. Pattern detectors (architectural impact)
4. Remaining large files

## Success Criteria
- [ ] All detector files <500 lines
- [ ] Common detector framework implemented
- [ ] Detector registry system working
- [ ] All tests pass
- [ ] Performance impact <10%
- [ ] Clear separation of concerns

## Performance Considerations
- Measure detection performance before/after
- Ensure parallel detection still works
- Optimize critical path detectors
- Profile memory usage

## Verification Commands
```bash
# Check file sizes
find src/analysis/detectors -name "*.rs" -exec wc -l {} + | sort -nr

# Run detector tests
cargo test detectors::

# Performance benchmark
cargo bench detector_performance

# Memory profiling
valgrind --tool=massif target/release/uveddi analyze test_project/
```

## Completion Notes
_To be filled by AI developer:_
- Files refactored: ___
- Average file size reduction: ___
- Performance impact: ___
- New detector framework features: ___
- Backward compatibility issues: ___