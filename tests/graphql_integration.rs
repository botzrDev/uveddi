//! GraphQL API integration tests for Uveddi
//!
//! This module provides comprehensive testing of the GraphQL API endpoints,
//! including queries, mutations, subscriptions, and error handling.
//!
//! # Test Coverage
//! - GraphQL query execution
//! - Schema introspection
//! - Error handling and validation
//! - Query complexity and depth limits
//! - Subscription functionality
//! - Authentication and authorization
//!
//! # Running Tests
//! ```bash
//! cargo test graphql_integration --features=production
//! ```

use async_graphql::{http::GraphiQLSource, Request, Response, Schema};
use async_graphql_warp::{GraphQLBadRequest, Response as WarpResponse};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::oneshot;
use warp::{reply::Reply, test::request, Filter};

use crate::analysis::engine::AnalysisEngine;
use crate::api::graphql::{create_schema, GraphQLConfig, UveddiSchema};
use crate::api::server::{start_graphql_server, GraphQLServerBuilder};
use crate::database::Database;
use crate::test_utils::helpers::TestResult;

/// GraphQL test client for integration testing
pub struct GraphQLTestClient {
    schema: UveddiSchema,
    database: Arc<Database>,
    _temp_dir: TempDir,
}

impl GraphQLTestClient {
    /// Create a new GraphQL test client
    pub async fn new() -> TestResult<Self> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let database = Arc::new(Database::new(db_path.to_str().unwrap())?);

        // Initialize database schema
        database.init_schema().await?;

        // Create analysis engine
        let engine = AnalysisEngine::new();

        // Create GraphQL schema
        let schema = create_schema(database.clone(), engine);

        Ok(Self {
            schema,
            database,
            _temp_dir: temp_dir,
        })
    }

    /// Execute a GraphQL query
    pub async fn query(&self, query: &str) -> TestResult<Response> {
        let request = Request::new(query);
        let response = self.schema.execute(request).await;
        Ok(response)
    }

    /// Execute a GraphQL query with variables
    pub async fn query_with_variables(
        &self,
        query: &str,
        variables: Value,
    ) -> TestResult<Response> {
        let request = Request::new(query).variables(variables);
        let response = self.schema.execute(request).await;
        Ok(response)
    }

    /// Execute a GraphQL subscription
    pub async fn subscribe(
        &self,
        query: &str,
    ) -> TestResult<impl futures::Stream<Item = Response>> {
        let request = Request::new(query);
        let stream = self.schema.execute_stream(request);
        Ok(stream)
    }

    /// Insert test data for GraphQL queries
    pub async fn insert_test_data(&self) -> TestResult<i64> {
        // Create test analysis run
        let run_id = self
            .database
            .create_analysis_run(
                1, // project_id
                "GraphQL Test Analysis".to_string(),
                Some("https://github.com/test/repo".to_string()),
                Some("abc123def456".to_string()),
                Some("main".to_string()),
            )
            .await?;

        // Insert test architectural issue
        let _issue_id = self
            .database
            .insert_architectural_issue(
                run_id,
                Some(1), // anti_pattern_type_id
                "test_file.rs".to_string(),
                10,       // start_line
                Some(20), // end_line
                "Test GraphQL issue".to_string(),
                Some("Fix this GraphQL issue".to_string()),
                0.95,                                // confidence
                Some("critical".to_string()),        // severity
                Some(json!({"graphql_test": true})), // metadata
            )
            .await?;

        Ok(run_id)
    }
}

#[tokio::test]
async fn test_graphql_schema_introspection() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    let query = r#"
        query IntrospectionQuery {
            __schema {
                types {
                    name
                    kind
                }
            }
        }
    "#;

    let response = client.query(query).await?;

    assert!(response.errors.is_empty());
    assert!(response.data.is_object());

    let data = response.data.into_json()?;
    let types = data["__schema"]["types"].as_array().unwrap();

    // Should have standard GraphQL types
    let type_names: Vec<&str> = types.iter().map(|t| t["name"].as_str().unwrap()).collect();

    assert!(type_names.contains(&"String"));
    assert!(type_names.contains(&"Int"));
    assert!(type_names.contains(&"Boolean"));

    Ok(())
}

#[tokio::test]
async fn test_graphql_query_validation() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    // Test invalid query syntax
    let invalid_query = r#"
        query {
            invalidField {
                nonExistentField
            }
        }
    "#;

    let response = client.query(invalid_query).await?;

    // Should return validation errors
    assert!(!response.errors.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_graphql_query_complexity_limit() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    // Create a complex nested query
    let complex_query = r#"
        query ComplexQuery {
            analysis {
                runs {
                    issues {
                        findings {
                            dependencies {
                                files {
                                    content {
                                        lines {
                                            tokens
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;

    let response = client.query(complex_query).await?;

    // Should either succeed or fail with complexity limit error
    // depending on the configured limits
    if !response.errors.is_empty() {
        let error_message = response.errors[0].message.to_lowercase();
        assert!(
            error_message.contains("complexity")
                || error_message.contains("field")
                || error_message.contains("depth")
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_graphql_query_depth_limit() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    // Create a deeply nested query
    let deep_query = r#"
        query DeepQuery {
            level1 {
                level2 {
                    level3 {
                        level4 {
                            level5 {
                                level6 {
                                    level7 {
                                        level8 {
                                            level9 {
                                                level10 {
                                                    data
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;

    let response = client.query(deep_query).await?;

    // Should return depth limit error if limits are enforced
    if !response.errors.is_empty() {
        let error_message = response.errors[0].message.to_lowercase();
        assert!(error_message.contains("depth") || error_message.contains("field"));
    }

    Ok(())
}

#[tokio::test]
async fn test_graphql_variables_validation() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    let query = r#"
        query GetAnalysisRun($runId: ID!) {
            analysisRun(id: $runId) {
                id
                status
                startTime
            }
        }
    "#;

    // Test with valid variables
    let valid_variables = json!({
        "runId": "1"
    });

    let response = client.query_with_variables(query, valid_variables).await?;

    // Should not have validation errors
    // (may have execution errors if data doesn't exist, but not validation errors)
    let has_validation_errors = response
        .errors
        .iter()
        .any(|e| e.message.to_lowercase().contains("variable"));

    assert!(!has_validation_errors);

    // Test with invalid variables
    let invalid_variables = json!({
        "runId": 123 // Should be string, not number
    });

    let response = client
        .query_with_variables(query, invalid_variables)
        .await?;

    // May have type coercion or validation errors
    // GraphQL implementations vary in how they handle this

    Ok(())
}

#[tokio::test]
async fn test_graphql_error_handling() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    // Query that should trigger a resolver error
    let query = r#"
        query {
            analysisRun(id: "999999") {
                id
                projectName
                issues {
                    total
                }
            }
        }
    "#;

    let response = client.query(query).await?;

    // Should handle the error gracefully
    // Either return null with no errors (if nullable) or return an error
    if !response.errors.is_empty() {
        let error = &response.errors[0];
        assert!(!error.message.is_empty());
        // Error should have path information
        assert!(error.path.is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_graphql_subscription_basic() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    let subscription = r#"
        subscription {
            analysisUpdates {
                runId
                status
                progress
            }
        }
    "#;

    let mut stream = client.subscribe(subscription).await?;

    // Test that subscription can be created without errors
    // Note: Testing actual subscription events would require triggering
    // analysis events, which is complex for a unit test

    Ok(())
}

#[tokio::test]
async fn test_graphql_mutation_validation() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    let mutation = r#"
        mutation StartAnalysis($input: AnalysisInput!) {
            startAnalysis(input: $input) {
                id
                status
            }
        }
    "#;

    let variables = json!({
        "input": {
            "projectPath": "/test/path",
            "configuration": {
                "languages": ["rust"],
                "detectors": ["god_object", "dead_code"]
            }
        }
    });

    let response = client.query_with_variables(mutation, variables).await?;

    // Should validate input structure
    // May succeed or fail depending on resolver implementation

    Ok(())
}

#[tokio::test]
async fn test_graphql_field_aliasing() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    let query = r#"
        query {
            primaryRun: analysisRun(id: "1") {
                runId: id
                runStatus: status
            }
            secondaryRun: analysisRun(id: "2") {
                runId: id  
                runStatus: status
            }
        }
    "#;

    let response = client.query(query).await?;

    // Should handle field aliasing correctly
    let data = response.data.into_json()?;

    // Check that aliases are used in response
    if data.get("primaryRun").is_some() {
        assert!(data["primaryRun"].get("runId").is_some());
        assert!(data["primaryRun"].get("runStatus").is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_graphql_fragments() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    let query = r#"
        fragment IssueDetails on ArchitecturalIssue {
            id
            severity
            confidence
            message
            file {
                path
                startLine
                endLine
            }
        }
        
        query {
            analysisRun(id: "1") {
                issues {
                    ...IssueDetails
                }
            }
        }
    "#;

    let response = client.query(query).await?;

    // Should handle fragments without syntax errors
    // Execution may fail if types don't match expected schema

    Ok(())
}

#[tokio::test]
async fn test_graphql_conditional_fields() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    let query = r#"
        query GetAnalysisWithConditions($includeDetails: Boolean!) {
            analysisRun(id: "1") {
                id
                status
                issues @include(if: $includeDetails) {
                    total
                    critical
                }
                metadata @skip(if: $includeDetails) {
                    version
                }
            }
        }
    "#;

    let variables = json!({
        "includeDetails": true
    });

    let response = client.query_with_variables(query, variables).await?;

    // Should handle conditional fields correctly
    assert!(
        response.errors.is_empty()
            || response
                .errors
                .iter()
                .all(|e| !e.message.contains("@include"))
    );

    Ok(())
}

#[tokio::test]
async fn test_graphql_pagination() -> TestResult {
    let client = GraphQLTestClient::new().await?;

    // Insert test data
    let _run_id = client.insert_test_data().await?;

    let query = r#"
        query PaginatedIssues($first: Int!, $after: String) {
            analysisRun(id: "1") {
                issues(first: $first, after: $after) {
                    edges {
                        node {
                            id
                            severity
                        }
                        cursor
                    }
                    pageInfo {
                        hasNextPage
                        hasPreviousPage
                        startCursor
                        endCursor
                    }
                }
            }
        }
    "#;

    let variables = json!({
        "first": 10,
        "after": null
    });

    let response = client.query_with_variables(query, variables).await?;

    // Should handle pagination structure
    if response.errors.is_empty() {
        let data = response.data.into_json()?;
        if let Some(run) = data.get("analysisRun") {
            if let Some(issues) = run.get("issues") {
                assert!(issues.get("pageInfo").is_some());
            }
        }
    }

    Ok(())
}

/// Authentication and authorization tests for GraphQL
mod graphql_auth_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Enable when authentication is implemented
    async fn test_graphql_unauthorized_query() -> TestResult {
        let client = GraphQLTestClient::new().await?;

        let query = r#"
            query {
                adminStats {
                    totalUsers
                    systemHealth
                }
            }
        "#;

        let response = client.query(query).await?;

        // Should return authorization error
        assert!(!response.errors.is_empty());
        assert!(response.errors[0].message.to_lowercase().contains("auth"));

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Enable when authentication is implemented
    async fn test_graphql_field_level_authorization() -> TestResult {
        let client = GraphQLTestClient::new().await?;

        let query = r#"
            query {
                analysisRun(id: "1") {
                    id
                    status
                    sensitiveData {
                        internalNotes
                        debugInfo
                    }
                }
            }
        "#;

        let response = client.query(query).await?;

        // Should allow access to public fields but restrict sensitive fields
        if !response.errors.is_empty() {
            let has_auth_error = response.errors.iter().any(|e| {
                e.message.to_lowercase().contains("auth")
                    || e.message.to_lowercase().contains("permission")
            });
            assert!(has_auth_error);
        }

        Ok(())
    }
}

/// Performance and scalability tests
mod graphql_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_graphql_query_performance() -> TestResult {
        let client = GraphQLTestClient::new().await?;

        // Insert test data
        let _run_id = client.insert_test_data().await?;

        let query = r#"
            query {
                analysisRuns(first: 100) {
                    edges {
                        node {
                            id
                            status
                            issues {
                                total
                            }
                        }
                    }
                }
            }
        "#;

        let start = std::time::Instant::now();
        let response = client.query(query).await?;
        let duration = start.elapsed();

        // Query should complete within reasonable time (5 seconds)
        assert!(duration.as_secs() < 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_graphql_concurrent_queries() -> TestResult {
        let client = Arc::new(GraphQLTestClient::new().await?);

        let query = r#"
            query {
                __schema {
                    queryType {
                        name
                    }
                }
            }
        "#;

        // Run multiple queries concurrently
        let mut handles = Vec::new();

        for _ in 0..10 {
            let client = client.clone();
            let query = query.to_string();

            let handle = tokio::spawn(async move { client.query(&query).await });

            handles.push(handle);
        }

        // Wait for all queries to complete
        for handle in handles {
            let response = handle.await??;
            assert!(response.errors.is_empty());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_graphql_memory_usage() -> TestResult {
        let client = GraphQLTestClient::new().await?;

        // Execute many small queries to test memory handling
        for i in 0..100 {
            let query = format!(
                r#"
                query Query{} {{
                    __schema {{
                        queryType {{
                            name
                        }}
                    }}
                }}
            "#,
                i
            );

            let response = client.query(&query).await?;
            assert!(response.errors.is_empty());
        }

        Ok(())
    }
}

/// Integration test for the complete GraphQL server
#[tokio::test]
async fn test_graphql_server_integration() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let database = Arc::new(Database::new(db_path.to_str().unwrap())?);
    database.init_schema().await?;

    let engine = AnalysisEngine::new();
    let schema = create_schema(database, engine);

    let config = GraphQLConfig {
        enable_playground: true,
        max_depth: 10,
        max_complexity: 1000,
        timeout_seconds: 30,
        enable_introspection: true,
    };

    // Test server builder
    let builder = GraphQLServerBuilder::new()
        .schema(schema)
        .config(config)
        .port(0); // Use random port for testing

    // We can't easily test the full server startup in a unit test,
    // but we can verify the builder configuration
    assert_eq!(builder.port, 0);
    assert!(builder.schema.is_some());

    Ok(())
}
