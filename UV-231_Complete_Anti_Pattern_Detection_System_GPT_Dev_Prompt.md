# UV-231: Complete Anti-Pattern Detection System Implementation - Senior Developer Prompt

## 🎯 **Mission Overview**

You are tasked with completing the implementation of the anti-pattern detection system for Uveddi, a Rust-based static code analysis tool. This involves integrating the plugin system, completing detector implementations, fixing hardcoded values, and implementing proper line number extraction from AST analysis.

## 📋 **Current Status Analysis**

### ✅ **What's Already Implemented**
1. **Plugin System Architecture**: Complete WASM plugin system with comprehensive modules
   - `src/plugins/` - Full plugin system with engine, registry, security, lifecycle management
   - `src/analysis/plugin_adapter.rs` - Complete adapter for WASM plugins as AnalysisDetectors
   - Plugin loading/unloading infrastructure exists

2. **Anti-Pattern Detectors**: Substantial implementations exist
   - **tight_coupling.rs**: ~1300 lines, comprehensive multi-language analysis with proper line number extraction
   - **leaky_abstraction.rs**: ~900 lines, advanced detection with architectural analysis
   - **god_object.rs**: ~1300 lines, complete implementation with proper AST line extraction

3. **Line Number Extraction**: Mostly implemented
   - Most detectors use `capture.node.start_position().row as u32 + 1` for accurate line numbers
   - Only a few hardcoded `Some(1)` instances remain in test files and mocks

### ❌ **What Still Needs Implementation**

#### 1. **Plugin Integration in Engine** (Critical)
- **File**: `src/analysis/engine.rs` line 868
- **Issue**: `// TODO: Implement proper plugin adapter integration`
- **Impact**: Plugin system cannot be used as detectors in analysis pipeline

#### 2. **Plugin Manager TODOs** (Critical)
- **File**: `src/analysis/components/plugin_manager.rs`
- **Line 146**: `// TODO: Implement actual plugin execution logic`
- **Line 169**: `// TODO: Implement actual plugin loading logic`
- **Impact**: Plugin manager is a stub implementation

#### 3. **Hardcoded Anti-Pattern Type IDs** (Medium)
- **tight_coupling.rs**: `anti_pattern_type_id: 1` (lines 343, 500)
- **leaky_abstraction.rs**: `anti_pattern_type_id: 1` with `// TODO: proper mapping` (line 657)
- **Impact**: Incorrect categorization and database inconsistency

#### 4. **Hardcoded Analysis Run IDs** (Medium)
- **leaky_abstraction.rs**: `analysis_run_id = 1; // TODO: Get from context` (line 878)
- **Impact**: Cannot track analysis sessions properly

## 🔧 **Implementation Tasks**

### **Task 1: Complete Plugin Integration in Analysis Engine**
**Priority**: Critical
**File**: `src/analysis/engine.rs`
**Location**: Line 868

```rust
// Current TODO:
// TODO: Implement proper plugin adapter integration

// Required Implementation:
// 1. Enable plugin adapters to be added as detectors
// 2. Integrate with existing detector pipeline
// 3. Handle plugin loading/unloading during analysis
// 4. Ensure proper error handling and resource cleanup
```

**Implementation Strategy**:
- Remove the TODO comment and implement plugin adapter integration
- Use the existing `WasmPluginDetectorAdapter` from `plugin_adapter.rs`
- Integrate with the component-based architecture using `PluginManagerHandle`
- Ensure plugins can be dynamically loaded and used as `AnalysisDetector` instances

### **Task 2: Complete Plugin Manager Implementation**
**Priority**: Critical
**File**: `src/analysis/components/plugin_manager.rs`
**Locations**: Lines 146, 169

```rust
// Current TODOs:
// Line 146: TODO: Implement actual plugin execution logic
// Line 169: TODO: Implement actual plugin loading logic

// Required Implementation:
// 1. Connect to WasmPluginEngine for actual plugin operations
// 2. Implement proper async plugin execution
// 3. Handle plugin lifecycle management
// 4. Implement resource monitoring and cleanup
```

**Implementation Strategy**:
- Replace TODO stubs with actual `WasmPluginEngine` integration
- Implement proper async plugin execution using the existing engine
- Add error handling and resource management
- Ensure thread-safe plugin operations

### **Task 3: Fix Hardcoded Anti-Pattern Type IDs**
**Priority**: Medium
**Files**: Multiple detector files

**Implementation Strategy**:
- Create a centralized `AntiPatternTypeRegistry` or use existing `AntiPatternType` enum
- Map detector types to proper IDs:
  - Tight Coupling: Use appropriate ID (not hardcoded 1)
  - Leaky Abstraction: Implement proper type mapping
  - God Object: Verify current implementation
- Update all detectors to use the centralized mapping

### **Task 4: Fix Hardcoded Analysis Run IDs**
**Priority**: Medium
**Files**: Detector implementations

**Implementation Strategy**:
- Pass analysis context through detector interface
- Modify `AnalysisDetector` trait to accept context if needed
- Ensure analysis run IDs are properly tracked and passed through the system

### **Task 5: Complete Line Number Extraction**
**Priority**: Low (mostly done)
**Status**: Only test files and mocks have hardcoded `Some(1)`

**Implementation Strategy**:
- Review remaining `Some(1)` instances in test files
- Ensure all production code uses proper AST line extraction
- Update any remaining hardcoded values in non-test code

## 🏗️ **Architecture Integration Points**

### **Component Dependencies**
```rust
// Key components to integrate:
- AnalysisEngine (main orchestrator)
- PluginManagerHandle (plugin operations)
- WasmPluginEngine (plugin runtime)
- WasmPluginDetectorAdapter (bridge to AnalysisDetector)
- DetectorFactory (detector registration)
```

### **Integration Flow**
1. **Engine Initialization**: Load and register plugin adapters as detectors
2. **Analysis Execution**: Run plugins alongside native detectors
3. **Result Aggregation**: Collect results from both plugin and native detectors
4. **Resource Management**: Proper cleanup and resource monitoring

## 🧪 **Testing Requirements**

### **Integration Tests**
- Plugin loading and unloading workflow
- Plugin execution as part of analysis pipeline
- Error handling for plugin failures
- Resource cleanup and memory management

### **Unit Tests**
- Plugin manager operations
- Anti-pattern type ID mapping
- Analysis run ID propagation
- Line number extraction accuracy

## 📁 **Key Files to Modify**

### **Primary Implementation Files**
1. `src/analysis/engine.rs` - Remove TODO, implement plugin integration
2. `src/analysis/components/plugin_manager.rs` - Complete plugin operations
3. `src/analysis/detectors/anti_patterns/tight_coupling.rs` - Fix hardcoded IDs
4. `src/analysis/detectors/anti_patterns/leaky_abstraction.rs` - Fix hardcoded IDs and analysis run ID

### **Supporting Files**
- `src/analysis/detector_factory.rs` - Ensure plugin detectors can be registered
- `src/database/models.rs` - Verify anti-pattern type definitions
- Test files - Update integration tests for plugin system

## 🎯 **Acceptance Criteria**

### **Functional Requirements**
- [ ] Plugin adapters can be loaded and used as detectors without errors
- [ ] Plugin loading/unloading works end-to-end in analysis engine
- [ ] All anti-pattern detectors provide meaningful analysis results
- [ ] Issues report accurate line numbers from AST analysis (no hardcoded values)
- [ ] No hardcoded anti-pattern type IDs or analysis run IDs in production code
- [ ] All TODO comments related to plugin integration are resolved

### **Quality Requirements**
- [ ] Comprehensive test coverage for plugin integration
- [ ] Proper error handling and resource cleanup
- [ ] Thread-safe plugin operations
- [ ] Performance monitoring and resource limits

### **Integration Requirements**
- [ ] Seamless integration with existing analysis pipeline
- [ ] Backward compatibility with existing detector system
- [ ] Proper component lifecycle management

## 🚀 **Implementation Priority**

1. **Phase 1** (Critical): Plugin integration in engine and plugin manager
2. **Phase 2** (Medium): Fix hardcoded IDs and analysis run context
3. **Phase 3** (Low): Clean up remaining hardcoded line numbers in tests

## 📊 **Success Metrics**

- Plugin system fully functional and integrated
- Zero TODO comments related to plugin integration
- All detectors use proper ID mapping and context
- Comprehensive test coverage for plugin workflow
- Clean, maintainable code following Rust best practices

## 🔍 **Code Quality Standards**

- Follow existing Rust patterns and error handling
- Maintain async/await consistency
- Use proper Arc/Mutex patterns for shared state
- Implement comprehensive logging and monitoring
- Ensure memory safety and resource cleanup

---

**This implementation will complete the core anti-pattern detection system and enable full plugin functionality for Uveddi's analysis engine.**