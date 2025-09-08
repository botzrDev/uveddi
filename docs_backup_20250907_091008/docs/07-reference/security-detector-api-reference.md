# Security Detector API Reference

## Overview

This document provides comprehensive API reference for the Uveddi Security Detector system. The security detector provides vulnerability analysis, multi-agent coordination, and comprehensive security reporting capabilities.

## Table of Contents

1. [Core Detector API](#core-detector-api)
2. [Multi-Agent System API](#multi-agent-system-api)
3. [Detection Strategy API](#detection-strategy-api)
4. [Taint Analysis API](#taint-analysis-api)
5. [Configuration API](#configuration-api)
6. [Knowledge Graph API](#knowledge-graph-api)
7. [Validation and Scoring API](#validation-and-scoring-api)
8. [Types and Enums](#types-and-enums)
9. [Error Handling](#error-handling)
10. [Examples](#examples)

## Core Detector API

### SecurityDetector

Main security detector interface that orchestrates vulnerability analysis.

```rust
pub struct MainSecurityDetector {
    orchestrator: SecurityOrchestrator,
    config: SecurityConfig,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
    vulnerability_db: Arc<VulnerabilityDatabase>,
}
```

#### Methods

##### `new() -> Result<Self, AnalysisError>`

Creates a new security detector with default configuration.

```rust
let detector = MainSecurityDetector::new()?;
```

##### `with_config(config: SecurityConfig) -> Result<Self, AnalysisError>`

Creates a new security detector with custom configuration.

```rust
let config = SecurityConfig {
    enable_taint_analysis: true,
    enable_sca: true,
    confidence_threshold: 0.8,
    ..Default::default()
};
let detector = MainSecurityDetector::with_config(config)?;
```

##### `analyze_file(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError>`

Analyzes a single parsed file for security vulnerabilities.

```rust
let issues = detector.analyze_file(&parsed_file).await?;
for issue in issues {
    println!("Found {}: {}", issue.issue_type, issue.title);
}
```

##### `analyze_codebase(&self, files: &[ParsedFile]) -> Result<SecurityAnalysisResult, AnalysisError>`

Analyzes an entire codebase with comprehensive reporting.

```rust
let result = detector.analyze_codebase(&parsed_files).await?;
println!("Total issues: {}", result.total_issues);
println!("Critical: {}", result.critical_count);
```

#### Trait Implementations

##### `AnalysisDetector`

The security detector implements the standard `AnalysisDetector` trait for integration with the Uveddi analysis engine.

```rust
#[async_trait]
impl AnalysisDetector for MainSecurityDetector {
    async fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    fn get_detector_name(&self) -> &'static str;
}
```

## Multi-Agent System API

### SecurityOrchestrator

Coordinates multiple security analysis agents for comprehensive vulnerability detection.

```rust
pub struct SecurityOrchestrator {
    agents: Vec<Box<dyn SecurityAgent>>,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
    validation_engine: ValidationEngine,
    config: MultiAgentConfig,
}
```

#### Methods

##### `new(config: MultiAgentConfig, knowledge_graph: Arc<SecurityKnowledgeGraph>, vulnerability_db: Arc<VulnerabilityDatabase>) -> Result<Self, AnalysisError>`

Creates a new security orchestrator with specified configuration.

```rust
let config = MultiAgentConfig {
    enable_taint_analysis: true,
    enable_config_analysis: true,
    enable_sca: true,
    parallel_execution: true,
    confidence_threshold: 0.7,
    ..Default::default()
};

let orchestrator = SecurityOrchestrator::new(
    config,
    knowledge_graph,
    vulnerability_db,
)?;
```

##### `analyze_file(&self, context: &SecurityContext) -> Result<SecurityAnalysisResult, AnalysisError>`

Performs multi-agent analysis on a single file context.

```rust
let context = SecurityContext::from_parsed_file(&file)?;
let result = orchestrator.analyze_file(&context).await?;
```

##### `analyze_codebase(&self, dependency_graph: &DependencyGraph) -> Result<SecurityAnalysisResult, AnalysisError>`

Performs comprehensive codebase analysis using dependency information.

```rust
let result = orchestrator.analyze_codebase(&dependency_graph).await?;
```

### SecurityAgent Trait

Interface for individual security analysis agents.

```rust
#[async_trait]
pub trait SecurityAgent: Send + Sync {
    async fn execute_task(&self, task_id: String, context: SecurityContext) -> Result<AgentResult, AnalysisError>;
    fn get_agent_id(&self) -> &str;
    fn get_capabilities(&self) -> Vec<TaskType>;
    fn clone_box(&self) -> Box<dyn SecurityAgent>;
}
```

#### Implementation Example

```rust
pub struct CustomSecurityAgent {
    agent_id: String,
    capabilities: Vec<TaskType>,
    config: CustomAgentConfig,
}

#[async_trait]
impl SecurityAgent for CustomSecurityAgent {
    async fn execute_task(&self, task_id: String, context: SecurityContext) -> Result<AgentResult, AnalysisError> {
        // Custom analysis logic
        let issues = self.perform_analysis(&context).await?;
        
        Ok(AgentResult {
            agent_id: self.agent_id.clone(),
            task_id,
            issues,
            execution_time: start_time.elapsed(),
            confidence_scores: self.calculate_confidence(&issues),
            metadata: HashMap::new(),
        })
    }
    
    fn get_agent_id(&self) -> &str { &self.agent_id }
    fn get_capabilities(&self) -> Vec<TaskType> { self.capabilities.clone() }
    fn clone_box(&self) -> Box<dyn SecurityAgent> { Box::new(self.clone()) }
}
```

### Built-in Agents

#### TaintAnalysisAgent

Performs data flow analysis to detect injection vulnerabilities.

```rust
pub struct TaintAnalysisAgent {
    engine: TaintAnalysisEngine,
    config: TaintAnalysisConfig,
}

impl TaintAnalysisAgent {
    pub fn new(config: TaintAnalysisConfig) -> Result<Self, AnalysisError>;
    pub fn add_custom_source(&mut self, source: TaintSource);
    pub fn add_custom_sink(&mut self, sink: TaintSink);
    pub fn add_custom_sanitizer(&mut self, sanitizer: SanitizationPoint);
}
```

#### ConfigAnalysisAgent

Analyzes configuration files for security misconfigurations.

```rust
pub struct ConfigAnalysisAgent {
    analyzers: HashMap<String, Box<dyn ConfigAnalyzer>>,
    rule_engine: ConfigRuleEngine,
}

impl ConfigAnalysisAgent {
    pub fn new() -> Self;
    pub fn add_custom_analyzer(&mut self, file_type: String, analyzer: Box<dyn ConfigAnalyzer>);
    pub fn add_custom_rule(&mut self, rule: ConfigSecurityRule);
}
```

#### DependencyAgent

Performs Software Composition Analysis (SCA) for vulnerable dependencies.

```rust
pub struct DependencyAgent {
    vulnerability_db: Arc<VulnerabilityDatabase>,
    parsers: HashMap<SourceLanguage, Box<dyn DependencyParser>>,
}

impl DependencyAgent {
    pub fn new(vulnerability_db: Arc<VulnerabilityDatabase>) -> Self;
    pub fn update_vulnerability_database(&mut self) -> Result<(), AnalysisError>;
    pub fn add_custom_parser(&mut self, language: SourceLanguage, parser: Box<dyn DependencyParser>);
}
```

#### ValidationAgent

Cross-validates findings to reduce false positives.

```rust
pub struct ValidationAgent {
    false_positive_mitigator: FalsePositiveMitigator,
    confidence_calculator: ConfidenceCalculator,
    bayesian_optimizer: BayesianOptimizer,
}

impl ValidationAgent {
    pub fn new(config: ValidationConfig) -> Self;
    pub fn validate_findings(&self, findings: &[SecurityIssue]) -> Result<Vec<SecurityIssue>, AnalysisError>;
    pub fn calculate_cross_validation_score(&self, findings: &[SecurityIssue]) -> f64;
}
```

## Detection Strategy API

### DetectionStrategy Trait

Interface for security detection strategies.

```rust
#[async_trait]
pub trait DetectionStrategy: Send + Sync {
    async fn analyze(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError>;
    fn get_name(&self) -> &'static str;
    fn get_supported_languages(&self) -> Vec<SourceLanguage>;
    fn get_confidence_level(&self) -> f64;
}
```

### Built-in Strategies

#### DeterministicPatternMatcher

Fast pattern-based vulnerability detection.

```rust
pub struct DeterministicPatternMatcher {
    patterns: HashMap<SourceLanguage, Vec<VulnerabilityPattern>>,
}

impl DeterministicPatternMatcher {
    pub fn new() -> Self;
    pub fn add_custom_pattern(&mut self, language: SourceLanguage, pattern: VulnerabilityPattern);
    pub fn remove_pattern(&mut self, language: SourceLanguage, pattern_name: &str);
    pub fn get_patterns_for_language(&self, language: SourceLanguage) -> Option<&Vec<VulnerabilityPattern>>;
}
```

#### ConfigFileAnalyzer

Configuration security analysis.

```rust
pub struct ConfigFileAnalyzer {
    config_patterns: HashMap<String, Vec<ConfigSecurityPattern>>,
}

impl ConfigFileAnalyzer {
    pub fn new() -> Self;
    pub fn add_config_pattern(&mut self, file_type: String, pattern: ConfigSecurityPattern);
    pub fn analyze_config_file(&self, file_path: &Path) -> Result<Vec<SecurityIssue>, AnalysisError>;
}
```

#### SoftwareCompositionAnalyzer

Dependency vulnerability analysis.

```rust
pub struct SoftwareCompositionAnalyzer {
    // Private fields
}

impl SoftwareCompositionAnalyzer {
    pub fn new() -> Self;
    pub async fn analyze_cargo_toml(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError>;
    pub async fn analyze_package_json(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError>;
    pub async fn analyze_requirements_txt(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError>;
}
```

#### VulnerabilityCorrelationEngine

Maps security issues to architectural anti-patterns.

```rust
pub struct VulnerabilityCorrelationEngine {
    correlation_rules: HashMap<SecurityIssueType, Vec<ArchitecturalCorrelationRule>>,
}

impl VulnerabilityCorrelationEngine {
    pub fn new() -> Self;
    pub fn correlate_issue(&self, security_issue: &SecurityIssue, detected_anti_patterns: &[String]) -> Vec<SecurityCorrelation>;
    pub fn add_correlation_rule(&mut self, issue_type: SecurityIssueType, rule: ArchitecturalCorrelationRule);
}
```

## Taint Analysis API

### TaintAnalysisEngine

Core taint analysis functionality for tracking data flows.

```rust
pub struct TaintAnalysisEngine {
    sources: Vec<TaintSource>,
    sinks: Vec<TaintSink>,
    sanitizers: Vec<SanitizationPoint>,
    config: TaintAnalysisConfig,
}
```

#### Methods

##### `new() -> Self`

Creates a new taint analysis engine with default configuration.

```rust
let engine = TaintAnalysisEngine::new();
```

##### `with_config(config: TaintAnalysisConfig) -> Self`

Creates a new taint analysis engine with custom configuration.

```rust
let config = TaintAnalysisConfig {
    max_depth: 10,
    cross_function_analysis: true,
    inter_procedural_analysis: true,
    ..Default::default()
};
let engine = TaintAnalysisEngine::with_config(config);
```

##### `add_source(&mut self, source: TaintSource)`

Adds a custom taint source.

```rust
let source = TaintSource {
    name: "user_input".to_string(),
    source_type: SourceType::UserInput,
    language: SourceLanguage::Python,
    patterns: vec!["request.GET".to_string(), "request.POST".to_string()],
};
engine.add_source(source);
```

##### `add_sink(&mut self, sink: TaintSink)`

Adds a custom taint sink.

```rust
let sink = TaintSink {
    name: "sql_execution".to_string(),
    sink_type: SinkType::SqlQuery,
    language: SourceLanguage::Python,
    patterns: vec!["cursor.execute".to_string(), "db.query".to_string()],
};
engine.add_sink(sink);
```

##### `add_sanitizer(&mut self, sanitizer: SanitizationPoint)`

Adds a custom sanitization point.

```rust
let sanitizer = SanitizationPoint {
    name: "html_escape".to_string(),
    sanitizer_type: SanitizerType::HtmlEscape,
    effectiveness: 0.95,
    applicable_vulnerabilities: vec![SecurityIssueType::CrossSiteScripting],
    patterns: vec!["html.escape".to_string()],
};
engine.add_sanitizer(sanitizer);
```

##### `analyze(&self, file: &ParsedFile) -> Result<TaintAnalysisResult, AnalysisError>`

Performs taint analysis on a parsed file.

```rust
let result = engine.analyze(&parsed_file).await?;
for flow in result.vulnerable_flows {
    println!("Taint flow: {} -> {}", flow.source.name, flow.sink.name);
}
```

### Data Flow Types

#### TaintSource

Represents a source of potentially malicious data.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintSource {
    pub name: String,
    pub source_type: SourceType,
    pub language: SourceLanguage,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    UserInput,
    NetworkRequest,
    FileRead,
    DatabaseRead,
    EnvironmentVariable,
    CommandLineArgument,
    Custom(String),
}
```

#### TaintSink

Represents a location where tainted data could cause harm.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintSink {
    pub name: String,
    pub sink_type: SinkType,
    pub language: SourceLanguage,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinkType {
    SqlQuery,
    CommandExecution,
    FileWrite,
    NetworkSend,
    HtmlOutput,
    JavaScriptExecution,
    TemplateInjection,
    Custom(String),
}
```

#### SanitizationPoint

Represents a function or operation that sanitizes tainted data.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationPoint {
    pub name: String,
    pub sanitizer_type: SanitizerType,
    pub effectiveness: f64,
    pub applicable_vulnerabilities: Vec<SecurityIssueType>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanitizerType {
    HtmlEscape,
    SqlParameterization,
    InputValidation,
    PathNormalization,
    CommandEscape,
    Custom(String),
}
```

## Configuration API

### SecurityConfig

Main configuration for the security detector system.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_taint_analysis: bool,
    pub enable_config_analysis: bool,
    pub enable_sca: bool,
    pub enable_ai_enhancement: bool,
    pub confidence_threshold: f64,
    pub multi_agent: MultiAgentConfig,
    pub validation: ValidationConfig,
    pub performance: PerformanceConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_config_analysis: true,
            enable_sca: true,
            enable_ai_enhancement: false,
            confidence_threshold: 0.7,
            multi_agent: MultiAgentConfig::default(),
            validation: ValidationConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}
```

### MultiAgentConfig

Configuration for the multi-agent system.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentConfig {
    pub parallel_execution: bool,
    pub max_concurrent_agents: usize,
    pub agent_timeout_seconds: u64,
    pub enable_cross_validation: bool,
    pub confidence_threshold: f64,
    pub agent_specific: HashMap<String, AgentConfig>,
}
```

### TaintAnalysisConfig

Configuration for taint analysis.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintAnalysisConfig {
    pub max_depth: usize,
    pub timeout_seconds: u64,
    pub cross_function_analysis: bool,
    pub inter_procedural_analysis: bool,
    pub enable_sanitizer_tracking: bool,
    pub confidence_adjustment_factor: f64,
}
```

### ValidationConfig

Configuration for result validation and false positive mitigation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub enable_false_positive_mitigation: bool,
    pub enable_confidence_scoring: bool,
    pub enable_bayesian_optimization: bool,
    pub cross_validation_threshold: usize,
    pub historical_data_weight: f64,
    pub context_analysis_weight: f64,
}
```

## Knowledge Graph API

### SecurityKnowledgeGraph

Provides contextual security intelligence through knowledge graphs.

```rust
pub struct SecurityKnowledgeGraph {
    structural_graph: StructuralSemanticGraph,
    code_rag: CodeCentricRAG,
    knowledge_base: KnowledgeBase,
}

impl SecurityKnowledgeGraph {
    pub fn new() -> Result<Self, AnalysisError>;
    pub fn add_code_context(&mut self, context: CodeContext) -> Result<(), AnalysisError>;
    pub fn query_vulnerability_context(&self, issue_type: SecurityIssueType) -> Result<VulnerabilityContext, AnalysisError>;
    pub fn get_remediation_suggestions(&self, issue: &SecurityIssue) -> Result<Vec<RemediationSuggestion>, AnalysisError>;
}
```

### StructuralSemanticGraph

Graph-based representation of code structure and semantics.

```rust
pub struct StructuralSemanticGraph {
    nodes: HashMap<NodeId, GraphNode>,
    edges: Vec<GraphEdge>,
    metadata: GraphMetadata,
}

impl StructuralSemanticGraph {
    pub fn new() -> Self;
    pub fn add_node(&mut self, node: GraphNode) -> NodeId;
    pub fn add_edge(&mut self, edge: GraphEdge);
    pub fn find_shortest_path(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>>;
    pub fn get_neighbors(&self, node: NodeId) -> Vec<NodeId>;
}
```

### CodeCentricRAG

Retrieval-Augmented Generation for code-specific security analysis.

```rust
pub struct CodeCentricRAG {
    vector_store: VectorStore,
    embedding_model: EmbeddingModel,
    retrieval_config: RetrievalConfig,
}

impl CodeCentricRAG {
    pub fn new(config: RAGConfig) -> Result<Self, AnalysisError>;
    pub fn index_code_patterns(&mut self, patterns: &[CodePattern]) -> Result<(), AnalysisError>;
    pub fn retrieve_similar_patterns(&self, query: &str, limit: usize) -> Result<Vec<CodePattern>, AnalysisError>;
    pub fn generate_context_aware_explanation(&self, issue: &SecurityIssue) -> Result<String, AnalysisError>;
}
```

## Validation and Scoring API

### ConfidenceCalculator

Calculates confidence scores for security findings.

```rust
pub struct ConfidenceCalculator {
    scoring_model: ScoringModel,
    historical_data: HistoricalData,
    config: ConfidenceConfig,
}

impl ConfidenceCalculator {
    pub fn new(config: ConfidenceConfig) -> Self;
    pub fn calculate_confidence(&self, issue: &SecurityIssue, context: &SecurityContext) -> f64;
    pub fn update_historical_data(&mut self, validation_results: &[ValidationResult]);
    pub fn get_confidence_distribution(&self) -> ConfidenceDistribution;
}
```

### FalsePositiveMitigator

Reduces false positives through advanced validation techniques.

```rust
pub struct FalsePositiveMitigator {
    mitigation_strategies: Vec<Box<dyn MitigationStrategy>>,
    threshold_config: ThresholdConfig,
}

impl FalsePositiveMitigator {
    pub fn new(config: FalsePositiveConfig) -> Self;
    pub fn mitigate_false_positives(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError>;
    pub fn add_mitigation_strategy(&mut self, strategy: Box<dyn MitigationStrategy>);
    pub fn get_mitigation_statistics(&self) -> MitigationStatistics;
}
```

### BayesianOptimizer

Optimizes detection parameters using Bayesian methods.

```rust
pub struct BayesianOptimizer {
    optimization_history: OptimizationHistory,
    current_parameters: ParameterSet,
    objective_function: ObjectiveFunction,
}

impl BayesianOptimizer {
    pub fn new(config: BayesianConfig) -> Self;
    pub fn optimize_parameters(&mut self, validation_data: &[ValidationResult]) -> Result<ParameterSet, AnalysisError>;
    pub fn suggest_next_parameters(&self) -> Result<ParameterSet, AnalysisError>;
    pub fn update_with_results(&mut self, parameters: ParameterSet, performance: f64);
}
```

## Types and Enums

### SecurityIssue

Primary type representing a security vulnerability.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub id: Option<String>,
    pub issue_type: SecurityIssueType,
    pub vulnerability_type: VulnerabilityType,
    pub title: String,
    pub description: String,
    pub location: SecurityLocation,
    pub severity: SecuritySeverity,
    pub confidence_score: f64,
    pub language: SourceLanguage,
    pub remediation: String,
    pub metadata: VulnerabilityMetadata,
    pub created_at: DateTime<Utc>,
    pub detector: String,
}
```

### SecurityIssueType

Enumeration of security issue categories based on OWASP Top 10.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityIssueType {
    // OWASP Top 10 2021
    BrokenAccessControl,
    CryptographicFailures,
    Injection,
    InsecureDesign,
    SecurityMisconfiguration,
    VulnerableComponents,
    AuthenticationFailures,
    SoftwareDataIntegrityFailures,
    SecurityLoggingFailures,
    ServerSideRequestForgery,
    
    // Additional categories
    CrossSiteScripting,
    HardcodedSecrets,
    ImproperErrorHandling,
    UseAfterFree,
    DeserializationVulnerabilities,
    PrivilegeEscalation,
    
    // Custom category
    Custom(String),
}
```

### SecuritySeverity

Security issue severity levels.

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}
```

### VulnerabilityType

Classification of vulnerability detection method.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilityType {
    Static,        // Static analysis detection
    Dynamic,       // Dynamic analysis detection
    DataFlow,      // Taint analysis detection
    Configuration, // Configuration analysis detection
    Dependency,    // SCA detection
    Behavioral,    // Behavioral pattern detection
}
```

### SecurityLocation

Location information for security issues.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLocation {
    pub file_path: PathBuf,
    pub start_line: i32,
    pub end_line: i32,
    pub start_column: Option<i32>,
    pub end_column: Option<i32>,
    pub function_name: Option<String>,
    pub class_name: Option<String>,
}
```

### VulnerabilityMetadata

Additional metadata for security issues.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityMetadata {
    pub cwe_id: Option<String>,
    pub cvss_score: Option<f64>,
    pub owasp_category: Option<String>,
    pub attack_vector: Option<AttackVector>,
    pub exploitability: Option<ExploitabilityLevel>,
    pub references: Vec<String>,
    pub tags: Vec<String>,
}
```

## Error Handling

### AnalysisError

Primary error type for security analysis operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("IO error: {message}")]
    IoError { message: String },
    
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("Agent error: {agent_id} - {message}")]
    AgentError { agent_id: String, message: String },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("Timeout error: operation timed out after {seconds} seconds")]
    TimeoutError { seconds: u64 },
    
    #[error("Unsupported language: {language:?}")]
    UnsupportedLanguage { language: SourceLanguage },
    
    #[error("Knowledge graph error: {message}")]
    KnowledgeGraphError { message: String },
}
```

### Error Construction Helpers

```rust
impl AnalysisError {
    pub fn file_system_error(path: String, source: std::io::Error) -> Self {
        Self::IoError {
            message: format!("File system error for {}: {}", path, source),
        }
    }
    
    pub fn agent_timeout(agent_id: String, timeout_seconds: u64) -> Self {
        Self::TimeoutError { seconds: timeout_seconds }
    }
    
    pub fn unsupported_context(message: &str) -> Self {
        Self::ConfigError {
            message: message.to_string(),
        }
    }
}
```

## Examples

### Basic Security Analysis

```rust
use uveddi::analysis::detectors::security::{MainSecurityDetector, SecurityConfig};
use uveddi::ast::{ParsedFile, SourceLanguage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create detector
    let detector = MainSecurityDetector::new()?;
    
    // Analyze file
    let parsed_file = ParsedFile {
        file_path: Arc::new(PathBuf::from("vulnerable.py")),
        language: SourceLanguage::Python,
        // ... other fields
    };
    
    let issues = detector.analyze_file(&parsed_file).await?;
    
    // Process results
    for issue in issues {
        println!("🚨 {} ({})", issue.title, issue.severity);
        println!("   📍 {}:{}", issue.location.file_path.display(), issue.location.start_line);
        println!("   🔧 {}", issue.remediation);
        println!("   📊 Confidence: {:.2}", issue.confidence_score);
        println!();
    }
    
    Ok(())
}
```

### Multi-Agent Analysis

```rust
use uveddi::analysis::detectors::security::{
    SecurityOrchestrator, MultiAgentConfig, TaintAnalysisAgent, ConfigAnalysisAgent
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure multi-agent analysis
    let config = MultiAgentConfig {
        parallel_execution: true,
        max_concurrent_agents: 4,
        enable_cross_validation: true,
        confidence_threshold: 0.8,
        ..Default::default()
    };
    
    // Create orchestrator
    let knowledge_graph = Arc::new(SecurityKnowledgeGraph::new()?);
    let vulnerability_db = Arc::new(VulnerabilityDatabase::new()?);
    
    let orchestrator = SecurityOrchestrator::new(
        config,
        knowledge_graph,
        vulnerability_db,
    )?;
    
    // Analyze codebase
    let result = orchestrator.analyze_codebase(&dependency_graph).await?;
    
    // Generate report
    println!("Security Analysis Report");
    println!("=======================");
    println!("Total Issues: {}", result.total_issues);
    println!("Critical: {}", result.critical_count);
    println!("High: {}", result.high_count);
    println!("Medium: {}", result.medium_count);
    println!("Low: {}", result.low_count);
    
    // Export SARIF
    let sarif_output = result.to_sarif()?;
    std::fs::write("security-analysis.sarif", sarif_output)?;
    
    Ok(())
}
```

### Custom Taint Analysis

```rust
use uveddi::analysis::detectors::security::taint_analysis::{
    TaintAnalysisEngine, TaintSource, TaintSink, SanitizationPoint
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = TaintAnalysisEngine::new();
    
    // Add custom source
    engine.add_source(TaintSource {
        name: "custom_input".to_string(),
        source_type: SourceType::UserInput,
        language: SourceLanguage::Python,
        patterns: vec!["request.custom_field".to_string()],
    });
    
    // Add custom sink
    engine.add_sink(TaintSink {
        name: "custom_output".to_string(),
        sink_type: SinkType::Custom("LogInjection".to_string()),
        language: SourceLanguage::Python,
        patterns: vec!["logger.info".to_string()],
    });
    
    // Add custom sanitizer
    engine.add_sanitizer(SanitizationPoint {
        name: "custom_sanitizer".to_string(),
        sanitizer_type: SanitizerType::InputValidation,
        effectiveness: 0.9,
        applicable_vulnerabilities: vec![SecurityIssueType::Injection],
        patterns: vec!["sanitize_input".to_string()],
    });
    
    // Perform analysis
    let result = engine.analyze(&parsed_file).await?;
    
    for flow in result.vulnerable_flows {
        println!("🔍 Taint Flow Detected:");
        println!("   Source: {} (line {})", flow.source.name, flow.source.line);
        println!("   Sink: {} (line {})", flow.sink.name, flow.sink.line);
        println!("   Confidence: {:.2}", flow.confidence);
        
        if !flow.sanitizers.is_empty() {
            println!("   Sanitizers: {:?}", flow.sanitizers);
        }
    }
    
    Ok(())
}
```

### Configuration Security Analysis

```rust
use uveddi::analysis::detectors::security::strategies::{
    ConfigFileAnalyzer, ConfigSecurityPattern
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = ConfigFileAnalyzer::new();
    
    // Add custom configuration pattern
    analyzer.add_config_pattern(
        "custom_config.yml".to_string(),
        ConfigSecurityPattern {
            key_pattern: "api_key".to_string(),
            dangerous_values: vec!["default".to_string(), "changeme".to_string()],
            vulnerability_type: SecurityIssueType::HardcodedSecrets,
            severity: SecuritySeverity::Critical,
            description: "Default API key detected".to_string(),
            remediation: "Use environment variables for API keys".to_string(),
        },
    );
    
    // Analyze configuration files
    let config_files = vec![
        "settings.py",
        "Dockerfile", 
        "nginx.conf",
        "custom_config.yml",
    ];
    
    for config_file in config_files {
        let issues = analyzer.analyze_config_file(Path::new(config_file)).await?;
        
        if !issues.is_empty() {
            println!("🔧 Configuration Issues in {}:", config_file);
            for issue in issues {
                println!("   {} ({})", issue.title, issue.severity);
                println!("   💡 {}", issue.remediation);
            }
            println!();
        }
    }
    
    Ok(())
}
```

---

This API reference provides comprehensive documentation for developers working with the Uveddi Security Detector system. For implementation examples and advanced usage patterns, refer to the [Security Detector Development Guide](../04-development/security-detector-development.md).