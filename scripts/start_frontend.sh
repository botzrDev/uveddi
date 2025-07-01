#!/bin/bash

PORT=7777
# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
# Assume the frontend directory is one level up from the scripts dir
FRONTEND_DIR="$SCRIPT_DIR/../frontend"
PID_FILE="$SCRIPT_DIR/.frontend.pid"

# Check if the frontend directory exists
if [ ! -d "$FRONTEND_DIR" ]; then
  echo "Error: Frontend directory not found at '$FRONTEND_DIR'."
  exit 1
fi

# Check if a PID file exists and if the process is running
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p $PID > /dev/null; then
        echo "Server is already running on port $PORT with PID $PID."
        echo "Access it at http://localhost:$PORT"
        exit 0
    fi
fi

# Check if port is in use by another process
if lsof -i :$PORT > /dev/null; then
  echo "Port $PORT is already in use by another process."
  exit 1
fi

echo "Starting server on http://localhost:$PORT"
# Navigate to the frontend directory, start the server in the background, and store its PID
(cd "$FRONTEND_DIR" && python3 -m http.server $PORT & echo $! > "$PID_FILE")

echo "Server started. PID stored in $PID_FILE"
echo "You can stop it later by running 'kill \$(cat $PID_FILE)'"
