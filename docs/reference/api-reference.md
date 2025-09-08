# Uveddi API Reference

Comprehensive API documentation for Uveddi, covering REST API endpoints, Rust library API, and plugin development interfaces.

## Table of Contents

1. [API Status Overview](#api-status-overview)
2. [REST API](#rest-api)
3. [Rust Library API](#rust-library-api)
4. [Plugin Development API](#plugin-development-api)
5. [Authentication & Authorization](#authentication--authorization)
6. [Error Handling](#error-handling)
7. [Rate Limiting](#rate-limiting)
8. [Examples](#examples)
9. [SDKs and Integrations](#sdks-and-integrations)

---

## API Status Overview

> **Version**: 0.9.0-alpha  
> **Last Updated**: January 2025  
> **Base URL**: `http://localhost:8080` (when service is running)

### ⚠️ Important: Current API Status

The API server exists but is **highly unstable** and many documented endpoints are **not implemented**. The web dashboard that would consume these APIs is **completely non-functional**.

### Starting the API Server

```bash
# Requires production build with web features
cargo build --release --features=production

# Start services (often crashes or hangs)
uveddi serve --port 8080 --rendering-port 3001

# Expected issues:
# - Port binding failures
# - Service health check timeouts
# - Rendering service crashes
# - Missing Playwright dependencies
```

---

## REST API

### Working Endpoints ✅

These endpoints actually exist and mostly work:

#### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.9.0-alpha",
  "timestamp": "2025-01-28T10:00:00Z"
}
```

**Example:**
```bash
curl http://localhost:8080/health
```

#### Trigger Analysis
```http
POST /api/v1/analyze
Content-Type: application/json

{
  "path": "/path/to/code",
  "output_format": "json",
  "enable_ai": false
}
```

**Status**: Partially Working  
**Issues**: 
- Requires proper build flags for anti-pattern detection
- May timeout on large codebases
- Response format inconsistent

**Example:**
```bash
curl -X POST http://localhost:8080/api/v1/analyze \
  -H "Content-Type: application/json" \
  -d '{"path": "./src", "output_format": "json"}'
```

#### Get Analysis Report
```http
GET /api/v1/reports/{id}
```

**Status**: Partially Working  
**Issues**:
- Report IDs not always generated correctly
- HTML reports may have broken styling
- Large reports may cause memory issues

**Example:**
```bash
curl http://localhost:8080/api/v1/reports/abc123
```

### Experimental Endpoints ⚠️

These endpoints exist but are unstable:

#### Get Analysis Status
```http
GET /api/v1/status
```

**Issues**: Often returns incorrect status

#### Update Configuration
```http
POST /api/v1/config
Content-Type: application/json

{
  "memory_limit_gb": 8,
  "enable_ai": true,
  "ai_provider": "ollama"
}
```

**Issues**: Changes may not persist

#### Get Performance Metrics
```http
GET /api/v1/metrics
```

**Issues**: Incomplete data, Prometheus integration broken

#### Plugin Management
```http
POST /api/v1/plugins/install
GET /api/v1/plugins
```

**Issues**: 
- Frequently crashes server
- Plugin validation fails
- Requires `wasm-plugins` feature

### Non-Functional Endpoints ❌

These endpoints are documented but **DO NOT EXIST**:

#### Authentication & Authorization
- `POST /api/v1/auth/login` - ❌ Not implemented
- `POST /api/v1/auth/logout` - ❌ Not implemented
- `POST /api/v1/auth/refresh` - ❌ Not implemented
- `GET /api/v1/auth/user` - ❌ Not implemented
- `POST /api/v1/auth/register` - ❌ Not implemented

#### Project Management
- `GET /api/v1/projects` - ❌ Not implemented
- `POST /api/v1/projects` - ❌ Not implemented
- `GET /api/v1/projects/{id}` - ❌ Not implemented
- `DELETE /api/v1/projects/{id}` - ❌ Not implemented

#### Dashboard & UI
- `GET /api/v1/dashboard` - ❌ Not implemented
- `GET /api/v1/dashboard/stats` - ❌ Not implemented
- `GET /api/v1/dashboard/trends` - ❌ Not implemented
- `WebSocket /api/v1/ws` - ❌ Not implemented

#### CI/CD Integration
- `POST /api/v1/webhooks/github` - ❌ Not implemented
- `POST /api/v1/webhooks/gitlab` - ❌ Not implemented
- `GET /api/v1/ci/status` - ❌ Not implemented
- `POST /api/v1/ci/trigger` - ❌ Not implemented

#### Advanced Analysis
- `POST /api/v1/analyze/incremental` - ❌ Not implemented
- `POST /api/v1/analyze/compare` - ❌ Not implemented
- `GET /api/v1/analyze/history` - ❌ Not implemented
- `POST /api/v1/analyze/schedule` - ❌ Not implemented

#### Team Collaboration
- `GET /api/v1/teams` - ❌ Not implemented
- `POST /api/v1/comments` - ❌ Not implemented
- `GET /api/v1/notifications` - ❌ Not implemented

---

## Rust Library API

Uveddi exposes a comprehensive Rust API for advanced integrations and custom tooling.

### Core Analysis Engine

```rust
use uveddi::analysis::AnalysisEngine;
use uveddi::config::AnalysisConfig;

// Create engine with default configuration
let engine = AnalysisEngine::new(AnalysisConfig::default())?;

// Analyze a project
let results = engine.analyze_project("./src").await?;

// Process results
for issue in results.anti_patterns {
    println!("Found {}: {}", issue.pattern_type, issue.description);
}
```

### Configuration API

```rust
use uveddi::config::{AnalysisConfig, ThresholdConfig, AiConfig};

let config = AnalysisConfig {
    thresholds: ThresholdConfig {
        god_object_threshold: 100,
        max_function_lines: 50,
        max_class_lines: 300,
        max_complexity: 15,
        dead_code_confidence: 0.8,
    },
    ai: AiConfig {
        enabled: true,
        provider: AiProvider::Ollama,
        model: "deepseek-coder:6.7b".to_string(),
        api_url: "http://localhost:11434".to_string(),
    },
    memory: MemoryConfig::auto_detect(),
    ..Default::default()
};

let engine = AnalysisEngine::new(config)?;
```

### Detector API

```rust
use uveddi::detectors::{Detector, DetectorResult};

// Use individual detectors
let god_object_detector = uveddi::detectors::GodObjectDetector::new(config);
let results = god_object_detector.detect(&ast, &context)?;

// Custom detector implementation
struct CustomDetector {
    threshold: usize,
}

impl Detector for CustomDetector {
    fn detect(&self, ast: &Ast, context: &AnalysisContext) -> DetectorResult {
        // Custom detection logic
        Ok(vec![])
    }
}
```

### AST Processing API

```rust
use uveddi::ast::{AstParser, Language};

let parser = AstParser::new(Language::Rust)?;
let ast = parser.parse(source_code, None)?;

// Query AST
let query = "(function_item name: (identifier) @name)";
let matches = ast.query(query)?;

for m in matches {
    let name_node = m.captures[0].node;
    let function_name = ast.node_text(name_node)?;
    println!("Found function: {}", function_name);
}
```

### Plugin System API

```rust
use uveddi::plugins::{PluginManager, PluginConfig};

let mut plugin_manager = PluginManager::new();

// Load plugin
plugin_manager.load_plugin("./custom-detector.wasm")?;

// Run plugin analysis
let plugin_results = plugin_manager.analyze_file(
    "custom-detector", 
    file_content, 
    &context
).await?;
```

### Report Generation API

```rust
use uveddi::reporting::{ReportGenerator, ReportFormat, ReportConfig};

let generator = ReportGenerator::new();

// Generate HTML report
let html_report = generator.generate(
    &analysis_results,
    ReportFormat::Html,
    ReportConfig {
        include_diagrams: true,
        theme: Theme::Dark,
        include_timing: true,
    }
)?;

// Generate JSON report
let json_report = generator.generate(
    &analysis_results,
    ReportFormat::Json,
    ReportConfig::default()
)?;
```

### Memory Management API

```rust
use uveddi::memory::{MemoryManager, MemoryProfile};

// Auto-detect optimal memory settings
let memory_config = MemoryManager::auto_configure()?;

// Manual memory configuration
let memory_config = MemoryConfig {
    profile: MemoryProfile::Large,
    limit_gb: 16.0,
    enable_optimization: true,
    pool_sizes: PoolSizes::large(),
};

let engine = AnalysisEngine::with_memory_config(config, memory_config)?;
```

### Error Types

```rust
use uveddi::error::{AnalysisError, ErrorKind};

match analysis_result {
    Ok(results) => println!("Analysis completed: {} issues", results.issues.len()),
    Err(AnalysisError::ParseError { file, line, message }) => {
        eprintln!("Parse error in {}:{}: {}", file, line, message);
    },
    Err(AnalysisError::MemoryLimitExceeded { limit, used }) => {
        eprintln!("Memory limit exceeded: used {}GB, limit {}GB", used, limit);
    },
    Err(AnalysisError::TimeoutError { timeout }) => {
        eprintln!("Analysis timed out after {}s", timeout);
    },
    Err(e) => eprintln!("Analysis failed: {}", e),
}
```

---

## Plugin Development API

Complete API for developing WASM-based plugins:

### Plugin Interface

Every plugin must export these functions:

#### analyze_file
```rust
#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    // Parse input
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => return create_error_result(&format!("Parse error: {}", e)),
    };
    
    // Perform analysis
    let issues = perform_analysis(&input);
    
    // Create result
    let result = PluginAnalysisResult {
        plugin_name: "my-plugin".to_string(),
        issues,
        metrics: Some(calculate_metrics(&input)),
        metadata: HashMap::new(),
    };
    
    serde_json::to_vec(&result).unwrap_or_else(|_| {
        create_error_result("Serialization failed")
    })
}
```

#### get_plugin_info
```rust
#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8> {
    let info = serde_json::json!({
        "id": "my-plugin",
        "name": "My Custom Plugin",
        "version": "1.0.0",
        "description": "Custom analysis plugin",
        "author": "Your Name",
        "license": "MIT",
        "api_version": "1.0",
        "supported_languages": ["rust", "python", "javascript"],
        "capabilities": {
            "static_analysis": true,
            "security_scanning": false,
            "performance_analysis": true,
            "metrics_calculation": true
        }
    });
    
    serde_json::to_vec(&info).unwrap_or_default()
}
```

### Host Functions Available to Plugins

#### Logging
```rust
extern "C" {
    pub fn host_log_debug(msg_ptr: *const u8, msg_len: usize);
    pub fn host_log_info(msg_ptr: *const u8, msg_len: usize);
    pub fn host_log_warn(msg_ptr: *const u8, msg_len: usize);
    pub fn host_log_error(msg_ptr: *const u8, msg_len: usize);
}

// Convenience wrapper
pub fn log_info(message: &str) {
    unsafe {
        host_log_info(message.as_ptr(), message.len());
    }
}
```

#### AST Parsing
```rust
extern "C" {
    pub fn host_parse_ast(
        code_ptr: *const u8, 
        code_len: usize, 
        lang_ptr: *const u8, 
        lang_len: usize
    ) -> u64;
    
    pub fn host_query_ast(
        ast_handle: u64,
        query_ptr: *const u8,
        query_len: usize
    ) -> u64;
    
    pub fn host_free_ast(ast_handle: u64);
}
```

#### Configuration Access
```rust
extern "C" {
    pub fn host_get_config(key_ptr: *const u8, key_len: usize) -> u64;
}

pub fn get_config_value(key: &str) -> Option<String> {
    unsafe {
        let handle = host_get_config(key.as_ptr(), key.len());
        if handle != 0 {
            // Extract string from handle
            Some("config_value".to_string())
        } else {
            None
        }
    }
}
```

### Plugin Data Structures

#### Input/Output Types
```rust
#[derive(Deserialize)]
pub struct PluginFileInput {
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub context: Option<AnalysisContext>,
}

#[derive(Serialize)]
pub struct PluginAnalysisResult {
    pub plugin_name: String,
    pub issues: Vec<PluginIssue>,
    pub metrics: Option<PluginMetrics>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct PluginIssue {
    pub issue_type: String,
    pub severity: String,
    pub message: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub column: Option<u32>,
    pub suggestion: Option<String>,
    pub confidence: Option<f64>,
    pub tags: Vec<String>,
}
```

---

## Authentication & Authorization

> **Status**: Not implemented in current version

### Planned Authentication Methods

#### JWT Token Authentication
```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "password"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "...",
  "expires_in": 3600,
  "user": {
    "id": "123",
    "username": "user@example.com",
    "roles": ["analyst"]
  }
}
```

#### API Key Authentication
```http
GET /api/v1/analyze
Authorization: Bearer your-api-key
```

#### OAuth2 Integration
```http
GET /api/v1/auth/oauth/github
GET /api/v1/auth/oauth/google
```

### Permission System

#### Role-Based Access Control
- **Admin**: Full system access
- **Analyst**: Analysis and reporting
- **Viewer**: Read-only access
- **CI**: Automated analysis only

#### Endpoint Permissions
```json
{
  "/api/v1/analyze": ["analyst", "admin", "ci"],
  "/api/v1/reports": ["viewer", "analyst", "admin"],
  "/api/v1/config": ["admin"],
  "/api/v1/plugins": ["admin"]
}
```

---

## Error Handling

### Standard Error Response Format

All API endpoints return errors in this format:

```json
{
  "error": {
    "code": "ANALYSIS_FAILED",
    "message": "Analysis failed due to parsing error",
    "details": {
      "file": "src/main.rs",
      "line": 42,
      "column": 15
    },
    "suggestion": "Check syntax around line 42",
    "timestamp": "2025-01-28T10:00:00Z",
    "request_id": "req_12345"
  }
}
```

### Error Codes

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | `INVALID_REQUEST` | Malformed request |
| 401 | `UNAUTHORIZED` | Authentication required |
| 403 | `FORBIDDEN` | Insufficient permissions |
| 404 | `NOT_FOUND` | Resource not found |
| 409 | `CONFLICT` | Resource conflict |
| 422 | `VALIDATION_ERROR` | Invalid input data |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Server error |
| 503 | `SERVICE_UNAVAILABLE` | Service temporarily unavailable |

### Analysis-Specific Errors

| Error Code | Description | Solution |
|------------|-------------|----------|
| `ANALYSIS_TIMEOUT` | Analysis exceeded time limit | Increase timeout or reduce scope |
| `MEMORY_LIMIT_EXCEEDED` | Analysis used too much memory | Increase memory limit |
| `PARSE_ERROR` | Could not parse source code | Check file syntax |
| `UNSUPPORTED_LANGUAGE` | Language not supported | Use supported language |
| `FILE_NOT_FOUND` | Source file not found | Verify file path |
| `AI_PROVIDER_ERROR` | AI service error | Check AI provider status |

---

## Rate Limiting

> **Status**: Planned feature

### Rate Limit Headers

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
X-RateLimit-Retry-After: 60
```

### Rate Limit Tiers

| Tier | Requests/Hour | Concurrent | Analysis Timeout |
|------|---------------|------------|------------------|
| Free | 100 | 1 | 5 minutes |
| Pro | 1000 | 5 | 30 minutes |
| Enterprise | Unlimited | 20 | Unlimited |

---

## Examples

### Working API Examples

#### Basic Health Check
```bash
# Simple health check
curl http://localhost:8080/health
```

#### Trigger Analysis
```bash
# Analyze project with JSON output
curl -X POST http://localhost:8080/api/v1/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "path": "./src",
    "output_format": "json",
    "enable_ai": false,
    "detectors": ["god-object", "dead-code"]
  }'
```

#### Get Analysis Results
```bash
# Retrieve analysis report
curl http://localhost:8080/api/v1/reports/abc123 \
  -H "Accept: application/json"
```

### Integration Examples

#### Python Integration
```python
import requests
import json

# Start analysis
response = requests.post('http://localhost:8080/api/v1/analyze', 
    json={
        'path': './src',
        'output_format': 'json',
        'enable_ai': True
    }
)

if response.status_code == 200:
    result = response.json()
    report_id = result['report_id']
    
    # Get report
    report_response = requests.get(f'http://localhost:8080/api/v1/reports/{report_id}')
    report = report_response.json()
    
    print(f"Found {len(report['issues'])} issues")
else:
    print(f"Analysis failed: {response.text}")
```

#### JavaScript/Node.js Integration
```javascript
const axios = require('axios');

async function analyzeProject(path) {
    try {
        const response = await axios.post('http://localhost:8080/api/v1/analyze', {
            path: path,
            output_format: 'json',
            enable_ai: false
        });
        
        const { report_id } = response.data;
        
        // Get report
        const reportResponse = await axios.get(`http://localhost:8080/api/v1/reports/${report_id}`);
        return reportResponse.data;
    } catch (error) {
        console.error('Analysis failed:', error.response?.data || error.message);
        throw error;
    }
}

analyzeProject('./src').then(report => {
    console.log(`Analysis complete: ${report.issues.length} issues found`);
});
```

#### Rust Integration
```rust
use serde_json::json;
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Trigger analysis
    let analysis_request = json!({
        "path": "./src",
        "output_format": "json",
        "enable_ai": true,
        "ai_provider": "ollama",
        "ollama_model": "deepseek-coder:6.7b"
    });
    
    let response = client
        .post("http://localhost:8080/api/v1/analyze")
        .json(&analysis_request)
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Analysis result: {}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}
```

---

## SDKs and Integrations

### Official SDKs (Planned)

#### Python SDK
```bash
pip install uveddi-python
```

```python
from uveddi import AnalysisClient

client = AnalysisClient("http://localhost:8080")
result = client.analyze("./src", enable_ai=True)
print(f"Found {len(result.issues)} issues")
```

#### Node.js SDK
```bash
npm install @uveddi/client
```

```javascript
const { AnalysisClient } = require('@uveddi/client');

const client = new AnalysisClient('http://localhost:8080');
const result = await client.analyze('./src', { enableAI: true });
console.log(`Found ${result.issues.length} issues`);
```

#### Go SDK
```bash
go get github.com/botzrDev/uveddi-go
```

```go
import "github.com/botzrDev/uveddi-go"

client := uveddi.NewClient("http://localhost:8080")
result, err := client.Analyze("./src", &uveddi.AnalysisOptions{
    EnableAI: true,
})
```

### IDE Integrations

#### VS Code Extension
```bash
# Install from marketplace
code --install-extension uveddi.vscode-uveddi

# Or install from VSIX
code --install-extension uveddi-vscode-1.0.0.vsix
```

#### IntelliJ Plugin
```bash
# Install from JetBrains Marketplace
# Settings > Plugins > Search "Uveddi"
```

### CI/CD Integrations

#### GitHub Actions
```yaml
name: Uveddi Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Uveddi Analysis
        uses: botzrDev/uveddi-action@v1
        with:
          path: './src'
          fail-on-critical: true
          output-format: 'json'
```

#### GitLab CI
```yaml
uveddi_analysis:
  image: uveddi/cli:latest
  script:
    - uveddi analyze ./src --output-format json --fail-on-critical
  artifacts:
    reports:
      codequality: analysis.json
```

---

## API Development Roadmap

### Current (v0.9.0-alpha)
- Basic REST structure implemented
- Health and analysis endpoints partially working
- No authentication or authorization
- Minimal error handling

### Beta (v0.9.5)
- Stabilize existing endpoints
- Add basic authentication
- Improve error responses
- Add request validation
- WebSocket support for real-time updates

### v1.0
- Full REST API implementation
- GraphQL support
- Comprehensive authentication system
- Rate limiting and caching
- OpenAPI 3.0 specification
- Official SDKs for Python, Node.js, Go

### v1.1+
- Webhook system for CI/CD integration
- Advanced analytics and reporting endpoints
- Team collaboration features
- Plugin marketplace API
- Multi-tenant support

---

This API reference reflects the current implementation status and planned features for Uveddi. Many endpoints are still under development - see the [Known Issues](../known-issues.md) for current limitations.