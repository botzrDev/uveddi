use std::path::Path;

use crate::analysis::detectors::dependency::analyzers::{AnalysisOutput, AnalyzerRegistry};
use crate::analysis::detectors::dependency::config::DependencyDetectorConfig;
use crate::analysis::detectors::dependency::language_support::LanguageParserRegistry;
use crate::analysis::detectors::dependency::licenses::{
    LicenseCheckOutput, LicenseCheckerRegistry,
};
use crate::analysis::detectors::dependency::types::*;
use crate::analysis::detectors::dependency::vulnerabilities::{
    VulnerabilityScanOutput, VulnerabilityScannerRegistry,
};

use crate::ast::tree_sitter_impl::{AstError, AstParser, ParsedFile, SourceLanguage};
pub use crate::database::models::{Dependency, DependencyType};

#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("AST parsing error: {0}")]
    AstError(#[from] AstError),
    #[error("IO error for path {0}: {1}")]
    IoError(std::path::PathBuf, #[source] std::io::Error),
    #[error("Query compilation error: {0}")]
    QueryError(String),
    #[error("Invalid file path: {path} - Reason: {reason}")]
    InvalidPath {
        path: std::path::PathBuf,
        reason: String,
    },
    #[error("Unsupported language for path: {0}")]
    UnsupportedLanguage(String),
}

impl From<DependencyError> for ExtractionError {
    fn from(err: DependencyError) -> Self {
        Self::QueryError(err.to_string())
    }
}

pub struct DependencyDetector {
    config: DependencyDetectorConfig,
    parser_registry: LanguageParserRegistry,
    analyzer_registry: AnalyzerRegistry,
    vulnerability_registry: VulnerabilityScannerRegistry,
    license_registry: LicenseCheckerRegistry,
    ast_parser: AstParser,
}

impl DependencyDetector {
    pub fn new() -> Result<Self, ExtractionError> {
        let config = DependencyDetectorConfig::default();
        let parser_registry = LanguageParserRegistry::new();
        let analyzer_registry = AnalyzerRegistry::new();
        let vulnerability_registry = VulnerabilityScannerRegistry::new();
        let license_registry = LicenseCheckerRegistry::new();
        let ast_parser = AstParser::new().map_err(ExtractionError::AstError)?;

        Ok(Self {
            config,
            parser_registry,
            analyzer_registry,
            vulnerability_registry,
            license_registry,
            ast_parser,
        })
    }

    pub fn with_config(mut self, config: DependencyDetectorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn analyze_project(
        &mut self,
        project_root: &Path,
    ) -> Result<DependencyAnalysisResult, ExtractionError> {
        let dependencies = self.discover_dependencies(project_root)?;
        self.analyze_dependencies(&dependencies)
    }

    pub fn analyze_file(&mut self, file_path: &Path) -> Result<Vec<Dependency>, ExtractionError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| ExtractionError::IoError(file_path.to_path_buf(), e))?;

        let language = SourceLanguage::from_path(file_path).ok_or_else(|| {
            ExtractionError::UnsupportedLanguage(file_path.to_string_lossy().to_string())
        })?;

        let parsed_file = self
            .ast_parser
            .parse_content(&content, file_path, language)
            .map_err(ExtractionError::AstError)?;

        self.extract_from_ast(&parsed_file)
    }

    fn discover_dependencies(
        &mut self,
        project_root: &Path,
    ) -> Result<Vec<DependencyInfo>, ExtractionError> {
        let mut all_dependencies = Vec::new();

        let manifest_deps = self
            .parser_registry
            .parse_all_manifests(project_root)
            .map_err(ExtractionError::from)?;

        all_dependencies.extend(manifest_deps);

        let source_deps = self.discover_source_dependencies(project_root)?;
        all_dependencies.extend(source_deps);

        self.deduplicate_dependencies(&mut all_dependencies);
        Ok(all_dependencies)
    }

    fn discover_source_dependencies(
        &mut self,
        project_root: &Path,
    ) -> Result<Vec<DependencyInfo>, ExtractionError> {
        let mut dependencies = Vec::new();

        if let Ok(entries) = std::fs::read_dir(project_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && self.is_source_file(&path) {
                    let file_deps = self.analyze_file(&path)?;
                    for dep in file_deps {
                        dependencies.push(DependencyInfo {
                            name: dep.to_module,
                            version: None,
                            source: DependencySource::Unknown,
                            scope: DependencyScope::Production,
                            resolved_path: Some(dep.from_file),
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(ext, "rs" | "py" | "js" | "ts" | "jsx" | "tsx")
        } else {
            false
        }
    }

    fn deduplicate_dependencies(&self, dependencies: &mut Vec<DependencyInfo>) {
        use std::collections::HashSet;
        let mut seen: HashSet<(String, Option<String>)> = HashSet::new();
        dependencies.retain(|dep| {
            let key = (dep.name.clone(), dep.version.clone());
            seen.insert(key)
        });
    }

    fn analyze_dependencies(
        &self,
        dependencies: &[DependencyInfo],
    ) -> Result<DependencyAnalysisResult, ExtractionError> {
        // Run analysis synchronously
        let analysis_results = self.analyzer_registry.run_all(dependencies, &self.config);
        let vulnerability_results = self
            .vulnerability_registry
            .run_all(dependencies, &self.config.vulnerability_scanning);
        let license_results = self
            .license_registry
            .run_all(dependencies, &self.config.license_checking);

        self.consolidate_results(analysis_results, vulnerability_results, license_results)
    }

    fn consolidate_results(
        &self,
        analysis_results: Vec<Result<AnalysisOutput, DependencyError>>,
        vulnerability_results: Vec<Result<VulnerabilityScanOutput, DependencyError>>,
        license_results: Vec<Result<LicenseCheckOutput, DependencyError>>,
    ) -> Result<DependencyAnalysisResult, ExtractionError> {
        use crate::analysis::detectors::dependency::analyzers::AnalysisOutput::*;

        let mut graph = DependencyGraph {
            nodes: std::collections::HashMap::new(),
            edges: Vec::new(),
            roots: std::collections::HashSet::new(),
        };
        let mut vulnerabilities = Vec::new();
        let mut license_conflicts = Vec::new();
        let mut circular_dependencies = Vec::new();
        let mut outdated_dependencies = Vec::new();
        let mut supply_chain_risks = Vec::new();

        for result in analysis_results {
            if let Ok(output) = result {
                match output {
                    Graph(graph_result) => {
                        graph = graph_result.graph;
                    }
                    Circular(circular_result) => {
                        circular_dependencies = circular_result.cycles;
                    }
                    Outdated(outdated_result) => {
                        outdated_dependencies = outdated_result.outdated_packages;
                    }
                    _ => {}
                }
            }
        }

        let metrics = DependencyMetrics {
            total_dependencies: graph.nodes.len(),
            direct_dependencies: graph.nodes.values().filter(|n| n.is_direct).count(),
            transitive_dependencies: graph.nodes.values().filter(|n| !n.is_direct).count(),
            outdated_count: outdated_dependencies.len(),
            vulnerable_count: vulnerabilities.len(),
            max_depth: graph.nodes.values().map(|n| n.depth).max().unwrap_or(0),
            avg_depth: if graph.nodes.is_empty() {
                0.0
            } else {
                graph.nodes.values().map(|n| n.depth as f64).sum::<f64>() / graph.nodes.len() as f64
            },
            license_types: std::collections::HashMap::new(),
        };

        Ok(DependencyAnalysisResult {
            graph,
            vulnerabilities,
            license_conflicts,
            circular_dependencies,
            outdated_dependencies,
            supply_chain_risks,
            metrics,
        })
    }

    pub fn extract_from_ast(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        #[cfg(feature = "tree-sitter")]
        {
            use crate::ast::tree_sitter::queries::{
                JAVASCRIPT_IMPORTS_QUERY, PYTHON_IMPORTS_QUERY, RUST_IMPORTS_QUERY,
            };
            use streaming_iterator::StreamingIterator;
            use tree_sitter::{Query, QueryCursor};

            let (query_str, dependency_type) = match parsed_file.language {
                SourceLanguage::Rust => (RUST_IMPORTS_QUERY, DependencyType::Use),
                SourceLanguage::Python => (PYTHON_IMPORTS_QUERY, DependencyType::Import),
                SourceLanguage::JavaScript => (JAVASCRIPT_IMPORTS_QUERY, DependencyType::Import),
                SourceLanguage::TypeScript => (JAVASCRIPT_IMPORTS_QUERY, DependencyType::Import),
                _ => return Ok(Vec::new()),
            };

            let query = Query::new(
                &parsed_file
                    .tree
                    .as_ref()
                    .expect("AST tree missing")
                    .language(),
                query_str,
            )
            .map_err(|e| ExtractionError::QueryError(e.to_string()))?;

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(
                &query,
                parsed_file
                    .tree
                    .as_ref()
                    .expect("AST tree missing")
                    .root_node(),
                parsed_file.source.as_bytes(),
            );

            let mut dependencies = Vec::new();
            while let Some(mat) = matches.next() {
                for capture in mat.captures {
                    let capture_name = query.capture_names()[capture.index as usize];
                    if capture_name != "path" {
                        continue;
                    }

                    let node = capture.node;
                    let line_number = node.start_position().row + 1;
                    let mut module_name = node
                        .utf8_text(parsed_file.source.as_bytes())
                        .unwrap_or("")
                        .to_string();

                    if module_name.starts_with('"') && module_name.ends_with('"')
                        || module_name.starts_with('\'') && module_name.ends_with('\'')
                    {
                        module_name = module_name[1..module_name.len() - 1].to_string();
                    }

                    dependencies.push(Dependency {
                        from_file: parsed_file.file_path.as_ref().clone(),
                        to_module: module_name,
                        dependency_type: dependency_type.clone(),
                        line_number: Some(line_number as u32),
                    });
                }
            }

            Ok(dependencies)
        }

        #[cfg(not(feature = "tree-sitter"))]
        {
            Ok(Vec::new())
        }
    }
}

impl Default for DependencyDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create DependencyDetector")
    }
}
