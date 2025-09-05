#!/bin/bash
# Comprehensive deployment script for Uveddi
# Supports staging and production deployments with comprehensive validation

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="/tmp/uveddi-deploy-$(date +%Y%m%d-%H%M%S).log"

# Default values
ENVIRONMENT="${1:-staging}"
IMAGE_TAG="${2:-latest}"
DRY_RUN="${3:-false}"
SKIP_TESTS="${SKIP_TESTS:-false}"
FORCE_DEPLOY="${FORCE_DEPLOY:-false}"

# Configuration
NAMESPACE="uveddi"
HELM_CHART_PATH="${PROJECT_ROOT}/helm/uveddi"
K8S_MANIFESTS_PATH="${PROJECT_ROOT}/k8s"

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
        failure "Deployment failed with exit code $exit_code"
        echo "Logs available at: $LOG_FILE"
        
        # Show recent logs for debugging
        if command -v kubectl &> /dev/null; then
            echo -e "\n${YELLOW}Recent pod events:${NC}"
            kubectl get events -n "$NAMESPACE" --sort-by=.metadata.creationTimestamp --tail=10 2>/dev/null || true
        fi
    fi
}

trap cleanup EXIT

# Validation functions
validate_environment() {
    case "$ENVIRONMENT" in
        staging|production)
            info "Deploying to $ENVIRONMENT environment"
            ;;
        *)
            failure "Invalid environment: $ENVIRONMENT. Must be 'staging' or 'production'"
            exit 1
            ;;
    esac
}

validate_prerequisites() {
    info "Validating prerequisites..."
    
    local required_commands=("kubectl" "helm" "docker" "curl" "jq")
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
    
    # Check Helm chart exists
    if [[ ! -d "$HELM_CHART_PATH" ]]; then
        failure "Helm chart not found at $HELM_CHART_PATH"
        exit 1
    fi
    
    success "Prerequisites validated"
}

validate_image() {
    info "Validating container image..."
    
    local registry_url="ghcr.io/botzrdev/uveddi:$IMAGE_TAG"
    
    # Check if image exists (this will fail if image doesn't exist or we don't have access)
    if ! docker manifest inspect "$registry_url" &> /dev/null; then
        if [[ "$FORCE_DEPLOY" != "true" ]]; then
            failure "Container image not found or not accessible: $registry_url"
            warning "Use FORCE_DEPLOY=true to skip image validation"
            exit 1
        else
            warning "Skipping image validation due to FORCE_DEPLOY=true"
        fi
    else
        success "Container image validated: $registry_url"
    fi
}

check_cluster_health() {
    info "Checking cluster health..."
    
    # Check nodes
    local unhealthy_nodes
    unhealthy_nodes=$(kubectl get nodes --no-headers | grep -v "Ready" | wc -l)
    
    if [[ $unhealthy_nodes -gt 0 ]]; then
        warning "$unhealthy_nodes nodes are not in Ready state"
        kubectl get nodes --no-headers | grep -v "Ready" || true
        
        if [[ "$FORCE_DEPLOY" != "true" ]]; then
            failure "Cluster has unhealthy nodes. Use FORCE_DEPLOY=true to ignore"
            exit 1
        fi
    fi
    
    # Check system pods
    local failing_pods
    failing_pods=$(kubectl get pods -n kube-system --no-headers | grep -v "Running\|Completed" | wc -l)
    
    if [[ $failing_pods -gt 0 ]]; then
        warning "$failing_pods system pods are not running properly"
        kubectl get pods -n kube-system --no-headers | grep -v "Running\|Completed" || true
    fi
    
    success "Cluster health check completed"
}

prepare_namespace() {
    info "Preparing namespace: $NAMESPACE"
    
    # Create namespace if it doesn't exist
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        kubectl create namespace "$NAMESPACE"
        info "Created namespace: $NAMESPACE"
    fi
    
    # Apply namespace configuration
    if [[ -f "$K8S_MANIFESTS_PATH/base/namespace.yaml" ]]; then
        kubectl apply -f "$K8S_MANIFESTS_PATH/base/namespace.yaml"
        info "Applied namespace configuration"
    fi
    
    success "Namespace prepared"
}

run_pre_deployment_checks() {
    info "Running pre-deployment checks..."
    
    # Check if there's an existing deployment
    if kubectl get deployment uveddi -n "$NAMESPACE" &> /dev/null; then
        info "Existing deployment found"
        
        # Check deployment status
        local ready_replicas
        ready_replicas=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
        local desired_replicas
        desired_replicas=$(kubectl get deployment uveddi -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
        
        info "Current deployment: $ready_replicas/$desired_replicas pods ready"
        
        if [[ "$ready_replicas" != "$desired_replicas" ]] && [[ "$FORCE_DEPLOY" != "true" ]]; then
            warning "Existing deployment is not fully healthy"
            warning "Use FORCE_DEPLOY=true to deploy anyway"
        fi
    else
        info "No existing deployment found - this will be a new deployment"
    fi
    
    success "Pre-deployment checks completed"
}

backup_current_deployment() {
    if kubectl get deployment uveddi -n "$NAMESPACE" &> /dev/null; then
        info "Backing up current deployment configuration..."
        
        local backup_dir="/tmp/uveddi-backup-$(date +%Y%m%d-%H%M%S)"
        mkdir -p "$backup_dir"
        
        # Backup deployment
        kubectl get deployment uveddi -n "$NAMESPACE" -o yaml > "$backup_dir/deployment.yaml"
        
        # Backup configmap
        kubectl get configmap uveddi-config -n "$NAMESPACE" -o yaml > "$backup_dir/configmap.yaml" 2>/dev/null || true
        
        # Backup secrets (without sensitive data)
        kubectl get secret uveddi-secrets -n "$NAMESPACE" -o yaml | sed 's/data:/data: # [REDACTED]/' > "$backup_dir/secrets.yaml" 2>/dev/null || true
        
        info "Backup created at: $backup_dir"
        echo "$backup_dir" > "/tmp/uveddi-last-backup.txt"
        
        success "Deployment backup completed"
    fi
}

run_database_migration() {
    info "Running database migration..."
    
    # Apply migration job
    local migration_job="$K8S_MANIFESTS_PATH/migrations/migration-job.yaml"
    if [[ -f "$migration_job" ]]; then
        # Delete existing migration job if exists
        kubectl delete job uveddi-migration -n "$NAMESPACE" --ignore-not-found=true
        
        # Update image in migration job and apply
        sed "s|ghcr.io/botzrdev/uveddi:latest|ghcr.io/botzrdev/uveddi:$IMAGE_TAG|g" "$migration_job" | \
            kubectl apply -n "$NAMESPACE" -f -
        
        # Wait for migration to complete
        info "Waiting for migration to complete..."
        if kubectl wait --for=condition=complete job/uveddi-migration -n "$NAMESPACE" --timeout=600s; then
            success "Database migration completed successfully"
            
            # Show migration logs
            info "Migration logs:"
            kubectl logs job/uveddi-migration -n "$NAMESPACE" --tail=20
        else
            failure "Database migration failed or timed out"
            kubectl logs job/uveddi-migration -n "$NAMESPACE" --tail=50
            exit 1
        fi
    else
        warning "Migration job not found, skipping database migration"
    fi
}

deploy_application() {
    info "Deploying application..."
    
    # Prepare Helm values file
    local values_file="$HELM_CHART_PATH/values-$ENVIRONMENT.yaml"
    if [[ ! -f "$values_file" ]]; then
        failure "Values file not found: $values_file"
        exit 1
    fi
    
    # Build Helm command
    local helm_cmd=(
        "helm" "upgrade" "--install" "uveddi" "$HELM_CHART_PATH"
        "--namespace" "$NAMESPACE"
        "--create-namespace"
        "--values" "$values_file"
        "--set" "image.tag=$IMAGE_TAG"
        "--set" "environment=$ENVIRONMENT"
        "--timeout" "600s"
        "--wait"
    )
    
    if [[ "$DRY_RUN" == "true" ]]; then
        helm_cmd+=("--dry-run" "--debug")
        info "Running Helm deployment (dry-run)..."
    else
        info "Running Helm deployment..."
    fi
    
    # Add dependency update
    info "Updating Helm dependencies..."
    helm dependency update "$HELM_CHART_PATH"
    
    # Execute deployment
    if "${helm_cmd[@]}"; then
        success "Application deployed successfully"
    else
        failure "Application deployment failed"
        exit 1
    fi
}

wait_for_deployment() {
    if [[ "$DRY_RUN" == "true" ]]; then
        info "Skipping deployment wait (dry-run mode)"
        return
    fi
    
    info "Waiting for deployment to be ready..."
    
    # Wait for deployment rollout
    if kubectl rollout status deployment/uveddi -n "$NAMESPACE" --timeout=600s; then
        success "Deployment rollout completed"
    else
        failure "Deployment rollout failed or timed out"
        
        # Show debugging information
        echo -e "\n${YELLOW}Deployment status:${NC}"
        kubectl describe deployment uveddi -n "$NAMESPACE"
        
        echo -e "\n${YELLOW}Pod status:${NC}"
        kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi
        
        exit 1
    fi
    
    # Check pod health
    info "Checking pod health..."
    local ready_pods
    ready_pods=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi --no-headers | grep "Running" | wc -l)
    local total_pods
    total_pods=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi --no-headers | wc -l)
    
    info "Pod status: $ready_pods/$total_pods pods ready"
    
    if [[ "$ready_pods" -eq 0 ]]; then
        failure "No pods are ready"
        exit 1
    fi
}

run_post_deployment_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        warning "Skipping post-deployment tests"
        return
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        info "Skipping tests (dry-run mode)"
        return
    fi
    
    info "Running post-deployment tests..."
    
    # Wait for pods to be fully ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=uveddi -n "$NAMESPACE" --timeout=300s
    
    # Port forward for testing
    info "Setting up port forwarding for testing..."
    kubectl port-forward service/uveddi 8080:80 -n "$NAMESPACE" &
    local port_forward_pid=$!
    
    # Cleanup function for port forward
    cleanup_port_forward() {
        if [[ -n "${port_forward_pid:-}" ]]; then
            kill "$port_forward_pid" 2>/dev/null || true
        fi
    }
    trap cleanup_port_forward EXIT
    
    # Wait for port forward to be ready
    sleep 10
    
    # Test health endpoint
    info "Testing health endpoint..."
    local max_attempts=10
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -f -s "http://localhost:8080/health" > /dev/null; then
            success "Health check passed"
            break
        else
            warning "Health check failed (attempt $attempt/$max_attempts)"
            if [[ $attempt -eq $max_attempts ]]; then
                failure "Health check failed after $max_attempts attempts"
                cleanup_port_forward
                exit 1
            fi
            sleep 10
            ((attempt++))
        fi
    done
    
    # Test readiness endpoint
    info "Testing readiness endpoint..."
    if curl -f -s "http://localhost:8080/ready" > /dev/null; then
        success "Readiness check passed"
    else
        warning "Readiness check failed"
    fi
    
    # Test metrics endpoint
    info "Testing metrics endpoint..."
    if curl -f -s "http://localhost:9090/metrics" > /dev/null 2>&1; then
        success "Metrics endpoint accessible"
    else
        warning "Metrics endpoint not accessible"
    fi
    
    cleanup_port_forward
    success "Post-deployment tests completed"
}

display_deployment_info() {
    info "Deployment Information:"
    echo "----------------------------------------"
    echo "Environment: $ENVIRONMENT"
    echo "Image Tag: $IMAGE_TAG"
    echo "Namespace: $NAMESPACE"
    echo "Deployment Time: $(date)"
    echo "Logs: $LOG_FILE"
    
    if [[ "$DRY_RUN" != "true" ]]; then
        echo ""
        echo "Kubernetes Resources:"
        kubectl get deployment,service,ingress,hpa,pdb -n "$NAMESPACE" 2>/dev/null || true
        
        echo ""
        echo "Pod Status:"
        kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi -o wide 2>/dev/null || true
    fi
    
    echo "----------------------------------------"
}

show_usage() {
    cat << EOF
Usage: $0 [ENVIRONMENT] [IMAGE_TAG] [DRY_RUN]

ENVIRONMENT: staging or production (default: staging)
IMAGE_TAG: Container image tag (default: latest)
DRY_RUN: true or false (default: false)

Environment Variables:
  SKIP_TESTS=true    Skip post-deployment tests
  FORCE_DEPLOY=true  Skip validation checks
  DEBUG=true         Enable debug logging

Examples:
  $0                                    # Deploy to staging with latest tag
  $0 production v1.2.3                 # Deploy to production with specific tag
  $0 staging latest true                # Dry-run deployment to staging
  SKIP_TESTS=true $0 production v1.2.3  # Deploy without running tests

EOF
}

# Main deployment function
main() {
    # Show usage if help requested
    if [[ "${1:-}" =~ ^(-h|--help)$ ]]; then
        show_usage
        exit 0
    fi
    
    info "Starting Uveddi deployment..."
    info "Target environment: $ENVIRONMENT"
    info "Image tag: $IMAGE_TAG"
    info "Dry run: $DRY_RUN"
    
    # Run deployment steps
    validate_environment
    validate_prerequisites
    validate_image
    check_cluster_health
    prepare_namespace
    run_pre_deployment_checks
    backup_current_deployment
    run_database_migration
    deploy_application
    wait_for_deployment
    run_post_deployment_tests
    display_deployment_info
    
    success "Deployment completed successfully!"
    
    if [[ "$ENVIRONMENT" == "production" ]]; then
        info "🚀 Production deployment is now live!"
        info "Monitor the application and be prepared to rollback if issues arise"
    fi
}

# Execute main function
main "$@"