# CodeAtlas Database Setup Guide

This guide provides comprehensive setup instructions for both the local SQLite database (Rust CLI) and the centralized PostgreSQL database (Python/FastAPI backend) for the CodeAtlas project.

## Part 1: Local SQLite Database Setup (Rust CLI)

### Overview
The CodeAtlas CLI uses SQLite for local storage of analysis results, cached ASTs, and user-specific configurations. This allows for privacy-focused, offline analysis and is crucial for the free tier.

### 1. Dependencies

The following dependencies have been added to `Cargo.toml`:

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled", "chrono"] }
refinery = { version = "0.8", features = ["rusqlite"] }
tokio = { version = "1.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.8"
```

### 2. Database Schema

The SQLite schema includes the following tables based on the ERD:

- **`anti_pattern_types`**: Catalog of predefined architectural anti-patterns
- **`projects`**: Local project tracking
- **`analysis_runs`**: Execution records of analysis runs
- **`architectural_issues`**: Detected issues/anti-patterns
- **`code_snippets`**: Code segments related to issues
- **`reports`**: Generated markdown reports
- **`diagrams`**: Visual diagrams embedded in reports
- **`plugins`**: Custom scanners/extensions

### 3. Migration System

Uses Refinery for database migrations:
- Migrations are stored in `migrations/` directory
- Initial schema is in `migrations/V1__initial_schema.sql`
- Migrations are embedded at compile time using `embed_migrations!("migrations")`

### 4. Usage Example

```rust
use codeatlas::database::DatabaseManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database (creates .codeatlas/data.db by default)
    let db = DatabaseManager::new(None).await?;
    
    // Database is ready to use
    println!("Database initialized successfully!");
    
    Ok(())
}
```

### 5. Database Location

- Default location: `.codeatlas/data.db` in the project root
- Can be customized by providing a path to `DatabaseManager::new(Some(path))`
- Parent directories are created automatically

### 6. CRUD Operations

The `database::crud` module provides methods for:
- Creating and updating architectural issues
- Managing code snippets
- Querying issues with filtering options
- Transaction support for complex operations

Example:
```rust
use codeatlas::database::{DatabaseManager, ArchitecturalIssue, CodeSnippet};

// Create an issue with associated code snippets
let (issue_id, snippet_ids) = db.create_issue_with_snippets(&issue, &snippets)?;

// Query issues by run ID
let issues = db.get_issues_by_run_id(run_id, false, Some("high"))?;

// Update ignored status
db.update_issue_ignored_status(issue_id, true, Some(user_id))?;
```

## Part 2: Centralized PostgreSQL Database Setup (Python/FastAPI)

### Overview
The centralized database serves the CodeAtlas subscriber system, CI/CD integrations, and manages centralized analysis data using PostgreSQL for scalability and robustness.

### 1. Dependencies

Create `backend/requirements.txt`:

```txt
# FastAPI and ASGI server
fastapi==0.104.1
uvicorn[standard]==0.24.0

# Database
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
alembic==1.13.1

# Authentication & Security
python-jose[cryptography]==3.3.0
passlib[bcrypt]==1.7.4
python-multipart==0.0.6

# Environment & Configuration
python-dotenv==1.0.0
pydantic==2.5.0
pydantic-settings==2.1.0

# Additional dependencies...
```

### 2. Database Configuration

Environment-based configuration in `backend/database.py`:

```python
class DatabaseSettings(BaseSettings):
    postgres_host: str = "localhost"
    postgres_port: int = 5432
    postgres_user: str = "codeatlas"
    postgres_password: str = "codeatlas"
    postgres_db: str = "codeatlas"
    
    @property
    def database_url(self) -> str:
        return (
            f"postgresql://{self.postgres_user}:{self.postgres_password}"
            f"@{self.postgres_host}:{self.postgres_port}/{self.postgres_db}"
        )
```

### 3. SQLAlchemy Models

Key entities defined in `backend/models.py`:
- **User**: CodeAtlas subscribers with role-based access
- **Organization**: Team/company accounts with subscription tiers
- **Project**: Codebases being analyzed
- **AnalysisRun**: Execution records with CI/CD support
- **ArchitecturalIssue**: Detected problems with AI suggestions
- **AntiPatternType**: Catalog of detectable patterns
- **Plugin**: Custom extensions and scanners

### 4. Alembic Migration Setup

Initialize Alembic:
```bash
cd backend
alembic init alembic
```

Create initial migration:
```bash
alembic revision --autogenerate -m "Initial tables"
alembic upgrade head
```

### 5. FastAPI Application

Dependency injection for database sessions:

```python
from database import get_async_session

async def get_db() -> AsyncSession:
    async for session in get_async_session():
        yield session

@app.post("/organizations", response_model=OrganizationResponse)
async def create_organization(
    organization_data: OrganizationCreate,
    db: AsyncSession = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    # Implementation...
```

### 6. Environment Configuration

Create `.env` file based on `.env.example`:

```bash
# Local development
DB_POSTGRES_HOST=localhost
DB_POSTGRES_PORT=5432
DB_POSTGRES_USER=codeatlas
DB_POSTGRES_PASSWORD=your_password
DB_POSTGRES_DB=codeatlas

# Production (cloud database)
# DATABASE_URL=postgresql://user:pass@host:port/db?sslmode=require
```

### 7. Running the Application

```bash
cd backend
pip install -r requirements.txt
uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

Access API documentation at: http://localhost:8000/docs

## Integration Notes

### Data Flow
1. **Local Analysis**: CLI performs analysis using SQLite for local storage
2. **Sync to Cloud**: Results can be synced to PostgreSQL backend for team sharing
3. **CI/CD Integration**: Automated runs store data directly in PostgreSQL
4. **Reporting**: Both systems generate markdown reports with embedded diagrams

### Security Considerations
- SQLite: File-based, controlled by filesystem permissions
- PostgreSQL: Network-based, requires connection encryption (sslmode=require)
- Authentication: JWT tokens for API access
- Authorization: Role-based access control (individual, org_member, org_admin)

### Deployment Recommendations

#### Local (SQLite)
- Database file in `.codeatlas/` directory (gitignored)
- Automatic backups before major operations
- Migration rollback support

#### Cloud (PostgreSQL)
- Use managed database services (AWS RDS, Google Cloud SQL, Azure Database)
- Enable automated backups and point-in-time recovery
- Configure connection pooling for high availability
- Use environment variables for all sensitive configuration

### Performance Optimization
- SQLite: Use WAL mode for better concurrency
- PostgreSQL: Implement connection pooling and query optimization
- Both: Index critical query paths (file_path, run_id, severity)

This setup provides a robust, scalable foundation for CodeAtlas while maintaining the ability to operate offline with local SQLite storage and scale to enterprise needs with the centralized PostgreSQL backend.
