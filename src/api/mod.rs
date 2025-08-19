//! API module for Uveddi
//!
//! Provides various API interfaces for accessing Uveddi's analysis capabilities,
//! including GraphQL for flexible queries and REST endpoints for interactive reports.

// Shared API types to prevent circular dependencies
pub mod types;

#[cfg(feature = "graphql-api")]
pub mod graphql;

#[cfg(feature = "graphql-api")]
pub mod server;

// REST API for interactive reports
pub mod rest;

// Re-export main types when GraphQL feature is enabled
#[cfg(feature = "graphql-api")]
pub use graphql::{create_schema, GraphQLConfig, UveddiSchema};

#[cfg(feature = "graphql-api")]
pub use server::start_graphql_server;

// Re-export shared types
pub use types::{ApiServer, RestApiConfig};

// Re-export REST API types
pub use rest::{CombinedApiServer, RestApiService};
