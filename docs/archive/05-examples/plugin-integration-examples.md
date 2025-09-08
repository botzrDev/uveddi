# Plugin Integration Examples

## Overview

This document provides comprehensive examples of integrating WASM plugins with Uveddi's analysis system. Examples cover common use cases, integration patterns, and real-world scenarios for extending Uveddi's analysis capabilities.

## Basic Integration Examples

### 1. Simple Code Quality Plugin

A basic plugin that detects code quality issues:

```rust
// src/lib.rs - Basic code quality plugin
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct PluginFileInput {
    pub file_path: String,
    pub content: String,
    pub language: String,
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
}

#[derive(Serialize)]
pub struct PluginAnalysisResult {
    pub plugin_name: String,
    pub issues: Vec<PluginIssue>,
    pub metadata: HashMap<String, String>,
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = serde_json::from_slice(file_data).unwrap();
    let mut issues = Vec::new();
    
    // Check for long lines
    for (line_num, line) in input.content.lines().enumerate() {
        if line.len() > 120 {
            issues.push(PluginIssue {
                issue_type: "LONG_LINE".to_string(),
                severity: "WARNING".to_string(),
                message: format!("Line length {} exceeds 120 characters", line.len()),
                file_path: input.file_path.clone(),
                line_number: Some(line_num as u32 + 1),
                column: Some(121),
                suggestion: Some("Consider breaking long lines for better readability".to_string()),
            });
        }
    }
    
    // Check for TODO comments
    for (line_num, line) in input.content.lines().enumerate() {
        if line.to_lowercase().contains("todo") || line.to_lowercase().contains("fixme") {
            issues.push(PluginIssue {
                issue_type: "TECH_DEBT".to_string(),
                severity: "INFO".to_string(),
                message: "Technical debt marker found".to_string(),
                file_path: input.file_path.clone(),
                line_number: Some(line_num as u32 + 1),
                column: line.find("TODO").or_else(|| line.find("FIXME")).map(|pos| pos as u32),
                suggestion: Some("Consider creating a proper issue to track this work".to_string()),
            });
        }
    }
    
    let result = PluginAnalysisResult {
        plugin_name: "code-quality-checker".to_string(),
        issues,
        metadata: [
            ("lines_analyzed".to_string(), input.content.lines().count().to_string()),
            ("file_size_bytes".to_string(), input.content.len().to_string()),
        ].into(),
    };
    
    serde_json::to_vec(&result).unwrap_or_default()
}

#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8> {
    let info = serde_json::json!({
        "name": "code-quality-checker",
        "version": "1.0.0",
        "description": "Basic code quality checks for line length and technical debt markers",
        "supported_languages": ["rust", "python", "javascript", "typescript"],
        "capabilities": ["static_analysis", "style_checking"]
    });
    
    serde_json::to_vec(&info).unwrap_or_default()
}
```

### Plugin Manifest

```toml
# plugin.toml
[plugin]
name = "code-quality-checker"
version = "1.0.0"
description = "Basic code quality analysis plugin"
author = "Uveddi Team <team@uveddi.dev>"
license = "MIT"

[capabilities]
permissions = ["ConfigRead", "Logging"]
max_memory_mb = 16
max_execution_seconds = 15
fuel_limit = 1000000

[dependencies]
min_uveddi_version = "0.9.0"
requires_tree_sitter = false
supported_languages = ["rust", "python", "javascript", "typescript"]

[detection]
anti_pattern_types = ["CODE_SMELL"]
issue_categories = ["STYLE", "MAINTAINABILITY"]
severity_levels = ["INFO", "WARNING"]
```

### Usage Example

```bash
# Build and install the plugin
cd plugins/code-quality-checker
cargo build --release --target wasm32-wasi
uveddi plugin install target/wasm32-wasi/release/code_quality_checker.wasm

# Run analysis with the plugin
uveddi analyze ./src --plugins code-quality-checker --output-format html --output report.html

# Test plugin directly
uveddi plugin test code-quality-checker --test-file src/main.rs --verbose
```

## Advanced Integration Examples

### 2. Security Analysis Plugin

A more sophisticated plugin that performs security-focused analysis:

```rust
// src/lib.rs - Security analysis plugin
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

// Host function bindings for advanced features
extern "C" {
    fn host_log_message(level: u32, msg_ptr: *const u8, msg_len: usize);
    fn host_get_config(key_ptr: *const u8, key_len: usize) -> u64;
}

pub fn log_info(message: &str) {
    unsafe {
        host_log_message(1, message.as_ptr(), message.len());
    }
}

pub fn get_config_value(key: &str) -> Option<String> {
    unsafe {
        let handle = host_get_config(key.as_ptr(), key.len());
        // Simplified - actual implementation would extract data from handle
        if handle != 0 {
            Some("config_value".to_string())
        } else {
            None
        }
    }
}

#[derive(Deserialize)]
pub struct PluginFileInput {
    pub file_path: String,
    pub content: String,
    pub language: String,
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
}

#[derive(Serialize)]
pub struct PluginAnalysisResult {
    pub plugin_name: String,
    pub issues: Vec<PluginIssue>,
    pub metadata: HashMap<String, String>,
}

pub struct SecurityPatterns {
    hardcoded_secrets: Regex,
    sql_injection: Regex,
    unsafe_functions: Regex,
    weak_crypto: Regex,
}

impl SecurityPatterns {
    pub fn new() -> Self {
        Self {
            hardcoded_secrets: Regex::new(r#"(?i)(password|secret|token|key)\s*[:=]\s*["'][^"']{8,}["']"#).unwrap(),
            sql_injection: Regex::new(r#"(?i)(select|insert|update|delete).*\+.*["']"#).unwrap(),
            unsafe_functions: Regex::new(r#"(?i)(eval|exec|system|shell_exec|passthru)\s*\("#).unwrap(),
            weak_crypto: Regex::new(r#"(?i)(md5|sha1|des|rc4)\s*\("#).unwrap(),
        }
    }
    
    pub fn check_hardcoded_secrets(&self, line: &str, line_num: usize, file_path: &str) -> Option<PluginIssue> {
        if self.hardcoded_secrets.is_match(line) {
            Some(PluginIssue {
                issue_type: "HARDCODED_SECRET".to_string(),
                severity: "ERROR".to_string(),
                message: "Potential hardcoded secret detected".to_string(),
                file_path: file_path.to_string(),
                line_number: Some(line_num as u32 + 1),
                column: self.hardcoded_secrets.find(line).map(|m| m.start() as u32),
                suggestion: Some("Move sensitive data to environment variables or secure configuration".to_string()),
            })
        } else {
            None
        }
    }
    
    pub fn check_sql_injection(&self, line: &str, line_num: usize, file_path: &str) -> Option<PluginIssue> {
        if self.sql_injection.is_match(line) {
            Some(PluginIssue {
                issue_type: "SQL_INJECTION_RISK".to_string(),
                severity: "ERROR".to_string(),
                message: "Potential SQL injection vulnerability detected".to_string(),
                file_path: file_path.to_string(),
                line_number: Some(line_num as u32 + 1),
                column: self.sql_injection.find(line).map(|m| m.start() as u32),
                suggestion: Some("Use parameterized queries or prepared statements".to_string()),
            })
        } else {
            None
        }
    }
    
    pub fn check_unsafe_functions(&self, line: &str, line_num: usize, file_path: &str) -> Option<PluginIssue> {
        if self.unsafe_functions.is_match(line) {
            Some(PluginIssue {
                issue_type: "UNSAFE_FUNCTION".to_string(),
                severity: "WARNING".to_string(),
                message: "Potentially unsafe function usage detected".to_string(),
                file_path: file_path.to_string(),
                line_number: Some(line_num as u32 + 1),
                column: self.unsafe_functions.find(line).map(|m| m.start() as u32),
                suggestion: Some("Consider safer alternatives or add input validation".to_string()),
            })
        } else {
            None
        }
    }
    
    pub fn check_weak_crypto(&self, line: &str, line_num: usize, file_path: &str) -> Option<PluginIssue> {
        if self.weak_crypto.is_match(line) {
            Some(PluginIssue {
                issue_type: "WEAK_CRYPTOGRAPHY".to_string(),
                severity: "WARNING".to_string(),
                message: "Weak cryptographic algorithm detected".to_string(),
                file_path: file_path.to_string(),
                line_number: Some(line_num as u32 + 1),
                column: self.weak_crypto.find(line).map(|m| m.start() as u32),
                suggestion: Some("Use modern cryptographic algorithms like SHA-256, AES-256".to_string()),
            })
        } else {
            None
        }
    }
}

#[export_name = "analyze_file"]
pub fn analyze_file(file_data: &[u8]) -> Vec<u8> {
    let input: PluginFileInput = match serde_json::from_slice(file_data) {
        Ok(input) => input,
        Err(e) => {
            log_info(&format!("Failed to parse input: {}", e));
            return create_error_result("Invalid input format");
        }
    };
    
    log_info(&format!("Analyzing file: {} ({} lines)", input.file_path, input.content.lines().count()));
    
    // Load configuration
    let severity_threshold = get_config_value("security_scanner.severity_threshold")
        .unwrap_or_else(|| "WARNING".to_string());
    
    let patterns = SecurityPatterns::new();
    let mut issues = Vec::new();
    
    // Analyze each line for security issues
    for (line_num, line) in input.content.lines().enumerate() {
        // Check for various security issues
        if let Some(issue) = patterns.check_hardcoded_secrets(line, line_num, &input.file_path) {
            issues.push(issue);
        }
        
        if let Some(issue) = patterns.check_sql_injection(line, line_num, &input.file_path) {
            issues.push(issue);
        }
        
        if let Some(issue) = patterns.check_unsafe_functions(line, line_num, &input.file_path) {
            issues.push(issue);
        }
        
        if let Some(issue) = patterns.check_weak_crypto(line, line_num, &input.file_path) {
            issues.push(issue);
        }
    }
    
    // Filter by severity threshold
    issues.retain(|issue| should_include_severity(&issue.severity, &severity_threshold));
    
    log_info(&format!("Found {} security issues", issues.len()));
    
    let result = PluginAnalysisResult {
        plugin_name: "security-scanner".to_string(),
        issues,
        metadata: [
            ("analysis_type".to_string(), "security".to_string()),
            ("patterns_checked".to_string(), "4".to_string()),
            ("severity_threshold".to_string(), severity_threshold),
            ("lines_scanned".to_string(), input.content.lines().count().to_string()),
        ].into(),
    };
    
    serde_json::to_vec(&result).unwrap_or_else(|_| create_error_result("Serialization failed"))
}

fn should_include_severity(issue_severity: &str, threshold: &str) -> bool {
    let severity_levels = ["INFO", "WARNING", "ERROR"];
    let issue_level = severity_levels.iter().position(|&s| s == issue_severity).unwrap_or(0);
    let threshold_level = severity_levels.iter().position(|&s| s == threshold).unwrap_or(1);
    issue_level >= threshold_level
}

fn create_error_result(error_msg: &str) -> Vec<u8> {
    let result = PluginAnalysisResult {
        plugin_name: "security-scanner".to_string(),
        issues: vec![],
        metadata: [("error".to_string(), error_msg.to_string())].into(),
    };
    serde_json::to_vec(&result).unwrap_or_default()
}

#[export_name = "get_plugin_info"]
pub fn get_plugin_info() -> Vec<u8> {
    let info = serde_json::json!({
        "name": "security-scanner",
        "version": "1.2.0",
        "description": "Security vulnerability scanner for common issues",
        "supported_languages": ["rust", "python", "javascript", "typescript", "php", "java"],
        "capabilities": ["security_analysis", "pattern_matching"],
        "patterns": {
            "hardcoded_secrets": "Detects potential hardcoded secrets and passwords",
            "sql_injection": "Identifies potential SQL injection vulnerabilities",
            "unsafe_functions": "Flags usage of potentially unsafe functions",
            "weak_cryptography": "Detects use of weak cryptographic algorithms"
        }
    });
    
    serde_json::to_vec(&info).unwrap_or_default()
}
```

### Security Plugin Manifest

```toml
# plugin.toml for security scanner
[plugin]
name = "security-scanner"
version = "1.2.0"
description = "Comprehensive security vulnerability scanner"
author = "Security Team <security@uveddi.dev>"
license = "Apache-2.0"
homepage = "https://github.com/uveddi/security-scanner-plugin"

[capabilities]
permissions = ["ConfigRead", "Logging", "TempFileCreate"]
max_memory_mb = 64
max_execution_seconds = 60
fuel_limit = 3000000

[dependencies]
min_uveddi_version = "0.9.0"
requires_tree_sitter = false
supported_languages = ["rust", "python", "javascript", "typescript", "php", "java", "go"]

[detection]
anti_pattern_types = ["SECURITY_VULNERABILITY", "INSECURE_CODING"]
issue_categories = ["SECURITY", "CRYPTOGRAPHY", "INPUT_VALIDATION"]
severity_levels = ["INFO", "WARNING", "ERROR", "CRITICAL"]

[configuration]
# Configuration keys that this plugin reads
config_keys = [
    "security_scanner.severity_threshold",
    "security_scanner.ignore_patterns",
    "security_scanner.custom_rules"
]
```

## CI/CD Integration Examples

### 3. GitHub Actions Integration

```yaml
# .github/workflows/security-analysis.yml
name: Security Analysis with Uveddi

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Uveddi
      run: |
        wget -O uveddi.tar.gz https://github.com/botzrDev/uveddi/releases/latest/download/uveddi-linux-amd64.tar.gz
        tar -xzf uveddi.tar.gz
        sudo mv uveddi /usr/local/bin/
        
    - name: Install Security Plugin
      run: |
        wget -O security-scanner.wasm https://github.com/uveddi/security-scanner-plugin/releases/latest/download/security-scanner.wasm
        uveddi plugin install security-scanner.wasm
        
    - name: Run Security Analysis
      run: |
        uveddi analyze ./src \
          --plugins security-scanner \
          --output-format json \
          --output security-report.json \
          --fail-on-issues \
          --max-issues 0 \
          --severity-threshold ERROR
          
    - name: Upload Security Report
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: security-report
        path: security-report.json
        
    - name: Comment PR with Results
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          try {
            const report = JSON.parse(fs.readFileSync('security-report.json', 'utf8'));
            const securityIssues = report.issues.filter(issue => 
              issue.detector === 'security-scanner' && 
              ['ERROR', 'CRITICAL'].includes(issue.severity)
            );
            
            if (securityIssues.length > 0) {
              const comment = `## 🔒 Security Analysis Results
              
Found ${securityIssues.length} security issues that require attention:

${securityIssues.map(issue => 
  `- **${issue.issue_type}** in ${issue.file_path}:${issue.line_number || '?'}\n  ${issue.message}`
).join('\n')}

Please address these security issues before merging.`;
              
              github.rest.issues.createComment({
                issue_number: context.issue.number,
                owner: context.repo.owner,
                repo: context.repo.repo,
                body: comment
              });
            }
          } catch (error) {
            console.log('No security report found or error reading report');
          }
```

### 4. Jenkins Pipeline Integration

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    stages {
        stage('Setup') {
            steps {
                // Install Uveddi
                sh '''
                    if ! command -v uveddi &> /dev/null; then
                        wget -O uveddi.tar.gz https://github.com/botzrDev/uveddi/releases/latest/download/uveddi-linux-amd64.tar.gz
                        tar -xzf uveddi.tar.gz
                        sudo mv uveddi /usr/local/bin/
                    fi
                '''
                
                // Install plugins
                sh '''
                    uveddi plugin list || true
                    
                    # Install security scanner if not present
                    if ! uveddi plugin list | grep -q security-scanner; then
                        wget -O security-scanner.wasm https://plugins.uveddi.dev/security-scanner.wasm
                        uveddi plugin install security-scanner.wasm
                    fi
                    
                    # Install code quality checker
                    if ! uveddi plugin list | grep -q code-quality-checker; then
                        wget -O code-quality.wasm https://plugins.uveddi.dev/code-quality-checker.wasm
                        uveddi plugin install code-quality.wasm
                    fi
                '''
            }
        }
        
        stage('Code Quality Analysis') {
            steps {
                sh '''
                    uveddi analyze ./src \
                        --plugins code-quality-checker \
                        --output-format json \
                        --output quality-report.json
                '''
            }
        }
        
        stage('Security Analysis') {
            steps {
                sh '''
                    uveddi analyze ./src \
                        --plugins security-scanner \
                        --output-format json \
                        --output security-report.json
                '''
            }
        }
        
        stage('Generate Combined Report') {
            steps {
                sh '''
                    uveddi analyze ./src \
                        --plugins security-scanner,code-quality-checker \
                        --output-format html \
                        --output combined-report.html \
                        --enable-ai \
                        --ollama-model deepseek-coder:6.7b
                '''
            }
        }
        
        stage('Quality Gates') {
            steps {
                script {
                    // Parse security report
                    def securityReport = readJSON file: 'security-report.json'
                    def criticalIssues = securityReport.issues.findAll { 
                        it.severity == 'CRITICAL' || it.severity == 'ERROR' 
                    }
                    
                    if (criticalIssues.size() > 0) {
                        error("Found ${criticalIssues.size()} critical security issues. Build failed.")
                    }
                    
                    // Parse quality report
                    def qualityReport = readJSON file: 'quality-report.json'
                    def qualityScore = calculateQualityScore(qualityReport)
                    
                    if (qualityScore < 80) {
                        unstable("Code quality score ${qualityScore}% is below threshold (80%). Build marked as unstable.")
                    }
                }
            }
        }
    }
    
    post {
        always {
            // Archive reports
            archiveArtifacts artifacts: '*.json,*.html', allowEmptyArchive: true
            
            // Publish test results if available
            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: '.',
                reportFiles: 'combined-report.html',
                reportName: 'Uveddi Analysis Report'
            ])
        }
        
        failure {
            emailext (
                subject: "Security Issues Found: ${env.JOB_NAME} - ${env.BUILD_NUMBER}",
                body: """
                Security analysis has found critical issues in ${env.JOB_NAME} build ${env.BUILD_NUMBER}.
                
                Please check the detailed report at: ${env.BUILD_URL}
                
                This build has been marked as failed due to security policy violations.
                """,
                to: "${env.SECURITY_TEAM_EMAIL}"
            )
        }
    }
}

def calculateQualityScore(report) {
    def totalIssues = report.issues.size()
    def errorCount = report.issues.count { it.severity == 'ERROR' }
    def warningCount = report.issues.count { it.severity == 'WARNING' }
    
    // Simple scoring formula
    def score = Math.max(0, 100 - (errorCount * 10) - (warningCount * 2))
    return score
}
```

## Multi-Plugin Analysis Examples

### 5. Comprehensive Analysis Pipeline

```bash
#!/bin/bash
# comprehensive-analysis.sh - Multi-plugin analysis script

set -e

# Configuration
PROJECT_DIR="${1:-.}"
OUTPUT_DIR="analysis-results"
PLUGINS="security-scanner,code-quality-checker,performance-analyzer,architecture-validator"

# Setup
echo "🔧 Setting up analysis environment..."
mkdir -p "$OUTPUT_DIR"
cd "$PROJECT_DIR"

# Verify plugins are installed
echo "🔍 Verifying plugins..."
IFS=',' read -ra PLUGIN_ARRAY <<< "$PLUGINS"
for plugin in "${PLUGIN_ARRAY[@]}"; do
    if ! uveddi plugin list | grep -q "$plugin"; then
        echo "❌ Plugin $plugin not found. Please install it first."
        exit 1
    fi
done

# Run individual analyses
echo "🏃 Running individual plugin analyses..."

# Security analysis
echo "  🔒 Security analysis..."
uveddi analyze . \
    --plugins security-scanner \
    --output-format json \
    --output "$OUTPUT_DIR/security-report.json" \
    --ignore-patterns "vendor/*,node_modules/*,target/*"

# Code quality analysis  
echo "  ✨ Code quality analysis..."
uveddi analyze . \
    --plugins code-quality-checker \
    --output-format json \
    --output "$OUTPUT_DIR/quality-report.json" \
    --include-tests

# Performance analysis
echo "  🚀 Performance analysis..."
uveddi analyze . \
    --plugins performance-analyzer \
    --output-format json \
    --output "$OUTPUT_DIR/performance-report.json" \
    --timeout-seconds 120

# Architecture analysis
echo "  🏗️ Architecture analysis..."
uveddi analyze . \
    --plugins architecture-validator \
    --output-format json \
    --output "$OUTPUT_DIR/architecture-report.json" \
    --max-depth 10

# Combined analysis with AI insights
echo "  🤖 Combined analysis with AI insights..."
uveddi analyze . \
    --plugins "$PLUGINS" \
    --output-format html \
    --output "$OUTPUT_DIR/comprehensive-report.html" \
    --enable-ai \
    --ollama-model deepseek-coder:6.7b \
    --parallel-jobs 4

# Generate summary report
echo "📊 Generating summary report..."
python3 << 'EOF'
import json
import os
from collections import defaultdict

def load_report(filename):
    try:
        with open(filename, 'r') as f:
            return json.load(f)
    except:
        return {"issues": [], "metadata": {}}

# Load all reports
reports = {
    "security": load_report("analysis-results/security-report.json"),
    "quality": load_report("analysis-results/quality-report.json"),
    "performance": load_report("analysis-results/performance-report.json"),
    "architecture": load_report("analysis-results/architecture-report.json")
}

# Aggregate statistics
total_issues = 0
severity_counts = defaultdict(int)
plugin_counts = defaultdict(int)
file_counts = defaultdict(int)

for report_name, report in reports.items():
    for issue in report.get("issues", []):
        total_issues += 1
        severity_counts[issue.get("severity", "UNKNOWN")] += 1
        plugin_counts[issue.get("detector", "unknown")] += 1
        file_counts[issue.get("file_path", "unknown")] += 1

# Generate summary
summary = {
    "analysis_date": "2024-01-01T00:00:00Z",  # Would be current time
    "total_issues": total_issues,
    "severity_breakdown": dict(severity_counts),
    "plugin_breakdown": dict(plugin_counts),
    "files_with_issues": len(file_counts),
    "top_problematic_files": sorted(file_counts.items(), 
                                   key=lambda x: x[1], 
                                   reverse=True)[:10],
    "reports_generated": list(reports.keys()),
    "quality_score": max(0, 100 - (severity_counts["ERROR"] * 10) - 
                        (severity_counts["WARNING"] * 2))
}

# Save summary
with open("analysis-results/analysis-summary.json", "w") as f:
    json.dump(summary, f, indent=2)

print(f"📈 Analysis Summary:")
print(f"  Total Issues: {total_issues}")
print(f"  Quality Score: {summary['quality_score']}%")
print(f"  Critical/Error Issues: {severity_counts.get('CRITICAL', 0) + severity_counts.get('ERROR', 0)}")
print(f"  Files Analyzed: {len(file_counts)}")
EOF

echo "✅ Analysis complete! Results saved to $OUTPUT_DIR/"
echo "📋 View comprehensive report: $OUTPUT_DIR/comprehensive-report.html"
echo "📊 View summary: $OUTPUT_DIR/analysis-summary.json"

# Check quality gates
python3 << 'EOF'
import json
import sys

with open("analysis-results/analysis-summary.json", "r") as f:
    summary = json.load(f)

# Quality gates
critical_errors = summary["severity_breakdown"].get("CRITICAL", 0) + \
                 summary["severity_breakdown"].get("ERROR", 0)
quality_score = summary["quality_score"]

print(f"\n🚨 Quality Gate Results:")
if critical_errors > 0:
    print(f"  ❌ FAIL: Found {critical_errors} critical/error issues (threshold: 0)")
    sys.exit(1)
elif quality_score < 80:
    print(f"  ⚠️  WARN: Quality score {quality_score}% below threshold (80%)")
    sys.exit(2)  # Unstable
else:
    print(f"  ✅ PASS: All quality gates passed (Score: {quality_score}%)")
    sys.exit(0)
EOF

echo "🎉 All analysis completed successfully!"
```

### 6. Custom Plugin Orchestration

```rust
// examples/custom-orchestrator/src/main.rs
// Example of programmatic plugin orchestration

use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

struct PluginOrchestrator {
    plugins: Vec<PluginConfig>,
    output_dir: String,
    analysis_config: AnalysisConfig,
}

struct PluginConfig {
    name: String,
    priority: u32,
    dependencies: Vec<String>,
    config: HashMap<String, Value>,
}

struct AnalysisConfig {
    parallel_execution: bool,
    max_concurrent_plugins: usize,
    timeout_seconds: u64,
    continue_on_failure: bool,
}

impl PluginOrchestrator {
    pub fn new(output_dir: String) -> Self {
        Self {
            plugins: Vec::new(),
            output_dir,
            analysis_config: AnalysisConfig {
                parallel_execution: true,
                max_concurrent_plugins: 4,
                timeout_seconds: 300,
                continue_on_failure: true,
            },
        }
    }
    
    pub fn add_plugin(&mut self, config: PluginConfig) {
        self.plugins.push(config);
    }
    
    pub fn analyze_project(&self, project_path: &Path) -> Result<AnalysisResults, Box<dyn std::error::Error>> {
        println!("🚀 Starting orchestrated analysis of {}", project_path.display());
        
        // Sort plugins by priority and dependencies
        let execution_order = self.resolve_execution_order()?;
        
        let mut results = AnalysisResults::new();
        
        if self.analysis_config.parallel_execution {
            results = self.execute_parallel(&execution_order, project_path)?;
        } else {
            results = self.execute_sequential(&execution_order, project_path)?;
        }
        
        // Generate combined report
        self.generate_combined_report(&results)?;
        
        println!("✅ Analysis completed successfully!");
        Ok(results)
    }
    
    fn resolve_execution_order(&self) -> Result<Vec<&PluginConfig>, Box<dyn std::error::Error>> {
        // Simple topological sort for dependency resolution
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        fn visit(
            plugin: &PluginConfig,
            all_plugins: &[PluginConfig],
            visited: &mut std::collections::HashSet<String>,
            sorted: &mut Vec<String>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if visited.contains(&plugin.name) {
                return Ok(());
            }
            
            visited.insert(plugin.name.clone());
            
            // Visit dependencies first
            for dep_name in &plugin.dependencies {
                if let Some(dep_plugin) = all_plugins.iter().find(|p| p.name == *dep_name) {
                    visit(dep_plugin, all_plugins, visited, sorted)?;
                }
            }
            
            sorted.push(plugin.name.clone());
            Ok(())
        }
        
        let mut plugin_names = Vec::new();
        let mut visited_set = std::collections::HashSet::new();
        
        for plugin in &self.plugins {
            visit(plugin, &self.plugins, &mut visited_set, &mut plugin_names)?;
        }
        
        // Convert back to plugin references sorted by priority within dependency constraints
        let mut ordered_plugins: Vec<&PluginConfig> = plugin_names
            .iter()
            .filter_map(|name| self.plugins.iter().find(|p| p.name == *name))
            .collect();
            
        ordered_plugins.sort_by_key(|p| std::cmp::Reverse(p.priority));
        
        Ok(ordered_plugins)
    }
    
    fn execute_sequential(
        &self,
        plugins: &[&PluginConfig],
        project_path: &Path,
    ) -> Result<AnalysisResults, Box<dyn std::error::Error>> {
        let mut results = AnalysisResults::new();
        
        for plugin in plugins {
            println!("  📊 Running plugin: {}", plugin.name);
            
            match self.execute_plugin(plugin, project_path) {
                Ok(plugin_result) => {
                    results.plugin_results.insert(plugin.name.clone(), plugin_result);
                    println!("    ✅ Plugin {} completed", plugin.name);
                }
                Err(e) => {
                    println!("    ❌ Plugin {} failed: {}", plugin.name, e);
                    if !self.analysis_config.continue_on_failure {
                        return Err(e);
                    }
                    results.failed_plugins.push(plugin.name.clone());
                }
            }
        }
        
        Ok(results)
    }
    
    fn execute_parallel(
        &self,
        plugins: &[&PluginConfig],
        project_path: &Path,
    ) -> Result<AnalysisResults, Box<dyn std::error::Error>> {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let results = Arc::new(Mutex::new(AnalysisResults::new()));
        let mut handles = Vec::new();
        let project_path = project_path.to_path_buf();
        
        // Execute plugins in batches based on max_concurrent_plugins
        for batch in plugins.chunks(self.analysis_config.max_concurrent_plugins) {
            let mut batch_handles = Vec::new();
            
            for plugin in batch {
                let plugin_config = (*plugin).clone();
                let results_clone = Arc::clone(&results);
                let project_path_clone = project_path.clone();
                let timeout = self.analysis_config.timeout_seconds;
                let continue_on_failure = self.analysis_config.continue_on_failure;
                
                let handle = thread::spawn(move || {
                    println!("  📊 Running plugin: {}", plugin_config.name);
                    
                    let plugin_result = execute_plugin_with_timeout(
                        &plugin_config,
                        &project_path_clone,
                        timeout,
                    );
                    
                    let mut results_guard = results_clone.lock().unwrap();
                    match plugin_result {
                        Ok(result) => {
                            results_guard.plugin_results.insert(plugin_config.name.clone(), result);
                            println!("    ✅ Plugin {} completed", plugin_config.name);
                        }
                        Err(e) => {
                            println!("    ❌ Plugin {} failed: {}", plugin_config.name, e);
                            if !continue_on_failure {
                                panic!("Plugin {} failed: {}", plugin_config.name, e);
                            }
                            results_guard.failed_plugins.push(plugin_config.name);
                        }
                    }
                });
                
                batch_handles.push(handle);
            }
            
            // Wait for batch to complete
            for handle in batch_handles {
                if let Err(e) = handle.join() {
                    if !self.analysis_config.continue_on_failure {
                        return Err(format!("Thread panic: {:?}", e).into());
                    }
                }
            }
        }
        
        let final_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
        Ok(final_results)
    }
    
    fn execute_plugin(
        &self,
        plugin: &PluginConfig,
        project_path: &Path,
    ) -> Result<PluginResult, Box<dyn std::error::Error>> {
        let output_file = format!("{}/{}-report.json", self.output_dir, plugin.name);
        
        let mut cmd = Command::new("uveddi");
        cmd.args([
            "analyze",
            project_path.to_str().unwrap(),
            "--plugins",
            &plugin.name,
            "--output-format",
            "json",
            "--output",
            &output_file,
        ]);
        
        // Add plugin-specific configuration
        for (key, value) in &plugin.config {
            cmd.args(["--config", &format!("{}={}", key, value)]);
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(format!(
                "Plugin {} failed: {}",
                plugin.name,
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }
        
        // Parse results
        let result_content = std::fs::read_to_string(&output_file)?;
        let plugin_result: PluginResult = serde_json::from_str(&result_content)?;
        
        Ok(plugin_result)
    }
    
    fn generate_combined_report(&self, results: &AnalysisResults) -> Result<(), Box<dyn std::error::Error>> {
        let combined_report = CombinedReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            plugin_count: results.plugin_results.len(),
            total_issues: results.plugin_results.values()
                .map(|r| r.issues.len())
                .sum(),
            failed_plugins: results.failed_plugins.clone(),
            plugin_results: results.plugin_results.clone(),
            summary: self.generate_summary(results),
        };
        
        // Save JSON report
        let json_output = format!("{}/combined-report.json", self.output_dir);
        std::fs::write(&json_output, serde_json::to_string_pretty(&combined_report)?)?;
        
        // Generate HTML report using Uveddi
        let mut cmd = Command::new("uveddi");
        cmd.args([
            "ui",
            "export",
            &json_output,
            &format!("{}/combined-report.html", self.output_dir),
        ]);
        
        cmd.output()?;
        
        println!("📊 Combined report generated:");
        println!("  JSON: {}", json_output);
        println!("  HTML: {}/combined-report.html", self.output_dir);
        
        Ok(())
    }
    
    fn generate_summary(&self, results: &AnalysisResults) -> AnalysisSummary {
        let mut severity_counts = HashMap::new();
        let mut category_counts = HashMap::new();
        
        for plugin_result in results.plugin_results.values() {
            for issue in &plugin_result.issues {
                *severity_counts.entry(issue.severity.clone()).or_insert(0) += 1;
                *category_counts.entry(issue.issue_type.clone()).or_insert(0) += 1;
            }
        }
        
        AnalysisSummary {
            total_plugins_run: results.plugin_results.len(),
            total_plugins_failed: results.failed_plugins.len(),
            severity_breakdown: severity_counts,
            category_breakdown: category_counts,
            quality_score: calculate_quality_score(&results),
        }
    }
}

// Supporting structures
#[derive(Clone)]
struct PluginConfig {
    name: String,
    priority: u32,
    dependencies: Vec<String>,
    config: HashMap<String, Value>,
}

struct AnalysisResults {
    plugin_results: HashMap<String, PluginResult>,
    failed_plugins: Vec<String>,
}

impl AnalysisResults {
    fn new() -> Self {
        Self {
            plugin_results: HashMap::new(),
            failed_plugins: Vec::new(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct PluginResult {
    plugin_name: String,
    issues: Vec<Issue>,
    metadata: HashMap<String, String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct Issue {
    issue_type: String,
    severity: String,
    message: String,
    file_path: String,
    line_number: Option<u32>,
}

#[derive(serde::Serialize)]
struct CombinedReport {
    timestamp: String,
    plugin_count: usize,
    total_issues: usize,
    failed_plugins: Vec<String>,
    plugin_results: HashMap<String, PluginResult>,
    summary: AnalysisSummary,
}

#[derive(serde::Serialize)]
struct AnalysisSummary {
    total_plugins_run: usize,
    total_plugins_failed: usize,
    severity_breakdown: HashMap<String, i32>,
    category_breakdown: HashMap<String, i32>,
    quality_score: f64,
}

fn execute_plugin_with_timeout(
    plugin: &PluginConfig,
    project_path: &Path,
    timeout_seconds: u64,
) -> Result<PluginResult, Box<dyn std::error::Error>> {
    // Implementation would include timeout handling
    // Simplified for example
    Ok(PluginResult {
        plugin_name: plugin.name.clone(),
        issues: Vec::new(),
        metadata: HashMap::new(),
    })
}

fn calculate_quality_score(results: &AnalysisResults) -> f64 {
    let total_issues = results.plugin_results.values()
        .map(|r| r.issues.len() as f64)
        .sum::<f64>();
    
    let error_weight = results.plugin_results.values()
        .flat_map(|r| &r.issues)
        .map(|issue| match issue.severity.as_str() {
            "ERROR" | "CRITICAL" => 10.0,
            "WARNING" => 2.0,
            _ => 0.5,
        })
        .sum::<f64>();
    
    (100.0 - error_weight).max(0.0)
}

// Example usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut orchestrator = PluginOrchestrator::new("analysis-output".to_string());
    
    // Configure plugins
    orchestrator.add_plugin(PluginConfig {
        name: "security-scanner".to_string(),
        priority: 100,
        dependencies: vec![],
        config: [
            ("severity_threshold".to_string(), Value::String("WARNING".to_string())),
        ].into(),
    });
    
    orchestrator.add_plugin(PluginConfig {
        name: "code-quality-checker".to_string(),
        priority: 90,
        dependencies: vec![],
        config: HashMap::new(),
    });
    
    orchestrator.add_plugin(PluginConfig {
        name: "performance-analyzer".to_string(),
        priority: 80,
        dependencies: vec!["code-quality-checker".to_string()],
        config: HashMap::new(),
    });
    
    // Run analysis
    let results = orchestrator.analyze_project(Path::new("./src"))?;
    
    println!("Analysis completed with {} plugins", results.plugin_results.len());
    if !results.failed_plugins.is_empty() {
        println!("Failed plugins: {:?}", results.failed_plugins);
    }
    
    Ok(())
}
```

These examples demonstrate various integration patterns for Uveddi's WASM plugin system, from basic single-plugin usage to sophisticated multi-plugin orchestration and CI/CD integration. The examples show how to leverage the plugin system's capabilities for comprehensive code analysis workflows.