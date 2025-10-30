# Repository Pattern Implementation Guide

## Introduction

This guide provides detailed information about the repository pattern implementation in Uveddi, including design decisions, implementation details, and best practices.

## Design Principles

### 1. Separation of Concerns

The repository pattern separates:
- **Domain Models**: Pure business entities without database concerns
- **Repository Interfaces**: Contracts defining data access operations
- **Repository Implementations**: Concrete database operations
- **Business Logic**: Services that use repositories without knowing database details

### 2. Dependency Inversion

High-level modules (services) depend on abstractions (repository traits), not concrete implementations:

```rust
// Service depends on trait, not implementation
pub struct ProjectService {
    repository: Arc<dyn ProjectRepository>,  // Abstract dependency
}
```

### 3. Single Responsibility

Each repository handles one aggregate root:
- `ProjectRepository` manages Project entities
- `AnalysisRepository` manages AnalysisRun entities
- `IssueRepository` manages Issue entities

## Repository Trait Hierarchy

### Base Repository Trait

All repositories implement the base `Repository` trait:

```rust
#[async_trait]
pub trait Repository: Send + Sync {
    type Entity;

    /// Find entity by ID
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Self::Entity>>;

    /// Find all entities
    async fn find_all(&self) -> RepositoryResult<Vec<Self::Entity>>;

    /// Create new entity
    async fn create(&self, entity: Self::Entity) -> RepositoryResult<Self::Entity>;

    /// Update existing entity
    async fn update(&self, entity: Self::Entity) -> RepositoryResult<Self::Entity>;

    /// Delete entity by ID
    async fn delete(&self, id: Uuid) -> RepositoryResult<bool>;
}
```

### Specialized Repository Traits

Each entity type has a specialized repository trait with domain-specific operations:

```rust
#[async_trait]
pub trait ProjectRepository: Repository<Entity = Project> {
    /// Find project by file path
    async fn find_by_path(&self, path: &str) -> RepositoryResult<Option<Project>>;

    /// Find recent projects
    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<Project>>;

    /// Search projects by query
    async fn search(&self, query: &str) -> RepositoryResult<Vec<Project>>;

    /// Count total projects
    async fn count(&self) -> RepositoryResult<usize>;
}

#[async_trait]
pub trait AnalysisRepository: Repository<Entity = AnalysisRun> {
    /// Find analyses for a project
    async fn find_by_project(&self, project_id: Uuid) -> RepositoryResult<Vec<AnalysisRun>>;

    /// Find recent analyses
    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<AnalysisRun>>;

    /// Find analyses with issues
    async fn find_with_issues(&self) -> RepositoryResult<Vec<AnalysisRun>>;

    /// Get analysis statistics
    async fn get_statistics(&self) -> RepositoryResult<AnalysisStatistics>;
}
```

## Implementation Details

### SQLite Repository Implementation

Example implementation for `SqliteProjectRepository`:

```rust
pub struct SqliteProjectRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteProjectRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    /// Convert database row to domain model
    fn row_to_project(row: &Row) -> rusqlite::Result<Project> {
        Ok(Project {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            name: row.get(1)?,
            path: row.get(2)?,
            language: row.get(3)?,
            framework: row.get(4)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .unwrap()
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                .unwrap()
                .with_timezone(&Utc),
        })
    }
}

#[async_trait]
impl Repository for SqliteProjectRepository {
    type Entity = Project;

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Self::Entity>> {
        let pool = Arc::clone(&self.pool);
        let id_str = id.to_string();

        // Use spawn_blocking for thread-safe database operations
        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()
                .map_err(|e| RepositoryError::Connection(e.to_string()))?;

            let mut stmt = conn.prepare(
                "SELECT id, name, path, language, framework, created_at, updated_at
                 FROM projects WHERE id = ?1"
            )?;

            let result = stmt.query_row([id_str], Self::row_to_project).optional()?;
            Ok(result)
        })
        .await
        .map_err(|e| RepositoryError::TaskJoin(e.to_string()))?
    }

    async fn create(&self, entity: Self::Entity) -> RepositoryResult<Self::Entity> {
        let pool = Arc::clone(&self.pool);

        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()
                .map_err(|e| RepositoryError::Connection(e.to_string()))?;

            conn.execute(
                "INSERT INTO projects (id, name, path, language, framework, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    entity.id.to_string(),
                    entity.name,
                    entity.path,
                    entity.language,
                    entity.framework,
                    entity.created_at.to_rfc3339(),
                    entity.updated_at.to_rfc3339(),
                ],
            )?;

            Ok(entity)
        })
        .await
        .map_err(|e| RepositoryError::TaskJoin(e.to_string()))?
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn find_by_path(&self, path: &str) -> RepositoryResult<Option<Project>> {
        let pool = Arc::clone(&self.pool);
        let path_owned = path.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()
                .map_err(|e| RepositoryError::Connection(e.to_string()))?;

            let mut stmt = conn.prepare(
                "SELECT id, name, path, language, framework, created_at, updated_at
                 FROM projects WHERE path = ?1"
            )?;

            let result = stmt.query_row([path_owned], Self::row_to_project).optional()?;
            Ok(result)
        })
        .await
        .map_err(|e| RepositoryError::TaskJoin(e.to_string()))?
    }

    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<Project>> {
        let pool = Arc::clone(&self.pool);

        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()
                .map_err(|e| RepositoryError::Connection(e.to_string()))?;

            let mut stmt = conn.prepare(
                "SELECT id, name, path, language, framework, created_at, updated_at
                 FROM projects
                 ORDER BY created_at DESC
                 LIMIT ?1"
            )?;

            let projects = stmt.query_map([limit], Self::row_to_project)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(projects)
        })
        .await
        .map_err(|e| RepositoryError::TaskJoin(e.to_string()))?
    }
}
```

### PostgreSQL Implementation

PostgreSQL repositories follow the same pattern with SQL dialect differences:

```rust
pub struct PostgresProjectRepository {
    pool: Arc<ConnectionPool>,
}

#[async_trait]
impl ProjectRepository for PostgresProjectRepository {
    async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<Project>> {
        let pool = Arc::clone(&self.pool);

        tokio::task::spawn_blocking(move || {
            let conn = pool.get_connection()
                .map_err(|e| RepositoryError::Connection(e.to_string()))?;

            // PostgreSQL specific SQL
            let query = "
                SELECT id, name, path, language, framework, created_at, updated_at
                FROM projects
                ORDER BY created_at DESC
                LIMIT $1
            ";

            // Implementation details...
            Ok(projects)
        })
        .await
        .map_err(|e| RepositoryError::TaskJoin(e.to_string()))?
    }
}
```

## Factory Pattern

### Repository Factory Trait

The factory creates repository instances:

```rust
pub trait RepositoryFactory: Send + Sync {
    fn create_project_repository(&self) -> Box<dyn ProjectRepository>;
    fn create_analysis_repository(&self) -> Box<dyn AnalysisRepository>;
    fn create_cache_repository(&self) -> Box<dyn CacheRepository>;
    fn create_metrics_repository(&self) -> Box<dyn MetricsRepository>;
    fn create_event_repository(&self) -> Box<dyn EventRepository>;
    fn create_issue_repository(&self) -> Box<dyn IssueRepository>;
    fn create_dependency_repository(&self) -> Box<dyn DependencyRepository>;
    fn create_security_repository(&self) -> Box<dyn SecurityRepository>;
    fn create_debt_repository(&self) -> Box<dyn DebtRepository>;
}
```

### SQLite Factory Implementation

```rust
pub struct SqliteRepositoryFactory {
    pool: Arc<ConnectionPool>,
}

impl SqliteRepositoryFactory {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

impl RepositoryFactory for SqliteRepositoryFactory {
    fn create_project_repository(&self) -> Box<dyn ProjectRepository> {
        Box::new(SqliteProjectRepository::new(Arc::clone(&self.pool)))
    }

    fn create_analysis_repository(&self) -> Box<dyn AnalysisRepository> {
        Box::new(SqliteAnalysisRepository::new(Arc::clone(&self.pool)))
    }

    // ... other repository creations
}
```

## Repository Manager

The manager provides caching and lifecycle management:

```rust
pub struct RepositoryManager {
    factory: Arc<dyn RepositoryFactory>,
    project_repo: Arc<RwLock<Option<Arc<dyn ProjectRepository>>>>,
    analysis_repo: Arc<RwLock<Option<Arc<dyn AnalysisRepository>>>>,
    // ... other cached repositories
}

impl RepositoryManager {
    pub fn new(factory: Arc<dyn RepositoryFactory>) -> Self {
        Self {
            factory,
            project_repo: Arc::new(RwLock::new(None)),
            analysis_repo: Arc::new(RwLock::new(None)),
            // ... initialize other fields
        }
    }

    pub async fn get_project_repository(&self) -> Arc<dyn ProjectRepository> {
        let cache = self.project_repo.read().await;
        if let Some(repo) = &*cache {
            return Arc::clone(repo);
        }
        drop(cache);

        let mut cache = self.project_repo.write().await;
        if let Some(repo) = &*cache {
            return Arc::clone(repo);
        }

        let repo = Arc::from(self.factory.create_project_repository());
        *cache = Some(Arc::clone(&repo));
        repo
    }

    // ... similar methods for other repositories
}
```

## Error Handling

### Error Types

Comprehensive error types for repository operations:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Entity not found: {entity_type} with {identifier}")]
    NotFound {
        entity_type: String,
        identifier: String,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Task join error: {0}")]
    TaskJoin(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Pool error: {0}")]
    Pool(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
```

### Error Conversion

Automatic conversion from database errors:

```rust
impl From<rusqlite::Error> for RepositoryError {
    fn from(err: rusqlite::Error) -> Self {
        RepositoryError::Database {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(err: serde_json::Error) -> Self {
        RepositoryError::Serialization(err.to_string())
    }
}
```

### Error Handling Macro

Convenient macro for error creation:

```rust
#[macro_export]
macro_rules! repo_error {
    ($variant:ident, $($arg:tt)*) => {
        RepositoryError::$variant(format!($($arg)*))
    };
}

// Usage
return Err(repo_error!(Validation, "Invalid project name: {}", name));
```

## Unit of Work Pattern

Transaction management across repositories:

```rust
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> RepositoryResult<()>;
    async fn commit(&self) -> RepositoryResult<()>;
    async fn rollback(&self) -> RepositoryResult<()>;

    fn project_repository(&self) -> &dyn ProjectRepository;
    fn analysis_repository(&self) -> &dyn AnalysisRepository;
    fn issue_repository(&self) -> &dyn IssueRepository;
}

pub struct SqliteUnitOfWork {
    connection: Arc<Mutex<Connection>>,
    project_repo: SqliteProjectRepository,
    analysis_repo: SqliteAnalysisRepository,
    issue_repo: SqliteIssueRepository,
}

impl SqliteUnitOfWork {
    pub async fn new(pool: Arc<ConnectionPool>) -> RepositoryResult<Self> {
        let connection = pool.get_connection()
            .map_err(|e| RepositoryError::Connection(e.to_string()))?;

        // Begin transaction
        connection.execute("BEGIN TRANSACTION", [])?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            project_repo: SqliteProjectRepository::new_with_connection(Arc::clone(&connection)),
            analysis_repo: SqliteAnalysisRepository::new_with_connection(Arc::clone(&connection)),
            issue_repo: SqliteIssueRepository::new_with_connection(Arc::clone(&connection)),
        })
    }
}

#[async_trait]
impl UnitOfWork for SqliteUnitOfWork {
    async fn commit(&self) -> RepositoryResult<()> {
        let conn = self.connection.lock().await;
        conn.execute("COMMIT", [])
            .map_err(|e| RepositoryError::Transaction(e.to_string()))?;
        Ok(())
    }

    async fn rollback(&self) -> RepositoryResult<()> {
        let conn = self.connection.lock().await;
        conn.execute("ROLLBACK", [])
            .map_err(|e| RepositoryError::Transaction(e.to_string()))?;
        Ok(())
    }
}
```

## Testing Strategies

### Mock Repositories

Using mockall for unit testing:

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    pub ProjectRepo {}

    #[async_trait]
    impl Repository for ProjectRepo {
        type Entity = Project;

        async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Project>>;
        async fn find_all(&self) -> RepositoryResult<Vec<Project>>;
        async fn create(&self, entity: Project) -> RepositoryResult<Project>;
        async fn update(&self, entity: Project) -> RepositoryResult<Project>;
        async fn delete(&self, id: Uuid) -> RepositoryResult<bool>;
    }

    #[async_trait]
    impl ProjectRepository for ProjectRepo {
        async fn find_by_path(&self, path: &str) -> RepositoryResult<Option<Project>>;
        async fn find_recent(&self, limit: usize) -> RepositoryResult<Vec<Project>>;
        async fn search(&self, query: &str) -> RepositoryResult<Vec<Project>>;
        async fn count(&self) -> RepositoryResult<usize>;
    }
}

#[tokio::test]
async fn test_project_service_with_mock() {
    let mut mock = MockProjectRepo::new();

    mock.expect_find_by_id()
        .with(eq(test_id))
        .times(1)
        .returning(|_| Ok(Some(test_project())));

    let service = ProjectService::new(Arc::new(mock));
    let result = service.get_project(test_id).await;

    assert!(result.is_ok());
}
```

### In-Memory Repository

For integration tests:

```rust
pub struct InMemoryProjectRepository {
    storage: Arc<RwLock<HashMap<Uuid, Project>>>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Repository for InMemoryProjectRepository {
    type Entity = Project;

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Self::Entity>> {
        let storage = self.storage.read().await;
        Ok(storage.get(&id).cloned())
    }

    async fn create(&self, entity: Self::Entity) -> RepositoryResult<Self::Entity> {
        let mut storage = self.storage.write().await;
        storage.insert(entity.id, entity.clone());
        Ok(entity)
    }
}
```

## Best Practices

### 1. Always Use Async/Await

All repository methods should be async for consistency:

```rust
// Good
async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Entity>>

// Bad
fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Entity>>
```

### 2. Use spawn_blocking for Database Operations

Prevent blocking the async runtime:

```rust
tokio::task::spawn_blocking(move || {
    // Database operations here
})
.await?
```

### 3. Return RepositoryResult

Use the standardized error type:

```rust
// Good
async fn find_all(&self) -> RepositoryResult<Vec<Entity>>

// Bad
async fn find_all(&self) -> Result<Vec<Entity>, Box<dyn Error>>
```

### 4. Implement Proper Cleanup

Ensure connections return to pool:

```rust
let conn = pool.get_connection()?;
let result = do_work(&conn)?;
pool.return_connection(conn).await?;  // Always return
```

### 5. Use Builder Pattern for Complex Queries

For complex queries, use builder pattern:

```rust
pub struct ProjectQueryBuilder {
    language: Option<String>,
    framework: Option<String>,
    created_after: Option<DateTime<Utc>>,
    limit: Option<usize>,
}

impl ProjectQueryBuilder {
    pub fn new() -> Self { ... }
    pub fn language(mut self, lang: &str) -> Self { ... }
    pub fn framework(mut self, fw: &str) -> Self { ... }
    pub fn created_after(mut self, date: DateTime<Utc>) -> Self { ... }
    pub fn limit(mut self, limit: usize) -> Self { ... }

    pub async fn execute(&self, repo: &dyn ProjectRepository) -> RepositoryResult<Vec<Project>> {
        // Build and execute query
    }
}

// Usage
let projects = ProjectQueryBuilder::new()
    .language("rust")
    .framework("actix-web")
    .limit(10)
    .execute(&project_repo)
    .await?;
```

## Performance Optimization

### 1. Connection Pooling

Configure appropriate pool settings:

```rust
PoolConfig {
    max_connections: 20,  // Based on workload
    min_idle: 5,          // Maintain ready connections
    max_lifetime: Some(Duration::from_secs(3600)),
    idle_timeout: Some(Duration::from_secs(600)),
}
```

### 2. Prepared Statements

Cache prepared statements when possible:

```rust
pub struct SqliteProjectRepository {
    pool: Arc<ConnectionPool>,
    statements: Arc<RwLock<HashMap<String, Statement>>>,
}
```

### 3. Batch Operations

Implement batch operations for efficiency:

```rust
async fn create_batch(&self, entities: Vec<Self::Entity>) -> RepositoryResult<Vec<Self::Entity>> {
    // Use transaction for batch insert
    // INSERT multiple rows in single statement
}
```

### 4. Lazy Loading

Implement lazy loading for related entities:

```rust
pub struct ProjectWithAnalyses {
    project: Project,
    analyses: OnceCell<Vec<AnalysisRun>>,
}

impl ProjectWithAnalyses {
    pub async fn analyses(&self, repo: &dyn AnalysisRepository) -> RepositoryResult<&Vec<AnalysisRun>> {
        self.analyses.get_or_try_init(|| async {
            repo.find_by_project(self.project.id).await
        }).await
    }
}
```

## Conclusion

The repository pattern implementation in Uveddi provides:

1. **Clean Architecture**: Clear separation between business logic and data access
2. **Testability**: Easy mocking and testing through interfaces
3. **Flexibility**: Swap database implementations without changing business logic
4. **Maintainability**: Consistent patterns across all data access
5. **Performance**: Optimized through pooling, caching, and batch operations

Follow these patterns and best practices to maintain a clean, efficient, and maintainable data access layer.