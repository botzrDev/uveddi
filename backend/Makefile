# Makefile for Uveddi backend

.PHONY: setup migrate run test clean docker-build docker-run

# Load environment variables from .env
include .env
export

# Default target
all: setup migrate run

# Set up the database
setup:
	@echo "Setting up the database..."
	./setup_database.sh

# Run database migrations
migrate:
	@echo "Running database migrations..."
	./run_migrations.sh

# Run the FastAPI backend
run:
	@echo "Starting the FastAPI backend..."
	./run_backend.sh

# Run tests
test:
	@echo "Running tests..."
	pytest

# Clean up generated files
clean:
	@echo "Cleaning up..."
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type d -name .pytest_cache -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

# Docker commands
docker-build:
	@echo "Building Docker image..."
	docker build -t uveddi-backend .

docker-run:
	@echo "Running Docker container..."
	docker run -p 8000:8080 --env-file .env uveddi-backend

# Google Cloud deployment
gcloud-build:
	@echo "Building for Google Cloud..."
	gcloud builds submit --tag gcr.io/$(GOOGLE_CLOUD_PROJECT)/uveddi-backend

gcloud-deploy:
	@echo "Deploying to Google Cloud Run..."
	gcloud run deploy uveddi-backend \
		--image gcr.io/$(GOOGLE_CLOUD_PROJECT)/uveddi-backend \
		--platform managed \
		--region us-central1 \
		--add-cloudsql-instances $(CLOUD_SQL_CONNECTION_NAME) \
		--set-env-vars "DB_POSTGRES_HOST=127.0.0.1,DB_POSTGRES_DB=$(DB_POSTGRES_DB),DB_POSTGRES_USER=$(DB_POSTGRES_USER),DB_ENVIRONMENT=production" \
		--set-secrets="DB_POSTGRES_PASSWORD=db-password:latest" \
		--allow-unauthenticated

# Create a new migration
new-migration:
	@echo "Creating a new migration..."
	@read -p "Enter migration description: " desc; \
	alembic revision --autogenerate -m "$$desc"

# Help target
help:
	@echo "Uveddi Backend Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make setup         - Set up the database"
	@echo "  make migrate       - Run database migrations"
	@echo "  make run           - Start the FastAPI backend"
	@echo "  make test          - Run tests"
	@echo "  make clean         - Clean up generated files"
	@echo "  make docker-build  - Build Docker image"
	@echo "  make docker-run    - Run Docker container"
	@echo "  make gcloud-build  - Build for Google Cloud"
	@echo "  make gcloud-deploy - Deploy to Google Cloud Run"
	@echo "  make new-migration - Create a new migration"
	@echo "  make help          - Show this help message"
