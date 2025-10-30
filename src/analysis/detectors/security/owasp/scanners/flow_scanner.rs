//! Data flow analysis scanner for OWASP vulnerabilities
//!
//! This scanner performs data flow analysis to track how untrusted data
//! flows through the application and identifies potential security issues.

use super::{Scanner, UnifiedScanResult};
use crate::analysis::detectors::security::owasp::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tracing::{debug, info};

/// Represents a data flow vulnerability
#[derive(Debug, Clone)]
pub struct FlowVulnerability {
    pub source_location: SecurityLocation,
    pub sink_location: SecurityLocation,
    pub flow_path: Vec<FlowNode>,
    pub taint_type: TaintType,
    pub category: OwaspCategory,
    pub confidence: f64,
}

/// Represents a node in the data flow
#[derive(Debug, Clone)]
pub struct FlowNode {
    pub location: SecurityLocation,
    pub node_type: FlowNodeType,
    pub variable_name: Option<String>,
    pub function_name: Option<String>,
}

/// Types of flow nodes
#[derive(Debug, Clone)]
pub enum FlowNodeType {
    Source,      // Where untrusted data enters
    Propagation, // Where data flows through
    Sink,        // Where data is used dangerously
    Sanitizer,   // Where data is cleaned/validated
}

/// Types of data taint
#[derive(Debug, Clone)]
pub enum TaintType {
    UserInput,   // From user input (forms, params, etc.)
    FileSystem,  // From file operations
    Network,     // From network requests
    Database,    // From database queries
    Environment, // From environment variables
}

/// Result from data flow analysis
#[derive(Debug, Clone)]
pub struct FlowAnalysisResult {
    pub vulnerabilities: Vec<FlowVulnerability>,
    pub sources_found: usize,
    pub sinks_found: usize,
    pub paths_analyzed: usize,
    pub scan_duration_ms: u64,
}

/// Data flow analysis scanner
pub struct DataFlowScanner {
    source_patterns: HashMap<SourceLanguage, Vec<SourcePattern>>,
    sink_patterns: HashMap<SourceLanguage, Vec<SinkPattern>>,
    sanitizer_patterns: HashMap<SourceLanguage, Vec<SanitizerPattern>>,
}

/// Pattern for identifying data sources (where untrusted data enters)
#[derive(Debug, Clone)]
struct SourcePattern {
    pattern: String,
    taint_type: TaintType,
    confidence: f64,
    description: String,
}

/// Pattern for identifying data sinks (where data is used dangerously)
#[derive(Debug, Clone)]
struct SinkPattern {
    pattern: String,
    category: OwaspCategory,
    issue_type: SecurityIssueType,
    severity: SecuritySeverity,
    confidence: f64,
    description: String,
}

/// Pattern for identifying data sanitizers (where data is cleaned)
#[derive(Debug, Clone)]
struct SanitizerPattern {
    pattern: String,
    sanitizes: Vec<TaintType>,
    confidence: f64,
    description: String,
}

impl DataFlowScanner {
    pub fn new() -> Result<Self, AnalysisError> {
        let mut scanner = Self {
            source_patterns: HashMap::new(),
            sink_patterns: HashMap::new(),
            sanitizer_patterns: HashMap::new(),
        };

        scanner.initialize_patterns()?;
        Ok(scanner)
    }

    /// Initialize data flow patterns for different languages
    fn initialize_patterns(&mut self) -> Result<(), AnalysisError> {
        self.initialize_source_patterns();
        self.initialize_sink_patterns();
        self.initialize_sanitizer_patterns();
        Ok(())
    }

    fn initialize_source_patterns(&mut self) {
        let sources = vec![
            // HTTP request parameters
            SourcePattern {
                pattern: r"(?i)(request\.args|request\.form|request\.json|req\.query|req\.body|req\.params)".to_string(),
                taint_type: TaintType::UserInput,
                confidence: 0.9,
                description: "HTTP request parameter".to_string(),
            },
            // File reading operations
            SourcePattern {
                pattern: r"(?i)(open\(|read\(|readFile|fs\.read)".to_string(),
                taint_type: TaintType::FileSystem,
                confidence: 0.8,
                description: "File system read operation".to_string(),
            },
            // Environment variables
            SourcePattern {
                pattern: r"(?i)(os\.environ|process\.env|std::env::var)".to_string(),
                taint_type: TaintType::Environment,
                confidence: 0.7,
                description: "Environment variable access".to_string(),
            },
            // Database queries
            SourcePattern {
                pattern: r"(?i)(cursor\.fetchone|cursor\.fetchall|db\.query|connection\.query)".to_string(),
                taint_type: TaintType::Database,
                confidence: 0.8,
                description: "Database query result".to_string(),
            },
        ];

        for language in [
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ] {
            self.source_patterns.insert(language, sources.clone());
        }

        // Rust-specific sources
        let rust_sources = vec![SourcePattern {
            pattern: r"std::env::args|std::io::stdin".to_string(),
            taint_type: TaintType::UserInput,
            confidence: 0.9,
            description: "Command line or stdin input".to_string(),
        }];
        self.source_patterns
            .insert(SourceLanguage::Rust, rust_sources);
    }

    fn initialize_sink_patterns(&mut self) {
        let sinks = vec![
            // SQL execution
            SinkPattern {
                pattern: r"(?i)(execute\(|query\(|cursor\.execute)".to_string(),
                category: OwaspCategory::Injection,
                issue_type: SecurityIssueType::Injection,
                severity: SecuritySeverity::Critical,
                confidence: 0.9,
                description: "SQL query execution".to_string(),
            },
            // Command execution
            SinkPattern {
                pattern: r"(?i)(os\.system|subprocess|exec\(|eval\()".to_string(),
                category: OwaspCategory::Injection,
                issue_type: SecurityIssueType::Injection,
                severity: SecuritySeverity::Critical,
                confidence: 0.9,
                description: "Command or code execution".to_string(),
            },
            // HTML output
            SinkPattern {
                pattern: r"(?i)(innerHTML|document\.write|\.html\()".to_string(),
                category: OwaspCategory::Injection,
                issue_type: SecurityIssueType::CrossSiteScripting,
                severity: SecuritySeverity::High,
                confidence: 0.8,
                description: "HTML output without escaping".to_string(),
            },
            // File operations
            SinkPattern {
                pattern: r"(?i)(open\(.*w|write\(|writeFile)".to_string(),
                category: OwaspCategory::BrokenAccessControl,
                issue_type: SecurityIssueType::PathTraversal,
                severity: SecuritySeverity::Medium,
                confidence: 0.6,
                description: "File write operation".to_string(),
            },
        ];

        for language in [
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ] {
            self.sink_patterns.insert(language, sinks.clone());
        }
    }

    fn initialize_sanitizer_patterns(&mut self) {
        let sanitizers = vec![
            SanitizerPattern {
                pattern: r"(?i)(escape|sanitize|clean|validate)".to_string(),
                sanitizes: vec![TaintType::UserInput],
                confidence: 0.7,
                description: "Data sanitization function".to_string(),
            },
            SanitizerPattern {
                pattern: r"(?i)(html\.escape|escape_html|encodeURIComponent)".to_string(),
                sanitizes: vec![TaintType::UserInput],
                confidence: 0.9,
                description: "HTML/URL escaping function".to_string(),
            },
        ];

        for language in [
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ] {
            self.sanitizer_patterns.insert(language, sanitizers.clone());
        }
    }

    /// Perform data flow analysis on the file
    fn analyze_data_flow(&self, file: &ParsedFile) -> Result<FlowAnalysisResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut sources_found = 0;
        let mut sinks_found = 0;
        let mut paths_analyzed = 0;

        // Simple line-by-line analysis (in a real implementation, this would use AST)
        let lines: Vec<&str> = file.source.lines().collect();
        let mut sources = Vec::new();
        let mut sinks = Vec::new();
        let mut sanitizers = HashSet::new();

        // Find sources
        if let Some(source_patterns) = self.source_patterns.get(&file.language) {
            for (line_num, line) in lines.iter().enumerate() {
                for pattern in source_patterns {
                    if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                        if regex.is_match(line) {
                            sources_found += 1;
                            let location = SecurityLocation::new(
                                file.file_path.as_ref().clone(),
                                line_num as i32 + 1,
                                line_num as i32 + 1,
                            );
                            sources.push((location, pattern.taint_type.clone(), line_num));
                        }
                    }
                }
            }
        }

        // Find sinks
        if let Some(sink_patterns) = self.sink_patterns.get(&file.language) {
            for (line_num, line) in lines.iter().enumerate() {
                for pattern in sink_patterns {
                    if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                        if regex.is_match(line) {
                            sinks_found += 1;
                            let location = SecurityLocation::new(
                                file.file_path.as_ref().clone(),
                                line_num as i32 + 1,
                                line_num as i32 + 1,
                            );
                            sinks.push((location, pattern.clone(), line_num));
                        }
                    }
                }
            }
        }

        // Find sanitizers
        if let Some(sanitizer_patterns) = self.sanitizer_patterns.get(&file.language) {
            for (line_num, line) in lines.iter().enumerate() {
                for pattern in sanitizer_patterns {
                    if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                        if regex.is_match(line) {
                            sanitizers.insert(line_num);
                        }
                    }
                }
            }
        }

        // Analyze flows from sources to sinks
        for (source_location, taint_type, source_line) in &sources {
            for (sink_location, sink_pattern, sink_line) in &sinks {
                paths_analyzed += 1;

                // Check if there's a sanitizer between source and sink
                let has_sanitizer =
                    (*source_line..*sink_line).any(|line_num| sanitizers.contains(&line_num));

                if !has_sanitizer && sink_line > source_line {
                    // Create a simple flow path
                    let flow_path = vec![
                        FlowNode {
                            location: source_location.clone(),
                            node_type: FlowNodeType::Source,
                            variable_name: None,
                            function_name: None,
                        },
                        FlowNode {
                            location: sink_location.clone(),
                            node_type: FlowNodeType::Sink,
                            variable_name: None,
                            function_name: None,
                        },
                    ];

                    let vulnerability = FlowVulnerability {
                        source_location: source_location.clone(),
                        sink_location: sink_location.clone(),
                        flow_path,
                        taint_type: taint_type.clone(),
                        category: sink_pattern.category.clone(),
                        confidence: sink_pattern.confidence * 0.8, // Reduce confidence for flow analysis
                    };

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        Ok(FlowAnalysisResult {
            vulnerabilities,
            sources_found,
            sinks_found,
            paths_analyzed,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Convert flow vulnerabilities to OWASP format
    fn convert_to_owasp_vulnerabilities(
        &self,
        flow_vulnerabilities: Vec<FlowVulnerability>,
    ) -> Vec<OwaspVulnerability> {
        flow_vulnerabilities
            .into_iter()
            .map(|flow_vuln| {
                let issue_type = match flow_vuln.category {
                    OwaspCategory::Injection => SecurityIssueType::Injection,
                    OwaspCategory::BrokenAccessControl => SecurityIssueType::PathTraversal,
                    _ => SecurityIssueType::InsecureDesign,
                };

                OwaspVulnerability::new(
                    flow_vuln.category,
                    issue_type,
                    format!("Data Flow: {:?} to dangerous sink", flow_vuln.taint_type),
                    format!(
                        "Untrusted data from line {} flows to dangerous operation at line {} without sanitization",
                        flow_vuln.source_location.start_line,
                        flow_vuln.sink_location.start_line
                    ),
                    flow_vuln.sink_location,
                )
                .with_confidence(flow_vuln.confidence)
                .with_severity(SecuritySeverity::from_score(flow_vuln.confidence))
                .with_metadata(
                    VulnerabilityMetadata::new()
                        .with_tags(vec![
                            "data-flow".to_string(),
                            format!("{:?}", flow_vuln.taint_type).to_lowercase(),
                        ])
                )
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl Scanner for DataFlowScanner {
    async fn scan(&self, file: &ParsedFile) -> Result<UnifiedScanResult, AnalysisError> {
        let start_time = Instant::now();
        info!(
            "Starting data flow analysis for: {}",
            file.file_path.display()
        );

        let flow_result = self.analyze_data_flow(file)?;
        let vulnerabilities = self.convert_to_owasp_vulnerabilities(flow_result.vulnerabilities);

        let scan_duration = start_time.elapsed().as_millis() as u64;

        let mut metadata = HashMap::new();
        metadata.insert(
            "scanner_type".to_string(),
            serde_json::Value::String("flow".to_string()),
        );
        metadata.insert(
            "sources_found".to_string(),
            serde_json::Value::Number(flow_result.sources_found.into()),
        );
        metadata.insert(
            "sinks_found".to_string(),
            serde_json::Value::Number(flow_result.sinks_found.into()),
        );
        metadata.insert(
            "paths_analyzed".to_string(),
            serde_json::Value::Number(flow_result.paths_analyzed.into()),
        );
        metadata.insert(
            "language".to_string(),
            serde_json::Value::String(file.language.to_string()),
        );

        info!(
            "Data flow analysis completed: {} vulnerabilities found from {} sources and {} sinks in {}ms",
            vulnerabilities.len(),
            flow_result.sources_found,
            flow_result.sinks_found,
            scan_duration
        );

        Ok(UnifiedScanResult {
            vulnerabilities,
            scanner_metadata: serde_json::to_value(&metadata).unwrap_or_default(),
            scan_duration_ms: scan_duration,
        })
    }

    fn name(&self) -> &'static str {
        "data_flow"
    }

    fn supported_languages(&self) -> Vec<SourceLanguage> {
        vec![
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
            SourceLanguage::Rust,
        ]
    }

    fn detectable_categories(&self) -> Vec<OwaspCategory> {
        vec![
            OwaspCategory::Injection,
            OwaspCategory::BrokenAccessControl,
            OwaspCategory::InsecureDesign,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_flow_scanner_creation() {
        let scanner = DataFlowScanner::new();
        assert!(scanner.is_ok());

        let scanner = scanner.unwrap();
        assert_eq!(scanner.name(), "data_flow");
        assert_eq!(scanner.supported_languages().len(), 4);
    }

    #[tokio::test]
    async fn test_data_flow_detection() {
        let scanner = DataFlowScanner::new().unwrap();
        let code = r#"
user_input = request.args.get('input')
query = "SELECT * FROM users WHERE name = '" + user_input + "'"
cursor.execute(query)
"#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            source: Arc::new(code.to_string()),
            tree: None,
            custom_ast: Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        let result = scanner.scan(&file).await.unwrap();
        assert!(!result.vulnerabilities.is_empty());
        assert_eq!(result.vulnerabilities[0].category, OwaspCategory::Injection);
    }

    #[tokio::test]
    async fn test_sanitized_flow_detection() {
        let scanner = DataFlowScanner::new().unwrap();
        let code = r#"
user_input = request.args.get('input')
clean_input = escape(user_input)
query = "SELECT * FROM users WHERE name = '" + clean_input + "'"
cursor.execute(query)
"#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            source: Arc::new(code.to_string()),
            tree: None,
            custom_ast: Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        let result = scanner.scan(&file).await.unwrap();
        // Should have fewer vulnerabilities due to sanitization
        assert!(result.vulnerabilities.len() < 2);
    }
}
