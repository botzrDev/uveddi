# GraphQL API Implementation Plan for Uveddi

## Overview

This document outlines the implementation plan for integrating a GraphQL API into Uveddi's architecture, providing a flexible query interface for analysis results. The API will enable efficient data retrieval, reduce over-fetching, and improve developer experience for consumers.

## Architecture Integration

### 1. Core Components

The GraphQL API consists of several key components:

- **Schema Definition** (`src/api/graphql/schema.graphql`) - Complete type definitions
- **Resolvers** (`src/api/graphql/resolvers.rs`) - Query and mutation handlers  
- **Type System** (`src/api/graphql/types.rs`) - Rust type definitions with async-graphql
- **Data Loaders** (`src/api/graphql/loaders.rs`) - N+1 query prevention
- **Pagination** (`src/api/graphql/pagination.rs`) - Cursor-based pagination utilities
- **Subscriptions** (`src/api/graphql/subscriptions.rs`) - Real-time event streaming
- **Context** (`src/api/graphql/context.rs`) - Shared state management

### 2. Technology Stack

- **async-graphql** - Rust GraphQL framework with strong type safety
- **tokio** - Async runtime (already integrated)
- **serde** - Serialization/deserialization (already integrated)
- **chrono** - Date/time handling (already integrated)
- **base64** - Cursor encoding for pagination

## Implementation Phases

### Phase 1: Foundation Setup (Week 1-2)

#### Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
async-graphql = "7.0"
async-graphql-warp = "7.0"  # or axum integration
tokio-stream = "0.1"
base64 = "0.22"

[features]
graphql-api = ["async-graphql", "async-graphql-warp", "tokio-stream"]
```

#### Database Integration
1. **Extend DatabaseManager** (`src/database/mod.rs`):
   ```rust
   impl DatabaseManager {
       // Batch loading methods for DataLoaders
       pub async fn get_projects_by_ids(&self, ids: &[i64]) -> Result<Vec<Project>, DatabaseError> { }
       pub async fn get_analysis_runs_by_ids(&self, ids: &[i64]) -> Result<Vec<AnalysisRun>, DatabaseError> { }
       pub async fn get_issues_by_analysis_run_ids(&self, ids: &[i64]) -> Result<HashMap<i64, Vec<ArchitecturalIssue>>, DatabaseError> { }
       
       // Pagination-aware queries
       pub async fn get_projects_paginated(&self, params: PaginationParams, filters: ProjectFilters) -> Result<PaginationResult<Project>, DatabaseError> { }
       pub async fn get_analysis_runs_paginated(&self, params: PaginationParams, filters: AnalysisRunFilters) -> Result<PaginationResult<AnalysisRun>, DatabaseError> { }
       pub async fn get_issues_paginated(&self, params: PaginationParams, filters: IssueFilters) -> Result<PaginationResult<ArchitecturalIssue>, DatabaseError> { }
   }
   ```

2. **Add Full-Text Search** support:
   ```sql
   -- Add search indexes to support the search query
   CREATE INDEX idx_issues_description_fts ON architectural_issues USING gin(to_tsvector('english', description));
   CREATE INDEX idx_files_path_fts ON files USING gin(to_tsvector('english', file_path));
   ```

### Phase 2: Core API Implementation (Week 3-4)

#### Server Integration
Create HTTP server wrapper (`src/api/server.rs`):
```rust
use async_graphql_warp::{GraphQLBadRequest, Response};
use std::convert::Infallible;
use warp::{Filter, Rejection};

pub async fn start_graphql_server(
    schema: UveddiSchema,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let graphql_post = async_graphql_warp::graphql(schema.clone())
        .and_then(|(schema, request): (UveddiSchema, async_graphql::Request)| async move {
            Ok::<_, Infallible>(Response::from(schema.execute(request).await))
        });

    let graphql_playground = warp::path("playground")
        .and(warp::get())
        .map(|| {
            warp::http::Response::builder()
                .header("content-type", "text/html")
                .body(async_graphql::http::playground_source(
                    async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
                ))
        });

    let routes = graphql_playground
        .or(graphql_post)
        .recover(|err: Rejection| async move {
            if let Some(GraphQLBadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    warp::http::StatusCode::BAD_REQUEST,
                ));
            }
            Ok(warp::reply::with_status(
                "NOT_FOUND".to_string(),
                warp::http::StatusCode::NOT_FOUND,
            ))
        });

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    Ok(())
}
```

#### CLI Integration
Extend CLI (`src/cli/mod.rs`) to support GraphQL server mode:
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    #[cfg(feature = "graphql-api")]
    Serve {
        /// Port to serve GraphQL API on
        #[arg(long, default_value = "4000")]
        port: u16,
        
        /// Enable GraphQL playground
        #[arg(long)]
        playground: bool,
        
        /// Database path
        #[arg(long, default_value = "uveddi.db")]
        database: PathBuf,
    },
}
```

### Phase 3: Resolver Implementation (Week 5-6)

#### Complete Resolver Functions
Implement all TODO placeholders in `resolvers.rs` with:
- Database queries using the extended DatabaseManager
- Error handling with proper GraphQL error types
- Field-level permissions and validation
- Efficient data loading with DataLoaders

#### Complex Resolvers
1. **Analytics Query** - Aggregate statistics across projects
2. **Search Query** - Full-text search across multiple entity types
3. **Dependency Graph** - Build and return dependency relationships
4. **AST Queries** - On-demand AST node retrieval (expensive operations)

### Phase 4: Subscription System (Week 7-8)

#### Real-time Integration
1. **Extend AnalysisEngine** to publish events:
   ```rust
   impl AnalysisEngine {
       pub fn set_event_broadcaster(&mut self, broadcaster: Arc<EventBroadcaster>) {
           self.event_broadcaster = Some(broadcaster);
       }
       
       // In analysis methods, publish progress updates
       async fn analyze_file(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
           // ... analysis logic
           
           if let Some(broadcaster) = &self.event_broadcaster {
               broadcaster.publish_analysis_progress(run_id, progress_update);
           }
           
           // ... continue analysis
       }
   }
   ```

2. **WebSocket Support** for subscriptions (requires additional dependencies)

### Phase 5: Performance Optimization (Week 9-10)

#### Query Optimization
1. **Implement DataLoader batch functions** in `loaders.rs`
2. **Add query complexity analysis** to prevent expensive queries
3. **Implement caching layer** for frequently accessed data
4. **Add database query optimization** with proper indexes

#### Memory Management
1. **Integrate with existing memory optimization** features (UV-210, UV-26)
2. **Implement result streaming** for large datasets
3. **Add query timeout handling**

## Migration Strategy

### Current State Analysis
Uveddi currently operates as a CLI tool with:
- SQLite database for persistence
- Direct database access patterns
- JSON/Markdown output formats
- No existing web API

### Migration Approach

#### 1. Additive Integration (Recommended)
- **No breaking changes** to existing CLI functionality
- GraphQL API as an **optional feature** (`--features graphql-api`)
- **Parallel operation** - CLI and API can run simultaneously
- **Shared database** - both access the same SQLite database

#### 2. Progressive Enhancement
```bash
# Phase 1: Enable GraphQL server alongside CLI
cargo run --features graphql-api -- serve --port 4000

# Phase 2: Use GraphQL for specific operations  
cargo run --features graphql-api -- analyze /path/to/code --serve-results

# Phase 3: Full API-first mode (future consideration)
```

#### 3. Data Model Compatibility
The GraphQL schema is designed to be **backward compatible** with existing database models:
- Direct mapping from existing `DatabaseManager` types
- Conversion traits implemented for seamless integration
- No schema changes required to existing database

### Deployment Considerations

#### Development Environment
```bash
# Install additional dependencies
cargo install --features graphql-api

# Run in development mode with playground
cargo run --features graphql-api -- serve --playground --port 4000
```

#### Production Environment
1. **Feature flag control** - enable GraphQL only when needed
2. **Security considerations**:
   - Disable introspection in production
   - Implement rate limiting
   - Add authentication/authorization layer
3. **Monitoring integration** with existing observability features

## Performance Characteristics

### Query Efficiency
- **N+1 Prevention**: DataLoaders batch database queries
- **Cursor Pagination**: Efficient for large result sets
- **Field Selection**: Only fetch requested data from database
- **Query Complexity**: Prevent expensive operations

### Memory Usage
- **Streaming Results**: Large datasets don't load entirely into memory
- **Connection Pooling**: Reuse database connections
- **Cache Integration**: Leverage existing Uveddi caching strategies

### Scalability
- **Horizontal Scaling**: Stateless GraphQL resolvers
- **Database Optimization**: Proper indexing for common query patterns
- **Incremental Loading**: Support for partial data loading

## Security Considerations

### Query Security
- **Query Depth Limiting**: Prevent deeply nested queries (max depth: 15)
- **Query Complexity Analysis**: Prevent expensive operations (max complexity: 1000)
- **Timeout Protection**: All queries timeout after 30 seconds
- **Field-level Authorization**: Control access to sensitive data

### Data Protection
- **Input Validation**: All mutations validate input data
- **SQL Injection Prevention**: Parameterized queries only
- **Error Handling**: No sensitive information in error messages
- **Audit Logging**: Track all mutation operations

## Testing Strategy

### Unit Tests
- **Resolver Tests**: Mock database and test individual resolvers
- **Type Conversion Tests**: Verify database model → GraphQL type conversion
- **Pagination Tests**: Cursor encoding/decoding and edge cases
- **Validation Tests**: Input validation and error handling

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_project_query() {
        let db = setup_test_database().await;
        let engine = setup_test_engine().await;
        let schema = create_schema(db, engine);
        
        let query = r#"
            query {
                projects(first: 10) {
                    edges {
                        node {
                            id
                            name
                            analysisRuns(first: 5) {
                                edges {
                                    node {
                                        status
                                        totalIssuesFound
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;
        
        let result = schema.execute(query).await;
        assert!(result.errors.is_empty());
    }
}
```

### Performance Tests
- **Load Testing**: Simulate concurrent GraphQL queries
- **Memory Profiling**: Ensure no memory leaks in long-running operations
- **Database Performance**: Monitor query execution times
- **Subscription Stress Testing**: High-frequency real-time updates

## Documentation

### API Documentation
- **Auto-generated Schema Docs**: GraphQL introspection provides complete type information
- **Query Examples**: Common use cases and query patterns
- **Migration Guide**: Help users transition from CLI to API usage
- **Performance Guidelines**: Best practices for efficient queries

### Developer Experience
- **GraphQL Playground**: Interactive query exploration (development only)
- **TypeScript Types**: Generate client types from schema
- **SDK Generation**: Auto-generate client libraries for popular languages

## Future Enhancements

### Potential Extensions
1. **Batch Mutations**: Allow multiple operations in single request
2. **File Upload**: Support for uploading code files for analysis
3. **Webhook Integration**: Notify external systems of analysis completion
4. **Multi-tenancy**: Support for multiple isolated projects
5. **Advanced Analytics**: Machine learning insights through GraphQL

### Integration Opportunities
1. **VS Code Extension**: Use GraphQL API for IDE integration
2. **CI/CD Plugins**: Automated analysis through API calls
3. **Dashboard Applications**: Rich web UIs built on GraphQL
4. **Mobile Applications**: Native apps consuming analysis data

## Conclusion

The GraphQL API implementation provides Uveddi with a modern, flexible interface for accessing analysis results while maintaining full backward compatibility with existing CLI functionality. The phased approach ensures minimal risk and allows for incremental adoption.

Key benefits:
- **Flexible Data Fetching**: Clients request exactly the data they need
- **Type Safety**: Strong typing throughout the stack
- **Real-time Capabilities**: Live updates during analysis
- **Developer Experience**: Self-documenting API with tooling support
- **Performance**: Efficient querying with N+1 prevention and pagination
- **Extensibility**: Easy to add new fields and operations

The implementation leverages Uveddi's existing architecture while providing a modern API layer that can support a wide range of client applications and integrations.