# Configuration Baseline Audit

**Date:** October 14, 2025  
**Author:** GitHub Copilot (Assignment 1)  
**Status:** Baseline Review Complete  
**Related Issues:** UV-102 (Magic Values Detection), UV-TBD (CLI Config Enhancement)

---

## Executive Summary

This audit documents the current configuration architecture of Uveddi, identifying strengths, limitations, and gaps that need addressing before implementing an enhanced CLI configuration workflow. The system has evolved through three distinct configuration generations, creating complexity but also providing migration paths.

**Key Findings:**
- ✅ Strong foundation with `StandardDetectorConfig` system (newest generation)
- ⚠️ Limited CLI support - only handles `ollama_model` via `config set`
- ❌ No way to configure detector thresholds via CLI
- ❌ Missing metadata layer for CLI discoverability (descriptions, help text)
- ✅ Comprehensive TOML support with language-specific overrides
- ⚠️ Three coexisting config formats (legacy, enhanced, standardized)

---

## 1. CLI Configuration Flow Analysis

### 1.1 Current CLI Implementation

**Location:** `src/cli/commands/config.rs`

**Supported Commands:**

```bash
uveddi config show [--file <path>]
uveddi config set <key> <value> [--file <path>]
uveddi config validate [--file <path>] [--suggestions] [--format <json|human>]
```

**Module Dependencies:**
```
cli/commands/config.rs
  ├─> config::Config              (src/config/mod.rs)
  ├─> config::validation          (src/config/validation.rs)
  └─> core::logging               (logging facade)
```

### 1.2 Config::Config Structure

**Location:** `src/config/mod.rs`

```rust
pub struct Config {
    pub ollama_model: Option<String>,
    pub dead_code: Option<DeadCodeConfig>,
    pub large_classes: Option<LargeClassConfig>,
}
```

**Limitations:**
1. **Top-level config only** - Contains AI settings and two detector configs
2. **Hardcoded in `set` command** - Only accepts `ollama_model` key
3. **No detector registry** - No way to enumerate or discover detectors
4. **Separate from analysis config** - Not connected to `AnalysisConfig`

### 1.3 AnalysisConfig Structure

**Location:** `src/analysis/config.rs`

```rust
pub struct AnalysisConfig {
    pub detectors: HashMap<String, DetectorConfig>,              // Legacy format
    pub enhanced_detectors: HashMap<String, EnhancedDetectorConfig>, // V2 format
    pub standard_detectors: HashMap<String, StandardDetectorConfig>, // V3 format (newest)
    pub cache_size: Option<usize>,
    pub enable_plugins: bool,
    pub cache_path: Option<String>,
    pub cache_settings: CacheConfig,
    pub performance_settings: PerformanceConfig,
}
```

**Key Methods:**
- `from_file(path: &Path)` - Loads TOML configuration
- `create_engine()` - Creates analysis engine from config
- `get_effective_detector_config(detector_name: &str)` - Priority resolution
- `set_detector_config()` - Update detector configuration

**Priority Chain:**
```
standard_detectors > enhanced_detectors > detectors (legacy) > default
```

### 1.4 Validation System

**Location:** `src/config/validation.rs`

**Features:**
- Smart validation with suggestions
- Performance scoring
- Completeness analysis
- Error severity levels (Critical, High, Medium, Low)
- Impact levels for warnings
- Prioritized suggestions
- JSON and human-readable output

**Currently validates but CLI cannot modify:**
- Detector thresholds
- Language-specific overrides
- Exclusion patterns
- Advanced settings

---

## 2. Detector Configuration Catalog

### 2.1 Active Detectors (from DetectorFactory)

**Location:** `src/analysis/detector_factory.rs::create_default_detectors()`

Seven detectors are created by default:

1. **GodObjectDetector** ✅
2. **CodeDuplicationDetector** ✅
3. **DeadCodeDetector** ✅
4. **LargeClassDetector** ✅
5. **TightCouplingDetector** ✅
6. **LongMethodsDetector** ✅
7. **MagicValuesDetector** ✅

(8. **MainSecurityDetector** - optional, requires `security` feature flag)

---

### 2.2 Detector-by-Detector Analysis

#### 2.2.1 GodObjectDetector

**Location:** `src/analysis/detectors/anti_patterns/god_object/`

**Config Structure:**
```rust
pub struct GodObjectConfig {
    pub method_thresholds: HashMap<SourceLanguage, usize>,
    pub field_thresholds: HashMap<SourceLanguage, usize>,
    pub framework_modules: HashSet<String>,
    pub generated_file_patterns: Vec<String>,
    pub recognize_patterns: bool,
    pub enable_behavioral_analysis: bool,
    pub enable_cohesion_analysis: bool,
    pub trivial_method_cc_threshold: usize,
}
```

**Defaults (from `default_for_detector`):**
| Language   | max_methods | max_fields |
|-----------|-------------|------------|
| Rust      | 25          | 20         |
| Python    | 20          | 15         |
| JavaScript| 18          | 12         |
| Default   | 20          | 15         |

**StandardDetectorConfig defaults:**
```toml
[detectors.god_object]
enabled = true
severity = "High"

[detectors.god_object.thresholds]
max_methods = 20
max_fields = 15

[detectors.god_object.language_overrides.rust]
max_methods = 25
max_fields = 20
```

**Key Thresholds:**
- `max_methods` (int) - Maximum methods per class/struct
- `max_fields` (int) - Maximum fields per class/struct
- `trivial_method_cc_threshold` (int, default: 2) - Complexity threshold for trivial methods
- `recognize_patterns` (bool, default: true) - Enable pattern recognition
- `enable_behavioral_analysis` (bool, default: true)
- `enable_cohesion_analysis` (bool, default: true)

**Exclusions:**
- Framework modules (React, Angular, Vue, etc.)
- Generated files (`*_pb2.py`, `*.g.cs`, `*.generated.*`)

**CLI Gaps:**
- ❌ Cannot enable/disable via CLI
- ❌ Cannot set thresholds via CLI
- ❌ Cannot configure language-specific overrides
- ❌ Cannot manage exclusion patterns

---

#### 2.2.2 CodeDuplicationDetector

**Location:** `src/analysis/detectors/anti_patterns/code_duplication/`

**Config Structure:**
```rust
pub struct DuplicationConfig {
    pub base: BaseConfig,
    pub min_tokens: usize,
    pub min_lines: usize,
    pub similarity_threshold: f64,
    pub fingerprint_length: usize,
    pub ignore_identifiers: bool,
    pub ignore_literals: bool,
    pub ignore_whitespace: bool,
    pub ignore_comments: bool,
    pub enable_cfg_analysis: bool,
    pub enable_semantic_features: bool,
    pub cfg_similarity_weight: f64,
    pub semantic_similarity_threshold: f64,
    pub wl_kernel_iterations: usize,
    pub max_cfg_nodes: usize,
    pub enable_parallel: bool,
    pub max_memory_mb: usize,
    pub timeout_per_file_secs: u64,
    pub min_clone_similarity: f64,
    pub max_clone_pairs: Option<usize>,
    pub skip_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
}
```

**Defaults (from DuplicationConfig::new()):**
```rust
min_tokens: 50
min_lines: 10
similarity_threshold: 0.8
fingerprint_length: 16
ignore_identifiers: true
ignore_literals: true
ignore_whitespace: true
ignore_comments: true
enable_cfg_analysis: false
enable_semantic_features: false
cfg_similarity_weight: 0.3
semantic_similarity_threshold: 0.7
wl_kernel_iterations: 3
max_cfg_nodes: 1000
enable_parallel: true
max_memory_mb: 512
timeout_per_file_secs: 30
min_clone_similarity: 0.8
max_clone_pairs: Some(100)
```

**StandardDetectorConfig defaults:**
```toml
[detectors.code_duplication]
enabled = true
severity = "Medium"

[detectors.code_duplication.thresholds]
min_tokens = 50
min_lines = 5
similarity_threshold = 0.8
fingerprint_length = 10
```

**Key Thresholds:**
- `min_tokens` (int, default: 50) - Minimum tokens for duplicate detection
- `min_lines` (int, default: 10) - Minimum lines for duplicate detection
- `similarity_threshold` (float, 0.0-1.0, default: 0.8) - Type-3 clone threshold
- `fingerprint_length` (int, default: 16) - Rolling hash window size
- `semantic_similarity_threshold` (float, 0.0-1.0, default: 0.7) - Type-4 threshold

**Advanced Settings:**
- `enable_cfg_analysis` (bool) - Control Flow Graph analysis
- `enable_semantic_features` (bool) - Semantic similarity detection
- `cfg_similarity_weight` (float, 0.0-1.0) - CFG importance in scoring
- `wl_kernel_iterations` (int) - Graph kernel computation iterations
- `max_cfg_nodes` (int) - Performance limit for CFG analysis

**Language-Specific Thresholds:**
Managed by `ThresholdManager` (src/.../threshold_manager.rs):
| Language   | type1 | type2 | type3 | type4 | min_tokens | min_lines |
|-----------|-------|-------|-------|-------|------------|-----------|
| Rust      | 0.95  | 0.85  | 0.75  | 0.65  | 50         | 8         |
| Python    | 0.92  | 0.80  | 0.70  | 0.60  | 40         | 6         |
| JavaScript| 0.90  | 0.78  | 0.68  | 0.58  | 35         | 5         |

**CLI Gaps:**
- ❌ Cannot tune Type-1/2/3/4 thresholds
- ❌ Cannot configure language-specific thresholds
- ❌ No way to enable/disable CFG or semantic analysis
- ❌ Cannot adjust performance settings

---

#### 2.2.3 LongMethodsDetector

**Location:** `src/analysis/detectors/anti_patterns/long_methods/`

**Config Structure:**
```rust
pub struct LongMethodsConfig {
    pub thresholds: HashMap<SourceLanguage, LanguageThresholds>,
    pub skip_test_files: bool,
    pub skip_generated_files: bool,
    pub min_severity_score: u32,
    pub max_issues: Option<usize>,
}

pub struct LanguageThresholds {
    pub max_logical_loc: u32,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_nesting_depth: u32,
    pub max_parameters: u32,
}
```

**Defaults (from LanguageThresholds::default()):**
```rust
max_logical_loc: 50
max_cyclomatic_complexity: 10
max_cognitive_complexity: 15
max_nesting_depth: 4
max_parameters: 5
```

**Preset Configurations:**
- **strict()** - 30% stricter (e.g., max_loc: 35, complexity: 8)
- **lenient()** - 50% more relaxed (e.g., max_loc: 75, complexity: 13)

**StandardDetectorConfig mapping:**
```toml
[detectors.long_methods]
enabled = true
severity = "Medium"

[detectors.long_methods.thresholds]
max_lines = 50
max_cyclomatic_complexity = 10
max_cognitive_complexity = 15
```

**Key Thresholds:**
- `max_logical_loc` (int, default: 50) - Max method lines (non-blank, non-comment)
- `max_cyclomatic_complexity` (int, default: 10) - McCabe complexity
- `max_cognitive_complexity` (int, default: 15) - Cognitive complexity
- `max_nesting_depth` (int, default: 4) - Max control flow nesting
- `max_parameters` (int, default: 5) - Max function parameters

**Language-Specific Defaults:**
All languages use same defaults currently. Config allows per-language customization via `thresholds: HashMap<SourceLanguage, LanguageThresholds>`.

**CLI Gaps:**
- ❌ Cannot switch between strict/lenient presets
- ❌ Cannot configure per-language thresholds
- ❌ Cannot adjust min_severity_score
- ❌ No control over test/generated file skipping

---

#### 2.2.4 DeadCodeDetector

**Location:** `src/analysis/detectors/anti_patterns/dead_code/`

**Config Structure (two locations!):**

1. **Detector-specific config:**
```rust
// src/analysis/detectors/anti_patterns/dead_code/config.rs
pub struct DeadCodeConfig {
    pub min_confidence: f64,
    pub library_mode: bool,
    pub ignore_patterns: Vec<String>,
    pub keep_alive_patterns: Vec<String>,
}
```

2. **Top-level config:**
```rust
// src/config/mod.rs
pub struct DeadCodeConfig {
    pub confidence_threshold: Option<f64>,
    pub library_mode: Option<bool>,
    pub ignore_patterns: Option<Vec<String>>,
    pub keep_alive_patterns: Option<Vec<String>>,
}
```

**Defaults (detector-specific):**
```rust
min_confidence: 0.5
library_mode: false
ignore_patterns: ["test", "tests", "spec", "mock"]
keep_alive_patterns: ["main", "init", "setup", "teardown"]
```

**StandardDetectorConfig defaults:**
```toml
[detectors.dead_code]
enabled = true
severity = "Low"

[detectors.dead_code.thresholds]
confidence_threshold = 0.7
```

**Key Thresholds:**
- `min_confidence` / `confidence_threshold` (float, 0.0-1.0, default: 0.5/0.7) - Minimum confidence
- `library_mode` (bool, default: false) - Treat exports as entry points
- `ignore_patterns` (array of strings) - File patterns to skip
- `keep_alive_patterns` (array of strings) - Symbols to preserve

**CLI Gaps:**
- ❌ Two config structs with same name - confusing
- ❌ Cannot toggle library_mode
- ❌ Cannot adjust confidence threshold
- ❌ Cannot manage keep-alive patterns

---

#### 2.2.5 TightCouplingDetector

**Location:** `src/analysis/detectors/anti_patterns/tight_coupling/`

**Config Structure:**
```rust
pub struct TightCouplingConfig {
    pub rust_thresholds: CouplingThresholds,
    pub python_thresholds: CouplingThresholds,
    pub javascript_thresholds: CouplingThresholds,
    pub enable_cross_file_analysis: bool,
    pub exclude_patterns: Vec<String>,
    pub include_test_files: bool,
    pub visualization_enabled: bool,
    pub generate_dependency_graph: bool,
    pub generate_coupling_matrix: bool,
    pub max_components_for_visualization: usize,
}

pub struct CouplingThresholds {
    pub fan_out_warning: usize,
    pub fan_out_critical: usize,
    pub cbo_warning: usize,
    pub cbo_critical: usize,
    pub rfc_warning: usize,
    pub rfc_critical: usize,
    pub production_multiplier: f64,
    pub test_multiplier: f64,
    pub framework_multiplier: f64,
}
```

**Defaults (language-specific):**
| Metric         | Rust W/C | Python W/C | JavaScript W/C |
|----------------|----------|------------|----------------|
| fan_out        | 3/7      | 10/15      | 12/18          |
| cbo            | 6/10     | 8/12       | 10/15          |
| rfc            | 15/25    | 18/30      | 20/35          |

**StandardDetectorConfig defaults:**
```toml
[detectors.tight_coupling]
enabled = true
severity = "Medium"

[detectors.tight_coupling.thresholds]
max_dependencies = 10
max_coupling_ratio = 0.3
```

**Key Thresholds:**
- `fan_out_warning/critical` (int) - Efferent coupling (dependencies)
- `cbo_warning/critical` (int) - Coupling Between Objects
- `rfc_warning/critical` (int) - Response For a Class
- `production_multiplier` (float, default: 1.0) - Production code adjustment
- `test_multiplier` (float, default: 1.5) - Test code adjustment
- `framework_multiplier` (float, default: 2.0) - Framework code adjustment

**Advanced Settings:**
- `enable_cross_file_analysis` (bool, default: true)
- `visualization_enabled` (bool, default: false)
- `generate_dependency_graph` (bool, default: false)
- `generate_coupling_matrix` (bool, default: false)
- `max_components_for_visualization` (int, default: 100)

**CLI Gaps:**
- ❌ Complex nested structure not CLI-friendly
- ❌ Warning vs Critical thresholds not in StandardDetectorConfig
- ❌ Cannot configure per-language thresholds
- ❌ No way to enable visualization features

---

#### 2.2.6 LargeClassDetector

**Location:** `src/analysis/detectors/anti_patterns/large_classes.rs`

**Config Structure:**
```rust
pub struct LargeClassConfig {
    pub rust_thresholds: LanguageThresholds,
    pub python_thresholds: LanguageThresholds,
    pub javascript_thresholds: LanguageThresholds,
    pub enable_lcom_analysis: bool,
    pub enable_coupling_analysis: bool,
    pub severity_weights: SeverityWeights,
}

pub struct LanguageThresholds {
    pub max_logical_loc: u32,
    pub max_methods: u32,
    pub max_fields: u32,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_lcom_score: f64,
    pub max_coupling: u32,
}

pub struct SeverityWeights {
    pub size_weight: f64,
    pub complexity_weight: f64,
    pub structural_weight: f64,
}
```

**Defaults (from constants):**
Located in `src/constants.rs` - `detector_thresholds` module:
```rust
// Rust thresholds
MAX_LOGICAL_LOC: 300
MAX_METHODS: 20
MAX_FIELDS: 15
MAX_CYCLOMATIC_COMPLEXITY: 50
MAX_COGNITIVE_COMPLEXITY: 40
MAX_LCOM_SCORE: 0.8
MAX_COUPLING: 30

// Python thresholds
MAX_LOGICAL_LOC: 400
MAX_METHODS: 25
MAX_FIELDS: 20
// ... etc
```

**StandardDetectorConfig defaults:**
```toml
[detectors.large_classes]
enabled = true
severity = "Medium"

[detectors.large_classes.thresholds]
max_lines = 300
max_methods = 20
max_complexity = 50
```

**Key Thresholds:**
- `max_logical_loc` (int) - Max non-blank, non-comment lines
- `max_methods` (int) - Max methods per class
- `max_fields` (int) - Max fields per class
- `max_cyclomatic_complexity` (int) - Total class complexity
- `max_cognitive_complexity` (int) - Total cognitive complexity
- `max_lcom_score` (float, 0.0-1.0) - Lack of Cohesion threshold
- `max_coupling` (int) - Max coupling count

**Advanced Settings:**
- `enable_lcom_analysis` (bool, default: true)
- `enable_coupling_analysis` (bool, default: true)
- `size_weight` (float, default: 0.4)
- `complexity_weight` (float, default: 0.3)
- `structural_weight` (float, default: 0.3)

**CLI Gaps:**
- ❌ No access to thresholds via CLI
- ❌ Cannot configure language-specific thresholds
- ❌ Cannot toggle LCOM or coupling analysis
- ❌ Cannot adjust severity weights

---

#### 2.2.7 MagicValuesDetector

**Location:** `src/analysis/detectors/anti_patterns/magic_values.rs`

**Config Structure:**
```rust
pub struct MagicValuesConfig {
    pub allowed_integers: HashSet<i64>,
    pub allowed_floats: HashSet<String>,
    pub ignore_powers_of_two: bool,
    pub ignore_array_indices: bool,
    pub ignore_const_declarations: bool,
    pub ignore_math_constants: bool,
    pub max_string_length: usize,
    pub ignore_simple_strings: bool,
}
```

**Defaults:**
```rust
allowed_integers: [0, 1, -1, 2]
allowed_floats: ["0.0", "1.0", "-1.0"]
ignore_powers_of_two: true
ignore_array_indices: true
ignore_const_declarations: true
ignore_math_constants: true
max_string_length: 50
ignore_simple_strings: true
```

**StandardDetectorConfig status:**
❌ **Not yet integrated with StandardDetectorConfig!**

**Key Thresholds:**
- `allowed_integers` (set of i64) - Universal exceptions
- `allowed_floats` (set of string) - Float exceptions
- `ignore_powers_of_two` (bool, default: true) - Ignore 2, 4, 8, 16, etc.
- `ignore_array_indices` (bool, default: true) - Ignore 0-99 in array contexts
- `ignore_const_declarations` (bool, default: true) - Ignore values in constants
- `ignore_math_constants` (bool, default: true) - Ignore π approximations
- `max_string_length` (int, default: 50) - Max string length to check
- `ignore_simple_strings` (bool, default: true) - Ignore punctuation/delimiters

**CLI Gaps:**
- ❌ Not in StandardDetectorConfig (newest system)
- ❌ No CLI access at all
- ❌ No TOML configuration support
- ❌ Complex HashSet fields not CLI-friendly

---

### 2.3 Detector Summary Table

| Detector               | Config Location | Std Config | Lang Specific | Severity Default | CLI Access |
|------------------------|-----------------|------------|---------------|------------------|------------|
| GodObject              | ✅ Dedicated    | ✅ Yes     | ✅ Yes        | High             | ❌ None    |
| CodeDuplication        | ✅ Dedicated    | ✅ Yes     | ✅ Via Manager| Medium           | ❌ None    |
| LongMethods            | ✅ Dedicated    | ✅ Yes     | ✅ Yes        | Medium           | ❌ None    |
| DeadCode               | ⚠️ Two places   | ✅ Yes     | ❌ No         | Low              | ❌ None    |
| TightCoupling          | ✅ Dedicated    | ✅ Yes     | ✅ Yes        | Medium           | ❌ None    |
| LargeClass             | ✅ Dedicated    | ✅ Yes     | ✅ Yes        | Medium           | ❌ None    |
| MagicValues            | ✅ Dedicated    | ❌ No      | ❌ No         | Low              | ❌ None    |

---

## 3. Metadata and Structural Gaps

### 3.1 Missing Metadata Layer

**Problem:** No centralized detector registry with metadata.

**Current State:**
- Detector names are hardcoded strings ("god_object", "code_duplication")
- No human-readable names or descriptions
- No help text for thresholds
- No validation rules exposed
- No categorization or grouping

**What's Needed:**

```rust
pub struct DetectorMetadata {
    pub name: String,                    // "god_object"
    pub display_name: String,            // "God Object Detector"
    pub description: String,             // "Identifies classes that..."
    pub category: DetectorCategory,      // AntiPattern, Security, Performance
    pub severity_default: IssueSeverity, // High, Medium, Low
    pub enabled_default: bool,
    pub thresholds: Vec<ThresholdMetadata>,
    pub language_support: Vec<SourceLanguage>,
    pub documentation_url: Option<String>,
}

pub struct ThresholdMetadata {
    pub name: String,                    // "max_methods"
    pub display_name: String,            // "Maximum Methods"
    pub description: String,             // "Maximum number of methods..."
    pub value_type: ThresholdType,       // Integer, Float, Boolean, String
    pub default_value: ConfigValue,
    pub min_value: Option<ConfigValue>,
    pub max_value: Option<ConfigValue>,
    pub validation_rules: Vec<ValidationRule>,
    pub examples: Vec<String>,
}
```

**StandardDetectorConfig has `DetectorMetadata` but it's underused:**
```rust
pub struct DetectorMetadata {
    pub version: String,       // Used for migration
    pub description: String,   // ❌ Empty by default
    pub author: String,        // ❌ Empty by default
    pub last_modified: String, // ❌ Empty by default
}
```

### 3.2 Missing CLI-Friendly Features

#### 3.2.1 No Detector Registry

**Gap:** No central registry to enumerate detectors.

**Current Workaround:**
```rust
DetectorFactory::create_default_detectors() // Returns Vec<Box<dyn AnalysisDetector>>
```

**Problem:**
- Returns trait objects, not metadata
- Can't introspect without instantiation
- Hard to build "list detectors" command

**Solution Needed:**
```rust
pub struct DetectorRegistry {
    detectors: HashMap<String, DetectorRegistration>,
}

pub struct DetectorRegistration {
    metadata: DetectorMetadata,
    factory: Box<dyn Fn(StandardDetectorConfig) -> Box<dyn AnalysisDetector>>,
}

impl DetectorRegistry {
    pub fn list_detectors() -> Vec<&DetectorMetadata>;
    pub fn get_metadata(name: &str) -> Option<&DetectorMetadata>;
    pub fn get_default_config(name: &str) -> StandardDetectorConfig;
    pub fn create_detector(name: &str, config: StandardDetectorConfig) -> Result<Box<dyn AnalysisDetector>>;
}
```

#### 3.2.2 No Enable/Disable Flags in Config Structs

**Gap:** Detector-specific configs don't have enable flags.

**Current State:**
```rust
// GodObjectConfig has no enable field
pub struct GodObjectConfig {
    pub method_thresholds: HashMap<SourceLanguage, usize>,
    // ... no enabled: bool
}
```

**StandardDetectorConfig has it:**
```rust
pub struct StandardDetectorConfig {
    pub enabled: bool,  // ✅ Good!
    // ...
}
```

**Problem:**
- Old detector configs can't be enabled/disabled individually
- Must use AnalysisConfig.standard_detectors map presence

#### 3.2.3 No Severity in Detector Configs

**Gap:** Detector configs don't expose severity.

**Current State:**
- Detectors return `ArchitecturalIssue` with hardcoded severity
- `StandardDetectorConfig` has severity ✅
- Old configs don't ❌

**Example from GodObjectDetector:**
```rust
// Hardcoded in detect() method
let issue = ArchitecturalIssue {
    severity: if violations > 5 { "high" } else { "medium" },
    // ...
};
```

**Solution:** All detectors should read severity from config.

#### 3.2.4 Threshold Validation Not Exposed

**Gap:** Validation logic is in `StandardConfigBuilder` but not accessible.

**Current:**
```rust
impl StandardConfigBuilder {
    fn validate_god_object(&self) -> Result<(), UveddiError> {
        // Validation logic here
    }
}
```

**Problem:**
- CLI can't validate user input before building
- No way to show valid ranges/types

**Solution:**
```rust
pub trait ThresholdValidator {
    fn validate(&self, value: &ConfigValue) -> Result<(), ValidationError>;
    fn get_constraints(&self) -> ThresholdConstraints;
}

pub struct ThresholdConstraints {
    pub min: Option<ConfigValue>,
    pub max: Option<ConfigValue>,
    pub allowed_values: Option<Vec<ConfigValue>>,
    pub value_type: ConfigValueType,
}
```

### 3.3 Three Coexisting Config Formats

**Legacy Format (V1):**
```rust
pub detectors: HashMap<String, DetectorConfig>

pub struct DetectorConfig {
    params: HashMap<String, i32>  // Very limited
}
```

**Enhanced Format (V2):**
```rust
pub enhanced_detectors: HashMap<String, EnhancedDetectorConfig>

pub struct EnhancedDetectorConfig {
    pub enabled: bool,
    pub severity: IssueSeverity,
    pub thresholds: DetectorThresholds,  // Still limited
}
```

**Standardized Format (V3 - Current):**
```rust
pub standard_detectors: HashMap<String, StandardDetectorConfig>

pub struct StandardDetectorConfig {
    pub enabled: bool,
    pub severity: IssueSeverity,
    pub thresholds: HashMap<String, ConfigValue>,
    pub language_overrides: HashMap<SourceLanguage, HashMap<String, ConfigValue>>,
    pub exclusions: ExclusionConfig,
    pub advanced: AdvancedConfig,
    pub metadata: DetectorMetadata,
}
```

**Problem:**
- Three parallel systems increase complexity
- Migration system exists but adds overhead
- CLI should target V3 only
- Old configs still in use (DeadCodeConfig duplication)

**Recommendation:**
- CLI should read/write **only** StandardDetectorConfig
- Migrate existing detector structs to use StandardDetectorConfig internally
- Deprecate V1 and V2 formats

### 3.4 Missing Preset System

**Gap:** No way to apply predefined configurations.

**What's Needed:**
```rust
pub enum ConfigPreset {
    Strict,      // Enforce best practices
    Balanced,    // Recommended defaults
    Lenient,     // Gradual improvement
    CI,          // Optimized for CI/CD
    Security,    // Security-focused
}

impl ConfigPreset {
    pub fn apply_to(&self, config: &mut AnalysisConfig);
    pub fn get_detector_config(&self, detector: &str) -> StandardDetectorConfig;
}
```

**Current State:**
- LongMethodsConfig has `strict()` and `lenient()` ✅
- TightCouplingConfig has `with_strict_thresholds()` ✅
- No system-wide presets ❌

### 3.5 Language Override Complexity

**Current State:**
```toml
[detectors.god_object.language_overrides.rust]
max_methods = 25
max_fields = 20

[detectors.god_object.language_overrides.python]
max_methods = 20
max_fields = 15
```

**Problems:**
1. Verbose TOML structure
2. No way to copy config from one language to another
3. No "inherit and modify" pattern
4. CLI would need deep path syntax: `god_object.language_overrides.rust.max_methods`

**Better CLI Syntax:**
```bash
# Set base threshold
uveddi config detector set god_object max_methods 20

# Set language-specific override
uveddi config language set rust god_object.max_methods 30

# Copy configuration
uveddi config language copy rust python

# Show effective configuration for a language
uveddi config language show rust --detector god_object
```

---

## 4. Actionable Recommendations

### 4.1 Immediate Needs (Before CLI Implementation)

#### Priority 1: Create Detector Registry
```rust
// src/analysis/detectors/registry.rs
pub struct DetectorRegistry {
    detectors: HashMap<String, DetectorInfo>,
}

pub struct DetectorInfo {
    name: &'static str,
    display_name: &'static str,
    description: &'static str,
    category: DetectorCategory,
    metadata: DetectorMetadata,
    factory: Box<dyn Fn(StandardDetectorConfig) -> Box<dyn AnalysisDetector>>,
}
```

**Benefits:**
- Enables `uveddi config detectors list`
- Enables `uveddi config detector show <name>`
- Provides metadata for help text
- Centralizes detector creation

#### Priority 2: Enhance DetectorMetadata
```rust
pub struct DetectorMetadata {
    // Existing fields
    pub version: String,
    pub description: String,
    
    // New fields
    pub display_name: String,
    pub category: DetectorCategory,
    pub enabled_default: bool,
    pub severity_default: IssueSeverity,
    pub thresholds: Vec<ThresholdInfo>,
    pub language_support: Vec<SourceLanguage>,
    pub documentation_url: Option<String>,
}

pub struct ThresholdInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub value_type: ConfigValueType,
    pub default_value: ConfigValue,
    pub constraints: Option<ThresholdConstraints>,
}
```

#### Priority 3: Migrate MagicValuesDetector to StandardDetectorConfig
Currently the only detector not using StandardDetectorConfig.

```toml
[detectors.magic_values]
enabled = true
severity = "Low"

[detectors.magic_values.thresholds]
allowed_integers = [0, 1, -1, 2]
allowed_floats = ["0.0", "1.0", "-1.0"]
ignore_powers_of_two = true
ignore_array_indices = true
ignore_const_declarations = true
max_string_length = 50
```

#### Priority 4: Consolidate DeadCodeConfig
Two structs with same name - merge into StandardDetectorConfig.

#### Priority 5: Add Config Read/Write Helpers
```rust
impl AnalysisConfig {
    pub fn get_detector_names(&self) -> Vec<String>;
    pub fn is_detector_enabled(&self, name: &str) -> bool;
    pub fn set_detector_threshold(&mut self, detector: &str, threshold: &str, value: ConfigValue);
    pub fn set_detector_severity(&mut self, detector: &str, severity: IssueSeverity);
    pub fn enable_detector(&mut self, detector: &str);
    pub fn disable_detector(&mut self, detector: &str);
}
```

### 4.2 CLI Implementation Plan

**Phase 1: Discovery Commands**
```bash
uveddi config detectors list
uveddi config detector show <name>
uveddi config detector thresholds <name>
```

**Phase 2: Modification Commands**
```bash
uveddi config detector enable <name>
uveddi config detector set <name> <threshold> <value>
uveddi config detector severity <name> <severity>
```

**Phase 3: Language-Specific Commands**
```bash
uveddi config language set <lang> <detector>.<threshold> <value>
uveddi config language show <lang>
```

**Phase 4: Preset System**
```bash
uveddi config preset apply <strict|balanced|lenient>
uveddi config preset save <name>
```

### 4.3 TOML Configuration Enhancement

**Current:**
```toml
[detectors.god_object]
enabled = true
severity = "High"

[detectors.god_object.thresholds]
max_methods = 20
```

**Enhanced with Comments and Metadata:**
```toml
# God Object Detector
# Identifies classes/structs with too many methods or fields
# Default severity: High | Category: Anti-Pattern
[detectors.god_object]
enabled = true
severity = "High"

[detectors.god_object.thresholds]
# Maximum methods per class (default: 20, range: 5-100)
max_methods = 20

# Maximum fields per class (default: 15, range: 3-50)
max_fields = 15

# Language-specific overrides
[detectors.god_object.language_overrides.rust]
max_methods = 25  # Rust commonly has more trait impls
max_fields = 20
```

**Generator Function:**
```rust
impl StandardDetectorConfig {
    pub fn to_toml_with_comments(&self, metadata: &DetectorMetadata) -> String;
}
```

---

## 5. Integration Points

### 5.1 Config → CLI Data Flow

```
User Command
    ↓
CLI Command Parser (clap)
    ↓
ConfigCommand::execute()
    ↓
AnalysisConfig::from_file() ← Load TOML
    ↓
AnalysisConfig::set_detector_threshold()
    ↓
AnalysisConfig::to_file() → Save TOML
```

### 5.2 CLI → Analysis Engine Flow

```
Updated TOML config
    ↓
AnalysisConfig::from_file()
    ↓
AnalysisConfig::create_engine()
    ↓
DetectorFactory::create_detector(name, config)
    ↓
GodObjectDetector::new() ← Uses StandardDetectorConfig
    ↓
AnalysisEngine::with_detector()
    ↓
Engine::analyze() ← Runs with configured thresholds
```

### 5.3 Required Module Changes

**src/cli/commands/config.rs:**
- Add detector subcommands
- Add language subcommands
- Add preset subcommands
- Enhanced validation display

**src/analysis/config.rs:**
- Add detector manipulation methods
- Add TOML round-trip helpers
- Add config diff/merge functions

**src/analysis/detectors/registry.rs:** (NEW)
- Central detector registration
- Metadata management
- Factory functions

**src/analysis/detector_factory.rs:**
- Integrate with registry
- Support StandardDetectorConfig for all detectors

**src/analysis/standardized_config.rs:**
- Add threshold validation API
- Add preset templates
- Enhance metadata system

---

## 6. Testing Considerations

### 6.1 Config Round-Trip Testing
Ensure TOML → Struct → TOML preserves values:
```rust
#[test]
fn test_config_roundtrip() {
    let original = AnalysisConfig::from_file("test.toml")?;
    original.to_file("output.toml")?;
    let reloaded = AnalysisConfig::from_file("output.toml")?;
    assert_eq!(original, reloaded);
}
```

### 6.2 CLI Command Testing
```rust
#[test]
fn test_detector_set_threshold() {
    let output = Command::new("uveddi")
        .args(&["config", "detector", "set", "god_object", "max_methods", "30"])
        .output()?;
    assert!(output.status.success());
    
    let config = AnalysisConfig::from_file("uveddi.toml")?;
    let threshold = config.get_detector_threshold("god_object", "max_methods")?;
    assert_eq!(threshold.as_int()?, 30);
}
```

### 6.3 Validation Testing
```rust
#[test]
fn test_invalid_threshold_rejected() {
    let result = StandardConfigBuilder::new("code_duplication")
        .threshold("similarity_threshold", 1.5) // Invalid: > 1.0
        .build();
    assert!(result.is_err());
}
```

---

## 7. Documentation Needs

### 7.1 User-Facing Documentation
- CLI command reference
- Configuration file guide
- Detector threshold guide
- Language-specific configuration guide
- Preset guide
- Migration guide (V1/V2 → V3)

### 7.2 Developer Documentation
- Detector registration guide
- Adding new detectors guide
- Config system architecture
- StandardDetectorConfig best practices

---

## 8. Summary of Gaps

### Critical Gaps (Block CLI Implementation)
1. ❌ No detector registry/metadata system
2. ❌ No CLI commands for detector configuration
3. ❌ MagicValuesDetector not in StandardDetectorConfig
4. ❌ DeadCodeConfig duplication (two structs)
5. ❌ No threshold validation API for CLI

### Major Gaps (Limit Functionality)
6. ⚠️ No preset system
7. ⚠️ No human-readable names/descriptions
8. ⚠️ No help text for thresholds
9. ⚠️ Three coexisting config formats
10. ⚠️ Complex language override syntax

### Minor Gaps (Nice to Have)
11. 🔹 No config diff command
12. 🔹 No config export/import
13. 🔹 No wizard/interactive mode
14. 🔹 No config test/preview
15. 🔹 No auto-calibration

---

## 9. Next Steps

**Assignment 1 Complete** ✅

**Ready for Assignment 2:**
- Implement detector registry
- Add CLI discovery commands
- Create detector metadata system

**Recommendations:**
1. Start with detector registry (enables discovery)
2. Migrate MagicValuesDetector to StandardDetectorConfig
3. Consolidate DeadCodeConfig
4. Add AnalysisConfig helper methods
5. Implement basic CLI commands (list, show, set)
6. Add preset system
7. Implement wizard/interactive mode

---

## Appendix A: File Locations

### Configuration Files
- `src/config/mod.rs` - Top-level config (AI, dead_code, large_classes)
- `src/config/validation.rs` - Validation system
- `src/analysis/config.rs` - AnalysisConfig (main config system)
- `src/analysis/standardized_config.rs` - StandardDetectorConfig (V3 format)

### Detector Configs
- `src/analysis/detectors/anti_patterns/god_object/config.rs`
- `src/analysis/detectors/anti_patterns/code_duplication/config.rs`
- `src/analysis/detectors/anti_patterns/long_methods/config.rs`
- `src/analysis/detectors/anti_patterns/dead_code/config.rs`
- `src/analysis/detectors/anti_patterns/tight_coupling/config.rs`
- `src/analysis/detectors/anti_patterns/large_classes.rs` (inline config)
- `src/analysis/detectors/anti_patterns/magic_values.rs` (inline config)

### Factory and Registry
- `src/analysis/detector_factory.rs` - Detector creation
- `src/engine/analysis/detector_factory.rs` - Engine-level factory

### CLI
- `src/cli/commands/config.rs` - Config CLI commands

### Constants
- `src/constants.rs` - Detector threshold constants

---

**End of Config Baseline Audit**
