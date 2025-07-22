#!/bin/bash
# scripts/deploy-uv243-production.sh
# Zero-downtime production deployment automation for UV-243

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEPLOY_USER="${DEPLOY_USER:-uveddi}"
DEPLOY_HOST="${DEPLOY_HOST:-localhost}"
SERVICE_NAME="uveddi"
BACKUP_RETENTION_DAYS=30
HEALTH_CHECK_TIMEOUT=60
ROLLBACK_TIMEOUT=300

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

# Error handling
cleanup_on_error() {
    local exit_code=$?
    log_error "Deployment failed with exit code $exit_code"
    log_info "Starting automatic rollback..."
    
    if [ -f "/tmp/uveddi_deployment.lock" ]; then
        rollback_deployment
        rm -f "/tmp/uveddi_deployment.lock"
    fi
    
    exit $exit_code
}

trap cleanup_on_error ERR

# Deployment functions
check_prerequisites() {
    log_step "Checking deployment prerequisites"
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_error "Not in a valid Uveddi project directory"
        return 1
    fi
    
    # Check for deployment lock
    if [ -f "/tmp/uveddi_deployment.lock" ]; then
        log_error "Another deployment is in progress (lock file exists)"
        return 1
    fi
    
    # Create deployment lock
    echo "$$" > "/tmp/uveddi_deployment.lock"
    
    # Check required commands
    local required_commands=("cargo" "systemctl" "curl" "rsync")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            log_error "Required command '$cmd' not found"
            return 1
        fi
    done
    
    log_success "Prerequisites check passed"
}

run_pre_deployment_validation() {
    log_step "Running pre-deployment validation"
    
    cd "$PROJECT_ROOT"
    
    # Run comprehensive validation
    if [ -f "$SCRIPT_DIR/validate-uv243-requirements.sh" ]; then
        log_info "Running UV-243 requirements validation..."
        if ! "$SCRIPT_DIR/validate-uv243-requirements.sh"; then
            log_error "Pre-deployment validation failed"
            return 1
        fi
    else
        log_warning "UV-243 validation script not found, running basic checks..."
        
        # Basic validation
        log_info "Running basic test suite..."
        cargo test --release --all-features
        
        log_info "Running security audit..."
        cargo audit
        
        log_info "Checking code formatting..."
        cargo fmt -- --check
        
        log_info "Running clippy..."
        cargo clippy --all-features --all-targets -- -D warnings
    fi
    
    log_success "Pre-deployment validation completed"
}

backup_current_deployment() {
    log_step "Creating backup of current deployment"
    
    local backup_dir="/opt/uveddi/backups/$(date +%Y%m%d_%H%M%S)"
    local binary_path="/usr/local/bin/$SERVICE_NAME"
    local config_dir="/etc/uveddi"
    local data_dir="/var/lib/uveddi"
    
    # Create backup directory
    sudo mkdir -p "$backup_dir"
    
    # Backup binary if it exists
    if [ -f "$binary_path" ]; then
        sudo cp "$binary_path" "$backup_dir/"
        log_info "Binary backed up to $backup_dir/"
    fi
    
    # Backup configuration
    if [ -d "$config_dir" ]; then
        sudo cp -r "$config_dir" "$backup_dir/"
        log_info "Configuration backed up"
    fi
    
    # Backup data (with size limit)
    if [ -d "$data_dir" ]; then
        local data_size=$(sudo du -sm "$data_dir" | cut -f1)
        if [ "$data_size" -lt 1000 ]; then  # Less than 1GB
            sudo cp -r "$data_dir" "$backup_dir/"
            log_info "Data directory backed up"
        else
            log_warning "Data directory too large ($data_size MB), skipping backup"
        fi
    fi
    
    # Store backup path for potential rollback
    echo "$backup_dir" > "/tmp/uveddi_last_backup.txt"
    
    log_success "Backup created at $backup_dir"
}

build_optimized_release() {
    log_step "Building optimized release"
    
    cd "$PROJECT_ROOT"
    
    # Clean previous build artifacts
    log_info "Cleaning previous builds..."
    cargo clean
    
    # Build with optimizations
    log_info "Building optimized release..."
    RUSTFLAGS="-C target-cpu=native" cargo build --release --all-features
    
    # Verify binary
    if [ ! -f "target/release/$SERVICE_NAME" ]; then
        log_error "Release binary not found"
        return 1
    fi
    
    # Check binary size and dependencies
    local binary_size=$(stat -c%s "target/release/$SERVICE_NAME")
    log_info "Binary size: $(numfmt --to=iec $binary_size)"
    
    # Strip debug symbols for production
    strip "target/release/$SERVICE_NAME"
    local stripped_size=$(stat -c%s "target/release/$SERVICE_NAME")
    log_info "Stripped binary size: $(numfmt --to=iec $stripped_size)"
    
    log_success "Optimized release build completed"
}

deploy_documentation() {
    log_step "Deploying documentation"
    
    cd "$PROJECT_ROOT"
    
    # Build documentation
    log_info "Building Rust documentation..."
    cargo doc --release --all-features --no-deps
    
    log_info "Building mdbook documentation..."
    mdbook build docs/
    
    # Deploy documentation
    local doc_dir="/var/www/uveddi-docs"
    sudo mkdir -p "$doc_dir"
    
    # Copy mdbook output
    if [ -d "docs/book" ]; then
        sudo rsync -av docs/book/ "$doc_dir/"
        log_info "mdbook documentation deployed"
    fi
    
    # Copy Rust docs
    if [ -d "target/doc" ]; then
        sudo mkdir -p "$doc_dir/api"
        sudo rsync -av target/doc/ "$doc_dir/api/"
        log_info "API documentation deployed"
    fi
    
    # Set proper permissions
    sudo chown -R www-data:www-data "$doc_dir" 2>/dev/null || true
    
    log_success "Documentation deployment completed"
}

perform_blue_green_deployment() {
    log_step "Performing blue-green deployment"
    
    local new_binary="$PROJECT_ROOT/target/release/$SERVICE_NAME"
    local current_binary="/usr/local/bin/$SERVICE_NAME"
    local staging_binary="/usr/local/bin/${SERVICE_NAME}-staging"
    local backup_binary="/usr/local/bin/${SERVICE_NAME}-backup"
    
    # Stop any existing staging service
    sudo systemctl stop "${SERVICE_NAME}-staging" 2>/dev/null || true
    
    # Deploy new binary to staging location
    log_info "Deploying new binary to staging..."
    sudo cp "$new_binary" "$staging_binary"
    sudo chmod +x "$staging_binary"
    
    # Create staging systemd service if it doesn't exist
    if [ ! -f "/etc/systemd/system/${SERVICE_NAME}-staging.service" ]; then
        log_info "Creating staging service configuration..."
        sudo tee "/etc/systemd/system/${SERVICE_NAME}-staging.service" > /dev/null <<EOF
[Unit]
Description=Uveddi Code Analysis Tool (Staging)
After=network.target

[Service]
Type=simple
User=$DEPLOY_USER
ExecStart=$staging_binary --port 8081
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=UVEDDI_ENV=staging

[Install]
WantedBy=multi-user.target
EOF
        sudo systemctl daemon-reload
    fi
    
    # Start staging service
    log_info "Starting staging service..."
    sudo systemctl start "${SERVICE_NAME}-staging"
    
    # Wait for service to be ready
    log_info "Waiting for staging service to be ready..."
    local ready=false
    for i in {1..30}; do
        if curl -f -s "http://localhost:8081/health" >/dev/null 2>&1; then
            ready=true
            break
        fi
        sleep 2
    done
    
    if [ "$ready" = false ]; then
        log_error "Staging service health check failed"
        return 1
    fi
    
    log_success "Staging service is healthy"
    
    # Switch production traffic
    log_info "Switching production traffic..."
    
    # Backup current binary
    if [ -f "$current_binary" ]; then
        sudo mv "$current_binary" "$backup_binary"
    fi
    
    # Move staging binary to production
    sudo mv "$staging_binary" "$current_binary"
    
    # Restart production service
    sudo systemctl restart "$SERVICE_NAME"
    
    log_success "Blue-green deployment completed"
}

run_health_checks() {
    log_step "Running post-deployment health checks"
    
    local health_url="http://localhost:8080/health"
    local api_url="http://localhost:8080/api/v1/status"
    
    # Basic health check
    log_info "Performing basic health check..."
    local healthy=false
    for i in $(seq 1 $HEALTH_CHECK_TIMEOUT); do
        if curl -f -s "$health_url" >/dev/null 2>&1; then
            healthy=true
            break
        fi
        sleep 1
    done
    
    if [ "$healthy" = false ]; then
        log_error "Basic health check failed after ${HEALTH_CHECK_TIMEOUT}s"
        return 1
    fi
    
    log_success "Basic health check passed"
    
    # API functionality check
    log_info "Checking API functionality..."
    if curl -f -s "$api_url" | grep -q "ok"; then
        log_success "API functionality check passed"
    else
        log_warning "API functionality check failed (non-critical)"
    fi
    
    # Service status check
    log_info "Checking service status..."
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        log_success "Service is active and running"
    else
        log_error "Service is not running"
        return 1
    fi
    
    # Memory and CPU check
    log_info "Checking resource usage..."
    local pid=$(pgrep -f "$SERVICE_NAME" | head -1)
    if [ -n "$pid" ]; then
        local memory_mb=$(ps -p "$pid" -o rss= | awk '{print int($1/1024)}')
        local cpu_percent=$(ps -p "$pid" -o %cpu= | awk '{print int($1)}')
        log_info "Resource usage: ${memory_mb}MB RAM, ${cpu_percent}% CPU"
        
        # Alert if resource usage is too high
        if [ "$memory_mb" -gt 1024 ]; then
            log_warning "High memory usage: ${memory_mb}MB"
        fi
        if [ "$cpu_percent" -gt 50 ]; then
            log_warning "High CPU usage: ${cpu_percent}%"
        fi
    fi
    
    log_success "Post-deployment health checks completed"
}

rollback_deployment() {
    log_step "Rolling back deployment"
    
    local backup_binary="/usr/local/bin/${SERVICE_NAME}-backup"
    local current_binary="/usr/local/bin/$SERVICE_NAME"
    local backup_dir
    
    # Get last backup directory
    if [ -f "/tmp/uveddi_last_backup.txt" ]; then
        backup_dir=$(cat "/tmp/uveddi_last_backup.txt")
    fi
    
    # Rollback binary
    if [ -f "$backup_binary" ]; then
        log_info "Rolling back to previous binary..."
        sudo mv "$backup_binary" "$current_binary"
        sudo systemctl restart "$SERVICE_NAME"
        
        # Wait for rollback to be healthy
        local healthy=false
        for i in $(seq 1 $ROLLBACK_TIMEOUT); do
            if curl -f -s "http://localhost:8080/health" >/dev/null 2>&1; then
                healthy=true
                break
            fi
            sleep 1
        done
        
        if [ "$healthy" = true ]; then
            log_success "Rollback completed successfully"
        else
            log_error "Rollback health check failed"
        fi
    else
        log_error "No backup binary found for rollback"
    fi
    
    # Rollback configuration if needed
    if [ -n "$backup_dir" ] && [ -d "$backup_dir/uveddi" ]; then
        log_info "Rolling back configuration..."
        sudo cp -r "$backup_dir/uveddi" "/etc/"
    fi
}

cleanup_old_backups() {
    log_step "Cleaning up old backups"
    
    local backup_root="/opt/uveddi/backups"
    if [ -d "$backup_root" ]; then
        # Remove backups older than retention period
        sudo find "$backup_root" -type d -name "20*" -mtime +$BACKUP_RETENTION_DAYS -exec rm -rf {} \; 2>/dev/null || true
        log_info "Cleaned up backups older than $BACKUP_RETENTION_DAYS days"
    fi
}

finalize_deployment() {
    log_step "Finalizing deployment"
    
    # Stop staging service
    sudo systemctl stop "${SERVICE_NAME}-staging" 2>/dev/null || true
    
    # Remove deployment lock
    rm -f "/tmp/uveddi_deployment.lock"
    
    # Update deployment timestamp
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)" | sudo tee "/var/lib/uveddi/last_deployment" > /dev/null || true
    
    # Log deployment success
    local version=$(grep '^version =' "$PROJECT_ROOT/Cargo.toml" | cut -d'"' -f2)
    log_info "Deployment completed - version: $version"
    
    log_success "Deployment finalized successfully"
}

# Main deployment function
main() {
    local dry_run=false
    local skip_validation=false
    local skip_backup=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --skip-validation)
                skip_validation=true
                shift
                ;;
            --skip-backup)
                skip_backup=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --dry-run         Perform all checks but don't deploy"
                echo "  --skip-validation Skip pre-deployment validation"
                echo "  --skip-backup     Skip backup creation"
                echo "  --help           Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    log_info "🚀 Starting UV-243 production deployment..."
    log_info "Deployment configuration:"
    log_info "  - Dry run: $dry_run"
    log_info "  - Skip validation: $skip_validation"
    log_info "  - Skip backup: $skip_backup"
    log_info "  - Deploy user: $DEPLOY_USER"
    log_info "  - Deploy host: $DEPLOY_HOST"
    
    # Execute deployment steps
    check_prerequisites
    
    if [ "$skip_validation" = false ]; then
        run_pre_deployment_validation
    fi
    
    if [ "$skip_backup" = false ]; then
        backup_current_deployment
    fi
    
    build_optimized_release
    
    if [ "$dry_run" = true ]; then
        log_info "🧪 Dry run mode - skipping actual deployment"
        log_success "Dry run completed successfully"
        rm -f "/tmp/uveddi_deployment.lock"
        exit 0
    fi
    
    deploy_documentation
    perform_blue_green_deployment
    run_health_checks
    cleanup_old_backups
    finalize_deployment
    
    log_success "🎉 UV-243 deployed successfully to production!"
    log_info "Service status: $(systemctl is-active $SERVICE_NAME)"
    log_info "Documentation: http://$DEPLOY_HOST/docs/"
    log_info "API Health: http://$DEPLOY_HOST:8080/health"
}

# Execute main function with all arguments
main "$@"