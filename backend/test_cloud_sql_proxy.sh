#!/bin/bash
# Script to set up and test Cloud SQL Proxy for Google Cloud SQL

# Change to the backend directory
cd "$(dirname "$0")"

# Load environment variables from .env
set -a
source .env
set +a

# Check if required environment variables are set
if [ -z "$GOOGLE_CLOUD_PROJECT" ] || [ -z "$CLOUD_SQL_CONNECTION_NAME" ]; then
    echo "ERROR: GOOGLE_CLOUD_PROJECT and CLOUD_SQL_CONNECTION_NAME must be set in .env file."
    echo "Please update your .env file with the following:"
    echo "GOOGLE_CLOUD_PROJECT=your-project-id"
    echo "CLOUD_SQL_CONNECTION_NAME=your-project:region:instance"
    exit 1
fi

echo "Setting up Cloud SQL Proxy for Google Cloud SQL..."

# Check if Cloud SQL Proxy is installed
if ! command -v cloud-sql-proxy &> /dev/null; then
    echo "Cloud SQL Proxy is not installed. Installing..."
    
    # Download Cloud SQL Proxy
    curl -o cloud-sql-proxy https://storage.googleapis.com/cloud-sql-connectors/cloud-sql-proxy/v2.0.0/cloud-sql-proxy.linux.amd64
    chmod +x cloud-sql-proxy
    sudo mv cloud-sql-proxy /usr/local/bin/
    
    echo "Cloud SQL Proxy installed successfully."
fi

# Check if Google Cloud SDK is installed and authenticated
if ! command -v gcloud &> /dev/null; then
    echo "Google Cloud SDK (gcloud) is not installed. Please install it first."
    echo "Visit: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Check authentication
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q "@"; then
    echo "Not authenticated with Google Cloud. Please run:"
    echo "gcloud auth login"
    exit 1
fi

# Run Cloud SQL Proxy in the background
echo "Starting Cloud SQL Proxy..."
cloud-sql-proxy --port=5432 "$CLOUD_SQL_CONNECTION_NAME" &
PROXY_PID=$!

# Wait for proxy to start
sleep 5

# Test the connection
echo "Testing connection to Cloud SQL..."
if PGPASSWORD="$DB_POSTGRES_PASSWORD" psql -h localhost -p 5432 -U "$DB_POSTGRES_USER" -d postgres -c '\conninfo' 2>/dev/null; then
    echo "Successfully connected to Cloud SQL via proxy!"
    echo "Your application can now connect to Cloud SQL using the following connection string:"
    echo "postgresql://${DB_POSTGRES_USER}:${DB_POSTGRES_PASSWORD}@localhost:5432/${DB_POSTGRES_DB}"
else
    echo "Failed to connect to Cloud SQL. Please check your credentials and connection settings."
fi

# Kill the proxy when done
kill $PROXY_PID

echo "Cloud SQL Proxy test completed."
