//! # Analysis Context
//!
//! Rich context provided to detectors containing all necessary information
//! for analysis including file info, AST, symbols, and project context.

use crate::engine::parsing::{Relation, Symbol};
use std::path::PathBuf;
use std::sync::Arc;

// Cache imports when feature is enabled
#[cfg(feature = "analysis-cache")]
use crate::engine::cache::{AnalysisCache, AstCache};
#[cfg(feature = "analysis-cache")]
use std::sync::Mutex;

// Tree-sitter imports with feature gate
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::Tree;
#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;

/// Comprehensive analysis context for detectors
#[derive(Debug)]
pub struct AnalysisContext {
    /// File metadata
    pub file_info: FileInfo,

    /// Parsed syntax tree
    pub syntax_tree: Option<Tree>,

    /// Source code content
    pub source: String,

    /// Extracted symbols from the file
    pub symbols: Vec<Symbol>,

    /// Relations between code elements
    pub relations: Vec<Relation>,

    /// Project-wide context
    pub project_context: ProjectContext,

    /// Cache handles (when caching is enabled)
    #[cfg(feature = "analysis-cache")]
    pub caches: Option<CacheHandles>,
}

/// File metadata and information
#[derive(Debug)]
pub struct FileInfo {
    /// Path to the file
    pub path: PathBuf,

    /// Detected language
    pub language: crate::ast::SourceLanguage,

    /// Lines of code
    pub lines_of_code: usize,

    /// File size in bytes
    pub size_bytes: usize,

    /// Last modification time
    pub modified_at: std::time::SystemTime,
}

/// Project-wide context information
#[derive(Debug)]
pub struct ProjectContext {
    /// Project root directory
    pub project_root: PathBuf,

    /// All files in the project
    pub project_files: Vec<PathBuf>,

    /// Project dependencies (from package.json, Cargo.toml, etc.)
    pub dependencies: Vec<ProjectDependency>,

    /// Global symbol table
    pub global_symbols: Vec<Symbol>,
}

/// External project dependency
#[derive(Debug)]
pub struct ProjectDependency {
    pub name: String,
    pub version: Option<String>,
    pub source: DependencySource,
}

/// Source of a dependency
#[derive(Debug)]
pub enum DependencySource {
    Registry,
    Git { url: String },
    Path { path: PathBuf },
}

/// Cache handles for accessing AST and analysis caches
#[cfg(feature = "analysis-cache")]
#[derive(Debug, Clone)]
pub struct CacheHandles {
    /// AST cache for parsed syntax trees
    pub ast_cache: Arc<Mutex<AstCache>>,
    /// Analysis cache for detector results
    pub analysis_cache: Arc<Mutex<AnalysisCache>>,
}

impl AnalysisContext {
    /// Create a new analysis context
    pub fn new(
        file_info: FileInfo,
        syntax_tree: Option<Tree>,
        source: String,
        symbols: Vec<Symbol>,
        relations: Vec<Relation>,
        project_context: ProjectContext,
    ) -> Self {
        Self {
            file_info,
            syntax_tree,
            source,
            symbols,
            relations,
            project_context,
            #[cfg(feature = "analysis-cache")]
            caches: None,
        }
    }

    /// Create a new analysis context with caches
    #[cfg(feature = "analysis-cache")]
    pub fn with_caches(
        file_info: FileInfo,
        syntax_tree: Option<Tree>,
        source: String,
        symbols: Vec<Symbol>,
        relations: Vec<Relation>,
        project_context: ProjectContext,
        caches: CacheHandles,
    ) -> Self {
        Self {
            file_info,
            syntax_tree,
            source,
            symbols,
            relations,
            project_context,
            caches: Some(caches),
        }
    }

    /// Get the file's relative path within the project
    pub fn relative_path(&self) -> Option<std::path::PathBuf> {
        self.file_info
            .path
            .strip_prefix(&self.project_context.project_root)
            .ok()
            .map(|p| p.to_path_buf())
    }
}

#[cfg(feature = "analysis-cache")]
impl CacheHandles {
    /// Create new cache handles
    pub fn new(ast_cache: Arc<Mutex<AstCache>>, analysis_cache: Arc<Mutex<AnalysisCache>>) -> Self {
        Self {
            ast_cache,
            analysis_cache,
        }
    }
}
