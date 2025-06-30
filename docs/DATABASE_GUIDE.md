# CodeAtlas Database Architecture Guide

This document provides a comprehensive guide for setting up, migrating, and deploying the CodeAtlas database architecture.

## Database Architecture Overview

CodeAtlas uses a dual-database architecture for robust, scalable, and developer-friendly operation:

1. **Local SQLite Database** (Rust CLI)
   - Embedded in the CLI tool
   - Stores local analysis data, configuration, plugin metadata, and history
   - Uses [Refinery](https://github.com/rust-db/refinery) for migrations
   - Local-only, not synchronized by default
   - Schema is defined in `migrations/V1__initial_schema_sqlite.sql`
   - Foreign key constraints are enforced (enabled via `PRAGMA foreign_keys = ON`)
   - Comprehensive integration tests are provided in `tests/sqlite_db.rs` and must pass in CI

2. **Centralized PostgreSQL Database** (Python/FastAPI Backend)
   - Stores organization and user data, projects, analysis history, anti-pattern catalog, plugin registry, and more
   - Aggregates analysis data from multiple clients and supports team/enterprise features
   - Uses SQLAlchemy ORM and Alembic for migrations
   - Deployable to Google Cloud SQL or any managed PostgreSQL service
   - Schema is defined in `migrations/V1__initial_schema_postgres.sql`
   - Default anti-pattern types are inserted by migration or test setup
   - Comprehensive integration tests are provided in `tests/postgres_db.py` and must pass in CI
   - Enforces check constraints (e.g., valid `subscription_tier` values)

### Key Practices
- **Migrations:**
  - SQLite: CLI runs migrations automatically on startup using Refinery.
  - PostgreSQL: Backend uses Alembic migrations, applied via scripts or Docker Compose.
- **Testing:**
  - Both databases have comprehensive integration tests for CRUD, relationships, constraints, and default data.
  - SQLite tests use in-memory databases with foreign key enforcement for isolation and speed.
  - PostgreSQL tests use a dedicated test database, truncate all tables before each test, and ensure constraints and default data.
- **Production Readiness:**
  - Both schemas are normalized, indexed, and enforce referential integrity.
  - All tests must pass before deployment or release.

## Quick Start with Docker Compose

The easiest way to get started is using Docker Compose:

```bash
cd backend
./docker-compose.sh up
```

This will:
1. Start a PostgreSQL container
2. Apply all migrations
3. Start the FastAPI backend

The API will be accessible at http://localhost:8000 with interactive documentation at http://localhost:8000/docs.

## Alternative Setup Methods

### Local PostgreSQL Setup

#### Prerequisites
- PostgreSQL 12+ installed
- Python 3.10+ installed
- Rust toolchain (for CLI)

#### Database Setup
```bash
# Navigate to the backend directory
cd backend

# Copy the example environment file and edit as needed
cp .env.example .env

# Run the database setup script
./setup_database.sh
```

### Running Migrations

Migrations are version-controlled database schema changes.

#### PostgreSQL Migrations (Backend)
```bash
# Apply all pending migrations
./run_migrations.sh

# Or using make:
make migrate

# Or with Docker Compose:
./docker-compose.sh migrate
```

#### SQLite Migrations (Rust CLI)
The Rust CLI automatically runs migrations on startup.

### Running the Backend

```bash
# Start the FastAPI backend
./run_backend.sh

# Or using make:
make run

# Or with Docker Compose:
./docker-compose.sh up
```

The API will be accessible at http://localhost:8000 with interactive documentation at http://localhost:8000/docs.

## Advanced Migration Management

### Creating New Migrations

After modifying models in `models.py`, create a new migration:

```bash
# Using alembic directly
alembic revision --autogenerate -m "Description of changes"

# Or using the Makefile
make new-migration
```

### Migration Management Commands

```bash
# View current migration version
alembic current

# Upgrade to a specific version
alembic upgrade +1        # Up one revision
alembic upgrade head      # Up to the latest revision

# Downgrade
alembic downgrade -1      # Down one revision
alembic downgrade base    # Down to the base (empty schema)

# Generate SQL without applying (for review)
alembic upgrade head --sql > migration.sql
```

## Google Cloud Deployment

### Setting Up Cloud SQL

1. **Create a Cloud SQL instance** in the Google Cloud Console
   - Choose PostgreSQL 12+
   - Configure machine type and storage
   - Enable private IP or public IP with authorized networks

2. **Create a database user** for CodeAtlas
   ```sql
   CREATE USER codeatlas WITH PASSWORD 'secure-password';
   CREATE DATABASE codeatlas;
   GRANT ALL PRIVILEGES ON DATABASE codeatlas TO codeatlas;
   ```

3. **Configure environment variables**
   Update your `.env` file with Cloud SQL settings:
   ```
   DB_POSTGRES_HOST=127.0.0.1  # When using Cloud SQL Proxy
   DB_POSTGRES_USER=codeatlas
   DB_POSTGRES_PASSWORD=secure-password
   DB_POSTGRES_DB=codeatlas
   DB_ENVIRONMENT=production
   GOOGLE_CLOUD_PROJECT=your-project-id
   CLOUD_SQL_CONNECTION_NAME=your-project:region:instance
   ```

### Using Cloud SQL Proxy

Cloud SQL Proxy provides a secure connection to your Cloud SQL instance:

```bash
# Test the connection
./test_cloud_sql_proxy.sh

# For regular use, start the proxy:
cloud-sql-proxy --port=5432 "your-project:region:instance"
```

### Deploying to Cloud Run

1. **Build and push the Docker image**
   ```bash
   make docker-build
   make gcloud-build
   ```

2. **Deploy to Cloud Run**
   ```bash
   make gcloud-deploy
   ```

3. **Setup Continuous Deployment (optional)**
   - Configure Cloud Build triggers on your repository
   - Add a `cloudbuild.yaml` file for automated deployment

## Troubleshooting

### Connection Issues

- **PostgreSQL not running**
  ```bash
  sudo service postgresql start
  ```

- **Wrong credentials**
  Verify credentials in `.env`

- **PostgreSQL authentication failure**
  Check `pg_hba.conf` configuration

- **Cloud SQL Proxy not working**
  - Verify service account permissions
  - Check network connectivity
  - Ensure correct connection name format

### Docker Compose Issues

- **Permission denied when accessing PostgreSQL volume**
  ```bash
  sudo chown -R $USER:$USER backend/postgres_data
  ```

- **Port 5432 already in use**
  Stop any running PostgreSQL service or modify the port in `docker-compose.yml`

### Migration Issues

- **Alembic error on autogenerate**
  - Make sure your models are imported in `env.py`
  - Verify the database connection works

- **Failed migrations**
  ```bash
  # Downgrade to working version
  alembic downgrade -1
  ```

## Resources

- [SQLAlchemy Documentation](https://docs.sqlalchemy.org/)
- [Alembic Documentation](https://alembic.sqlalchemy.org/)
- [Cloud SQL Documentation](https://cloud.google.com/sql/docs)
- [Cloud Run Documentation](https://cloud.google.com/run/docs)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
