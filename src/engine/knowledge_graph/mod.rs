//! # Knowledge Graph Module
//!
//! Manages the knowledge graph for code relationships and dependencies.
//! Provides efficient querying and incremental building capabilities.

pub mod builder;
pub mod query;
pub mod relations;

// Re-export main types
pub use builder::{GraphBuilder, KnowledgeGraph};
pub use query::QueryBuilder;
pub use relations::{GraphRelation, RelationType};
