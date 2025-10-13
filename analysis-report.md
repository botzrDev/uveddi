# Code Analysis Report

**Generated:** 2025-10-13 19:32:47 UTC

## Summary

- **Files Analyzed:** 812
- **Issues Found:** 427
- **Analysis Duration:** 26.66s

---

## Issues by Severity

### 🔴 Critical (21 issues)

#### Long method 'tree_to_custom_ast' detected: 246 lines, 253 statements, complexity 53

- **File:** `./src/ast/tree_sitter_impl.rs`
- **Line:** 295

**Code:**
```
fn tree_to_custom_ast(
        tree: &Tree,
        source: &str,
        language: &SourceLanguage,
    ) -> Result<CustomAst, AstError> {
...
```

**Recommendation:** Consider breaking down 'tree_to_custom_ast' into smaller, more focused methods. Current metrics: LOC=246, Statements=253, Complexity=53, Nesting=22

---

#### Long method 'tree_to_custom_ast' detected: 246 lines, 253 statements, complexity 53

- **File:** `./src/ast/tree_sitter_impl.rs`
- **Line:** 295

**Code:**
```
fn tree_to_custom_ast(
        tree: &Tree,
        source: &str,
        language: &SourceLanguage,
    ) -> Result<CustomAst, AstError> {
...
```

**Recommendation:** Consider breaking down 'tree_to_custom_ast' into smaller, more focused methods. Current metrics: LOC=246, Statements=253, Complexity=53, Nesting=22

---

#### Long method 'check_database_recursive' detected: 157 lines, 123 statements, complexity 30

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/database_patterns.rs`
- **Line:** 22

**Code:**
```
fn check_database_recursive(value: &YamlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                // Check if this looks like a database configuration
                let is_db_config = map.keys().any(|k| {
...
```

**Recommendation:** Consider breaking down 'check_database_recursive' into smaller, more focused methods. Current metrics: LOC=157, Statements=123, Complexity=30, Nesting=14

---

#### Long method 'check_database_recursive' detected: 157 lines, 123 statements, complexity 30

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/database_patterns.rs`
- **Line:** 22

**Code:**
```
fn check_database_recursive(value: &YamlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                // Check if this looks like a database configuration
                let is_db_config = map.keys().any(|k| {
...
```

**Recommendation:** Consider breaking down 'check_database_recursive' into smaller, more focused methods. Current metrics: LOC=157, Statements=123, Complexity=30, Nesting=14

---

#### Long method 'build' detected: 148 lines, 136 statements, complexity 29

- **File:** `./src/analysis/engine_builder.rs`
- **Line:** 205

**Code:**
```
pub fn build(self) -> crate::error::Result<crate::analysis::AnalysisEngine> {
        if self.enable_plugins {
            warn!("Plugins enabled, but building synchronously. Use `build_async().await` for proper plugin initialization.");
        }

...
```

**Recommendation:** Consider breaking down 'build' into smaller, more focused methods. Current metrics: LOC=148, Statements=136, Complexity=29, Nesting=9

---

#### Long method 'build' detected: 148 lines, 136 statements, complexity 29

- **File:** `./src/analysis/engine_builder.rs`
- **Line:** 205

**Code:**
```
pub fn build(self) -> crate::error::Result<crate::analysis::AnalysisEngine> {
        if self.enable_plugins {
            warn!("Plugins enabled, but building synchronously. Use `build_async().await` for proper plugin initialization.");
        }

...
```

**Recommendation:** Consider breaking down 'build' into smaller, more focused methods. Current metrics: LOC=148, Statements=136, Complexity=29, Nesting=9

---

#### Long method 'build_async' detected: 183 lines, 142 statements, complexity 35

- **File:** `./src/analysis/engine_builder.rs`
- **Line:** 401

**Code:**
```
pub async fn build_async(self) -> crate::error::Result<crate::analysis::AnalysisEngine> {
        // Extract all values before using self methods to avoid partial moves
        let enable_knowledge = self.enable_knowledge_enhancement;
        let enable_ai = self.enable_ai_explanations;
        let enable_plugins = self.enable_plugins;
...
```

**Recommendation:** Consider breaking down 'build_async' into smaller, more focused methods. Current metrics: LOC=183, Statements=142, Complexity=35, Nesting=9

---

#### Long method 'build_async' detected: 183 lines, 142 statements, complexity 35

- **File:** `./src/analysis/engine_builder.rs`
- **Line:** 401

**Code:**
```
pub async fn build_async(self) -> crate::error::Result<crate::analysis::AnalysisEngine> {
        // Extract all values before using self methods to avoid partial moves
        let enable_knowledge = self.enable_knowledge_enhancement;
        let enable_ai = self.enable_ai_explanations;
        let enable_plugins = self.enable_plugins;
...
```

**Recommendation:** Consider breaking down 'build_async' into smaller, more focused methods. Current metrics: LOC=183, Statements=142, Complexity=35, Nesting=9

---

#### Long method 'validate_pattern_completeness' detected: 146 lines, 116 statements, complexity 21

- **File:** `./src/ai/knowledge/validation.rs`
- **Line:** 201

**Code:**
```
fn validate_pattern_completeness(
        &self,
        library: &KnowledgeLibrary,
        issues: &mut Vec<ValidationIssue>,
    ) {
...
```

**Recommendation:** Consider breaking down 'validate_pattern_completeness' into smaller, more focused methods. Current metrics: LOC=146, Statements=116, Complexity=21, Nesting=8

---

#### Long method 'validate_pattern_completeness' detected: 146 lines, 116 statements, complexity 21

- **File:** `./src/ai/knowledge/validation.rs`
- **Line:** 201

**Code:**
```
fn validate_pattern_completeness(
        &self,
        library: &KnowledgeLibrary,
        issues: &mut Vec<ValidationIssue>,
    ) {
...
```

**Recommendation:** Consider breaking down 'validate_pattern_completeness' into smaller, more focused methods. Current metrics: LOC=146, Statements=116, Complexity=21, Nesting=8

---

#### God Object detected: 'AnalyzeCommand' has 16 methods and 41 fields. (Thresholds: methods>30, fields>20) LCOM4 score: 1 (>1 indicates low cohesion)

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 122

**Code:**
```
pub struct AnalyzeCommand {
    /// Path to the project or directory to analyze
    ///
    /// Can be a file or directory. When analyzing a directory,
    /// all supported source files will be recursively processed.
    pub path: PathBuf,

    /// Output format for the analysis report
    ///
    /// Supported formats:
    /// - `text`: Plain text format for terminal output
    /// - `json`: Structured JSON format for programmatic consumption
    /// - `markdown`: Markdown format for documentation
    /// - `html`: Interactive HTML format with embedded diagrams and dark/light themes
    #[arg(long, default_value = "markdown")]
    pub output_format: String,

    /// Optional output file path
    ///
    /// If not specified, the report will be written to stdout.
    /// The file extension should match the chosen output format.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Enable AI-powered analysis and explanations
    ///
    /// When enabled, the analysis will include AI-generated explanations
    /// for detected issues and architectural recommendations.
    /// Requires either an API key or a local Ollama instance.
    #[arg(long)]
    pub enable_ai: bool,

    /// Ollama API URL for local AI analysis
    ///
    /// Used when `--enable-ai` is specified and you want to use
    /// a local Ollama instance instead of external AI services.
    ///
    /// Can also be set via the `OLLAMA_API_URL` environment variable.
    #[arg(long, env = "OLLAMA_API_URL")]
    pub ollama_api_url: Option<String>,

    /// Ollama model name for local AI analysis
    ///
    /// Specifies which Ollama model to use for analysis.
    /// Common options include "deepseek-coder:6.7b-instruct-q4_0",
    /// "codellama:7b-instruct", etc.
    ///
    /// Can also be set via the `OLLAMA_MODEL` environment variable.
    #[arg(long, env = "OLLAMA_MODEL")]
    pub ollama_model: Option<String>,

    /// Confidence threshold for dead code detection (0.0 to 1.0)
    ///
    /// Only report dead code issues with confidence above this threshold.
    /// Higher values reduce false positives but may miss some issues.
    #[arg(long, value_name = "THRESHOLD")]
    pub dead_code_confidence: Option<f64>,

    /// Enable library mode for dead code detection
    ///
    /// In library mode, exported symbols are treated more conservatively
    /// to avoid false positives for public APIs.
    #[arg(long)]
    pub dead_code_library_mode: bool,

    /// Patterns to ignore during dead code detection
    ///
    /// Comma-separated list of patterns to exclude from analysis.
    /// Example: "test,spec,mock,generated"
    #[arg(long, value_delimiter = ',')]
    pub dead_code_ignore_patterns: Option<Vec<String>>,

    /// Symbols to always keep alive during dead code detection
    ///
    /// Comma-separated list of symbol patterns that should never be
    /// reported as dead code. Example: "main,init,setup,teardown"
    #[arg(long, value_delimiter = ',')]
    pub dead_code_keep_alive: Option<Vec<String>>,

    /// Maximum logical lines of code threshold for large classes
    ///
    /// Classes exceeding this threshold will be flagged as potentially too large.
    /// Default varies by language (Rust: 400, Python: 1000, JavaScript: 800)
    #[arg(long, value_name = "LINES")]
    pub large_classes_max_loc: Option<u32>,

    /// Maximum number of methods threshold for large classes
    ///
    /// Classes with more methods than this threshold will be flagged.
    /// Default varies by language (Rust: 20, Python: 20, JavaScript: 25)
    #[arg(long, value_name = "COUNT")]
    pub large_classes_max_methods: Option<u32>,

    /// Maximum number of fields threshold for large classes
    ///
    /// Classes with more fields than this threshold will be flagged.
    /// Default varies by language (Rust: 15, Python: 7, JavaScript: 12)
    #[arg(long, value_name = "COUNT")]
    pub large_classes_max_fields: Option<u32>,

    /// Maximum cyclomatic complexity threshold for large classes
    ///
    /// Classes with higher complexity will be flagged as potentially too complex.
    /// Default varies by language (Rust: 50, Python: 60, JavaScript: 55)
    #[arg(long, value_name = "COMPLEXITY")]
    pub large_classes_max_complexity: Option<u32>,

    /// Maximum LCOM (Lack of Cohesion in Methods) score threshold
    ///
    /// Higher values indicate lower cohesion. Range: 0.0 to 1.0
    /// Default: 0.8 for all languages
    #[arg(long, value_name = "SCORE")]
    pub large_classes_max_lcom: Option<f64>,

    /// Patterns to ignore during large classes detection
    ///
    /// Comma-separated list of patterns to exclude from analysis.
    /// Example: "test,spec,mock,generated,fixture"
    #[arg(long, value_delimiter = ',')]
    pub large_classes_ignore_patterns: Option<Vec<String>>,

    /// Minimum severity score for large classes reporting (0-100)
    ///
    /// Only report issues with severity above this threshold.
    /// 0-25: Info, 26-50: Low, 51-75: Medium, 76-90: High, 91-100: Critical
    #[arg(long, value_name = "SCORE", default_value = "25")]
    pub large_classes_min_severity: Option<u32>,

    /// Disable memory optimization features
    ///
    /// By default, Uveddi uses memory optimization (object pooling, arena allocation,
    /// and zero-copy AST caching) for better performance. Use this flag to disable optimizations.
    #[arg(long)]
    pub disable_memory_optimization: bool,

    /// Memory limit in gigabytes for analysis
    ///
    /// Sets a soft limit on memory usage. The analysis will attempt to
    /// stay within this limit by using more aggressive memory management.
    #[arg(long, value_name = "GB")]
    pub memory_limit_gb: Option<f64>,

    /// Memory profile for optimization settings
    ///
    /// Selects pre-configured memory optimization settings:
    /// - `small`: Optimized for small projects (< 1000 files)
    /// - `default`: Balanced settings for most projects (default)
    /// - `large`: Optimized for large codebases (> 10000 files)
    #[arg(long, value_name = "PROFILE", default_value = "default")]
    pub memory_profile: Option<String>,

    // === HYBRID RENDERING OPTIONS ===
    /// Enable image rendering (requires rendering service)
    ///
    /// When enabled, diagrams will be rendered as images using the rendering service.
    /// Falls back to Mermaid-only mode if service is unavailable (unless --no-fallback is used).
    #[arg(long)]
    pub enable_image_rendering: bool,

    /// Force Mermaid-only mode (no image rendering, zero hosting costs)
    ///
    /// Generate only Mermaid code with helpful rendering instructions.
    /// This is the default mode to eliminate hosting costs.
    #[arg(long)]
    pub mermaid_only: bool,

    /// Rendering service URL
    ///
    /// URL of the rendering service for image generation.
    /// Only used when --enable-image-rendering is specified.
    #[arg(long, default_value = "http://localhost:3001")]
    pub rendering_service_url: String,

    /// Disable fallback to Mermaid-only (fail if image rendering unavailable)
    ///
    /// When enabled, analysis will fail if image rendering is requested but unavailable.
    /// By default, the system gracefully falls back to Mermaid-only mode.
    #[arg(long)]
    pub no_fallback: bool,

    /// Check rendering service availability without running analysis
    ///
    /// Performs a health check on the rendering service and exits.
    /// Useful for verifying service configuration before running analysis.
    #[arg(long)]
    pub check_rendering_service: bool,

    /// Disable diagram generation completely
    ///
    /// Skip all diagram generation to speed up analysis when only textual output is needed.
    #[arg(long)]
    pub no_diagrams: bool,

    /// Maximum number of diagrams to generate per report
    ///
    /// Limits the total number of diagrams to prevent performance issues with large codebases.
    /// Set to 0 for no limit.
    #[arg(long, default_value = "20")]
    pub max_diagrams: usize,

    /// Output directory for diagram files
    ///
    /// When using image rendering modes, diagrams will be saved to this directory.
    /// If not specified, a subdirectory next to the report will be created.
    #[arg(long)]
    pub diagram_output_dir: Option<PathBuf>,

    /// Maximum analysis timeout in seconds
    ///
    /// Sets the maximum time to wait for analysis completion before aborting.
    /// Default is 300 seconds (5 minutes). Set to 0 to disable timeout.
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Show detailed error information and stack traces
    ///
    /// When enabled, errors will include additional debug information,
    /// stack traces, and context to help diagnose issues.
    #[arg(long)]
    pub verbose: bool,

    /// Progress reporting format
    ///
    /// Controls how progress is displayed during analysis:
    /// - `terminal`: Rich terminal output with progress bars and estimates (default)
    /// - `json`: JSON progress events for programmatic consumption
    /// - `silent`: No progress output (quiet mode)
    #[arg(long, default_value = "terminal")]
    pub progress_format: String,

    /// Show detailed progress information
    ///
    /// When enabled with terminal progress format, displays current file being processed,
    /// throughput metrics, and detailed timing information.
    #[arg(long)]
    pub progress_details: bool,

    /// Automatically open dashboard after analysis completes
    ///
    /// When enabled, the analysis dashboard will be launched in the default web browser
    /// after analysis is complete, showing the results in a visual interface.
    #[arg(long)]
    pub open_dashboard: bool,

    // === SECURITY ANALYSIS OPTIONS ===
    /// Enable security vulnerability analysis
    ///
    /// Performs comprehensive security analysis including OWASP Top 10 coverage,
    /// taint flow analysis, and vulnerability detection.
    #[arg(long)]
    pub security: bool,

    /// Run only security analysis (skip other detectors)
    ///
    /// When enabled, only security-related anti-pattern detection will be performed.
    /// This provides faster analysis when only security concerns are relevant.
    #[arg(long)]
    pub security_only: bool,

    /// Minimum confidence threshold for security findings (0.0 to 1.0)
    ///
    /// Only report security issues with confidence above this threshold.
    /// Higher values reduce false positives but may miss some vulnerabilities.
    #[arg(long, value_name = "THRESHOLD", default_value = "0.5")]
    pub min_security_confidence: Option<f64>,

    /// Export security findings in SARIF 2.1.0 format
    ///
    /// Generates SARIF-compliant output suitable for GitHub Security tab,
    /// CI/CD pipeline integration, and security toolchain interoperability.
    #[arg(long)]
    pub export_sarif: bool,

    /// SARIF output file path
    ///
    /// Specifies where to save the SARIF export file. If not provided,
    /// defaults to '<output-file-stem>.sarif' or 'security-findings.sarif'.
    #[arg(long)]
    pub sarif_output: Option<PathBuf>,

    /// Enable taint flow analysis for data flow vulnerabilities
    ///
    /// Performs source-to-sink taint analysis to identify potential
    /// injection vulnerabilities and unsafe data flows.
    #[arg(long)]
    pub enable_taint_analysis: bool,

    /// Maximum depth for taint flow analysis (1-20)
    ///
    /// Controls how deep the taint analysis traverses call chains.
    /// Higher values increase accuracy but also analysis time.
    #[arg(long, value_name = "DEPTH", default_value = "10")]
    pub taint_analysis_depth: Option<u8>,

    /// OWASP categories to focus on (comma-separated)
    ///
    /// Limit security analysis to specific OWASP Top 10 2021 categories.
    /// Examples: "A01,A03,A06" or "injection,broken_access_control"
    #[arg(long, value_delimiter = ',')]
    pub owasp_categories: Option<Vec<String>>,
}
```

---

#### Long method 'validate_inputs' detected: 143 lines, 144 statements, complexity 28

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 589

**Code:**
```
pub fn validate_inputs(&self) -> Result<(), SecurityError> {
        // Enhanced CLI argument validation using new security framework
        let path_str = self.path.to_string_lossy();
        validate_cli_argument(&path_str, "path", CliArgumentType::FilePath)?;

...
```

**Recommendation:** Consider breaking down 'validate_inputs' into smaller, more focused methods. Current metrics: LOC=143, Statements=144, Complexity=28, Nesting=6

---

#### Long method 'validate_inputs' detected: 143 lines, 144 statements, complexity 28

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 589

**Code:**
```
pub fn validate_inputs(&self) -> Result<(), SecurityError> {
        // Enhanced CLI argument validation using new security framework
        let path_str = self.path.to_string_lossy();
        validate_cli_argument(&path_str, "path", CliArgumentType::FilePath)?;

...
```

**Recommendation:** Consider breaking down 'validate_inputs' into smaller, more focused methods. Current metrics: LOC=143, Statements=144, Complexity=28, Nesting=6

---

#### Long method 'execute' detected: 166 lines, 159 statements, complexity 35

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 901

**Code:**
```
pub async fn execute(&self) -> Result<(), UveddiError> {
        info!("Starting analysis of: {}", self.path.display());

        // Set up enhanced progress reporting
        let progress_reporter =
...
```

**Recommendation:** Consider breaking down 'execute' into smaller, more focused methods. Current metrics: LOC=166, Statements=159, Complexity=35, Nesting=12

---

#### Long method 'execute' detected: 166 lines, 159 statements, complexity 35

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 901

**Code:**
```
pub async fn execute(&self) -> Result<(), UveddiError> {
        info!("Starting analysis of: {}", self.path.display());

        // Set up enhanced progress reporting
        let progress_reporter =
...
```

**Recommendation:** Consider breaking down 'execute' into smaller, more focused methods. Current metrics: LOC=166, Statements=159, Complexity=35, Nesting=12

---

#### Long method 'display_validation_results' detected: 109 lines, 101 statements, complexity 36

- **File:** `./src/cli/commands/config.rs`
- **Line:** 182

**Code:**
```
fn display_validation_results(
        &self,
        result: &crate::config::validation::ValidationResult,
        show_suggestions: bool,
    ) {
...
```

**Recommendation:** Consider breaking down 'display_validation_results' into smaller, more focused methods. Current metrics: LOC=109, Statements=101, Complexity=36, Nesting=7

---

#### Long method 'display_validation_results' detected: 109 lines, 101 statements, complexity 36

- **File:** `./src/cli/commands/config.rs`
- **Line:** 182

**Code:**
```
fn display_validation_results(
        &self,
        result: &crate::config::validation::ValidationResult,
        show_suggestions: bool,
    ) {
...
```

**Recommendation:** Consider breaking down 'display_validation_results' into smaller, more focused methods. Current metrics: LOC=109, Statements=101, Complexity=36, Nesting=7

---

#### Long method 'register_custom_filters' detected: 162 lines, 199 statements, complexity 41

- **File:** `./src/report/modern_generator.rs`
- **Line:** 767

**Code:**
```
fn register_custom_filters(tera: &mut Tera) {
        // Add custom filters for report formatting
        tera.register_filter(
            "severity_color",
            |value: &tera::Value, _: &HashMap<String, tera::Value>| {
...
```

**Recommendation:** Consider breaking down 'register_custom_filters' into smaller, more focused methods. Current metrics: LOC=162, Statements=199, Complexity=41, Nesting=15

---

#### Long method 'register_custom_filters' detected: 162 lines, 199 statements, complexity 41

- **File:** `./src/report/modern_generator.rs`
- **Line:** 767

**Code:**
```
fn register_custom_filters(tera: &mut Tera) {
        // Add custom filters for report formatting
        tera.register_filter(
            "severity_color",
            |value: &tera::Value, _: &HashMap<String, tera::Value>| {
...
```

**Recommendation:** Consider breaking down 'register_custom_filters' into smaller, more focused methods. Current metrics: LOC=162, Statements=199, Complexity=41, Nesting=15

---

#### Long method 'validate' detected: 99 lines, 99 statements, complexity 26

- **File:** `./src/config/mod.rs`
- **Line:** 324

**Code:**
```
fn validate(&self) -> std::result::Result<(), SecurityError> {
        // Validate Ollama model name if provided
        if let Some(ref model) = self.ollama_model {
            security::validate_model_name(model)?;
        }
...
```

**Recommendation:** Consider breaking down 'validate' into smaller, more focused methods. Current metrics: LOC=99, Statements=99, Complexity=26, Nesting=11

---

#### Long method 'validate' detected: 99 lines, 99 statements, complexity 26

- **File:** `./src/config/mod.rs`
- **Line:** 324

**Code:**
```
fn validate(&self) -> std::result::Result<(), SecurityError> {
        // Validate Ollama model name if provided
        if let Some(ref model) = self.ollama_model {
            security::validate_model_name(model)?;
        }
...
```

**Recommendation:** Consider breaking down 'validate' into smaller, more focused methods. Current metrics: LOC=99, Statements=99, Complexity=26, Nesting=11

---

### 🟠 High (22 issues)

#### Long method 'analyze_rust_interface' detected: 124 lines, 92 statements, complexity 20

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/interface/rust_interface.rs`
- **Line:** 24

**Code:**
```
pub fn analyze_rust_interface(
        &self,
        parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_interface' into smaller, more focused methods. Current metrics: LOC=124, Statements=92, Complexity=20, Nesting=16

---

#### Long method 'analyze_rust_interface' detected: 124 lines, 92 statements, complexity 20

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/interface/rust_interface.rs`
- **Line:** 24

**Code:**
```
pub fn analyze_rust_interface(
        &self,
        parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_interface' into smaller, more focused methods. Current metrics: LOC=124, Statements=92, Complexity=20, Nesting=16

---

#### Long method 'analyze_rust_visibility' detected: 110 lines, 86 statements, complexity 20

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/implementation/visibility_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_rust_visibility(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationVisibilityIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_visibility' into smaller, more focused methods. Current metrics: LOC=110, Statements=86, Complexity=20, Nesting=18

---

#### Long method 'analyze_rust_visibility' detected: 110 lines, 86 statements, complexity 20

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/implementation/visibility_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_rust_visibility(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationVisibilityIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_visibility' into smaller, more focused methods. Current metrics: LOC=110, Statements=86, Complexity=20, Nesting=18

---

#### Long method 'analyze_rust_methods' detected: 97 lines, 77 statements, complexity 17

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/implementation/method_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_rust_methods(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<(Vec<TypeLeakage>, Vec<ImplementationVisibilityIssue>), AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_methods' into smaller, more focused methods. Current metrics: LOC=97, Statements=77, Complexity=17, Nesting=18

---

#### Long method 'analyze_rust_methods' detected: 97 lines, 77 statements, complexity 17

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/implementation/method_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_rust_methods(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<(Vec<TypeLeakage>, Vec<ImplementationVisibilityIssue>), AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_methods' into smaller, more focused methods. Current metrics: LOC=97, Statements=77, Complexity=17, Nesting=18

---

#### Long method 'analyze_generic_literals' detected: 95 lines, 86 statements, complexity 18

- **File:** `./src/analysis/detectors/anti_patterns/magic_values.rs`
- **Line:** 380

**Code:**
```
async fn analyze_generic_literals(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MagicValue>, AnalysisError> {
        let mut magic_values = Vec::new();
...
```

**Recommendation:** Consider breaking down 'analyze_generic_literals' into smaller, more focused methods. Current metrics: LOC=95, Statements=86, Complexity=18, Nesting=16

---

#### Long method 'analyze_generic_literals' detected: 95 lines, 86 statements, complexity 18

- **File:** `./src/analysis/detectors/anti_patterns/magic_values.rs`
- **Line:** 380

**Code:**
```
async fn analyze_generic_literals(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MagicValue>, AnalysisError> {
        let mut magic_values = Vec::new();
...
```

**Recommendation:** Consider breaking down 'analyze_generic_literals' into smaller, more focused methods. Current metrics: LOC=95, Statements=86, Complexity=18, Nesting=16

---

#### Long method 'parse_yarn_lock' detected: 91 lines, 122 statements, complexity 25

- **File:** `./src/analysis/detectors/dependency/language_support/javascript.rs`
- **Line:** 270

**Code:**
```
fn parse_yarn_lock(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read yarn.lock: {}", e)))?;

        let mut dependencies = Vec::new();
...
```

**Recommendation:** Consider breaking down 'parse_yarn_lock' into smaller, more focused methods. Current metrics: LOC=91, Statements=122, Complexity=25, Nesting=15

---

#### Long method 'parse_yarn_lock' detected: 91 lines, 122 statements, complexity 25

- **File:** `./src/analysis/detectors/dependency/language_support/javascript.rs`
- **Line:** 270

**Code:**
```
fn parse_yarn_lock(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read yarn.lock: {}", e)))?;

        let mut dependencies = Vec::new();
...
```

**Recommendation:** Consider breaking down 'parse_yarn_lock' into smaller, more focused methods. Current metrics: LOC=91, Statements=122, Complexity=25, Nesting=15

---

#### Long method 'analyze_data_flow' detected: 95 lines, 101 statements, complexity 19

- **File:** `./src/analysis/detectors/security/owasp/scanners/flow_scanner.rs`
- **Line:** 248

**Code:**
```
fn analyze_data_flow(&self, file: &ParsedFile) -> Result<FlowAnalysisResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut sources_found = 0;
        let mut sinks_found = 0;
...
```

**Recommendation:** Consider breaking down 'analyze_data_flow' into smaller, more focused methods. Current metrics: LOC=95, Statements=101, Complexity=19, Nesting=11

---

#### Long method 'analyze_data_flow' detected: 95 lines, 101 statements, complexity 19

- **File:** `./src/analysis/detectors/security/owasp/scanners/flow_scanner.rs`
- **Line:** 248

**Code:**
```
fn analyze_data_flow(&self, file: &ParsedFile) -> Result<FlowAnalysisResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut sources_found = 0;
        let mut sinks_found = 0;
...
```

**Recommendation:** Consider breaking down 'analyze_data_flow' into smaller, more focused methods. Current metrics: LOC=95, Statements=101, Complexity=19, Nesting=11

---

#### Long method 'check_rust_dependencies' detected: 117 lines, 76 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/language_support/toml/rust_deps.rs`
- **Line:** 25

**Code:**
```
pub fn check_rust_dependencies(
        &self,
        deps: &TomlValue,
        section: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'check_rust_dependencies' into smaller, more focused methods. Current metrics: LOC=117, Statements=76, Complexity=13, Nesting=11

---

#### Long method 'check_rust_dependencies' detected: 117 lines, 76 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/language_support/toml/rust_deps.rs`
- **Line:** 25

**Code:**
```
pub fn check_rust_dependencies(
        &self,
        deps: &TomlValue,
        section: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'check_rust_dependencies' into smaller, more focused methods. Current metrics: LOC=117, Statements=76, Complexity=13, Nesting=11

---

#### Long method 'test_stress_performance' detected: 103 lines, 89 statements, complexity 14

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 468

**Code:**
```
async fn test_stress_performance(&self) -> Result<TestResult, ValidationError> {
        // Test with complex diagrams under load
        let complex_diagram = r#"
graph TD
    subgraph "Frontend Layer"
...
```

**Recommendation:** Consider breaking down 'test_stress_performance' into smaller, more focused methods. Current metrics: LOC=103, Statements=89, Complexity=14, Nesting=10

---

#### Long method 'test_stress_performance' detected: 103 lines, 89 statements, complexity 14

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 468

**Code:**
```
async fn test_stress_performance(&self) -> Result<TestResult, ValidationError> {
        // Test with complex diagrams under load
        let complex_diagram = r#"
graph TD
    subgraph "Frontend Layer"
...
```

**Recommendation:** Consider breaking down 'test_stress_performance' into smaller, more focused methods. Current metrics: LOC=103, Statements=89, Complexity=14, Nesting=10

---

#### God Object detected: 'MarketplacePlugin' has 0 methods and 26 fields. (Thresholds: methods>30, fields>20) LCOM4 score: 1 (>1 indicates low cohesion)

- **File:** `./src/plugins/marketplace.rs`
- **Line:** 25

**Code:**
```
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: String,
    pub tags: Vec<String>,
    pub supported_languages: Vec<String>,
    pub download_url: String,
    pub manifest_url: String,
    pub repository_url: Option<String>,
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
    pub license: String,
    pub downloads: u64,
    pub rating: f64,
    pub reviews: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub minimum_uveddi_version: String,
    pub file_size: u64,
    pub checksum: String,
    pub verified: bool,
    pub featured: bool,
    pub dependencies: Vec<String>,
    pub screenshots: Vec<String>,
}
```

---

#### Long method 'execute' detected: 128 lines, 108 statements, complexity 25

- **File:** `./src/cli/commands/config.rs`
- **Line:** 49

**Code:**
```
pub async fn execute(&self) -> crate::error::Result<()> {
        match &self.command {
            ConfigSubcommand::Show { file } => {
                if let Some(path) = file {
                    let path_str = path.to_str().ok_or_else(|| {
...
```

**Recommendation:** Consider breaking down 'execute' into smaller, more focused methods. Current metrics: LOC=128, Statements=108, Complexity=25, Nesting=10

---

#### Long method 'execute' detected: 128 lines, 108 statements, complexity 25

- **File:** `./src/cli/commands/config.rs`
- **Line:** 49

**Code:**
```
pub async fn execute(&self) -> crate::error::Result<()> {
        match &self.command {
            ConfigSubcommand::Show { file } => {
                if let Some(path) = file {
                    let path_str = path.to_str().ok_or_else(|| {
...
```

**Recommendation:** Consider breaking down 'execute' into smaller, more focused methods. Current metrics: LOC=128, Statements=108, Complexity=25, Nesting=10

---

#### Long method 'check_ollama_models' detected: 109 lines, 77 statements, complexity 18

- **File:** `./src/health/ai.rs`
- **Line:** 134

**Code:**
```
async fn check_ollama_models() -> HealthCheck {
    let ollama_url =
        std::env::var("OLLAMA_API_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let expected_model =
        std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-coder:6.7b".to_string());
...
```

**Recommendation:** Consider breaking down 'check_ollama_models' into smaller, more focused methods. Current metrics: LOC=109, Statements=77, Complexity=18, Nesting=19

---

#### Long method 'validate_dead_code_config' detected: 98 lines, 75 statements, complexity 13

- **File:** `./src/config/validation.rs`
- **Line:** 255

**Code:**
```
fn validate_dead_code_config(
        &self,
        config: &Config,
        _errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
...
```

**Recommendation:** Consider breaking down 'validate_dead_code_config' into smaller, more focused methods. Current metrics: LOC=98, Statements=75, Complexity=13, Nesting=9

---

#### Long method 'validate_dead_code_config' detected: 98 lines, 75 statements, complexity 13

- **File:** `./src/config/validation.rs`
- **Line:** 255

**Code:**
```
fn validate_dead_code_config(
        &self,
        config: &Config,
        _errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
...
```

**Recommendation:** Consider breaking down 'validate_dead_code_config' into smaller, more focused methods. Current metrics: LOC=98, Statements=75, Complexity=13, Nesting=9

---

### 🟡 Medium (37 issues)

#### Long method 'parse_file' detected: 103 lines, 153 statements, complexity 13

- **File:** `./src/ast/tree_sitter_impl.rs`
- **Line:** 552

**Code:**
```
pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        // Security validation: Validate file size, type, and path
        security::validate_file_size(file_path)
            .map_err(|e| AstError::Other(format!("Security validation failed: {}", e)))?;
        security::validate_file_type(file_path)
...
```

**Recommendation:** Consider breaking down 'parse_file' into smaller, more focused methods. Current metrics: LOC=103, Statements=153, Complexity=13, Nesting=10

---

#### Long method 'parse_file' detected: 103 lines, 153 statements, complexity 13

- **File:** `./src/ast/tree_sitter_impl.rs`
- **Line:** 552

**Code:**
```
pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        // Security validation: Validate file size, type, and path
        security::validate_file_size(file_path)
            .map_err(|e| AstError::Other(format!("Security validation failed: {}", e)))?;
        security::validate_file_type(file_path)
...
```

**Recommendation:** Consider breaking down 'parse_file' into smaller, more focused methods. Current metrics: LOC=103, Statements=153, Complexity=13, Nesting=10

---

#### God Object detected: 'DuplicationConfig' has 4 methods and 22 fields. (Thresholds: methods>30, fields>20) LCOM4 score: 1 (>1 indicates low cohesion)

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/config.rs`
- **Line:** 9

**Code:**
```
pub struct DuplicationConfig {
    /// Base detector configuration
    pub base: BaseConfig,

    // Core detection parameters
    /// Minimum number of tokens a code block must have
    pub min_tokens: usize,
    /// Minimum number of lines a code block must have
    pub min_lines: usize,
    /// Similarity threshold for Type-3 (near-miss) clones
    pub similarity_threshold: f64,
    /// Length of token window for rolling hash fingerprints
    pub fingerprint_length: usize,

    // Normalization options
    /// Replace identifiers with placeholders for Type-2 detection
    pub ignore_identifiers: bool,
    /// Replace literals with placeholders for Type-2 detection
    pub ignore_literals: bool,
    /// Ignore whitespace and formatting differences
    pub ignore_whitespace: bool,
    /// Ignore comments when comparing blocks
    pub ignore_comments: bool,

    // Advanced analysis settings
    /// Enable Control Flow Graph analysis for semantic detection
    pub enable_cfg_analysis: bool,
    /// Enable semantic feature extraction for Type-4 detection
    pub enable_semantic_features: bool,
    /// Weight for CFG similarity in overall score
    pub cfg_similarity_weight: f64,
    /// Threshold for semantic similarity (Type-4 detection)
    pub semantic_similarity_threshold: f64,
    /// Number of iterations for Weisfeiler-Lehman kernel
    pub wl_kernel_iterations: usize,
    /// Maximum CFG nodes to process (performance limit)
    pub max_cfg_nodes: usize,

    // Performance settings
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Timeout per file in seconds
    pub timeout_per_file_secs: u64,

    // Filtering options
    /// Minimum clone pair similarity to report
    pub min_clone_similarity: f64,
    /// Maximum number of clone pairs to report
    pub max_clone_pairs: Option<usize>,
    /// Skip files matching these patterns
    pub skip_patterns: Vec<String>,
    /// Only analyze files matching these patterns
    pub include_patterns: Vec<String>,
}
```

---

#### Long method 'analyze_python_interface' detected: 85 lines, 73 statements, complexity 14

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/interface/python_interface.rs`
- **Line:** 24

**Code:**
```
pub fn analyze_python_interface(
        &self,
        parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_python_interface' into smaller, more focused methods. Current metrics: LOC=85, Statements=73, Complexity=14, Nesting=12

---

#### Long method 'analyze_python_interface' detected: 85 lines, 73 statements, complexity 14

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/interface/python_interface.rs`
- **Line:** 24

**Code:**
```
pub fn analyze_python_interface(
        &self,
        parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_python_interface' into smaller, more focused methods. Current metrics: LOC=85, Statements=73, Complexity=14, Nesting=12

---

#### Long method 'analyze_rust_fields' detected: 91 lines, 76 statements, complexity 17

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/implementation/field_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_rust_fields(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<(Vec<ImplementationExposure>, Vec<TypeLeakage>), AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_fields' into smaller, more focused methods. Current metrics: LOC=91, Statements=76, Complexity=17, Nesting=18

---

#### Long method 'analyze_rust_fields' detected: 91 lines, 76 statements, complexity 17

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/implementation/field_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_rust_fields(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<(Vec<ImplementationExposure>, Vec<TypeLeakage>), AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_rust_fields' into smaller, more focused methods. Current metrics: LOC=91, Statements=76, Complexity=17, Nesting=18

---

#### Long method 'analyze_rust_literals' detected: 84 lines, 82 statements, complexity 14

- **File:** `./src/analysis/detectors/anti_patterns/magic_values.rs`
- **Line:** 280

**Code:**
```
async fn analyze_rust_literals(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MagicValue>, AnalysisError> {
        let mut magic_values = Vec::new();
...
```

**Recommendation:** Consider breaking down 'analyze_rust_literals' into smaller, more focused methods. Current metrics: LOC=84, Statements=82, Complexity=14, Nesting=16

---

#### Long method 'analyze_rust_literals' detected: 84 lines, 82 statements, complexity 14

- **File:** `./src/analysis/detectors/anti_patterns/magic_values.rs`
- **Line:** 280

**Code:**
```
async fn analyze_rust_literals(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MagicValue>, AnalysisError> {
        let mut magic_values = Vec::new();
...
```

**Recommendation:** Consider breaking down 'analyze_rust_literals' into smaller, more focused methods. Current metrics: LOC=84, Statements=82, Complexity=14, Nesting=16

---

#### Long method 'parse_package_lock_json' detected: 82 lines, 75 statements, complexity 25

- **File:** `./src/analysis/detectors/dependency/language_support/javascript.rs`
- **Line:** 176

**Code:**
```
fn parse_package_lock_json(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path).map_err(|e| {
            DependencyError::ParseError(format!("Failed to read package-lock.json: {}", e))
        })?;

...
```

**Recommendation:** Consider breaking down 'parse_package_lock_json' into smaller, more focused methods. Current metrics: LOC=82, Statements=75, Complexity=25, Nesting=10

---

#### Long method 'parse_package_lock_json' detected: 82 lines, 75 statements, complexity 25

- **File:** `./src/analysis/detectors/dependency/language_support/javascript.rs`
- **Line:** 176

**Code:**
```
fn parse_package_lock_json(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path).map_err(|e| {
            DependencyError::ParseError(format!("Failed to read package-lock.json: {}", e))
        })?;

...
```

**Recommendation:** Consider breaking down 'parse_package_lock_json' into smaller, more focused methods. Current metrics: LOC=82, Statements=75, Complexity=25, Nesting=10

---

#### Long method 'apply_sanitization' detected: 56 lines, 18 statements, complexity 30

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/sanitizer_detector.rs`
- **Line:** 73

**Code:**
```
pub fn apply_sanitization(
        &self,
        sanitizer: &SanitizationPoint,
        current_taint: TaintLevel,
    ) -> TaintLevel {
...
```

**Recommendation:** Consider breaking down 'apply_sanitization' into smaller, more focused methods. Current metrics: LOC=56, Statements=18, Complexity=30, Nesting=11

---

#### Long method 'apply_sanitization' detected: 56 lines, 18 statements, complexity 30

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/sanitizer_detector.rs`
- **Line:** 73

**Code:**
```
pub fn apply_sanitization(
        &self,
        sanitizer: &SanitizationPoint,
        current_taint: TaintLevel,
    ) -> TaintLevel {
...
```

**Recommendation:** Consider breaking down 'apply_sanitization' into smaller, more focused methods. Current metrics: LOC=56, Statements=18, Complexity=30, Nesting=11

---

#### Long method 'build_known_vulnerability_patterns' detected: 101 lines, 75 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/patterns/vulnerability_patterns/known_vulns.rs`
- **Line:** 10

**Code:**
```
pub fn build_known_vulnerability_patterns(
) -> Result<HashMap<String, VulnerabilityPattern>, AnalysisError> {
    let mut patterns = HashMap::new();

    // Log4j Configuration Vulnerability
...
```

**Recommendation:** Consider breaking down 'build_known_vulnerability_patterns' into smaller, more focused methods. Current metrics: LOC=101, Statements=75, Complexity=1, Nesting=2

---

#### Long method 'check_container_recursive' detected: 88 lines, 89 statements, complexity 18

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/container_patterns.rs`
- **Line:** 22

**Code:**
```
fn check_container_recursive(value: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                // Check for Kubernetes securityContext
                if let Some(security_context) =
...
```

**Recommendation:** Consider breaking down 'check_container_recursive' into smaller, more focused methods. Current metrics: LOC=88, Statements=89, Complexity=18, Nesting=8

---

#### Long method 'check_container_recursive' detected: 88 lines, 89 statements, complexity 18

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/container_patterns.rs`
- **Line:** 22

**Code:**
```
fn check_container_recursive(value: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                // Check for Kubernetes securityContext
                if let Some(security_context) =
...
```

**Recommendation:** Consider breaking down 'check_container_recursive' into smaller, more focused methods. Current metrics: LOC=88, Statements=89, Complexity=18, Nesting=8

---

#### Long method 'generate_summary_report' detected: 101 lines, 97 statements, complexity 7

- **File:** `./src/analysis/performance/baseline.rs`
- **Line:** 658

**Code:**
```
fn generate_summary_report(&self, baseline: &PerformanceBaseline) -> String {
        let mut report = String::new();

        report.push_str("# UV-49 Performance Baseline Analysis Report\n\n");
        report.push_str(&format!("**Generated:** {}\n\n", baseline.timestamp));
...
```

**Recommendation:** Consider breaking down 'generate_summary_report' into smaller, more focused methods. Current metrics: LOC=101, Statements=97, Complexity=7, Nesting=5

---

#### Long method 'generate_summary_report' detected: 101 lines, 97 statements, complexity 7

- **File:** `./src/analysis/performance/baseline.rs`
- **Line:** 658

**Code:**
```
fn generate_summary_report(&self, baseline: &PerformanceBaseline) -> String {
        let mut report = String::new();

        report.push_str("# UV-49 Performance Baseline Analysis Report\n\n");
        report.push_str(&format!("**Generated:** {}\n\n", baseline.timestamp));
...
```

**Recommendation:** Consider breaking down 'generate_summary_report' into smaller, more focused methods. Current metrics: LOC=101, Statements=97, Complexity=7, Nesting=5

---

#### Long method 'test_concurrent_load' detected: 92 lines, 81 statements, complexity 15

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 288

**Code:**
```
async fn test_concurrent_load(&self) -> Result<TestResult, ValidationError> {
        let concurrent_requests = 20;
        let test_diagram = r#"
graph TD
    A[Concurrent] --> B[Load]
...
```

**Recommendation:** Consider breaking down 'test_concurrent_load' into smaller, more focused methods. Current metrics: LOC=92, Statements=81, Complexity=15, Nesting=10

---

#### Long method 'test_concurrent_load' detected: 92 lines, 81 statements, complexity 15

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 288

**Code:**
```
async fn test_concurrent_load(&self) -> Result<TestResult, ValidationError> {
        let concurrent_requests = 20;
        let test_diagram = r#"
graph TD
    A[Concurrent] --> B[Load]
...
```

**Recommendation:** Consider breaking down 'test_concurrent_load' into smaller, more focused methods. Current metrics: LOC=92, Statements=81, Complexity=15, Nesting=10

---

#### Long method 'main' detected: 146 lines, 178 statements, complexity 16

- **File:** `./src/bin/community_demo.rs`
- **Line:** 367

**Code:**
```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Community Database Demo");
    println!("==========================\n");

    // Initialize the community database
...
```

**Recommendation:** Consider breaking down 'main' into smaller, more focused methods. Current metrics: LOC=146, Statements=178, Complexity=16, Nesting=5

---

#### Long method 'main' detected: 97 lines, 106 statements, complexity 7

- **File:** `./src/bin/detector_cache_validator.rs`
- **Line:** 662

**Code:**
```
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let matches = Command::new("detector-cache-validator")
...
```

**Recommendation:** Consider breaking down 'main' into smaller, more focused methods. Current metrics: LOC=97, Statements=106, Complexity=7, Nesting=3

---

#### Long method 'extract_symbols' detected: 88 lines, 83 statements, complexity 19

- **File:** `./src/engine/parsing/parsers/typescript_parser.rs`
- **Line:** 84

**Code:**
```
fn extract_symbols(&self, tree: &Tree, source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root = tree.root_node();

        #[cfg(feature = "tree-sitter")]
...
```

**Recommendation:** Consider breaking down 'extract_symbols' into smaller, more focused methods. Current metrics: LOC=88, Statements=83, Complexity=19, Nesting=11

---

#### Long method 'extract_symbols' detected: 88 lines, 83 statements, complexity 19

- **File:** `./src/engine/parsing/parsers/typescript_parser.rs`
- **Line:** 84

**Code:**
```
fn extract_symbols(&self, tree: &Tree, source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root = tree.root_node();

        #[cfg(feature = "tree-sitter")]
...
```

**Recommendation:** Consider breaking down 'extract_symbols' into smaller, more focused methods. Current metrics: LOC=88, Statements=83, Complexity=19, Nesting=11

---

#### Long method 'extract_symbols' detected: 88 lines, 83 statements, complexity 19

- **File:** `./src/engine/parsing/parsers/rust_parser.rs`
- **Line:** 80

**Code:**
```
fn extract_symbols(&self, tree: &Tree, source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root = tree.root_node();

        #[cfg(feature = "tree-sitter")]
...
```

**Recommendation:** Consider breaking down 'extract_symbols' into smaller, more focused methods. Current metrics: LOC=88, Statements=83, Complexity=19, Nesting=11

---

#### Long method 'extract_symbols' detected: 88 lines, 83 statements, complexity 19

- **File:** `./src/engine/parsing/parsers/rust_parser.rs`
- **Line:** 80

**Code:**
```
fn extract_symbols(&self, tree: &Tree, source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root = tree.root_node();

        #[cfg(feature = "tree-sitter")]
...
```

**Recommendation:** Consider breaking down 'extract_symbols' into smaller, more focused methods. Current metrics: LOC=88, Statements=83, Complexity=19, Nesting=11

---

#### Long method 'enhance_patterns_from_detectors' detected: 175 lines, 111 statements, complexity 6

- **File:** `./src/ai/knowledge/population.rs`
- **Line:** 211

**Code:**
```
fn enhance_patterns_from_detectors(library: &mut KnowledgeLibrary) -> Result<(), PopulationError> {
    debug!("Enhancing patterns with detector-specific knowledge");

    // God Object enhancements from the actual detector
    if let Some(god_object) = library.universal_patterns.get_mut("god_object") {
...
```

**Recommendation:** Consider breaking down 'enhance_patterns_from_detectors' into smaller, more focused methods. Current metrics: LOC=175, Statements=111, Complexity=6, Nesting=3

---

#### Long method 'list_hooks' detected: 86 lines, 79 statements, complexity 18

- **File:** `./src/cli/commands/hooks.rs`
- **Line:** 278

**Code:**
```
async fn list_hooks(&self, repo_path: PathBuf, detailed: bool) -> Result<(), UveddiError> {
        if !crate::hooks::is_git_repository(&repo_path) {
            return Err(UveddiError::config_error(
                &format!("Not a Git repository: {}", repo_path.display()),
                "cli",
...
```

**Recommendation:** Consider breaking down 'list_hooks' into smaller, more focused methods. Current metrics: LOC=86, Statements=79, Complexity=18, Nesting=11

---

#### Long method 'list_hooks' detected: 86 lines, 79 statements, complexity 18

- **File:** `./src/cli/commands/hooks.rs`
- **Line:** 278

**Code:**
```
async fn list_hooks(&self, repo_path: PathBuf, detailed: bool) -> Result<(), UveddiError> {
        if !crate::hooks::is_git_repository(&repo_path) {
            return Err(UveddiError::config_error(
                &format!("Not a Git repository: {}", repo_path.display()),
                "cli",
...
```

**Recommendation:** Consider breaking down 'list_hooks' into smaller, more focused methods. Current metrics: LOC=86, Statements=79, Complexity=18, Nesting=11

---

#### Long method 'verify_plugin' detected: 115 lines, 99 statements, complexity 9

- **File:** `./src/cli/commands/plugin.rs`
- **Line:** 325

**Code:**
```
async fn verify_plugin(
        &self,
        _binary_path: PathBuf,
        _manifest_path: PathBuf,
    ) -> Result<(), crate::error::UveddiError> {
...
```

**Recommendation:** Consider breaking down 'verify_plugin' into smaller, more focused methods. Current metrics: LOC=115, Statements=99, Complexity=9, Nesting=9

---

#### Long method 'verify_plugin' detected: 115 lines, 99 statements, complexity 9

- **File:** `./src/cli/commands/plugin.rs`
- **Line:** 325

**Code:**
```
async fn verify_plugin(
        &self,
        _binary_path: PathBuf,
        _manifest_path: PathBuf,
    ) -> Result<(), crate::error::UveddiError> {
...
```

**Recommendation:** Consider breaking down 'verify_plugin' into smaller, more focused methods. Current metrics: LOC=115, Statements=99, Complexity=9, Nesting=9

---

#### Long method 'generate_severity_summary' detected: 96 lines, 96 statements, complexity 13

- **File:** `./src/report/executive_summary.rs`
- **Line:** 79

**Code:**
```
pub fn generate_severity_summary(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut summary = String::new();

        // Group issues by severity
        let high_issues: Vec<&ArchitecturalIssue> = issues
...
```

**Recommendation:** Consider breaking down 'generate_severity_summary' into smaller, more focused methods. Current metrics: LOC=96, Statements=96, Complexity=13, Nesting=8

---

#### Long method 'generate_severity_summary' detected: 96 lines, 96 statements, complexity 13

- **File:** `./src/report/executive_summary.rs`
- **Line:** 79

**Code:**
```
pub fn generate_severity_summary(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut summary = String::new();

        // Group issues by severity
        let high_issues: Vec<&ArchitecturalIssue> = issues
...
```

**Recommendation:** Consider breaking down 'generate_severity_summary' into smaller, more focused methods. Current metrics: LOC=96, Statements=96, Complexity=13, Nesting=8

---

#### Long method 'render_batch' detected: 97 lines, 83 statements, complexity 21

- **File:** `./src/report/image_renderer.rs`
- **Line:** 343

**Code:**
```
pub async fn render_batch(
        &self,
        diagrams: Vec<(String, Option<(u32, u32)>)>, // (mermaid_code, dimensions)
        format: ImageFormat,
    ) -> Result<Vec<Result<RenderedImage, String>>, RenderingServiceError> {
...
```

**Recommendation:** Consider breaking down 'render_batch' into smaller, more focused methods. Current metrics: LOC=97, Statements=83, Complexity=21, Nesting=13

---

#### Long method 'render_batch' detected: 97 lines, 83 statements, complexity 21

- **File:** `./src/report/image_renderer.rs`
- **Line:** 343

**Code:**
```
pub async fn render_batch(
        &self,
        diagrams: Vec<(String, Option<(u32, u32)>)>, // (mermaid_code, dimensions)
        format: ImageFormat,
    ) -> Result<Vec<Result<RenderedImage, String>>, RenderingServiceError> {
...
```

**Recommendation:** Consider breaking down 'render_batch' into smaller, more focused methods. Current metrics: LOC=97, Statements=83, Complexity=21, Nesting=13

---

#### God Object detected: 'LegacyAnalysisConfig' has 0 methods and 24 fields. (Thresholds: methods>30, fields>20) LCOM4 score: 1 (>1 indicates low cohesion)

- **File:** `./src/application/mod_verbose.rs`
- **Line:** 42

**Code:**
```
pub struct LegacyAnalysisConfig {
    pub target_path: PathBuf,
    pub output_format: String,
    pub output_file: Option<PathBuf>,
    pub enable_ai: bool,
    pub ollama_api_url: Option<String>,
    pub ollama_model: Option<String>,
    pub dead_code_confidence: Option<f64>,
    pub dead_code_library_mode: bool,
    pub dead_code_ignore_patterns: Option<Vec<String>>,
    pub dead_code_keep_alive: Option<Vec<String>>,
    pub large_classes_max_loc: Option<u32>,
    pub large_classes_max_methods: Option<u32>,
    pub large_classes_max_fields: Option<u32>,
    pub large_classes_max_complexity: Option<u32>,
    pub large_classes_max_lcom: Option<f64>,
    pub large_classes_ignore_patterns: Option<Vec<String>>,
    pub large_classes_min_severity: Option<u32>,
    pub enable_memory_optimization: bool,
    pub memory_limit_gb: Option<f64>,
    pub memory_profile: Option<String>,
    pub timeout_seconds: u64,
    pub enable_resource_management: bool,
    pub resource_config: Option<crate::resource_management::ResourceConfig>,

    #[cfg(feature = "memory-optimization")]
    pub memory_optimization: Option<crate::analysis::memory::MemoryOptimizationConfig>,
}
```

---

#### Long method 'run_app' detected: 134 lines, 103 statements, complexity 16

- **File:** `./src/application/mod_verbose.rs`
- **Line:** 220

**Code:**
```
pub async fn run_app() -> Result<(), UveddiError> {
    warn!("Using deprecated run_app function. Consider migrating to the new modular API.");

    // Import the main CLI functionality
    use crate::cli::{
...
```

**Recommendation:** Consider breaking down 'run_app' into smaller, more focused methods. Current metrics: LOC=134, Statements=103, Complexity=16, Nesting=9

---

### ⚪ Low (347 issues)

#### Large class 'RepositoryManager' detected: 0 LOC, 11 methods, 4 fields

- **File:** `./src/database/repositories/factory.rs`
- **Line:** 66

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteIssueRepository' detected: 0 LOC, 11 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/issue_repository.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteProjectRepository' detected: 0 LOC, 13 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/project_repository.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteCacheRepository' detected: 0 LOC, 13 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/cache_repository.rs`
- **Line:** 13

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteDebtRepository' detected: 0 LOC, 11 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/debt_repository.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteDependencyRepository' detected: 0 LOC, 12 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/dependency_repository.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteAnalysisRepository' detected: 0 LOC, 14 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/analysis_repository.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteSecurityRepository' detected: 0 LOC, 11 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/security_repository.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteMetricsRepository' detected: 0 LOC, 11 methods, 1 fields

- **File:** `./src/database/repositories/sqlite/metrics_repository.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'MigrationManager' detected: 0 LOC, 14 methods, 3 fields

- **File:** `./src/database/migration_manager.rs`
- **Line:** 21

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'start_monitoring' detected: 62 lines, 78 statements, complexity 11

- **File:** `./src/database/monitoring.rs`
- **Line:** 41

**Code:**
```
pub fn start_monitoring(&self) -> MonitoringHandle {
        let database = self.database.clone();
        let config = self.config.clone();
        let metrics_history = self.metrics_history.clone();
        let alert_sender = self.alert_sender.clone();
...
```

**Recommendation:** Consider breaking down 'start_monitoring' into smaller, more focused methods. Current metrics: LOC=62, Statements=78, Complexity=11, Nesting=7

---

#### Long method 'start_monitoring' detected: 62 lines, 78 statements, complexity 11

- **File:** `./src/database/monitoring.rs`
- **Line:** 41

**Code:**
```
pub fn start_monitoring(&self) -> MonitoringHandle {
        let database = self.database.clone();
        let config = self.config.clone();
        let metrics_history = self.metrics_history.clone();
        let alert_sender = self.alert_sender.clone();
...
```

**Recommendation:** Consider breaking down 'start_monitoring' into smaller, more focused methods. Current metrics: LOC=62, Statements=78, Complexity=11, Nesting=7

---

#### Large class 'ScalableDatabase' detected: 0 LOC, 29 methods, 5 fields

- **File:** `./src/database/scalable_manager.rs`
- **Line:** 24

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DatabaseConfigManager' detected: 0 LOC, 15 methods, 2 fields

- **File:** `./src/database/config_manager.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DatabaseQueryOptimizer' detected: 0 LOC, 16 methods, 10 fields

- **File:** `./src/database/performance/query_optimizer.rs`
- **Line:** 195

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PostgreSqlProvider' detected: 0 LOC, 26 methods, 4 fields

- **File:** `./src/database/connection/providers/postgresql_provider.rs`
- **Line:** 48

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SqliteProvider' detected: 0 LOC, 26 methods, 4 fields

- **File:** `./src/database/connection/providers/sqlite_provider.rs`
- **Line:** 49

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ArchitecturalIssue' detected: 0 LOC, 2 methods, 16 fields

- **File:** `./src/database/models/architectural_issue.rs`
- **Line:** 27

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ParsedFileCompat' detected: 0 LOC, 11 methods, 6 fields

- **File:** `./src/ast/compatibility_shim.rs`
- **Line:** 34

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AstParser' detected: 0 LOC, 22 methods, 5 fields

- **File:** `./src/ast/tree_sitter_impl.rs`
- **Line:** 77

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'LocalDependencyGraph' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/analysis/graph/dependency.rs`
- **Line:** 54

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'MermaidGenerator' detected: 0 LOC, 21 methods, 4 fields

- **File:** `./src/analysis/mermaid_generator.rs`
- **Line:** 36

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'register_builtin_templates' detected: 153 lines, 46 statements, complexity 1

- **File:** `./src/analysis/mermaid_generator.rs`
- **Line:** 516

**Code:**
```
fn register_builtin_templates(tera: &mut Tera) -> Result<(), MermaidGenerationError> {
        // Component diagram template
        tera.add_raw_template(
            "component_diagram",
            r#"graph {{ layout }}
...
```

**Recommendation:** Consider breaking down 'register_builtin_templates' into smaller, more focused methods. Current metrics: LOC=153, Statements=46, Complexity=1, Nesting=3

---

#### Long method 'register_builtin_templates' detected: 153 lines, 46 statements, complexity 1

- **File:** `./src/analysis/mermaid_generator.rs`
- **Line:** 516

**Code:**
```
fn register_builtin_templates(tera: &mut Tera) -> Result<(), MermaidGenerationError> {
        // Component diagram template
        tera.add_raw_template(
            "component_diagram",
            r#"graph {{ layout }}
...
```

**Recommendation:** Consider breaking down 'register_builtin_templates' into smaller, more focused methods. Current metrics: LOC=153, Statements=46, Complexity=1, Nesting=3

---

#### Large class 'AnalysisEngine' detected: 0 LOC, 38 methods, 13 fields

- **File:** `./src/analysis/engine.rs`
- **Line:** 67

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SemanticAnalyzer' detected: 0 LOC, 17 methods, 2 fields

- **File:** `./src/analysis/semantic/mod.rs`
- **Line:** 146

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PerformanceAnalysisService' detected: 0 LOC, 14 methods, 4 fields

- **File:** `./src/analysis/services/performance_service.rs`
- **Line:** 207

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DependencyAnalysisService' detected: 0 LOC, 18 methods, 6 fields

- **File:** `./src/analysis/services/dependency_service.rs`
- **Line:** 53

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AnalysisService' detected: 0 LOC, 14 methods, 8 fields

- **File:** `./src/analysis/services/analysis_service.rs`
- **Line:** 45

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AnalysisAggregator' detected: 0 LOC, 15 methods, 2 fields

- **File:** `./src/analysis/components/analysis_aggregator.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'CacheManagerImpl' detected: 0 LOC, 11 methods, 4 fields

- **File:** `./src/analysis/components/cache_manager.rs`
- **Line:** 58

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DependencyGraphBuilderImpl' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/analysis/components/dependency_graph_builder.rs`
- **Line:** 20

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DetectorScheduler' detected: 0 LOC, 13 methods, 6 fields

- **File:** `./src/analysis/components/detector_scheduler.rs`
- **Line:** 27

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PluginManager' detected: 0 LOC, 13 methods, 4 fields

- **File:** `./src/analysis/components/plugin_manager.rs`
- **Line:** 36

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ConfigurationService' detected: 0 LOC, 15 methods, 5 fields

- **File:** `./src/analysis/components/config_service.rs`
- **Line:** 10

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'WorkspaceDetector' detected: 0 LOC, 27 methods, 0 fields

- **File:** `./src/analysis/workspace.rs`
- **Line:** 117

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'create_loose_file_workspace' detected: 83 lines, 77 statements, complexity 22

- **File:** `./src/analysis/workspace.rs`
- **Line:** 324

**Code:**
```
async fn create_loose_file_workspace(
        start_path: &Path,
    ) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        // Use the file discovery to check if there are any supported source files
        let discovered_files =
...
```

**Recommendation:** Consider breaking down 'create_loose_file_workspace' into smaller, more focused methods. Current metrics: LOC=83, Statements=77, Complexity=22, Nesting=12

---

#### Long method 'create_loose_file_workspace' detected: 83 lines, 77 statements, complexity 22

- **File:** `./src/analysis/workspace.rs`
- **Line:** 324

**Code:**
```
async fn create_loose_file_workspace(
        start_path: &Path,
    ) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        // Use the file discovery to check if there are any supported source files
        let discovered_files =
...
```

**Recommendation:** Consider breaking down 'create_loose_file_workspace' into smaller, more focused methods. Current metrics: LOC=83, Statements=77, Complexity=22, Nesting=12

---

#### Long method 'analyze_python_project' detected: 56 lines, 68 statements, complexity 9

- **File:** `./src/analysis/workspace.rs`
- **Line:** 727

**Code:**
```
async fn analyze_python_project(project_root: &Path) -> Result<WorkspaceInfo, AnalysisError> {
        let mut manifest_paths = Vec::new();
        let mut dependencies = Vec::new();
        let mut project_name = project_root
            .file_name()
...
```

**Recommendation:** Consider breaking down 'analyze_python_project' into smaller, more focused methods. Current metrics: LOC=56, Statements=68, Complexity=9, Nesting=11

---

#### Long method 'analyze_python_project' detected: 56 lines, 68 statements, complexity 9

- **File:** `./src/analysis/workspace.rs`
- **Line:** 727

**Code:**
```
async fn analyze_python_project(project_root: &Path) -> Result<WorkspaceInfo, AnalysisError> {
        let mut manifest_paths = Vec::new();
        let mut dependencies = Vec::new();
        let mut project_name = project_root
            .file_name()
...
```

**Recommendation:** Consider breaking down 'analyze_python_project' into smaller, more focused methods. Current metrics: LOC=56, Statements=68, Complexity=9, Nesting=11

---

#### Large class 'DetectorRegistry' detected: 0 LOC, 14 methods, 3 fields

- **File:** `./src/analysis/detector_registry.rs`
- **Line:** 49

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'parse_file_robust' detected: 84 lines, 59 statements, complexity 19

- **File:** `./src/analysis/robust_parser.rs`
- **Line:** 108

**Code:**
```
pub async fn parse_file_robust(
        &self,
        file_path: std::path::PathBuf,
        timeout_duration: Option<Duration>,
    ) -> Result<crate::ast::ParsedFile, ParseFileError> {
...
```

**Recommendation:** Consider breaking down 'parse_file_robust' into smaller, more focused methods. Current metrics: LOC=84, Statements=59, Complexity=19, Nesting=16

---

#### Long method 'parse_file_robust' detected: 84 lines, 59 statements, complexity 19

- **File:** `./src/analysis/robust_parser.rs`
- **Line:** 108

**Code:**
```
pub async fn parse_file_robust(
        &self,
        file_path: std::path::PathBuf,
        timeout_duration: Option<Duration>,
    ) -> Result<crate::ast::ParsedFile, ParseFileError> {
...
```

**Recommendation:** Consider breaking down 'parse_file_robust' into smaller, more focused methods. Current metrics: LOC=84, Statements=59, Complexity=19, Nesting=16

---

#### Large class 'StandardConfigBuilder' detected: 0 LOC, 17 methods, 2 fields

- **File:** `./src/analysis/standardized_config.rs`
- **Line:** 255

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'default_for_detector' detected: 89 lines, 106 statements, complexity 8

- **File:** `./src/analysis/standardized_config.rs`
- **Line:** 458

**Code:**
```
pub fn default_for_detector(detector_name: &str) -> Self {
        let mut config = Self::default();

        // Set detector-specific defaults
        match detector_name {
...
```

**Recommendation:** Consider breaking down 'default_for_detector' into smaller, more focused methods. Current metrics: LOC=89, Statements=106, Complexity=8, Nesting=4

---

#### Long method 'default_for_detector' detected: 89 lines, 106 statements, complexity 8

- **File:** `./src/analysis/standardized_config.rs`
- **Line:** 458

**Code:**
```
pub fn default_for_detector(detector_name: &str) -> Self {
        let mut config = Self::default();

        // Set detector-specific defaults
        match detector_name {
...
```

**Recommendation:** Consider breaking down 'default_for_detector' into smaller, more focused methods. Current metrics: LOC=89, Statements=106, Complexity=8, Nesting=4

---

#### Large class 'DetectorRegistry' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/registry/mod.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'JavaScriptAnalyzer' detected: 0 LOC, 11 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/dead_code/language_support/javascript.rs`
- **Line:** 15

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'extract_symbols' detected: 65 lines, 80 statements, complexity 7

- **File:** `./src/analysis/detectors/anti_patterns/dead_code/language_support/python.rs`
- **Line:** 36

**Code:**
```
fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
...
```

**Recommendation:** Consider breaking down 'extract_symbols' into smaller, more focused methods. Current metrics: LOC=65, Statements=80, Complexity=7, Nesting=8

---

#### Long method 'extract_symbols' detected: 65 lines, 80 statements, complexity 7

- **File:** `./src/analysis/detectors/anti_patterns/dead_code/language_support/python.rs`
- **Line:** 36

**Code:**
```
fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
...
```

**Recommendation:** Consider breaking down 'extract_symbols' into smaller, more focused methods. Current metrics: LOC=65, Statements=80, Complexity=7, Nesting=8

---

#### Large class 'ContextDeadCodeDetector' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/analysis/detectors/anti_patterns/dead_code/context_detector.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'analyze_derive_macros' detected: 38 lines, 52 statements, complexity 9

- **File:** `./src/analysis/detectors/anti_patterns/god_object/languages/rust/patterns.rs`
- **Line:** 58

**Code:**
```
pub fn analyze_derive_macros(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashMap<String, Vec<String>>, AnalysisError> {
        let mut derive_attributes: HashMap<String, Vec<String>> = HashMap::new();
...
```

**Recommendation:** Consider breaking down 'analyze_derive_macros' into smaller, more focused methods. Current metrics: LOC=38, Statements=52, Complexity=9, Nesting=17

---

#### Long method 'analyze_derive_macros' detected: 38 lines, 52 statements, complexity 9

- **File:** `./src/analysis/detectors/anti_patterns/god_object/languages/rust/patterns.rs`
- **Line:** 58

**Code:**
```
pub fn analyze_derive_macros(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashMap<String, Vec<String>>, AnalysisError> {
        let mut derive_attributes: HashMap<String, Vec<String>> = HashMap::new();
...
```

**Recommendation:** Consider breaking down 'analyze_derive_macros' into smaller, more focused methods. Current metrics: LOC=38, Statements=52, Complexity=9, Nesting=17

---

#### Large class 'GodObjectDetector' detected: 0 LOC, 11 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/god_object/mod.rs`
- **Line:** 75

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'GodObjectConfig' detected: 0 LOC, 13 methods, 8 fields

- **File:** `./src/analysis/detectors/anti_patterns/god_object/config.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'CouplingCalculator' detected: 0 LOC, 13 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/tight_coupling/analysis/coupling_calculator.rs`
- **Line:** 9

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'calculate_abstractness' detected: 36 lines, 29 statements, complexity 21

- **File:** `./src/analysis/detectors/anti_patterns/tight_coupling/metrics/instability_calculator.rs`
- **Line:** 101

**Code:**
```
pub fn calculate_abstractness(&self, component: &ComponentNode) -> f64 {
        match component {
            ComponentNode::Class { name, .. } => {
                // Heuristics based on naming conventions and patterns
                if name.contains("Abstract") || name.contains("Base") {
...
```

**Recommendation:** Consider breaking down 'calculate_abstractness' into smaller, more focused methods. Current metrics: LOC=36, Statements=29, Complexity=21, Nesting=15

---

#### Long method 'calculate_abstractness' detected: 36 lines, 29 statements, complexity 21

- **File:** `./src/analysis/detectors/anti_patterns/tight_coupling/metrics/instability_calculator.rs`
- **Line:** 101

**Code:**
```
pub fn calculate_abstractness(&self, component: &ComponentNode) -> f64 {
        match component {
            ComponentNode::Class { name, .. } => {
                // Heuristics based on naming conventions and patterns
                if name.contains("Abstract") || name.contains("Base") {
...
```

**Recommendation:** Consider breaking down 'calculate_abstractness' into smaller, more focused methods. Current metrics: LOC=36, Statements=29, Complexity=21, Nesting=15

---

#### Large class 'TightCouplingDetector' detected: 0 LOC, 16 methods, 5 fields

- **File:** `./src/analysis/detectors/anti_patterns/tight_coupling/detector.rs`
- **Line:** 21

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'GraphBuilder' detected: 0 LOC, 12 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/tight_coupling/visualization/graph_builder.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DataClumpsDetector' detected: 0 LOC, 18 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/data_clumps.rs`
- **Line:** 66

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'TypeScriptLanguageSupport' detected: 0 LOC, 13 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/language_support/typescript.rs`
- **Line:** 9

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'RustLanguageSupport' detected: 0 LOC, 11 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/language_support/rust.rs`
- **Line:** 9

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PythonLanguageSupport' detected: 0 LOC, 11 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/language_support/python.rs`
- **Line:** 9

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SimilarityCalculator' detected: 0 LOC, 12 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/metrics/similarity_calculator.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ThresholdManager' detected: 0 LOC, 12 methods, 3 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/metrics/threshold_manager.rs`
- **Line:** 9

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ContextCodeDuplicationDetector' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/context_detector.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'CodeDuplicationDetector' detected: 0 LOC, 21 methods, 8 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/detector.rs`
- **Line:** 26

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'TokenBasedDetector' detected: 0 LOC, 15 methods, 4 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/algorithms/token_based.rs`
- **Line:** 13

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AstBasedDetector' detected: 0 LOC, 11 methods, 6 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/algorithms/ast_based.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SemanticDetector' detected: 0 LOC, 18 methods, 4 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/algorithms/semantic_based.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DuplicationConfig' detected: 0 LOC, 15 methods, 22 fields

- **File:** `./src/analysis/detectors/anti_patterns/code_duplication/config.rs`
- **Line:** 9

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ShotgunSurgeryDetector' detected: 0 LOC, 15 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/shotgun_surgery.rs`
- **Line:** 46

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'FeatureEnvyDetector' detected: 0 LOC, 15 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/feature_envy.rs`
- **Line:** 57

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'TightCouplingPattern' detected: 0 LOC, 13 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/patterns/tight_coupling_patterns.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DataStructureLeaksPattern' detected: 0 LOC, 16 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/patterns/data_structure_leaks.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ExposedInternalsPattern' detected: 0 LOC, 11 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/patterns/exposed_internals.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'StateExposurePattern' detected: 0 LOC, 14 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/patterns/state_exposure.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'EncapsulationChecker' detected: 0 LOC, 12 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/validation/encapsulation_checker.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'BoundaryValidator' detected: 0 LOC, 15 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/validation/boundary_validator.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AbstractionScorer' detected: 0 LOC, 14 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/validation/abstraction_scorer.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ClassAnalyzer' detected: 0 LOC, 13 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/language_support/typescript/class_analyzer.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'InterfaceParser' detected: 0 LOC, 11 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/language_support/typescript/interface_parser.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'analyze_interface_violations' detected: 82 lines, 56 statements, complexity 11

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/language_support/typescript/interface_parser.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_interface_violations(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_interface_violations' into smaller, more focused methods. Current metrics: LOC=82, Statements=56, Complexity=11, Nesting=12

---

#### Long method 'analyze_interface_violations' detected: 82 lines, 56 statements, complexity 11

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/language_support/typescript/interface_parser.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_interface_violations(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_interface_violations' into smaller, more focused methods. Current metrics: LOC=82, Statements=56, Complexity=11, Nesting=12

---

#### Long method 'analyze_import_violations' detected: 80 lines, 61 statements, complexity 12

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/language_support/typescript/module_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_import_violations(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_import_violations' into smaller, more focused methods. Current metrics: LOC=80, Statements=61, Complexity=12, Nesting=12

---

#### Long method 'analyze_import_violations' detected: 80 lines, 61 statements, complexity 12

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/language_support/typescript/module_analyzer.rs`
- **Line:** 25

**Code:**
```
pub fn analyze_import_violations(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
...
```

**Recommendation:** Consider breaking down 'analyze_import_violations' into smaller, more focused methods. Current metrics: LOC=80, Statements=61, Complexity=12, Nesting=12

---

#### Large class 'LeakyAbstractionDetector' detected: 0 LOC, 18 methods, 5 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/detector.rs`
- **Line:** 21

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'VisibilityAnalyzer' detected: 0 LOC, 13 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/implementation/visibility_analyzer.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'LayerValidator' detected: 0 LOC, 14 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/validation/layer_validator.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'LeakDetector' detected: 0 LOC, 21 methods, 3 fields

- **File:** `./src/analysis/detectors/anti_patterns/leaky_abstraction/analyzers/leak_detector.rs`
- **Line:** 17

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'MagicValuesDetector' detected: 0 LOC, 15 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/magic_values.rs`
- **Line:** 110

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PythonMethodAnalyzer' detected: 0 LOC, 11 methods, 0 fields

- **File:** `./src/analysis/detectors/anti_patterns/long_methods/language_support/python.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'LongMethodsDetector' detected: 0 LOC, 15 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/long_methods/detector.rs`
- **Line:** 21

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'LargeClassDetector' detected: 0 LOC, 23 methods, 1 fields

- **File:** `./src/analysis/detectors/anti_patterns/large_classes.rs`
- **Line:** 129

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'find_rust_impl_metrics' detected: 41 lines, 42 statements, complexity 8

- **File:** `./src/analysis/detectors/anti_patterns/large_classes.rs`
- **Line:** 294

**Code:**
```
fn find_rust_impl_metrics(
        &self,
        struct_name: &str,
        tree: &Tree,
        source: &[u8],
...
```

**Recommendation:** Consider breaking down 'find_rust_impl_metrics' into smaller, more focused methods. Current metrics: LOC=41, Statements=42, Complexity=8, Nesting=15

---

#### Long method 'find_rust_impl_metrics' detected: 41 lines, 42 statements, complexity 8

- **File:** `./src/analysis/detectors/anti_patterns/large_classes.rs`
- **Line:** 294

**Code:**
```
fn find_rust_impl_metrics(
        &self,
        struct_name: &str,
        tree: &Tree,
        source: &[u8],
...
```

**Recommendation:** Consider breaking down 'find_rust_impl_metrics' into smaller, more focused methods. Current metrics: LOC=41, Statements=42, Complexity=8, Nesting=15

---

#### Large class 'AdvisoryScanner' detected: 0 LOC, 11 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/vulnerabilities/advisory_scanner.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SupplyChainAnalyzer' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/analysis/detectors/dependency/vulnerabilities/supply_chain.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'IntegrityChecker' detected: 0 LOC, 15 methods, 2 fields

- **File:** `./src/analysis/detectors/dependency/vulnerabilities/integrity_checker.rs`
- **Line:** 9

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'MalwareScanner' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/dependency/vulnerabilities/malware_scanner.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ConflictDetector' detected: 0 LOC, 17 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/licenses/conflict_detector.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'LicenseAnalyzer' detected: 0 LOC, 13 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/licenses/license_analyzer.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ComplianceChecker' detected: 0 LOC, 12 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/licenses/compliance_checker.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PolicyEnforcer' detected: 0 LOC, 16 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/licenses/policy_enforcer.rs`
- **Line:** 8

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'JavaScriptDependencyParser' detected: 0 LOC, 12 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/language_support/javascript.rs`
- **Line:** 47

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PythonDependencyParser' detected: 0 LOC, 12 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/language_support/python.rs`
- **Line:** 55

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'parse_pyproject_toml' detected: 58 lines, 52 statements, complexity 14

- **File:** `./src/analysis/detectors/dependency/language_support/python.rs`
- **Line:** 243

**Code:**
```
fn parse_pyproject_toml(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path).map_err(|e| {
            DependencyError::ParseError(format!("Failed to read pyproject.toml: {}", e))
        })?;

...
```

**Recommendation:** Consider breaking down 'parse_pyproject_toml' into smaller, more focused methods. Current metrics: LOC=58, Statements=52, Complexity=14, Nesting=11

---

#### Long method 'parse_pyproject_toml' detected: 58 lines, 52 statements, complexity 14

- **File:** `./src/analysis/detectors/dependency/language_support/python.rs`
- **Line:** 243

**Code:**
```
fn parse_pyproject_toml(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path).map_err(|e| {
            DependencyError::ParseError(format!("Failed to read pyproject.toml: {}", e))
        })?;

...
```

**Recommendation:** Consider breaking down 'parse_pyproject_toml' into smaller, more focused methods. Current metrics: LOC=58, Statements=52, Complexity=14, Nesting=11

---

#### Long method 'parse_pipfile' detected: 72 lines, 57 statements, complexity 19

- **File:** `./src/analysis/detectors/dependency/language_support/python.rs`
- **Line:** 309

**Code:**
```
fn parse_pipfile(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read Pipfile: {}", e)))?;

        let pipfile: PipfileTom = toml::from_str(&content)
...
```

**Recommendation:** Consider breaking down 'parse_pipfile' into smaller, more focused methods. Current metrics: LOC=72, Statements=57, Complexity=19, Nesting=13

---

#### Long method 'parse_pipfile' detected: 72 lines, 57 statements, complexity 19

- **File:** `./src/analysis/detectors/dependency/language_support/python.rs`
- **Line:** 309

**Code:**
```
fn parse_pipfile(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read Pipfile: {}", e)))?;

        let pipfile: PipfileTom = toml::from_str(&content)
...
```

**Recommendation:** Consider breaking down 'parse_pipfile' into smaller, more focused methods. Current metrics: LOC=72, Statements=57, Complexity=19, Nesting=13

---

#### Large class 'DependencyDetector' detected: 0 LOC, 12 methods, 6 fields

- **File:** `./src/analysis/detectors/dependency/detector.rs`
- **Line:** 40

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'OutdatedDependencyChecker' detected: 0 LOC, 14 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/analyzers/outdated_deps.rs`
- **Line:** 41

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'TransitiveAnalyzer' detected: 0 LOC, 12 methods, 2 fields

- **File:** `./src/analysis/detectors/dependency/analyzers/transitive_analyzer.rs`
- **Line:** 48

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DependencyGraphAnalyzer' detected: 0 LOC, 16 methods, 2 fields

- **File:** `./src/analysis/detectors/dependency/analyzers/dependency_graph.rs`
- **Line:** 34

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'CircularDependencyDetector' detected: 0 LOC, 14 methods, 4 fields

- **File:** `./src/analysis/detectors/dependency/analyzers/circular_deps.rs`
- **Line:** 15

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'VersionAnalyzer' detected: 0 LOC, 14 methods, 1 fields

- **File:** `./src/analysis/detectors/dependency/analyzers/version_analyzer.rs`
- **Line:** 69

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'find_entities_within_radius' detected: 41 lines, 48 statements, complexity 11

- **File:** `./src/analysis/detectors/security/knowledge_graph/graph/traversal.rs`
- **Line:** 77

**Code:**
```
pub fn find_entities_within_radius(
        &self,
        start_id: &str,
        radius: usize,
        entity_filter: Option<&dyn Fn(&CodeEntity) -> bool>,
...
```

**Recommendation:** Consider breaking down 'find_entities_within_radius' into smaller, more focused methods. Current metrics: LOC=41, Statements=48, Complexity=11, Nesting=13

---

#### Long method 'find_entities_within_radius' detected: 41 lines, 48 statements, complexity 11

- **File:** `./src/analysis/detectors/security/knowledge_graph/graph/traversal.rs`
- **Line:** 77

**Code:**
```
pub fn find_entities_within_radius(
        &self,
        start_id: &str,
        radius: usize,
        entity_filter: Option<&dyn Fn(&CodeEntity) -> bool>,
...
```

**Recommendation:** Consider breaking down 'find_entities_within_radius' into smaller, more focused methods. Current metrics: LOC=41, Statements=48, Complexity=11, Nesting=13

---

#### Large class 'PythonTaintAnalyzer' detected: 0 LOC, 17 methods, 3 fields

- **File:** `./src/analysis/detectors/security/taint_analysis/language_support/python/analyzer.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'get_python_sources' detected: 94 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/taint_analysis/language_support/python/patterns.rs`
- **Line:** 10

**Code:**
```
pub fn get_python_sources() -> Vec<TaintSource> {
    vec![
        // Command line and environment
        TaintSource::new(
            "python_sys_argv".to_string(),
...
```

**Recommendation:** Consider breaking down 'get_python_sources' into smaller, more focused methods. Current metrics: LOC=94, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'get_python_sinks' detected: 116 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/taint_analysis/language_support/python/patterns.rs`
- **Line:** 112

**Code:**
```
pub fn get_python_sinks() -> Vec<TaintSink> {
    vec![
        // Code execution
        TaintSink::new(
            "python_eval".to_string(),
...
```

**Recommendation:** Consider breaking down 'get_python_sinks' into smaller, more focused methods. Current metrics: LOC=116, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'get_typescript_sources' detected: 125 lines, 2 statements, complexity 1

- **File:** `./src/analysis/detectors/security/taint_analysis/language_support/typescript/patterns/sources.rs`
- **Line:** 7

**Code:**
```
pub fn get_typescript_sources() -> Vec<TaintSource> {
    let language = SourceLanguage::TypeScript; // Used for both TS and JS

    vec![
        // Command line and environment (Node.js)
...
```

**Recommendation:** Consider breaking down 'get_typescript_sources' into smaller, more focused methods. Current metrics: LOC=125, Statements=2, Complexity=1, Nesting=1

---

#### Long method 'get_typescript_sinks' detected: 152 lines, 2 statements, complexity 1

- **File:** `./src/analysis/detectors/security/taint_analysis/language_support/typescript/patterns/sinks.rs`
- **Line:** 8

**Code:**
```
pub fn get_typescript_sinks() -> Vec<TaintSink> {
    let language = SourceLanguage::TypeScript;

    vec![
        // DOM manipulation (XSS)
...
```

**Recommendation:** Consider breaking down 'get_typescript_sinks' into smaller, more focused methods. Current metrics: LOC=152, Statements=2, Complexity=1, Nesting=1

---

#### Large class 'TypeScriptTaintAnalyzer' detected: 0 LOC, 17 methods, 3 fields

- **File:** `./src/analysis/detectors/security/taint_analysis/language_support/typescript/analyzer.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'RustTaintAnalyzer' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/analysis/detectors/security/taint_analysis/language_support/rust/analyzer.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'create_command_execution_sinks' detected: 102 lines, 5 statements, complexity 5

- **File:** `./src/analysis/detectors/security/taint_analysis/sinks/command_sinks.rs`
- **Line:** 88

**Code:**
```
fn create_command_execution_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSink::new(
                    "rust_command_new".to_string(),
...
```

**Recommendation:** Consider breaking down 'create_command_execution_sinks' into smaller, more focused methods. Current metrics: LOC=102, Statements=5, Complexity=5, Nesting=3

---

#### Long method 'create_command_execution_sinks' detected: 102 lines, 5 statements, complexity 5

- **File:** `./src/analysis/detectors/security/taint_analysis/sinks/command_sinks.rs`
- **Line:** 88

**Code:**
```
fn create_command_execution_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSink::new(
                    "rust_command_new".to_string(),
...
```

**Recommendation:** Consider breaking down 'create_command_execution_sinks' into smaller, more focused methods. Current metrics: LOC=102, Statements=5, Complexity=5, Nesting=3

---

#### Large class 'PathTracker' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/path_tracker.rs`
- **Line:** 7

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SanitizerDetector' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/sanitizer_detector.rs`
- **Line:** 11

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'get_javascript_sanitizers' detected: 97 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/patterns.rs`
- **Line:** 92

**Code:**
```
pub fn get_javascript_sanitizers() -> Vec<SanitizationPoint> {
    vec![
        SanitizationPoint::new(
            "js_dompurify".to_string(),
            "DOMPurify.sanitize".to_string(),
...
```

**Recommendation:** Consider breaking down 'get_javascript_sanitizers' into smaller, more focused methods. Current metrics: LOC=97, Statements=1, Complexity=1, Nesting=1

---

#### Large class 'FlowAnalyzer' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/flow_analyzer.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'trace_taint_from_source' detected: 63 lines, 73 statements, complexity 9

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/flow_analyzer.rs`
- **Line:** 35

**Code:**
```
pub fn trace_taint_from_source(
        &self,
        graph: &DataFlowGraph,
        source_id: &str,
        path_tracker: &PathTracker,
...
```

**Recommendation:** Consider breaking down 'trace_taint_from_source' into smaller, more focused methods. Current metrics: LOC=63, Statements=73, Complexity=9, Nesting=9

---

#### Long method 'trace_taint_from_source' detected: 63 lines, 73 statements, complexity 9

- **File:** `./src/analysis/detectors/security/taint_analysis/propagation/flow_analyzer.rs`
- **Line:** 35

**Code:**
```
pub fn trace_taint_from_source(
        &self,
        graph: &DataFlowGraph,
        source_id: &str,
        path_tracker: &PathTracker,
...
```

**Recommendation:** Consider breaking down 'trace_taint_from_source' into smaller, more focused methods. Current metrics: LOC=63, Statements=73, Complexity=9, Nesting=9

---

#### Large class 'SecurityIssue' detected: 0 LOC, 14 methods, 14 fields

- **File:** `./src/analysis/detectors/security/types.rs`
- **Line:** 356

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'to_string' detected: 42 lines, 30 statements, complexity 30

- **File:** `./src/analysis/detectors/security/types.rs`
- **Line:** 152

**Code:**
```
fn to_string(&self) -> String {
        match self {
            SecurityIssueType::BrokenAccessControl => "Broken Access Control".to_string(),
            SecurityIssueType::CryptographicFailures => "Cryptographic Failures".to_string(),
            SecurityIssueType::Injection => "Injection".to_string(),
...
```

**Recommendation:** Consider breaking down 'to_string' into smaller, more focused methods. Current metrics: LOC=42, Statements=30, Complexity=30, Nesting=4

---

#### Long method 'to_string' detected: 42 lines, 30 statements, complexity 30

- **File:** `./src/analysis/detectors/security/types.rs`
- **Line:** 152

**Code:**
```
fn to_string(&self) -> String {
        match self {
            SecurityIssueType::BrokenAccessControl => "Broken Access Control".to_string(),
            SecurityIssueType::CryptographicFailures => "Cryptographic Failures".to_string(),
            SecurityIssueType::Injection => "Injection".to_string(),
...
```

**Recommendation:** Consider breaking down 'to_string' into smaller, more focused methods. Current metrics: LOC=42, Statements=30, Complexity=30, Nesting=4

---

#### Long method 'initialize_version_patterns' detected: 311 lines, 9 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/implementations/tls_analysis.rs`
- **Line:** 107

**Code:**
```
fn initialize_version_patterns() -> HashMap<SourceLanguage, Vec<TlsVersionPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_version_patterns' into smaller, more focused methods. Current metrics: LOC=311, Statements=9, Complexity=1, Nesting=1

---

#### Long method 'initialize_version_patterns' detected: 311 lines, 9 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/implementations/tls_analysis.rs`
- **Line:** 107

**Code:**
```
fn initialize_version_patterns() -> HashMap<SourceLanguage, Vec<TlsVersionPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_version_patterns' into smaller, more focused methods. Current metrics: LOC=311, Statements=9, Complexity=1, Nesting=1

---

#### Long method 'initialize_weak_algorithms' detected: 215 lines, 9 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/algorithms/symmetric.rs`
- **Line:** 69

**Code:**
```
fn initialize_weak_algorithms() -> HashMap<SourceLanguage, Vec<WeakSymmetricPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_weak_algorithms' into smaller, more focused methods. Current metrics: LOC=215, Statements=9, Complexity=1, Nesting=1

---

#### Long method 'initialize_weak_algorithms' detected: 215 lines, 9 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/algorithms/symmetric.rs`
- **Line:** 69

**Code:**
```
fn initialize_weak_algorithms() -> HashMap<SourceLanguage, Vec<WeakSymmetricPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_weak_algorithms' into smaller, more focused methods. Current metrics: LOC=215, Statements=9, Complexity=1, Nesting=1

---

#### Long method 'initialize_weak_hashes' detected: 271 lines, 11 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/algorithms/hashing.rs`
- **Line:** 90

**Code:**
```
fn initialize_weak_hashes() -> HashMap<SourceLanguage, Vec<WeakHashPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_weak_hashes' into smaller, more focused methods. Current metrics: LOC=271, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'initialize_weak_hashes' detected: 271 lines, 11 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/algorithms/hashing.rs`
- **Line:** 90

**Code:**
```
fn initialize_weak_hashes() -> HashMap<SourceLanguage, Vec<WeakHashPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_weak_hashes' into smaller, more focused methods. Current metrics: LOC=271, Statements=11, Complexity=1, Nesting=1

---

#### Large class 'RandomGenerationAnalyzer' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/analysis/detectors/security/crypto/algorithms/random_generation.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'initialize_weak_algorithms' detected: 189 lines, 9 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/algorithms/asymmetric.rs`
- **Line:** 69

**Code:**
```
fn initialize_weak_algorithms() -> HashMap<SourceLanguage, Vec<WeakAsymmetricPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_weak_algorithms' into smaller, more focused methods. Current metrics: LOC=189, Statements=9, Complexity=1, Nesting=1

---

#### Long method 'initialize_weak_algorithms' detected: 189 lines, 9 statements, complexity 1

- **File:** `./src/analysis/detectors/security/crypto/algorithms/asymmetric.rs`
- **Line:** 69

**Code:**
```
fn initialize_weak_algorithms() -> HashMap<SourceLanguage, Vec<WeakAsymmetricPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
...
```

**Recommendation:** Consider breaking down 'initialize_weak_algorithms' into smaller, more focused methods. Current metrics: LOC=189, Statements=9, Complexity=1, Nesting=1

---

#### Large class 'PatternMatcher' detected: 0 LOC, 12 methods, 2 fields

- **File:** `./src/analysis/detectors/security/agents/analysis/pattern_matcher.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AgentPatternDatabase' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/security/agents/patterns/agent_patterns.rs`
- **Line:** 42

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'load_default_patterns' detected: 104 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/agents/patterns/agent_patterns.rs`
- **Line:** 54

**Code:**
```
fn load_default_patterns() -> Vec<AgentPattern> {
        vec![
            AgentPattern {
                name: "autonomous_execution".to_string(),
                description: "Patterns indicating autonomous code execution".to_string(),
...
```

**Recommendation:** Consider breaking down 'load_default_patterns' into smaller, more focused methods. Current metrics: LOC=104, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'load_default_patterns' detected: 104 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/agents/patterns/agent_patterns.rs`
- **Line:** 54

**Code:**
```
fn load_default_patterns() -> Vec<AgentPattern> {
        vec![
            AgentPattern {
                name: "autonomous_execution".to_string(),
                description: "Patterns indicating autonomous code execution".to_string(),
...
```

**Recommendation:** Consider breaking down 'load_default_patterns' into smaller, more focused methods. Current metrics: LOC=104, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'load_default_patterns' detected: 119 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/agents/patterns/malicious/signatures.rs`
- **Line:** 60

**Code:**
```
pub fn load_default_patterns() -> Vec<MaliciousPattern> {
    vec![
        MaliciousPattern {
            name: "backdoor_signature".to_string(),
            description: "Known backdoor signatures and patterns".to_string(),
...
```

**Recommendation:** Consider breaking down 'load_default_patterns' into smaller, more focused methods. Current metrics: LOC=119, Statements=1, Complexity=1, Nesting=1

---

#### Large class 'MaliciousPatternDatabase' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/security/agents/patterns/malicious/behaviors.rs`
- **Line:** 19

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'analyze_file' detected: 57 lines, 84 statements, complexity 8

- **File:** `./src/analysis/detectors/security/agents/detector/orchestrator.rs`
- **Line:** 133

**Code:**
```
pub async fn analyze_file(
        &self,
        context: &SecurityContext,
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        info!(
...
```

**Recommendation:** Consider breaking down 'analyze_file' into smaller, more focused methods. Current metrics: LOC=57, Statements=84, Complexity=8, Nesting=6

---

#### Long method 'analyze_file' detected: 57 lines, 84 statements, complexity 8

- **File:** `./src/analysis/detectors/security/agents/detector/orchestrator.rs`
- **Line:** 133

**Code:**
```
pub async fn analyze_file(
        &self,
        context: &SecurityContext,
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        info!(
...
```

**Recommendation:** Consider breaking down 'analyze_file' into smaller, more focused methods. Current metrics: LOC=57, Statements=84, Complexity=8, Nesting=6

---

#### Long method 'new' detected: 111 lines, 37 statements, complexity 1

- **File:** `./src/analysis/detectors/security/owasp/language_support/web_frameworks.rs`
- **Line:** 24

**Code:**
```
pub fn new() -> Self {
        let mut framework_patterns = HashMap::new();

        // Rust frameworks
        framework_patterns.insert(
...
```

**Recommendation:** Consider breaking down 'new' into smaller, more focused methods. Current metrics: LOC=111, Statements=37, Complexity=1, Nesting=1

---

#### Long method 'new' detected: 111 lines, 37 statements, complexity 1

- **File:** `./src/analysis/detectors/security/owasp/language_support/web_frameworks.rs`
- **Line:** 24

**Code:**
```
pub fn new() -> Self {
        let mut framework_patterns = HashMap::new();

        // Rust frameworks
        framework_patterns.insert(
...
```

**Recommendation:** Consider breaking down 'new' into smaller, more focused methods. Current metrics: LOC=111, Statements=37, Complexity=1, Nesting=1

---

#### Long method 'new' detected: 96 lines, 26 statements, complexity 1

- **File:** `./src/analysis/detectors/security/owasp/language_support/api_security.rs`
- **Line:** 30

**Code:**
```
pub fn new() -> Self {
        let mut api_patterns = HashMap::new();

        // REST API security patterns
        api_patterns.insert(
...
```

**Recommendation:** Consider breaking down 'new' into smaller, more focused methods. Current metrics: LOC=96, Statements=26, Complexity=1, Nesting=1

---

#### Long method 'new' detected: 96 lines, 26 statements, complexity 1

- **File:** `./src/analysis/detectors/security/owasp/language_support/api_security.rs`
- **Line:** 30

**Code:**
```
pub fn new() -> Self {
        let mut api_patterns = HashMap::new();

        // REST API security patterns
        api_patterns.insert(
...
```

**Recommendation:** Consider breaking down 'new' into smaller, more focused methods. Current metrics: LOC=96, Statements=26, Complexity=1, Nesting=1

---

#### Large class 'DataFlowScanner' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/analysis/detectors/security/owasp/scanners/flow_scanner.rs`
- **Line:** 67

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'initialize_patterns' detected: 111 lines, 12 statements, complexity 3

- **File:** `./src/analysis/detectors/security/owasp/scanners/pattern_scanner.rs`
- **Line:** 73

**Code:**
```
fn initialize_patterns(&mut self) -> Result<(), AnalysisError> {
        let patterns = vec![
            // SQL Injection patterns
            VulnerabilityPattern {
                id: "sql_injection_concatenation".to_string(),
...
```

**Recommendation:** Consider breaking down 'initialize_patterns' into smaller, more focused methods. Current metrics: LOC=111, Statements=12, Complexity=3, Nesting=5

---

#### Long method 'initialize_patterns' detected: 111 lines, 12 statements, complexity 3

- **File:** `./src/analysis/detectors/security/owasp/scanners/pattern_scanner.rs`
- **Line:** 73

**Code:**
```
fn initialize_patterns(&mut self) -> Result<(), AnalysisError> {
        let patterns = vec![
            // SQL Injection patterns
            VulnerabilityPattern {
                id: "sql_injection_concatenation".to_string(),
...
```

**Recommendation:** Consider breaking down 'initialize_patterns' into smaller, more focused methods. Current metrics: LOC=111, Statements=12, Complexity=3, Nesting=5

---

#### Large class 'DependencyScanner' detected: 0 LOC, 13 methods, 1 fields

- **File:** `./src/analysis/detectors/security/owasp/scanners/dependency_scanner.rs`
- **Line:** 79

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'build_database_patterns' detected: 130 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/defaults/database_defaults.rs`
- **Line:** 71

**Code:**
```
fn build_database_patterns() -> Vec<DatabaseDefaultPattern> {
        vec![
            DatabaseDefaultPattern {
                name: "Default Database Credentials".to_string(),
                description: "Default database username or password detected".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_database_patterns' into smaller, more focused methods. Current metrics: LOC=130, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'build_database_patterns' detected: 130 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/defaults/database_defaults.rs`
- **Line:** 71

**Code:**
```
fn build_database_patterns() -> Vec<DatabaseDefaultPattern> {
        vec![
            DatabaseDefaultPattern {
                name: "Default Database Credentials".to_string(),
                description: "Default database username or password detected".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_database_patterns' into smaller, more focused methods. Current metrics: LOC=130, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'build_crypto_patterns' detected: 153 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/defaults/crypto_defaults.rs`
- **Line:** 71

**Code:**
```
fn build_crypto_patterns() -> Vec<CryptoDefaultPattern> {
        vec![
            CryptoDefaultPattern {
                name: "Default Encryption Key".to_string(),
                description: "Default or weak encryption key detected".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_crypto_patterns' into smaller, more focused methods. Current metrics: LOC=153, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'build_crypto_patterns' detected: 153 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/defaults/crypto_defaults.rs`
- **Line:** 71

**Code:**
```
fn build_crypto_patterns() -> Vec<CryptoDefaultPattern> {
        vec![
            CryptoDefaultPattern {
                name: "Default Encryption Key".to_string(),
                description: "Default or weak encryption key detected".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_crypto_patterns' into smaller, more focused methods. Current metrics: LOC=153, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'build_password_patterns' detected: 115 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/defaults/password_defaults.rs`
- **Line:** 71

**Code:**
```
fn build_password_patterns() -> Vec<PasswordDefaultPattern> {
        vec![
            PasswordDefaultPattern {
                name: "Default Password".to_string(),
                description: "Common default password detected".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_password_patterns' into smaller, more focused methods. Current metrics: LOC=115, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'build_password_patterns' detected: 115 lines, 1 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/defaults/password_defaults.rs`
- **Line:** 71

**Code:**
```
fn build_password_patterns() -> Vec<PasswordDefaultPattern> {
        vec![
            PasswordDefaultPattern {
                name: "Default Password".to_string(),
                description: "Common default password detected".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_password_patterns' into smaller, more focused methods. Current metrics: LOC=115, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'build_access_control_rules' detected: 102 lines, 3 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/access_control.rs`
- **Line:** 65

**Code:**
```
fn build_access_control_rules() -> Result<Vec<AccessControlRule>, AnalysisError> {
        let rules = vec![
            AccessControlRule {
                name: "Allow All Access".to_string(),
                description: "Access control configured to allow all users/IPs".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_access_control_rules' into smaller, more focused methods. Current metrics: LOC=102, Statements=3, Complexity=1, Nesting=1

---

#### Long method 'build_access_control_rules' detected: 102 lines, 3 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/access_control.rs`
- **Line:** 65

**Code:**
```
fn build_access_control_rules() -> Result<Vec<AccessControlRule>, AnalysisError> {
        let rules = vec![
            AccessControlRule {
                name: "Allow All Access".to_string(),
                description: "Access control configured to allow all users/IPs".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_access_control_rules' into smaller, more focused methods. Current metrics: LOC=102, Statements=3, Complexity=1, Nesting=1

---

#### Long method 'check_kubernetes_security_context' detected: 45 lines, 43 statements, complexity 9

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/analyzer.rs`
- **Line:** 77

**Code:**
```
fn check_kubernetes_security_context(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_kubernetes_security_context' into smaller, more focused methods. Current metrics: LOC=45, Statements=43, Complexity=9, Nesting=13

---

#### Long method 'check_kubernetes_security_context' detected: 45 lines, 43 statements, complexity 9

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/analyzer.rs`
- **Line:** 77

**Code:**
```
fn check_kubernetes_security_context(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_kubernetes_security_context' into smaller, more focused methods. Current metrics: LOC=45, Statements=43, Complexity=9, Nesting=13

---

#### Long method 'check_database_permissions_recursive' detected: 44 lines, 40 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/analyzer.rs`
- **Line:** 139

**Code:**
```
fn check_database_permissions_recursive(
        &self,
        value: &YamlValue,
        path: &str,
        issues: &mut Vec<ConfigIssue>,
...
```

**Recommendation:** Consider breaking down 'check_database_permissions_recursive' into smaller, more focused methods. Current metrics: LOC=44, Statements=40, Complexity=13, Nesting=14

---

#### Long method 'check_database_permissions_recursive' detected: 44 lines, 40 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/analyzer.rs`
- **Line:** 139

**Code:**
```
fn check_database_permissions_recursive(
        &self,
        value: &YamlValue,
        path: &str,
        issues: &mut Vec<ConfigIssue>,
...
```

**Recommendation:** Consider breaking down 'check_database_permissions_recursive' into smaller, more focused methods. Current metrics: LOC=44, Statements=40, Complexity=13, Nesting=14

---

#### Long method 'check_service_user_configuration' detected: 34 lines, 34 statements, complexity 9

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/analyzer.rs`
- **Line:** 189

**Code:**
```
fn check_service_user_configuration(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_service_user_configuration' into smaller, more focused methods. Current metrics: LOC=34, Statements=34, Complexity=9, Nesting=17

---

#### Long method 'check_service_user_configuration' detected: 34 lines, 34 statements, complexity 9

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/analyzer.rs`
- **Line:** 189

**Code:**
```
fn check_service_user_configuration(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_service_user_configuration' into smaller, more focused methods. Current metrics: LOC=34, Statements=34, Complexity=9, Nesting=17

---

#### Long method 'build_privilege_escalation_rules' detected: 103 lines, 3 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/privilege_escalation.rs`
- **Line:** 65

**Code:**
```
fn build_privilege_escalation_rules() -> Result<Vec<PrivilegeEscalationRule>, AnalysisError> {
        let rules = vec![
            PrivilegeEscalationRule {
                name: "Passwordless Sudo".to_string(),
                description: "Sudo configured without password requirement".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_privilege_escalation_rules' into smaller, more focused methods. Current metrics: LOC=103, Statements=3, Complexity=1, Nesting=1

---

#### Long method 'build_privilege_escalation_rules' detected: 103 lines, 3 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/analysis/permissions/privilege_escalation.rs`
- **Line:** 65

**Code:**
```
fn build_privilege_escalation_rules() -> Result<Vec<PrivilegeEscalationRule>, AnalysisError> {
        let rules = vec![
            PrivilegeEscalationRule {
                name: "Passwordless Sudo".to_string(),
                description: "Sudo configured without password requirement".to_string(),
...
```

**Recommendation:** Consider breaking down 'build_privilege_escalation_rules' into smaller, more focused methods. Current metrics: LOC=103, Statements=3, Complexity=1, Nesting=1

---

#### Large class 'ComplianceValidator' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/analysis/detectors/security/config/validation/compliance/validator.rs`
- **Line:** 15

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'EnvValidator' detected: 0 LOC, 12 methods, 1 fields

- **File:** `./src/analysis/detectors/security/config/language_support/env/validator.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'build_secret_patterns' detected: 95 lines, 3 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/language_support/env/secret_patterns.rs`
- **Line:** 68

**Code:**
```
fn build_secret_patterns() -> Result<Vec<EnvSecretPattern>, AnalysisError> {
        let patterns = vec![
            EnvSecretPattern {
                name: "AWS Access Key".to_string(),
                pattern: Regex::new(r"^AWS_ACCESS_KEY.*=.*AKIA[0-9A-Z]{16}")?,
...
```

**Recommendation:** Consider breaking down 'build_secret_patterns' into smaller, more focused methods. Current metrics: LOC=95, Statements=3, Complexity=1, Nesting=1

---

#### Long method 'build_secret_patterns' detected: 95 lines, 3 statements, complexity 1

- **File:** `./src/analysis/detectors/security/config/language_support/env/secret_patterns.rs`
- **Line:** 68

**Code:**
```
fn build_secret_patterns() -> Result<Vec<EnvSecretPattern>, AnalysisError> {
        let patterns = vec![
            EnvSecretPattern {
                name: "AWS Access Key".to_string(),
                pattern: Regex::new(r"^AWS_ACCESS_KEY.*=.*AKIA[0-9A-Z]{16}")?,
...
```

**Recommendation:** Consider breaking down 'build_secret_patterns' into smaller, more focused methods. Current metrics: LOC=95, Statements=3, Complexity=1, Nesting=1

---

#### Long method 'check_volume_mounts' detected: 64 lines, 58 statements, complexity 12

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/kubernetes_patterns.rs`
- **Line:** 92

**Code:**
```
pub fn check_volume_mounts(volumes: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        if let YamlValue::Sequence(volume_list) = volumes {
            for volume in volume_list {
                if let YamlValue::Mapping(volume_map) = volume {
                    // Check for sensitive host paths
...
```

**Recommendation:** Consider breaking down 'check_volume_mounts' into smaller, more focused methods. Current metrics: LOC=64, Statements=58, Complexity=12, Nesting=17

---

#### Long method 'check_volume_mounts' detected: 64 lines, 58 statements, complexity 12

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/kubernetes_patterns.rs`
- **Line:** 92

**Code:**
```
pub fn check_volume_mounts(volumes: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        if let YamlValue::Sequence(volume_list) = volumes {
            for volume in volume_list {
                if let YamlValue::Mapping(volume_map) = volume {
                    // Check for sensitive host paths
...
```

**Recommendation:** Consider breaking down 'check_volume_mounts' into smaller, more focused methods. Current metrics: LOC=64, Statements=58, Complexity=12, Nesting=17

---

#### Long method 'check_capabilities' detected: 44 lines, 42 statements, complexity 10

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/kubernetes_patterns.rs`
- **Line:** 162

**Code:**
```
pub fn check_capabilities(capabilities: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        if let YamlValue::Mapping(cap_map) = capabilities {
            // Check for dangerous capabilities being added
            if let Some(add_caps) = cap_map.get(&YamlValue::String("add".to_string())) {
                if let YamlValue::Sequence(cap_list) = add_caps {
...
```

**Recommendation:** Consider breaking down 'check_capabilities' into smaller, more focused methods. Current metrics: LOC=44, Statements=42, Complexity=10, Nesting=13

---

#### Long method 'check_capabilities' detected: 44 lines, 42 statements, complexity 10

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/kubernetes_patterns.rs`
- **Line:** 162

**Code:**
```
pub fn check_capabilities(capabilities: &YamlValue, issues: &mut Vec<ConfigIssue>) {
        if let YamlValue::Mapping(cap_map) = capabilities {
            // Check for dangerous capabilities being added
            if let Some(add_caps) = cap_map.get(&YamlValue::String("add".to_string())) {
                if let YamlValue::Sequence(cap_list) = add_caps {
...
```

**Recommendation:** Consider breaking down 'check_capabilities' into smaller, more focused methods. Current metrics: LOC=44, Statements=42, Complexity=10, Nesting=13

---

#### Long method 'check_duplicate_keys_recursive' detected: 61 lines, 53 statements, complexity 19

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/validator.rs`
- **Line:** 124

**Code:**
```
fn check_duplicate_keys_recursive(
        &self,
        value: &YamlValue,
        path: &str,
        issues: &mut Vec<ConfigIssue>,
...
```

**Recommendation:** Consider breaking down 'check_duplicate_keys_recursive' into smaller, more focused methods. Current metrics: LOC=61, Statements=53, Complexity=19, Nesting=15

---

#### Long method 'check_duplicate_keys_recursive' detected: 61 lines, 53 statements, complexity 19

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/validator.rs`
- **Line:** 124

**Code:**
```
fn check_duplicate_keys_recursive(
        &self,
        value: &YamlValue,
        path: &str,
        issues: &mut Vec<ConfigIssue>,
...
```

**Recommendation:** Consider breaking down 'check_duplicate_keys_recursive' into smaller, more focused methods. Current metrics: LOC=61, Statements=53, Complexity=19, Nesting=15

---

#### Long method 'check_environment_config' detected: 33 lines, 37 statements, complexity 11

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/validator.rs`
- **Line:** 192

**Code:**
```
pub fn check_environment_config(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            // Look for environment-specific sections
...
```

**Recommendation:** Consider breaking down 'check_environment_config' into smaller, more focused methods. Current metrics: LOC=33, Statements=37, Complexity=11, Nesting=21

---

#### Long method 'check_environment_config' detected: 33 lines, 37 statements, complexity 11

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/validator.rs`
- **Line:** 192

**Code:**
```
pub fn check_environment_config(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            // Look for environment-specific sections
...
```

**Recommendation:** Consider breaking down 'check_environment_config' into smaller, more focused methods. Current metrics: LOC=33, Statements=37, Complexity=11, Nesting=21

---

#### Long method 'check_credentials_recursive' detected: 40 lines, 37 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/patterns.rs`
- **Line:** 92

**Code:**
```
fn check_credentials_recursive(value: &YamlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
...
```

**Recommendation:** Consider breaking down 'check_credentials_recursive' into smaller, more focused methods. Current metrics: LOC=40, Statements=37, Complexity=13, Nesting=14

---

#### Long method 'check_credentials_recursive' detected: 40 lines, 37 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/patterns.rs`
- **Line:** 92

**Code:**
```
fn check_credentials_recursive(value: &YamlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
...
```

**Recommendation:** Consider breaking down 'check_credentials_recursive' into smaller, more focused methods. Current metrics: LOC=40, Statements=37, Complexity=13, Nesting=14

---

#### Long method 'check_cors_settings' detected: 26 lines, 29 statements, complexity 9

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/patterns.rs`
- **Line:** 204

**Code:**
```
fn check_cors_settings(map: &serde_yaml::Mapping, issues: &mut Vec<ConfigIssue>) {
        for (key, val) in map {
            if let Some(key_str) = key.as_str() {
                if key_str.to_lowercase().contains("cors") {
                    if let YamlValue::Mapping(cors_config) = val {
...
```

**Recommendation:** Consider breaking down 'check_cors_settings' into smaller, more focused methods. Current metrics: LOC=26, Statements=29, Complexity=9, Nesting=17

---

#### Long method 'check_cors_settings' detected: 26 lines, 29 statements, complexity 9

- **File:** `./src/analysis/detectors/security/config/language_support/yaml/patterns.rs`
- **Line:** 204

**Code:**
```
fn check_cors_settings(map: &serde_yaml::Mapping, issues: &mut Vec<ConfigIssue>) {
        for (key, val) in map {
            if let Some(key_str) = key.as_str() {
                if key_str.to_lowercase().contains("cors") {
                    if let YamlValue::Mapping(cors_config) = val {
...
```

**Recommendation:** Consider breaking down 'check_cors_settings' into smaller, more focused methods. Current metrics: LOC=26, Statements=29, Complexity=9, Nesting=17

---

#### Long method 'check_poetry_dependencies' detected: 57 lines, 40 statements, complexity 11

- **File:** `./src/analysis/detectors/security/config/language_support/toml/python_deps.rs`
- **Line:** 90

**Code:**
```
pub fn check_poetry_dependencies(&self, table: &toml::value::Table) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for Poetry sources
        if let Some(TomlValue::Array(sources)) = table.get("source") {
...
```

**Recommendation:** Consider breaking down 'check_poetry_dependencies' into smaller, more focused methods. Current metrics: LOC=57, Statements=40, Complexity=11, Nesting=11

---

#### Long method 'check_poetry_dependencies' detected: 57 lines, 40 statements, complexity 11

- **File:** `./src/analysis/detectors/security/config/language_support/toml/python_deps.rs`
- **Line:** 90

**Code:**
```
pub fn check_poetry_dependencies(&self, table: &toml::value::Table) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for Poetry sources
        if let Some(TomlValue::Array(sources)) = table.get("source") {
...
```

**Recommendation:** Consider breaking down 'check_poetry_dependencies' into smaller, more focused methods. Current metrics: LOC=57, Statements=40, Complexity=11, Nesting=11

---

#### Long method 'check_workspace_security' detected: 58 lines, 38 statements, complexity 10

- **File:** `./src/analysis/detectors/security/config/language_support/toml/build_checker.rs`
- **Line:** 151

**Code:**
```
fn check_workspace_security(
        &self,
        table: &toml::value::Table,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_workspace_security' into smaller, more focused methods. Current metrics: LOC=58, Statements=38, Complexity=10, Nesting=11

---

#### Long method 'check_workspace_security' detected: 58 lines, 38 statements, complexity 10

- **File:** `./src/analysis/detectors/security/config/language_support/toml/build_checker.rs`
- **Line:** 151

**Code:**
```
fn check_workspace_security(
        &self,
        table: &toml::value::Table,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_workspace_security' into smaller, more focused methods. Current metrics: LOC=58, Statements=38, Complexity=10, Nesting=11

---

#### Long method 'check_dependency_sources' detected: 64 lines, 45 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/language_support/toml/rust_deps.rs`
- **Line:** 157

**Code:**
```
pub fn check_dependency_sources(
        &self,
        table: &toml::value::Table,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_dependency_sources' into smaller, more focused methods. Current metrics: LOC=64, Statements=45, Complexity=13, Nesting=15

---

#### Long method 'check_dependency_sources' detected: 64 lines, 45 statements, complexity 13

- **File:** `./src/analysis/detectors/security/config/language_support/toml/rust_deps.rs`
- **Line:** 157

**Code:**
```
pub fn check_dependency_sources(
        &self,
        table: &toml::value::Table,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
...
```

**Recommendation:** Consider breaking down 'check_dependency_sources' into smaller, more focused methods. Current metrics: LOC=64, Statements=45, Complexity=13, Nesting=15

---

#### Long method 'check_db_table' detected: 79 lines, 50 statements, complexity 11

- **File:** `./src/analysis/detectors/security/config/language_support/toml/credential_checker.rs`
- **Line:** 116

**Code:**
```
fn check_db_table(
        &self,
        db_table: &toml::value::Table,
        section_name: &str,
        issues: &mut Vec<ConfigIssue>,
...
```

**Recommendation:** Consider breaking down 'check_db_table' into smaller, more focused methods. Current metrics: LOC=79, Statements=50, Complexity=11, Nesting=9

---

#### Long method 'check_db_table' detected: 79 lines, 50 statements, complexity 11

- **File:** `./src/analysis/detectors/security/config/language_support/toml/credential_checker.rs`
- **Line:** 116

**Code:**
```
fn check_db_table(
        &self,
        db_table: &toml::value::Table,
        section_name: &str,
        issues: &mut Vec<ConfigIssue>,
...
```

**Recommendation:** Consider breaking down 'check_db_table' into smaller, more focused methods. Current metrics: LOC=79, Statements=50, Complexity=11, Nesting=9

---

#### Long method 'extract_connections' detected: 25 lines, 42 statements, complexity 10

- **File:** `./src/analysis/interactive_diagram_generator.rs`
- **Line:** 181

**Code:**
```
fn extract_connections(&self, node_id: &str, mermaid_code: &str) -> Vec<String> {
        let mut connections = Vec::new();

        // Parse Mermaid code to find connections
        // This is a simplified implementation - could be enhanced with proper parsing
...
```

**Recommendation:** Consider breaking down 'extract_connections' into smaller, more focused methods. Current metrics: LOC=25, Statements=42, Complexity=10, Nesting=15

---

#### Long method 'extract_connections' detected: 25 lines, 42 statements, complexity 10

- **File:** `./src/analysis/interactive_diagram_generator.rs`
- **Line:** 181

**Code:**
```
fn extract_connections(&self, node_id: &str, mermaid_code: &str) -> Vec<String> {
        let mut connections = Vec::new();

        // Parse Mermaid code to find connections
        // This is a simplified implementation - could be enhanced with proper parsing
...
```

**Recommendation:** Consider breaking down 'extract_connections' into smaller, more focused methods. Current metrics: LOC=25, Statements=42, Complexity=10, Nesting=15

---

#### Large class 'ComponentExtractor' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/analysis/component_extractor.rs`
- **Line:** 17

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'analyze_dependencies_in_ast' detected: 80 lines, 66 statements, complexity 15

- **File:** `./src/analysis/component_extractor.rs`
- **Line:** 179

**Code:**
```
fn analyze_dependencies_in_ast(
        &self,
        ast_node: &CustomAst,
        components: &mut [ArchitecturalComponent],
        name_to_component: &HashMap<String, usize>,
...
```

**Recommendation:** Consider breaking down 'analyze_dependencies_in_ast' into smaller, more focused methods. Current metrics: LOC=80, Statements=66, Complexity=15, Nesting=13

---

#### Long method 'analyze_dependencies_in_ast' detected: 80 lines, 66 statements, complexity 15

- **File:** `./src/analysis/component_extractor.rs`
- **Line:** 179

**Code:**
```
fn analyze_dependencies_in_ast(
        &self,
        ast_node: &CustomAst,
        components: &mut [ArchitecturalComponent],
        name_to_component: &HashMap<String, usize>,
...
```

**Recommendation:** Consider breaking down 'analyze_dependencies_in_ast' into smaller, more focused methods. Current metrics: LOC=80, Statements=66, Complexity=15, Nesting=13

---

#### Large class 'AnalysisEngineBuilder' detected: 0 LOC, 14 methods, 10 fields

- **File:** `./src/analysis/engine_builder.rs`
- **Line:** 91

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AnalysisOrchestrator' detected: 0 LOC, 14 methods, 4 fields

- **File:** `./src/analysis/orchestrator.rs`
- **Line:** 51

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PerformanceAnalyzer' detected: 0 LOC, 14 methods, 2 fields

- **File:** `./src/analysis/performance/baseline.rs`
- **Line:** 80

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PerformanceValidator' detected: 0 LOC, 12 methods, 2 fields

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 64

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'test_cache_performance' detected: 84 lines, 70 statements, complexity 13

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 192

**Code:**
```
async fn test_cache_performance(&self) -> Result<TestResult, ValidationError> {
        let test_diagram = r#"
graph TD
    A[Cache Test] --> B[Performance]
    B --> C[Validation]
...
```

**Recommendation:** Consider breaking down 'test_cache_performance' into smaller, more focused methods. Current metrics: LOC=84, Statements=70, Complexity=13, Nesting=9

---

#### Long method 'test_cache_performance' detected: 84 lines, 70 statements, complexity 13

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 192

**Code:**
```
async fn test_cache_performance(&self) -> Result<TestResult, ValidationError> {
        let test_diagram = r#"
graph TD
    A[Cache Test] --> B[Performance]
    B --> C[Validation]
...
```

**Recommendation:** Consider breaking down 'test_cache_performance' into smaller, more focused methods. Current metrics: LOC=84, Statements=70, Complexity=13, Nesting=9

---

#### Long method 'generate_summary_report' detected: 84 lines, 87 statements, complexity 12

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 759

**Code:**
```
fn generate_summary_report(&self, test_suite: &PerformanceTestSuite) -> String {
        let mut report = String::new();

        report.push_str("# UV-48 Performance Validation Report\n\n");
        report.push_str(&format!("**Generated:** {}\n\n", test_suite.timestamp));
...
```

**Recommendation:** Consider breaking down 'generate_summary_report' into smaller, more focused methods. Current metrics: LOC=84, Statements=87, Complexity=12, Nesting=8

---

#### Long method 'generate_summary_report' detected: 84 lines, 87 statements, complexity 12

- **File:** `./src/analysis/performance/testing.rs`
- **Line:** 759

**Code:**
```
fn generate_summary_report(&self, test_suite: &PerformanceTestSuite) -> String {
        let mut report = String::new();

        report.push_str("# UV-48 Performance Validation Report\n\n");
        report.push_str(&format!("**Generated:** {}\n\n", test_suite.timestamp));
...
```

**Recommendation:** Consider breaking down 'generate_summary_report' into smaller, more focused methods. Current metrics: LOC=84, Statements=87, Complexity=12, Nesting=8

---

#### Large class 'LargeCodebaseOptimizer' detected: 0 LOC, 18 methods, 10 fields

- **File:** `./src/analysis/performance/large_codebase_optimizer.rs`
- **Line:** 162

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'MultiLanguageProcessor' detected: 0 LOC, 19 methods, 9 fields

- **File:** `./src/analysis/parallel/multi_language_processor.rs`
- **Line:** 411

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ZeroCopyAstCache' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/analysis/memory/zero_copy.rs`
- **Line:** 187

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'InvalidationManager' detected: 0 LOC, 13 methods, 5 fields

- **File:** `./src/analysis/diagram_cache/invalidation_manager.rs`
- **Line:** 72

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DiagramDependencyTracker' detected: 0 LOC, 12 methods, 3 fields

- **File:** `./src/analysis/diagram_cache/diagram_tracker.rs`
- **Line:** 37

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DiagramCacheEngine' detected: 0 LOC, 15 methods, 7 fields

- **File:** `./src/analysis/diagram_cache/cache_engine.rs`
- **Line:** 22

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'CompressionEngine' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/analysis/diagram_cache/compression.rs`
- **Line:** 34

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AnalysisConfig' detected: 0 LOC, 15 methods, 8 fields

- **File:** `./src/analysis/config.rs`
- **Line:** 77

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DetectorCacheValidator' detected: 0 LOC, 17 methods, 2 fields

- **File:** `./src/bin/detector_cache_validator.rs`
- **Line:** 104

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'generate_rust_project' detected: 191 lines, 38 statements, complexity 1

- **File:** `./src/bin/generate_benchmark_data.rs`
- **Line:** 52

**Code:**
```
fn generate_rust_project(dir: impl AsRef<Path>) -> std::io::Result<()> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir.join("src"))?;

    // Create Cargo.toml
...
```

**Recommendation:** Consider breaking down 'generate_rust_project' into smaller, more focused methods. Current metrics: LOC=191, Statements=38, Complexity=1, Nesting=2

---

#### Long method 'generate_python_project' detected: 111 lines, 36 statements, complexity 1

- **File:** `./src/bin/generate_benchmark_data.rs`
- **Line:** 294

**Code:**
```
fn generate_python_project(dir: impl AsRef<Path>) -> std::io::Result<()> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir.join("src"))?;

    // Create requirements.txt
...
```

**Recommendation:** Consider breaking down 'generate_python_project' into smaller, more focused methods. Current metrics: LOC=111, Statements=36, Complexity=1, Nesting=2

---

#### Long method 'generate_javascript_project' detected: 139 lines, 33 statements, complexity 1

- **File:** `./src/bin/generate_benchmark_data.rs`
- **Line:** 444

**Code:**
```
fn generate_javascript_project(dir: impl AsRef<Path>) -> std::io::Result<()> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir.join("src"))?;

    // Create package.json
...
```

**Recommendation:** Consider breaking down 'generate_javascript_project' into smaller, more focused methods. Current metrics: LOC=139, Statements=33, Complexity=1, Nesting=2

---

#### Long method 'print_report' detected: 66 lines, 78 statements, complexity 8

- **File:** `./src/bin/cache_memory_profiler.rs`
- **Line:** 84

**Code:**
```
pub fn print_report(&self) {
        println!("\n🧠 Cache Memory Profiling Results");
        println!("=================================");

        println!("\n📊 Configuration Comparison:");
...
```

**Recommendation:** Consider breaking down 'print_report' into smaller, more focused methods. Current metrics: LOC=66, Statements=78, Complexity=8, Nesting=7

---

#### Long method 'print_report' detected: 66 lines, 78 statements, complexity 8

- **File:** `./src/bin/cache_memory_profiler.rs`
- **Line:** 84

**Code:**
```
pub fn print_report(&self) {
        println!("\n🧠 Cache Memory Profiling Results");
        println!("=================================");

        println!("\n📊 Configuration Comparison:");
...
```

**Recommendation:** Consider breaking down 'print_report' into smaller, more focused methods. Current metrics: LOC=66, Statements=78, Complexity=8, Nesting=7

---

#### Long method 'find_source_files' detected: 38 lines, 55 statements, complexity 15

- **File:** `./src/bin/cache_benchmark.rs`
- **Line:** 170

**Code:**
```
fn find_source_files<'a>(path: &'a Path, patterns: &'a [String]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>, Box<dyn std::error::Error>>> + Send + 'a>> {
    Box::pin(async move {
        let mut files = Vec::new();

        if path.is_file() {
...
```

**Recommendation:** Consider breaking down 'find_source_files' into smaller, more focused methods. Current metrics: LOC=38, Statements=55, Complexity=15, Nesting=15

---

#### Long method 'build_context' detected: 85 lines, 65 statements, complexity 9

- **File:** `./src/engine/analysis/pipeline.rs`
- **Line:** 108

**Code:**
```
pub fn build_context(
        &self,
        file_path: &Path,
        project_context: ProjectContext,
    ) -> Result<AnalysisContext, PipelineError> {
...
```

**Recommendation:** Consider breaking down 'build_context' into smaller, more focused methods. Current metrics: LOC=85, Statements=65, Complexity=9, Nesting=10

---

#### Long method 'build_context' detected: 85 lines, 65 statements, complexity 9

- **File:** `./src/engine/analysis/pipeline.rs`
- **Line:** 108

**Code:**
```
pub fn build_context(
        &self,
        file_path: &Path,
        project_context: ProjectContext,
    ) -> Result<AnalysisContext, PipelineError> {
...
```

**Recommendation:** Consider breaking down 'build_context' into smaller, more focused methods. Current metrics: LOC=85, Statements=65, Complexity=9, Nesting=10

---

#### Large class 'GraphAwarePipeline' detected: 0 LOC, 11 methods, 6 fields

- **File:** `./src/engine/analysis/graph_pipeline.rs`
- **Line:** 20

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'GraphBuilder' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/engine/knowledge_graph/builder.rs`
- **Line:** 40

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'execute' detected: 58 lines, 82 statements, complexity 17

- **File:** `./src/engine/knowledge_graph/query.rs`
- **Line:** 118

**Code:**
```
pub fn execute(self, graph: &KnowledgeGraph) -> QueryResult {
        let mut visited = HashSet::new();
        let mut result_nodes = Vec::new();
        let mut result_paths = Vec::new();

...
```

**Recommendation:** Consider breaking down 'execute' into smaller, more focused methods. Current metrics: LOC=58, Statements=82, Complexity=17, Nesting=15

---

#### Long method 'execute' detected: 58 lines, 82 statements, complexity 17

- **File:** `./src/engine/knowledge_graph/query.rs`
- **Line:** 118

**Code:**
```
pub fn execute(self, graph: &KnowledgeGraph) -> QueryResult {
        let mut visited = HashSet::new();
        let mut result_nodes = Vec::new();
        let mut result_paths = Vec::new();

...
```

**Recommendation:** Consider breaking down 'execute' into smaller, more focused methods. Current metrics: LOC=58, Statements=82, Complexity=17, Nesting=15

---

#### Long method 'create_test_context' detected: 157 lines, 18 statements, complexity 1

- **File:** `./src/engine/test_integration.rs`
- **Line:** 16

**Code:**
```
fn create_test_context() -> AnalysisContext {
        let source = r#"
struct TestStruct {
    field1: i32,
    field2: String,
...
```

**Recommendation:** Consider breaking down 'create_test_context' into smaller, more focused methods. Current metrics: LOC=157, Statements=18, Complexity=1, Nesting=1

---

#### Long method 'parse_file' detected: 60 lines, 77 statements, complexity 6

- **File:** `./src/engine/parsing/ast_builder.rs`
- **Line:** 168

**Code:**
```
pub fn parse_file(&self, file_path: &Path) -> Result<ParseResult, ParseError> {
        // Security validation
        security::validate_file_size(file_path)
            .map_err(|e| ParseError::ParseFailed(format!("Security validation failed: {}", e)))?;
        security::validate_file_type(file_path)
...
```

**Recommendation:** Consider breaking down 'parse_file' into smaller, more focused methods. Current metrics: LOC=60, Statements=77, Complexity=6, Nesting=8

---

#### Long method 'parse_file' detected: 60 lines, 77 statements, complexity 6

- **File:** `./src/engine/parsing/ast_builder.rs`
- **Line:** 168

**Code:**
```
pub fn parse_file(&self, file_path: &Path) -> Result<ParseResult, ParseError> {
        // Security validation
        security::validate_file_size(file_path)
            .map_err(|e| ParseError::ParseFailed(format!("Security validation failed: {}", e)))?;
        security::validate_file_type(file_path)
...
```

**Recommendation:** Consider breaking down 'parse_file' into smaller, more focused methods. Current metrics: LOC=60, Statements=77, Complexity=6, Nesting=8

---

#### Large class 'AiAnalysisEngine' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/ai/engine.rs`
- **Line:** 15

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'create_god_object_pattern' detected: 189 lines, 11 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/universal.rs`
- **Line:** 75

**Code:**
```
fn create_god_object_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "god_object".to_string(),
        name: "God Object".to_string(),
        definition: CompressedString::new(
...
```

**Recommendation:** Consider breaking down 'create_god_object_pattern' into smaller, more focused methods. Current metrics: LOC=189, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'create_tight_coupling_pattern' detected: 105 lines, 11 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/universal.rs`
- **Line:** 354

**Code:**
```
fn create_tight_coupling_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "tight_coupling".to_string(),
        name: "Tight Coupling".to_string(),
        definition: CompressedString::new(
...
```

**Recommendation:** Consider breaking down 'create_tight_coupling_pattern' into smaller, more focused methods. Current metrics: LOC=105, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'create_rust_patterns' detected: 349 lines, 30 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/language_specific.rs`
- **Line:** 37

**Code:**
```
fn create_rust_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in Rust context
    patterns.insert("god_object".to_string(), PatternKnowledge {
...
```

**Recommendation:** Consider breaking down 'create_rust_patterns' into smaller, more focused methods. Current metrics: LOC=349, Statements=30, Complexity=1, Nesting=1

---

#### Long method 'create_rust_idioms' detected: 210 lines, 1 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/language_specific.rs`
- **Line:** 411

**Code:**
```
fn create_rust_idioms() -> Vec<LanguageIdiom> {
    vec![
        LanguageIdiom {
            name: "Builder Pattern".to_string(),
            description: CompressedString::new(
...
```

**Recommendation:** Consider breaking down 'create_rust_idioms' into smaller, more focused methods. Current metrics: LOC=210, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'create_python_patterns' detected: 149 lines, 16 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/language_specific.rs`
- **Line:** 778

**Code:**
```
fn create_python_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in Python context
    patterns.insert(
...
```

**Recommendation:** Consider breaking down 'create_python_patterns' into smaller, more focused methods. Current metrics: LOC=149, Statements=16, Complexity=1, Nesting=1

---

#### Long method 'create_javascript_patterns' detected: 157 lines, 16 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/language_specific.rs`
- **Line:** 1052

**Code:**
```
fn create_javascript_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in JavaScript context
    patterns.insert("god_object".to_string(), PatternKnowledge {
...
```

**Recommendation:** Consider breaking down 'create_javascript_patterns' into smaller, more focused methods. Current metrics: LOC=157, Statements=16, Complexity=1, Nesting=1

---

#### Long method 'create_typescript_patterns' detected: 153 lines, 16 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/language_specific.rs`
- **Line:** 1373

**Code:**
```
fn create_typescript_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // Any Type Abuse in TypeScript
    patterns.insert("any_type_abuse".to_string(), PatternKnowledge {
...
```

**Recommendation:** Consider breaking down 'create_typescript_patterns' into smaller, more focused methods. Current metrics: LOC=153, Statements=16, Complexity=1, Nesting=1

---

#### Long method 'create_java_patterns' detected: 166 lines, 16 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/language_specific.rs`
- **Line:** 1670

**Code:**
```
fn create_java_patterns() -> HashMap<String, PatternKnowledge> {
    let mut patterns = HashMap::new();

    // God Object in Java context
    patterns.insert("god_object".to_string(), PatternKnowledge {
...
```

**Recommendation:** Consider breaking down 'create_java_patterns' into smaller, more focused methods. Current metrics: LOC=166, Statements=16, Complexity=1, Nesting=1

---

#### Long method 'create_java_idioms' detected: 104 lines, 1 statements, complexity 1

- **File:** `./src/ai/knowledge/patterns/language_specific.rs`
- **Line:** 1845

**Code:**
```
fn create_java_idioms() -> Vec<LanguageIdiom> {
    vec![
        LanguageIdiom {
            name: "Builder Pattern".to_string(),
            description: CompressedString::new(
...
```

**Recommendation:** Consider breaking down 'create_java_idioms' into smaller, more focused methods. Current metrics: LOC=104, Statements=1, Complexity=1, Nesting=1

---

#### Large class 'KnowledgeLibraryBuilder' detected: 0 LOC, 16 methods, 3 fields

- **File:** `./src/ai/knowledge/build_integration.rs`
- **Line:** 33

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'KnowledgePerformanceMonitor' detected: 0 LOC, 17 methods, 4 fields

- **File:** `./src/ai/knowledge/performance.rs`
- **Line:** 269

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'IndexBuilder' detected: 0 LOC, 15 methods, 6 fields

- **File:** `./src/ai/knowledge/indexing.rs`
- **Line:** 153

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'KnowledgeLibraryLookup' detected: 0 LOC, 11 methods, 2 fields

- **File:** `./src/ai/knowledge/indexing.rs`
- **Line:** 520

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'KnowledgeValidator' detected: 0 LOC, 15 methods, 6 fields

- **File:** `./src/ai/knowledge/validation.rs`
- **Line:** 77

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'validate_content_quality' detected: 84 lines, 87 statements, complexity 10

- **File:** `./src/ai/knowledge/validation.rs`
- **Line:** 365

**Code:**
```
fn validate_content_quality(
        &self,
        library: &KnowledgeLibrary,
        issues: &mut Vec<ValidationIssue>,
    ) {
...
```

**Recommendation:** Consider breaking down 'validate_content_quality' into smaller, more focused methods. Current metrics: LOC=84, Statements=87, Complexity=10, Nesting=9

---

#### Long method 'validate_content_quality' detected: 84 lines, 87 statements, complexity 10

- **File:** `./src/ai/knowledge/validation.rs`
- **Line:** 365

**Code:**
```
fn validate_content_quality(
        &self,
        library: &KnowledgeLibrary,
        issues: &mut Vec<ValidationIssue>,
    ) {
...
```

**Recommendation:** Consider breaking down 'validate_content_quality' into smaller, more focused methods. Current metrics: LOC=84, Statements=87, Complexity=10, Nesting=9

---

#### Large class 'LanguageKnowledgeIntegrator' detected: 0 LOC, 15 methods, 2 fields

- **File:** `./src/ai/knowledge/language_integration.rs`
- **Line:** 56

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'KnowledgeLibraryLoader' detected: 0 LOC, 11 methods, 1 fields

- **File:** `./src/ai/knowledge/loader.rs`
- **Line:** 58

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'DynamicContextSelector' detected: 0 LOC, 24 methods, 4 fields

- **File:** `./src/ai/knowledge/context_selection.rs`
- **Line:** 13

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'new' detected: 205 lines, 30 statements, complexity 1

- **File:** `./src/ai/knowledge/compression.rs`
- **Line:** 336

**Code:**
```
pub fn new() -> Self {
        let programming_keywords = [
            // Rust keywords
            "fn",
            "let",
...
```

**Recommendation:** Consider breaking down 'new' into smaller, more focused methods. Current metrics: LOC=205, Statements=30, Complexity=1, Nesting=2

---

#### Long method 'new' detected: 205 lines, 30 statements, complexity 1

- **File:** `./src/ai/knowledge/compression.rs`
- **Line:** 336

**Code:**
```
pub fn new() -> Self {
        let programming_keywords = [
            // Rust keywords
            "fn",
            "let",
...
```

**Recommendation:** Consider breaking down 'new' into smaller, more focused methods. Current metrics: LOC=205, Statements=30, Complexity=1, Nesting=2

---

#### Large class 'SmartPromptBuilder' detected: 0 LOC, 11 methods, 4 fields

- **File:** `./src/ai/prompts/smart_prompting.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'EnhancedPromptTemplateSystem' detected: 0 LOC, 19 methods, 4 fields

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 18

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'register_base_templates' detected: 96 lines, 19 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 985

**Code:**
```
fn register_base_templates(&mut self) {
        // God Object Analysis Template
        self.base_templates.insert(
            AnalysisType::GodObjectAnalysis,
            BaseTemplate {
...
```

**Recommendation:** Consider breaking down 'register_base_templates' into smaller, more focused methods. Current metrics: LOC=96, Statements=19, Complexity=1, Nesting=1

---

#### Long method 'register_base_templates' detected: 96 lines, 19 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 985

**Code:**
```
fn register_base_templates(&mut self) {
        // God Object Analysis Template
        self.base_templates.insert(
            AnalysisType::GodObjectAnalysis,
            BaseTemplate {
...
```

**Recommendation:** Consider breaking down 'register_base_templates' into smaller, more focused methods. Current metrics: LOC=96, Statements=19, Complexity=1, Nesting=1

---

#### Long method 'register_tight_coupling_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1102

**Code:**
```
fn register_tight_coupling_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::TightCouplingAnalysis,
            BaseTemplate {
                id: "tight_coupling_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_tight_coupling_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'register_tight_coupling_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1102

**Code:**
```
fn register_tight_coupling_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::TightCouplingAnalysis,
            BaseTemplate {
                id: "tight_coupling_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_tight_coupling_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'register_dead_code_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1213

**Code:**
```
fn register_dead_code_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::DeadCodeAnalysis,
            BaseTemplate {
                id: "dead_code_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_dead_code_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'register_dead_code_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1213

**Code:**
```
fn register_dead_code_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::DeadCodeAnalysis,
            BaseTemplate {
                id: "dead_code_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_dead_code_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'register_performance_analysis_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1324

**Code:**
```
fn register_performance_analysis_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::PerformanceAnalysis,
            BaseTemplate {
                id: "performance_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_performance_analysis_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'register_performance_analysis_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1324

**Code:**
```
fn register_performance_analysis_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::PerformanceAnalysis,
            BaseTemplate {
                id: "performance_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_performance_analysis_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'register_general_analysis_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1435

**Code:**
```
fn register_general_analysis_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::GeneralAnalysis,
            BaseTemplate {
                id: "general_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_general_analysis_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'register_general_analysis_template' detected: 93 lines, 11 statements, complexity 1

- **File:** `./src/ai/prompts/enhanced_templates.rs`
- **Line:** 1435

**Code:**
```
fn register_general_analysis_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::GeneralAnalysis,
            BaseTemplate {
                id: "general_analysis".to_string(),
...
```

**Recommendation:** Consider breaking down 'register_general_analysis_template' into smaller, more focused methods. Current metrics: LOC=93, Statements=11, Complexity=1, Nesting=1

---

#### Long method 'get_system_memory_gb' detected: 35 lines, 42 statements, complexity 10

- **File:** `./src/ai/ollama_provider.rs`
- **Line:** 207

**Code:**
```
fn get_system_memory_gb() -> f64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs;

...
```

**Recommendation:** Consider breaking down 'get_system_memory_gb' into smaller, more focused methods. Current metrics: LOC=35, Statements=42, Complexity=10, Nesting=12

---

#### Long method 'get_system_memory_gb' detected: 35 lines, 42 statements, complexity 10

- **File:** `./src/ai/ollama_provider.rs`
- **Line:** 207

**Code:**
```
fn get_system_memory_gb() -> f64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs;

...
```

**Recommendation:** Consider breaking down 'get_system_memory_gb' into smaller, more focused methods. Current metrics: LOC=35, Statements=42, Complexity=10, Nesting=12

---

#### Large class 'HookManager' detected: 0 LOC, 13 methods, 3 fields

- **File:** `./src/hooks/manager.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PluginKnowledgeIntegrator' detected: 0 LOC, 20 methods, 4 fields

- **File:** `./src/plugins/integration.rs`
- **Line:** 213

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ExampleAntiPatternPlugin' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/plugins/development.rs`
- **Line:** 110

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'WasmPluginEngine' detected: 0 LOC, 15 methods, 6 fields

- **File:** `./src/plugins/engine.rs`
- **Line:** 20

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PluginMarketplace' detected: 0 LOC, 21 methods, 4 fields

- **File:** `./src/plugins/marketplace.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'MarketplacePlugin' detected: 0 LOC, 0 methods, 26 fields

- **File:** `./src/plugins/marketplace.rs`
- **Line:** 25

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'simulate_search_request' detected: 100 lines, 60 statements, complexity 14

- **File:** `./src/plugins/marketplace.rs`
- **Line:** 466

**Code:**
```
async fn simulate_search_request(&self, _url: &str, criteria: &PluginSearchCriteria) -> Result<PluginSearchResults> {
        // In a real implementation, this would make HTTP requests
        // For now, return mock data based on search criteria
        
        let mut mock_plugins = vec![
...
```

**Recommendation:** Consider breaking down 'simulate_search_request' into smaller, more focused methods. Current metrics: LOC=100, Statements=60, Complexity=14, Nesting=6

---

#### Long method 'simulate_search_request' detected: 100 lines, 60 statements, complexity 14

- **File:** `./src/plugins/marketplace.rs`
- **Line:** 466

**Code:**
```
async fn simulate_search_request(&self, _url: &str, criteria: &PluginSearchCriteria) -> Result<PluginSearchResults> {
        // In a real implementation, this would make HTTP requests
        // For now, return mock data based on search criteria
        
        let mut mock_plugins = vec![
...
```

**Recommendation:** Consider breaking down 'simulate_search_request' into smaller, more focused methods. Current metrics: LOC=100, Statements=60, Complexity=14, Nesting=6

---

#### Large class 'PluginTestingFramework' detected: 0 LOC, 19 methods, 4 fields

- **File:** `./src/plugins/testing_framework.rs`
- **Line:** 16

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'load_builtin_test_suites' detected: 105 lines, 21 statements, complexity 1

- **File:** `./src/plugins/testing_framework.rs`
- **Line:** 255

**Code:**
```
fn load_builtin_test_suites(&mut self) -> Result<()> {
        // Standard test suite
        let standard_suite = TestSuite {
            name: "standard".to_string(),
            description: "Standard plugin validation test suite".to_string(),
...
```

**Recommendation:** Consider breaking down 'load_builtin_test_suites' into smaller, more focused methods. Current metrics: LOC=105, Statements=21, Complexity=1, Nesting=1

---

#### Long method 'load_builtin_test_suites' detected: 105 lines, 21 statements, complexity 1

- **File:** `./src/plugins/testing_framework.rs`
- **Line:** 255

**Code:**
```
fn load_builtin_test_suites(&mut self) -> Result<()> {
        // Standard test suite
        let standard_suite = TestSuite {
            name: "standard".to_string(),
            description: "Standard plugin validation test suite".to_string(),
...
```

**Recommendation:** Consider breaking down 'load_builtin_test_suites' into smaller, more focused methods. Current metrics: LOC=105, Statements=21, Complexity=1, Nesting=1

---

#### Long method 'print_test_results' detected: 59 lines, 85 statements, complexity 16

- **File:** `./src/plugins/testing_framework.rs`
- **Line:** 790

**Code:**
```
fn print_test_results(&self, result: &PluginTestResult) {
        println!("\n📊 Test Results Summary");
        println!("========================");
        println!("Plugin: {}", result.plugin_id);
        println!("Test Suite: {}", result.test_suite);
...
```

**Recommendation:** Consider breaking down 'print_test_results' into smaller, more focused methods. Current metrics: LOC=59, Statements=85, Complexity=16, Nesting=6

---

#### Long method 'print_test_results' detected: 59 lines, 85 statements, complexity 16

- **File:** `./src/plugins/testing_framework.rs`
- **Line:** 790

**Code:**
```
fn print_test_results(&self, result: &PluginTestResult) {
        println!("\n📊 Test Results Summary");
        println!("========================");
        println!("Plugin: {}", result.plugin_id);
        println!("Test Suite: {}", result.test_suite);
...
```

**Recommendation:** Consider breaking down 'print_test_results' into smaller, more focused methods. Current metrics: LOC=59, Statements=85, Complexity=16, Nesting=6

---

#### Large class 'PluginRegistry' detected: 0 LOC, 14 methods, 3 fields

- **File:** `./src/plugins/registry.rs`
- **Line:** 17

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PluginKnowledgeLibrary' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/plugins/knowledge.rs`
- **Line:** 723

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PluginTemplateGenerator' detected: 0 LOC, 15 methods, 2 fields

- **File:** `./src/plugins/template_generator.rs`
- **Line:** 14

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'PluginRuntime' detected: 0 LOC, 13 methods, 4 fields

- **File:** `./src/plugins/runtime.rs`
- **Line:** 15

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SecurityPolicy' detected: 0 LOC, 15 methods, 6 fields

- **File:** `./src/plugins/security.rs`
- **Line:** 12

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'enhance_cli_error' detected: 73 lines, 99 statements, complexity 9

- **File:** `./src/cli/enhanced_help.rs`
- **Line:** 128

**Code:**
```
pub fn enhance_cli_error(original_error: &str, _command_context: Option<&str>) -> CliError {
    let lower_error = original_error.to_lowercase();

    // Handle "command not found" errors
    if lower_error.contains("no such subcommand") || lower_error.contains("unrecognized subcommand")
...
```

**Recommendation:** Consider breaking down 'enhance_cli_error' into smaller, more focused methods. Current metrics: LOC=73, Statements=99, Complexity=9, Nesting=6

---

#### Long method 'generate_topic_help' detected: 393 lines, 20 statements, complexity 12

- **File:** `./src/cli/enhanced_help.rs`
- **Line:** 295

**Code:**
```
pub fn generate_topic_help(topic: &str) -> Option<String> {
    match topic {
        "installation" => Some(r#"
📦 Installation Help

...
```

**Recommendation:** Consider breaking down 'generate_topic_help' into smaller, more focused methods. Current metrics: LOC=393, Statements=20, Complexity=12, Nesting=3

---

#### Long method 'serve' detected: 90 lines, 90 statements, complexity 12

- **File:** `./src/cli/ui_command.rs`
- **Line:** 102

**Code:**
```
async fn serve(
        &self,
        args: &ServeArgs,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🚀 Starting Uveddi Interactive Reports Server");
...
```

**Recommendation:** Consider breaking down 'serve' into smaller, more focused methods. Current metrics: LOC=90, Statements=90, Complexity=12, Nesting=7

---

#### Long method 'serve' detected: 90 lines, 90 statements, complexity 12

- **File:** `./src/cli/ui_command.rs`
- **Line:** 102

**Code:**
```
async fn serve(
        &self,
        args: &ServeArgs,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🚀 Starting Uveddi Interactive Reports Server");
...
```

**Recommendation:** Consider breaking down 'serve' into smaller, more focused methods. Current metrics: LOC=90, Statements=90, Complexity=12, Nesting=7

---

#### Large class 'AnalyzeCommand' detected: 0 LOC, 16 methods, 41 fields

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 122

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'generate_markdown_report' detected: 62 lines, 101 statements, complexity 16

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 1119

**Code:**
```
async fn generate_markdown_report(
        &self,
        report: &crate::application::orchestrator::AnalysisResult,
        output_path: &std::path::Path,
    ) -> Result<(), UveddiError> {
...
```

**Recommendation:** Consider breaking down 'generate_markdown_report' into smaller, more focused methods. Current metrics: LOC=62, Statements=101, Complexity=16, Nesting=10

---

#### Long method 'generate_markdown_report' detected: 62 lines, 101 statements, complexity 16

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 1119

**Code:**
```
async fn generate_markdown_report(
        &self,
        report: &crate::application::orchestrator::AnalysisResult,
        output_path: &std::path::Path,
    ) -> Result<(), UveddiError> {
...
```

**Recommendation:** Consider breaking down 'generate_markdown_report' into smaller, more focused methods. Current metrics: LOC=62, Statements=101, Complexity=16, Nesting=10

---

#### Long method 'discover_files_recursive' detected: 68 lines, 82 statements, complexity 14

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 1223

**Code:**
```
pub fn discover_files_recursive(
        dir: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, SecurityError> {
        use ignore::WalkBuilder;
        use std::collections::HashSet;
...
```

**Recommendation:** Consider breaking down 'discover_files_recursive' into smaller, more focused methods. Current metrics: LOC=68, Statements=82, Complexity=14, Nesting=11

---

#### Long method 'discover_files_recursive' detected: 68 lines, 82 statements, complexity 14

- **File:** `./src/cli/commands/analyze.rs`
- **Line:** 1223

**Code:**
```
pub fn discover_files_recursive(
        dir: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, SecurityError> {
        use ignore::WalkBuilder;
        use std::collections::HashSet;
...
```

**Recommendation:** Consider breaking down 'discover_files_recursive' into smaller, more focused methods. Current metrics: LOC=68, Statements=82, Complexity=14, Nesting=11

---

#### Long method 'test_hook' detected: 75 lines, 77 statements, complexity 15

- **File:** `./src/cli/commands/hooks.rs`
- **Line:** 373

**Code:**
```
async fn test_hook(
        &self,
        repo_path: PathBuf,
        hook_type: TestHookType,
        files: Option<Vec<PathBuf>>,
...
```

**Recommendation:** Consider breaking down 'test_hook' into smaller, more focused methods. Current metrics: LOC=75, Statements=77, Complexity=15, Nesting=8

---

#### Long method 'test_hook' detected: 75 lines, 77 statements, complexity 15

- **File:** `./src/cli/commands/hooks.rs`
- **Line:** 373

**Code:**
```
async fn test_hook(
        &self,
        repo_path: PathBuf,
        hook_type: TestHookType,
        files: Option<Vec<PathBuf>>,
...
```

**Recommendation:** Consider breaking down 'test_hook' into smaller, more focused methods. Current metrics: LOC=75, Statements=77, Complexity=15, Nesting=8

---

#### Large class 'InitCommand' detected: 0 LOC, 16 methods, 5 fields

- **File:** `./src/cli/commands/init.rs`
- **Line:** 15

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'InteractiveReportGenerator' detected: 0 LOC, 14 methods, 2 fields

- **File:** `./src/report/interactive_generator.rs`
- **Line:** 62

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'generate_demo_report' detected: 123 lines, 32 statements, complexity 1

- **File:** `./src/report/interactive_generator.rs`
- **Line:** 256

**Code:**
```
pub fn generate_demo_report() -> InteractiveReport {
        let mut demo = InteractiveReport::default();

        // Enhance demo data with more realistic content
        demo.project.name = "Demo Rust Project".to_string();
...
```

**Recommendation:** Consider breaking down 'generate_demo_report' into smaller, more focused methods. Current metrics: LOC=123, Statements=32, Complexity=1, Nesting=1

---

#### Long method 'generate_demo_report' detected: 123 lines, 32 statements, complexity 1

- **File:** `./src/report/interactive_generator.rs`
- **Line:** 256

**Code:**
```
pub fn generate_demo_report() -> InteractiveReport {
        let mut demo = InteractiveReport::default();

        // Enhance demo data with more realistic content
        demo.project.name = "Demo Rust Project".to_string();
...
```

**Recommendation:** Consider breaking down 'generate_demo_report' into smaller, more focused methods. Current metrics: LOC=123, Statements=32, Complexity=1, Nesting=1

---

#### Large class 'ModernReportGenerator' detected: 0 LOC, 18 methods, 3 fields

- **File:** `./src/report/modern_generator.rs`
- **Line:** 42

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'resolve_templates_with_fallback' detected: 70 lines, 86 statements, complexity 14

- **File:** `./src/report/modern_generator.rs`
- **Line:** 73

**Code:**
```
fn resolve_templates_with_fallback() -> Result<Tera, ModernReportError> {
        use crate::core::logging::{info, warn};
        use std::env;
        use std::path::PathBuf;

...
```

**Recommendation:** Consider breaking down 'resolve_templates_with_fallback' into smaller, more focused methods. Current metrics: LOC=70, Statements=86, Complexity=14, Nesting=9

---

#### Long method 'resolve_templates_with_fallback' detected: 70 lines, 86 statements, complexity 14

- **File:** `./src/report/modern_generator.rs`
- **Line:** 73

**Code:**
```
fn resolve_templates_with_fallback() -> Result<Tera, ModernReportError> {
        use crate::core::logging::{info, warn};
        use std::env;
        use std::path::PathBuf;

...
```

**Recommendation:** Consider breaking down 'resolve_templates_with_fallback' into smaller, more focused methods. Current metrics: LOC=70, Statements=86, Complexity=14, Nesting=9

---

#### Large class 'ImageRenderer' detected: 0 LOC, 13 methods, 2 fields

- **File:** `./src/report/image_renderer.rs`
- **Line:** 58

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'MarkdownReportGenerator' detected: 0 LOC, 23 methods, 2 fields

- **File:** `./src/report/markdown_generator.rs`
- **Line:** 77

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'generate_diagram_content' detected: 89 lines, 47 statements, complexity 11

- **File:** `./src/report/markdown_generator.rs`
- **Line:** 159

**Code:**
```
async fn generate_diagram_content(
        &self,
        title: &str,
        mermaid_code: &str,
        diagram_id: &str,
...
```

**Recommendation:** Consider breaking down 'generate_diagram_content' into smaller, more focused methods. Current metrics: LOC=89, Statements=47, Complexity=11, Nesting=11

---

#### Long method 'generate_diagram_content' detected: 89 lines, 47 statements, complexity 11

- **File:** `./src/report/markdown_generator.rs`
- **Line:** 159

**Code:**
```
async fn generate_diagram_content(
        &self,
        title: &str,
        mermaid_code: &str,
        diagram_id: &str,
...
```

**Recommendation:** Consider breaking down 'generate_diagram_content' into smaller, more focused methods. Current metrics: LOC=89, Statements=47, Complexity=11, Nesting=11

---

#### Long method 'generate_issues_breakdown' detected: 55 lines, 63 statements, complexity 18

- **File:** `./src/report/markdown_generator.rs`
- **Line:** 551

**Code:**
```
fn generate_issues_breakdown(
        &self,
        issues: &[ArchitecturalIssue],
        _anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
...
```

**Recommendation:** Consider breaking down 'generate_issues_breakdown' into smaller, more focused methods. Current metrics: LOC=55, Statements=63, Complexity=18, Nesting=12

---

#### Long method 'generate_issues_breakdown' detected: 55 lines, 63 statements, complexity 18

- **File:** `./src/report/markdown_generator.rs`
- **Line:** 551

**Code:**
```
fn generate_issues_breakdown(
        &self,
        issues: &[ArchitecturalIssue],
        _anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
...
```

**Recommendation:** Consider breaking down 'generate_issues_breakdown' into smaller, more focused methods. Current metrics: LOC=55, Statements=63, Complexity=18, Nesting=12

---

#### Long method 'generate_recommendations' detected: 89 lines, 94 statements, complexity 9

- **File:** `./src/report/markdown_generator.rs`
- **Line:** 691

**Code:**
```
fn generate_recommendations(
        &self,
        issues: &[ArchitecturalIssue],
        ai_insights: Option<&[AiInsight]>,
    ) -> String {
...
```

**Recommendation:** Consider breaking down 'generate_recommendations' into smaller, more focused methods. Current metrics: LOC=89, Statements=94, Complexity=9, Nesting=7

---

#### Long method 'generate_recommendations' detected: 89 lines, 94 statements, complexity 9

- **File:** `./src/report/markdown_generator.rs`
- **Line:** 691

**Code:**
```
fn generate_recommendations(
        &self,
        issues: &[ArchitecturalIssue],
        ai_insights: Option<&[AiInsight]>,
    ) -> String {
...
```

**Recommendation:** Consider breaking down 'generate_recommendations' into smaller, more focused methods. Current metrics: LOC=89, Statements=94, Complexity=9, Nesting=7

---

#### Long method 'generate_html_head' detected: 209 lines, 1 statements, complexity 1

- **File:** `./src/report/html_generator.rs`
- **Line:** 70

**Code:**
```
pub fn generate_html_head(&self) -> String {
        format!(
            r#"
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
...
```

**Recommendation:** Consider breaking down 'generate_html_head' into smaller, more focused methods. Current metrics: LOC=209, Statements=1, Complexity=1, Nesting=1

---

#### Long method 'generate_html_head' detected: 209 lines, 1 statements, complexity 1

- **File:** `./src/report/html_generator.rs`
- **Line:** 70

**Code:**
```
pub fn generate_html_head(&self) -> String {
        format!(
            r#"
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
...
```

**Recommendation:** Consider breaking down 'generate_html_head' into smaller, more focused methods. Current metrics: LOC=209, Statements=1, Complexity=1, Nesting=1

---

#### Large class 'ReportSecurityValidator' detected: 0 LOC, 13 methods, 1 fields

- **File:** `./src/report/security.rs`
- **Line:** 70

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'Finding' detected: 0 LOC, 0 methods, 17 fields

- **File:** `./src/report/interactive_models.rs`
- **Line:** 124

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'build_dependency_graph' detected: 88 lines, 72 statements, complexity 7

- **File:** `./src/report/interactive_models.rs`
- **Line:** 855

**Code:**
```
fn build_dependency_graph(
        components: Option<&[ArchitecturalComponent]>,
        dependencies: &[Dependency],
    ) -> DependencyGraph {
        let mut nodes = Vec::new();
...
```

**Recommendation:** Consider breaking down 'build_dependency_graph' into smaller, more focused methods. Current metrics: LOC=88, Statements=72, Complexity=7, Nesting=7

---

#### Long method 'build_dependency_graph' detected: 88 lines, 72 statements, complexity 7

- **File:** `./src/report/interactive_models.rs`
- **Line:** 855

**Code:**
```
fn build_dependency_graph(
        components: Option<&[ArchitecturalComponent]>,
        dependencies: &[Dependency],
    ) -> DependencyGraph {
        let mut nodes = Vec::new();
...
```

**Recommendation:** Consider breaking down 'build_dependency_graph' into smaller, more focused methods. Current metrics: LOC=88, Statements=72, Complexity=7, Nesting=7

---

#### Large class 'ReportGenerator' detected: 0 LOC, 15 methods, 10 fields

- **File:** `./src/report/generators/orchestrator.rs`
- **Line:** 23

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ProgressTracker' detected: 0 LOC, 19 methods, 4 fields

- **File:** `./src/application/services/progress_tracker.rs`
- **Line:** 17

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ServiceRegistry' detected: 0 LOC, 14 methods, 3 fields

- **File:** `./src/application/services/registry.rs`
- **Line:** 47

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'FileProcessor' detected: 0 LOC, 16 methods, 4 fields

- **File:** `./src/application/services/file_processor.rs`
- **Line:** 19

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'process_files' detected: 69 lines, 76 statements, complexity 7

- **File:** `./src/application/services/file_processor.rs`
- **Line:** 207

**Code:**
```
pub async fn process_files(
        &mut self,
        files: Vec<FileInfo>,
    ) -> Result<ProcessingResult, UveddiError> {
        if !self.is_running {
...
```

**Recommendation:** Consider breaking down 'process_files' into smaller, more focused methods. Current metrics: LOC=69, Statements=76, Complexity=7, Nesting=8

---

#### Long method 'process_files' detected: 69 lines, 76 statements, complexity 7

- **File:** `./src/application/services/file_processor.rs`
- **Line:** 207

**Code:**
```
pub async fn process_files(
        &mut self,
        files: Vec<FileInfo>,
    ) -> Result<ProcessingResult, UveddiError> {
        if !self.is_running {
...
```

**Recommendation:** Consider breaking down 'process_files' into smaller, more focused methods. Current metrics: LOC=69, Statements=76, Complexity=7, Nesting=8

---

#### Long method 'discover_files_recursive' detected: 60 lines, 72 statements, complexity 13

- **File:** `./src/application/services/file_processor.rs`
- **Line:** 302

**Code:**
```
async fn discover_files_recursive(
        path: &Path,
        config: &FileProcessorConfig,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<Vec<FileInfo>, UveddiError> {
...
```

**Recommendation:** Consider breaking down 'discover_files_recursive' into smaller, more focused methods. Current metrics: LOC=60, Statements=72, Complexity=13, Nesting=10

---

#### Long method 'discover_files_recursive' detected: 60 lines, 72 statements, complexity 13

- **File:** `./src/application/services/file_processor.rs`
- **Line:** 302

**Code:**
```
async fn discover_files_recursive(
        path: &Path,
        config: &FileProcessorConfig,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<Vec<FileInfo>, UveddiError> {
...
```

**Recommendation:** Consider breaking down 'discover_files_recursive' into smaller, more focused methods. Current metrics: LOC=60, Statements=72, Complexity=13, Nesting=10

---

#### Large class 'LegacyAnalysisConfig' detected: 0 LOC, 0 methods, 24 fields

- **File:** `./src/application/mod_verbose.rs`
- **Line:** 42

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ConfigValidator' detected: 0 LOC, 12 methods, 0 fields

- **File:** `./src/application/configuration/validation.rs`
- **Line:** 7

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AiConfig' detected: 0 LOC, 11 methods, 3 fields

- **File:** `./src/application/configuration/ai_config.rs`
- **Line:** 7

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AiWorkflow' detected: 0 LOC, 15 methods, 4 fields

- **File:** `./src/application/workflows/ai_workflow.rs`
- **Line:** 19

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'execute_ai_analysis' detected: 58 lines, 78 statements, complexity 13

- **File:** `./src/application/workflows/ai_workflow.rs`
- **Line:** 255

**Code:**
```
async fn execute_ai_analysis(
        &mut self,
        issues: &[ArchitecturalIssue],
        metrics: &mut AiMetrics,
    ) -> Result<Vec<AiInsight>, UveddiError> {
...
```

**Recommendation:** Consider breaking down 'execute_ai_analysis' into smaller, more focused methods. Current metrics: LOC=58, Statements=78, Complexity=13, Nesting=9

---

#### Long method 'execute_ai_analysis' detected: 58 lines, 78 statements, complexity 13

- **File:** `./src/application/workflows/ai_workflow.rs`
- **Line:** 255

**Code:**
```
async fn execute_ai_analysis(
        &mut self,
        issues: &[ArchitecturalIssue],
        metrics: &mut AiMetrics,
    ) -> Result<Vec<AiInsight>, UveddiError> {
...
```

**Recommendation:** Consider breaking down 'execute_ai_analysis' into smaller, more focused methods. Current metrics: LOC=58, Statements=78, Complexity=13, Nesting=9

---

#### Large class 'ReportWorkflow' detected: 0 LOC, 14 methods, 4 fields

- **File:** `./src/application/workflows/report_workflow.rs`
- **Line:** 21

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AnalysisWorkflow' detected: 0 LOC, 14 methods, 6 fields

- **File:** `./src/application/workflows/analysis_workflow.rs`
- **Line:** 17

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'StartupManager' detected: 0 LOC, 17 methods, 7 fields

- **File:** `./src/application/startup.rs`
- **Line:** 21

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'AnalysisOrchestrator' detected: 0 LOC, 19 methods, 4 fields

- **File:** `./src/application/orchestrator.rs`
- **Line:** 37

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'ApplicationPluginManager' detected: 0 LOC, 11 methods, 5 fields

- **File:** `./src/application/plugin_manager.rs`
- **Line:** 22

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Large class 'SmartConfigValidator' detected: 0 LOC, 17 methods, 1 fields

- **File:** `./src/config/validation.rs`
- **Line:** 106

**Recommendation:** Consider breaking this class into smaller, more focused classes

---

#### Long method 'validate_large_classes_config' detected: 84 lines, 54 statements, complexity 14

- **File:** `./src/config/validation.rs`
- **Line:** 364

**Code:**
```
fn validate_large_classes_config(
        &self,
        config: &Config,
        _errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
...
```

**Recommendation:** Consider breaking down 'validate_large_classes_config' into smaller, more focused methods. Current metrics: LOC=84, Statements=54, Complexity=14, Nesting=11

---

#### Long method 'validate_large_classes_config' detected: 84 lines, 54 statements, complexity 14

- **File:** `./src/config/validation.rs`
- **Line:** 364

**Code:**
```
fn validate_large_classes_config(
        &self,
        config: &Config,
        _errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
...
```

**Recommendation:** Consider breaking down 'validate_large_classes_config' into smaller, more focused methods. Current metrics: LOC=84, Statements=54, Complexity=14, Nesting=11

---

