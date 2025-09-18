# Assignment 05: Refactor Database Layer

## Priority: HIGH
## Estimated Time: 4-5 hours
## Directory: `/src/database/`

## Objective
Refactor database layer to eliminate coupling issues and improve separation of concerns.

## Current Problems
- Potential circular dependency with application layer
- Mixed concerns (connection management + domain logic)
- Large files mixing different database operations
- Inconsistent error handling

## Tasks

### 1. Audit Current Database Structure
```bash
find src/database -name "*.rs" -exec wc -l {} +
grep -r "use crate::application" src/database/
grep -r "use crate::database" src/application/
```

### 2. Create Clean Database Architecture
```
src/database/
├── mod.rs (public API only)
├── connection/
│   ├── mod.rs
│   ├── pool.rs (connection pooling)
│   ├── sqlite.rs (SQLite implementation)
│   └── postgres.rs (PostgreSQL implementation)
├── repositories/
│   ├── mod.rs
│   ├── analysis_repository.rs
│   ├── project_repository.rs
│   └── cache_repository.rs
├── models/
│   ├── mod.rs
│   ├── analysis_result.rs
│   ├── project.rs
│   └── cache_entry.rs
├── migrations/
│   ├── mod.rs
│   └── version_*.rs
└── schema/
    ├── mod.rs
    └── definitions.rs
```

### 3. Extract Connection Management
- Move all connection pooling to `connection/pool.rs`
- Create `DatabaseConnection` trait for abstraction
- Implement database-specific connections
- Target: <200 lines per file

### 4. Implement Repository Pattern
Create repositories for domain entities:

```rust
// In repositories/analysis_repository.rs
pub trait AnalysisRepository {
    async fn save_analysis(&self, analysis: &AnalysisResult) -> Result<AnalysisId>;
    async fn get_analysis(&self, id: AnalysisId) -> Result<Option<AnalysisResult>>;
    async fn list_analyses(&self, project_id: ProjectId) -> Result<Vec<AnalysisResult>>;
    async fn delete_analysis(&self, id: AnalysisId) -> Result<()>;
}

pub struct SqliteAnalysisRepository {
    pool: Arc<SqlitePool>,
}

impl AnalysisRepository for SqliteAnalysisRepository {
    // Implementation
}
```

### 5. Extract Domain Models
- Move all database models to `models/` directory
- Separate persistence models from domain models
- Implement conversion traits between them
- Target: <150 lines per model

### 6. Eliminate Circular Dependencies
- Remove all references from database layer to application layer
- Use dependency inversion principle
- Pass required data as parameters instead of importing application types
- Create events/callbacks if needed

### 7. Implement Database Abstraction
Create database-agnostic interface:
```rust
pub trait Database {
    type Connection: DatabaseConnection;

    async fn connect(&self, config: &DatabaseConfig) -> Result<Self::Connection>;
    async fn migrate(&self, connection: &Self::Connection) -> Result<()>;
    async fn health_check(&self, connection: &Self::Connection) -> Result<()>;
}
```

### 8. Create Migration System
- Extract migration logic to dedicated module
- Version all schema changes
- Support both up and down migrations
- Target: <100 lines per migration

### 9. Standardize Error Handling
Create database-specific error types:
```rust
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}
```

### 10. Update Integration Points
- Update application layer to use repository pattern
- Remove direct database imports from other modules
- Use dependency injection for database services

## Breaking Changes
- Repository interfaces replace direct database calls
- Database configuration changes
- Some internal APIs reorganized

## Success Criteria
- [ ] No circular dependencies with application layer
- [ ] Repository pattern implemented
- [ ] All database files <300 lines
- [ ] Clean separation of concerns
- [ ] All tests pass
- [ ] Migration system works

## Database-Specific Considerations
- Ensure SQLite and PostgreSQL support maintained
- Test connection pooling under load
- Verify transaction handling
- Check query performance

## Verification Commands
```bash
# Check circular dependencies
cargo deny check

# Check file sizes
find src/database -name "*.rs" -exec wc -l {} +

# Run database tests
cargo test database::

# Test migrations
cargo run -- migrate --dry-run
```

## Completion Notes
_To be filled by AI developer:_
- Circular dependencies resolved: ___
- Repository interfaces created: ___
- Performance impact: ___
- Migration system complexity: ___
- Breaking changes: ___