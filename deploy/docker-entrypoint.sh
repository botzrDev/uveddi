#!/bin/bash
set -e

# Secure Docker entrypoint script for Uveddi
# Runs as non-root user with security best practices

# Function to log messages with timestamp
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') [ENTRYPOINT] $1"
}

# Function to handle signals gracefully
cleanup() {
    log "Received shutdown signal, cleaning up..."
    if [ ! -z "$UVEDDI_PID" ]; then
        kill -TERM "$UVEDDI_PID" 2>/dev/null || true
        wait "$UVEDDI_PID" 2>/dev/null || true
    fi
    log "Cleanup completed"
    exit 0
}

# Set up signal handlers
trap cleanup SIGTERM SIGINT

# Validate environment
log "Starting Uveddi container with security hardening"
log "Running as user: $(id -u):$(id -g)"
log "Working directory: $(pwd)"

# Ensure we're running as non-root
if [ "$(id -u)" = "0" ]; then
    log "ERROR: Container should not run as root user"
    exit 1
fi

# Validate required directories exist and are writable
for dir in "/app/data" "/app/cache" "/app/logs"; do
    if [ ! -d "$dir" ]; then
        log "ERROR: Required directory $dir does not exist"
        exit 1
    fi
    if [ ! -w "$dir" ]; then
        log "ERROR: Directory $dir is not writable"
        exit 1
    fi
done

# Set secure file creation mask
umask 027

# Security: Limit file descriptors
ulimit -n 1024

# Security: Limit memory if not set by container runtime
if [ -z "$MEMORY_LIMIT" ]; then
    # Default to 2GB memory limit
    ulimit -v 2097152  # 2GB in KB
fi

# Set default configuration if not provided
export RUST_LOG="${RUST_LOG:-info}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
export DATABASE_URL="${DATABASE_URL:-sqlite:///app/data/uveddi.db}"
export CACHE_DIR="${CACHE_DIR:-/app/cache}"
export LOG_DIR="${LOG_DIR:-/app/logs}"

# Validate configuration
log "Configuration:"
log "  RUST_LOG: ${RUST_LOG}"
log "  DATABASE_URL: ${DATABASE_URL}"
log "  CACHE_DIR: ${CACHE_DIR}"
log "  LOG_DIR: ${LOG_DIR}"

# Security: Validate environment variables to prevent injection
validate_env_var() {
    local var_name="$1"
    local var_value="$2"
    
    # Check for dangerous characters
    if echo "$var_value" | grep -qE '[;&|`$()]'; then
        log "ERROR: Environment variable $var_name contains dangerous characters"
        exit 1
    fi
}

validate_env_var "RUST_LOG" "$RUST_LOG"
validate_env_var "DATABASE_URL" "$DATABASE_URL"

# Create log file with proper permissions
LOG_FILE="${LOG_DIR}/uveddi.log"
touch "$LOG_FILE"
chmod 640 "$LOG_FILE"

# Health check function
health_check() {
    curl -sf http://localhost:8080/health > /dev/null 2>&1
    return $?
}

# Start the application
log "Starting Uveddi application..."

if [ "$1" = "uveddi" ] || [ "$#" -eq 0 ]; then
    # Default command: start Uveddi
    log "Executing: /usr/local/bin/uveddi"
    
    # Start Uveddi in background so we can handle signals
    /usr/local/bin/uveddi >> "$LOG_FILE" 2>&1 &
    UVEDDI_PID=$!
    
    log "Uveddi started with PID: $UVEDDI_PID"
    
    # Wait for the application to start
    sleep 5
    
    # Initial health check
    if health_check; then
        log "Health check passed - application is ready"
    else
        log "WARNING: Initial health check failed"
    fi
    
    # Wait for the process and handle signals
    wait "$UVEDDI_PID"
    exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log "Uveddi exited with code: $exit_code"
    else
        log "Uveddi exited normally"
    fi
    
    exit $exit_code
    
elif [ "$1" = "health" ]; then
    # Health check command
    if health_check; then
        echo "OK"
        exit 0
    else
        echo "FAILED"
        exit 1
    fi
    
else
    # Custom command
    log "Executing custom command: $*"
    exec "$@"
fi