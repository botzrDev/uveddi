# Security Detector Development Guide

## Overview

This guide provides comprehensive information for developers working on or extending the Uveddi Security Detector system. It covers the internal architecture, development workflows, testing strategies, and contribution guidelines.

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Development Environment](#development-environment)
3. [Code Organization](#code-organization)
4. [Creating Custom Detectors](#creating-custom-detectors)
5. [Testing Framework](#testing-framework)
6. [Performance Optimization](#performance-optimization)
7. [Debugging and Troubleshooting](#debugging-and-troubleshooting)
8. [Contributing Guidelines](#contributing-guidelines)

## System Architecture

### Module Structure

```
src/analysis/detectors/security/
├── mod.rs                    # Main module and orchestrator
├── agents.rs                 # Multi-agent system implementation
├── config.rs                 # Configuration management
├── core.rs                   # Core types and interfaces
├── detector.rs               # Main detector implementation
├── knowledge_graph.rs        # Knowledge graph and RAG system
├── owasp.rs                  # OWASP Top 10 detection
├── strategies.rs             # Detection strategy implementations
├── taint_analysis.rs         # Taint analysis engine
├── types.rs                  # Security-specific types
└── validation.rs             # Result validation and confidence scoring
```

### Core Traits and Interfaces

#### SecurityAgent Trait
```rust
#[async_trait]
pub trait SecurityAgent: Send + Sync {
    /// Execute a security analysis task
    async fn execute_task(
        &self,
        task_id: String,
        context: SecurityContext,
    ) -> Result<AgentResult, AnalysisError>;
    
    /// Get unique agent identifier
    fn get_agent_id(&self) -> &str;
    
    /// Get agent capabilities
    fn get_capabilities(&self) -> Vec<TaskType>;
    
    /// Clone the agent for parallel execution
    fn clone_box(&self) -> Box<dyn SecurityAgent>;
}
```

#### DetectionStrategy Trait
```rust
#[async_trait]
pub trait DetectionStrategy: Send + Sync {
    /// Analyze a parsed file for security issues
    async fn analyze(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError>;
    
    /// Get strategy name
    fn get_name(&self) -> &'static str;
    
    /// Get supported languages
    fn get_supported_languages(&self) -> Vec<SourceLanguage>;
    
    /// Get detection confidence level
    fn get_confidence_level(&self) -> f64;
}
```

### Agent Architecture

#### Agent Lifecycle
```rust
pub struct AgentLifecycle {
    pub initialization: Duration,
    pub task_execution: Duration,
    pub result_processing: Duration,
    pub cleanup: Duration,
}

impl SecurityAgent for TaintAnalysisAgent {
    async fn execute_task(
        &self,
        task_id: String,
        context: SecurityContext,
    ) -> Result<AgentResult, AnalysisError> {
        let start_time = Instant::now();
        
        // Initialize analysis context
        let analysis_context = self.prepare_analysis_context(&context)?;
        
        // Perform taint analysis
        let taint_flows = self.engine.analyze_data_flows(&analysis_context).await?;
        
        // Validate and score results
        let validated_flows = self.validate_flows(taint_flows)?;
        
        // Convert to standard result format
        let issues = self.convert_to_security_issues(validated_flows)?;
        
        Ok(AgentResult {
            agent_id: self.get_agent_id().to_string(),
            task_id,
            issues,
            execution_time: start_time.elapsed(),
            confidence_scores: self.calculate_confidence_scores(&issues),
            metadata: self.get_execution_metadata(),
        })
    }
}
```

## Development Environment

### Prerequisites

```bash
# Rust toolchain (1.70.0 or later)
rustup install stable
rustup default stable

# Development dependencies
cargo install cargo-tarpaulin    # Code coverage
cargo install cargo-audit        # Security auditing
cargo install cargo-deny         # Dependency auditing
cargo install cargo-nextest      # Fast test runner

# Additional tools
npm install -g @microsoft/sarif-cli  # SARIF validation
```

### Building the Security Detector

```bash
# Development build (fast compilation)
cargo build --features=dev-core

# Full security features
cargo build --features=production

# Security detector only
cargo build --features=security

# With AI enhancements
cargo build --features=security,local-ai
```

### Environment Configuration

```bash
# Set development environment variables
export RUST_LOG=debug
export UVEDDI_DEV_MODE=true
export UVEDDI_SECURITY_TEST_DATA=./tests/data/security
export UVEDDI_CACHE_DIR=./target/cache

# Optional: AI integration
export OLLAMA_API_URL=http://localhost:11434
export OLLAMA_MODEL=deepseek-coder:6.7b
```

## Code Organization

### Adding New Detection Strategies

#### 1. Define the Strategy Structure
```rust
// src/analysis/detectors/security/strategies.rs

pub struct CustomPatternMatcher {
    patterns: HashMap<SourceLanguage, Vec<CustomPattern>>,
    confidence_calculator: ConfidenceCalculator,
}

#[derive(Debug, Clone)]
pub struct CustomPattern {
    pub pattern: String,
    pub vulnerability_type: SecurityIssueType,
    pub severity: SecuritySeverity,
    pub confidence_base: f64,
    pub context_requirements: Vec<ContextRequirement>,
}

impl CustomPatternMatcher {
    pub fn new() -> Self {
        let mut matcher = Self {
            patterns: HashMap::new(),
            confidence_calculator: ConfidenceCalculator::new(),
        };
        matcher.initialize_patterns();
        matcher
    }
    
    fn initialize_patterns(&mut self) {
        // Add language-specific patterns
        self.add_rust_patterns();
        self.add_python_patterns();
        self.add_javascript_patterns();
    }
}
```

#### 2. Implement DetectionStrategy Trait
```rust
#[async_trait]
impl DetectionStrategy for CustomPatternMatcher {
    async fn analyze(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let content = std::fs::read_to_string(&**file.file_path)?;
        let mut issues = Vec::new();
        
        if let Some(patterns) = self.patterns.get(&file.language) {
            for (line_num, line) in content.lines().enumerate() {
                for pattern in patterns {
                    if let Some(issue) = self.check_pattern(pattern, line, line_num, file)? {
                        issues.push(issue);
                    }
                }
            }
        }
        
        Ok(issues)
    }
    
    fn get_name(&self) -> &'static str {
        "CustomPatternMatcher"
    }
    
    fn get_supported_languages(&self) -> Vec<SourceLanguage> {
        self.patterns.keys().cloned().collect()
    }
    
    fn get_confidence_level(&self) -> f64 {
        0.8  // Base confidence level
    }
}
```

#### 3. Register the Strategy
```rust
// src/analysis/detectors/security/detector.rs

impl SecurityDetector {
    pub fn with_custom_strategies(mut self) -> Self {
        self.strategies.push(Box::new(CustomPatternMatcher::new()));
        self
    }
}
```

### Creating Custom Agents

#### 1. Define Agent Structure
```rust
// src/analysis/detectors/security/agents.rs

pub struct CustomSecurityAgent {
    agent_id: String,
    capabilities: Vec<TaskType>,
    analyzer: CustomAnalyzer,
    config: CustomAgentConfig,
}

#[derive(Debug, Clone)]
pub struct CustomAgentConfig {
    pub timeout_seconds: u64,
    pub max_files_per_batch: usize,
    pub enable_deep_analysis: bool,
    pub custom_rules: Vec<CustomRule>,
}

impl CustomSecurityAgent {
    pub fn new(config: CustomAgentConfig) -> Self {
        Self {
            agent_id: "custom_security_agent".to_string(),
            capabilities: vec![
                TaskType::StaticAnalysis,
                TaskType::PatternMatching,
                TaskType::CustomAnalysis,
            ],
            analyzer: CustomAnalyzer::new(&config),
            config,
        }
    }
}
```

#### 2. Implement SecurityAgent Trait
```rust
#[async_trait]
impl SecurityAgent for CustomSecurityAgent {
    async fn execute_task(
        &self,
        task_id: String,
        context: SecurityContext,
    ) -> Result<AgentResult, AnalysisError> {
        info!("CustomSecurityAgent executing task: {}", task_id);
        
        // Validate context
        if !self.can_handle_context(&context) {
            return Err(AnalysisError::unsupported_context(
                "CustomSecurityAgent cannot handle this context"
            ));
        }
        
        // Perform custom analysis
        let start_time = Instant::now();
        let analysis_results = self.analyzer.analyze(&context).await?;
        
        // Convert results to security issues
        let issues = self.convert_results_to_issues(analysis_results)?;
        
        // Calculate confidence scores
        let confidence_scores = self.calculate_confidence_scores(&issues);
        
        Ok(AgentResult {
            agent_id: self.agent_id.clone(),
            task_id,
            issues,
            execution_time: start_time.elapsed(),
            confidence_scores,
            metadata: HashMap::new(),
        })
    }
    
    fn get_agent_id(&self) -> &str {
        &self.agent_id
    }
    
    fn get_capabilities(&self) -> Vec<TaskType> {
        self.capabilities.clone()
    }
    
    fn clone_box(&self) -> Box<dyn SecurityAgent> {
        Box::new(self.clone())
    }
}
```

#### 3. Register with Orchestrator
```rust
// src/analysis/detectors/security/mod.rs

impl SecurityOrchestrator {
    pub fn with_custom_agent(mut self, agent: Box<dyn SecurityAgent>) -> Self {
        self.agents.push(agent);
        self
    }
}

// Usage
let custom_agent = Box::new(CustomSecurityAgent::new(config));
let orchestrator = SecurityOrchestrator::new(multi_agent_config)?
    .with_custom_agent(custom_agent);
```

## Testing Framework

### Test Structure

```
tests/
├── unit/
│   └── security/
│       ├── agents_test.rs           # Agent unit tests
│       ├── config_test.rs           # Configuration tests
│       ├── core_test.rs             # Core functionality tests
│       ├── knowledge_graph_test.rs  # Knowledge graph tests
│       ├── owasp_test.rs           # OWASP detector tests
│       ├── strategies_test.rs       # Strategy tests
│       ├── taint_analysis_test.rs   # Taint analysis tests
│       ├── types_test.rs           # Type tests
│       └── validation_test.rs       # Validation tests
├── integration/
│   └── security/
│       ├── end_to_end_test.rs      # Full workflow tests
│       ├── multi_agent_test.rs     # Multi-agent integration
│       └── performance_test.rs      # Performance benchmarks
└── data/
    └── security/
        ├── vulnerable_samples/      # Test code samples
        ├── clean_samples/          # Non-vulnerable samples
        └── configurations/         # Test configurations
```

### Writing Unit Tests

#### Testing Detection Strategies
```rust
// tests/unit/security/strategies_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_parsed_file;
    
    #[tokio::test]
    async fn test_sql_injection_detection() {
        let detector = DeterministicPatternMatcher::new();
        
        // Create test file with SQL injection vulnerability
        let test_content = r#"
            def get_user(user_id):
                query = f"SELECT * FROM users WHERE id = {user_id}"
                cursor.execute(query)
                return cursor.fetchone()
        "#;
        
        let test_file = create_test_file_with_content(
            "vulnerable.py",
            SourceLanguage::Python,
            test_content,
        );
        
        let issues = detector.analyze(&test_file).await.unwrap();
        
        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, SecurityIssueType::Injection);
        assert_eq!(issues[0].severity, SecuritySeverity::Critical);
        assert!(issues[0].confidence_score > 0.8);
    }
    
    #[test]
    fn test_pattern_loading() {
        let detector = DeterministicPatternMatcher::new();
        
        // Verify patterns are loaded for all supported languages
        assert!(detector.patterns.contains_key(&SourceLanguage::Rust));
        assert!(detector.patterns.contains_key(&SourceLanguage::Python));
        assert!(detector.patterns.contains_key(&SourceLanguage::JavaScript));
        assert!(detector.patterns.contains_key(&SourceLanguage::TypeScript));
        
        // Verify pattern content
        let python_patterns = &detector.patterns[&SourceLanguage::Python];
        assert!(python_patterns.iter().any(|p| p.pattern.contains("shell=True")));
        assert!(python_patterns.iter().any(|p| p.pattern.contains("pickle.load")));
    }
}
```

#### Testing Agents
```rust
// tests/unit/security/agents_test.rs

#[tokio::test]
async fn test_taint_analysis_agent() {
    let config = TaintAnalysisConfig::default();
    let agent = TaintAnalysisAgent::new(config).unwrap();
    
    // Create test context
    let context = SecurityContext {
        target_files: vec![create_vulnerable_file()],
        analysis_scope: AnalysisScope::SingleFile,
        language_context: LanguageContext::Python,
        project_metadata: ProjectMetadata::default(),
    };
    
    // Execute analysis
    let result = agent.execute_task("test_task".to_string(), context).await.unwrap();
    
    // Verify results
    assert_eq!(result.agent_id, "taint_analysis_agent");
    assert!(!result.issues.is_empty());
    assert!(result.execution_time.as_millis() > 0);
    
    // Verify taint flow detection
    let taint_issues: Vec<_> = result.issues.iter()
        .filter(|issue| matches!(issue.vulnerability_type, VulnerabilityType::DataFlow))
        .collect();
    assert!(!taint_issues.is_empty());
}

#[test]
fn test_agent_capabilities() {
    let agent = TaintAnalysisAgent::new(TaintAnalysisConfig::default()).unwrap();
    
    let capabilities = agent.get_capabilities();
    assert!(capabilities.contains(&TaskType::TaintAnalysis));
    assert!(capabilities.contains(&TaskType::DataFlowAnalysis));
}
```

### Integration Testing

#### End-to-End Workflow Tests
```rust
// tests/integration/security/end_to_end_test.rs

#[tokio::test]
async fn test_complete_security_analysis_workflow() {
    // Setup test environment
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_project(&temp_dir);
    
    // Configure security detector
    let config = SecurityConfig {
        enable_taint_analysis: true,
        enable_config_analysis: true,
        enable_sca: true,
        confidence_threshold: 0.7,
        ..Default::default()
    };
    
    let detector = MainSecurityDetector::with_config(config).unwrap();
    
    // Analyze test project
    let results = analyze_project(&detector, temp_dir.path()).await.unwrap();
    
    // Verify comprehensive analysis
    assert!(results.len() > 0);
    
    // Verify OWASP coverage
    let owasp_categories: HashSet<_> = results.iter()
        .map(|issue| &issue.issue_type)
        .collect();
    
    assert!(owasp_categories.contains(&SecurityIssueType::Injection));
    assert!(owasp_categories.contains(&SecurityIssueType::BrokenAccessControl));
    
    // Verify confidence scoring
    for issue in &results {
        assert!(issue.confidence_score >= 0.0 && issue.confidence_score <= 1.0);
    }
    
    // Verify remediation information
    for issue in &results {
        assert!(!issue.remediation.is_empty());
    }
}

fn create_test_project(temp_dir: &tempfile::TempDir) {
    // Create vulnerable Python file
    let python_file = temp_dir.path().join("vulnerable.py");
    std::fs::write(&python_file, r#"
import subprocess
import pickle

def unsafe_command(user_input):
    # Command injection vulnerability
    subprocess.run(f"echo {user_input}", shell=True)

def unsafe_deserialize(data):
    # Deserialization vulnerability
    return pickle.loads(data)

# Hardcoded secret
API_KEY = "secret-key-12345"
    "#).unwrap();
    
    // Create vulnerable JavaScript file
    let js_file = temp_dir.path().join("vulnerable.js");
    std::fs::write(&js_file, r#"
function renderContent(userInput) {
    // XSS vulnerability
    document.write(userInput);
}

function processData(data) {
    // Potential prototype pollution
    merge(target, data);
}
    "#).unwrap();
    
    // Create insecure configuration
    let config_file = temp_dir.path().join("settings.py");
    std::fs::write(&config_file, r#"
DEBUG = True
SECRET_KEY = "django-insecure-default-key"
ALLOWED_HOSTS = ['*']
    "#).unwrap();
}
```

### Performance Testing

```rust
// tests/integration/security/performance_test.rs

#[tokio::test]
async fn test_large_codebase_performance() {
    let temp_dir = create_large_test_codebase(10000).await; // 10k lines of code
    
    let config = SecurityConfig::default();
    let detector = MainSecurityDetector::with_config(config).unwrap();
    
    let start_time = Instant::now();
    let results = analyze_project(&detector, &temp_dir).await.unwrap();
    let analysis_time = start_time.elapsed();
    
    // Performance assertions
    assert!(analysis_time.as_secs() < 60, "Analysis should complete within 1 minute");
    assert!(!results.is_empty(), "Should find security issues in large codebase");
    
    // Memory usage should be reasonable
    let memory_usage = get_current_memory_usage();
    assert!(memory_usage < 1024 * 1024 * 1024, "Memory usage should be under 1GB");
}

#[tokio::test]
async fn test_parallel_agent_performance() {
    let test_files = create_multiple_test_files(100);
    
    // Test sequential execution
    let start_time = Instant::now();
    let sequential_results = analyze_files_sequentially(&test_files).await.unwrap();
    let sequential_time = start_time.elapsed();
    
    // Test parallel execution
    let start_time = Instant::now();
    let parallel_results = analyze_files_in_parallel(&test_files).await.unwrap();
    let parallel_time = start_time.elapsed();
    
    // Parallel should be faster and produce same results
    assert!(parallel_time < sequential_time);
    assert_eq!(sequential_results.len(), parallel_results.len());
}
```

### Test Utilities

```rust
// tests/unit/security/test_helpers.rs

use std::sync::Arc;
use std::path::PathBuf;
use uveddi::ast::{ParsedFile, SourceLanguage};
use uveddi::analysis::cache::wrappers::ArchivableSystemTime;

pub fn create_test_parsed_file(path: &str, language: SourceLanguage) -> ParsedFile {
    ParsedFile {
        file_path: Arc::new(PathBuf::from(path)),
        language,
        tree: None,
        source: Arc::new(String::new()),
        custom_ast: Arc::new(None),
        modified_at: ArchivableSystemTime::now(),
    }
}

pub fn create_test_file_with_content(
    path: &str,
    language: SourceLanguage,
    content: &str,
) -> ParsedFile {
    ParsedFile {
        file_path: Arc::new(PathBuf::from(path)),
        language,
        tree: None,
        source: Arc::new(content.to_string()),
        custom_ast: Arc::new(None),
        modified_at: ArchivableSystemTime::now(),
    }
}

pub fn create_vulnerable_python_file() -> ParsedFile {
    let content = r#"
import subprocess

def execute_command(user_input):
    subprocess.run(f"ls {user_input}", shell=True)
    "#;
    
    create_test_file_with_content("vulnerable.py", SourceLanguage::Python, content)
}

pub fn create_security_context_for_file(file: ParsedFile) -> SecurityContext {
    SecurityContext {
        target_files: vec![file],
        analysis_scope: AnalysisScope::SingleFile,
        language_context: LanguageContext::Multi,
        project_metadata: ProjectMetadata::default(),
    }
}
```

## Performance Optimization

### Profiling and Benchmarking

```bash
# Profile security detector performance
cargo build --release --features=production
perf record --call-graph=dwarf target/release/uveddi analyze ./large_codebase --security
perf report

# Memory profiling with valgrind
cargo build --features=production
valgrind --tool=massif target/debug/uveddi analyze ./test_project --security

# Benchmark with different configurations
cargo bench --features=security -- security_detector
```

### Optimization Strategies

#### 1. Parallel Processing
```rust
// src/analysis/detectors/security/mod.rs

impl SecurityOrchestrator {
    pub async fn analyze_files_parallel(
        &self,
        files: &[ParsedFile],
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        use tokio::task::JoinSet;
        
        let mut join_set = JoinSet::new();
        let chunk_size = self.config.max_concurrent_files;
        
        for chunk in files.chunks(chunk_size) {
            for file in chunk {
                let agent = self.select_optimal_agent_for_file(file);
                let context = SecurityContext::from_file(file.clone());
                
                join_set.spawn(async move {
                    agent.execute_task(
                        format!("file_{}", file.file_path.display()),
                        context,
                    ).await
                });
            }
        }
        
        let mut all_issues = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(agent_result)) => all_issues.extend(agent_result.issues),
                Ok(Err(e)) => warn!("Agent task failed: {}", e),
                Err(e) => error!("Join error: {}", e),
            }
        }
        
        Ok(all_issues)
    }
}
```

#### 2. Caching Implementation
```rust
// src/analysis/detectors/security/cache.rs

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SecurityAnalysisCache {
    results: Arc<RwLock<HashMap<FileHash, CachedResult>>>,
    config: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct CachedResult {
    pub issues: Vec<SecurityIssue>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub file_hash: u64,
    pub config_hash: u64,
}

impl SecurityAnalysisCache {
    pub async fn get_or_analyze<F, Fut>(
        &self,
        file: &ParsedFile,
        analyzer: F,
    ) -> Result<Vec<SecurityIssue>, AnalysisError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<SecurityIssue>, AnalysisError>>,
    {
        let file_hash = self.calculate_file_hash(file);
        
        // Check cache first
        {
            let cache = self.results.read().await;
            if let Some(cached) = cache.get(&file_hash) {
                if self.is_cache_valid(cached) {
                    return Ok(cached.issues.clone());
                }
            }
        }
        
        // Perform analysis
        let issues = analyzer().await?;
        
        // Cache results
        {
            let mut cache = self.results.write().await;
            cache.insert(file_hash, CachedResult {
                issues: issues.clone(),
                timestamp: chrono::Utc::now(),
                file_hash: file_hash.0,
                config_hash: self.config.hash,
            });
        }
        
        Ok(issues)
    }
}
```

#### 3. Memory Optimization
```rust
// src/analysis/detectors/security/memory.rs

pub struct MemoryOptimizedAnalyzer {
    memory_threshold: usize,
    current_usage: Arc<AtomicUsize>,
    streaming_enabled: bool,
}

impl MemoryOptimizedAnalyzer {
    pub async fn analyze_with_memory_limit(
        &self,
        files: &[ParsedFile],
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();
        
        for chunk in self.create_memory_aware_chunks(files) {
            // Monitor memory usage
            if self.current_usage.load(Ordering::Relaxed) > self.memory_threshold {
                self.trigger_garbage_collection().await;
            }
            
            // Process chunk
            let chunk_issues = self.analyze_chunk(chunk).await?;
            issues.extend(chunk_issues);
            
            // Clear intermediate data if streaming
            if self.streaming_enabled {
                self.clear_intermediate_data();
            }
        }
        
        Ok(issues)
    }
    
    fn create_memory_aware_chunks(&self, files: &[ParsedFile]) -> Vec<&[ParsedFile]> {
        let mut chunks = Vec::new();
        let mut current_size = 0;
        let mut chunk_start = 0;
        
        for (i, file) in files.iter().enumerate() {
            let file_size = self.estimate_file_memory_usage(file);
            
            if current_size + file_size > self.memory_threshold && i > chunk_start {
                chunks.push(&files[chunk_start..i]);
                chunk_start = i;
                current_size = 0;
            }
            
            current_size += file_size;
        }
        
        if chunk_start < files.len() {
            chunks.push(&files[chunk_start..]);
        }
        
        chunks
    }
}
```

## Debugging and Troubleshooting

### Debugging Tools

#### 1. Debug Logging Configuration
```rust
// src/analysis/detectors/security/debug.rs

pub fn enable_security_debug_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "uveddi::analysis::detectors::security=debug"
        ))
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
        )
        .init();
}

#[macro_export]
macro_rules! security_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "security_detector", $($arg)*);
    };
}
```

#### 2. Analysis Metrics Collection
```rust
// src/analysis/detectors/security/metrics.rs

#[derive(Debug, Clone, Serialize)]
pub struct SecurityAnalysisMetrics {
    pub total_files_analyzed: usize,
    pub total_issues_found: usize,
    pub analysis_time: Duration,
    pub memory_peak_usage: usize,
    pub agent_performance: HashMap<String, AgentMetrics>,
    pub false_positive_rate: f64,
    pub confidence_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentMetrics {
    pub execution_count: usize,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub issues_found: usize,
    pub error_count: usize,
}

impl SecurityAnalysisMetrics {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = self.to_json()?;
        std::fs::write(path, json)
    }
}
```

#### 3. Interactive Debugging
```rust
// src/analysis/detectors/security/interactive.rs

pub struct InteractiveDebugger {
    pub breakpoints: Vec<Breakpoint>,
    pub step_mode: bool,
    pub variable_inspector: VariableInspector,
}

impl InteractiveDebugger {
    pub async fn debug_analysis_step(
        &mut self,
        step: AnalysisStep,
        context: &SecurityContext,
    ) -> Result<(), AnalysisError> {
        if self.should_break_at_step(&step) {
            println!("Breakpoint hit at step: {:?}", step);
            println!("Context: {:#?}", context);
            
            self.enter_interactive_mode().await?;
        }
        
        Ok(())
    }
    
    async fn enter_interactive_mode(&mut self) -> Result<(), AnalysisError> {
        use std::io::{self, Write};
        
        loop {
            print!("(security-debug) ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "continue" | "c" => break,
                "step" | "s" => {
                    self.step_mode = true;
                    break;
                }
                "inspect" | "i" => self.show_variable_inspector(),
                "help" | "h" => self.show_help(),
                "quit" | "q" => return Err(AnalysisError::user_cancelled()),
                cmd => println!("Unknown command: {}", cmd),
            }
        }
        
        Ok(())
    }
}
```

### Common Issues and Solutions

#### Issue: High False Positive Rate
```rust
// Increase confidence threshold
let config = SecurityConfig {
    confidence_threshold: 0.9,  // Increase from default 0.7
    enable_cross_validation: true,
    enable_bayesian_optimization: true,
    ..Default::default()
};

// Enable additional validation
let validation_config = ValidationConfig {
    enable_false_positive_mitigation: true,
    historical_data_weight: 0.3,
    context_analysis_weight: 0.4,
    pattern_specificity_weight: 0.3,
};
```

#### Issue: Slow Analysis Performance
```rust
// Enable parallel processing
let performance_config = PerformanceConfig {
    enable_parallel_agents: true,
    max_concurrent_agents: num_cpus::get(),
    enable_file_parallelism: true,
    max_concurrent_files: 10,
    enable_streaming_analysis: true,
    memory_threshold_mb: 1024,
};

// Use incremental analysis
let incremental_config = IncrementalConfig {
    enable_incremental: true,
    baseline_file: ".uveddi/security-baseline.json",
    analyze_changed_dependencies: true,
};
```

#### Issue: Memory Usage Too High
```rust
// Configure memory limits
let memory_config = MemoryConfig {
    max_memory_usage_mb: 512,
    enable_streaming: true,
    enable_garbage_collection: true,
    gc_threshold_mb: 256,
    chunk_size_mb: 64,
};

// Use memory-optimized analysis
let detector = MainSecurityDetector::with_config(SecurityConfig {
    memory_optimization: memory_config,
    ..Default::default()
})?;
```

## Contributing Guidelines

### Code Standards

#### 1. Code Style
```rust
// Use consistent naming conventions
pub struct SecurityDetector { /* PascalCase for types */ }
pub fn analyze_file() { /* snake_case for functions */ }
pub const MAX_ANALYSIS_DEPTH: usize = 10; /* SCREAMING_SNAKE_CASE for constants */

// Document public APIs
/// Analyzes a parsed file for security vulnerabilities
/// 
/// # Arguments
/// * `file` - The parsed file to analyze
/// 
/// # Returns
/// Vector of security issues found in the file
/// 
/// # Errors
/// Returns `AnalysisError` if file analysis fails
pub async fn analyze_file(&self, file: &ParsedFile) -> Result<Vec<SecurityIssue>, AnalysisError> {
    // Implementation
}

// Use appropriate error handling
match self.analyze_internal(file).await {
    Ok(issues) => Ok(issues),
    Err(e) => {
        error!("Analysis failed for {}: {}", file.file_path.display(), e);
        Err(e)
    }
}
```

#### 2. Testing Requirements
```rust
// Every public function must have tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_new_function_basic_case() {
        // Test basic functionality
    }
    
    #[tokio::test]
    async fn test_new_function_edge_cases() {
        // Test edge cases
    }
    
    #[tokio::test]
    async fn test_new_function_error_conditions() {
        // Test error handling
    }
}

// Integration tests for new features
#[tokio::test]
async fn test_new_feature_integration() {
    // End-to-end test
}
```

#### 3. Performance Considerations
```rust
// Benchmark new algorithms
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_new_algorithm(c: &mut Criterion) {
        c.bench_function("new_algorithm", |b| {
            b.iter(|| {
                // Benchmark code
                black_box(new_algorithm(black_box(test_input)))
            })
        });
    }
    
    criterion_group!(benches, benchmark_new_algorithm);
    criterion_main!(benches);
}
```

### Submission Process

#### 1. Development Workflow
```bash
# Create feature branch
git checkout -b feature/new-security-detector

# Make changes with good commit messages
git commit -m "feat(security): add new vulnerability pattern matcher

- Implement pattern matcher for dependency injection vulnerabilities
- Add support for Spring Framework specific patterns
- Include confidence scoring based on context analysis
- Add comprehensive test coverage

Closes #123"

# Run comprehensive tests
cargo test --features=security
cargo test --test integration_security

# Run security-specific linting
cargo clippy --features=security -- -D warnings
cargo audit
cargo deny check

# Check performance impact
cargo bench --features=security
```

#### 2. Pull Request Requirements
- **Tests**: Minimum 90% code coverage for new code
- **Documentation**: All public APIs documented with examples
- **Performance**: No significant performance regression
- **Security**: No new security vulnerabilities introduced
- **Backwards Compatibility**: Maintain API compatibility unless major version

#### 3. Code Review Checklist
- [ ] Code follows established patterns and conventions
- [ ] Comprehensive test coverage including edge cases
- [ ] Documentation is clear and includes examples
- [ ] Performance impact is acceptable
- [ ] Security implications have been considered
- [ ] Error handling is robust and informative
- [ ] Logging is appropriate and helpful for debugging

### Additional Resources

- **Architecture Documentation**: [docs/01-architecture/](../01-architecture/)
- **Security Model**: [docs/01-architecture/security-model.md](../01-architecture/security-model.md)
- **Contributing Guide**: [docs/06-community/CONTRIBUTING.md](../06-community/CONTRIBUTING.md)
- **Testing Strategy**: [docs/07-reference/testing_strategy.md](../07-reference/testing_strategy.md)

---

This development guide provides the foundation for contributing to and extending the Uveddi Security Detector. For specific implementation questions, refer to the existing code examples and test cases in the repository.