#!/bin/sh
# Production entrypoint script for Uveddi Analysis Engine
# Handles initialization, health checks, and graceful shutdown

set -euo pipefail

# Configuration
CONFIG_FILE="${CONFIG_FILE:-/app/config/default.yaml}"
LOG_LEVEL="${LOG_LEVEL:-info}"
CACHE_WARMUP="${CACHE_WARMUP:-true}"

# Function: Log with timestamp
log() {
    echo "[$(date -Iseconds)] $*" >&2
}

# Function: Health check
health_check() {
    if /app/bin/uveddi health --timeout 5; then
        return 0
    else
        return 1
    fi
}

# Function: Initialize application
initialize() {
    log "Initializing Uveddi Analysis Engine..."

    # Ensure directories exist
    mkdir -p /app/data /app/cache /app/logs /app/temp

    # Validate configuration
    if [ ! -f "$CONFIG_FILE" ]; then
        log "ERROR: Configuration file not found: $CONFIG_FILE"
        exit 1
    fi

    log "Using configuration: $CONFIG_FILE"

    # Pre-warm cache if enabled
    if [ "$CACHE_WARMUP" = "true" ]; then
        log "Pre-warming cache..."
        /app/bin/uveddi cache warmup --config "$CONFIG_FILE" || log "Cache warmup failed (non-fatal)"
    fi

    log "Initialization complete"
}

# Function: Start main application
start_application() {
    log "Starting Uveddi Analysis Engine..."

    exec /app/bin/uveddi "$@" \
        --config "$CONFIG_FILE" \
        --log-level "$LOG_LEVEL"
}

# Function: Graceful shutdown handler
shutdown_handler() {
    log "Received shutdown signal, initiating graceful shutdown..."

    # Send SIGTERM to child processes
    if [ -n "${UVEDDI_PID:-}" ]; then
        kill -TERM "$UVEDDI_PID" 2>/dev/null || true

        # Wait for graceful shutdown (max 30 seconds)
        for i in $(seq 1 30); do
            if ! kill -0 "$UVEDDI_PID" 2>/dev/null; then
                log "Application shut down gracefully"
                exit 0
            fi
            sleep 1
        done

        # Force kill if graceful shutdown failed
        log "Forcing shutdown..."
        kill -KILL "$UVEDDI_PID" 2>/dev/null || true
    fi

    exit 0
}

# Set up signal handlers
trap 'shutdown_handler' TERM INT

# Main execution
main() {
    # Initialize
    initialize

    # Start application in background to handle signals
    start_application "$@" &
    UVEDDI_PID=$!

    # Wait for completion or signal
    wait "$UVEDDI_PID"
}

# Handle different commands
case "${1:-serve}" in
    "serve"|"server")
        main "$@"
        ;;
    "analyze")
        log "Running one-time analysis..."
        exec /app/bin/uveddi "$@" --config "$CONFIG_FILE"
        ;;
    "health")
        health_check
        ;;
    "cache")
        log "Cache management operation..."
        exec /app/bin/uveddi "$@" --config "$CONFIG_FILE"
        ;;
    *)
        log "Running custom command: $*"
        exec /app/bin/uveddi "$@"
        ;;
esac