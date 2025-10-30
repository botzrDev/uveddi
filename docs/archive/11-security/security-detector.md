# Security Vulnerability Detection System

## Overview

The Uveddi Security Detector is a comprehensive, multi-layered security analysis system that combines deterministic pattern matching, AI-powered analysis, and architectural correlation to identify security vulnerabilities across codebases. Built with a privacy-first, developer-centric approach, it provides actionable insights while minimizing false positives.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Components](#core-components)
3. [Detection Capabilities](#detection-capabilities)
4. [Multi-Agent System](#multi-agent-system)
5. [Configuration](#configuration)
6. [Usage Examples](#usage-examples)
7. [Integration](#integration)
8. [Performance](#performance)
9. [Troubleshooting](#troubleshooting)

## Architecture Overview

The Security Detector follows a multi-layered architecture that combines deterministic analysis with AI-powered enhancement:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Security Detector                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐    ┌──────────────────┐    ┌──────────────┐  │
│  │ Taint Analysis│    │  Config Analysis │    │ SCA Analysis │  │
│  │    Agent      │    │      Agent       │    │    Agent     │  │
│  └───────────────┘    └──────────────────┘    └──────────────┘  │
│           │                     │                       │       │
│           └─────────────────────┼───────────────────────┘       │
│                                 │                               │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │            Knowledge Graph (Code-Centric RAG)              │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                 │                               │
│  ┌───────────────┐    ┌──────────────────┐    ┌──────────────┐  │
│  │   Validation  │    │   AI Enhancement │    │ Architectural│  │
│  │    Engine     │    │      Layer       │    │ Correlation  │  │
│  └───────────────┘    └──────────────────┘    └──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Principles

- **Privacy-First**: All analysis performed locally, no code sent to external services
- **Multi-Layered Detection**: Combines multiple analysis techniques for comprehensive coverage
- **Contextual Intelligence**: Uses knowledge graphs for code-aware analysis
- **Developer-Centric**: Provides actionable insights with clear remediation guidance
- **Minimal False Positives**: Advanced validation and confidence scoring

## Core Components

### 1. Security Orchestrator

The central coordinator that manages the multi-agent analysis workflow:

```rust
pub struct SecurityOrchestrator {
    agents: Vec<Box<dyn SecurityAgent>>,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
    validation_engine: ValidationEngine,
    config: MultiAgentConfig,
}
```

**Responsibilities:**
- Coordinates agent execution
- Manages task distribution
- Aggregates and validates results
- Provides confidence scoring

### 2. Detection Strategies

#### Deterministic Pattern Matcher
- **Purpose**: Fast, reliable detection of known vulnerability patterns
- **Coverage**: Language-specific vulnerability patterns (Rust, Python, JavaScript, TypeScript)
- **Examples**: SQL injection patterns, XSS vectors, unsafe operations

#### Configuration File Analyzer
- **Purpose**: Detect security misconfigurations
- **Scope**: Framework configs (Django, Docker, Nginx), deployment files
- **Examples**: Debug mode in production, default secrets, insecure SSL settings

#### Software Composition Analyzer (SCA)
- **Purpose**: Identify vulnerable dependencies
- **Coverage**: Rust (Cargo.toml), Python (requirements.txt), Node.js (package.json)
- **Integration**: RustSec, Safety DB, npm audit databases

#### Vulnerability Correlation Engine
- **Purpose**: Map security issues to architectural anti-patterns
- **Intelligence**: Links vulnerability types to design flaws
- **Examples**: God Objects → Access Control issues, Leaky Abstractions → Injection vulnerabilities

### 3. Multi-Agent System

#### Taint Analysis Agent
Tracks data flow from sources to sinks to identify injection vulnerabilities:

```rust
pub struct TaintAnalysisAgent {
    engine: TaintAnalysisEngine,
    sources: Vec<TaintSource>,
    sinks: Vec<TaintSink>,
    sanitizers: Vec<SanitizationPoint>,
}
```

**Capabilities:**
- Data flow tracking across function boundaries
- Cross-language taint propagation
- Custom source/sink definitions
- Sanitization point recognition

#### Config Analysis Agent
Specialized analysis for configuration files and infrastructure code:

```rust
pub struct ConfigAnalysisAgent {
    analyzers: HashMap<String, Box<dyn ConfigAnalyzer>>,
    rule_engine: ConfigRuleEngine,
}
```

**Supported Formats:**
- Application configs (settings.py, application.yml)
- Container configs (Dockerfile, docker-compose.yml)
- Server configs (nginx.conf, apache.conf)
- Cloud configs (terraform, kubernetes manifests)

#### Dependency Agent
Performs Software Composition Analysis (SCA):

```rust
pub struct DependencyAgent {
    vulnerability_db: Arc<VulnerabilityDatabase>,
    parsers: HashMap<SourceLanguage, Box<dyn DependencyParser>>,
}
```

**Features:**
- Multi-language dependency parsing
- Vulnerability database integration
- License compliance checking
- Dependency graph analysis

#### Validation Agent
Cross-validates findings to reduce false positives:

```rust
pub struct ValidationAgent {
    false_positive_mitigator: FalsePositiveMitigator,
    confidence_calculator: ConfidenceCalculator,
    bayesian_optimizer: BayesianOptimizer,
}
```

**Validation Techniques:**
- Cross-reference multiple detection methods
- Historical false positive patterns
- Code context analysis
- Bayesian confidence scoring

## Detection Capabilities

### OWASP Top 10 2021 Coverage

| OWASP Category | Detection Method | Confidence | Coverage |
|----------------|------------------|------------|----------|
| **A01: Broken Access Control** | Pattern + Taint Analysis | High | 90% |
| **A02: Cryptographic Failures** | Pattern + Config Analysis | High | 85% |
| **A03: Injection** | Taint Analysis + Patterns | Very High | 95% |
| **A04: Insecure Design** | Architectural Correlation | Medium | 70% |
| **A05: Security Misconfiguration** | Config Analysis | High | 90% |
| **A06: Vulnerable Components** | SCA + Dependency Analysis | Very High | 98% |
| **A07: Authentication Failures** | Pattern + Config Analysis | High | 85% |
| **A08: Software Data Integrity** | Pattern + Taint Analysis | Medium | 75% |
| **A09: Security Logging Failures** | Pattern + Config Analysis | High | 80% |
| **A10: Server-Side Request Forgery** | Taint Analysis | High | 90% |

### Language-Specific Patterns

#### Rust Security Patterns
```rust
// Detected: Use of unsafe memory operations
std::ptr::read_volatile  // High severity
.unwrap()               // Medium severity - potential panic

// Configuration patterns
[dependencies]
openssl = "0.10.0"      // Detected as outdated vulnerable version
```

#### Python Security Patterns
```python
# Detected: Unsafe deserialization
pickle.load(file)       # Critical severity

# Command injection
subprocess.call(cmd, shell=True)  # Critical severity

# Django configuration issues
DEBUG = True           # High severity in production
SECRET_KEY = "django-insecure-..."  # Critical severity
```

#### JavaScript/TypeScript Patterns
```javascript
// XSS vulnerabilities
document.write(userInput)  // High severity

// Prototype pollution
obj[prop] = value  // Medium severity with validation

// Insecure dependencies
"lodash": "4.17.19"   // Known vulnerable version
```

### Taint Analysis Engine

The taint analysis engine tracks data flow to identify injection vulnerabilities:

#### Data Flow Tracking
```rust
pub struct DataFlowGraph {
    nodes: HashMap<NodeId, DataFlowNode>,
    edges: Vec<DataFlowEdge>,
    entry_points: Vec<NodeId>,
}

pub struct TaintSource {
    name: String,
    source_type: SourceType,  // UserInput, NetworkRequest, FileRead
    language: SourceLanguage,
    patterns: Vec<String>,
}

pub struct TaintSink {
    name: String,
    sink_type: SinkType,      // SqlQuery, CommandExecution, FileWrite
    language: SourceLanguage,
    patterns: Vec<String>,
}
```

#### Taint Propagation Rules
- **Direct assignment**: `$tainted = $source`
- **Function parameters**: `function($tainted_param)`
- **Return values**: `return $tainted`
- **Array operations**: `$array[$tainted_key]`
- **Object properties**: `$object->$tainted_property`

#### Sanitization Recognition
```rust
pub struct SanitizationPoint {
    name: String,
    sanitizer_type: SanitizerType,
    effectiveness: f64,  // 0.0 to 1.0
    applicable_vulnerabilities: Vec<SecurityIssueType>,
}
```

Common sanitizers:
- **SQL Injection**: Prepared statements, parameterized queries
- **XSS**: HTML encoding, DOM sanitization
- **Command Injection**: Input validation, safe APIs
- **Path Traversal**: Path normalization, allowlist validation

## Configuration

### Basic Configuration

```toml
[security]
# Enable security detector
enabled = true

# Analysis depth
max_analysis_depth = 10
timeout_seconds = 300

# Confidence thresholds
min_confidence_threshold = 0.7
high_confidence_threshold = 0.9

[security.multi_agent]
# Enable specific agents
enable_taint_analysis = true
enable_config_analysis = true
enable_sca = true
enable_ai_enhancement = false  # Requires AI features

# Agent-specific timeouts
taint_analysis_timeout = 120
config_analysis_timeout = 60
sca_timeout = 180

[security.detection]
# OWASP categories to analyze
owasp_categories = [
    "injection",
    "broken_access_control", 
    "security_misconfiguration",
    "vulnerable_components"
]

# Language-specific analysis
rust_unsafe_analysis = true
python_pickle_detection = true
javascript_xss_analysis = true

[security.validation]
# False positive mitigation
enable_cross_validation = true
enable_confidence_scoring = true
enable_bayesian_optimization = true

# Validation thresholds
cross_validation_threshold = 2  # Require 2+ detection methods
confidence_adjustment_factor = 0.1
```

### Advanced Configuration

```toml
[security.taint_analysis]
# Custom taint sources
[[security.taint_analysis.sources]]
name = "custom_input"
source_type = "UserInput"
language = "python"
patterns = ["request.GET", "request.POST", "request.json"]

# Custom taint sinks
[[security.taint_analysis.sinks]]
name = "custom_sql"
sink_type = "SqlQuery"
language = "python"
patterns = ["cursor.execute", "db.raw"]

# Custom sanitizers
[[security.taint_analysis.sanitizers]]
name = "custom_escape"
sanitizer_type = "XssPrevention"
effectiveness = 0.95
patterns = ["escape_html", "sanitize_input"]

[security.patterns]
# Custom vulnerability patterns
[[security.patterns.rust]]
pattern = "env::var().unwrap()"
vulnerability_type = "ImproperErrorHandling"
severity = "Medium"
confidence = 0.6
description = "Environment variable access without error handling"
remediation = "Use env::var().unwrap_or_default() or proper error handling"

[security.knowledge_graph]
# Knowledge graph configuration
enable_structural_analysis = true
enable_semantic_analysis = true
context_window_size = 50
relationship_weight_threshold = 0.3
```

## Usage Examples

### Basic Security Analysis

```rust
use uveddi::analysis::detectors::security::{SecurityDetector, SecurityConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create detector with default configuration
    let config = SecurityConfig::default();
    let detector = SecurityDetector::new(config)?;
    
    // Analyze a single file
    let issues = detector.detect_issues(&parsed_file).await?;
    
    for issue in issues {
        println!("Security Issue: {} (Confidence: {:.2})", 
                issue.title, issue.confidence_score);
        println!("  Severity: {:?}", issue.severity);
        println!("  Location: {}:{}", issue.file_path, issue.line_number);
        println!("  Remediation: {}", issue.remediation);
    }
    
    Ok(())
}
```

### Multi-Agent Analysis

```rust
use uveddi::analysis::detectors::security::{SecurityOrchestrator, MultiAgentConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure multi-agent analysis
    let config = MultiAgentConfig {
        enable_taint_analysis: true,
        enable_config_analysis: true,
        enable_sca: true,
        enable_ai_enhancement: false,
        confidence_threshold: 0.7,
        cross_validation_enabled: true,
        parallel_execution: true,
        agent_timeout_seconds: 120,
    };
    
    // Create orchestrator
    let orchestrator = SecurityOrchestrator::new(config)?;
    
    // Analyze entire codebase
    let analysis_result = orchestrator.analyze_codebase(&dependency_graph).await?;
    
    // Process results
    println!("Security Analysis Results:");
    println!("  Total Issues: {}", analysis_result.total_issues);
    println!("  Critical: {}", analysis_result.critical_count);
    println!("  High: {}", analysis_result.high_count);
    println!("  Medium: {}", analysis_result.medium_count);
    println!("  Low: {}", analysis_result.low_count);
    
    // Generate SARIF report
    let sarif_output = analysis_result.to_sarif()?;
    std::fs::write("security-report.sarif", sarif_output)?;
    
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
    // Create custom taint analysis configuration
    let mut engine = TaintAnalysisEngine::new();
    
    // Add custom source
    let custom_source = TaintSource {
        name: "database_input".to_string(),
        source_type: SourceType::DatabaseRead,
        language: SourceLanguage::Python,
        patterns: vec![
            "User.objects.get".to_string(),
            "cursor.fetchone".to_string(),
        ],
    };
    engine.add_source(custom_source);
    
    // Add custom sink
    let custom_sink = TaintSink {
        name: "template_render".to_string(),
        sink_type: SinkType::TemplateInjection,
        language: SourceLanguage::Python,
        patterns: vec![
            "Template().render".to_string(),
            "render_template_string".to_string(),
        ],
    };
    engine.add_sink(custom_sink);
    
    // Add sanitizer
    let sanitizer = SanitizationPoint {
        name: "django_escape".to_string(),
        sanitizer_type: SanitizerType::HtmlEscape,
        effectiveness: 0.95,
        applicable_vulnerabilities: vec![
            SecurityIssueType::CrossSiteScripting,
            SecurityIssueType::TemplateInjection,
        ],
        patterns: vec!["escape".to_string(), "mark_safe".to_string()],
    };
    engine.add_sanitizer(sanitizer);
    
    // Perform analysis
    let taint_result = engine.analyze(&parsed_file).await?;
    
    for flow in taint_result.vulnerable_flows {
        println!("Vulnerable Data Flow:");
        println!("  Source: {} ({}:{})", flow.source.name, 
                flow.source.file_path, flow.source.line);
        println!("  Sink: {} ({}:{})", flow.sink.name,
                flow.sink.file_path, flow.sink.line);
        println!("  Confidence: {:.2}", flow.confidence);
        
        if !flow.sanitizers.is_empty() {
            println!("  Sanitizers: {:?}", flow.sanitizers);
        }
    }
    
    Ok(())
}
```

### Configuration Security Analysis

```rust
use uveddi::analysis::detectors::security::strategies::ConfigFileAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = ConfigFileAnalyzer::new();
    
    // Analyze Django settings
    let django_issues = analyzer.analyze_django_settings("settings.py").await?;
    
    // Analyze Docker configuration
    let docker_issues = analyzer.analyze_dockerfile("Dockerfile").await?;
    
    // Analyze Nginx configuration
    let nginx_issues = analyzer.analyze_nginx_config("nginx.conf").await?;
    
    let all_issues = [django_issues, docker_issues, nginx_issues].concat();
    
    for issue in all_issues {
        println!("Configuration Issue: {}", issue.title);
        println!("  File: {}", issue.file_path);
        println!("  Severity: {:?}", issue.severity);
        println!("  Issue: {}", issue.description);
        println!("  Fix: {}", issue.remediation);
    }
    
    Ok(())
}
```

## Integration

### CLI Integration

```bash
# Enable security analysis in Uveddi CLI
uveddi analyze ./src --security --output-format sarif --output security-report.sarif

# Configure security detection
uveddi analyze ./src --security \
    --security-config security.toml \
    --min-confidence 0.8 \
    --enable-taint-analysis \
    --enable-sca

# Generate security report
uveddi security-report ./src \
    --format html \
    --output security-report.html \
    --include-remediation \
    --group-by-severity
```

### CI/CD Integration

```yaml
# GitHub Actions workflow
name: Security Analysis
on: [push, pull_request]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Uveddi
        run: |
          curl -sSL https://install.uveddi.com | sh
          
      - name: Run Security Analysis
        run: |
          uveddi analyze . \
            --security \
            --output-format sarif \
            --output security-results.sarif \
            --fail-on-severity critical
            
      - name: Upload SARIF results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: security-results.sarif
          
      - name: Security Gate
        run: |
          # Fail build if critical vulnerabilities found
          if uveddi security-summary security-results.sarif --has-critical; then
            echo "Critical security vulnerabilities found!"
            exit 1
          fi
```

### IDE Integration

```json
// VS Code settings.json
{
  "uveddi.security.enabled": true,
  "uveddi.security.realTimeAnalysis": true,
  "uveddi.security.showInlineWarnings": true,
  "uveddi.security.confidenceThreshold": 0.7,
  "uveddi.security.excludePatterns": [
    "**/test/**",
    "**/vendor/**"
  ]
}
```

### API Integration

```rust
// REST API endpoint for security analysis
use warp::Filter;
use uveddi::analysis::detectors::security::SecurityDetector;

#[tokio::main]
async fn main() {
    let security_route = warp::path("security")
        .and(warp::path("analyze"))
        .and(warp::post())
        .and(warp::body::json())
        .and_then(analyze_security);
        
    warp::serve(security_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn analyze_security(
    request: SecurityAnalysisRequest
) -> Result<impl warp::Reply, warp::Rejection> {
    let detector = SecurityDetector::new(SecurityConfig::default())
        .map_err(|_| warp::reject::custom(InternalError))?;
        
    let issues = detector.analyze_code(&request.source_code).await
        .map_err(|_| warp::reject::custom(InternalError))?;
        
    let response = SecurityAnalysisResponse {
        issues,
        summary: SecuritySummary::from_issues(&issues),
        metadata: AnalysisMetadata {
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };
    
    Ok(warp::reply::json(&response))
}
```

## Performance

### Benchmarks

| Codebase Size | Analysis Time | Memory Usage | Issues Found |
|---------------|---------------|--------------|--------------|
| Small (< 1K LoC) | 2-5 seconds | 50 MB | 5-15 |
| Medium (1K-10K LoC) | 15-30 seconds | 150 MB | 20-50 |
| Large (10K-100K LoC) | 2-5 minutes | 500 MB | 50-200 |
| Enterprise (> 100K LoC) | 10-20 minutes | 1-2 GB | 100-500 |

### Performance Optimization

#### Parallel Execution
```toml
[security.performance]
# Enable parallel agent execution
parallel_agents = true
max_concurrent_agents = 4

# File processing parallelism
parallel_file_analysis = true
max_concurrent_files = 8

# Memory optimization
enable_streaming_analysis = true
memory_threshold_mb = 1024
```

#### Caching
```toml
[security.cache]
# Enable analysis result caching
enable_caching = true
cache_directory = ".uveddi/cache/security"

# Cache invalidation
cache_ttl_hours = 24
invalidate_on_file_change = true
invalidate_on_config_change = true
```

#### Incremental Analysis
```toml
[security.incremental]
# Only analyze changed files
enable_incremental = true
baseline_file = ".uveddi/security-baseline.json"

# Dependency analysis
analyze_dependencies_of_changed = true
analyze_dependents_of_changed = true
```

## Troubleshooting

### Common Issues

#### High Memory Usage
```bash
# Reduce memory usage
uveddi analyze ./src --security \
    --memory-limit 512m \
    --disable-parallel-processing \
    --streaming-analysis

# Or configure in security.toml
[security.performance]
memory_limit_mb = 512
streaming_analysis = true
parallel_agents = false
```

#### False Positives
```bash
# Increase confidence threshold
uveddi analyze ./src --security --min-confidence 0.9

# Enable additional validation
uveddi analyze ./src --security \
    --enable-cross-validation \
    --enable-bayesian-optimization

# Use suppressions
uveddi analyze ./src --security \
    --suppressions-file .uveddi/security-suppressions.json
```

#### Slow Analysis
```bash
# Profile analysis performance
uveddi analyze ./src --security --profile --verbose

# Optimize for speed
uveddi analyze ./src --security \
    --quick-scan \
    --disable-deep-analysis \
    --parallel-processing
```

### Debugging

#### Enable Debug Logging
```bash
RUST_LOG=debug uveddi analyze ./src --security --verbose
```

#### Generate Analysis Metrics
```bash
uveddi analyze ./src --security \
    --output-metrics metrics.json \
    --include-timing \
    --include-memory-usage
```

#### Validate Configuration
```bash
uveddi config validate security.toml
uveddi config test-patterns security.toml
```

### Support

For additional support and troubleshooting:

- **Documentation**: [docs/11-security/](../11-security/)
- **GitHub Issues**: [Issues](https://github.com/botzr/uveddi/issues)
- **Community**: [Discussions](https://github.com/botzr/uveddi/discussions)
- **Security Contact**: [security@uveddi.com](mailto:security@uveddi.com)

---

The Uveddi Security Detector provides comprehensive, privacy-preserving security analysis with enterprise-grade accuracy and developer-friendly insights. For implementation details and API reference, see the [Security Development Guide](../04-development/security-detector-development.md).