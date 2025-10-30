#!/bin/bash

# Rendering Service Test Management Script
# Starts the rendering service for test environments with proper setup and health checks

set -e  # Exit on any error

# Configuration
RENDERING_SERVICE_DIR="rendering-service"
HEALTH_CHECK_URL="http://localhost:3001/health"
PID_FILE="../service.pid"
STARTUP_TIMEOUT=30  # seconds
HEALTH_CHECK_RETRIES=10

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if service is already running
check_existing_service() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if ps -p $pid > /dev/null 2>&1; then
            log_warn "Service already running with PID $pid"
            log_info "Testing existing service health..."
            if curl -f -s "$HEALTH_CHECK_URL" > /dev/null 2>&1; then
                log_info "Existing service is healthy"
                return 0
            else
                log_warn "Existing service is unhealthy, will restart"
                kill $pid 2>/dev/null || true
                sleep 2
                rm -f "$PID_FILE"
            fi
        else
            log_warn "PID file exists but process not running, cleaning up"
            rm -f "$PID_FILE"
        fi
    fi
    return 1
}

# Function to wait for service startup
wait_for_service() {
    local count=0
    log_info "Waiting for service to start..."
    
    while [ $count -lt $HEALTH_CHECK_RETRIES ]; do
        # Check if the process is still running
        if ! ps -p $SERVICE_PID > /dev/null 2>&1; then
            log_error "Service process died unexpectedly"
            return 1
        fi
        
        if curl -f -s "$HEALTH_CHECK_URL" > /dev/null 2>&1; then
            log_info "Service is healthy and ready"
            return 0
        fi
        count=$((count + 1))
        echo -n "."
        sleep 3
    done
    
    echo ""
    log_error "Service failed to start within $STARTUP_TIMEOUT seconds"
    return 1
}

# Function to verify service health
verify_service_health() {
    log_info "Verifying service health..."
    
    local response=$(curl -f -s "$HEALTH_CHECK_URL" 2>&1)
    if [ $? -eq 0 ]; then
        log_info "Health check successful"
        echo "$response" | grep -q '"status":"healthy"' && log_info "Service reports healthy status"
        return 0
    else
        log_error "Health check failed: $response"
        return 1
    fi
}

# Function to cleanup on exit
cleanup() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if ps -p $pid > /dev/null 2>&1; then
            log_info "Cleaning up background process (PID: $pid)"
            kill $pid 2>/dev/null || true
        fi
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Main execution
main() {
    log_info "Starting rendering service for tests..."
    
    # Check if already running
    if check_existing_service; then
        log_info "Service startup completed (already running)"
        exit 0
    fi
    
    # Verify rendering service directory exists
    if [ ! -d "$RENDERING_SERVICE_DIR" ]; then
        log_error "Rendering service directory not found: $RENDERING_SERVICE_DIR"
        exit 1
    fi
    
    # Navigate to rendering service directory
    cd "$RENDERING_SERVICE_DIR"
    
    # Install dependencies
    log_info "Installing npm dependencies..."
    if ! npm install --silent; then
        log_error "Failed to install npm dependencies"
        exit 1
    fi
    
    # Install Playwright browsers
    log_info "Installing Playwright browsers..."
    if ! npm run install-browsers --silent; then
        log_error "Failed to install Playwright browsers"
        exit 1
    fi
    
    # Start service in background
    log_info "Starting rendering service..."
    npm start > ../service.log 2>&1 &
    SERVICE_PID=$!
    
    # Store PID for cleanup
    echo $SERVICE_PID > "$PID_FILE"
    log_info "Service started with PID: $SERVICE_PID"
    
    # Wait for service to be ready
    if ! wait_for_service; then
        log_error "Service failed to start properly"
        kill $SERVICE_PID 2>/dev/null || true
        rm -f "$PID_FILE"
        exit 1
    fi
    
    # Verify health
    if ! verify_service_health; then
        log_error "Service health check failed"
        kill $SERVICE_PID 2>/dev/null || true
        rm -f "$PID_FILE"
        exit 1
    fi
    
    log_info "Rendering service successfully started and verified"
    log_info "Health check URL: $HEALTH_CHECK_URL"
    log_info "PID file: $PID_FILE"
    log_info "Service log: service.log"
    
    # Don't cleanup on successful exit
    trap - EXIT
    exit 0
}

# Run main function
main "$@"