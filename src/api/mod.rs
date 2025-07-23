//! API module for Uveddi
//! 
//! Provides various API interfaces for accessing Uveddi's analysis capabilities,
//! including GraphQL for flexible queries and potential REST endpoints.

#[cfg(feature = "graphql-api")]
pub mod graphql;

#[cfg(feature = "graphql-api")]
pub mod server;

// Re-export main types when GraphQL feature is enabled
#[cfg(feature = "graphql-api")]
pub use graphql::{create_schema, UveddiSchema, GraphQLConfig};

#[cfg(feature = "graphql-api")]
pub use server::start_graphql_server;