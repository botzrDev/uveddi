//! # Engine Module
//!
//! Centralized analysis engine providing clean separation between parsing and analysis.
//! This module orchestrates AST parsing, knowledge graph construction, and analysis
//! pipeline execution.

pub mod analysis;
pub mod cache;
pub mod knowledge_graph;
pub mod parsing;

#[cfg(test)]
mod test_compilation;
#[cfg(test)]
mod test_integration;

// Re-export commonly used types
pub use analysis::{AnalysisContext, AnalysisPipeline};
pub use cache::{AnalysisCache, AstCache};
pub use knowledge_graph::{KnowledgeGraph, QueryBuilder};
pub use parsing::{AstBuilder, LanguageParser, ParseResult, ParsedFile};

// Cache handles for context (when caching is enabled)
#[cfg(feature = "analysis-cache")]
pub use analysis::context::CacheHandles;
