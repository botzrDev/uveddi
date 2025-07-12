# UV-156: AnalysisEngine Dependency Injection Refactor - GPT Dev Implementation Guide

## 🎯 **Task Overview**
**Jira Issue**: UV-156  
**Title**: Refactor AnalysisEngine to use dependency injection pattern  
**Status**: In Progress  
**Priority**: Medium  
**Estimated Effort**: 24 hours (4 phases)

## 📋 **Current Architecture Analysis**

### **Problematic Code Location**: `src/analysis/engine.rs` (Lines 112-118)
```rust
// CURRENT PROBLEMATIC DESIGN - TIGHTLY COUPLED
detectors: vec![
    Box::new(GodObjectDetector::new(5, 8)), // Hardcoded
    Box::new(CodeDuplicationDetector::new()), // Hardcoded
    Box::new(DeadCodeDetector::with_default_config()), // Hardcoded
    Box::new(LargeClassDetector::with_default_config()), // Hardcoded
    Box::new(TightCouplingDetector::default()), // Hardcoded
],
```

### **Key Problems Identified**:
1. **Tight Coupling**: Engine hardcodes specific detector types
2. **Poor Testability**: Cannot inject mock detectors
3. **Limited Extensibility**: Adding detectors requires engine modification
4. **Plugin Integration Issues**: WASM plugins can't be dynamically added
5. **Configuration Inflexibility**: No external configuration support

## 🚀 **Implementation Plan - 4 Phases**

---

## **PHASE 1: Core Dependency Injection (8 hours)**

### **Task 1.1: Create AnalysisEngineBuilder**
**File**: `src/analysis/engine_builder.rs` (NEW FILE)

```rust
use crate::analysis::{AnalysisDetector, AnalysisEngine};
use crate::error::UveddiError;
use std::path::PathBuf;

/// Builder pattern for configuring AnalysisEngine with dependency injection
#[derive(Default)]
pub struct AnalysisEngineBuilder {
    detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    cache_path: Option<PathBuf>,
    enable_plugins: bool,
    use_memory_cache: bool,
}

impl AnalysisEngineBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a detector to the engine
    pub fn add_detector(mut self, detector: Box<dyn AnalysisDetector + Send + Sync>) -> Self {
        self.detectors.push(detector);
        self
    }
    
    /// Set custom cache path
    pub fn with_cache_path(mut self, path: PathBuf) -> Self {
        self.cache_path = Some(path);
        self
    }
    
    /// Enable plugin support
    pub fn enable_plugins(mut self) -> Self {
        self.enable_plugins = true;
        self
    }
    
    /// Use in-memory cache (for testing)
    pub fn with_memory_cache(mut self) -> Self {
        self.use_memory_cache = true;
        self
    }
    
    /// Build the AnalysisEngine with configured options
    pub async fn build(self) -> Result<AnalysisEngine, UveddiError> {
        // Implementation here
        todo!("Implement builder pattern")
    }
}
```

### **Task 1.2: Add Dependency Injection Constructor**
**File**: `src/analysis/engine.rs`

**ADD NEW METHOD**:
```rust
impl AnalysisEngine {
    /// Create engine with injected detectors (DEPENDENCY INJECTION)
    pub fn with_detectors(
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
        cache_path: Option<&Path>,
        enable_plugins: bool,
    ) -> crate::error::Result<Self> {
        let cache = if let Some(path) = cache_path {
            ResultCache::new(path)?
        } else {
            ResultCache::new_in_memory()?
        };
        
        Ok(Self {
            ast_parser: AstParser::new()?,
            dependency_extractor: DependencyExtractor::new()?,
            symbol_extractor: SymbolExtractor::new(),
            detectors, // INJECTED DETECTORS
            cycle_detector: CycleDetector::new(),
            files_analyzed: 0,
            cache,
            symbol_table: GlobalSymbolTable::new(),
            plugin_engine: if enable_plugins {
                // Initialize plugin engine
                None // TODO: Implement in Phase 3
            } else {
                None
            },
        })
    }
}
```

### **Task 1.3: Create Default Detector Factory**
**File**: `src/analysis/detector_factory.rs` (NEW FILE)

```rust
use crate::analysis::AnalysisDetector;
use crate::analysis::detectors::anti_patterns::*;

pub struct DetectorFactory;

impl DetectorFactory {
    /// Create default detector set for backward compatibility
    pub fn create_default_detectors() -> Vec<Box<dyn AnalysisDetector + Send + Sync>> {
        vec![
            Box::new(GodObjectDetector::new(5, 8)),
            Box::new(CodeDuplicationDetector::new()),
            Box::new(DeadCodeDetector::with_default_config()),
            Box::new(LargeClassDetector::with_default_config()),
            Box::new(TightCouplingDetector::default()),
        ]
    }
    
    /// Create detector by name with configuration
    pub fn create_detector(
        name: &str, 
        config: &DetectorConfig
    ) -> Result<Box<dyn AnalysisDetector + Send + Sync>, UveddiError> {
        match name {
            "god_object" => Ok(Box::new(GodObjectDetector::new(
                config.get("threshold_methods").unwrap_or(5),
                config.get("threshold_fields").unwrap_or(8),
            ))),
            "code_duplication" => Ok(Box::new(CodeDuplicationDetector::new())),
            "dead_code" => Ok(Box::new(DeadCodeDetector::with_default_config())),
            "large_classes" => Ok(Box::new(LargeClassDetector::with_default_config())),
            "tight_coupling" => Ok(Box::new(TightCouplingDetector::default())),
            _ => Err(UveddiError::InvalidConfiguration(format!("Unknown detector: {}", name))),
        }
    }
}

// Configuration structure for detectors
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    params: std::collections::HashMap<String, i32>,
}

impl DetectorConfig {
    pub fn new() -> Self {
        Self {
            params: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_param(mut self, key: &str, value: i32) -> Self {
        self.params.insert(key.to_string(), value);
        self
    }
    
    pub fn get(&self, key: &str) -> Option<i32> {
        self.params.get(key).copied()
    }
}
```

### **Task 1.4: Update Existing Constructor for Backward Compatibility**
**File**: `src/analysis/engine.rs`

**MODIFY EXISTING `new()` METHOD**:
```rust
impl AnalysisEngine {
    /// Creates a new analysis engine with default configuration (BACKWARD COMPATIBLE)
    pub fn new() -> crate::error::Result<Self> {
        let default_detectors = DetectorFactory::create_default_detectors();
        let cache_path = PathBuf::from("uveddi_cache.db");
        Self::with_detectors(default_detectors, Some(&cache_path), false)
    }
}
```

---

## **PHASE 2: Registry & Configuration (6 hours)**

### **Task 2.1: Create Detector Registry**
**File**: `src/analysis/detector_registry.rs` (NEW FILE)

```rust
use crate::analysis::{AnalysisDetector, DetectorFactory, DetectorConfig};
use crate::error::UveddiError;
use std::collections::HashMap;

/// Registry for managing detector instances
pub struct DetectorRegistry {
    detectors: HashMap<String, Box<dyn AnalysisDetector + Send + Sync>>,
    factory: DetectorFactory,
}

impl DetectorRegistry {
    pub fn new() -> Self {
        Self {
            detectors: HashMap::new(),
            factory: DetectorFactory,
        }
    }
    
    /// Register a detector with a name
    pub fn register(&mut self, name: String, detector: Box<dyn AnalysisDetector + Send + Sync>) {
        self.detectors.insert(name, detector);
    }
    
    /// Get all registered detectors
    pub fn get_all_detectors(self) -> Vec<Box<dyn AnalysisDetector + Send + Sync>> {
        self.detectors.into_values().collect()
    }
    
    /// Load detectors from configuration
    pub fn load_from_config(&mut self, config: &AnalysisConfig) -> Result<(), UveddiError> {
        for (name, detector_config) in &config.detectors {
            let detector = self.factory.create_detector(name, detector_config)?;
            self.register(name.clone(), detector);
        }
        Ok(())
    }
    
    /// Load default detector set
    pub fn load_defaults(&mut self) {
        let defaults = DetectorFactory::create_default_detectors();
        for (i, detector) in defaults.into_iter().enumerate() {
            self.register(format!("default_{}", i), detector);
        }
    }
}
```

### **Task 2.2: Configuration Support**
**File**: `src/analysis/config.rs` (NEW FILE)

```rust
use crate::analysis::{DetectorConfig, DetectorRegistry, AnalysisEngine};
use crate::error::UveddiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Configuration for the analysis engine
#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisConfig {
    pub detectors: HashMap<String, DetectorConfig>,
    pub cache_size: Option<usize>,
    pub enable_plugins: bool,
    pub cache_path: Option<String>,
}

impl AnalysisConfig {
    /// Load configuration from TOML file
    pub fn from_file(path: &Path) -> Result<Self, UveddiError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| UveddiError::ConfigurationError(format!("Failed to read config: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| UveddiError::ConfigurationError(format!("Failed to parse config: {}", e)))
    }
    
    /// Create engine from this configuration
    pub async fn create_engine(&self) -> Result<AnalysisEngine, UveddiError> {
        let mut registry = DetectorRegistry::new();
        registry.load_from_config(self)?;
        
        let detectors = registry.get_all_detectors();
        let cache_path = self.cache_path.as_ref().map(|p| Path::new(p));
        
        AnalysisEngine::with_detectors(detectors, cache_path, self.enable_plugins)
    }
    
    /// Default configuration
    pub fn default() -> Self {
        let mut detectors = HashMap::new();
        detectors.insert("god_object".to_string(), 
            DetectorConfig::new()
                .with_param("threshold_methods", 5)
                .with_param("threshold_fields", 8)
        );
        detectors.insert("code_duplication".to_string(), DetectorConfig::new());
        detectors.insert("dead_code".to_string(), DetectorConfig::new());
        detectors.insert("large_classes".to_string(), DetectorConfig::new());
        detectors.insert("tight_coupling".to_string(), DetectorConfig::new());
        
        Self {
            detectors,
            cache_size: Some(1000),
            enable_plugins: false,
            cache_path: Some("uveddi_cache.db".to_string()),
        }
    }
}
```

### **Task 2.3: Example Configuration File**
**File**: `examples/analysis_config.toml` (NEW FILE)

```toml
# Uveddi Analysis Engine Configuration

# Cache settings
cache_size = 1000
cache_path = "uveddi_cache.db"
enable_plugins = true

# Detector configurations
[detectors.god_object]
threshold_methods = 5
threshold_fields = 8

[detectors.code_duplication]
# Uses default settings

[detectors.dead_code]
# Uses default settings

[detectors.large_classes]
# Uses default settings

[detectors.tight_coupling]
# Uses default settings

[detectors.magic_values]
threshold = 3
exclude = [0, 1, -1]
```

---

## **PHASE 3: Plugin Integration (4 hours)**

### **Task 3.1: Plugin Detector Adapter**
**File**: `src/plugins/detector_adapter.rs` (NEW FILE)

```rust
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::plugins::{WasmPlugin, PluginId};

/// Adapter to make WASM plugins work as AnalysisDetectors
pub struct WasmPluginAdapter {
    plugin: WasmPlugin,
    plugin_id: PluginId,
}

impl WasmPluginAdapter {
    pub fn new(plugin: WasmPlugin, plugin_id: PluginId) -> Self {
        Self { plugin, plugin_id }
    }
}

impl AnalysisDetector for WasmPluginAdapter {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Convert ParsedFile to plugin format and call WASM plugin
        // This will be implemented based on the plugin interface
        todo!("Implement WASM plugin adapter")
    }
    
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        // Get anti-pattern types from plugin metadata
        todo!("Get types from plugin")
    }
    
    fn get_detector_name(&self) -> &'static str {
        "wasm-plugin-detector"
    }
}
```

### **Task 3.2: Update AnalysisEngine for Plugin Integration**
**File**: `src/analysis/engine.rs`

**ADD METHOD**:
```rust
impl AnalysisEngine {
    /// Add plugin detectors dynamically
    pub async fn add_plugin_detectors(&mut self) -> Result<(), UveddiError> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            let loaded_plugins = plugin_engine.list_loaded_plugins().await;
            
            for plugin_id in loaded_plugins {
                if let Some(plugin) = plugin_engine.get_plugin(&plugin_id).await {
                    let adapter = WasmPluginAdapter::new(plugin, plugin_id);
                    self.detectors.push(Box::new(adapter));
                }
            }
        }
        Ok(())
    }
    
    /// Remove plugin detectors
    pub fn remove_plugin_detectors(&mut self) {
        self.detectors.retain(|detector| {
            detector.get_detector_name() != "wasm-plugin-detector"
        });
    }
}
```

---

## **PHASE 4: Testing & Documentation (6 hours)**

### **Task 4.1: Comprehensive Unit Tests**
**File**: `tests/analysis/dependency_injection.rs` (NEW FILE)

```rust
use uveddi::analysis::{AnalysisEngine, AnalysisDetector, AnalysisEngineBuilder};
use uveddi::ast::ParsedFile;
use uveddi::database::models::{AntiPatternType, ArchitecturalIssue};
use mockall::mock;

mock! {
    TestDetector {}
    impl AnalysisDetector for TestDetector {
        fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, uveddi::error::UveddiError>;
        fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
        fn get_detector_name(&self) -> &'static str;
    }
}

#[tokio::test]
async fn test_dependency_injection_constructor() {
    let mut mock_detector = MockTestDetector::new();
    mock_detector
        .expect_detect_issues()
        .returning(|_| Ok(vec![]));
    mock_detector
        .expect_get_detector_name()
        .returning(|| "test-detector");
    mock_detector
        .expect_get_anti_pattern_types()
        .returning(|| vec![]);
    
    let detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>> = vec![
        Box::new(mock_detector)
    ];
    
    let engine = AnalysisEngine::with_detectors(detectors, None, false).unwrap();
    assert_eq!(engine.get_anti_pattern_types().len(), 1); // Includes cycle detector
}

#[tokio::test]
async fn test_builder_pattern() {
    let mut mock_detector = MockTestDetector::new();
    mock_detector
        .expect_get_detector_name()
        .returning(|| "test-detector");
    mock_detector
        .expect_get_anti_pattern_types()
        .returning(|| vec![]);
    
    let engine = AnalysisEngineBuilder::new()
        .add_detector(Box::new(mock_detector))
        .with_memory_cache()
        .build()
        .await
        .unwrap();
    
    assert!(engine.get_anti_pattern_types().len() > 0);
}

#[tokio::test]
async fn test_configuration_driven_setup() {
    use uveddi::analysis::AnalysisConfig;
    
    let config = AnalysisConfig::default();
    let engine = config.create_engine().await.unwrap();
    
    // Should have default detectors
    assert!(engine.get_anti_pattern_types().len() >= 5);
}

#[test]
fn test_backward_compatibility() {
    let engine = AnalysisEngine::new().unwrap();
    
    // Should have default detectors
    assert!(engine.get_anti_pattern_types().len() >= 5);
}
```

### **Task 4.2: Integration Tests**
**File**: `tests/analysis/engine_integration.rs` (NEW FILE)

```rust
use uveddi::analysis::{AnalysisEngine, AnalysisConfig};
use std::path::Path;
use tempfile::tempdir;

#[tokio::test]
async fn test_config_file_integration() {
    let config_content = r#"
cache_size = 500
enable_plugins = false

[detectors.god_object]
threshold_methods = 10
threshold_fields = 15

[detectors.code_duplication]
"#;
    
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    std::fs::write(&config_path, config_content).unwrap();
    
    let config = AnalysisConfig::from_file(&config_path).unwrap();
    let engine = config.create_engine().await.unwrap();
    
    assert!(engine.get_anti_pattern_types().len() >= 2);
}

#[tokio::test]
async fn test_plugin_integration() {
    let mut engine = AnalysisEngine::new_with_plugins().await.unwrap();
    
    // Test that plugin support is available
    assert!(engine.has_plugin_support());
    
    // Test loading plugins (will be empty in test environment)
    let loaded_count = engine.load_plugins().await.unwrap();
    assert_eq!(loaded_count, 0); // No plugins in test environment
}
```

### **Task 4.3: Performance Validation**
**File**: `benches/dependency_injection.rs` (NEW FILE)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uveddi::analysis::{AnalysisEngine, DetectorFactory};
use std::path::Path;

fn benchmark_old_vs_new_construction(c: &mut Criterion) {
    c.bench_function("old_constructor", |b| {
        b.iter(|| {
            let engine = AnalysisEngine::new().unwrap();
            black_box(engine);
        })
    });
    
    c.bench_function("new_constructor_with_di", |b| {
        b.iter(|| {
            let detectors = DetectorFactory::create_default_detectors();
            let engine = AnalysisEngine::with_detectors(detectors, None, false).unwrap();
            black_box(engine);
        })
    });
}

criterion_group!(benches, benchmark_old_vs_new_construction);
criterion_main!(benches);
```

---

## **📝 DETAILED TODO CHECKLIST**

### **Phase 1: Core Refactoring (8 hours)**
- [ ] **1.1** Create `src/analysis/engine_builder.rs` with builder pattern
- [ ] **1.2** Add `with_detectors()` method to `AnalysisEngine`
- [ ] **1.3** Create `src/analysis/detector_factory.rs` with factory methods
- [ ] **1.4** Update existing `new()` method for backward compatibility
- [ ] **1.5** Update `src/analysis/mod.rs` to export new modules
- [ ] **1.6** Add required dependencies to `Cargo.toml` (serde, toml)
- [ ] **1.7** Run `cargo check` and fix compilation errors
- [ ] **1.8** Run existing tests to ensure backward compatibility

### **Phase 2: Registry & Configuration (6 hours)**
- [ ] **2.1** Create `src/analysis/detector_registry.rs`
- [ ] **2.2** Create `src/analysis/config.rs` with TOML support
- [ ] **2.3** Create example configuration file `examples/analysis_config.toml`
- [ ] **2.4** Update builder to support configuration loading
- [ ] **2.5** Add configuration validation and error handling
- [ ] **2.6** Update documentation with configuration examples
- [ ] **2.7** Test configuration loading and validation

### **Phase 3: Plugin Integration (4 hours)**
- [ ] **3.1** Create `src/plugins/detector_adapter.rs`
- [ ] **3.2** Update `AnalysisEngine` with plugin detector methods
- [ ] **3.3** Integrate adapter with existing plugin system
- [ ] **3.4** Update builder to support plugin loading
- [ ] **3.5** Test plugin integration (with mock plugins)
- [ ] **3.6** Update plugin documentation

### **Phase 4: Testing & Documentation (6 hours)**
- [ ] **4.1** Create comprehensive unit tests in `tests/analysis/dependency_injection.rs`
- [ ] **4.2** Create integration tests in `tests/analysis/engine_integration.rs`
- [ ] **4.3** Create performance benchmarks in `benches/dependency_injection.rs`
- [ ] **4.4** Update API documentation with examples
- [ ] **4.5** Create migration guide for existing users
- [ ] **4.6** Update README with new configuration options
- [ ] **4.7** Run full test suite and ensure >90% coverage
- [ ] **4.8** Performance validation (<5% overhead)

---

## **🔧 IMPLEMENTATION REQUIREMENTS**

### **Code Quality Standards**
- [ ] All new code must pass `cargo check --all-targets`
- [ ] All tests must pass with `cargo test`
- [ ] Code coverage must be >90% for new modules
- [ ] All public APIs must have documentation with examples
- [ ] Follow existing error handling patterns with `UveddiError`

### **Backward Compatibility Requirements**
- [ ] Existing `AnalysisEngine::new()` must continue to work
- [ ] All existing public APIs must remain functional
- [ ] Default behavior must be identical to current implementation
- [ ] Performance impact must be <5%

### **Integration Requirements**
- [ ] Must integrate with existing plugin system
- [ ] Must support WASM plugin adapters
- [ ] Must work with existing cache system
- [ ] Must maintain symbol table functionality

---

## **🚨 CRITICAL SUCCESS CRITERIA**

### **Functional Requirements**
- [ ] ✅ Engine can be created with custom detector sets
- [ ] ✅ All detectors can be mocked for testing
- [ ] ✅ Configuration file can specify detector setup
- [ ] ✅ Plugin detectors integrate seamlessly
- [ ] ✅ Backward compatibility maintained
- [ ] ✅ Performance impact <5%

### **Technical Requirements**
- [ ] ✅ Builder pattern implemented correctly
- [ ] ✅ Dependency injection working as specified
- [ ] ✅ Registry pattern functional
- [ ] ✅ Configuration loading robust
- [ ] ✅ Error handling comprehensive
- [ ] ✅ Thread safety maintained (Send + Sync)

### **Testing Requirements**
- [ ] ✅ Unit tests with mocks pass
- [ ] ✅ Integration tests pass
- [ ] ✅ Performance benchmarks show <5% overhead
- [ ] ✅ Backward compatibility tests pass
- [ ] ✅ Configuration validation tests pass

---

## **🎯 VALIDATION COMMANDS**

```bash
# Compilation check
cargo check --all-targets

# Run all tests
cargo test

# Run specific module tests
cargo test analysis::dependency_injection

# Run benchmarks
cargo bench dependency_injection

# Check formatting
cargo fmt --check

# Lint check
cargo clippy -- -D warnings

# Documentation check
cargo doc --no-deps --open
```

---

## **📚 REFERENCE FILES TO EXAMINE**

### **Current Implementation**
- `src/analysis/engine.rs` - Current AnalysisEngine implementation
- `src/analysis/mod.rs` - Module exports and AnalysisDetector trait
- `src/analysis/detectors/anti_patterns/mod.rs` - Detector implementations
- `src/plugins/engine.rs` - Plugin system integration

### **Related Tests**
- `tests/analysis/engine.rs` - Existing engine tests
- `tests/integration_sprint1.rs` - Integration test patterns

### **Configuration Examples**
- `Cargo.toml` - Dependency management
- `examples/` - Existing example patterns

---

## **🔄 MIGRATION GUIDE FOR USERS**

### **Before (Current)**
```rust
let engine = AnalysisEngine::new()?;
```

### **After (New Options)**
```rust
// Option 1: Backward compatible (no changes needed)
let engine = AnalysisEngine::new()?;

// Option 2: Custom detectors
let detectors = vec![Box::new(GodObjectDetector::new(10, 15))];
let engine = AnalysisEngine::with_detectors(detectors, None, false)?;

// Option 3: Builder pattern
let engine = AnalysisEngineBuilder::new()
    .add_detector(Box::new(GodObjectDetector::new(10, 15)))
    .enable_plugins()
    .build().await?;

// Option 4: Configuration file
let config = AnalysisConfig::from_file("config.toml")?;
let engine = config.create_engine().await?;
```

---

## **⚡ QUICK START IMPLEMENTATION**

1. **Start with Phase 1** - Focus on dependency injection core
2. **Maintain backward compatibility** - Ensure existing code works
3. **Test incrementally** - Run tests after each major change
4. **Document as you go** - Add examples to new APIs
5. **Performance monitor** - Benchmark critical paths

**Ready to transform the Uveddi AnalysisEngine! 🚀**