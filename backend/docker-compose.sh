#!/bin/bash
# Script to run the Uveddi backend with Docker Compose

# Change to the backend directory
cd "$(dirname "$0")"

echo "Starting Uveddi backend with Docker Compose..."

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "Docker Compose is not installed. Please install it first."
    echo "Visit: https://docs.docker.com/compose/install/"
    exit 1
fi

# Run PostgreSQL only (for local development with Python directly)
if [ "$1" == "db-only" ]; then
    echo "Starting PostgreSQL container only..."
    docker-compose up -d postgres
    echo "PostgreSQL is running at localhost:5433 (mapped from container port 5432)"
    echo "User: uveddi"
    echo "Password: uveddi"
    echo "Database: uveddi"
    exit 0
fi

# Run both PostgreSQL and the backend
if [ "$1" == "up" ] || [ -z "$1" ]; then
    echo "Starting PostgreSQL and FastAPI backend..."
    docker-compose up -d
    echo "Backend is running at http://localhost:8000"
    echo "API documentation is available at http://localhost:8000/docs"
    exit 0
fi

# Stop all containers
if [ "$1" == "down" ]; then
    echo "Stopping all containers..."
    docker-compose down
    exit 0
fi

# Show logs
if [ "$1" == "logs" ]; then
    echo "Showing logs..."
    docker-compose logs -f
    exit 0
fi

# Run migrations
if [ "$1" == "migrate" ]; then
    echo "Running migrations..."
    docker-compose exec backend alembic upgrade head
    exit 0
fi

# Run tests
if [ "$1" == "test" ]; then
    echo "Running tests..."
    docker-compose exec backend python test_database.py
    exit 0
fi

# If no valid command is provided, show help
echo "Usage: $0 [command]"
echo "Commands:"
echo "  up        Start PostgreSQL and FastAPI backend (default)"
echo "  db-only   Start only PostgreSQL container"
echo "  down      Stop all containers"
echo "  logs      Show container logs"
echo "  migrate   Run database migrations"
echo "  test      Run database tests"
exit 1
