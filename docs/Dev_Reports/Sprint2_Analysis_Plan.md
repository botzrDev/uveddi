# Sprint 2 Analysis & Implementation Plan

## Overview

Based on the project documentation and ERD analysis, Sprint 2 represents a critical architectural expansion phase. This sprint transforms Uveddi from a simple cyclic dependency detector into a multi-language AST-powered analysis engine with AI integration.

## Key Observations

### 1. **Architectural Alignment Opportunities**

The current Sprint 2 tasks align well with the ERD specifications but need refinement to support the long-term architecture:

- **AST Integration** maps directly to ERD's AST Parsing Module requirements
- **Multi-language support** fulfills ER-F-002 (Rust, Python, JavaScript)
- **AI Integration** needs to be designed with RAG foundations from the start

### 2. **Missing Database Integration**

Sprint 2 currently lacks database integration, but the ERD defines critical models (`AnalysisRun`, `ArchitecturalIssue`, `AntiPatternType`) that should be implemented early for proper data persistence and analysis tracking.

### 3. **AI Integration Complexity**

The sprint underestimates AI integration complexity. Based on the research documents, we need structured prompting and contextual analysis, not just simple API calls.

## Refined Sprint 2 Implementation Plan

### Phase 1: AST Foundation with Database Integration (Days 1-4)

#### **Day 1-2: Database Models & AST Parser Core**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core analysis run tracking - aligns with ERD AnalysisRun entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub run_id: Option<i64>,
    pub project_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String, // "running", "completed", "failed"
    pub total_files_analyzed: Option<i32>,
    pub total_issues_found: Option<i32>,
    pub analysis_config: String, // JSON serialized config
}

/// Architectural issues with severity and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalIssue {
    pub issue_id: Option<i64>,
    pub analysis_run_id: i64,
    pub anti_pattern_type_id: i64,
    pub file_path: String,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub severity: String, // "low", "medium", "high", "critical"
    pub description: String,
    pub code_snippet: Option<String>,
    pub ai_explanation: Option<String>,
}

/// Anti-pattern type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternType {
    pub type_id: Option<i64>,
    pub name: String,
    pub description: String,
    pub category: String, // "structural", "behavioral", "creational"
}
```

```rust
use tree_sitter::{Language, Parser, Tree};
use std::path::Path;

/// Multi-language AST parser following ERD specifications
pub struct AstParser {
    rust_parser: Parser,
    python_parser: Parser,
    javascript_parser: Parser,
}

impl AstParser {
    /// Initialize parsers for supported languages (ER-F-002)
    pub fn new() -> Result<Self, AstError> {
        let mut rust_parser = Parser::new();
        rust_parser.set_language(tree_sitter_rust::language())?;
        
        let mut python_parser = Parser::new();
        python_parser.set_language(tree_sitter_python::language())?;
        
        let mut javascript_parser = Parser::new();
        javascript_parser.set_language(tree_sitter_javascript::language())?;
        
        Ok(AstParser {
            rust_parser,
            python_parser,
            javascript_parser,
        })
    }

    /// Parse file and extract dependencies (ER-F-003)
    pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        let source = std::fs::read_to_string(file_path)?;
        let language = self.detect_language(file_path)?;
        
        let parser = match language {
            Language::Rust => &mut self.rust_parser,
            Language::Python => &mut self.python_parser,
            Language::JavaScript => &mut self.javascript_parser,
        };
        
        let tree = parser.parse(&source, None)
            .ok_or(AstError::ParseFailed)?;
            
        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree,
            source,
        })
    }
}

#[derive(Debug)]
pub struct ParsedFile {
    pub path: std::path::PathBuf,
    pub language: Language,
    pub tree: Tree,
    pub source: String,
}

#[derive(Debug, Clone)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
}
```

#### **Day 3-4: Enhanced Analysis Engine**

```rust
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::ast::ParsedFile;

/// Core analysis trait for all detectors
pub trait AnalysisDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    fn get_detector_name(&self) -> &'static str;
}

/// God Object detector using AST metrics
pub struct GodObjectDetector {
    max_methods: usize,
    max_lines: usize,
    max_dependencies: usize,
}

impl GodObjectDetector {
    pub fn new() -> Self {
        Self {
            max_methods: 20,      // Configurable thresholds
            max_lines: 500,
            max_dependencies: 15,
        }
    }
}

impl AnalysisDetector for GodObjectDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        
        // Use tree-sitter queries to analyze class/struct complexity
        let classes = self.extract_classes(file)?;
        
        for class in classes {
            let metrics = self.calculate_metrics(&class, file)?;
            
            if self.is_god_object(&metrics) {
                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0, // Will be set by analysis engine
                    anti_pattern_type_id: 1, // God Object type ID
                    file_path: file.path.to_string_lossy().to_string(),
                    start_line: Some(class.start_line),
                    end_line: Some(class.end_line),
                    severity: self.calculate_severity(&metrics),
                    description: format!(
                        "God Object detected: {} methods, {} lines, {} dependencies",
                        metrics.method_count,
                        metrics.line_count,
                        metrics.dependency_count
                    ),
                    code_snippet: Some(self.extract_code_snippet(&class, file)),
                    ai_explanation: None, // Will be populated by AI engine
                });
            }
        }
        
        Ok(issues)
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that knows too much or does too much".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "god_object"
    }
}
```

### Phase 2: AI Integration with RAG Foundation (Days 5-8)

```rust
use crate::database::models::ArchitecturalIssue;
use serde_json::Value;

/// AI analysis engine with provider abstraction
pub struct AiAnalysisEngine {
    api_provider: Option<Box<dyn LlmProvider>>,
    local_provider: Option<Box<dyn LlmProvider>>,
    context_builder: ContextBuilder,
}

impl AiAnalysisEngine {
    pub fn new() -> Self {
        Self {
            api_provider: None,
            local_provider: None,
            context_builder: ContextBuilder::new(),
        }
    }

    /// Configure OpenAI API provider
    pub fn with_openai_api(mut self, api_key: String) -> Self {
        self.api_provider = Some(Box::new(OpenAiProvider::new(api_key)));
        self
    }

    /// Analyze issue with contextual information (ER-F-008: Smart Prompting)
    pub async fn analyze_issue(&self, 
        issue: &mut ArchitecturalIssue, 
        file_context: &ParsedFile
    ) -> Result<(), AiError> {
        // Build rich context for the AI prompt
        let context = self.context_builder
            .add_issue_context(issue)
            .add_code_context(file_context)
            .add_project_context() // For future RAG implementation
            .build();

        let prompt = self.build_structured_prompt(issue, &context);
        
        // Try API provider first, then local, then skip
        if let Some(provider) = &self.api_provider {
            if let Ok(explanation) = provider.generate_explanation(&prompt).await {
                issue.ai_explanation = Some(explanation);
                return Ok(());
            }
        }
        
        if let Some(provider) = &self.local_provider {
            if let Ok(explanation) = provider.generate_explanation(&prompt).await {
                issue.ai_explanation = Some(explanation);
                return Ok(());
            }
        }
        
        // Continue without AI explanation
        log::warn!("No AI provider available for issue analysis");
        Ok(())
    }

    /// Build structured prompt following research guidelines
    fn build_structured_prompt(&self, issue: &ArchitecturalIssue, context: &IssueContext) -> String {
        format!(
            r#"You are a software architecture expert. Analyze this code issue:

**Issue Type:** {}
**File:** {}
**Lines:** {}-{}
**Severity:** {}

**Code Context:**
```{}
{}
```

**Description:** {}

Please provide:
1. A clear explanation of why this is problematic
2. Specific impact on code maintainability
3. Concrete refactoring suggestions
4. Example of improved code structure

Keep the response concise and actionable."#,
            "God Object", // Would be dynamic based on anti-pattern type
            issue.file_path,
            issue.start_line.unwrap_or(0),
            issue.end_line.unwrap_or(0),
            issue.severity,
            context.language,
            context.code_snippet,
            issue.description
        )
    }
}

/// Contextual information for AI analysis
#[derive(Debug)]
pub struct IssueContext {
    pub language: String,
    pub code_snippet: String,
    pub surrounding_context: String,
    pub project_patterns: Vec<String>, // For future RAG implementation
}

/// Context builder for rich AI prompts
pub struct ContextBuilder {
    context: IssueContext,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: IssueContext {
                language: String::new(),
                code_snippet: String::new(),
                surrounding_context: String::new(),
                project_patterns: Vec::new(),
            }
        }
    }

    pub fn add_issue_context(mut self, issue: &ArchitecturalIssue) -> Self {
        if let Some(snippet) = &issue.code_snippet {
            self.context.code_snippet = snippet.clone();
        }
        self
    }

    pub fn add_code_context(mut self, file: &ParsedFile) -> Self {
        self.context.language = format!("{:?}", file.language);
        // Add surrounding context extraction logic
        self
    }

    pub fn add_project_context(self) -> Self {
        // Future: Add project-wide patterns and conventions
        self
    }

    pub fn build(self) -> IssueContext {
        self.context
    }
}
```

### Phase 3: Enhanced Reporting (Days 9-12)

```rust
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use serde_json::Value;

/// Report generator following ERD specifications (ER-F-011 to ER-F-014)
pub struct ReportGenerator {
    include_ai_explanations: bool,
    include_code_snippets: bool,
    include_diagrams: bool,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {
            include_ai_explanations: true,
            include_code_snippets: true,
            include_diagrams: false, // Future sprint feature
        }
    }

    /// Generate comprehensive markdown report (ER-F-014)
    pub fn generate_markdown_report(&self, 
        analysis_run: &AnalysisRun, 
        issues: &[ArchitecturalIssue]
    ) -> Result<String, ReportError> {
        let mut report = String::new();
        
        // Report header with analysis summary
        report.push_str(&self.generate_header(analysis_run, issues));
        
        // Executive summary
        report.push_str(&self.generate_summary(issues));
        
        // Issues by severity
        report.push_str(&self.generate_issues_by_severity(issues));
        
        // Detailed issue analysis
        report.push_str(&self.generate_detailed_issues(issues));
        
        Ok(report)
    }

    /// Generate structured JSON output
    pub fn generate_json_report(&self, 
        analysis_run: &AnalysisRun, 
        issues: &[ArchitecturalIssue]
    ) -> Result<Value, ReportError> {
        let report = serde_json::json!({
            "analysis_run": {
                "run_id": analysis_run.run_id,
                "start_time": analysis_run.start_time,
                "end_time": analysis_run.end_time,
                "status": analysis_run.status,
                "total_files_analyzed": analysis_run.total_files_analyzed,
                "total_issues_found": analysis_run.total_issues_found
            },
            "summary": {
                "total_issues": issues.len(),
                "by_severity": self.calculate_severity_breakdown(issues),
                "by_category": self.calculate_category_breakdown(issues)
            },
            "issues": issues
        });
        
        Ok(report)
    }

    fn generate_header(&self, analysis_run: &AnalysisRun, issues: &[ArchitecturalIssue]) -> String {
        format!(
            r#"# Uveddi Analysis Report

**Analysis ID:** {}
**Start Time:** {}
**Duration:** {}
**Files Analyzed:** {}
**Issues Found:** {}

---

"#,
            analysis_run.run_id.unwrap_or(0),
            analysis_run.start_time.format("%Y-%m-%d %H:%M:%S UTC"),
            self.calculate_duration(analysis_run),
            analysis_run.total_files_analyzed.unwrap_or(0),
            issues.len()
        )
    }

    fn generate_detailed_issues(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut content = String::from("## Detailed Issue Analysis\n\n");
        
        for (index, issue) in issues.iter().enumerate() {
            content.push_str(&format!(
                r#"### Issue #{}: {}

**File:** `{}`
**Lines:** {}-{}
**Severity:** {}

**Description:**
{}

"#,
                index + 1,
                self.get_anti_pattern_name(issue.anti_pattern_type_id),
                issue.file_path,
                issue.start_line.unwrap_or(0),
                issue.end_line.unwrap_or(0),
                issue.severity.to_uppercase(),
                issue.description
            ));

            // Add code snippet if available
            if self.include_code_snippets {
                if let Some(snippet) = &issue.code_snippet {
                    content.push_str(&format!(
                        r#"**Code Context:**
```rust
{}
```

"#,
                        snippet
                    ));
                }
            }

            // Add AI explanation if available
            if self.include_ai_explanations {
                if let Some(explanation) = &issue.ai_explanation {
                    content.push_str(&format!(
                        r#"**AI Analysis:**
{}

"#,
                        explanation
                    ));
                }
            }

            content.push_str("---\n\n");
        }
        
        content
    }
}
```

### Phase 4: Integration & Testing (Days 13-14)

```rust
use crate::analysis::AnalysisEngine;
use crate::ai::AiAnalysisEngine;
use crate::database::Database;
use crate::reporting::ReportGenerator;
use clap::Args;

#[derive(Args)]
pub struct AnalyzeCommand {
    /// Path to analyze
    pub path: std::path::PathBuf,
    
    /// Output format (text, json, markdown)
    #[arg(long, default_value = "markdown")]
    pub output_format: String,
    
    /// Output file path
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    
    /// Enable AI analysis (requires API key or local model)
    #[arg(long)]
    pub enable_ai: bool,
    
    /// OpenAI API key
    #[arg(long, env = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,
}

impl AnalyzeCommand {
    pub async fn execute(&self) -> Result<(), AnalysisError> {
        // Initialize components
        let database = Database::new()?;
        let mut analysis_engine = AnalysisEngine::new();
        let mut ai_engine = AiAnalysisEngine::new();
        
        // Configure AI if enabled
        if self.enable_ai {
            if let Some(api_key) = &self.openai_api_key {
                ai_engine = ai_engine.with_openai_api(api_key.clone());
            }
        }
        
        // Create analysis run
        let analysis_run = database.create_analysis_run(&self.path)?;
        
        // Run analysis
        let mut issues = analysis_engine.analyze_directory(&self.path).await?;
        
        // Enhance with AI analysis
        if self.enable_ai {
            for issue in &mut issues {
                // This would need file context - simplified for example
                ai_engine.analyze_issue(issue, &parsed_file).await?;
            }
        }
        
        // Store results
        database.store_issues(&issues)?;
        
        // Generate report
        let report_generator = ReportGenerator::new();
        let report = match self.output_format.as_str() {
            "json" => report_generator.generate_json_report(&analysis_run, &issues)?.to_string(),
            "markdown" => report_generator.generate_markdown_report(&analysis_run, &issues)?,
            _ => return Err(AnalysisError::UnsupportedOutputFormat(self.output_format.clone())),
        };
        
        // Output report
        if let Some(output_path) = &self.output {
            std::fs::write(output_path, report)?;
            println!("Report written to: {}", output_path.display());
        } else {
            println!("{}", report);
        }
        
        Ok(())
    }
}
```

## Key Dependencies for `Cargo.toml`

```toml
[dependencies]
# Existing dependencies
clap = { version = "4.0", features = ["derive"] }
env_logger = "0.10"
log = "0.4"

# Sprint 2 additions
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-python = "0.20"
tree-sitter-javascript = "0.20"

# AI Integration
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database (SQLite for now)
rusqlite = { version = "0.29", features = ["chrono"] }
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.0"
tokio-test = "0.4"
```

## Success Criteria & Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_god_object_detection() {
        // Create test file with known god object
        let test_code = r#"
        struct GodObject {
            // Many fields...
        }
        
        impl GodObject {
            // Many methods...
        }
        "#;
        
        // Test detection logic
    }

    #[tokio::test]
    async fn test_ai_integration() {
        // Test AI analysis with mock provider
    }

    #[test]
    fn test_multi_language_parsing() {
        // Test Rust, Python, JavaScript parsing
    }
}
```

## Implementation Recommendations

### 1. **Prioritize Database Integration Early**

The database models should be implemented in Days 1-2 to ensure proper tracking from the start:

```rust
// Add this to Sprint 2 tasks
pub fn create_analysis_run(&self, project_path: &Path) -> Result<AnalysisRun, Error> {
    let analysis_run = AnalysisRun {
        run_id: None,
        project_id: self.get_or_create_project_id(project_path)?,
        start_time: SystemTime::now(),
        status: "running".to_string(),
        // ... other fields from the model
    };
    self.database.store_analysis_run(analysis_run)
}
```

### 2. **Structure for Plugin System Early**

Even though plugins are Sprint 3, the analysis architecture should support it:

```rust
// Design the analysis engine to support the plugin system from ERD
pub trait AnalysisPlugin {
    fn detect_issues(&self, ast: &Ast, context: &AnalysisContext) -> Vec<ArchitecturalIssue>;
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
}

// Core detector as a plugin
pub struct CoreDetectorPlugin;

impl AnalysisPlugin for CoreDetectorPlugin {
    fn detect_issues(&self, ast: &Ast, context: &AnalysisContext) -> Vec<ArchitecturalIssue> {
        // God Object detection, etc.
    }
}
```

### 3. **Follow ERD Report Generation Specifications**

The markdown report generation should match ERD requirements (ER-F-011 through ER-F-014):

```rust
// ER-F-012: Include Mermaid.js syntax for diagrams
pub struct ReportGenerator {
    // Prepare for the advanced diagram generation from Ai_Diagrams.md
}

impl ReportGenerator {
    pub fn generate_markdown_report(&self, analysis_run: &AnalysisRun, issues: &[ArchitecturalIssue]) -> String {
        // Include all required elements from ER-F-014
        // - Anti-pattern name
        // - File path and line numbers  
        // - Severity
        // - AI-generated explanations
        // - Code snippets
        // - Optional Mermaid.js diagrams
    }
}
```

## Modified Sprint 2 Task Recommendations

1. **Days 1-3**: AST integration with database model alignment
2. **Days 4-6**: Enhanced analysis with proper issue modeling
3. **Days 7-9**: AI integration with structured prompting foundation
4. **Days 10-12**: Report generation following ERD specifications
5. **Days 13-14**: Integration testing and plugin-ready architecture

## Sprint 2 Definition of Done

- [ ] **Multi-language AST parsing** for Rust, Python, JavaScript
- [ ] **Database integration** with proper models and persistence
- [ ] **God Object detection** using AST metrics
- [ ] **AI explanation generation** with structured prompting
- [ ] **Enhanced reporting** in JSON and Markdown formats
- [ ] **Graceful AI fallback** when API keys unavailable
- [ ] **Comprehensive test coverage** for all new components
- [ ] **Performance benchmarks** for AST parsing pipeline

This refined approach ensures Sprint 2 builds a solid architectural foundation while delivering immediate value through enhanced analysis capabilities and AI integration that aligns with the sophisticated architecture described in the research documents.
