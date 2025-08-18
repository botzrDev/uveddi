//! HTTP server implementation for the GraphQL API

use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_warp::{GraphQLBadRequest, Response};
use std::convert::Infallible;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};

use super::graphql::subscriptions::EventBroadcaster;
use super::graphql::{GraphQLConfig, UveddiSchema};

/// Start the GraphQL HTTP server
pub async fn start_graphql_server(
    schema: UveddiSchema,
    config: GraphQLConfig,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting GraphQL server on http://localhost:{}", port);

    // GraphQL endpoint
    let graphql_post = async_graphql_warp::graphql(schema.clone()).and_then(
        move |(schema, request): (UveddiSchema, async_graphql::Request)| {
            async move {
                // Apply query complexity and depth limits
                let request = request
                    .limit_depth(config.max_depth)
                    .limit_complexity(config.max_complexity);

                let response = schema.execute(request).await;
                Ok::<_, Infallible>(Response::from(response))
            }
        },
    );

    // GraphQL Playground/GraphiQL endpoint (only in development)
    let graphql_playground = if config.enable_playground {
        Some(
            warp::path("playground")
                .and(warp::get())
                .map(|| {
                    warp::http::Response::builder()
                        .header("content-type", "text/html")
                        .body(GraphiQLSource::build().endpoint("/graphql").finish())
                        .unwrap()
                })
                .boxed(),
        )
    } else {
        None
    };

    // Health check endpoint
    let health = warp::path("health").and(warp::get()).map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "service": "uveddi-graphql-api",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    });

    // Metrics endpoint (basic)
    let metrics = warp::path("metrics").and(warp::get()).map(|| {
        // TODO: Integrate with existing monitoring/metrics system
        warp::reply::json(&serde_json::json!({
            "queries_executed": 0,
            "active_subscriptions": 0,
            "cache_hit_rate": 0.0
        }))
    });

    // CORS configuration for browser clients
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    // Combine all routes
    let mut routes = graphql_post
        .or(health)
        .or(metrics)
        .with(cors)
        .recover(handle_rejection)
        .boxed();

    if let Some(playground) = graphql_playground {
        routes = playground.or(routes).boxed();
    }

    // Add request logging middleware
    let routes = routes.with(warp::log("graphql_api"));

    // Start server with graceful shutdown
    let (_, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c signal");
            println!("Received shutdown signal, gracefully shutting down GraphQL server...");
        });

    server.await;
    Ok(())
}

/// Handle HTTP rejections and convert them to appropriate responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(GraphQLBadRequest(err)) = err.find() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "GraphQL Bad Request",
                "message": err.to_string()
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if err.is_not_found() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "Not Found",
                "message": "The requested resource was not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        ));
    }

    if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "Bad Request",
                "message": "Invalid request body"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Generic error handler
    eprintln!("Unhandled rejection: {:?}", err);
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
    ))
}

/// Server configuration builder
pub struct GraphQLServerBuilder {
    schema: Option<UveddiSchema>,
    config: GraphQLConfig,
    port: u16,
    event_broadcaster: Option<Arc<EventBroadcaster>>,
}

impl GraphQLServerBuilder {
    pub fn new() -> Self {
        Self {
            schema: None,
            config: GraphQLConfig::default(),
            port: 4000,
            event_broadcaster: None,
        }
    }

    pub fn schema(mut self, schema: UveddiSchema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn config(mut self, config: GraphQLConfig) -> Self {
        self.config = config;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn event_broadcaster(mut self, broadcaster: Arc<EventBroadcaster>) -> Self {
        self.event_broadcaster = Some(broadcaster);
        self
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let schema = self.schema.ok_or("Schema is required")?;

        // Add event broadcaster to schema context if provided
        let schema = if let Some(broadcaster) = self.event_broadcaster {
            schema.data(broadcaster)
        } else {
            schema
        };

        start_graphql_server(schema, self.config, self.port).await
    }
}

impl Default for GraphQLServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::engine::AnalysisEngine;
    use crate::api::graphql::create_schema;
    use crate::database::Database;

    async fn create_test_schema() -> UveddiSchema {
        // This would normally use real instances, but for testing we create mocks
        let db = Database::new(":memory:").unwrap();
        let engine = AnalysisEngine::new();
        create_schema(db, engine)
    }

    #[tokio::test]
    async fn test_server_builder() {
        let schema = create_test_schema().await;
        let config = GraphQLConfig {
            enable_playground: true,
            max_depth: 10,
            max_complexity: 500,
            ..Default::default()
        };

        let builder = GraphQLServerBuilder::new()
            .schema(schema)
            .config(config)
            .port(4001);

        // We can't actually start the server in tests, but we can verify the builder works
        assert_eq!(builder.port, 4001);
        assert!(builder.schema.is_some());
    }

    #[test]
    fn test_graphql_config_defaults() {
        let config = GraphQLConfig::default();
        assert_eq!(config.max_depth, 15);
        assert_eq!(config.max_complexity, 1000);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.enable_introspection);
    }
}
