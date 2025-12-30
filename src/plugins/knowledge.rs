//! Plugin System for Custom Knowledge
//!
//! This module implements a comprehensive plugin system that enables external developers
//! and organizations to extend the AI Knowledge Library with custom anti-patterns,
//! domain-specific knowledge, and specialized detection methods.

#[cfg(feature = "ai")]
use crate::ai::knowledge::context_selection::*;
#[cfg(feature = "ai")]
use crate::ai::knowledge::schema::*;
use crate::plugins::PluginError;
// Import stub types when AI features are disabled
#[cfg(not(feature = "ai"))]
use crate::plugins::integration::{
    AnalysisContext, AntiPatternCategory, DetectionMethod, LanguageKnowledge, LocationContext,
    PatternKnowledge, SeverityLevel, SolutionPattern, SourceLanguage,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "wasm-plugins")]
use std::sync::Mutex;
#[cfg(feature = "wasm-plugins")]
use wasmtime::component::{Component, Linker};
#[cfg(feature = "wasm-plugins")]
use wasmtime::{Engine, Store};
#[cfg(feature = "wasm-plugins")]
use crate::plugins::types::{HostContext, HostState, PluginConfig};
#[cfg(feature = "wasm-plugins")]
use crate::plugins::wasm::CoreAnalysis;

/// Plugin system manager for knowledge extensions
pub struct KnowledgePluginSystem {
    /// Plugin registry
    registry: KnowledgePluginRegistry,
    /// Plugin loader and validator
    loader: KnowledgePluginLoader,
    /// Plugin security manager
    security: KnowledgePluginSecurityManager,
    /// Plugin performance monitor
    performance: KnowledgePluginPerformanceMonitor,
    /// Plugin configuration
    config: KnowledgePluginSystemConfig,
    /// Active plugins
    active_plugins: Arc<RwLock<HashMap<String, Box<dyn KnowledgePlugin>>>>,
}

/// Plugin registry for managing knowledge plugins
pub struct KnowledgePluginRegistry {
    /// Registered plugins by ID
    plugins: HashMap<String, RegisteredKnowledgePlugin>,
    /// Plugin dependency graph
    dependencies: PluginDependencyGraph,
    /// Plugin metadata cache
    metadata_cache: HashMap<String, KnowledgePluginMetadata>,
}

/// Individual knowledge plugin registration
#[derive(Debug, Clone)]
pub struct RegisteredKnowledgePlugin {
    /// Plugin metadata
    pub metadata: KnowledgePluginMetadata,
    /// Plugin status
    pub status: KnowledgePluginStatus,
    /// Performance metrics
    pub metrics: KnowledgePluginMetrics,
}

/// Knowledge plugin metadata and manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePluginMetadata {
    /// Plugin unique identifier
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin description
    pub description: String,
    /// Plugin type
    pub plugin_type: KnowledgePluginType,
    /// Supported languages
    pub supported_languages: Vec<SourceLanguage>,
    /// Required dependencies
    pub dependencies: Vec<PluginDependency>,
    /// API version compatibility
    pub api_version: String,
    /// Plugin capabilities
    pub capabilities: KnowledgePluginCapabilities,
    /// Security permissions
    pub permissions: KnowledgePluginPermissions,
}

/// Types of knowledge plugins supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgePluginType {
    /// Anti-pattern definition plugin
    AntiPattern,
    /// Language support plugin
    Language,
    /// Framework-specific plugin
    Framework,
    /// Detection method plugin
    Detector,
    /// Solution pattern plugin
    Solution,
    /// Custom knowledge domain plugin
    Domain,
    /// Enterprise/proprietary plugin
    Enterprise,
}

/// Plugin capabilities declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePluginCapabilities {
    /// Provides anti-pattern definitions
    pub provides_patterns: bool,
    /// Provides detection methods
    pub provides_detectors: bool,
    /// Provides solution patterns
    pub provides_solutions: bool,
    /// Provides language support
    pub provides_language_support: bool,
    /// Provides framework knowledge
    pub provides_framework_knowledge: bool,
    /// Requires network access
    pub requires_network: bool,
    /// Requires file system access
    pub requires_filesystem: bool,
}

/// Plugin permissions for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePluginPermissions {
    /// Network access permissions
    pub network: NetworkPermissions,
    /// File system access permissions
    pub filesystem: FilesystemPermissions,
    /// System resource permissions
    pub system: SystemPermissions,
    /// Data access permissions
    pub data: DataPermissions,
}

/// Network permissions for knowledge plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// Allow outbound HTTP requests
    pub allow_http: bool,
    /// Allow outbound HTTPS requests
    pub allow_https: bool,
    /// Allowed domains
    pub allowed_domains: Vec<String>,
    /// Allowed ports
    pub allowed_ports: Vec<u16>,
}

/// Filesystem permissions for knowledge plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemPermissions {
    /// Allow read access
    pub allow_read: bool,
    /// Allow write access
    pub allow_write: bool,
    /// Allowed paths
    pub allowed_paths: Vec<PathBuf>,
}

/// System permissions for knowledge plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPermissions {
    /// Allow process execution
    pub allow_process_execution: bool,
    /// Allow environment variable access
    pub allow_env_access: bool,
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
}

/// Data access permissions for knowledge plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPermissions {
    /// Allow access to existing knowledge library
    pub allow_knowledge_read: bool,
    /// Allow modification of knowledge library
    pub allow_knowledge_write: bool,
    /// Allow access to analysis results
    pub allow_analysis_data: bool,
}

/// Plugin dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency plugin ID
    pub plugin_id: String,
    /// Required version range
    pub version_requirement: String,
    /// Whether dependency is optional
    pub optional: bool,
}

/// Plugin dependency graph for resolution
#[derive(Debug, Clone)]
pub struct PluginDependencyGraph {
    /// Adjacency list representation
    dependencies: HashMap<String, Vec<String>>,
    /// Reverse dependencies for impact analysis
    dependents: HashMap<String, Vec<String>>,
}

/// Plugin status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgePluginStatus {
    /// Plugin is installed but not active
    Installed,
    /// Plugin is active and running
    Active,
    /// Plugin is disabled
    Disabled,
    /// Plugin has errors
    Error(String),
    /// Plugin is being updated
    Updating,
}

/// Plugin performance metrics
#[derive(Debug, Clone)]
pub struct KnowledgePluginMetrics {
    /// Plugin initialization time
    pub init_time_ms: u64,
    /// Average pattern retrieval time
    pub avg_retrieval_time_ms: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Number of patterns provided
    pub pattern_count: usize,
    /// Success rate for operations
    pub success_rate: f64,
    /// Last health check timestamp
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Core trait that all knowledge plugins must implement
#[async_trait]
pub trait KnowledgePlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &KnowledgePluginMetadata;

    /// Initialize plugin
    async fn initialize(&mut self) -> Result<(), PluginError>;

    /// Shutdown plugin
    async fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Get plugin-provided patterns
    async fn get_patterns(&self) -> Result<Vec<PatternKnowledge>, PluginError>;

    /// Get plugin-provided detection methods
    async fn get_detectors(&self) -> Result<Vec<DetectionMethod>, PluginError>;

    /// Get plugin-provided solutions
    async fn get_solutions(&self) -> Result<Vec<SolutionPattern>, PluginError>;

    /// Get language-specific knowledge
    async fn get_language_knowledge(
        &self,
        language: SourceLanguage,
    ) -> Result<Option<LanguageKnowledge>, PluginError>;

    /// Get framework-specific knowledge
    async fn get_framework_knowledge(
        &self,
        framework: &str,
    ) -> Result<Option<FrameworkKnowledge>, PluginError>;

    /// Validate plugin health
    async fn health_check(&self) -> Result<PluginHealthStatus, PluginError>;
}

/// Trait for plugins that provide custom anti-patterns
#[async_trait]
pub trait AntiPatternPlugin: KnowledgePlugin {
    /// Get custom anti-pattern definitions
    async fn get_custom_patterns(&self) -> Result<Vec<CustomPatternDefinition>, PluginError>;

    /// Validate pattern against plugin's domain knowledge
    async fn validate_pattern(
        &self,
        pattern: &PatternKnowledge,
    ) -> Result<ValidationResult, PluginError>;
}

/// Trait for plugins that provide detection capabilities
#[async_trait]
pub trait DetectorPlugin: KnowledgePlugin {
    /// Analyze code for plugin-specific patterns
    async fn analyze_code(
        &self,
        code: &str,
        language: SourceLanguage,
        context: &AnalysisContext,
    ) -> Result<Vec<DetectedIssue>, PluginError>;

    /// Get detection confidence for a specific pattern
    async fn get_detection_confidence(
        &self,
        pattern_id: &str,
        context: &AnalysisContext,
    ) -> Result<f32, PluginError>;
}

/// Trait for enterprise/proprietary plugins
#[async_trait]
pub trait EnterprisePlugin: KnowledgePlugin {
    /// Get organization-specific patterns
    async fn get_organization_patterns(
        &self,
        org_id: &str,
    ) -> Result<Vec<PatternKnowledge>, PluginError>;

    /// Get compliance-related knowledge
    async fn get_compliance_knowledge(
        &self,
        compliance_framework: &str,
    ) -> Result<ComplianceKnowledge, PluginError>;

    /// Validate against organization policies
    async fn validate_against_policies(
        &self,
        code: &str,
        policies: &[OrganizationPolicy],
    ) -> Result<PolicyValidationResult, PluginError>;
}

/// Plugin health status
#[derive(Debug, Clone)]
pub enum PluginHealthStatus {
    /// Plugin is healthy and operating normally
    Healthy,
    /// Plugin has warnings but is still functional
    Warning(String),
    /// Plugin is unhealthy and may not function properly
    Unhealthy(String),
}

/// Custom pattern definition from plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPatternDefinition {
    /// Pattern identifier
    pub id: String,
    /// Pattern name
    pub name: String,
    /// Pattern definition
    pub definition: String,
    /// Pattern category
    pub category: AntiPatternCategory,
    /// Supported languages
    pub languages: Vec<SourceLanguage>,
    /// Detection methods
    pub detection_methods: Vec<DetectionMethod>,
    /// Solution patterns
    pub solutions: Vec<SolutionPattern>,
}

/// Framework-specific knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkKnowledge {
    /// Framework name
    pub name: String,
    /// Framework version
    pub version: String,
    /// Framework-specific patterns
    pub patterns: Vec<PatternKnowledge>,
    /// Best practices
    pub best_practices: Vec<String>,
    /// Common pitfalls
    pub pitfalls: Vec<String>,
}

/// Compliance knowledge for enterprise plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceKnowledge {
    /// Compliance framework name
    pub framework: String,
    /// Required patterns
    pub required_patterns: Vec<String>,
    /// Forbidden patterns
    pub forbidden_patterns: Vec<String>,
    /// Compliance rules
    pub rules: Vec<ComplianceRule>,
}

/// Compliance rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    /// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Severity level
    pub severity: SeverityLevel,
    /// Detection method
    pub detection: DetectionMethod,
}

/// Organization policy for enterprise validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationPolicy {
    /// Policy identifier
    pub id: String,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Applicable patterns
    pub patterns: Vec<String>,
    /// Enforcement level
    pub enforcement: EnforcementLevel,
}

/// Policy enforcement levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    /// Issue a warning but allow the operation to continue
    Warning,
    /// Report as an error but allow the operation to continue
    Error,
    /// Block the operation from proceeding
    Blocking,
}

/// Validation result from plugins
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation messages
    pub messages: Vec<String>,
    /// Confidence score
    pub confidence: f32,
}

/// Policy validation result
#[derive(Debug, Clone)]
pub struct PolicyValidationResult {
    /// Overall compliance status
    pub compliant: bool,
    /// Policy violations
    pub violations: Vec<PolicyViolation>,
    /// Compliance score
    pub score: f32,
}

/// Policy violation details
#[derive(Debug, Clone)]
pub struct PolicyViolation {
    /// Policy that was violated
    pub policy_id: String,
    /// Violation description
    pub description: String,
    /// Severity level
    pub severity: SeverityLevel,
    /// Location of violation
    pub location: LocationContext,
}

/// Detected issue from plugin analysis
#[derive(Debug, Clone)]
pub struct DetectedIssue {
    /// Issue identifier
    pub id: String,
    /// Issue description
    pub description: String,
    /// Severity level
    pub severity: SeverityLevel,
    /// Detection confidence
    pub confidence: f32,
    /// Location context
    pub location: LocationContext,
    /// Suggested solutions
    pub solutions: Vec<String>,
}

impl KnowledgePluginSystem {
    /// Create new knowledge plugin system
    pub fn new(config: KnowledgePluginSystemConfig) -> Self {
        Self {
            registry: KnowledgePluginRegistry::new(),
            loader: KnowledgePluginLoader::new(&config),
            security: KnowledgePluginSecurityManager::new(&config.security),
            performance: KnowledgePluginPerformanceMonitor::new(),
            config,
            active_plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Install and register a new knowledge plugin
    pub async fn install_plugin(&mut self, plugin_path: &PathBuf) -> Result<String, PluginError> {
        // 1. Load and validate plugin
        let plugin_package = self.loader.load_plugin(plugin_path).await?;

        // 2. Security validation
        self.security.validate_plugin(&plugin_package).await?;

        // 3. Dependency resolution
        self.resolve_dependencies(&plugin_package.metadata).await?;

        // 4. Performance validation
        self.performance
            .validate_performance(&plugin_package)
            .await?;

        // 5. Register plugin
        let plugin_id = plugin_package.metadata.id.clone();
        let registered_plugin = RegisteredKnowledgePlugin {
            metadata: plugin_package.metadata,
            status: KnowledgePluginStatus::Installed,
            metrics: KnowledgePluginMetrics::new(),
        };

        self.registry
            .register_plugin(plugin_id.clone(), registered_plugin)?;

        // 6. Initialize plugin
        self.initialize_plugin(&plugin_id).await?;

        Ok(plugin_id)
    }

    /// Load custom knowledge from all active plugins
    pub async fn load_plugin_knowledge(&self) -> Result<PluginKnowledgeLibrary, PluginError> {
        let mut plugin_knowledge = PluginKnowledgeLibrary::new();
        let active_plugins = self.active_plugins.read().await;

        for (plugin_id, plugin) in active_plugins.iter() {
            // Load knowledge from each active plugin
            let knowledge = self.load_knowledge_from_plugin(plugin_id, plugin).await?;
            plugin_knowledge.merge(knowledge)?;
        }

        // Validate and optimize plugin knowledge
        plugin_knowledge.validate()?;
        plugin_knowledge.optimize_for_performance()?;

        Ok(plugin_knowledge)
    }

    /// Get plugin-specific knowledge for context selection
    pub async fn get_plugin_context(
        &self,
        analysis_context: &AnalysisContext,
    ) -> Result<PluginContext, PluginError> {
        let mut plugin_context = PluginContext::new();
        let active_plugins = self.active_plugins.read().await;

        // Query relevant plugins based on analysis context
        let relevant_plugins = self.find_relevant_plugins(analysis_context)?;

        for plugin_id in relevant_plugins {
            if let Some(plugin) = active_plugins.get(&plugin_id) {
                let context = self
                    .get_context_from_plugin(&plugin_id, plugin, analysis_context)
                    .await?;
                plugin_context.add_plugin_context(plugin_id, context);
            }
        }

        Ok(plugin_context)
    }

    /// Initialize a plugin
    async fn initialize_plugin(&mut self, _plugin_id: &str) -> Result<(), PluginError> {
        // Implementation for plugin initialization
        // This would load the actual plugin implementation and call initialize()
        Ok(())
    }

    /// Resolve plugin dependencies
    async fn resolve_dependencies(
        &self,
        _metadata: &KnowledgePluginMetadata,
    ) -> Result<(), PluginError> {
        // Implementation for dependency resolution
        Ok(())
    }

    /// Load knowledge from a specific plugin
    async fn load_knowledge_from_plugin(
        &self,
        plugin_id: &str,
        plugin: &Box<dyn KnowledgePlugin>,
    ) -> Result<PluginKnowledge, PluginError> {
        let patterns = plugin.get_patterns().await?;
        let detectors = plugin.get_detectors().await?;
        let solutions = plugin.get_solutions().await?;

        Ok(PluginKnowledge {
            plugin_id: plugin_id.to_string(),
            patterns,
            detectors,
            solutions,
        })
    }

    /// Find relevant plugins for analysis context
    fn find_relevant_plugins(&self, context: &AnalysisContext) -> Result<Vec<String>, PluginError> {
        let mut relevant_plugins = Vec::new();

        for (plugin_id, plugin) in &self.registry.plugins {
            if self.is_plugin_relevant(plugin, context) {
                relevant_plugins.push(plugin_id.clone());
            }
        }

        Ok(relevant_plugins)
    }

    /// Check if plugin is relevant for analysis context
    fn is_plugin_relevant(
        &self,
        plugin: &RegisteredKnowledgePlugin,
        context: &AnalysisContext,
    ) -> bool {
        // Check language support
        if !plugin
            .metadata
            .supported_languages
            .contains(&context.language)
            && !plugin
                .metadata
                .supported_languages
                .contains(&SourceLanguage::Universal)
        {
            return false;
        }

        // Check framework relevance
        if !context.frameworks.is_empty() {
            // Plugin should support at least one of the detected frameworks
            // This would require additional metadata in the plugin
        }

        true
    }

    /// Get context from a specific plugin
    async fn get_context_from_plugin(
        &self,
        _plugin_id: &str,
        plugin: &Box<dyn KnowledgePlugin>,
        analysis_context: &AnalysisContext,
    ) -> Result<PluginSpecificContext, PluginError> {
        let language_knowledge = plugin
            .get_language_knowledge(analysis_context.language)
            .await?;

        let mut framework_knowledge = HashMap::new();
        for framework in &analysis_context.frameworks {
            if let Some(knowledge) = plugin.get_framework_knowledge(framework).await? {
                framework_knowledge.insert(framework.clone(), knowledge);
            }
        }

        Ok(PluginSpecificContext {
            language_knowledge,
            framework_knowledge,
        })
    }
}

/// Plugin knowledge collection
#[derive(Debug, Clone)]
pub struct PluginKnowledge {
    /// Source plugin ID
    pub plugin_id: String,
    /// Patterns provided by plugin
    pub patterns: Vec<PatternKnowledge>,
    /// Detection methods provided by plugin
    pub detectors: Vec<DetectionMethod>,
    /// Solutions provided by plugin
    pub solutions: Vec<SolutionPattern>,
}

/// Plugin-specific context for analysis
#[derive(Debug, Clone)]
pub struct PluginSpecificContext {
    /// Language-specific knowledge from plugin
    pub language_knowledge: Option<LanguageKnowledge>,
    /// Framework-specific knowledge from plugin
    pub framework_knowledge: HashMap<String, FrameworkKnowledge>,
}

/// Plugin context collection
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Context from each plugin
    plugin_contexts: HashMap<String, PluginSpecificContext>,
}

impl PluginContext {
    /// Create new plugin context
    pub fn new() -> Self {
        Self {
            plugin_contexts: HashMap::new(),
        }
    }

    /// Add context from a plugin
    pub fn add_plugin_context(&mut self, plugin_id: String, context: PluginSpecificContext) {
        self.plugin_contexts.insert(plugin_id, context);
    }

    /// Get context from a specific plugin
    pub fn get_plugin_context(&self, plugin_id: &str) -> Option<&PluginSpecificContext> {
        self.plugin_contexts.get(plugin_id)
    }

    /// Get all plugin contexts
    pub fn get_all_contexts(&self) -> &HashMap<String, PluginSpecificContext> {
        &self.plugin_contexts
    }
}

/// Plugin knowledge library containing all plugin-provided knowledge
#[derive(Debug, Clone)]
pub struct PluginKnowledgeLibrary {
    /// Knowledge from all plugins
    plugin_knowledge: HashMap<String, PluginKnowledge>,
    /// Merged pattern index
    pattern_index: HashMap<String, String>, // pattern_id -> plugin_id
    /// Performance metrics
    performance_metrics: PluginLibraryMetrics,
}

impl PluginKnowledgeLibrary {
    /// Create new plugin knowledge library
    pub fn new() -> Self {
        Self {
            plugin_knowledge: HashMap::new(),
            pattern_index: HashMap::new(),
            performance_metrics: PluginLibraryMetrics::new(),
        }
    }

    /// Merge knowledge from a plugin
    pub fn merge(&mut self, knowledge: PluginKnowledge) -> Result<(), PluginError> {
        let plugin_id = knowledge.plugin_id.clone();

        // Index patterns
        for pattern in &knowledge.patterns {
            self.pattern_index
                .insert(pattern.id.clone(), plugin_id.clone());
        }

        self.plugin_knowledge.insert(plugin_id, knowledge);
        Ok(())
    }

    /// Validate plugin knowledge library
    pub fn validate(&self) -> Result<(), PluginError> {
        // Validate no pattern ID conflicts
        // Validate pattern dependencies
        // Validate detection method compatibility
        Ok(())
    }

    /// Optimize for performance
    pub fn optimize_for_performance(&mut self) -> Result<(), PluginError> {
        // Build optimized indices
        // Compress pattern data
        // Pre-compute common queries
        Ok(())
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, pattern_id: &str) -> Option<&PatternKnowledge> {
        if let Some(plugin_id) = self.pattern_index.get(pattern_id) {
            if let Some(plugin_knowledge) = self.plugin_knowledge.get(plugin_id) {
                return plugin_knowledge
                    .patterns
                    .iter()
                    .find(|p| p.id == pattern_id);
            }
        }
        None
    }

    /// Get all patterns from a specific plugin
    pub fn get_plugin_patterns(&self, plugin_id: &str) -> Option<&Vec<PatternKnowledge>> {
        self.plugin_knowledge.get(plugin_id).map(|k| &k.patterns)
    }

    /// Get all plugin knowledge entries
    pub fn get_all_plugin_knowledge(&self) -> &HashMap<String, PluginKnowledge> {
        &self.plugin_knowledge
    }

    /// Get plugin knowledge by ID
    pub fn get_plugin_knowledge(&self, plugin_id: &str) -> Option<&PluginKnowledge> {
        self.plugin_knowledge.get(plugin_id)
    }

    /// Get number of plugins
    pub fn plugin_count(&self) -> usize {
        self.plugin_knowledge.len()
    }

    /// Get all plugin IDs
    pub fn plugin_ids(&self) -> Vec<String> {
        self.plugin_knowledge.keys().cloned().collect()
    }

    /// Get all patterns from all plugins
    pub fn get_all_patterns(&self) -> Vec<&PatternKnowledge> {
        self.plugin_knowledge
            .values()
            .flat_map(|k| k.patterns.iter())
            .collect()
    }
}

/// Performance metrics for plugin library
#[derive(Debug, Clone)]
pub struct PluginLibraryMetrics {
    /// Total number of plugins
    pub plugin_count: usize,
    /// Total number of patterns
    pub pattern_count: usize,
    /// Average retrieval time
    pub avg_retrieval_time_ms: f64,
    /// Memory usage
    pub memory_usage_bytes: usize,
}

impl PluginLibraryMetrics {
    /// Creates a new set of plugin library metrics
    pub fn new() -> Self {
        Self {
            plugin_count: 0,
            pattern_count: 0,
            avg_retrieval_time_ms: 0.0,
            memory_usage_bytes: 0,
        }
    }
}

impl KnowledgePluginMetrics {
    /// Creates a new set of knowledge plugin metrics
    pub fn new() -> Self {
        Self {
            init_time_ms: 0,
            avg_retrieval_time_ms: 0.0,
            memory_usage_bytes: 0,
            pattern_count: 0,
            success_rate: 1.0,
            last_health_check: chrono::Utc::now(),
        }
    }
}

impl KnowledgePluginRegistry {
    /// Creates a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            dependencies: PluginDependencyGraph::new(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Registers a new knowledge plugin
    pub fn register_plugin(
        &mut self,
        plugin_id: String,
        plugin: RegisteredKnowledgePlugin,
    ) -> Result<(), PluginError> {
        self.plugins.insert(plugin_id, plugin);
        Ok(())
    }
}

impl PluginDependencyGraph {
    /// Creates a new plugin dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }
}

/// Plugin system configuration
#[derive(Debug, Clone)]
pub struct KnowledgePluginSystemConfig {
    /// Security configuration
    pub security: SecurityConfig,
    /// Performance thresholds
    pub performance: PerformanceConfig,
    /// Plugin directories
    pub plugin_directories: Vec<PathBuf>,
    /// Enable plugin hot-reloading
    pub enable_hot_reload: bool,
}

/// Security configuration for knowledge plugins
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable strict sandboxing
    pub enable_sandboxing: bool,
    /// Maximum memory usage per plugin
    pub max_memory_per_plugin: usize,
    /// Maximum execution time per plugin operation
    pub max_execution_time_ms: u64,
    /// Enable security auditing
    pub enable_security_auditing: bool,
}

/// Performance configuration for knowledge plugins
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum plugin initialization time
    pub max_init_time_ms: u64,
    /// Maximum pattern retrieval time
    pub max_pattern_retrieval_ms: u64,
    /// Maximum memory usage per plugin
    pub max_memory_usage_mb: usize,
    /// Maximum plugin count before performance degradation
    pub max_active_plugins: usize,
}

/// Plugin loader for knowledge plugins
pub struct KnowledgePluginLoader {
    config: KnowledgePluginSystemConfig,
}

impl KnowledgePluginLoader {
    /// Creates a new plugin loader with the given configuration
    pub fn new(config: &KnowledgePluginSystemConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn load_plugin(
        &self,
        plugin_path: &PathBuf,
    ) -> Result<KnowledgePluginPackage, PluginError> {
        #[cfg(feature = "wasm-plugins")]
        {
             // 1. Read plugin binary
            let binary = tokio::fs::read(plugin_path).await
                .map_err(|e| PluginError::Loading(format!("Failed to read plugin binary: {}", e)))?;
            
            // 2. Parse manifest from sibling .toml file (simple assumption for now)
            // In a real implementation we might embedded it or use a proper manifest loader
            // For now, construct a default metadata
            let plugin_id = plugin_path.file_stem().unwrap().to_string_lossy().to_string();
            let metadata = KnowledgePluginMetadata {
                id: plugin_id.clone(),
                name: plugin_id,
                version: "0.1.0".to_string(),
                description: "Loaded via knowledge system".to_string(),
                author: "Unknown".to_string(),
                plugin_type: KnowledgePluginType::Detector,
                supported_languages: vec![SourceLanguage::Universal],
                dependencies: vec![],
                api_version: "1.0.0".to_string(),
                capabilities: KnowledgePluginCapabilities {
                    provides_patterns: false,
                    provides_detectors: true,
                    provides_solutions: false,
                    provides_language_support: false,
                    provides_framework_knowledge: false,
                    requires_network: false,
                    requires_filesystem: false,
                },
                permissions: KnowledgePluginPermissions {
                    network: NetworkPermissions {
                        allow_http: false,
                        allow_https: false,
                        allowed_domains: vec![],
                        allowed_ports: vec![],
                    },
                    filesystem: FilesystemPermissions {
                        allow_read: true,
                        allow_write: false,
                        allowed_paths: vec![],
                    },
                    system: SystemPermissions {
                        allow_process_execution: false,
                        allow_env_access: false,
                        max_memory_mb: 256,
                        max_execution_time_ms: 30000,
                    },
                    data: DataPermissions {
                        allow_knowledge_read: true,
                        allow_knowledge_write: false,
                        allow_analysis_data: true,
                    },
                },
            };

            // 3. Create Component and instantiate
            // Configure Engine (simplified)
            let mut config = wasmtime::Config::new();
            config.wasm_component_model(true);
            let engine = Engine::new(&config).map_err(|e| PluginError::Loading(e.to_string()))?;

            let component = Component::new(&engine, &binary)
                .map_err(|e| PluginError::Loading(format!("Failed to create component: {}", e)))?;

            let mut linker = Linker::<HostContext>::new(&engine);

            // Add host functions (skip WASI Preview 1 as it's incompatible with Component Model Linker)
            crate::plugins::wasm::CoreAnalysis::add_to_linker::<_, wasmtime::component::HasSelf<HostContext>>(&mut linker, |host| host)
                .map_err(|e| PluginError::Loading(e.to_string()))?;

            // Create host state and WASI context
            let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new()
                .inherit_stdio()
                .build_p1();

            let host_state = crate::plugins::types::HostState {
                plugin_id: crate::plugins::types::PluginId::from_name(&metadata.id),
                config: crate::plugins::types::PluginConfig::default(),
                resource_limits: crate::plugins::types::ResourceLimits::default(),
                security_policy: crate::plugins::SecurityPolicy::default(),
            };

            // Create database and analysis engine
            let database_config = crate::database::DatabaseConfig::default();
            let database = crate::database::ScalableDatabase::new(database_config).await
                .map_err(|e| PluginError::Loading(format!("Failed to create database: {}", e)))?;
            let analysis_engine = crate::analysis::AnalysisEngine::new()
                .map_err(|e| PluginError::Loading(format!("Failed to create analysis engine: {}", e)))?;

            let host_context = HostContext {
                host_state,
                wasi_ctx,
                table: wasmtime::component::ResourceTable::new(),
                database: Arc::new(database),
                analysis_engine: Arc::new(RwLock::new(analysis_engine)),
                config_store: Arc::new(RwLock::new(std::collections::HashMap::new())),
                ast_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            };

            let mut store = Store::new(&engine, host_context);

            // Instantiate (sync, not async)
            let bindings = CoreAnalysis::instantiate(&mut store, &component, &linker)
                 .map_err(|e| PluginError::Loading(format!("Failed to instantiate: {}", e)))?;
                 
            // Create adapter
            let adapter = WasmKnowledgeAdapter {
                store: Arc::new(Mutex::new(store)),
                bindings,
                metadata: metadata.clone(),
            };
            
            Ok(KnowledgePluginPackage {
                metadata,
                implementation: Box::new(adapter),
            })
        }
        
        #[cfg(not(feature = "wasm-plugins"))]
        {
            Err(PluginError::Unsupported("WASM plugins not enabled".to_string()))
        }
    }
}

#[cfg(feature = "wasm-plugins")]
struct WasmKnowledgeAdapter {
    store: Arc<Mutex<Store<HostContext>>>,
    bindings: CoreAnalysis,
    metadata: KnowledgePluginMetadata,
}

#[cfg(feature = "wasm-plugins")]
#[async_trait]
impl KnowledgePlugin for WasmKnowledgeAdapter {
    fn metadata(&self) -> &KnowledgePluginMetadata { &self.metadata }

    async fn initialize(&mut self) -> Result<(), PluginError> { Ok(()) }
    async fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }

    async fn get_patterns(&self) -> Result<Vec<PatternKnowledge>, PluginError> {
        Ok(Vec::new()) // Placeholder
    }

    async fn get_detectors(&self) -> Result<Vec<DetectionMethod>, PluginError> {
        Ok(Vec::new()) // Placeholder
    }

    async fn get_solutions(&self) -> Result<Vec<SolutionPattern>, PluginError> {
        Ok(Vec::new()) // Placeholder
    }

    async fn get_language_knowledge(
        &self,
        _language: SourceLanguage,
    ) -> Result<Option<LanguageKnowledge>, PluginError> {
        Ok(None)
    }

    async fn get_framework_knowledge(
        &self,
        _framework: &str,
    ) -> Result<Option<FrameworkKnowledge>, PluginError> {
        Ok(None)
    }

    async fn health_check(&self) -> Result<PluginHealthStatus, PluginError> {
        Ok(PluginHealthStatus::Healthy)
    }
}

#[cfg(feature = "wasm-plugins")]
#[async_trait]
impl DetectorPlugin for WasmKnowledgeAdapter {
    async fn analyze_code(
        &self,
        code: &str,
        language: SourceLanguage,
        _context: &AnalysisContext,
    ) -> Result<Vec<DetectedIssue>, PluginError> {
        let mut store_guard = self.store.lock().map_err(|e| PluginError::Execution(e.to_string()))?;
        
        // Map types and call bindings like in lifecycle.rs
        // Simplified for brevity
        Ok(Vec::new()) 
    }

    async fn get_detection_confidence(
        &self,
        _pattern_id: &str,
        _context: &AnalysisContext,
    ) -> Result<f32, PluginError> {
        Ok(1.0)
    }
}

/// Plugin package containing metadata and implementation
pub struct KnowledgePluginPackage {
    /// Plugin metadata information
    pub metadata: KnowledgePluginMetadata,
    /// Plugin implementation
    pub implementation: Box<dyn KnowledgePlugin>,
}

/// Plugin security manager for knowledge plugins
pub struct KnowledgePluginSecurityManager {
    config: SecurityConfig,
}

impl KnowledgePluginSecurityManager {
    /// Creates a new plugin security manager
    pub fn new(config: &SecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Validates plugin security requirements
    pub async fn validate_plugin(
        &self,
        _package: &KnowledgePluginPackage,
    ) -> Result<(), PluginError> {
        // Implementation for security validation
        // This would check permissions, scan for vulnerabilities, etc.
        Ok(())
    }
}

/// Plugin performance monitor for knowledge plugins
pub struct KnowledgePluginPerformanceMonitor {
    // Performance monitoring implementation
}

impl KnowledgePluginPerformanceMonitor {
    /// Creates a new performance monitor
    pub fn new() -> Self {
        Self {}
    }

    /// Validates plugin performance requirements
    pub async fn validate_performance(
        &self,
        _package: &KnowledgePluginPackage,
    ) -> Result<(), PluginError> {
        // Implementation for performance validation
        // This would test initialization time, memory usage, etc.
        Ok(())
    }
}
