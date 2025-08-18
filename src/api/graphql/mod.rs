//! GraphQL API module for Uveddi analysis results
//!
//! Provides a flexible query interface for code analysis data with efficient
//! data fetching, pagination, and real-time subscriptions.

pub mod resolvers;
// pub mod schema; // TODO: Create schema.rs for GraphQL schema types - disabled for v1.0
pub mod context;
pub mod loaders;
pub mod pagination;
pub mod subscriptions;
pub mod types;

use crate::analysis::engine::AnalysisEngine;
use crate::database::DatabaseManager;
use async_graphql::{EmptySubscription, Schema};

pub use context::GraphQLContext;
pub use resolvers::{MutationRoot, QueryRoot};
pub use types::*;

/// GraphQL schema type for Uveddi
pub type UveddiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Create a new GraphQL schema with the provided database and analysis engine
pub fn create_schema(db: DatabaseManager, analysis_engine: AnalysisEngine) -> UveddiSchema {
    let context = GraphQLContext::new(db, analysis_engine);

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(context)
        .finish()
}

/// Configuration for GraphQL API
#[derive(Debug, Clone)]
pub struct GraphQLConfig {
    /// Maximum query depth to prevent DoS attacks
    pub max_depth: usize,
    /// Maximum query complexity
    pub max_complexity: usize,
    /// Enable GraphQL playground in development
    pub enable_playground: bool,
    /// Enable introspection
    pub enable_introspection: bool,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Enable real-time subscriptions
    pub enable_subscriptions: bool,
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            max_depth: 15,
            max_complexity: 1000,
            enable_playground: cfg!(debug_assertions),
            enable_introspection: true,
            timeout_seconds: 30,
            enable_subscriptions: true,
        }
    }
}
