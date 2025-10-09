# Assignment 18: Extract Core Interfaces

## Priority: HIGH
## Estimated Time: 4-5 hours
## Dependencies: Phases 1-3 (God Object elimination, Feature flags, Dependencies)

## Objective
Extract and define clear interfaces for all major components to enable dependency inversion and improve testability.

## Current Problem
- Components directly depend on concrete implementations
- Difficult to test components in isolation
- Tight coupling prevents component substitution
- No clear contracts between modules

## Tasks

### 1. Identify Interface Extraction Opportunities

#### A. Audit Current Concrete Dependencies:
```bash
# Find direct struct usage that should be interfaces
rg "use crate::[^:]*::\w+;" src/ --type rust | head -20

# Find struct instantiations
rg "::new\(\)" src/ --type rust

# Find direct method calls that could be abstracted
rg "\.\w+\(" src/ --type rust | head -30
```

#### B. Map Component Interactions:
```markdown
# Component Interaction Analysis

## Major Components Needing Interfaces:
1. Analysis Engine -> AnalysisEngine trait
2. Report Generator -> ReportGenerator trait
3. Database Access -> Repository traits
4. Configuration -> ConfigProvider trait
5. Logging -> Logger trait
6. Cache -> CacheProvider trait
7. File System -> FileSystem trait
8. Plugin System -> PluginManager trait

## Interface Priority:
- High: Analysis, Database, Configuration
- Medium: Reporting, Caching, Logging
- Low: File System, Plugins
```

### 2. Design Core Interface Architecture

#### A. Create Interface Module Structure:
```
src/interfaces/
├── mod.rs (re-exports all interfaces)
├── analysis.rs (analysis engine interfaces)
├── persistence.rs (database/storage interfaces)
├── reporting.rs (report generation interfaces)
├── configuration.rs (config provider interfaces)
├── caching.rs (cache provider interfaces)
├── logging.rs (logging interfaces)
├── filesystem.rs (file system interfaces)
└── plugins.rs (plugin manager interfaces)
```

#### B. Define Interface Design Principles:
```rust
// src/interfaces/mod.rs
//! Core interfaces for dependency inversion
//!
//! Design principles:
//! 1. Interfaces are async-first
//! 2. All methods return Result types
//! 3. Interfaces are minimal and focused
//! 4. No implementation details leak through
//! 5. Easy to mock for testing

use crate::error::Result;
use async_trait::async_trait;

// Common interface patterns
pub trait ServiceHealth {
    async fn health_check(&self) -> Result<HealthStatus>;
}

pub trait ServiceInfo {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}
```

### 3. Extract Analysis Engine Interface

#### A. Define Analysis Interface:
```rust
// src/interfaces/analysis.rs
use crate::types::*;
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait AnalysisEngine: Send + Sync {
    /// Analyze a complete project
    async fn analyze_project(&self, config: &AnalysisConfig) -> Result<AnalysisResult>;

    /// Analyze a single file
    async fn analyze_file(&self, file_path: &Path) -> Result<FileAnalysis>;

    /// Get supported languages
    fn supported_languages(&self) -> &[Language];

    /// Get available detectors
    fn available_detectors(&self) -> Vec<DetectorInfo>;

    /// Check if language is supported
    fn supports_language(&self, language: &Language) -> bool {
        self.supported_languages().contains(language)
    }
}

#[async_trait]
pub trait Detector: Send + Sync {
    /// Detector metadata
    fn metadata(&self) -> DetectorMetadata;

    /// Run detection on analysis context
    async fn detect(&self, context: &AnalysisContext) -> Result<Vec<Finding>>;

    /// Languages this detector supports
    fn supported_languages(&self) -> &[Language];

    /// Detector category
    fn category(&self) -> DetectorCategory;
}

#[async_trait]
pub trait LanguageParser: Send + Sync {
    /// Parse source code into AST
    async fn parse(&self, source: &str) -> Result<SyntaxTree>;

    /// Extract symbols from AST
    fn extract_symbols(&self, tree: &SyntaxTree) -> Result<Vec<Symbol>>;

    /// Build dependency graph
    fn build_dependencies(&self, tree: &SyntaxTree) -> Result<Vec<Dependency>>;

    /// Language this parser handles
    fn language(&self) -> Language;
}

// Supporting types
#[derive(Debug, Clone)]
pub struct DetectorMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: DetectorCategory,
}

#[derive(Debug, Clone)]
pub struct DetectorInfo {
    pub metadata: DetectorMetadata,
    pub enabled: bool,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum DetectorCategory {
    Security,
    Performance,
    Maintainability,
    CodeQuality,
    AntiPatterns,
}
```

#### B. Create Analysis Context Interface:
```rust
// src/interfaces/analysis.rs (continued)
pub trait AnalysisContext {
    fn file_path(&self) -> &Path;
    fn source_code(&self) -> &str;
    fn language(&self) -> Language;
    fn syntax_tree(&self) -> Option<&SyntaxTree>;
    fn symbols(&self) -> &[Symbol];
    fn dependencies(&self) -> &[Dependency];
    fn project_context(&self) -> &ProjectContext;
}

pub struct DefaultAnalysisContext {
    file_path: PathBuf,
    source_code: String,
    language: Language,
    syntax_tree: Option<SyntaxTree>,
    symbols: Vec<Symbol>,
    dependencies: Vec<Dependency>,
    project_context: ProjectContext,
}

impl AnalysisContext for DefaultAnalysisContext {
    fn file_path(&self) -> &Path {
        &self.file_path
    }

    fn source_code(&self) -> &str {
        &self.source_code
    }

    fn language(&self) -> Language {
        self.language
    }

    fn syntax_tree(&self) -> Option<&SyntaxTree> {
        self.syntax_tree.as_ref()
    }

    fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    fn project_context(&self) -> &ProjectContext {
        &self.project_context
    }
}
```

### 4. Extract Repository Interfaces

#### A. Define Database Interfaces:
```rust
// src/interfaces/persistence.rs
use crate::types::*;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait AnalysisRepository: Send + Sync {
    async fn save(&self, analysis: &AnalysisResult) -> Result<AnalysisId>;
    async fn load(&self, id: &AnalysisId) -> Result<AnalysisResult>;
    async fn list(&self, filter: &AnalysisFilter) -> Result<Vec<AnalysisMetadata>>;
    async fn delete(&self, id: &AnalysisId) -> Result<()>;
    async fn update(&self, id: &AnalysisId, analysis: &AnalysisResult) -> Result<()>;
}

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn save_project(&self, project: &Project) -> Result<ProjectId>;
    async fn load_project(&self, id: &ProjectId) -> Result<Project>;
    async fn list_projects(&self) -> Result<Vec<ProjectMetadata>>;
    async fn delete_project(&self, id: &ProjectId) -> Result<()>;
}

#[async_trait]
pub trait CacheRepository: Send + Sync {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned;

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: serde::Serialize;

    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    async fn execute(&self, query: &str) -> Result<()>;
    async fn query<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned;
    async fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&dyn DatabaseConnection) -> Result<R> + Send;
}

// Supporting types
pub type AnalysisId = String;
pub type ProjectId = String;

#[derive(Debug, Clone)]
pub struct AnalysisFilter {
    pub project_id: Option<ProjectId>,
    pub language: Option<Language>,
    pub date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AnalysisMetadata {
    pub id: AnalysisId,
    pub project_id: ProjectId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub language: Language,
    pub file_count: usize,
    pub finding_count: usize,
}
```

### 5. Extract Configuration Interface

#### A. Define Configuration Provider:
```rust
// src/interfaces/configuration.rs
use crate::types::*;
use crate::error::Result;
use std::path::Path;

pub trait ConfigProvider: Send + Sync {
    fn get_analysis_config(&self) -> &AnalysisConfig;
    fn get_database_config(&self) -> &DatabaseConfig;
    fn get_logging_config(&self) -> &LoggingConfig;
    fn get_cache_config(&self) -> &CacheConfig;

    #[cfg(feature = "ai-integration")]
    fn get_ai_config(&self) -> Option<&AiConfig>;

    #[cfg(feature = "web-dashboard")]
    fn get_web_config(&self) -> Option<&WebConfig>;

    fn validate(&self) -> Result<()>;
    fn reload(&mut self) -> Result<()>;
}

pub trait ConfigLoader: Send + Sync {
    fn load_from_file(&self, path: &Path) -> Result<Box<dyn ConfigProvider>>;
    fn load_from_env(&self) -> Result<Box<dyn ConfigProvider>>;
    fn load_from_args(&self, args: &[String]) -> Result<Box<dyn ConfigProvider>>;
}

pub trait ConfigValidator: Send + Sync {
    fn validate_analysis_config(&self, config: &AnalysisConfig) -> Result<()>;
    fn validate_database_config(&self, config: &DatabaseConfig) -> Result<()>;
    fn validate_paths(&self, config: &AnalysisConfig) -> Result<()>;
}

// Configuration merging
pub trait ConfigMerger {
    fn merge(&self, base: Box<dyn ConfigProvider>, override_config: Box<dyn ConfigProvider>)
        -> Result<Box<dyn ConfigProvider>>;
}

// Environment-specific configs
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub target_path: PathBuf,
    pub output_format: OutputFormat,
    pub output_path: Option<PathBuf>,
    pub languages: Vec<Language>,
    pub detectors: Vec<String>,
    pub parallel_analysis: bool,
    pub max_file_size: usize,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub migration_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Plain,
    Json,
    Structured,
}

#[derive(Debug, Clone)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File,
    Combined,
}
```

### 6. Extract Reporting Interface

#### A. Define Report Generation Interface:
```rust
// src/interfaces/reporting.rs
use crate::types::*;
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait ReportGenerator: Send + Sync {
    async fn generate(&self, analysis: &AnalysisResult, format: &ReportFormat) -> Result<String>;
    async fn export(&self, content: &str, path: &Path) -> Result<()>;
    fn supported_formats(&self) -> &[ReportFormat];
}

#[async_trait]
pub trait TemplateEngine: Send + Sync {
    async fn render(&self, template: &str, context: &serde_json::Value) -> Result<String>;
    fn register_helper(&mut self, name: &str, helper: Box<dyn TemplateHelper>);
}

pub trait TemplateHelper: Send + Sync {
    fn apply(&self, input: &serde_json::Value) -> Result<serde_json::Value>;
}

#[async_trait]
pub trait ReportFormatter: Send + Sync {
    async fn format_findings(&self, findings: &[Finding]) -> Result<String>;
    async fn format_metrics(&self, metrics: &AnalysisMetrics) -> Result<String>;
    async fn format_summary(&self, summary: &AnalysisSummary) -> Result<String>;
    fn format_type(&self) -> ReportFormat;
}

// HTML-specific interface
#[async_trait]
pub trait HtmlReportGenerator: ReportGenerator {
    async fn generate_interactive(&self, analysis: &AnalysisResult) -> Result<String>;
    async fn generate_static(&self, analysis: &AnalysisResult) -> Result<String>;
    fn include_charts(&mut self, include: bool);
    fn set_theme(&mut self, theme: HtmlTheme);
}

// Chart generation interface
pub trait ChartGenerator: Send + Sync {
    fn generate_metrics_chart(&self, metrics: &AnalysisMetrics) -> Result<String>;
    fn generate_trend_chart(&self, data: &[TrendDataPoint]) -> Result<String>;
    fn generate_dependency_graph(&self, dependencies: &[Dependency]) -> Result<String>;
}

#[derive(Debug, Clone)]
pub enum ReportFormat {
    Html,
    Markdown,
    Json,
    Xml,
    Pdf,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum HtmlTheme {
    Light,
    Dark,
    HighContrast,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct TrendDataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub label: String,
}
```

### 7. Extract Logging Interface

#### A. Define Logging Interface:
```rust
// src/interfaces/logging.rs
use crate::error::Result;
use serde_json::Value;
use std::fmt::Display;

pub trait Logger: Send + Sync {
    fn trace(&self, message: &str);
    fn debug(&self, message: &str);
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);

    fn trace_with_context(&self, message: &str, context: &Value);
    fn debug_with_context(&self, message: &str, context: &Value);
    fn info_with_context(&self, message: &str, context: &Value);
    fn warn_with_context(&self, message: &str, context: &Value);
    fn error_with_context(&self, message: &str, context: &Value);
}

pub trait StructuredLogger: Logger {
    fn log_analysis_start(&self, config: &AnalysisConfig);
    fn log_analysis_complete(&self, result: &AnalysisResult);
    fn log_detector_execution(&self, detector: &str, duration: Duration);
    fn log_performance_metrics(&self, metrics: &PerformanceMetrics);
}

pub trait LogContext {
    fn with_field(&self, key: &str, value: &Value) -> Box<dyn LogContext>;
    fn with_target(&self, target: &str) -> Box<dyn LogContext>;
    fn with_span(&self, span: &str) -> Box<dyn LogContext>;
}

// Specialized logging for different domains
pub trait AnalysisLogger: Send + Sync {
    fn log_file_analysis_start(&self, file_path: &Path);
    fn log_file_analysis_complete(&self, file_path: &Path, duration: Duration);
    fn log_detector_finding(&self, detector: &str, finding: &Finding);
    fn log_parsing_error(&self, file_path: &Path, error: &str);
}

pub trait PerformanceLogger: Send + Sync {
    fn log_timing(&self, operation: &str, duration: Duration);
    fn log_memory_usage(&self, operation: &str, bytes: u64);
    fn log_cache_hit(&self, key: &str);
    fn log_cache_miss(&self, key: &str);
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub analysis_duration: Duration,
    pub files_analyzed: usize,
    pub memory_peak: u64,
    pub cache_hit_rate: f64,
}
```

### 8. Extract File System Interface

#### A. Define File System Interface:
```rust
// src/interfaces/filesystem.rs
use crate::error::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn read_to_string(&self, path: &Path) -> Result<String>;
    async fn read_to_bytes(&self, path: &Path) -> Result<Vec<u8>>;
    async fn write_string(&self, path: &Path, content: &str) -> Result<()>;
    async fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;

    async fn exists(&self, path: &Path) -> bool;
    async fn is_file(&self, path: &Path) -> bool;
    async fn is_directory(&self, path: &Path) -> bool;

    async fn create_dir_all(&self, path: &Path) -> Result<()>;
    async fn remove_file(&self, path: &Path) -> Result<()>;
    async fn remove_dir_all(&self, path: &Path) -> Result<()>;

    async fn list_files(&self, dir: &Path, recursive: bool) -> Result<Vec<PathBuf>>;
    async fn list_files_with_extension(&self, dir: &Path, extension: &str) -> Result<Vec<PathBuf>>;
}

pub trait PathMatcher: Send + Sync {
    fn matches(&self, path: &Path) -> bool;
    fn should_analyze(&self, path: &Path) -> bool;
    fn should_ignore(&self, path: &Path) -> bool;
}

pub trait FileScanner: Send + Sync {
    fn scan_directory(&self, path: &Path, matcher: &dyn PathMatcher) -> Result<Vec<PathBuf>>;
    fn scan_with_filters(&self, path: &Path, include: &[String], exclude: &[String]) -> Result<Vec<PathBuf>>;
}

// Mock filesystem for testing
#[cfg(test)]
pub struct MockFileSystem {
    files: std::collections::HashMap<PathBuf, String>,
}

#[cfg(test)]
impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: std::collections::HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, content: String) {
        self.files.insert(path, content);
    }
}

#[cfg(test)]
#[async_trait]
impl FileSystem for MockFileSystem {
    async fn read_to_string(&self, path: &Path) -> Result<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| crate::error::UveddiError::Io(
                std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")
            ))
    }

    async fn write_string(&self, _path: &Path, _content: &str) -> Result<()> {
        // Mock implementation
        Ok(())
    }

    // ... other mock implementations
}
```

### 9. Create Interface Implementation Guidelines

#### A. Implementation Pattern:
```rust
// src/implementations/mod.rs
//! Concrete implementations of interfaces
//!
//! Guidelines for implementing interfaces:
//! 1. Each implementation should be in its own module
//! 2. Use dependency injection for all dependencies
//! 3. Implement proper error handling
//! 4. Add comprehensive logging
//! 5. Include health checks where applicable

use crate::interfaces::*;

// Example implementation
pub struct DefaultAnalysisEngine {
    parsers: HashMap<Language, Box<dyn LanguageParser>>,
    detectors: Vec<Box<dyn Detector>>,
    logger: Arc<dyn Logger>,
    config: Arc<dyn ConfigProvider>,
}

impl DefaultAnalysisEngine {
    pub fn new(
        logger: Arc<dyn Logger>,
        config: Arc<dyn ConfigProvider>,
    ) -> Result<Self> {
        let mut engine = Self {
            parsers: HashMap::new(),
            detectors: Vec::new(),
            logger,
            config,
        };

        engine.initialize_parsers()?;
        engine.initialize_detectors()?;

        Ok(engine)
    }

    fn initialize_parsers(&mut self) -> Result<()> {
        // Register language parsers
        #[cfg(feature = "rust-support")]
        self.parsers.insert(Language::Rust, Box::new(RustParser::new()));

        #[cfg(feature = "python-support")]
        self.parsers.insert(Language::Python, Box::new(PythonParser::new()));

        Ok(())
    }

    fn initialize_detectors(&mut self) -> Result<()> {
        // Register detectors based on configuration
        let detector_config = self.config.get_analysis_config();

        for detector_name in &detector_config.enabled_detectors {
            match detector_name.as_str() {
                "god-object" => self.detectors.push(Box::new(GodObjectDetector::new())),
                "security" => self.detectors.push(Box::new(SecurityDetector::new())),
                _ => self.logger.warn(&format!("Unknown detector: {}", detector_name)),
            }
        }

        Ok(())
    }
}

#[async_trait]
impl AnalysisEngine for DefaultAnalysisEngine {
    async fn analyze_project(&self, config: &AnalysisConfig) -> Result<AnalysisResult> {
        self.logger.info_with_context(
            "Starting project analysis",
            &json!({ "target_path": config.target_path.display() })
        );

        // Implementation...
        todo!("Implement project analysis using injected dependencies")
    }

    async fn analyze_file(&self, file_path: &Path) -> Result<FileAnalysis> {
        // Implementation using interface dependencies
        todo!("Implement file analysis")
    }

    fn supported_languages(&self) -> &[Language] {
        // Return languages based on available parsers
        todo!("Return supported languages")
    }

    fn available_detectors(&self) -> Vec<DetectorInfo> {
        self.detectors
            .iter()
            .map(|detector| DetectorInfo {
                metadata: detector.metadata(),
                enabled: true,
                configuration: serde_json::Value::Null,
            })
            .collect()
    }
}
```

### 10. Update Dependency Injection Container

#### A. Register Interfaces in Container:
```rust
// src/container/interface_registration.rs
impl ServiceContainer {
    pub async fn register_interfaces(&mut self) -> Result<()> {
        // Register concrete implementations of interfaces

        // Analysis interfaces
        let logger = self.resolve::<dyn Logger>();
        let config = self.resolve::<dyn ConfigProvider>();

        let analysis_engine = DefaultAnalysisEngine::new(logger.clone(), config.clone())?;
        self.register::<dyn AnalysisEngine>(analysis_engine);

        // Repository interfaces
        let db_config = config.get_database_config();
        let analysis_repo = SqliteAnalysisRepository::new(db_config).await?;
        self.register::<dyn AnalysisRepository>(analysis_repo);

        // Report generation interfaces
        let report_generator = DefaultReportGenerator::new(config.clone())?;
        self.register::<dyn ReportGenerator>(report_generator);

        // File system interface
        #[cfg(not(test))]
        self.register::<dyn FileSystem>(DefaultFileSystem::new());

        #[cfg(test)]
        self.register::<dyn FileSystem>(MockFileSystem::new());

        Ok(())
    }
}
```

## Success Criteria
- [ ] All major components use interface abstractions
- [ ] Dependency injection working for all interfaces
- [ ] Components easily mockable for testing
- [ ] Clear interface contracts defined
- [ ] Implementation details hidden behind interfaces
- [ ] All tests pass with new interface system

## Interface Design Quality Checklist
- [ ] Interfaces are focused and cohesive
- [ ] Methods have clear contracts (parameters, return values, errors)
- [ ] Async interfaces use async_trait consistently
- [ ] All interface methods are testable
- [ ] Interfaces support different implementations

## Verification Commands
```bash
# Test interface implementations
cargo test interfaces::

# Test dependency injection with interfaces
cargo test container::

# Check that implementations satisfy interfaces
cargo check --all-features

# Run integration tests with mocked interfaces
cargo test --test integration_tests
```

## Completion Notes
_To be filled by AI developer:_
- Interfaces extracted: ___
- Components using DI: ___
- Mock implementations created: ___
- Test coverage improvement: ___
- Interface design quality: ___