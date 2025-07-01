#!/bin/bash
# Script to run Alembic migrations for Uveddi backend

# Change to the backend directory
cd "$(dirname "$0")"

# Load environment variables from .env
set -a
source .env
set +a

echo "Running Alembic migrations for Uveddi backend..."

# Check if alembic is installed
if ! command -v alembic &> /dev/null; then
    echo "Alembic is not installed. Installing required packages..."
    pip install -r requirements.txt
fi

# Run migrations
alembic upgrade head

# Check migration status
echo "Current migration status:"
alembic current

echo "Migration complete. You can now start the FastAPI backend."
