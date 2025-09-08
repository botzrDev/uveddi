#!/bin/bash
# Process Manager for Uveddi Services
# Provides unified start/stop/restart/status management for all services

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PID_DIR="$PROJECT_ROOT/.pids"
LOG_DIR="$PROJECT_ROOT/logs"

# Service Configuration
declare -A SERVICES=(
    ["api"]="$PROJECT_ROOT/api-server:node server.js:8000:/health"
    ["frontend"]="$PROJECT_ROOT/frontend:npm run dev:8001:/"
    ["rendering"]="$PROJECT_ROOT/rendering-service:npm start:3001:/health"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $(date '+%H:%M:%S') - $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $(date '+%H:%M:%S') - $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $(date '+%H:%M:%S') - $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $(date '+%H:%M:%S') - $1"; }

# Initialize directories
init_dirs() {
    mkdir -p "$PID_DIR" "$LOG_DIR"
}

# Parse service configuration: dir:command:port:health_path
parse_service_config() {
    local service="$1"
    local config="${SERVICES[$service]}"
    
    if [ -z "$config" ]; then
        log_error "Unknown service: $service"
        return 1
    fi
    
    IFS=':' read -r SERVICE_DIR SERVICE_CMD SERVICE_PORT HEALTH_PATH <<< "$config"
}

# Check if port is in use
check_port() {
    local port="$1"
    ss -tuln | grep -q ":$port "
}

# Get PID for service
get_service_pid() {
    local service="$1"
    local pid_file="$PID_DIR/$service.pid"
    
    if [ -f "$pid_file" ]; then
        cat "$pid_file"
    fi
}

# Check if service is running
is_service_running() {
    local service="$1"
    local pid=$(get_service_pid "$service")
    
    if [ -n "$pid" ] && ps -p "$pid" > /dev/null 2>&1; then
        return 0
    fi
    return 1
}

# Health check for service
health_check() {
    local service="$1"
    parse_service_config "$service"
    
    local health_url="http://localhost:$SERVICE_PORT$HEALTH_PATH"
    curl -f -s --connect-timeout 5 --max-time 10 "$health_url" > /dev/null 2>&1
}

# Start a service
start_service() {
    local service="$1"
    local force="$2"
    
    parse_service_config "$service"
    
    # Check if already running
    if is_service_running "$service" && [ "$force" != "force" ]; then
        log_warning "Service $service is already running"
        return 0
    fi
    
    # Kill existing process if force restart
    if [ "$force" = "force" ]; then
        stop_service "$service"
        sleep 2
    fi
    
    # Check port availability
    if check_port "$SERVICE_PORT"; then
        log_error "Port $SERVICE_PORT is already in use"
        return 1
    fi
    
    log_info "Starting $service service..."
    
    # Change to service directory and start
    cd "$SERVICE_DIR"
    
    # Start service with output redirection
    nohup bash -c "$SERVICE_CMD" > "$LOG_DIR/$service.log" 2>&1 &
    local pid=$!
    
    # Store PID
    echo "$pid" > "$PID_DIR/$service.pid"
    
    # Wait for startup and health check
    log_info "Waiting for $service to start..."
    local attempts=0
    local max_attempts=30
    
    while [ $attempts -lt $max_attempts ]; do
        if health_check "$service"; then
            log_success "$service started successfully (PID: $pid)"
            return 0
        fi
        
        # Check if process is still running
        if ! ps -p "$pid" > /dev/null 2>&1; then
            log_error "$service process died during startup"
            return 1
        fi
        
        sleep 2
        attempts=$((attempts + 1))
    done
    
    log_error "$service failed to start within $((max_attempts * 2)) seconds"
    stop_service "$service"
    return 1
}

# Stop a service
stop_service() {
    local service="$1"
    local pid=$(get_service_pid "$service")
    
    if [ -z "$pid" ]; then
        log_warning "No PID found for $service"
        return 0
    fi
    
    if ! ps -p "$pid" > /dev/null 2>&1; then
        log_warning "Process $pid for $service is not running"
        rm -f "$PID_DIR/$service.pid"
        return 0
    fi
    
    log_info "Stopping $service (PID: $pid)..."
    
    # Try graceful shutdown first
    kill -TERM "$pid" 2>/dev/null || true
    
    # Wait for graceful shutdown
    local attempts=0
    while [ $attempts -lt 10 ] && ps -p "$pid" > /dev/null 2>&1; do
        sleep 1
        attempts=$((attempts + 1))
    done
    
    # Force kill if still running
    if ps -p "$pid" > /dev/null 2>&1; then
        log_warning "Force killing $service..."
        kill -KILL "$pid" 2>/dev/null || true
    fi
    
    # Clean up PID file
    rm -f "$PID_DIR/$service.pid"
    log_success "$service stopped"
}

# Restart a service
restart_service() {
    local service="$1"
    
    log_info "Restarting $service..."
    stop_service "$service"
    sleep 2
    start_service "$service"
}

# Get service status
service_status() {
    local service="$1"
    parse_service_config "$service"
    
    local pid=$(get_service_pid "$service")
    local status="STOPPED"
    local health="UNKNOWN"
    
    if [ -n "$pid" ] && ps -p "$pid" > /dev/null 2>&1; then
        status="RUNNING"
        if health_check "$service"; then
            health="HEALTHY"
        else
            health="UNHEALTHY"
        fi
    fi
    
    printf "%-12s %-8s %-10s %-8s %s\n" "$service" "$status" "$health" "$SERVICE_PORT" "$pid"
}

# Show status of all services
status_all() {
    printf "%-12s %-8s %-10s %-8s %s\n" "SERVICE" "STATUS" "HEALTH" "PORT" "PID"
    echo "------------------------------------------------------------"
    
    for service in "${!SERVICES[@]}"; do
        service_status "$service"
    done
}

# Start all services
start_all() {
    log_info "Starting all Uveddi services..."
    
    # Start in dependency order
    for service in "rendering" "api" "frontend"; do
        if ! start_service "$service"; then
            log_error "Failed to start $service, aborting"
            return 1
        fi
        sleep 2
    done
    
    log_success "All services started successfully"
    status_all
}

# Stop all services
stop_all() {
    log_info "Stopping all Uveddi services..."
    
    # Stop in reverse dependency order
    for service in "frontend" "api" "rendering"; do
        stop_service "$service"
    done
    
    log_success "All services stopped"
}

# Monitor services (continuous health checking)
monitor() {
    log_info "Starting service monitor..."
    echo "Press Ctrl+C to stop monitoring"
    
    while true; do
        clear
        echo "=== Uveddi Service Monitor ==="
        echo "$(date)"
        echo
        status_all
        
        # Check for unhealthy services and attempt restart
        for service in "${!SERVICES[@]}"; do
            if is_service_running "$service" && ! health_check "$service"; then
                log_warning "Service $service is unhealthy, attempting restart..."
                restart_service "$service"
            fi
        done
        
        sleep 10
    done
}

# Main command handler
main() {
    init_dirs
    
    case "${1:-status}" in
        "start")
            if [ -n "$2" ]; then
                start_service "$2"
            else
                start_all
            fi
            ;;
        "stop")
            if [ -n "$2" ]; then
                stop_service "$2"
            else
                stop_all
            fi
            ;;
        "restart")
            if [ -n "$2" ]; then
                restart_service "$2"
            else
                stop_all
                sleep 2
                start_all
            fi
            ;;
        "status")
            if [ -n "$2" ]; then
                service_status "$2"
            else
                status_all
            fi
            ;;
        "monitor")
            monitor
            ;;
        "logs")
            local service="${2:-api}"
            if [ -f "$LOG_DIR/$service.log" ]; then
                tail -f "$LOG_DIR/$service.log"
            else
                log_error "No log file found for $service"
            fi
            ;;
        "health")
            local service="$2"
            if [ -n "$service" ]; then
                if health_check "$service"; then
                    log_success "$service is healthy"
                else
                    log_error "$service is unhealthy"
                    exit 1
                fi
            else
                # Check all services
                local unhealthy=0
                for svc in "${!SERVICES[@]}"; do
                    if ! health_check "$svc"; then
                        log_error "$svc is unhealthy"
                        unhealthy=1
                    fi
                done
                [ $unhealthy -eq 0 ] && log_success "All services are healthy"
                exit $unhealthy
            fi
            ;;
        *)
            echo "Usage: $0 {start|stop|restart|status|monitor|logs|health} [service]"
            echo
            echo "Services: ${!SERVICES[*]}"
            echo
            echo "Commands:"
            echo "  start [service]   - Start service(s)"
            echo "  stop [service]    - Stop service(s)"
            echo "  restart [service] - Restart service(s)"
            echo "  status [service]  - Show service status"
            echo "  monitor          - Continuous monitoring with auto-restart"
            echo "  logs [service]   - Show service logs"
            echo "  health [service] - Check service health"
            exit 1
            ;;
    esac
}

# Handle signals for graceful shutdown
trap 'log_info "Shutting down..."; stop_all; exit 0' SIGINT SIGTERM

main "$@"
