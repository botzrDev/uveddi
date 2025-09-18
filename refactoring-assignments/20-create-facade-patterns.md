# Assignment 20: Create Facade Patterns

## Priority: MEDIUM
## Estimated Time: 3-4 hours
## Dependencies: Assignments 18-19 (Interfaces and SOLID principles)

## Objective
Implement facade patterns to simplify complex subsystem interactions and provide clean APIs for external consumers.

## Current Problem
- Complex interfaces with many dependencies make the system hard to use
- External consumers need to understand internal system architecture
- High coupling between client code and internal components
- Difficult to maintain backward compatibility when internal structure changes

## Tasks

### 1. Identify Facade Opportunities

#### A. Analyze Current API Usage Patterns:
```bash
# Find complex initialization patterns
rg "new\(" src/ --type rust -A 5 -B 2 | head -30

# Find multiple service dependencies
rg "Arc<dyn" src/ --type rust | head -20

# Look for complex interaction patterns
rg "\.resolve\(" src/ --type rust
```

#### B. Map Subsystem Complexity:
```markdown
# Subsystem Complexity Analysis

## High-Complexity Subsystems Needing Facades:
1. **Analysis System**: Multiple engines, detectors, parsers
2. **Reporting System**: Multiple formatters, templates, exporters
3. **Configuration System**: Multiple providers, validators, mergers
4. **Plugin System**: Multiple types, loaders, registries
5. **Database System**: Multiple repositories, connections, migrations

## Facade Priority:
- High: Analysis, Configuration
- Medium: Reporting, Database
- Low: Plugin System
```

### 2. Create Analysis Facade

#### A. Design Unified Analysis Interface:
```rust
// src/facades/analysis.rs
use crate::interfaces::*;
use crate::types::*;
use crate::error::Result;

/// High-level facade for analysis operations
///
/// Simplifies complex analysis subsystem interactions into a single,
/// easy-to-use interface. Handles dependency management internally.
pub struct AnalysisFacade {
    // Internal complex dependencies (hidden from users)
    engine: Arc<dyn AnalysisEngine>,
    repository: Arc<dyn AnalysisRepository>,
    cache: Arc<dyn CacheRepository>,
    logger: Arc<dyn Logger>,
    file_system: Arc<dyn FileSystem>,
}

impl AnalysisFacade {
    /// Create new facade from service container
    pub fn new(container: &ServiceContainer) -> Result<Self> {
        Ok(Self {
            engine: container.resolve::<dyn AnalysisEngine>(),
            repository: container.resolve::<dyn AnalysisRepository>(),
            cache: container.resolve::<dyn CacheRepository>(),
            logger: container.resolve::<dyn Logger>(),
            file_system: container.resolve::<dyn FileSystem>(),
        })
    }

    /// Simple analysis with minimal configuration
    ///
    /// # Example
    /// ```rust
    /// let facade = AnalysisFacade::new(&container)?;
    /// let result_id = facade.analyze_simple("./src").await?;
    /// ```
    pub async fn analyze_simple(&self, target_path: &str) -> Result<String> {
        let config = SimpleAnalysisConfig {
            target_path: target_path.into(),
            language: None, // Auto-detect
            output_format: OutputFormat::Json,
        };

        self.analyze_with_config(config).await
    }

    /// Analysis with basic configuration options
    pub async fn analyze_with_config(&self, config: SimpleAnalysisConfig) -> Result<String> {
        let full_config = self.expand_config(config).await?;
        self.run_full_analysis(full_config).await
    }

    /// Analysis with complete control over configuration
    pub async fn analyze_advanced(&self, config: AdvancedAnalysisConfig) -> Result<AnalysisResult> {
        let internal_config = self.convert_advanced_config(config)?;

        // Use caching for repeated analyses
        let cache_key = self.generate_cache_key(&internal_config);
        if let Some(cached) = self.cache.get::<AnalysisResult>(&cache_key).await? {
            self.logger.info("Using cached analysis result");
            return Ok(cached);
        }

        self.logger.info("Starting new analysis");
        let result = self.engine.analyze_project(&internal_config).await?;

        // Cache result for future use
        self.cache.set(&cache_key, &result, Some(Duration::from_hours(1))).await?;

        Ok(result)
    }

    /// Get analysis results by ID
    pub async fn get_analysis_result(&self, analysis_id: &str) -> Result<AnalysisResult> {
        self.repository.load(&analysis_id.to_string()).await
    }

    /// List all analysis results with filtering
    pub async fn list_analyses(&self, filter: Option<AnalysisListFilter>) -> Result<Vec<AnalysisMetadata>> {
        let internal_filter = filter.unwrap_or_default().into();
        self.repository.list(&internal_filter).await
    }

    /// Delete analysis result
    pub async fn delete_analysis(&self, analysis_id: &str) -> Result<()> {
        self.repository.delete(&analysis_id.to_string()).await
    }

    // Private helper methods (hidden complexity)
    async fn expand_config(&self, simple: SimpleAnalysisConfig) -> Result<AnalysisConfig> {
        let mut config = AnalysisConfig::default();
        config.target_path = simple.target_path;
        config.output_format = simple.output_format;

        // Auto-detect language if not specified
        if let Some(language) = simple.language {
            config.languages = vec![language];
        } else {
            config.languages = self.detect_languages(&config.target_path).await?;
        }

        // Use reasonable defaults for other settings
        config.detectors = vec![
            "god-object".to_string(),
            "security".to_string(),
            "code-quality".to_string(),
        ];
        config.parallel_analysis = true;
        config.timeout = Duration::from_secs(300); // 5 minutes

        Ok(config)
    }

    async fn run_full_analysis(&self, config: AnalysisConfig) -> Result<String> {
        let result = self.engine.analyze_project(&config).await?;
        let analysis_id = self.repository.save(&result).await?;

        self.logger.info(&format!(
            "Analysis completed successfully. ID: {}, Files: {}, Findings: {}",
            analysis_id,
            result.summary.files_analyzed,
            result.findings.len()
        ));

        Ok(analysis_id)
    }

    fn convert_advanced_config(&self, advanced: AdvancedAnalysisConfig) -> Result<AnalysisConfig> {
        // Convert user-friendly advanced config to internal config
        Ok(AnalysisConfig {
            target_path: advanced.target_path,
            output_format: advanced.output_format,
            languages: advanced.languages,
            detectors: advanced.enabled_detectors,
            parallel_analysis: advanced.parallel_execution,
            timeout: advanced.timeout_seconds.map(Duration::from_secs).unwrap_or(Duration::from_secs(300)),
            max_file_size: advanced.max_file_size_mb.map(|mb| mb * 1024 * 1024).unwrap_or(10 * 1024 * 1024),
            exclude_patterns: advanced.exclude_patterns.unwrap_or_default(),
        })
    }

    async fn detect_languages(&self, path: &Path) -> Result<Vec<Language>> {
        let files = self.file_system.list_files(path, true).await?;
        let mut languages = std::collections::HashSet::new();

        for file in files.iter().take(100) { // Sample first 100 files
            if let Some(lang) = Language::from_file_extension(file) {
                languages.insert(lang);
            }
        }

        Ok(languages.into_iter().collect())
    }

    fn generate_cache_key(&self, config: &AnalysisConfig) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        config.target_path.hash(&mut hasher);
        config.languages.hash(&mut hasher);
        config.detectors.hash(&mut hasher);

        format!("analysis::{:x}", hasher.finish())
    }
}

// Simplified configuration types for facade users
#[derive(Debug, Clone)]
pub struct SimpleAnalysisConfig {
    pub target_path: PathBuf,
    pub language: Option<Language>,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone)]
pub struct AdvancedAnalysisConfig {
    pub target_path: PathBuf,
    pub languages: Vec<Language>,
    pub enabled_detectors: Vec<String>,
    pub output_format: OutputFormat,
    pub parallel_execution: bool,
    pub timeout_seconds: Option<u64>,
    pub max_file_size_mb: Option<usize>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisListFilter {
    pub language: Option<Language>,
    pub date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub limit: Option<usize>,
}

impl From<AnalysisListFilter> for crate::interfaces::AnalysisFilter {
    fn from(filter: AnalysisListFilter) -> Self {
        Self {
            project_id: None,
            language: filter.language,
            date_range: filter.date_range,
            limit: filter.limit,
        }
    }
}
```

### 3. Create Reporting Facade

#### A. Unified Reporting Interface:
```rust
// src/facades/reporting.rs
use crate::interfaces::*;
use crate::types::*;
use crate::error::Result;

/// High-level facade for report generation and export
pub struct ReportingFacade {
    generator: Arc<dyn ReportGenerator>,
    template_engine: Arc<dyn TemplateEngine>,
    file_system: Arc<dyn FileSystem>,
    logger: Arc<dyn Logger>,
}

impl ReportingFacade {
    pub fn new(container: &ServiceContainer) -> Result<Self> {
        Ok(Self {
            generator: container.resolve::<dyn ReportGenerator>(),
            template_engine: container.resolve::<dyn TemplateEngine>(),
            file_system: container.resolve::<dyn FileSystem>(),
            logger: container.resolve::<dyn Logger>(),
        })
    }

    /// Generate report from analysis ID with automatic format detection
    pub async fn generate_report_auto(&self, analysis_id: &str, output_path: &Path) -> Result<()> {
        let format = self.detect_format_from_extension(output_path)?;
        self.generate_report_with_format(analysis_id, output_path, format).await
    }

    /// Generate report with specific format
    pub async fn generate_report_with_format(
        &self,
        analysis_id: &str,
        output_path: &Path,
        format: ReportFormat,
    ) -> Result<()> {
        // Load analysis result
        let analysis = self.load_analysis_result(analysis_id).await?;

        // Generate report content
        let content = self.generator.generate(&analysis, &format).await?;

        // Export to file
        self.generator.export(&content, output_path).await?;

        self.logger.info(&format!(
            "Report generated successfully: {} (format: {:?})",
            output_path.display(),
            format
        ));

        Ok(())
    }

    /// Generate multiple reports in different formats
    pub async fn generate_multi_format_reports(
        &self,
        analysis_id: &str,
        base_path: &Path,
        formats: &[ReportFormat],
    ) -> Result<Vec<PathBuf>> {
        let analysis = self.load_analysis_result(analysis_id).await?;
        let mut generated_files = Vec::new();

        for format in formats {
            let extension = match format {
                ReportFormat::Html => "html",
                ReportFormat::Markdown => "md",
                ReportFormat::Json => "json",
                ReportFormat::Pdf => "pdf",
                _ => "txt",
            };

            let output_path = base_path.with_extension(extension);
            let content = self.generator.generate(&analysis, format).await?;
            self.generator.export(&content, &output_path).await?;

            generated_files.push(output_path);
        }

        self.logger.info(&format!(
            "Generated {} reports for analysis {}",
            generated_files.len(),
            analysis_id
        ));

        Ok(generated_files)
    }

    /// Generate report with custom template
    pub async fn generate_custom_report(
        &self,
        analysis_id: &str,
        template_path: &Path,
        output_path: &Path,
        custom_data: Option<serde_json::Value>,
    ) -> Result<()> {
        let analysis = self.load_analysis_result(analysis_id).await?;
        let template = self.file_system.read_to_string(template_path).await?;

        // Prepare template context
        let mut context = serde_json::to_value(&analysis)?;
        if let Some(custom) = custom_data {
            if let (Some(context_obj), Some(custom_obj)) = (context.as_object_mut(), custom.as_object()) {
                for (key, value) in custom_obj {
                    context_obj.insert(key.clone(), value.clone());
                }
            }
        }

        // Render template
        let content = self.template_engine.render(&template, &context).await?;

        // Export result
        self.file_system.write_string(output_path, &content).await?;

        self.logger.info(&format!(
            "Custom report generated: {} using template {}",
            output_path.display(),
            template_path.display()
        ));

        Ok(())
    }

    /// Generate interactive dashboard (HTML with charts)
    pub async fn generate_dashboard(
        &self,
        analysis_id: &str,
        output_dir: &Path,
        include_charts: bool,
    ) -> Result<PathBuf> {
        let analysis = self.load_analysis_result(analysis_id).await?;

        // Create output directory
        self.file_system.create_dir_all(output_dir).await?;

        // Generate main dashboard HTML
        let dashboard_content = self.generate_dashboard_html(&analysis, include_charts).await?;
        let main_file = output_dir.join("dashboard.html");
        self.file_system.write_string(&main_file, &dashboard_content).await?;

        // Generate supporting files
        if include_charts {
            self.generate_chart_data(&analysis, output_dir).await?;
        }
        self.copy_dashboard_assets(output_dir).await?;

        self.logger.info(&format!("Dashboard generated at: {}", main_file.display()));
        Ok(main_file)
    }

    // Private helper methods
    async fn load_analysis_result(&self, analysis_id: &str) -> Result<AnalysisResult> {
        // This would delegate to AnalysisFacade or directly to repository
        // For now, simulate loading
        todo!("Load analysis result from repository")
    }

    fn detect_format_from_extension(&self, path: &Path) -> Result<ReportFormat> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("html") | Some("htm") => Ok(ReportFormat::Html),
            Some("md") | Some("markdown") => Ok(ReportFormat::Markdown),
            Some("json") => Ok(ReportFormat::Json),
            Some("pdf") => Ok(ReportFormat::Pdf),
            _ => Err(UveddiError::Validation(ValidationError::FieldError {
                field: "file_extension".to_string(),
                details: "Unsupported file extension for report format".to_string(),
            })),
        }
    }

    async fn generate_dashboard_html(&self, analysis: &AnalysisResult, include_charts: bool) -> Result<String> {
        // Generate interactive HTML dashboard
        todo!("Generate dashboard HTML")
    }

    async fn generate_chart_data(&self, analysis: &AnalysisResult, output_dir: &Path) -> Result<()> {
        // Generate chart data files (JSON) for dashboard
        todo!("Generate chart data")
    }

    async fn copy_dashboard_assets(&self, output_dir: &Path) -> Result<()> {
        // Copy CSS, JS, and other assets
        todo!("Copy dashboard assets")
    }
}
```

### 4. Create Configuration Facade

#### A. Simplified Configuration Management:
```rust
// src/facades/configuration.rs
use crate::interfaces::*;
use crate::types::*;
use crate::error::Result;

/// High-level facade for configuration management
pub struct ConfigurationFacade {
    provider: Box<dyn ConfigProvider>,
    validator: Box<dyn ConfigValidator>,
    logger: Arc<dyn Logger>,
}

impl ConfigurationFacade {
    /// Create facade with automatic configuration loading
    pub fn new() -> Result<Self> {
        let mut facade = Self {
            provider: Box::new(DefaultConfigProvider::new()),
            validator: Box::new(DefaultConfigValidator::new()),
            logger: Arc::new(DefaultLogger::new()),
        };

        facade.load_default_configuration()?;
        Ok(facade)
    }

    /// Load configuration from multiple sources with priority
    pub fn load_configuration(&mut self, options: ConfigurationOptions) -> Result<()> {
        let mut configs = Vec::new();

        // Load configurations in priority order
        if let Some(file_path) = options.config_file {
            let file_config = self.load_from_file(&file_path)?;
            configs.push(file_config);
        }

        if options.use_environment {
            let env_config = self.load_from_environment()?;
            configs.push(env_config);
        }

        if let Some(args) = options.command_line_args {
            let cli_config = self.load_from_command_line(&args)?;
            configs.push(cli_config);
        }

        // Merge configurations (later sources override earlier ones)
        self.provider = self.merge_configurations(configs)?;

        // Validate final configuration
        self.validator.validate_analysis_config(self.provider.get_analysis_config())?;
        self.validator.validate_database_config(self.provider.get_database_config())?;

        self.logger.info("Configuration loaded and validated successfully");
        Ok(())
    }

    /// Get simplified analysis configuration
    pub fn get_analysis_settings(&self) -> AnalysisSettings {
        let config = self.provider.get_analysis_config();
        AnalysisSettings {
            default_language: config.languages.first().cloned(),
            timeout_minutes: config.timeout.as_secs() / 60,
            max_file_size_mb: config.max_file_size / (1024 * 1024),
            parallel_processing: config.parallel_analysis,
            enabled_detectors: config.detectors.clone(),
        }
    }

    /// Update analysis settings
    pub fn update_analysis_settings(&mut self, settings: AnalysisSettings) -> Result<()> {
        // Update internal configuration
        // This would typically create a new config provider
        todo!("Update analysis settings")
    }

    /// Get database connection string with sensitive data hidden
    pub fn get_database_info(&self) -> DatabaseInfo {
        let config = self.provider.get_database_config();
        DatabaseInfo {
            database_type: self.detect_database_type(&config.connection_string),
            host: self.extract_host(&config.connection_string),
            port: self.extract_port(&config.connection_string),
            // Sensitive data like password not exposed
        }
    }

    /// Validate current configuration
    pub fn validate_configuration(&self) -> Result<ConfigurationStatus> {
        let mut status = ConfigurationStatus {
            valid: true,
            issues: Vec::new(),
            warnings: Vec::new(),
        };

        // Check analysis configuration
        if let Err(e) = self.validator.validate_analysis_config(self.provider.get_analysis_config()) {
            status.valid = false;
            status.issues.push(format!("Analysis config: {}", e));
        }

        // Check database configuration
        if let Err(e) = self.validator.validate_database_config(self.provider.get_database_config()) {
            status.valid = false;
            status.issues.push(format!("Database config: {}", e));
        }

        // Check paths exist
        if let Err(e) = self.validator.validate_paths(self.provider.get_analysis_config()) {
            status.warnings.push(format!("Path validation: {}", e));
        }

        Ok(status)
    }

    /// Export configuration to file
    pub fn export_configuration(&self, output_path: &Path) -> Result<()> {
        let config_data = self.serialize_configuration()?;
        std::fs::write(output_path, config_data)?;

        self.logger.info(&format!("Configuration exported to: {}", output_path.display()));
        Ok(())
    }

    // Private helper methods
    fn load_default_configuration(&mut self) -> Result<()> {
        // Load sensible defaults
        let options = ConfigurationOptions {
            config_file: None,
            use_environment: true,
            command_line_args: None,
        };
        self.load_configuration(options)
    }

    fn load_from_file(&self, path: &Path) -> Result<Box<dyn ConfigProvider>> {
        todo!("Load configuration from file")
    }

    fn load_from_environment(&self) -> Result<Box<dyn ConfigProvider>> {
        todo!("Load configuration from environment")
    }

    fn load_from_command_line(&self, args: &[String]) -> Result<Box<dyn ConfigProvider>> {
        todo!("Load configuration from command line")
    }

    fn merge_configurations(&self, configs: Vec<Box<dyn ConfigProvider>>) -> Result<Box<dyn ConfigProvider>> {
        todo!("Merge multiple configurations")
    }

    fn detect_database_type(&self, connection_string: &str) -> DatabaseType {
        if connection_string.starts_with("sqlite:") {
            DatabaseType::SQLite
        } else if connection_string.starts_with("postgresql:") {
            DatabaseType::PostgreSQL
        } else {
            DatabaseType::Unknown
        }
    }

    fn extract_host(&self, connection_string: &str) -> Option<String> {
        // Parse host from connection string
        None
    }

    fn extract_port(&self, connection_string: &str) -> Option<u16> {
        // Parse port from connection string
        None
    }

    fn serialize_configuration(&self) -> Result<String> {
        // Serialize current configuration to string
        todo!("Serialize configuration")
    }
}

// Simplified types for facade users
#[derive(Debug, Clone)]
pub struct ConfigurationOptions {
    pub config_file: Option<PathBuf>,
    pub use_environment: bool,
    pub command_line_args: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct AnalysisSettings {
    pub default_language: Option<Language>,
    pub timeout_minutes: u64,
    pub max_file_size_mb: usize,
    pub parallel_processing: bool,
    pub enabled_detectors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub database_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ConfigurationStatus {
    pub valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}
```

### 5. Create Main Application Facade

#### A. Top-Level Unified Interface:
```rust
// src/facades/mod.rs
use crate::container::ServiceContainer;
use crate::error::Result;

pub mod analysis;
pub mod reporting;
pub mod configuration;

pub use analysis::AnalysisFacade;
pub use reporting::ReportingFacade;
pub use configuration::ConfigurationFacade;

/// Main application facade that provides unified access to all subsystems
pub struct UveddiFacade {
    analysis: AnalysisFacade,
    reporting: ReportingFacade,
    configuration: ConfigurationFacade,
    container: Arc<ServiceContainer>,
}

impl UveddiFacade {
    /// Initialize Uveddi with default configuration
    pub async fn initialize() -> Result<Self> {
        let configuration = ConfigurationFacade::new()?;
        Self::initialize_with_config(configuration).await
    }

    /// Initialize Uveddi with custom configuration
    pub async fn initialize_with_config(configuration: ConfigurationFacade) -> Result<Self> {
        // Build service container
        let container = ServiceContainer::build_from_config(&configuration).await?;

        // Create subsystem facades
        let analysis = AnalysisFacade::new(&container)?;
        let reporting = ReportingFacade::new(&container)?;

        Ok(Self {
            analysis,
            reporting,
            configuration,
            container: Arc::new(container),
        })
    }

    /// Access analysis subsystem
    pub fn analysis(&self) -> &AnalysisFacade {
        &self.analysis
    }

    /// Access reporting subsystem
    pub fn reporting(&self) -> &ReportingFacade {
        &self.reporting
    }

    /// Access configuration subsystem
    pub fn configuration(&self) -> &ConfigurationFacade {
        &self.configuration
    }

    /// Run complete analysis workflow
    pub async fn run_analysis_workflow(&self, config: AnalysisWorkflowConfig) -> Result<AnalysisWorkflowResult> {
        // 1. Analyze project
        let analysis_id = match config.analysis_config {
            WorkflowAnalysisConfig::Simple { target_path } => {
                self.analysis.analyze_simple(&target_path).await?
            }
            WorkflowAnalysisConfig::Advanced(advanced_config) => {
                let result = self.analysis.analyze_advanced(advanced_config).await?;
                // Save result to get ID (this could be improved)
                todo!("Convert AnalysisResult to ID")
            }
        };

        // 2. Generate reports
        let mut report_files = Vec::new();
        for report_config in config.report_configs {
            match report_config {
                ReportConfig::SingleFormat { output_path, format } => {
                    self.reporting.generate_report_with_format(&analysis_id, &output_path, format).await?;
                    report_files.push(output_path);
                }
                ReportConfig::MultiFormat { base_path, formats } => {
                    let files = self.reporting.generate_multi_format_reports(&analysis_id, &base_path, &formats).await?;
                    report_files.extend(files);
                }
                ReportConfig::Dashboard { output_dir, include_charts } => {
                    let dashboard_file = self.reporting.generate_dashboard(&analysis_id, &output_dir, include_charts).await?;
                    report_files.push(dashboard_file);
                }
            }
        }

        Ok(AnalysisWorkflowResult {
            analysis_id,
            report_files,
            summary: self.get_analysis_summary(&analysis_id).await?,
        })
    }

    /// Health check for all subsystems
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        let mut result = HealthCheckResult {
            overall_status: HealthStatus::Healthy,
            subsystem_status: std::collections::HashMap::new(),
        };

        // Check database connectivity
        // Check file system access
        // Check configuration validity
        // etc.

        Ok(result)
    }

    async fn get_analysis_summary(&self, analysis_id: &str) -> Result<WorkflowSummary> {
        let analysis = self.analysis.get_analysis_result(analysis_id).await?;
        Ok(WorkflowSummary {
            files_analyzed: analysis.summary.files_analyzed,
            issues_found: analysis.findings.len(),
            analysis_duration: analysis.summary.duration,
        })
    }
}

// Workflow configuration types
#[derive(Debug, Clone)]
pub struct AnalysisWorkflowConfig {
    pub analysis_config: WorkflowAnalysisConfig,
    pub report_configs: Vec<ReportConfig>,
}

#[derive(Debug, Clone)]
pub enum WorkflowAnalysisConfig {
    Simple { target_path: String },
    Advanced(AdvancedAnalysisConfig),
}

#[derive(Debug, Clone)]
pub enum ReportConfig {
    SingleFormat { output_path: PathBuf, format: ReportFormat },
    MultiFormat { base_path: PathBuf, formats: Vec<ReportFormat> },
    Dashboard { output_dir: PathBuf, include_charts: bool },
}

#[derive(Debug, Clone)]
pub struct AnalysisWorkflowResult {
    pub analysis_id: String,
    pub report_files: Vec<PathBuf>,
    pub summary: WorkflowSummary,
}

#[derive(Debug, Clone)]
pub struct WorkflowSummary {
    pub files_analyzed: usize,
    pub issues_found: usize,
    pub analysis_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub overall_status: HealthStatus,
    pub subsystem_status: std::collections::HashMap<String, HealthStatus>,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}
```

### 6. Create CLI Facade Integration

#### A. Update CLI to Use Facades:
```rust
// src/cli/commands.rs
use crate::facades::*;

pub async fn handle_analyze_command(args: AnalyzeArgs) -> Result<()> {
    // Initialize facade
    let uvedi = UveddiFacade::initialize().await?;

    match args.complexity {
        CommandComplexity::Simple => {
            let result_id = uvedi.analysis().analyze_simple(&args.target_path).await?;
            println!("Analysis completed. ID: {}", result_id);
        }
        CommandComplexity::Advanced => {
            let config = convert_args_to_advanced_config(args)?;
            let result = uvedi.analysis().analyze_advanced(config).await?;
            println!("Analysis completed. Found {} issues in {} files",
                result.findings.len(),
                result.summary.files_analyzed
            );
        }
    }

    Ok(())
}

pub async fn handle_report_command(args: ReportArgs) -> Result<()> {
    let uvedi = UveddiFacade::initialize().await?;

    match args.format {
        Some(format) => {
            uvedi.reporting()
                .generate_report_with_format(&args.analysis_id, &args.output_path, format)
                .await?;
        }
        None => {
            uvedi.reporting()
                .generate_report_auto(&args.analysis_id, &args.output_path)
                .await?;
        }
    }

    println!("Report generated: {}", args.output_path.display());
    Ok(())
}

pub async fn handle_workflow_command(args: WorkflowArgs) -> Result<()> {
    let uvedi = UveddiFacade::initialize().await?;

    let workflow_config = AnalysisWorkflowConfig {
        analysis_config: WorkflowAnalysisConfig::Simple {
            target_path: args.target_path,
        },
        report_configs: vec![
            ReportConfig::MultiFormat {
                base_path: args.output_base.clone(),
                formats: vec![ReportFormat::Html, ReportFormat::Json],
            },
            ReportConfig::Dashboard {
                output_dir: args.output_base.join("dashboard"),
                include_charts: true,
            },
        ],
    };

    let result = uvedi.run_analysis_workflow(workflow_config).await?;

    println!("Workflow completed!");
    println!("  Analysis ID: {}", result.analysis_id);
    println!("  Files analyzed: {}", result.summary.files_analyzed);
    println!("  Issues found: {}", result.summary.issues_found);
    println!("  Reports generated: {}", result.report_files.len());

    Ok(())
}
```

### 7. Create API Facade Integration

#### A. REST API Using Facades:
```rust
// src/api/handlers.rs (if web API exists)
use crate::facades::*;
use axum::{Json, extract::Path};

pub async fn analyze_project_endpoint(
    Json(request): Json<AnalyzeProjectRequest>,
) -> Result<Json<AnalyzeProjectResponse>, ApiError> {
    let uvedi = UveddiFacade::initialize().await.map_err(ApiError::from)?;

    let config = AdvancedAnalysisConfig {
        target_path: request.target_path,
        languages: request.languages.unwrap_or_default(),
        enabled_detectors: request.detectors.unwrap_or_else(|| vec![
            "god-object".to_string(),
            "security".to_string(),
        ]),
        output_format: OutputFormat::Json,
        parallel_execution: true,
        timeout_seconds: request.timeout_seconds,
        max_file_size_mb: None,
        exclude_patterns: request.exclude_patterns,
    };

    let result = uvedi.analysis().analyze_advanced(config).await.map_err(ApiError::from)?;

    Ok(Json(AnalyzeProjectResponse {
        analysis_id: "generated_id".to_string(), // Would be properly generated
        status: "completed".to_string(),
        findings_count: result.findings.len(),
        files_analyzed: result.summary.files_analyzed,
    }))
}

pub async fn generate_report_endpoint(
    Path(analysis_id): Path<String>,
    Json(request): Json<GenerateReportRequest>,
) -> Result<Json<GenerateReportResponse>, ApiError> {
    let uvedi = UveddiFacade::initialize().await.map_err(ApiError::from)?;

    let report_path = PathBuf::from(&request.output_path);
    uvedi.reporting()
        .generate_report_with_format(&analysis_id, &report_path, request.format)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(GenerateReportResponse {
        success: true,
        output_path: request.output_path,
        format: request.format,
    }))
}
```

### 8. Create Testing Support for Facades

#### A. Mock Facades for Testing:
```rust
// src/facades/testing.rs
#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct MockAnalysisFacade {
        pub analysis_results: std::sync::Mutex<std::collections::HashMap<String, AnalysisResult>>,
        pub should_fail: bool,
    }

    impl MockAnalysisFacade {
        pub fn new() -> Self {
            Self {
                analysis_results: std::sync::Mutex::new(std::collections::HashMap::new()),
                should_fail: false,
            }
        }

        pub fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        pub fn add_result(&self, id: String, result: AnalysisResult) {
            self.analysis_results.lock().unwrap().insert(id, result);
        }
    }

    impl MockAnalysisFacade {
        pub async fn analyze_simple(&self, _target_path: &str) -> Result<String> {
            if self.should_fail {
                return Err(UveddiError::Internal {
                    message: "Mock failure".to_string(),
                });
            }
            Ok("mock_analysis_id_123".to_string())
        }

        pub async fn get_analysis_result(&self, analysis_id: &str) -> Result<AnalysisResult> {
            self.analysis_results
                .lock()
                .unwrap()
                .get(analysis_id)
                .cloned()
                .ok_or_else(|| UveddiError::Database(DatabaseError::NotFound {
                    id: analysis_id.to_string(),
                }))
        }
    }

    // Similar mock implementations for other facades
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mocks::*;

    #[tokio::test]
    async fn test_facade_simple_analysis() {
        let facade = MockAnalysisFacade::new();
        let result = facade.analyze_simple("./test_project").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock_analysis_id_123");
    }

    #[tokio::test]
    async fn test_facade_failure_handling() {
        let facade = MockAnalysisFacade::new().with_failure();
        let result = facade.analyze_simple("./test_project").await;
        assert!(result.is_err());
    }
}
```

## Success Criteria
- [ ] Complex subsystem interactions simplified through facades
- [ ] Clean, user-friendly APIs for external consumers
- [ ] Internal complexity hidden from facade users
- [ ] Backward compatibility maintained when internal structure changes
- [ ] Facades easily testable with mock implementations
- [ ] CLI and API updated to use facades
- [ ] Comprehensive documentation for facade usage

## API Design Quality
- [ ] Facades provide intuitive method names
- [ ] Configuration types are user-friendly
- [ ] Error messages are clear and actionable
- [ ] Common use cases have simple methods
- [ ] Advanced use cases still possible but not required

## Verification Commands
```bash
# Test facade functionality
cargo test facades::

# Test CLI with facades
cargo run -- analyze ./src --simple
cargo run -- workflow ./src --output reports/

# Test facade error handling
cargo test facade_error_handling

# Check facade API documentation
cargo doc --open --package uveddi-facades
```

## Completion Notes
_To be filled by AI developer:_
- Facades implemented: ___
- API complexity reduction: ___
- CLI integration: ___
- Test coverage: ___
- Documentation quality: ___