#!/bin/bash
# Comprehensive rollback script for Uveddi deployments
# Supports quick rollback with validation and monitoring

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="/tmp/uveddi-rollback-$(date +%Y%m%d-%H%M%S).log"

# Default values
ENVIRONMENT="${1:-staging}"
REVISION="${2:-0}"  # 0 means previous revision
DRY_RUN="${3:-false}"
CONFIRM="${CONFIRM:-true}"

# Configuration
NAMESPACE="uveddi"
RELEASE_NAME="uveddi"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    local level="$1"
    shift
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [$level] $*" | tee -a "$LOG_FILE"
}

info() { log "INFO" "$@"; }
warn() { log "WARN" "$@"; }
error() { log "ERROR" "$@"; }
debug() { [[ "${DEBUG:-}" == "true" ]] && log "DEBUG" "$@" || true; }

success() {
    echo -e "${GREEN}✅ $*${NC}"
    info "SUCCESS: $*"
}

failure() {
    echo -e "${RED}❌ $*${NC}"
    error "FAILURE: $*"
}

warning() {
    echo -e "${YELLOW}⚠️  $*${NC}"
    warn "WARNING: $*"
}

# Cleanup function
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        failure "Rollback failed with exit code $exit_code"
        echo "Logs available at: $LOG_FILE"
    fi
}

trap cleanup EXIT

# Validation functions
validate_environment() {
    case "$ENVIRONMENT" in
        staging|production)
            info "Rolling back in $ENVIRONMENT environment"
            ;;
        *)
            failure "Invalid environment: $ENVIRONMENT. Must be 'staging' or 'production'"
            exit 1
            ;;
    esac
}

validate_prerequisites() {
    info "Validating prerequisites..."
    
    local required_commands=("kubectl" "helm" "curl" "jq")
    local missing_commands=()
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_commands+=("$cmd")
        fi
    done
    
    if [[ ${#missing_commands[@]} -gt 0 ]]; then
        failure "Missing required commands: ${missing_commands[*]}"
        exit 1
    fi
    
    # Check kubectl connection
    if ! kubectl cluster-info &> /dev/null; then
        failure "kubectl not configured or cluster not accessible"
        exit 1
    fi
    
    # Check if Helm release exists
    if ! helm list -n "$NAMESPACE" | grep -q "$RELEASE_NAME"; then
        failure "Helm release '$RELEASE_NAME' not found in namespace '$NAMESPACE'"
        exit 1
    fi
    
    success "Prerequisites validated"
}

get_current_deployment_info() {
    info "Getting current deployment information..."
    
    # Get current Helm revision
    local current_revision
    current_revision=$(helm list -n "$NAMESPACE" | grep "$RELEASE_NAME" | awk '{print $3}' || echo "unknown")
    
    # Get current image tag
    local current_image
    current_image=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null || echo "unknown")
    
    # Get deployment status
    local ready_replicas
    ready_replicas=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
    local desired_replicas
    desired_replicas=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
    
    info "Current deployment info:"
    info "  Helm revision: $current_revision"
    info "  Container image: $current_image"
    info "  Pod status: $ready_replicas/$desired_replicas ready"
    
    echo "$current_revision" > "/tmp/uveddi-current-revision.txt"
}

determine_target_revision() {
    info "Determining target revision for rollback..."
    
    # Get Helm history
    local helm_history
    helm_history=$(helm history "$RELEASE_NAME" -n "$NAMESPACE" --max 20 -o json)
    
    if [[ -z "$helm_history" ]]; then
        failure "Unable to retrieve Helm release history"
        exit 1
    fi
    
    # Get current revision
    local current_revision
    current_revision=$(echo "$helm_history" | jq -r 'map(select(.status == "deployed")) | .[0].revision' 2>/dev/null || echo "1")
    
    # Determine target revision
    local target_revision
    if [[ "$REVISION" == "0" ]]; then
        # Find the previous successfully deployed revision
        target_revision=$(echo "$helm_history" | jq -r --argjson current "$current_revision" \
            'map(select(.status == "deployed" and .revision < $current)) | sort_by(.revision) | reverse | .[0].revision' 2>/dev/null || echo "")
        
        if [[ -z "$target_revision" || "$target_revision" == "null" ]]; then
            failure "No previous successful revision found to rollback to"
            exit 1
        fi
    else
        target_revision="$REVISION"
        
        # Validate that the target revision exists and was successful
        local revision_status
        revision_status=$(echo "$helm_history" | jq -r --argjson target "$target_revision" \
            'map(select(.revision == $target)) | .[0].status' 2>/dev/null || echo "")
        
        if [[ -z "$revision_status" || "$revision_status" == "null" ]]; then
            failure "Revision $target_revision not found in release history"
            exit 1
        fi
        
        if [[ "$revision_status" != "deployed" ]]; then
            warning "Target revision $target_revision was not successfully deployed (status: $revision_status)"
            if [[ "$CONFIRM" == "true" ]] && [[ "$DRY_RUN" != "true" ]]; then
                read -p "Do you want to continue? (y/N): " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                    info "Rollback cancelled by user"
                    exit 0
                fi
            fi
        fi
    fi
    
    info "Rollback target:"
    info "  Current revision: $current_revision"
    info "  Target revision: $target_revision"
    
    echo "$target_revision" > "/tmp/uveddi-target-revision.txt"
    REVISION="$target_revision"
}

show_rollback_plan() {
    info "Rollback Plan Summary:"
    echo "----------------------------------------"
    echo "Environment: $ENVIRONMENT"
    echo "Namespace: $NAMESPACE"
    echo "Release: $RELEASE_NAME"
    
    local current_revision
    current_revision=$(cat "/tmp/uveddi-current-revision.txt" 2>/dev/null || echo "unknown")
    
    echo "Current Revision: $current_revision"
    echo "Target Revision: $REVISION"
    echo "Dry Run: $DRY_RUN"
    
    # Get target revision info
    local target_info
    target_info=$(helm history "$RELEASE_NAME" -n "$NAMESPACE" --max 20 -o json | \
                  jq -r --argjson target "$REVISION" 'map(select(.revision == $target)) | .[0]' 2>/dev/null || echo "{}")
    
    if [[ "$target_info" != "{}" ]] && [[ "$target_info" != "null" ]]; then
        local target_date
        target_date=$(echo "$target_info" | jq -r '.updated' 2>/dev/null || echo "unknown")
        local target_description
        target_description=$(echo "$target_info" | jq -r '.description' 2>/dev/null || echo "unknown")
        
        echo "Target Date: $target_date"
        echo "Target Description: $target_description"
    fi
    
    echo "----------------------------------------"
    
    if [[ "$ENVIRONMENT" == "production" ]]; then
        warning "⚠️  PRODUCTION ROLLBACK ⚠️"
        warning "This will rollback the production environment!"
    fi
}

confirm_rollback() {
    if [[ "$CONFIRM" != "true" ]] || [[ "$DRY_RUN" == "true" ]]; then
        return
    fi
    
    echo ""
    if [[ "$ENVIRONMENT" == "production" ]]; then
        warning "You are about to rollback PRODUCTION!"
        read -p "Type 'ROLLBACK PRODUCTION' to confirm: " -r
        if [[ "$REPLY" != "ROLLBACK PRODUCTION" ]]; then
            info "Rollback cancelled"
            exit 0
        fi
    else
        read -p "Do you want to proceed with rollback? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "Rollback cancelled"
            exit 0
        fi
    fi
}

create_rollback_backup() {
    info "Creating pre-rollback backup..."
    
    local backup_dir="/tmp/uveddi-pre-rollback-backup-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$backup_dir"
    
    # Backup current deployment state
    kubectl get deployment uveddi -n "$NAMESPACE" -o yaml > "$backup_dir/deployment.yaml" 2>/dev/null || true
    kubectl get configmap -n "$NAMESPACE" -o yaml > "$backup_dir/configmaps.yaml" 2>/dev/null || true
    kubectl get secret -n "$NAMESPACE" -o yaml | sed 's/data:/data: # [REDACTED]/' > "$backup_dir/secrets.yaml" 2>/dev/null || true
    kubectl get service -n "$NAMESPACE" -o yaml > "$backup_dir/services.yaml" 2>/dev/null || true
    kubectl get ingress -n "$NAMESPACE" -o yaml > "$backup_dir/ingress.yaml" 2>/dev/null || true
    
    # Save Helm values
    helm get values "$RELEASE_NAME" -n "$NAMESPACE" > "$backup_dir/helm-values.yaml" 2>/dev/null || true
    
    info "Pre-rollback backup created at: $backup_dir"
    echo "$backup_dir" > "/tmp/uveddi-rollback-backup.txt"
}

check_current_health() {
    info "Checking current deployment health..."
    
    # Check if deployment exists and get its status
    if kubectl get deployment uveddi -n "$NAMESPACE" &> /dev/null; then
        local ready_replicas
        ready_replicas=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
        local desired_replicas
        desired_replicas=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
        
        info "Current deployment status: $ready_replicas/$desired_replicas pods ready"
        
        if [[ "$ready_replicas" == "0" ]]; then
            warning "Current deployment has no ready pods - rollback may be necessary"
        elif [[ "$ready_replicas" != "$desired_replicas" ]]; then
            warning "Current deployment is partially unhealthy - rollback may help"
        else
            warning "Current deployment appears healthy - confirm rollback is necessary"
        fi
    else
        warning "No current deployment found"
    fi
    
    # Check recent events
    local error_events
    error_events=$(kubectl get events -n "$NAMESPACE" --field-selector type=Warning --sort-by=.metadata.creationTimestamp --no-headers | tail -5 | wc -l)
    
    if [[ "$error_events" -gt 0 ]]; then
        warning "Recent warning events detected:"
        kubectl get events -n "$NAMESPACE" --field-selector type=Warning --sort-by=.metadata.creationTimestamp --no-headers | tail -5
    fi
}

perform_rollback() {
    info "Performing Helm rollback..."
    
    local rollback_cmd=(
        "helm" "rollback" "$RELEASE_NAME" "$REVISION"
        "--namespace" "$NAMESPACE"
        "--wait"
        "--timeout" "600s"
    )
    
    if [[ "$DRY_RUN" == "true" ]]; then
        rollback_cmd+=("--dry-run")
        info "Executing rollback (dry-run)..."
    else
        info "Executing rollback..."
    fi
    
    # Perform the rollback
    if "${rollback_cmd[@]}"; then
        success "Helm rollback completed successfully"
    else
        failure "Helm rollback failed"
        exit 1
    fi
}

wait_for_rollback() {
    if [[ "$DRY_RUN" == "true" ]]; then
        info "Skipping rollback wait (dry-run mode)"
        return
    fi
    
    info "Waiting for rollback to complete..."
    
    # Wait for deployment rollout
    if kubectl rollout status deployment/uveddi -n "$NAMESPACE" --timeout=600s; then
        success "Rollback rollout completed"
    else
        failure "Rollback rollout failed or timed out"
        
        # Show debugging information
        echo -e "\n${YELLOW}Deployment status:${NC}"
        kubectl describe deployment uveddi -n "$NAMESPACE"
        
        echo -e "\n${YELLOW}Pod status:${NC}"
        kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi
        
        exit 1
    fi
}

validate_rollback_success() {
    if [[ "$DRY_RUN" == "true" ]]; then
        info "Skipping rollback validation (dry-run mode)"
        return
    fi
    
    info "Validating rollback success..."
    
    # Wait for pods to be ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=uveddi -n "$NAMESPACE" --timeout=300s
    
    # Check pod health
    local ready_pods
    ready_pods=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi --no-headers | grep "Running" | wc -l)
    local total_pods
    total_pods=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi --no-headers | wc -l)
    
    info "Pod status after rollback: $ready_pods/$total_pods pods ready"
    
    if [[ "$ready_pods" -eq 0 ]]; then
        failure "No pods are ready after rollback"
        exit 1
    fi
    
    # Test health endpoint
    info "Testing application health..."
    kubectl port-forward service/uveddi 8080:80 -n "$NAMESPACE" &
    local port_forward_pid=$!
    
    cleanup_port_forward() {
        kill "$port_forward_pid" 2>/dev/null || true
    }
    trap cleanup_port_forward EXIT
    
    sleep 10
    
    local max_attempts=5
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -f -s "http://localhost:8080/health" > /dev/null; then
            success "Health check passed after rollback"
            break
        else
            warning "Health check failed (attempt $attempt/$max_attempts)"
            if [[ $attempt -eq $max_attempts ]]; then
                failure "Health check failed after rollback"
                cleanup_port_forward
                exit 1
            fi
            sleep 5
            ((attempt++))
        fi
    done
    
    cleanup_port_forward
    success "Rollback validation completed successfully"
}

display_rollback_result() {
    info "Rollback Summary:"
    echo "----------------------------------------"
    echo "Environment: $ENVIRONMENT"
    echo "Rollback Time: $(date)"
    echo "Target Revision: $REVISION"
    echo "Logs: $LOG_FILE"
    
    if [[ "$DRY_RUN" != "true" ]]; then
        echo ""
        echo "Current Status:"
        kubectl get deployment,service,pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi 2>/dev/null || true
        
        # Get new image info
        local current_image
        current_image=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null || echo "unknown")
        echo ""
        echo "Current Image: $current_image"
    fi
    
    echo "----------------------------------------"
}

show_usage() {
    cat << EOF
Usage: $0 [ENVIRONMENT] [REVISION] [DRY_RUN]

ENVIRONMENT: staging or production (default: staging)
REVISION: Target revision number, or 0 for previous (default: 0)
DRY_RUN: true or false (default: false)

Environment Variables:
  CONFIRM=false      Skip confirmation prompts
  DEBUG=true         Enable debug logging

Examples:
  $0                           # Rollback to previous revision in staging
  $0 production                # Rollback to previous revision in production
  $0 staging 5                 # Rollback to specific revision 5 in staging
  $0 production 0 true         # Dry-run rollback in production
  CONFIRM=false $0 staging     # Rollback without confirmation

EOF
}

# Main rollback function
main() {
    # Show usage if help requested
    if [[ "${1:-}" =~ ^(-h|--help)$ ]]; then
        show_usage
        exit 0
    fi
    
    info "Starting Uveddi rollback..."
    info "Target environment: $ENVIRONMENT"
    info "Target revision: $REVISION"
    info "Dry run: $DRY_RUN"
    
    # Run rollback steps
    validate_environment
    validate_prerequisites
    get_current_deployment_info
    determine_target_revision
    show_rollback_plan
    confirm_rollback
    create_rollback_backup
    check_current_health
    perform_rollback
    wait_for_rollback
    validate_rollback_success
    display_rollback_result
    
    success "Rollback completed successfully!"
    
    if [[ "$ENVIRONMENT" == "production" ]]; then
        info "🔄 Production rollback is complete!"
        info "Monitor the application to ensure it's functioning properly"
    fi
}

# Execute main function
main "$@"