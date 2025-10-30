#!/bin/bash

# --- Configuration ---
FRONTEND_PORT=9999
BACKEND_PORT=8000

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
FRONTEND_DIR="$PROJECT_ROOT/frontend"
BACKEND_DIR="$PROJECT_ROOT/backend"

FRONTEND_PID_FILE="$SCRIPT_DIR/.frontend.pid"
BACKEND_PID_FILE="$SCRIPT_DIR/.backend.pid"

# --- Stop existing services ---
echo "Stopping existing services..."

# Stop frontend by PID if available
if [ -f "$FRONTEND_PID_FILE" ]; then
    PID=$(cat "$FRONTEND_PID_FILE")
    if ps -p $PID > /dev/null; then
        echo "Stopping frontend process with PID $PID..."
        kill $PID
        sleep 1 # Give it a moment to shut down
    fi
    rm -f "$FRONTEND_PID_FILE"
fi
# Kill any process on the frontend port as a fallback
echo "Ensuring nothing is running on frontend port $FRONTEND_PORT..."
fuser -k -n tcp $FRONTEND_PORT &>/dev/null

# Stop backend by PID if available
if [ -f "$BACKEND_PID_FILE" ]; then
    PID=$(cat "$BACKEND_PID_FILE")
    if ps -p $PID > /dev/null; then
        echo "Stopping backend process with PID $PID..."
        kill $PID
        sleep 1 # Give it a moment to shut down
    fi
    rm -f "$BACKEND_PID_FILE"
fi
# Kill any process on the backend port as a fallback
echo "Ensuring nothing is running on backend port $BACKEND_PORT..."
fuser -k -n tcp $BACKEND_PORT &>/dev/null

echo "All services stopped."
echo

# --- Start new services ---
echo "Starting fresh services..."

# Start backend
echo "Starting backend server on http://localhost:$BACKEND_PORT"
if [ -d "$BACKEND_DIR/venv" ]; then
    # Activate venv, start backend, and then come back
    (
        source "$BACKEND_DIR/venv/bin/activate"
        cd "$BACKEND_DIR" && uvicorn main:app --host 0.0.0.0 --port $BACKEND_PORT --reload & echo $! > "$BACKEND_PID_FILE"
    )
    echo "Backend server started. PID stored in $BACKEND_PID_FILE"
else
    echo "Warning: Backend virtual environment not found at '$BACKEND_DIR/venv'. Cannot start backend."
fi

# Start frontend
echo "Starting frontend server on http://localhost:$FRONTEND_PORT"
if [ -d "$FRONTEND_DIR" ]; then
    (cd "$FRONTEND_DIR" && npm run dev -- --host 0.0.0.0 --port $FRONTEND_PORT & echo $! > "$FRONTEND_PID_FILE")
    echo "Frontend server started. PID stored in $FRONTEND_PID_FILE"
else
    echo "Error: Frontend directory not found at '$FRONTEND_DIR'."
    exit 1
fi

echo
echo "All services started successfully."
echo "You can stop them later by running:"
echo "kill \$(cat $FRONTEND_PID_FILE) \$(cat $BACKEND_PID_FILE)"
