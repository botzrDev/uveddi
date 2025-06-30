# CodeAtlas Backend

This directory contains the optional FastAPI backend for CodeAtlas.

## Purpose of the Backend

The CodeAtlas backend is a Python FastAPI service that provides optional cloud-based features for the CodeAtlas Rust CLI tool. While the Rust CLI performs all core code analysis and reporting locally, the backend enables team collaboration, centralized storage, and future web dashboards.

### Interaction with the Rust Application
- The Rust CLI analyzes codebases and can upload analysis results to the backend via REST API endpoints.
- The backend stores results in a PostgreSQL database and exposes endpoints for retrieving, sharing, and managing analysis data.
- This separation allows users to run CodeAtlas fully offline or opt-in to cloud features for team workflows.

## API

The API is documented using the OpenAPI standard. When the backend is running, you can find the interactive API documentation at `http://localhost:8000/docs`.

## Database Architecture

The CodeAtlas system uses a two-database architecture:
1. **Local SQLite database** in the Rust CLI for storing local analysis data.
2. **Centralized PostgreSQL database** in this FastAPI backend for cloud synchronization.

## Setup Options

### Option 1: Docker Compose (Recommended)

The easiest way to get started is using Docker Compose, which sets up both PostgreSQL and the FastAPI backend:

```bash
# Start PostgreSQL and the FastAPI backend
./docker-compose.sh up

# Start only the PostgreSQL database (for local development)
./docker-compose.sh db-only

# Run database migrations
./docker-compose.sh migrate

# Run tests
./docker-compose.sh test

# View logs
./docker-compose.sh logs

# Stop all containers
./docker-compose.sh down
```

The API will be available at http://localhost:8000 with interactive documentation at http://localhost:8000/docs

> **Note:** The PostgreSQL database will be accessible at `localhost:5433` to avoid conflicts with any existing PostgreSQL installations.

### Option 2: Local Development Setup

If you prefer to run PostgreSQL and the backend directly on your system:

1. **Create and configure the environment file**

   ```bash
   cp .env.example .env
   # Edit .env with your preferred settings
   ```

2. **Set up the PostgreSQL database**

   ```bash
   # Run the database setup script
   ./setup_database.sh
   ```

3. **Run the database migrations**

   ```bash
   # Apply all migrations to the database
   ./run_migrations.sh
   ```

4. **Start the FastAPI backend**

   ```bash
   # Start the development server
   ./run_backend.sh
   ```

   The API will be available at http://localhost:8000 with interactive documentation at http://localhost:8000/docs

### Option 3: Using the Makefile

The Makefile provides convenient shortcuts for various tasks:

```bash
# Set up the database
make setup

# Run migrations
make migrate

# Start the backend
make run

# Build Docker image
make docker-build

# Run Docker container
make docker-run

# Deploy to Google Cloud
make gcloud-deploy
```

## Database Migration Management

The backend uses Alembic for database migrations. Common tasks:

```bash
# Create a new migration (after modifying models.py)
alembic revision --autogenerate -m "Description of changes"

# Apply all pending migrations
alembic upgrade head

# Roll back the last migration
alembic downgrade -1

# Check current migration status
alembic current
```

## Google Cloud Deployment

### Setting Up Cloud SQL

1. **Create a Cloud SQL PostgreSQL instance** in the Google Cloud Console

2. **Configure the .env file** with Google Cloud settings:

   ```
   # For Google Cloud SQL
   GOOGLE_CLOUD_PROJECT=your-project-id
   CLOUD_SQL_CONNECTION_NAME=your-project:region:instance
   GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account-file.json
   ```

3. **Test the Cloud SQL Proxy connection**:

   ```bash
   ./test_cloud_sql_proxy.sh
   ```

### Docker Container Deployment

The included Dockerfile is configured for Google Cloud Run with Cloud SQL Proxy support:

```bash
# Build the Docker image
docker build -t codeatlas-backend .

# Run the container locally (for testing)
docker run -p 8000:8080 \
  --env-file .env \
  --mount type=bind,source=/path/to/service-account.json,target=/secrets/cloudsql/credentials.json,readonly \
  codeatlas-backend
```

### Deploying to Cloud Run

```bash
# Build and push the image to Google Container Registry
gcloud builds submit --tag gcr.io/PROJECT_ID/codeatlas-backend

# Deploy to Cloud Run with Cloud SQL connection
gcloud run deploy codeatlas-backend \
  --image gcr.io/PROJECT_ID/codeatlas-backend \
  --platform managed \
  --add-cloudsql-instances PROJECT_ID:REGION:INSTANCE_NAME \
  --set-env-vars "DB_POSTGRES_HOST=127.0.0.1,DB_POSTGRES_DB=codeatlas,DB_POSTGRES_USER=codeatlas,DB_ENVIRONMENT=production" \
  --set-secrets="DB_POSTGRES_PASSWORD=db-password:latest" \
  --allow-unauthenticated
```

## Troubleshooting

If you encounter issues with database connectivity:

1. Ensure PostgreSQL is running: `sudo service postgresql status`
2. Check database exists: `psql -U postgres -c '\l' | grep codeatlas`
3. Verify credentials in `.env` file
4. Check migration status: `alembic current`
5. For Cloud SQL issues, verify IAM permissions and network connectivity
