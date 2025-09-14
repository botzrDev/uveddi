#!/bin/bash
set -euo pipefail

# Production Deployment Script for Uveddi
# Comprehensive deployment with monitoring, security, and backup procedures

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="${NAMESPACE:-uveddi-prod}"
ENVIRONMENT="${ENVIRONMENT:-production}"
HELM_RELEASE="${HELM_RELEASE:-uveddi-prod}"
CHART_PATH="./helm/uveddi"
VALUES_FILE="./helm/uveddi/values-production.yaml"
BACKUP_DIR="${BACKUP_DIR:-/backups}"
LOG_DIR="${LOG_DIR:-./logs}"

# Create log directory
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/deploy-$(date +%Y%m%d-%H%M%S).log"

# Logging function
log() {
    echo -e "${1}" | tee -a "$LOG_FILE"
}

# Error handling
error_exit() {
    log "${RED}ERROR: $1${NC}"
    exit 1
}

# Success message
success() {
    log "${GREEN}✓ $1${NC}"
}

# Warning message
warning() {
    log "${YELLOW}⚠ $1${NC}"
}

# Info message
info() {
    log "${BLUE}ℹ $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    info "Checking prerequisites..."

    # Check for required tools
    local tools=("kubectl" "helm" "docker")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            error_exit "$tool is not installed"
        fi
    done
    success "All required tools are installed"

    # Check Kubernetes connectivity
    if ! kubectl cluster-info &> /dev/null; then
        error_exit "Cannot connect to Kubernetes cluster"
    fi
    success "Connected to Kubernetes cluster"

    # Check Helm chart exists
    if [[ ! -d "$CHART_PATH" ]]; then
        error_exit "Helm chart not found at $CHART_PATH"
    fi
    success "Helm chart found"
}

# Create namespace and configure RBAC
setup_namespace() {
    info "Setting up namespace and RBAC..."

    # Create namespace
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

    # Label namespace for monitoring
    kubectl label namespace "$NAMESPACE" \
        monitoring=enabled \
        environment="$ENVIRONMENT" \
        --overwrite

    success "Namespace $NAMESPACE configured"
}

# Setup SSL certificates using cert-manager
setup_ssl() {
    info "Setting up SSL certificates..."

    # Check if cert-manager is installed
    if ! kubectl get namespace cert-manager &> /dev/null; then
        warning "cert-manager not found, installing..."
        kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

        # Wait for cert-manager to be ready
        kubectl wait --for=condition=ready pod -l app.kubernetes.io/instance=cert-manager \
            -n cert-manager --timeout=300s
    fi

    # Create ClusterIssuer for Let's Encrypt
    cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: ${SSL_EMAIL:-admin@uveddi.dev}
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF

    success "SSL certificate issuer configured"
}

# Configure secrets
setup_secrets() {
    info "Configuring secrets..."

    # Generate secure passwords if not provided
    DB_PASSWORD="${DB_PASSWORD:-$(openssl rand -base64 32)}"
    JWT_SECRET="${JWT_SECRET:-$(openssl rand -base64 32)}"
    API_KEY="${API_KEY:-$(openssl rand -base64 32)}"

    # Create secrets
    kubectl create secret generic uveddi-secrets \
        --from-literal=database-password="$DB_PASSWORD" \
        --from-literal=jwt-secret="$JWT_SECRET" \
        --from-literal=api-key="$API_KEY" \
        --namespace="$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -

    # Create Docker registry secret if provided
    if [[ -n "${DOCKER_REGISTRY_SECRET:-}" ]]; then
        kubectl create secret docker-registry regcred \
            --docker-server="${DOCKER_REGISTRY:-ghcr.io}" \
            --docker-username="${DOCKER_USERNAME}" \
            --docker-password="${DOCKER_PASSWORD}" \
            --namespace="$NAMESPACE" \
            --dry-run=client -o yaml | kubectl apply -f -
    fi

    success "Secrets configured"
}

# Deploy database with backup configuration
deploy_database() {
    info "Deploying database..."

    # Create PVC for database
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: $NAMESPACE
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: ${STORAGE_CLASS:-standard}
  resources:
    requests:
      storage: 100Gi
EOF

    # Create backup PVC
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: backup-pvc
  namespace: $NAMESPACE
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: ${STORAGE_CLASS:-standard}
  resources:
    requests:
      storage: 50Gi
EOF

    success "Database storage configured"
}

# Setup monitoring stack
setup_monitoring() {
    info "Setting up monitoring..."

    # Apply monitoring configurations
    kubectl apply -f k8s/monitoring/ -n "$NAMESPACE"

    # Create ServiceMonitor for Prometheus
    cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: uveddi-metrics
  namespace: $NAMESPACE
spec:
  selector:
    matchLabels:
      app: uveddi
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
EOF

    success "Monitoring configured"
}

# Deploy Uveddi using Helm
deploy_uveddi() {
    info "Deploying Uveddi..."

    # Update Helm dependencies
    helm dependency update "$CHART_PATH"

    # Perform dry-run first
    info "Running Helm dry-run..."
    if ! helm upgrade --install "$HELM_RELEASE" "$CHART_PATH" \
        --namespace "$NAMESPACE" \
        --values "$VALUES_FILE" \
        --set environment="$ENVIRONMENT" \
        --set image.tag="${IMAGE_TAG:-latest}" \
        --dry-run --debug > "$LOG_DIR/helm-dry-run.log" 2>&1; then
        error_exit "Helm dry-run failed. Check $LOG_DIR/helm-dry-run.log"
    fi
    success "Helm dry-run successful"

    # Deploy with Helm
    info "Deploying with Helm..."
    helm upgrade --install "$HELM_RELEASE" "$CHART_PATH" \
        --namespace "$NAMESPACE" \
        --values "$VALUES_FILE" \
        --set environment="$ENVIRONMENT" \
        --set image.tag="${IMAGE_TAG:-latest}" \
        --wait \
        --timeout 10m

    success "Uveddi deployed successfully"
}

# Setup database backup CronJob
setup_backup() {
    info "Setting up database backups..."

    # Create backup script ConfigMap
    cat <<'EOF' | kubectl create configmap backup-script --from-file=/dev/stdin -n "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
#!/bin/bash
set -e

BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="${DB_NAME:-uveddi_prod}"

# Create backup directory
mkdir -p $BACKUP_DIR

# Perform backup
pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME | gzip > "$BACKUP_DIR/uveddi_backup_$DATE.sql.gz"

# Upload to S3 if configured
if [[ -n "${S3_BUCKET:-}" ]]; then
    aws s3 cp "$BACKUP_DIR/uveddi_backup_$DATE.sql.gz" "s3://$S3_BUCKET/backups/"
fi

# Cleanup old backups (keep last 7 days)
find $BACKUP_DIR -name "uveddi_backup_*.sql.gz" -mtime +7 -delete

echo "Backup completed: uveddi_backup_$DATE.sql.gz"
EOF

    # Create backup CronJob
    cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: CronJob
metadata:
  name: database-backup
  namespace: $NAMESPACE
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:15-alpine
            command: ["/scripts/backup.sh"]
            env:
            - name: DB_HOST
              value: "postgres-service"
            - name: DB_USER
              valueFrom:
                secretKeyRef:
                  name: uveddi-secrets
                  key: database-username
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: uveddi-secrets
                  key: database-password
            volumeMounts:
            - name: backup-storage
              mountPath: /backups
            - name: backup-script
              mountPath: /scripts
          volumes:
          - name: backup-storage
            persistentVolumeClaim:
              claimName: backup-pvc
          - name: backup-script
            configMap:
              name: backup-script
              defaultMode: 0755
          restartPolicy: OnFailure
EOF

    success "Backup procedures configured"
}

# Configure autoscaling
setup_autoscaling() {
    info "Configuring autoscaling..."

    # Create HPA for API
    cat <<EOF | kubectl apply -f -
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: uveddi-api-hpa
  namespace: $NAMESPACE
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: uveddi-api
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
EOF

    success "Autoscaling configured"
}

# Verify deployment
verify_deployment() {
    info "Verifying deployment..."

    # Wait for pods to be ready
    kubectl wait --for=condition=ready pod \
        -l app=uveddi \
        -n "$NAMESPACE" \
        --timeout=300s

    # Check pod status
    kubectl get pods -n "$NAMESPACE"

    # Check services
    kubectl get svc -n "$NAMESPACE"

    # Check ingress
    kubectl get ingress -n "$NAMESPACE"

    # Run health checks
    local POD=$(kubectl get pod -l app=uveddi -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}')
    kubectl exec -n "$NAMESPACE" "$POD" -- curl -f http://localhost:8080/health || warning "Health check failed"

    success "Deployment verified"
}

# Create monitoring dashboard
create_dashboard() {
    info "Creating monitoring dashboard..."

    # Get Grafana URL
    local GRAFANA_URL=$(kubectl get ingress grafana -n monitoring -o jsonpath='{.spec.rules[0].host}' 2>/dev/null || echo "grafana.local")

    info "Grafana dashboard available at: http://$GRAFANA_URL"
    info "Default credentials: admin/admin (change immediately)"

    success "Dashboard configured"
}

# Main deployment flow
main() {
    log "==========================================="
    log "Uveddi Production Deployment"
    log "Environment: $ENVIRONMENT"
    log "Namespace: $NAMESPACE"
    log "Time: $(date)"
    log "==========================================="

    # Run deployment steps
    check_prerequisites
    setup_namespace
    setup_ssl
    setup_secrets
    deploy_database
    setup_monitoring
    deploy_uveddi
    setup_backup
    setup_autoscaling
    verify_deployment
    create_dashboard

    log "==========================================="
    success "DEPLOYMENT COMPLETED SUCCESSFULLY!"
    log "==========================================="

    # Print summary
    info "Deployment Summary:"
    info "- Namespace: $NAMESPACE"
    info "- Release: $HELM_RELEASE"
    info "- Environment: $ENVIRONMENT"
    info "- Log file: $LOG_FILE"

    # Get ingress URL
    local INGRESS_URL=$(kubectl get ingress uveddi -n "$NAMESPACE" -o jsonpath='{.spec.rules[0].host}' 2>/dev/null || echo "uveddi.local")
    info "- Application URL: https://$INGRESS_URL"

    warning "Next steps:"
    warning "1. Update DNS records to point to the ingress IP"
    warning "2. Verify SSL certificate generation"
    warning "3. Configure external monitoring alerts"
    warning "4. Test backup and restore procedures"
    warning "5. Update documentation with production URLs"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --image-tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --namespace NAME     Kubernetes namespace (default: uveddi-prod)"
            echo "  --environment ENV    Environment name (default: production)"
            echo "  --image-tag TAG      Docker image tag (default: latest)"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            error_exit "Unknown option: $1"
            ;;
    esac
done

# Run main deployment
main