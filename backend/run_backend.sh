#!/bin/bash
# Script to run the FastAPI backend for CodeAtlas

# Change to the backend directory
cd "$(dirname "$0")"

# Load environment variables from .env
set -a
source .env
set +a

echo "Starting CodeAtlas FastAPI backend..."

# Check if uvicorn is installed
if ! command -v uvicorn &> /dev/null; then
    echo "Uvicorn is not installed. Installing required packages..."
    pip install -r requirements.txt
fi

# Run the FastAPI server
uvicorn main:app --host ${APP_HOST} --port ${APP_PORT} --reload

echo "FastAPI backend stopped."
