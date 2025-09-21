//! Static analysis scanner for OWASP vulnerabilities
//!
//! This scanner performs static code analysis by examining the AST structure
//! to identify potential security vulnerabilities without executing the code.

use super::{Scanner, UnifiedScanResult};
use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};

/// Static analysis vulnerability finding
#[derive(Debug, Clone)]
pub struct StaticVulnerability {
    pub category: OwaspCategory,
    pub issue_type: SecurityIssueType,
    pub confidence: f64,
    pub location: SecurityLocation,
    pub context: String,
    pub ast_node_type: Option<String>,
}

/// Result from static analysis scan
#[derive(Debug, Clone)]
pub struct StaticScanResult {
    pub vulnerabilities: Vec<StaticVulnerability>,
    pub analysis_metadata: HashMap<String, serde_json::Value>,
    pub nodes_analyzed: usize,
    pub scan_duration_ms: u64,
}

/// Static analysis scanner implementation
pub struct StaticAnalysisScanner {
    language_analyzers: HashMap<SourceLanguage, Box<dyn LanguageAnalyzer>>,
}

impl StaticAnalysisScanner {
    pub fn new() -> Result<Self, AnalysisError> {
        let mut language_analyzers: HashMap<SourceLanguage, Box<dyn LanguageAnalyzer>> = HashMap::new();

        // Initialize language-specific analyzers
        language_analyzers.insert(SourceLanguage::Rust, Box::new(RustStaticAnalyzer::new()));
        language_analyzers.insert(SourceLanguage::Python, Box::new(PythonStaticAnalyzer::new()));
        language_analyzers.insert(SourceLanguage::JavaScript, Box::new(JavaScriptStaticAnalyzer::new()));
        language_analyzers.insert(SourceLanguage::TypeScript, Box::new(TypeScriptStaticAnalyzer::new()));

        Ok(Self { language_analyzers })
    }

    /// Analyze file structure for architectural issues
    fn analyze_structure(&self, file: &ParsedFile) -> Vec<StaticVulnerability> {
        let mut vulnerabilities = Vec::new();

        // Check for large files (potential god objects)
        if file.source.lines().count() > 1000 {
            let location = SecurityLocation::new(
                file.file_path.as_ref().clone(),
                1,
                file.source.lines().count() as i32,
            );

            vulnerabilities.push(StaticVulnerability {
                category: OwaspCategory::InsecureDesign,
                issue_type: SecurityIssueType::InsecureDesign,
                confidence: 0.6,
                location,
                context: "Large file may indicate poor separation of concerns".to_string(),
                ast_node_type: Some("file".to_string()),
            });
        }

        vulnerabilities
    }

    /// Convert static vulnerabilities to OWASP format
    fn convert_to_owasp(&self, static_vulns: Vec<StaticVulnerability>) -> Vec<OwaspVulnerability> {
        static_vulns
            .into_iter()
            .map(|v| {
                OwaspVulnerability::new(
                    v.category,
                    v.issue_type,
                    format!("Static Analysis: {}", v.issue_type.to_string()),
                    v.context,
                    v.location,
                )
                .with_confidence(v.confidence)
                .with_severity(SecuritySeverity::from_score(v.confidence))
                .with_metadata(VulnerabilityMetadata::new().with_tags(vec![
                    "static-analysis".to_string(),
                    v.ast_node_type.unwrap_or_default(),
                ]))
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl Scanner for StaticAnalysisScanner {
    async fn scan(&self, file: &ParsedFile) -> Result<UnifiedScanResult, AnalysisError> {
        let start_time = Instant::now();
        info!("Starting static analysis scan for: {}", file.file_path.display());

        let mut all_vulnerabilities = Vec::new();
        let mut nodes_analyzed = 0;

        // Perform structural analysis
        let structural_vulns = self.analyze_structure(file);
        all_vulnerabilities.extend(structural_vulns);

        // Perform language-specific analysis
        if let Some(analyzer) = self.language_analyzers.get(&file.language) {
            match analyzer.analyze(file).await {
                Ok(mut lang_vulns) => {
                    nodes_analyzed = lang_vulns.analysis_metadata
                        .get("nodes_analyzed")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    all_vulnerabilities.extend(lang_vulns.vulnerabilities);
                }
                Err(e) => {
                    debug!("Language-specific analysis failed: {}", e);
                }
            }
        }

        let scan_duration = start_time.elapsed().as_millis() as u64;
        let owasp_vulnerabilities = self.convert_to_owasp(all_vulnerabilities);

        let mut metadata = HashMap::new();
        metadata.insert("scanner_type".to_string(), serde_json::Value::String("static".to_string()));
        metadata.insert("nodes_analyzed".to_string(), serde_json::Value::Number(nodes_analyzed.into()));
        metadata.insert("language".to_string(), serde_json::Value::String(file.language.to_string()));

        info!(
            "Static analysis completed: {} vulnerabilities found in {}ms",
            owasp_vulnerabilities.len(),
            scan_duration
        );

        Ok(UnifiedScanResult {
            vulnerabilities: owasp_vulnerabilities,
            scanner_metadata: metadata,
            scan_duration_ms: scan_duration,
        })
    }

    fn name(&self) -> &'static str {
        "static_analysis"
    }

    fn supported_languages(&self) -> Vec<SourceLanguage> {
        vec![
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ]
    }

    fn detectable_categories(&self) -> Vec<OwaspCategory> {
        vec![
            OwaspCategory::InsecureDesign,
            OwaspCategory::SecurityMisconfiguration,
            OwaspCategory::CryptographicFailures,
            OwaspCategory::BrokenAccessControl,
        ]
    }
}

/// Trait for language-specific static analysis
#[async_trait::async_trait]
trait LanguageAnalyzer: Send + Sync {
    async fn analyze(&self, file: &ParsedFile) -> Result<StaticScanResult, AnalysisError>;
}

/// Rust-specific static analyzer
struct RustStaticAnalyzer;

impl RustStaticAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LanguageAnalyzer for RustStaticAnalyzer {
    async fn analyze(&self, file: &ParsedFile) -> Result<StaticScanResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut nodes_analyzed = 0;

        // Analyze for unsafe blocks
        for (line_num, line) in file.source.lines().enumerate() {
            nodes_analyzed += 1;
            if line.trim_start().starts_with("unsafe") {
                let location = SecurityLocation::new(
                    file.file_path.as_ref().clone(),
                    line_num as i32 + 1,
                    line_num as i32 + 1,
                );

                vulnerabilities.push(StaticVulnerability {
                    category: OwaspCategory::SecurityMisconfiguration,
                    issue_type: SecurityIssueType::SecurityMisconfiguration,
                    confidence: 0.7,
                    location,
                    context: "Unsafe block detected - review for memory safety".to_string(),
                    ast_node_type: Some("unsafe_block".to_string()),
                });
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("language".to_string(), serde_json::Value::String("rust".to_string()));
        metadata.insert("nodes_analyzed".to_string(), serde_json::Value::Number(nodes_analyzed.into()));

        Ok(StaticScanResult {
            vulnerabilities,
            analysis_metadata: metadata,
            nodes_analyzed,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

/// Python-specific static analyzer
struct PythonStaticAnalyzer;

impl PythonStaticAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LanguageAnalyzer for PythonStaticAnalyzer {
    async fn analyze(&self, file: &ParsedFile) -> Result<StaticScanResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut nodes_analyzed = 0;

        // Look for eval() usage
        for (line_num, line) in file.source.lines().enumerate() {
            nodes_analyzed += 1;
            if line.contains("eval(") {
                let location = SecurityLocation::new(
                    file.file_path.as_ref().clone(),
                    line_num as i32 + 1,
                    line_num as i32 + 1,
                );

                vulnerabilities.push(StaticVulnerability {
                    category: OwaspCategory::Injection,
                    issue_type: SecurityIssueType::Injection,
                    confidence: 0.9,
                    location,
                    context: "Use of eval() function can lead to code injection".to_string(),
                    ast_node_type: Some("call_expression".to_string()),
                });
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("language".to_string(), serde_json::Value::String("python".to_string()));
        metadata.insert("nodes_analyzed".to_string(), serde_json::Value::Number(nodes_analyzed.into()));

        Ok(StaticScanResult {
            vulnerabilities,
            analysis_metadata: metadata,
            nodes_analyzed,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

/// JavaScript-specific static analyzer
struct JavaScriptStaticAnalyzer;

impl JavaScriptStaticAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LanguageAnalyzer for JavaScriptStaticAnalyzer {
    async fn analyze(&self, file: &ParsedFile) -> Result<StaticScanResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut nodes_analyzed = 0;

        // Look for innerHTML usage
        for (line_num, line) in file.source.lines().enumerate() {
            nodes_analyzed += 1;
            if line.contains("innerHTML") && !line.contains("textContent") {
                let location = SecurityLocation::new(
                    file.file_path.as_ref().clone(),
                    line_num as i32 + 1,
                    line_num as i32 + 1,
                );

                vulnerabilities.push(StaticVulnerability {
                    category: OwaspCategory::Injection,
                    issue_type: SecurityIssueType::CrossSiteScripting,
                    confidence: 0.8,
                    location,
                    context: "innerHTML usage may lead to XSS vulnerabilities".to_string(),
                    ast_node_type: Some("member_expression".to_string()),
                });
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("language".to_string(), serde_json::Value::String("javascript".to_string()));
        metadata.insert("nodes_analyzed".to_string(), serde_json::Value::Number(nodes_analyzed.into()));

        Ok(StaticScanResult {
            vulnerabilities,
            analysis_metadata: metadata,
            nodes_analyzed,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

/// TypeScript-specific static analyzer
struct TypeScriptStaticAnalyzer;

impl TypeScriptStaticAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LanguageAnalyzer for TypeScriptStaticAnalyzer {
    async fn analyze(&self, file: &ParsedFile) -> Result<StaticScanResult, AnalysisError> {
        let start_time = Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut nodes_analyzed = 0;

        // Look for any type assertions that bypass type safety
        for (line_num, line) in file.source.lines().enumerate() {
            nodes_analyzed += 1;
            if line.contains(" as any") || line.contains("<any>") {
                let location = SecurityLocation::new(
                    file.file_path.as_ref().clone(),
                    line_num as i32 + 1,
                    line_num as i32 + 1,
                );

                vulnerabilities.push(StaticVulnerability {
                    category: OwaspCategory::InsecureDesign,
                    issue_type: SecurityIssueType::InsecureDesign,
                    confidence: 0.6,
                    location,
                    context: "Type assertion to 'any' bypasses type safety".to_string(),
                    ast_node_type: Some("type_assertion".to_string()),
                });
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("language".to_string(), serde_json::Value::String("typescript".to_string()));
        metadata.insert("nodes_analyzed".to_string(), serde_json::Value::Number(nodes_analyzed.into()));

        Ok(StaticScanResult {
            vulnerabilities,
            analysis_metadata: metadata,
            nodes_analyzed,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_static_scanner_creation() {
        let scanner = StaticAnalysisScanner::new();
        assert!(scanner.is_ok());

        let scanner = scanner.unwrap();
        assert_eq!(scanner.name(), "static_analysis");
        assert_eq!(scanner.supported_languages().len(), 4);
    }

    #[tokio::test]
    async fn test_rust_unsafe_detection() {
        let scanner = StaticAnalysisScanner::new().unwrap();
        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            source: Arc::new("fn test() {\n    unsafe {\n        // dangerous code\n    }\n}".to_string()),
            tree: None,
            custom_ast: Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        let result = scanner.scan(&file).await.unwrap();
        assert!(!result.vulnerabilities.is_empty());
        assert_eq!(result.vulnerabilities[0].category, OwaspCategory::SecurityMisconfiguration);
    }
}
