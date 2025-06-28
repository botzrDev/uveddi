#!/bin/bash
# Script to check PostgreSQL connection and create database if needed

# Load environment variables from .env
set -a
source .env
set +a

echo "Checking PostgreSQL connection..."

# Check if psql is installed
if ! command -v psql &> /dev/null; then
    echo "PostgreSQL client (psql) is not installed. Please install it first."
    echo "On Ubuntu/Debian: sudo apt-get install postgresql-client"
    echo "On Fedora/RHEL: sudo dnf install postgresql"
    echo "On macOS with Homebrew: brew install postgresql"
    exit 1
fi

# Try to connect to PostgreSQL
if PGPASSWORD="$DB_POSTGRES_PASSWORD" psql -h "$DB_POSTGRES_HOST" -p "$DB_POSTGRES_PORT" -U "$DB_POSTGRES_USER" -d postgres -c '\q' 2>/dev/null; then
    echo "Successfully connected to PostgreSQL server."
else
    echo "Could not connect to PostgreSQL server. Please check if PostgreSQL is running."
    echo "On Ubuntu/Debian: sudo service postgresql start"
    echo "On Fedora/RHEL: sudo systemctl start postgresql"
    echo "On macOS with Homebrew: brew services start postgresql"
    exit 1
fi

# Check if database exists
if PGPASSWORD="$DB_POSTGRES_PASSWORD" psql -h "$DB_POSTGRES_HOST" -p "$DB_POSTGRES_PORT" -U "$DB_POSTGRES_USER" -d postgres -lqt | cut -d \| -f 1 | grep -qw "$DB_POSTGRES_DB"; then
    echo "Database '$DB_POSTGRES_DB' already exists."
else
    echo "Creating database '$DB_POSTGRES_DB'..."
    PGPASSWORD="$DB_POSTGRES_PASSWORD" psql -h "$DB_POSTGRES_HOST" -p "$DB_POSTGRES_PORT" -U "$DB_POSTGRES_USER" -d postgres -c "CREATE DATABASE $DB_POSTGRES_DB;" || {
        echo "Failed to create database."
        exit 1
    }
    echo "Database '$DB_POSTGRES_DB' created successfully."
fi

echo "PostgreSQL connection setup is complete."
echo "You can now run the Alembic migrations."
