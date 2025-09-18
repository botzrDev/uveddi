# Assignment 21: Implement Builder Patterns

## Priority: MEDIUM
## Estimated Time: 3-4 hours
## Dependencies: Assignments 18-20 (Interfaces, SOLID, Facades)

## Objective
Implement builder patterns to simplify complex object construction and provide fluent APIs for configuration.

## Current Problem
- Complex configuration objects with many optional parameters
- Difficult to create objects with sensible defaults
- Constructors with too many parameters
- Configuration validation scattered throughout codebase

## Tasks

### 1. Identify Builder Pattern Opportunities

#### A. Find Complex Configuration Objects:
```bash
# Find structs with many fields
rg "pub struct.*Config" src/ --type rust -A 20 | grep "pub.*:" | wc -l

# Find constructors with many parameters
rg "fn new\(" src/ --type rust -A 5 | grep -E "\w+:" | head -20

# Find configuration validation code
rg "validate.*config\|config.*validate" src/ --type rust
```

#### B. Analyze Configuration Complexity:
```markdown
# Configuration Objects Needing Builders

## High Priority:
1. **AnalysisConfig**: 15+ fields with complex validation
2. **DatabaseConfig**: Multiple connection parameters
3. **ReportConfig**: Various output options and formats
4. **PluginConfig**: Complex security and loading options

## Medium Priority:
5. **LoggingConfig**: Multiple output and format options
6. **CacheConfig**: Various caching strategies
7. **AiConfig**: Different providers and models

## Low Priority:
8. **WebConfig**: Server configuration options
```

### 2. Create Analysis Configuration Builder

#### A. Design Fluent Builder API:
```rust
// src/builders/analysis_config.rs
use crate::types::*;
use crate::error::{Result, ValidationError, UveddiError};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Builder for AnalysisConfig with fluent API and validation
#[derive(Debug, Clone)]
pub struct AnalysisConfigBuilder {
    target_path: Option<PathBuf>,
    languages: Vec<Language>,
    detectors: Vec<String>,
    output_format: OutputFormat,
    output_path: Option<PathBuf>,
    parallel_analysis: bool,
    timeout: Duration,
    max_file_size: usize,
    exclude_patterns: Vec<String>,
    include_patterns: Vec<String>,
    custom_rules: Vec<CustomRule>,
    cache_enabled: bool,
    validation_errors: Vec<String>,
}

impl AnalysisConfigBuilder {
    /// Create new builder with sensible defaults
    pub fn new() -> Self {
        Self {
            target_path: None,
            languages: vec![Language::Rust], // Default to Rust
            detectors: vec![
                "god-object".to_string(),
                "security".to_string(),
                "code-quality".to_string(),
            ],
            output_format: OutputFormat::Json,
            output_path: None,
            parallel_analysis: true,
            timeout: Duration::from_secs(300), // 5 minutes
            max_file_size: 10 * 1024 * 1024, // 10MB
            exclude_patterns: vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                ".git/**".to_string(),
            ],
            include_patterns: Vec::new(),
            custom_rules: Vec::new(),
            cache_enabled: true,
            validation_errors: Vec::new(),
        }
    }

    /// Set target path for analysis (required)
    pub fn target_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.target_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Add language to analyze
    pub fn language(mut self, language: Language) -> Self {
        if !self.languages.contains(&language) {
            self.languages.push(language);
        }
        self
    }

    /// Set languages to analyze (replaces existing)
    pub fn languages(mut self, languages: Vec<Language>) -> Self {
        self.languages = languages;
        self
    }

    /// Auto-detect languages from target directory
    pub fn auto_detect_languages(mut self) -> Self {
        // This would be implemented to scan the target directory
        // For now, set common languages as default
        self.languages = vec![
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
        ];
        self
    }

    /// Add detector to analysis
    pub fn detector<S: Into<String>>(mut self, detector: S) -> Self {
        let detector_name = detector.into();
        if !self.detectors.contains(&detector_name) {
            self.detectors.push(detector_name);
        }
        self
    }

    /// Set detectors (replaces existing)
    pub fn detectors<S: Into<String>>(mut self, detectors: Vec<S>) -> Self {
        self.detectors = detectors.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Use only security detectors
    pub fn security_only(mut self) -> Self {
        self.detectors = vec![
            "security-vulnerabilities".to_string(),
            "hardcoded-secrets".to_string(),
            "unsafe-code".to_string(),
        ];
        self
    }

    /// Use only performance detectors
    pub fn performance_only(mut self) -> Self {
        self.detectors = vec![
            "performance-bottlenecks".to_string(),
            "memory-leaks".to_string(),
            "inefficient-algorithms".to_string(),
        ];
        self
    }

    /// Use all available detectors
    pub fn all_detectors(mut self) -> Self {
        self.detectors = vec![
            "god-object".to_string(),
            "security-vulnerabilities".to_string(),
            "performance-bottlenecks".to_string(),
            "code-quality".to_string(),
            "anti-patterns".to_string(),
            "maintainability".to_string(),
        ];
        self
    }

    /// Set output format
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Set output path
    pub fn output_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Generate output path based on target and format
    pub fn auto_output_path(mut self) -> Self {
        if let Some(target) = &self.target_path {
            let extension = match self.output_format {
                OutputFormat::Html => "html",
                OutputFormat::Json => "json",
                OutputFormat::Markdown => "md",
                OutputFormat::Xml => "xml",
            };

            let filename = format!(
                "analysis_{}_{}.{}",
                target.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("project"),
                chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                extension
            );

            self.output_path = Some(PathBuf::from(filename));
        }
        self
    }

    /// Enable or disable parallel analysis
    pub fn parallel_analysis(mut self, enabled: bool) -> Self {
        self.parallel_analysis = enabled;
        self
    }

    /// Set analysis timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set timeout in seconds (convenience method)
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    /// Set timeout in minutes (convenience method)
    pub fn timeout_minutes(mut self, minutes: u64) -> Self {
        self.timeout = Duration::from_secs(minutes * 60);
        self
    }

    /// Set maximum file size to analyze
    pub fn max_file_size(mut self, bytes: usize) -> Self {
        self.max_file_size = bytes;
        self
    }

    /// Set maximum file size in MB (convenience method)
    pub fn max_file_size_mb(mut self, mb: usize) -> Self {
        self.max_file_size = mb * 1024 * 1024;
        self
    }

    /// Add exclude pattern
    pub fn exclude_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Set exclude patterns (replaces existing)
    pub fn exclude_patterns<S: Into<String>>(mut self, patterns: Vec<S>) -> Self {
        self.exclude_patterns = patterns.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add include pattern
    pub fn include_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.include_patterns.push(pattern.into());
        self
    }

    /// Enable or disable caching
    pub fn cache_enabled(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// Add custom analysis rule
    pub fn custom_rule(mut self, rule: CustomRule) -> Self {
        self.custom_rules.push(rule);
        self
    }

    /// Configure for CI/CD environment
    pub fn ci_mode(mut self) -> Self {
        self.parallel_analysis = true;
        self.cache_enabled = false; // Don't use cache in CI
        self.timeout = Duration::from_secs(600); // 10 minutes for CI
        self.output_format = OutputFormat::Json; // Machine-readable output
        self
    }

    /// Configure for development environment
    pub fn dev_mode(mut self) -> Self {
        self.parallel_analysis = true;
        self.cache_enabled = true; // Use cache for faster development
        self.timeout = Duration::from_secs(120); // 2 minutes for quick feedback
        self.output_format = OutputFormat::Html; // Human-readable output
        self
    }

    /// Configure for production analysis
    pub fn production_mode(mut self) -> Self {
        self.parallel_analysis = true;
        self.cache_enabled = true;
        self.timeout = Duration::from_secs(1800); // 30 minutes for thorough analysis
        self.all_detectors() // Use all detectors for comprehensive analysis
    }

    /// Validate configuration and build
    pub fn build(mut self) -> Result<AnalysisConfig> {
        self.validate()?;

        if !self.validation_errors.is_empty() {
            return Err(UveddiError::Validation(ValidationError::FieldError {
                field: "configuration".to_string(),
                details: self.validation_errors.join("; "),
            }));
        }

        Ok(AnalysisConfig {
            target_path: self.target_path.unwrap(), // Validated to exist
            languages: self.languages,
            detectors: self.detectors,
            output_format: self.output_format,
            output_path: self.output_path,
            parallel_analysis: self.parallel_analysis,
            timeout: self.timeout,
            max_file_size: self.max_file_size,
            exclude_patterns: self.exclude_patterns,
            include_patterns: self.include_patterns,
            custom_rules: self.custom_rules,
            cache_enabled: self.cache_enabled,
        })
    }

    /// Validate configuration without building
    pub fn validate(&mut self) -> Result<()> {
        self.validation_errors.clear();

        // Validate required fields
        if self.target_path.is_none() {
            self.validation_errors.push("Target path is required".to_string());
        }

        // Validate target path exists
        if let Some(path) = &self.target_path {
            if !path.exists() {
                self.validation_errors.push(format!("Target path does not exist: {}", path.display()));
            }
        }

        // Validate languages
        if self.languages.is_empty() {
            self.validation_errors.push("At least one language must be specified".to_string());
        }

        // Validate detectors
        if self.detectors.is_empty() {
            self.validation_errors.push("At least one detector must be enabled".to_string());
        }

        // Validate timeout
        if self.timeout.as_secs() < 10 {
            self.validation_errors.push("Timeout must be at least 10 seconds".to_string());
        }

        if self.timeout.as_secs() > 7200 {
            self.validation_errors.push("Timeout cannot exceed 2 hours".to_string());
        }

        // Validate file size
        if self.max_file_size < 1024 {
            self.validation_errors.push("Maximum file size must be at least 1KB".to_string());
        }

        if self.max_file_size > 1024 * 1024 * 1024 {
            self.validation_errors.push("Maximum file size cannot exceed 1GB".to_string());
        }

        // Validate output path if specified
        if let Some(output_path) = &self.output_path {
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    self.validation_errors.push(format!(
                        "Output directory does not exist: {}",
                        parent.display()
                    ));
                }
            }
        }

        if self.validation_errors.is_empty() {
            Ok(())
        } else {
            Err(UveddiError::Validation(ValidationError::FieldError {
                field: "configuration".to_string(),
                details: self.validation_errors.join("; "),
            }))
        }
    }

    /// Get current validation errors
    pub fn validation_errors(&self) -> &[String] {
        &self.validation_errors
    }

    /// Check if configuration is valid
    pub fn is_valid(&mut self) -> bool {
        self.validate().is_ok()
    }
}

impl Default for AnalysisConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct CustomRule {
    pub name: String,
    pub pattern: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}
```

### 3. Create Database Configuration Builder

#### A. Database Connection Builder:
```rust
// src/builders/database_config.rs
use crate::types::*;
use crate::error::Result;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfigBuilder {
    database_type: Option<DatabaseType>,
    host: Option<String>,
    port: Option<u16>,
    database_name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    connection_string: Option<String>,
    max_connections: u32,
    min_connections: u32,
    connection_timeout: Duration,
    idle_timeout: Duration,
    ssl_mode: SslMode,
    migration_timeout: Duration,
}

impl DatabaseConfigBuilder {
    pub fn new() -> Self {
        Self {
            database_type: None,
            host: None,
            port: None,
            database_name: None,
            username: None,
            password: None,
            connection_string: None,
            max_connections: 10,
            min_connections: 1,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            ssl_mode: SslMode::Prefer,
            migration_timeout: Duration::from_secs(300),
        }
    }

    /// Configure for SQLite database
    pub fn sqlite<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        self.database_type = Some(DatabaseType::SQLite);
        self.connection_string = Some(format!("sqlite:{}", path.as_ref().display()));
        self.max_connections = 1; // SQLite doesn't support multiple connections
        self.ssl_mode = SslMode::Disable; // SQLite doesn't use SSL
        self
    }

    /// Configure for PostgreSQL database
    pub fn postgresql(mut self) -> Self {
        self.database_type = Some(DatabaseType::PostgreSQL);
        self.port = Some(5432); // Default PostgreSQL port
        self
    }

    /// Configure for MySQL database
    pub fn mysql(mut self) -> Self {
        self.database_type = Some(DatabaseType::MySQL);
        self.port = Some(3306); // Default MySQL port
        self
    }

    /// Set database host
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set database port
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set database name
    pub fn database<S: Into<String>>(mut self, name: S) -> Self {
        self.database_name = Some(name.into());
        self
    }

    /// Set username
    pub fn username<S: Into<String>>(mut self, username: S) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set password
    pub fn password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set custom connection string (overrides individual parameters)
    pub fn connection_string<S: Into<String>>(mut self, connection_string: S) -> Self {
        self.connection_string = Some(connection_string.into());
        self
    }

    /// Set maximum number of connections in pool
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set minimum number of connections in pool
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Set connection timeout
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set SSL mode
    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    /// Enable SSL (convenience method)
    pub fn enable_ssl(mut self) -> Self {
        self.ssl_mode = SslMode::Require;
        self
    }

    /// Disable SSL (convenience method)
    pub fn disable_ssl(mut self) -> Self {
        self.ssl_mode = SslMode::Disable;
        self
    }

    /// Configure for development environment
    pub fn development_mode(mut self) -> Self {
        self.max_connections = 5;
        self.min_connections = 1;
        self.connection_timeout = Duration::from_secs(10);
        self.ssl_mode = SslMode::Disable; // Simplify for development
        self
    }

    /// Configure for production environment
    pub fn production_mode(mut self) -> Self {
        self.max_connections = 20;
        self.min_connections = 5;
        self.connection_timeout = Duration::from_secs(30);
        self.idle_timeout = Duration::from_secs(300);
        self.ssl_mode = SslMode::Require; // Secure for production
        self
    }

    /// Build the database configuration
    pub fn build(self) -> Result<DatabaseConfig> {
        let connection_string = if let Some(cs) = self.connection_string {
            cs
        } else {
            self.build_connection_string()?
        };

        Ok(DatabaseConfig {
            connection_string,
            max_connections: self.max_connections,
            min_connections: self.min_connections,
            connection_timeout: self.connection_timeout,
            idle_timeout: self.idle_timeout,
            ssl_mode: self.ssl_mode,
            migration_timeout: self.migration_timeout,
        })
    }

    fn build_connection_string(&self) -> Result<String> {
        match self.database_type {
            Some(DatabaseType::PostgreSQL) => {
                let host = self.host.as_ref().ok_or_else(|| {
                    UveddiError::Validation(ValidationError::FieldError {
                        field: "host".to_string(),
                        details: "Host is required for PostgreSQL".to_string(),
                    })
                })?;

                let port = self.port.unwrap_or(5432);
                let database = self.database_name.as_ref().ok_or_else(|| {
                    UveddiError::Validation(ValidationError::FieldError {
                        field: "database".to_string(),
                        details: "Database name is required for PostgreSQL".to_string(),
                    })
                })?;

                let mut url = format!("postgresql://{}:{}/{}", host, port, database);

                if let (Some(username), Some(password)) = (&self.username, &self.password) {
                    url = format!("postgresql://{}:{}@{}:{}/{}", username, password, host, port, database);
                }

                // Add SSL mode
                match self.ssl_mode {
                    SslMode::Require => url.push_str("?sslmode=require"),
                    SslMode::Disable => url.push_str("?sslmode=disable"),
                    SslMode::Prefer => url.push_str("?sslmode=prefer"),
                }

                Ok(url)
            }
            Some(DatabaseType::SQLite) => {
                // SQLite connection string should already be set
                Err(UveddiError::Validation(ValidationError::FieldError {
                    field: "connection_string".to_string(),
                    details: "SQLite requires explicit connection string".to_string(),
                }))
            }
            _ => Err(UveddiError::Validation(ValidationError::FieldError {
                field: "database_type".to_string(),
                details: "Database type must be specified".to_string(),
            })),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    MySQL,
}

#[derive(Debug, Clone)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}
```

### 4. Create Report Configuration Builder

#### A. Flexible Report Builder:
```rust
// src/builders/report_config.rs
use crate::types::*;
use crate::error::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ReportConfigBuilder {
    formats: Vec<ReportFormat>,
    output_directory: Option<PathBuf>,
    output_filename: Option<String>,
    template_path: Option<PathBuf>,
    include_charts: bool,
    include_metrics: bool,
    include_recommendations: bool,
    theme: ReportTheme,
    custom_css: Option<String>,
    embed_assets: bool,
    compress_output: bool,
    sections: Vec<ReportSection>,
}

impl ReportConfigBuilder {
    pub fn new() -> Self {
        Self {
            formats: vec![ReportFormat::Html],
            output_directory: Some(PathBuf::from("./reports")),
            output_filename: None,
            template_path: None,
            include_charts: true,
            include_metrics: true,
            include_recommendations: true,
            theme: ReportTheme::Light,
            custom_css: None,
            embed_assets: true,
            compress_output: false,
            sections: vec![
                ReportSection::Summary,
                ReportSection::Findings,
                ReportSection::Metrics,
                ReportSection::Recommendations,
            ],
        }
    }

    /// Add output format
    pub fn format(mut self, format: ReportFormat) -> Self {
        if !self.formats.contains(&format) {
            self.formats.push(format);
        }
        self
    }

    /// Set output formats (replaces existing)
    pub fn formats(mut self, formats: Vec<ReportFormat>) -> Self {
        self.formats = formats;
        self
    }

    /// Generate HTML report only
    pub fn html_only(mut self) -> Self {
        self.formats = vec![ReportFormat::Html];
        self
    }

    /// Generate multiple common formats
    pub fn multi_format(mut self) -> Self {
        self.formats = vec![
            ReportFormat::Html,
            ReportFormat::Json,
            ReportFormat::Markdown,
        ];
        self
    }

    /// Set output directory
    pub fn output_directory<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_directory = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Set output filename (without extension)
    pub fn filename<S: Into<String>>(mut self, name: S) -> Self {
        self.output_filename = Some(name.into());
        self
    }

    /// Generate filename with timestamp
    pub fn timestamped_filename(mut self, base_name: &str) -> Self {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        self.output_filename = Some(format!("{}_{}", base_name, timestamp));
        self
    }

    /// Set custom template path
    pub fn template<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.template_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Include or exclude charts
    pub fn include_charts(mut self, include: bool) -> Self {
        self.include_charts = include;
        self
    }

    /// Include or exclude metrics section
    pub fn include_metrics(mut self, include: bool) -> Self {
        self.include_metrics = include;
        self
    }

    /// Include or exclude recommendations
    pub fn include_recommendations(mut self, include: bool) -> Self {
        self.include_recommendations = include;
        self
    }

    /// Set report theme
    pub fn theme(mut self, theme: ReportTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Use dark theme
    pub fn dark_theme(mut self) -> Self {
        self.theme = ReportTheme::Dark;
        self
    }

    /// Use light theme
    pub fn light_theme(mut self) -> Self {
        self.theme = ReportTheme::Light;
        self
    }

    /// Add custom CSS
    pub fn custom_css<S: Into<String>>(mut self, css: S) -> Self {
        self.custom_css = Some(css.into());
        self
    }

    /// Load custom CSS from file
    pub fn custom_css_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let css = std::fs::read_to_string(path)?;
        self.custom_css = Some(css);
        Ok(self)
    }

    /// Embed assets in output files
    pub fn embed_assets(mut self, embed: bool) -> Self {
        self.embed_assets = embed;
        self
    }

    /// Compress output files
    pub fn compress_output(mut self, compress: bool) -> Self {
        self.compress_output = compress;
        self
    }

    /// Add report section
    pub fn section(mut self, section: ReportSection) -> Self {
        if !self.sections.contains(&section) {
            self.sections.push(section);
        }
        self
    }

    /// Set report sections (replaces existing)
    pub fn sections(mut self, sections: Vec<ReportSection>) -> Self {
        self.sections = sections;
        self
    }

    /// Minimal report (summary and findings only)
    pub fn minimal_report(mut self) -> Self {
        self.sections = vec![ReportSection::Summary, ReportSection::Findings];
        self.include_charts = false;
        self.include_metrics = false;
        self.include_recommendations = false;
        self
    }

    /// Detailed report (all sections)
    pub fn detailed_report(mut self) -> Self {
        self.sections = vec![
            ReportSection::Summary,
            ReportSection::Findings,
            ReportSection::Metrics,
            ReportSection::Recommendations,
            ReportSection::TechnicalDetails,
            ReportSection::Appendix,
        ];
        self.include_charts = true;
        self.include_metrics = true;
        self.include_recommendations = true;
        self
    }

    /// Executive summary report (high-level overview)
    pub fn executive_summary(mut self) -> Self {
        self.sections = vec![
            ReportSection::Summary,
            ReportSection::KeyFindings,
            ReportSection::Recommendations,
        ];
        self.include_charts = true;
        self.include_metrics = false;
        self
    }

    /// Configure for CI/CD environment
    pub fn ci_mode(mut self) -> Self {
        self.formats = vec![ReportFormat::Json, ReportFormat::Junit]; // Machine-readable
        self.include_charts = false;
        self.embed_assets = false;
        self.compress_output = true;
        self
    }

    /// Configure for development environment
    pub fn development_mode(mut self) -> Self {
        self.formats = vec![ReportFormat::Html];
        self.include_charts = true;
        self.embed_assets = true;
        self.theme = ReportTheme::Light;
        self
    }

    /// Build the report configuration
    pub fn build(self) -> Result<ReportConfig> {
        // Validate configuration
        if self.formats.is_empty() {
            return Err(UveddiError::Validation(ValidationError::FieldError {
                field: "formats".to_string(),
                details: "At least one output format must be specified".to_string(),
            }));
        }

        if self.sections.is_empty() {
            return Err(UveddiError::Validation(ValidationError::FieldError {
                field: "sections".to_string(),
                details: "At least one report section must be included".to_string(),
            }));
        }

        // Create output directory if it doesn't exist
        if let Some(dir) = &self.output_directory {
            std::fs::create_dir_all(dir)?;
        }

        Ok(ReportConfig {
            formats: self.formats,
            output_directory: self.output_directory,
            output_filename: self.output_filename,
            template_path: self.template_path,
            include_charts: self.include_charts,
            include_metrics: self.include_metrics,
            include_recommendations: self.include_recommendations,
            theme: self.theme,
            custom_css: self.custom_css,
            embed_assets: self.embed_assets,
            compress_output: self.compress_output,
            sections: self.sections,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Html,
    Json,
    Markdown,
    Pdf,
    Xml,
    Junit, // For CI/CD integration
}

#[derive(Debug, Clone)]
pub enum ReportTheme {
    Light,
    Dark,
    HighContrast,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReportSection {
    Summary,
    Findings,
    KeyFindings,
    Metrics,
    Recommendations,
    TechnicalDetails,
    Appendix,
}
```

### 5. Create Service Container Builder

#### A. Dependency Injection Builder:
```rust
// src/builders/service_container.rs
use crate::container::ServiceContainer;
use crate::interfaces::*;
use crate::implementations::*;
use crate::error::Result;

#[derive(Debug)]
pub struct ServiceContainerBuilder {
    database_config: Option<DatabaseConfig>,
    logging_config: Option<LoggingConfig>,
    cache_config: Option<CacheConfig>,
    analysis_config: Option<AnalysisConfig>,
    ai_config: Option<AiConfig>,
    custom_services: Vec<Box<dyn std::any::Any + Send + Sync>>,
}

impl ServiceContainerBuilder {
    pub fn new() -> Self {
        Self {
            database_config: None,
            logging_config: None,
            cache_config: None,
            analysis_config: None,
            ai_config: None,
            custom_services: Vec::new(),
        }
    }

    /// Set database configuration
    pub fn with_database_config(mut self, config: DatabaseConfig) -> Self {
        self.database_config = Some(config);
        self
    }

    /// Build database config using builder
    pub fn with_database<F>(mut self, builder_fn: F) -> Result<Self>
    where
        F: FnOnce(DatabaseConfigBuilder) -> DatabaseConfigBuilder,
    {
        let config = builder_fn(DatabaseConfigBuilder::new()).build()?;
        self.database_config = Some(config);
        Ok(self)
    }

    /// Set logging configuration
    pub fn with_logging_config(mut self, config: LoggingConfig) -> Self {
        self.logging_config = Some(config);
        self
    }

    /// Set cache configuration
    pub fn with_cache_config(mut self, config: CacheConfig) -> Self {
        self.cache_config = Some(config);
        self
    }

    /// Set analysis configuration
    pub fn with_analysis_config(mut self, config: AnalysisConfig) -> Self {
        self.analysis_config = Some(config);
        self
    }

    /// Build analysis config using builder
    pub fn with_analysis<F>(mut self, builder_fn: F) -> Result<Self>
    where
        F: FnOnce(AnalysisConfigBuilder) -> AnalysisConfigBuilder,
    {
        let config = builder_fn(AnalysisConfigBuilder::new()).build()?;
        self.analysis_config = Some(config);
        Ok(self)
    }

    /// Set AI configuration
    #[cfg(feature = "ai-integration")]
    pub fn with_ai_config(mut self, config: AiConfig) -> Self {
        self.ai_config = Some(config);
        self
    }

    /// Add custom service
    pub fn with_service<T: 'static + Send + Sync>(mut self, service: T) -> Self {
        self.custom_services.push(Box::new(service));
        self
    }

    /// Configure for development environment
    pub fn development_environment(mut self) -> Result<Self> {
        // Database: SQLite for simplicity
        self = self.with_database(|builder| {
            builder
                .sqlite("./dev_analysis.db")
                .development_mode()
        })?;

        // Logging: Debug level to console
        self.logging_config = Some(LoggingConfig {
            level: LogLevel::Debug,
            format: LogFormat::Plain,
            output: LogOutput::Stdout,
            file_path: None,
        });

        // Cache: Simple in-memory cache
        self.cache_config = Some(CacheConfig {
            provider: CacheProvider::Memory,
            max_size: 100 * 1024 * 1024, // 100MB
            ttl: Duration::from_secs(3600), // 1 hour
        });

        Ok(self)
    }

    /// Configure for production environment
    pub fn production_environment(mut self) -> Result<Self> {
        // Database: PostgreSQL (from environment)
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost:5432/uveddi".to_string());

        self = self.with_database(|builder| {
            builder
                .connection_string(db_url)
                .production_mode()
        })?;

        // Logging: Info level to file
        self.logging_config = Some(LoggingConfig {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::File,
            file_path: Some(PathBuf::from("/var/log/uveddi/application.log")),
        });

        // Cache: Redis cache
        self.cache_config = Some(CacheConfig {
            provider: CacheProvider::Redis,
            connection_string: std::env::var("REDIS_URL").ok(),
            max_size: 1024 * 1024 * 1024, // 1GB
            ttl: Duration::from_secs(3600 * 24), // 24 hours
        });

        Ok(self)
    }

    /// Configure for testing environment
    pub fn test_environment(mut self) -> Result<Self> {
        // Database: In-memory SQLite
        self = self.with_database(|builder| {
            builder.sqlite(":memory:")
        })?;

        // Logging: Error level only
        self.logging_config = Some(LoggingConfig {
            level: LogLevel::Error,
            format: LogFormat::Plain,
            output: LogOutput::Stderr,
            file_path: None,
        });

        // Cache: Disabled for predictable tests
        self.cache_config = Some(CacheConfig {
            provider: CacheProvider::None,
            max_size: 0,
            ttl: Duration::from_secs(0),
        });

        Ok(self)
    }

    /// Build the service container
    pub async fn build(self) -> Result<ServiceContainer> {
        let mut container = ServiceContainer::new();

        // Register configuration services
        if let Some(db_config) = self.database_config {
            container.register_config("database", db_config);
        }

        if let Some(logging_config) = self.logging_config {
            container.register_config("logging", logging_config);
        }

        if let Some(cache_config) = self.cache_config {
            container.register_config("cache", cache_config);
        }

        // Initialize core services
        container.initialize_core_services().await?;

        // Register implementations
        container.register_implementations().await?;

        // Register custom services
        for service in self.custom_services {
            // This would need type information to register properly
            // For now, this is a placeholder
        }

        Ok(container)
    }
}

// Supporting configuration types
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub provider: CacheProvider,
    pub connection_string: Option<String>,
    pub max_size: usize,
    pub ttl: Duration,
}

#[derive(Debug, Clone)]
pub enum CacheProvider {
    None,
    Memory,
    Redis,
    File,
}
```

### 6. Create Builder Integration

#### A. Update Facades to Use Builders:
```rust
// src/facades/analysis.rs (updated)
impl AnalysisFacade {
    /// Create analysis configuration with builder
    pub fn config_builder() -> AnalysisConfigBuilder {
        AnalysisConfigBuilder::new()
    }

    /// Analyze with builder configuration
    pub async fn analyze_with_builder<F>(&self, builder_fn: F) -> Result<AnalysisResult>
    where
        F: FnOnce(AnalysisConfigBuilder) -> AnalysisConfigBuilder,
    {
        let config = builder_fn(AnalysisConfigBuilder::new()).build()?;
        self.engine.analyze_project(&config).await
    }

    /// Quick analysis with builder syntax
    pub async fn quick_analysis<P: AsRef<Path>>(&self, target_path: P) -> Result<String> {
        self.analyze_with_builder(|builder| {
            builder
                .target_path(target_path)
                .auto_detect_languages()
                .all_detectors()
                .dev_mode()
        }).await?;

        // Convert result to ID (simplified)
        Ok("analysis_id".to_string())
    }
}
```

### 7. Create Testing Support

#### A. Builder Test Utilities:
```rust
// src/builders/testing.rs
#[cfg(test)]
pub mod test_builders {
    use super::*;

    impl AnalysisConfigBuilder {
        /// Create builder for testing with minimal setup
        pub fn for_testing() -> Self {
            Self::new()
                .target_path("./test_data")
                .timeout_seconds(10)
                .cache_enabled(false)
        }
    }

    impl DatabaseConfigBuilder {
        /// Create in-memory SQLite for testing
        pub fn for_testing() -> Self {
            Self::new().sqlite(":memory:")
        }
    }

    impl ServiceContainerBuilder {
        /// Create minimal container for testing
        pub fn for_testing() -> Self {
            Self::new()
                .test_environment()
                .expect("Failed to configure test environment")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_builders::*;

    #[test]
    fn test_analysis_config_builder() {
        let config = AnalysisConfigBuilder::for_testing()
            .language(Language::Rust)
            .detector("god-object")
            .output_format(OutputFormat::Json)
            .build()
            .expect("Failed to build config");

        assert_eq!(config.target_path, PathBuf::from("./test_data"));
        assert!(config.languages.contains(&Language::Rust));
        assert!(config.detectors.contains(&"god-object".to_string()));
    }

    #[test]
    fn test_builder_validation() {
        let result = AnalysisConfigBuilder::new()
            // Missing target_path
            .language(Language::Rust)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_fluent_api() {
        let config = AnalysisConfigBuilder::new()
            .target_path("./src")
            .auto_detect_languages()
            .all_detectors()
            .timeout_minutes(5)
            .max_file_size_mb(50)
            .ci_mode()
            .build()
            .expect("Failed to build config");

        assert_eq!(config.timeout, Duration::from_secs(300));
        assert_eq!(config.max_file_size, 50 * 1024 * 1024);
        assert!(config.parallel_analysis);
    }
}
```

## Success Criteria
- [ ] Complex configuration objects have builder patterns
- [ ] Fluent APIs provide intuitive configuration
- [ ] Builders include validation and error handling
- [ ] Default configurations for common scenarios
- [ ] Builder patterns integrated with facades
- [ ] Comprehensive testing of builder functionality
- [ ] Builder patterns improve developer experience

## API Usability Checklist
- [ ] Method names are intuitive and discoverable
- [ ] Common configurations have convenience methods
- [ ] Validation provides clear error messages
- [ ] Builders support method chaining
- [ ] Default values are sensible

## Verification Commands
```bash
# Test builder patterns
cargo test builders::

# Test fluent API usage
cargo test fluent_api::

# Test builder validation
cargo test builder_validation::

# Check builder documentation
cargo doc --open --package uveddi-builders
```

## Completion Notes
_To be filled by AI developer:_
- Builders implemented: ___
- Configuration complexity reduced: ___
- API usability improved: ___
- Test coverage: ___
- Documentation quality: ___