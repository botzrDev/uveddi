# UV-156 Quick TODO Summary for GPT Dev

## 🎯 **IMMEDIATE ACTION ITEMS**

### **PHASE 1: Core Dependency Injection (START HERE)**
```bash
# Priority Order - Complete in sequence:
1. Create src/analysis/engine_builder.rs
2. Create src/analysis/detector_factory.rs  
3. Add with_detectors() method to src/analysis/engine.rs
4. Update existing new() method for backward compatibility
5. Update src/analysis/mod.rs exports
```

### **KEY FILES TO CREATE/MODIFY**

#### **NEW FILES TO CREATE:**
- `src/analysis/engine_builder.rs` - Builder pattern implementation
- `src/analysis/detector_factory.rs` - Factory for creating detectors
- `src/analysis/detector_registry.rs` - Registry pattern (Phase 2)
- `src/analysis/config.rs` - Configuration support (Phase 2)
- `tests/analysis/dependency_injection.rs` - Unit tests (Phase 4)

#### **EXISTING FILES TO MODIFY:**
- `src/analysis/engine.rs` - Add dependency injection constructor
- `src/analysis/mod.rs` - Export new modules
- `Cargo.toml` - Add serde, toml dependencies

### **CRITICAL IMPLEMENTATION POINTS**

#### **1. Dependency Injection Constructor (PRIORITY 1)**
```rust
// ADD TO src/analysis/engine.rs
impl AnalysisEngine {
    pub fn with_detectors(
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
        cache_path: Option<&Path>,
        enable_plugins: bool,
    ) -> crate::error::Result<Self> {
        // Implementation replaces hardcoded detector creation
    }
}
```

#### **2. Builder Pattern (PRIORITY 2)**
```rust
// NEW FILE: src/analysis/engine_builder.rs
pub struct AnalysisEngineBuilder {
    detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    cache_path: Option<PathBuf>,
    enable_plugins: bool,
}
```

#### **3. Backward Compatibility (PRIORITY 3)**
```rust
// MODIFY existing new() method in src/analysis/engine.rs
pub fn new() -> crate::error::Result<Self> {
    let default_detectors = DetectorFactory::create_default_detectors();
    Self::with_detectors(default_detectors, Some(&PathBuf::from("uveddi_cache.db")), false)
}
```

### **VALIDATION CHECKLIST**
- [ ] `cargo check --all-targets` passes
- [ ] `cargo test` passes (all existing tests)
- [ ] Backward compatibility maintained
- [ ] New dependency injection works
- [ ] Builder pattern functional

### **CURRENT PROBLEM LOCATION**
**File**: `src/analysis/engine.rs` **Lines**: 112-118
```rust
// THIS IS THE PROBLEMATIC CODE TO REPLACE:
detectors: vec![
    Box::new(GodObjectDetector::new(5, 8)), // HARDCODED
    Box::new(CodeDuplicationDetector::new()), // HARDCODED
    // ... more hardcoded detectors
],
```

### **SUCCESS CRITERIA**
1. ✅ Engine accepts injected detectors
2. ✅ Mock detectors work in tests  
3. ✅ Existing code unchanged (backward compatible)
4. ✅ Performance impact <5%
5. ✅ Plugin integration ready

## 📋 **PHASE BREAKDOWN**

| Phase | Hours | Key Deliverable |
|-------|-------|----------------|
| 1 | 8h | Dependency injection + builder pattern |
| 2 | 6h | Registry + configuration support |
| 3 | 4h | Plugin integration |
| 4 | 6h | Testing + documentation |

**Start with Phase 1 - Focus on dependency injection core!**

## 🚀 **QUICK START COMMAND**
```bash
# 1. Create the builder file first
touch src/analysis/engine_builder.rs

# 2. Create the factory file  
touch src/analysis/detector_factory.rs

# 3. Start implementing dependency injection in engine.rs
# Focus on the with_detectors() method first
```

**Full implementation guide**: `docs/11-prompts/UV-156_Dependency_Injection_Implementation.md`