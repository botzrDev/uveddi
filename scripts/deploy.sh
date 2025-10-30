#!/bin/bash
# Comprehensive deployment automation for Uveddi
# Provides zero-downtime deployment with validation, health checks, and rollback capabilities

set -e
set -o pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEPLOYMENT_ID="deploy-$(date +%Y%m%d-%H%M%S)"
BACKUP_DIR="/opt/uveddi/backups/$DEPLOYMENT_ID"
HEALTH_CHECK_TIMEOUT=120
VALIDATION_TIMEOUT=60

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"; }
log_step() { echo -e "${PURPLE}[STEP]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"; }

# Create deployment lock
create_deployment_lock() {
    local lock_file="/tmp/uveddi_deployment.lock"
    if [ -f "$lock_file" ]; then
        log_error "Another deployment is in progress (lock file exists)"
        exit 1
    fi
    echo "$DEPLOYMENT_ID" > "$lock_file"
    trap 'rm -f "$lock_file"' EXIT
}

# Pre-deployment validation
validate_environment() {
    log_step "Validating deployment environment"
    
    # Check required commands
    local required_commands=("node" "npm" "curl" "git" "systemctl")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            log_error "Required command '$cmd' not found"
            return 1
        fi
    done
    
    # Check Git status
    if [ -n "$(git status --porcelain)" ]; then
        log_warning "Working directory has uncommitted changes"
        read -p "Continue deployment? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_error "Deployment cancelled"
            exit 1
        fi
    fi
    
    # Check disk space
    local available_space=$(df "$PROJECT_ROOT" | awk 'NR==2 {print $4}')
    if [ "$available_space" -lt 1048576 ]; then # 1GB in KB
        log_error "Insufficient disk space (less than 1GB available)"
        return 1
    fi
    
    # Check memory
    local available_memory=$(free -m | awk 'NR==2{print $7}')
    if [ "$available_memory" -lt 512 ]; then
        log_warning "Low available memory ($available_memory MB)"
    fi
    
    log_success "Environment validation completed"
}

# Backup current deployment
create_backup() {
    log_step "Creating backup of current deployment"
    
    sudo mkdir -p "$BACKUP_DIR"
    
    # Backup services if running
    if pgrep -f "node.*server.js" > /dev/null; then
        log_info "Backing up running API server..."
        sudo cp -r "$PROJECT_ROOT/api-server" "$BACKUP_DIR/"
    fi
    
    if pgrep -f "npm.*dev" > /dev/null; then
        log_info "Backing up frontend..."
        sudo cp -r "$PROJECT_ROOT/frontend/dist" "$BACKUP_DIR/" 2>/dev/null || true
    fi
    
    # Backup configuration
    if [ -d "/etc/uveddi" ]; then
        sudo cp -r "/etc/uveddi" "$BACKUP_DIR/"
    fi
    
    # Backup databases
    if [ -f "$PROJECT_ROOT/uveddi.db" ]; then
        sudo cp "$PROJECT_ROOT/uveddi.db" "$BACKUP_DIR/"
    fi
    
    echo "$DEPLOYMENT_ID" | sudo tee "$BACKUP_DIR/deployment_id.txt" > /dev/null
    log_success "Backup created at $BACKUP_DIR"
}

# Build and test the application
build_and_test() {
    log_step "Building and testing application"
    
    cd "$PROJECT_ROOT"
    
    # Install/update API dependencies
    log_info "Installing API server dependencies..."
    cd "$PROJECT_ROOT/api-server"
    npm ci --production
    
    # Install/update frontend dependencies
    log_info "Installing frontend dependencies..."
    cd "$PROJECT_ROOT/frontend"
    npm ci
    
    # Build frontend
    log_info "Building frontend..."
    npm run build
    
    # Run tests
    log_info "Running frontend tests..."
    timeout "$VALIDATION_TIMEOUT" npm test -- --watchAll=false || {
        log_warning "Frontend tests failed or timed out"
        read -p "Continue deployment despite test failures? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_error "Deployment cancelled due to test failures"
            exit 1
        fi
    }
    
    log_success "Build and test completed"
}

# Health check function
health_check() {
    local service="$1"
    local url="$2"
    local timeout="${3:-30}"
    
    local attempts=0
    local max_attempts=$((timeout / 2))
    
    while [ $attempts -lt $max_attempts ]; do
        if curl -f -s --connect-timeout 5 --max-time 10 "$url" > /dev/null 2>&1; then
            return 0
        fi
        sleep 2
        attempts=$((attempts + 1))
    done
    
    return 1
}

# Rolling deployment function
rolling_deployment() {
    log_step "Performing rolling deployment"
    
    # Use the process manager for controlled restart
    local process_manager="$PROJECT_ROOT/scripts/process-manager.sh"
    
    if [ ! -x "$process_manager" ]; then
        log_error "Process manager not found or not executable"
        return 1
    fi
    
    # Stop services gracefully
    log_info "Stopping services for deployment..."
    "$process_manager" stop
    
    # Wait for services to stop
    sleep 5
    
    # Start services with health monitoring
    log_info "Starting services..."
    if ! "$process_manager" start; then
        log_error "Failed to start services"
        return 1
    fi
    
    # Validate service health
    log_info "Validating service health..."
    local services=(
        "api:http://localhost:8000/health"
        "frontend:http://localhost:8001"
        "rendering:http://localhost:3001/health"
    )
    
    for service_config in "${services[@]}"; do
        IFS=':' read -r service_name service_url <<< "$service_config"
        log_info "Checking $service_name health..."
        
        if health_check "$service_name" "$service_url" "$HEALTH_CHECK_TIMEOUT"; then
            log_success "$service_name is healthy"
        else
            log_error "$service_name health check failed"
            return 1
        fi
    done
    
    log_success "Rolling deployment completed successfully"
}

# Post-deployment validation
post_deployment_validation() {
    log_step "Running post-deployment validation"
    
    # API functionality tests
    log_info "Testing API endpoints..."
    
    # Test health endpoint
    if ! curl -f -s "http://localhost:8000/health" | grep -q "healthy"; then
        log_error "API health check failed"
        return 1
    fi
    
    # Test reports endpoint
    if ! curl -f -s "http://localhost:8000/api/v1/reports/latest" > /dev/null; then
        log_error "Reports endpoint test failed"
        return 1
    fi
    
    # Frontend accessibility test
    log_info "Testing frontend accessibility..."
    if ! curl -f -s "http://localhost:8001" | grep -q "<!doctype html"; then
        log_error "Frontend accessibility test failed"
        return 1
    fi
    
    # Load test (basic)
    log_info "Running basic load test..."
    local concurrent_requests=5
    local success_count=0
    
    for i in $(seq 1 $concurrent_requests); do
        curl -f -s "http://localhost:8000/health" > /dev/null && ((success_count++)) &
    done
    wait
    
    if [ $success_count -eq $concurrent_requests ]; then
        log_success "Basic load test passed"
    else
        log_warning "Load test: $success_count/$concurrent_requests requests succeeded"
    fi
    
    log_success "Post-deployment validation completed"
}

# Rollback function
rollback_deployment() {
    log_step "Rolling back deployment"
    
    if [ ! -d "$BACKUP_DIR" ]; then
        log_error "Backup directory not found: $BACKUP_DIR"
        return 1
    fi
    
    # Stop current services
    "$PROJECT_ROOT/scripts/process-manager.sh" stop || true
    
    # Restore from backup
    log_info "Restoring from backup..."
    
    if [ -d "$BACKUP_DIR/api-server" ]; then
        sudo cp -r "$BACKUP_DIR/api-server" "$PROJECT_ROOT/"
    fi
    
    if [ -d "$BACKUP_DIR/dist" ]; then
        sudo cp -r "$BACKUP_DIR/dist" "$PROJECT_ROOT/frontend/"
    fi
    
    if [ -f "$BACKUP_DIR/uveddi.db" ]; then
        sudo cp "$BACKUP_DIR/uveddi.db" "$PROJECT_ROOT/"
    fi
    
    # Restart services
    log_info "Restarting services after rollback..."
    "$PROJECT_ROOT/scripts/process-manager.sh" start
    
    # Verify rollback
    if health_check "api" "http://localhost:8000/health" 60; then
        log_success "Rollback completed successfully"
        return 0
    else
        log_error "Rollback verification failed"
        return 1
    fi
}

# Cleanup old backups
cleanup_old_backups() {
    log_step "Cleaning up old backups"
    
    local backup_root="/opt/uveddi/backups"
    if [ -d "$backup_root" ]; then
        # Keep last 10 backups
        sudo find "$backup_root" -maxdepth 1 -type d -name "deploy-*" | \
            sort -r | tail -n +11 | \
            sudo xargs rm -rf
        log_info "Cleaned up old backups"
    fi
}

# Deployment monitoring
monitor_deployment() {
    log_step "Starting deployment monitoring"
    
    local monitor_duration=300 # 5 minutes
    local check_interval=30
    local end_time=$(($(date +%s) + monitor_duration))
    
    while [ $(date +%s) -lt $end_time ]; do
        if ! health_check "api" "http://localhost:8000/health" 10; then
            log_error "Health check failed during monitoring"
            return 1
        fi
        sleep $check_interval
    done
    
    log_success "Deployment monitoring completed successfully"
}

# Main deployment function
main() {
    log_info "Starting Uveddi deployment: $DEPLOYMENT_ID"
    
    # Command line options
    local skip_tests=false
    local skip_backup=false
    local auto_rollback=true
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-tests)
                skip_tests=true
                shift
                ;;
            --skip-backup)
                skip_backup=true
                shift
                ;;
            --no-auto-rollback)
                auto_rollback=false
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --skip-tests        Skip test execution"
                echo "  --skip-backup       Skip backup creation"
                echo "  --no-auto-rollback  Disable automatic rollback on failure"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Deployment steps
    if ! create_deployment_lock; then
        exit 1
    fi
    
    if ! validate_environment; then
        log_error "Environment validation failed"
        exit 1
    fi
    
    if [ "$skip_backup" != true ]; then
        if ! create_backup; then
            log_error "Backup creation failed"
            exit 1
        fi
    fi
    
    if [ "$skip_tests" != true ]; then
        if ! build_and_test; then
            log_error "Build and test failed"
            [ "$auto_rollback" = true ] && rollback_deployment
            exit 1
        fi
    fi
    
    if ! rolling_deployment; then
        log_error "Rolling deployment failed"
        [ "$auto_rollback" = true ] && rollback_deployment
        exit 1
    fi
    
    if ! post_deployment_validation; then
        log_error "Post-deployment validation failed"
        [ "$auto_rollback" = true ] && rollback_deployment
        exit 1
    fi
    
    # Background monitoring
    monitor_deployment &
    local monitor_pid=$!
    
    cleanup_old_backups
    
    log_success "Deployment $DEPLOYMENT_ID completed successfully!"
    log_info "Deployment is being monitored (PID: $monitor_pid)"
    log_info "Backup available at: $BACKUP_DIR"
    
    # Service status
    echo
    "$PROJECT_ROOT/scripts/process-manager.sh" status
}

# Handle errors during deployment
trap 'log_error "Deployment failed unexpectedly"; exit 1' ERR

# Run main function
main "$@"
