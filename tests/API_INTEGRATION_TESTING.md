# API Integration Testing Guide

This document provides comprehensive guidance for running and extending the API integration tests for Uveddi.

## Overview

The Uveddi API integration tests provide thorough coverage of both REST and GraphQL endpoints, ensuring reliability and correctness of the API layer. Tests are organized into focused modules that can be run independently or as a complete suite.

## Test Structure

```
tests/
├── api_integration.rs          # REST API integration tests
├── graphql_integration.rs      # GraphQL API integration tests
├── test_utils/                 # Shared test utilities
│   ├── helpers.rs             # Test helper functions
│   ├── mocks.rs               # Mock implementations
│   └── fixtures.rs            # Test data fixtures
└── API_INTEGRATION_TESTING.md # This documentation
```

## Test Categories

### 1. REST API Tests (`api_integration.rs`)

#### Health Endpoints
- **`test_health_endpoint_success`**: Validates health check endpoint returns correct status
- **`test_health_endpoint_cors_headers`**: Verifies CORS headers are properly configured

#### Report Management
- **`test_list_reports_empty_database`**: Tests report listing with empty database
- **`test_list_reports_with_data`**: Tests report listing with actual data
- **`test_get_demo_report`**: Validates demo report structure and content
- **`test_get_report_not_found`**: Tests 404 handling for missing reports
- **`test_get_report_invalid_id`**: Tests handling of invalid report IDs

#### Security Endpoints (Feature-Gated)
- **`test_security_issues_endpoint`**: Tests security issues listing
- **`test_security_issue_by_id`**: Tests individual security issue retrieval
- **`test_security_summary`**: Validates security summary statistics
- **`test_owasp_coverage`**: Tests OWASP Top 10 coverage reporting
- **`test_taint_flows`**: Tests taint flow analysis endpoints
- **`test_sarif_export`**: Tests SARIF format export functionality

#### Error Handling and Edge Cases
- **`test_invalid_endpoint_returns_404`**: Tests 404 for non-existent endpoints
- **`test_malformed_request_body`**: Tests handling of malformed JSON
- **`test_content_type_validation`**: Tests content type validation
- **`test_large_request_handling`**: Tests handling of large payloads
- **`test_concurrent_requests`**: Tests concurrent request handling
- **`test_request_timeout_handling`**: Tests timeout behavior

#### Authentication Tests (Future)
- **`test_unauthorized_access`**: Tests unauthorized access blocking
- **`test_invalid_token`**: Tests invalid JWT token handling
- **`test_expired_token`**: Tests expired token handling
- **`test_insufficient_permissions`**: Tests role-based access control

### 2. GraphQL API Tests (`graphql_integration.rs`)

#### Schema and Validation
- **`test_graphql_schema_introspection`**: Tests GraphQL schema introspection
- **`test_graphql_query_validation`**: Tests query syntax validation
- **`test_graphql_query_complexity_limit`**: Tests query complexity limits
- **`test_graphql_query_depth_limit`**: Tests query depth limits
- **`test_graphql_variables_validation`**: Tests variable type validation

#### Query Features
- **`test_graphql_field_aliasing`**: Tests field aliasing functionality
- **`test_graphql_fragments`**: Tests GraphQL fragments
- **`test_graphql_conditional_fields`**: Tests @include and @skip directives
- **`test_graphql_pagination`**: Tests cursor-based pagination

#### Real-time Features
- **`test_graphql_subscription_basic`**: Tests subscription setup
- **`test_graphql_mutation_validation`**: Tests mutation input validation

#### Performance Tests
- **`test_graphql_query_performance`**: Tests query execution performance
- **`test_graphql_concurrent_queries`**: Tests concurrent query handling
- **`test_graphql_memory_usage`**: Tests memory usage under load

## Running Tests

### Prerequisites

Ensure you have the required features enabled:

```bash
# For full API testing including security features
cargo test --features=production

# For basic API testing without security features
cargo test --features=dev-core
```

### Running Specific Test Suites

```bash
# Run all API integration tests
cargo test api_integration

# Run all GraphQL integration tests  
cargo test graphql_integration

# Run specific test by name
cargo test test_health_endpoint_success

# Run tests with output
cargo test api_integration -- --nocapture

# Run tests in parallel (default)
cargo test api_integration -- --test-threads=4

# Run tests serially (if needed for database conflicts)
cargo test api_integration -- --test-threads=1
```

### Running with Different Feature Sets

```bash
# Production features (includes security, tree-sitter, web-full)
cargo test --features=production api_integration

# Development features (faster compilation)
cargo test --features=dev-core api_integration

# Security-specific tests
cargo test --features=security test_security

# Without tree-sitter (faster compilation)
cargo test --features=dev-minimal api_integration
```

### Filtering Tests by Category

```bash
# Run only authentication tests (when implemented)
cargo test auth_tests

# Run only performance tests
cargo test performance_tests

# Run only error handling tests
cargo test error

# Run only database integration tests
cargo test database_integration
```

## Test Configuration

### Environment Variables

The tests respect the following environment variables:

```bash
# Test database configuration
export UVEDDI_TEST_DB_PATH="/tmp/uveddi_test.db"

# Test timeouts (in seconds)
export UVEDDI_TEST_TIMEOUT=30

# Enable test debug output
export RUST_LOG=debug

# Test server ports (for parallel testing)
export UVEDDI_TEST_PORT_BASE=8000
```

### Test Database

Each test creates an isolated SQLite database in a temporary directory. The database is automatically cleaned up after test completion.

```rust
// Example test setup
let temp_dir = tempfile::tempdir()?;
let db_path = temp_dir.path().join("test.db");
let database = Arc::new(Database::new(db_path.to_str().unwrap())?);
database.init_schema().await?;
```

## Writing New Tests

### Test Structure Template

```rust
#[tokio::test]
async fn test_new_endpoint() -> TestResult {
    // Setup
    let client = ApiTestClient::new().await?;
    
    // Optional: Insert test data
    let test_data = client.insert_test_analysis().await?;
    
    // Execute
    let (status, headers, body) = client.get("/api/v1/new-endpoint").await?;
    
    // Assert
    assert_eq!(status, StatusCode::OK);
    assert!(body["expected_field"].is_string());
    
    Ok(())
}
```

### Adding Security Tests

When adding security-related tests, use feature gating:

```rust
#[cfg(feature = "security")]
#[tokio::test]
async fn test_security_feature() -> TestResult {
    // Security test implementation
    Ok(())
}
```

### Adding Authentication Tests

When authentication is implemented, use the ignore attribute:

```rust
#[tokio::test]
#[ignore] // Remove when authentication is implemented
async fn test_protected_endpoint() -> TestResult {
    let client = ApiTestClient::new().await?;
    
    let (status, _, _) = client.authenticated_request(
        "GET", 
        "/api/v1/protected", 
        None
    ).await?;
    
    assert_eq!(status, StatusCode::OK);
    Ok(())
}
```

## Test Utilities

### ApiTestClient

The `ApiTestClient` provides convenient methods for testing:

```rust
// Create client
let client = ApiTestClient::new().await?;

// Make requests
let (status, headers, body) = client.get("/api/v1/reports").await?;
let (status, headers, body) = client.post("/api/v1/reports", Some(json_body), None).await?;

// Authenticated requests (when auth is implemented)
let (status, headers, body) = client.authenticated_request("GET", "/api/v1/admin", None).await?;

// Insert test data
let run_id = client.insert_test_analysis().await?;
```

### GraphQLTestClient

The `GraphQLTestClient` provides GraphQL-specific testing:

```rust
// Create client
let client = GraphQLTestClient::new().await?;

// Execute queries
let response = client.query("{ __schema { types { name } } }").await?;

// Execute with variables
let response = client.query_with_variables(query, variables).await?;

// Test subscriptions
let stream = client.subscribe("subscription { updates }").await?;
```

## Test Data Management

### Fixtures

Use the fixture utilities for consistent test data:

```rust
use crate::test_utils::fixtures::*;

// Create sample analysis data
let analysis_fixture = AnalysisFixture::new()
    .with_issues(5)
    .with_security_issues(2)
    .build();
```

### Mocks

Use mocks for external dependencies:

```rust
use crate::test_utils::mocks::*;

// Mock AI provider
let mock_ai = MockAiProvider::new()
    .with_response("Mock AI explanation")
    .build();
```

## Continuous Integration

### GitHub Actions Configuration

```yaml
name: API Integration Tests
on: [push, pull_request]

jobs:
  api-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      # Fast development tests
      - name: Run API tests (dev)
        run: cargo test --features=dev-core api_integration
        
      # Full production tests
      - name: Run API tests (production)
        run: cargo test --features=production api_integration
        
      # GraphQL tests
      - name: Run GraphQL tests
        run: cargo test --features=production graphql_integration
```

### Test Performance Monitoring

Track test execution times:

```bash
# Generate test timing report
cargo test --features=production -- -Z unstable-options --report-time

# Run with timing and save to file
cargo test api_integration -- --nocapture 2>&1 | tee test_results.txt
```

## Troubleshooting

### Common Issues

#### Database Lock Errors
```bash
# Run tests serially to avoid database conflicts
cargo test api_integration -- --test-threads=1
```

#### Port Conflicts
```bash
# Use different port ranges for parallel test runs
UVEDDI_TEST_PORT_BASE=9000 cargo test api_integration
```

#### Memory Issues
```bash
# Use minimal features for faster, lighter tests
cargo test --features=dev-minimal api_integration
```

#### GraphQL Schema Errors
```bash
# Ensure GraphQL features are enabled
cargo test --features=production graphql_integration
```

### Debug Output

Enable detailed logging for debugging:

```bash
# Enable debug logging
RUST_LOG=debug cargo test api_integration -- --nocapture

# Enable trace logging for specific modules
RUST_LOG=uveddi::api=trace cargo test api_integration -- --nocapture
```

### Test Isolation Issues

If tests are interfering with each other:

```bash
# Run with single thread
cargo test api_integration -- --test-threads=1

# Run specific test in isolation
cargo test test_specific_function -- --exact
```

## Extending the Test Suite

### Adding New Endpoint Tests

1. **Identify the endpoint** to test in `src/api/rest.rs` or `src/api/server.rs`
2. **Create test function** following the naming convention `test_endpoint_scenario`
3. **Use appropriate test client** (`ApiTestClient` for REST, `GraphQLTestClient` for GraphQL)
4. **Test both success and failure scenarios**
5. **Validate response structure and content**
6. **Add error handling tests**

### Adding Performance Tests

1. **Create performance test module** under the appropriate test category
2. **Use timing measurements** with `std::time::Instant`
3. **Test concurrent access** with `tokio::spawn` and `Arc<Client>`
4. **Validate response times** and resource usage
5. **Consider load testing** with multiple requests

### Adding Security Tests

1. **Use feature gating** with `#[cfg(feature = "security")]`
2. **Test input validation** and sanitization
3. **Test authentication** and authorization
4. **Test rate limiting** when implemented
5. **Test CORS** and security headers

## Best Practices

### Test Organization
- **Group related tests** in modules
- **Use descriptive test names** that explain the scenario
- **Keep tests focused** on single functionality
- **Use setup and teardown** helpers for common operations

### Assertions
- **Use specific assertions** rather than generic ones
- **Validate response structure** and content types
- **Check error messages** for meaningful content
- **Test edge cases** and boundary conditions

### Test Data
- **Use isolated test databases** for each test
- **Create minimal test data** needed for the test
- **Clean up resources** properly (automatic with TempDir)
- **Use realistic test data** that matches production patterns

### Performance
- **Run tests in parallel** when possible
- **Use appropriate feature sets** for faster compilation
- **Minimize test database** setup and teardown
- **Cache common test fixtures** when appropriate

This comprehensive testing framework ensures the reliability and robustness of the Uveddi API layer, supporting both current functionality and future extensibility.