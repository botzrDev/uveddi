# Database Refactoring Documentation

## Overview

This document provides comprehensive documentation for the database refactoring project completed in the Uveddi codebase. The refactoring transformed the database layer from a monolithic CRUD-based architecture to a clean repository pattern with proper separation of concerns.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Repository Pattern Implementation](#repository-pattern-implementation)
3. [Migration System](#migration-system)
4. [API Reference](#api-reference)
5. [Migration Guide](#migration-guide)
6. [Examples](#examples)
7. [Testing](#testing)
8. [Performance Considerations](#performance-considerations)

## Architecture Overview

### Before Refactoring

The original architecture had:
- **Monolithic Database Struct**: Single `Database` struct in `src/database/crud.rs` with 971 lines
- **Direct SQL Queries**: Business logic mixed with database queries
- **Tight Coupling**: Models directly tied to database representation
- **No Abstraction**: Services directly calling database methods

### After Refactoring

The new architecture features:
- **Repository Pattern**: Clean abstraction between domain and persistence
- **Dependency Injection**: Factory pattern for repository creation
- **Migration System**: Versioned database schema management
- **Error Handling**: Standardized error types across repositories
- **Async/Await Support**: Proper async trait implementation

### Component Structure

```
src/database/
├── connection/           # Connection pooling and providers
│   ├── config.rs        # Database configuration
│   ├── mod.rs           # Connection manager
│   ├── pool.rs          # Connection pool implementation
│   └── providers/       # SQLite and PostgreSQL providers
├── migrations/          # Schema migration system
│   ├── mod.rs           # Migration types and registry
│   ├── runner.rs        # Migration execution engine
│   ├── schema.rs        # Schema validation
│   └── sql/             # SQL migration files
├── models/              # Domain models (isolated)
│   ├── analysis.rs      # AnalysisRun model
│   ├── cache.rs         # Cache entry model
│   ├── project.rs       # Project model
│   └── ...              # Other domain models
├── repositories/        # Repository pattern implementation
│   ├── traits.rs        # Repository interfaces
│   ├── factory.rs       # Repository factory
│   ├── errors.rs        # Error types
│   └── sqlite/          # SQLite implementations
└── crud.rs              # Legacy CRUD (deprecated)
```

## Repository Pattern Implementation

### Core Traits

The repository pattern is built on trait interfaces that define contracts for data access:

```rust
// Base repository trait
#[async_trait]
pub trait Repository: Send + Sync {
    type Entity;

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Self::Entity>>;
    async fn find_all(&self) -> RepositoryResult<Vec<Self::Entity>>;
    async fn create(&self, entity: Self::Entity) -> RepositoryResult<Self::Entity>;
    async fn update(&self, entity: Self::Entity) -> RepositoryResult<Self::Entity>;
    async fn delete(&self, id: Uuid) -> RepositoryResult<bool>;
}

// Specialized repository traits
#[async_trait]
pub trait ProjectRepository: Repository<Entity = Project> {
    async fn find_by_path(&self, path: &str) -> RepositoryResult<Option<Project>>;
    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<Project>>;
    async fn search(&self, query: &str) -> RepositoryResult<Vec<Project>>;
}
```

### Repository Factory

The factory pattern provides dependency injection and repository creation:

```rust
pub trait RepositoryFactory: Send + Sync {
    fn create_project_repository(&self) -> Box<dyn ProjectRepository>;
    fn create_analysis_repository(&self) -> Box<dyn AnalysisRepository>;
    fn create_cache_repository(&self) -> Box<dyn CacheRepository>;
    // ... other repository creation methods
}

pub struct SqliteRepositoryFactory {
    pool: Arc<ConnectionPool>,
}

impl SqliteRepositoryFactory {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}
```

### Repository Manager

The manager provides caching and lifecycle management for repository instances:

```rust
pub struct RepositoryManager {
    factory: Arc<dyn RepositoryFactory>,
    project_repo: Arc<RwLock<Option<Arc<dyn ProjectRepository>>>>,
    analysis_repo: Arc<RwLock<Option<Arc<dyn AnalysisRepository>>>>,
    // ... other cached repositories
}

impl RepositoryManager {
    pub async fn get_project_repository(&self) -> Arc<dyn ProjectRepository> {
        // Returns cached instance or creates new one
    }
}
```

## Migration System

### Migration Structure

The migration system provides versioned schema management with rollback support:

```rust
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
    pub checksum: String,
}

pub struct MigrationRunner {
    pool: Arc<ConnectionPool>,
    registry: MigrationRegistry,
}

impl MigrationRunner {
    pub async fn run_pending_migrations(&self) -> Result<Vec<MigrationResult>, MigrationError>;
    pub async fn rollback_migration(&self, version: u32) -> Result<(), MigrationError>;
    pub async fn get_current_version(&self) -> Result<u32, MigrationError>;
}
```

### Migration Files

All migrations are stored in `src/database/migrations/sql/`:

1. **001_create_cache_table.sql** - Cache storage with TTL
2. **002_create_metrics_table.sql** - Performance metrics
3. **003_create_events_table.sql** - System events
4. **004_create_issues_table.sql** - Code issues
5. **005_create_dependencies_table.sql** - Dependency graph
6. **006_create_security_findings_table.sql** - Security vulnerabilities
7. **007_create_technical_debt_table.sql** - Technical debt tracking

### Running Migrations

```rust
// Initialize migration runner
let runner = MigrationRunner::new(pool).await?;

// Run all pending migrations
let results = runner.run_pending_migrations().await?;

// Check current version
let version = runner.get_current_version().await?;

// Rollback specific migration
runner.rollback_migration(5).await?;
```

## API Reference

### Database Initialization

```rust
// Old way (deprecated)
let database = Database::new(Some(path))?;

// New way (recommended)
let config = DatabaseConfig {
    connection_string: "sqlite://uveddi.db".to_string(),
    pool_config: PoolConfig::default(),
};

let pool = Arc::new(ConnectionPool::new(config.pool_config));
let factory = Arc::new(SqliteRepositoryFactory::new(pool));
let database = Database::new_with_repositories(Some(config), factory).await?;
```

### Repository Usage

```rust
// Get repository from database
let project_repo = database.get_repository_manager()
    .expect("Repository manager not initialized")
    .get_project_repository()
    .await;

// Create a project
let project = Project {
    id: Uuid::new_v4(),
    name: "My Project".to_string(),
    path: "/path/to/project".to_string(),
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

let created = project_repo.create(project).await?;

// Find by ID
let found = project_repo.find_by_id(created.id).await?;

// Find recent projects
let recent = project_repo.find_recent(10).await?;

// Search projects
let results = project_repo.search("query").await?;
```

### Error Handling

```rust
use crate::database::repositories::errors::{RepositoryError, RepositoryResult};

// Repository methods return RepositoryResult<T>
match project_repo.find_by_id(id).await {
    Ok(Some(project)) => println!("Found: {}", project.name),
    Ok(None) => println!("Project not found"),
    Err(RepositoryError::Database { message, .. }) => {
        eprintln!("Database error: {}", message);
    }
    Err(RepositoryError::Validation(msg)) => {
        eprintln!("Validation error: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Migration Guide

### For Existing Code

Replace direct database calls with repository pattern:

#### Before
```rust
// Direct database access
let database = Database::new(Some(path))?;
let project = database.get_project_by_id(id)?;
let analysis = database.create_analysis_run(&project_path)?;
database.store_issues(&analysis_id, issues)?;
```

#### After
```rust
// Repository pattern
let config = DatabaseConfig::from_path(path)?;
let pool = Arc::new(ConnectionPool::new(config.pool_config));
let factory = Arc::new(SqliteRepositoryFactory::new(pool));
let database = Database::new_with_repositories(Some(config), factory).await?;

let repo_manager = database.get_repository_manager().unwrap();
let project_repo = repo_manager.get_project_repository().await;
let analysis_repo = repo_manager.get_analysis_repository().await;

let project = project_repo.find_by_id(id).await?;
let analysis = analysis_repo.create(analysis_data).await?;
```

### For New Code

Always use the repository pattern:

```rust
// In service layer
pub struct ProjectService {
    repository: Arc<dyn ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_project(&self, id: Uuid) -> Result<Option<Project>> {
        self.repository.find_by_id(id).await
    }
}

// In API handler
pub async fn list_projects(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse> {
    let repo_manager = app_state.database
        .get_repository_manager()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let project_repo = repo_manager.get_project_repository().await;
    let projects = project_repo.find_all().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(projects))
}
```

## Examples

### Complete Example: Project Management

```rust
use uveddi::database::{
    DatabaseConfig,
    connection::pool::{ConnectionPool, PoolConfig},
    repositories::{
        factory::SqliteRepositoryFactory,
        traits::ProjectRepository,
    },
    Database,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database with repositories
    let config = DatabaseConfig {
        connection_string: "sqlite://./uveddi.db".to_string(),
        pool_config: PoolConfig {
            max_connections: 10,
            min_idle: 2,
            max_lifetime: Some(Duration::from_secs(3600)),
            idle_timeout: Some(Duration::from_secs(600)),
        },
    };

    let pool = Arc::new(ConnectionPool::new(config.pool_config));
    let factory = Arc::new(SqliteRepositoryFactory::new(Arc::clone(&pool)));
    let database = Database::new_with_repositories(Some(config), factory).await?;

    // Run migrations
    let migration_runner = database.get_migration_runner()?;
    migration_runner.run_pending_migrations().await?;

    // Get repository
    let repo_manager = database.get_repository_manager().unwrap();
    let project_repo = repo_manager.get_project_repository().await;

    // Create a new project
    let project = Project {
        id: Uuid::new_v4(),
        name: "My Application".to_string(),
        path: "/workspace/my-app".to_string(),
        language: Some("rust".to_string()),
        framework: Some("actix-web".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created_project = project_repo.create(project).await?;
    println!("Created project: {}", created_project.name);

    // Find the project
    let found = project_repo.find_by_id(created_project.id).await?;
    match found {
        Some(p) => println!("Found project: {}", p.name),
        None => println!("Project not found"),
    }

    // List recent projects
    let recent = project_repo.find_recent(5).await?;
    for project in recent {
        println!("Recent: {} at {}", project.name, project.path);
    }

    Ok(())
}
```

### Transaction Example

```rust
use uveddi::database::repositories::traits::UnitOfWork;

// Start a transaction
let uow = repo_manager.begin_transaction().await?;

// Perform multiple operations
let project = uow.project_repository().create(project_data).await?;
let analysis = uow.analysis_repository().create(analysis_data).await?;
uow.issue_repository().create_batch(issues).await?;

// Commit or rollback
if everything_ok {
    uow.commit().await?;
} else {
    uow.rollback().await?;
}
```

## Testing

### Unit Testing Repositories

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        ProjectRepo {}

        #[async_trait]
        impl ProjectRepository for ProjectRepo {
            async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Project>>;
            async fn create(&self, project: Project) -> RepositoryResult<Project>;
        }
    }

    #[tokio::test]
    async fn test_project_service() {
        let mut mock_repo = MockProjectRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(test_project())));

        let service = ProjectService::new(Arc::new(mock_repo));
        let result = service.get_project(Uuid::new_v4()).await;

        assert!(result.is_ok());
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_repository_integration() {
    // Create in-memory database
    let config = DatabaseConfig::in_memory();
    let pool = Arc::new(ConnectionPool::new(config.pool_config));
    let factory = Arc::new(SqliteRepositoryFactory::new(pool));
    let database = Database::new_with_repositories(Some(config), factory).await.unwrap();

    // Run migrations
    let runner = database.get_migration_runner().unwrap();
    runner.run_pending_migrations().await.unwrap();

    // Test repository operations
    let repo_manager = database.get_repository_manager().unwrap();
    let project_repo = repo_manager.get_project_repository().await;

    // Create and verify
    let project = test_project();
    let created = project_repo.create(project.clone()).await.unwrap();
    assert_eq!(created.name, project.name);

    // Find and verify
    let found = project_repo.find_by_id(created.id).await.unwrap();
    assert!(found.is_some());
}
```

## Performance Considerations

### Connection Pooling

The repository pattern uses connection pooling to optimize database access:

```rust
pub struct PoolConfig {
    pub max_connections: u32,      // Maximum connections in pool
    pub min_idle: u32,              // Minimum idle connections
    pub max_lifetime: Option<Duration>, // Connection lifetime
    pub idle_timeout: Option<Duration>, // Idle connection timeout
}

// Recommended settings
let config = PoolConfig {
    max_connections: 20,
    min_idle: 5,
    max_lifetime: Some(Duration::from_secs(3600)),
    idle_timeout: Some(Duration::from_secs(600)),
};
```

### Async/Await Optimization

Repositories use `tokio::task::spawn_blocking` for CPU-bound operations:

```rust
pub async fn complex_query(&self) -> RepositoryResult<Vec<Entity>> {
    let pool = Arc::clone(&self.pool);

    tokio::task::spawn_blocking(move || {
        let conn = pool.get_connection()?;
        // Perform blocking database operations
        // This runs in a separate thread pool
        Ok(results)
    })
    .await
    .map_err(|e| RepositoryError::TaskJoin(e.to_string()))?
}
```

### Caching Strategy

The RepositoryManager implements caching for frequently accessed repositories:

```rust
// Repository instances are cached and reused
let project_repo = repo_manager.get_project_repository().await; // Created
let project_repo2 = repo_manager.get_project_repository().await; // Cached
```

### Query Optimization

All migration scripts include proper indexing:

```sql
CREATE INDEX idx_projects_path ON projects(path);
CREATE INDEX idx_projects_created_at ON projects(created_at);
CREATE INDEX idx_analysis_runs_project_id ON analysis_runs(project_id);
CREATE INDEX idx_issues_analysis_run_id ON issues(analysis_run_id);
```

## Troubleshooting

### Common Issues

1. **Async/Send Trait Errors**
   - Solution: Use `spawn_blocking` for database operations
   - Ensure all repository implementations are Send + Sync

2. **Connection Pool Exhaustion**
   - Increase `max_connections` in PoolConfig
   - Ensure connections are properly returned to pool

3. **Migration Failures**
   - Check migration checksums match
   - Verify SQL syntax for target database
   - Use rollback for failed migrations

4. **Performance Issues**
   - Enable connection pooling
   - Add appropriate database indexes
   - Use repository caching via RepositoryManager

## Conclusion

The database refactoring project successfully transformed Uveddi's data layer into a modern, maintainable architecture. The repository pattern provides:

- **Clean separation of concerns** between business logic and data access
- **Testability** through dependency injection and mockable interfaces
- **Flexibility** to switch between database providers
- **Scalability** through connection pooling and caching
- **Maintainability** with standardized patterns and error handling

The migration system ensures smooth schema evolution, while the comprehensive error handling provides robust operation in production environments.