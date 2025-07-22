//! Dependency graph builder implementation
//!
//! Constructs dependency graphs from source code analysis.

use super::traits::{AstProvider, DependencyGraphBuilder};
use crate::analysis::detectors::dependency::{Dependency, DependencyExtractor};
use crate::analysis::graph::dependency::{
    ComponentNode, LocalDependencyGraph, LocalDependencyType,
};
use crate::error::UveddiError;
use crate::ingestion::AsyncWalker;

use async_trait::async_trait;
use log::{info, warn};
use std::path::Path;
use std::sync::Arc;
use tokio_stream::StreamExt;

/// Dependency graph builder implementation
pub struct DependencyGraphBuilderImpl {
    ast_provider: Arc<dyn AstProvider>,
    dependency_extractor: DependencyExtractor,
}

impl DependencyGraphBuilderImpl {
    /// Creates a new dependency graph builder
    pub fn new(ast_provider: Arc<dyn AstProvider>) -> Result<Self, UveddiError> {
        let dependency_extractor = DependencyExtractor::new()?;

        Ok(Self {
            ast_provider,
            dependency_extractor,
        })
    }

    /// Creates a new dependency graph builder with custom dependency extractor
    pub fn with_extractor(
        ast_provider: Arc<dyn AstProvider>,
        dependency_extractor: DependencyExtractor,
    ) -> Self {
        Self {
            ast_provider,
            dependency_extractor,
        }
    }

    /// Extracts dependencies from a single file
    async fn extract_file_dependencies(
        &self,
        file_path: &Path,
    ) -> Result<Vec<Dependency>, UveddiError> {
        // Get AST for the file
        let ast = self.ast_provider.get_ast(file_path).await?;

        // Create a minimal ParsedFile structure for dependency extraction
        let file_content =
            std::fs::read_to_string(file_path).map_err(|e| UveddiError::IoError {
                operation: "reading file for dependency extraction".to_string(),
                path: file_path.to_string_lossy().to_string(),
                message: e.to_string(),
                suggestion: "Ensure the file exists and is readable".to_string(),
                source: Some(e),
            })?;

        let parsed_file = crate::ast::ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            language: self.detect_language_from_path(file_path),
            tree: Some((*ast).clone()),
            source: Arc::new(file_content),
            custom_ast: Arc::new(None),
            modified_at: std::fs::metadata(file_path)?.modified()?,
        };

        // Extract dependencies using the dependency extractor
        self.dependency_extractor
            .extract_from_ast(&parsed_file)
            .map_err(|e| UveddiError::DependencyExtractionError {
                module: file_path.to_string_lossy().to_string(),
                message: format!("Failed to extract dependencies from AST: {}", e),
                suggestion: "Check if the file contains valid dependency declarations".to_string(),
                source: None,
            })
    }

    /// Detects programming language from file path extension
    fn detect_language_from_path(&self, path: &Path) -> crate::ast::SourceLanguage {
        use crate::ast::SourceLanguage;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => SourceLanguage::JavaScript,
            _ => SourceLanguage::JavaScript, // Default fallback
        }
    }

    /// Converts a file path to a component node
    fn path_to_component_node(&self, path: &Path) -> ComponentNode {
        ComponentNode::Module {
            path: path.to_string_lossy().into_owned(),
        }
    }

    /// Determines the dependency type based on the dependency information
    fn determine_dependency_type(&self, _dependency: &Dependency) -> LocalDependencyType {
        // For now, treat all dependencies as imports
        // This could be enhanced to distinguish between different types
        // based on the dependency information
        LocalDependencyType::Import
    }
}

#[async_trait]
impl DependencyGraphBuilder for DependencyGraphBuilderImpl {
    async fn build_graph(&self, root_path: &Path) -> Result<LocalDependencyGraph, UveddiError> {
        info!(
            "Building dependency graph from root path: {}",
            root_path.display()
        );

        let mut all_dependencies = Vec::new();
        let walker = AsyncWalker::for_source_code();
        let mut file_stream = walker.walk(root_path);
        let mut files_processed = 0;

        // Walk through all source files and extract dependencies
        while let Some(file_result) = file_stream.next().await {
            match file_result {
                Ok(file_path) => match self.extract_file_dependencies(&file_path).await {
                    Ok(mut file_dependencies) => {
                        info!(
                            "Extracted {} dependencies from {}",
                            file_dependencies.len(),
                            file_path.display()
                        );
                        all_dependencies.append(&mut file_dependencies);
                        files_processed += 1;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to extract dependencies from {}: {}",
                            file_path.display(),
                            e
                        );
                    }
                },
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                }
            }
        }

        info!(
            "Processed {} files, found {} total dependencies",
            files_processed,
            all_dependencies.len()
        );

        // Build the graph from dependencies
        Ok(self.build_from_dependencies(all_dependencies))
    }

    fn build_from_dependencies(&self, dependencies: Vec<Dependency>) -> LocalDependencyGraph {
        let mut graph = LocalDependencyGraph::new();

        for dependency in dependencies {
            let from_node = self.path_to_component_node(&dependency.from_file);
            let to_node = ComponentNode::Module {
                path: dependency.to_module.clone(),
            };
            let dependency_type = self.determine_dependency_type(&dependency);

            graph.add_dependency(&from_node, &to_node, dependency_type);
        }

        info!("Built dependency graph with {} nodes", graph.node_count());
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::components::AstProviderImpl;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    async fn create_test_ast_provider() -> Arc<dyn AstProvider> {
        Arc::new(AstProviderImpl::new().unwrap())
    }

    #[tokio::test]
    async fn test_dependency_graph_builder_creation() {
        let ast_provider = create_test_ast_provider().await;
        let builder = DependencyGraphBuilderImpl::new(ast_provider);
        assert!(builder.is_ok());
    }

    #[tokio::test]
    async fn test_build_from_dependencies() {
        let ast_provider = create_test_ast_provider().await;
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        // Create test dependencies
        let dependencies = vec![
            Dependency {
                from_file: std::path::PathBuf::from("src/main.rs"),
                to_module: "std::collections::HashMap".to_string(),
                dependency_type: crate::database::models::DependencyType::Import,
                line_number: Some(1),
            },
            Dependency {
                from_file: std::path::PathBuf::from("src/lib.rs"),
                to_module: "serde::Serialize".to_string(),
                dependency_type: crate::database::models::DependencyType::Import,
                line_number: Some(2),
            },
        ];

        let graph = builder.build_from_dependencies(dependencies);

        // Should have created nodes and edges
        assert!(graph.node_count() > 0);
    }

    #[tokio::test]
    async fn test_language_detection() {
        let ast_provider = create_test_ast_provider().await;
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        // Test language detection
        assert!(matches!(
            builder.detect_language_from_path(Path::new("test.rs")),
            crate::ast::SourceLanguage::Rust
        ));

        assert!(matches!(
            builder.detect_language_from_path(Path::new("test.py")),
            crate::ast::SourceLanguage::Python
        ));

        assert!(matches!(
            builder.detect_language_from_path(Path::new("test.js")),
            crate::ast::SourceLanguage::JavaScript
        ));
    }

    #[tokio::test]
    async fn test_build_graph_empty_directory() {
        let ast_provider = create_test_ast_provider().await;
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        // Create an empty temporary directory
        let temp_dir = TempDir::new().unwrap();

        let graph = builder.build_graph(temp_dir.path()).await;
        assert!(graph.is_ok());

        let graph = graph.unwrap();
        assert_eq!(graph.node_count(), 0);
    }

    #[tokio::test]
    async fn test_build_graph_with_files() {
        let ast_provider = create_test_ast_provider().await;
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create a simple Rust file
        let main_rs = src_dir.join("main.rs");
        let mut file = fs::File::create(&main_rs).unwrap();
        writeln!(file, "use std::collections::HashMap;").unwrap();
        writeln!(file, "fn main() {{ }}").unwrap();

        // Create another Rust file
        let lib_rs = src_dir.join("lib.rs");
        let mut file = fs::File::create(&lib_rs).unwrap();
        writeln!(file, "pub mod utils;").unwrap();

        let graph = builder.build_graph(&src_dir).await;
        assert!(graph.is_ok());

        // The graph should have been built, though exact structure depends on
        // dependency extraction implementation
        let graph = graph.unwrap();
        // At minimum, we should have processed the files without error
    }

    #[test]
    fn test_path_to_component_node() {
        let ast_provider = Arc::new(AstProviderImpl::new().unwrap());
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        let path = Path::new("src/main.rs");
        let node = builder.path_to_component_node(path);

        match node {
            ComponentNode::Module { path: node_path } => {
                assert_eq!(node_path, "src/main.rs");
            }
            node => {
                eprintln!("Expected Module node but got: {:?}", node);
                assert!(false, "Expected ComponentNode::Module");
            }
        }
    }

    #[test]
    fn test_determine_dependency_type() {
        let ast_provider = Arc::new(AstProviderImpl::new().unwrap());
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        let dependency = Dependency {
            from_file: std::path::PathBuf::from("src/main.rs"),
            to_module: "std::collections::HashMap".to_string(),
            dependency_type: crate::database::models::DependencyType::Import,
            line_number: Some(1),
        };

        let dep_type = builder.determine_dependency_type(&dependency);
        assert!(matches!(dep_type, LocalDependencyType::Import));
    }
}
