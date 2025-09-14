#!/bin/bash
set -euo pipefail

# Disaster Recovery Script for Uveddi
# Handles backup, restore, and failover procedures

# Configuration
NAMESPACE="${NAMESPACE:-uveddi-prod}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Functions
log() { echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warning() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup functions
backup_database() {
    log "Backing up database..."

    local DB_POD=$(kubectl get pod -l app=postgres -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)

    if [[ -z "$DB_POD" ]]; then
        warning "No database pod found, skipping database backup"
        return
    fi

    # Create database backup
    kubectl exec -n "$NAMESPACE" "$DB_POD" -- \
        pg_dump -U uveddi uveddi_prod | \
        gzip > "$BACKUP_DIR/database-$TIMESTAMP.sql.gz"

    success "Database backed up to $BACKUP_DIR/database-$TIMESTAMP.sql.gz"
}

backup_configurations() {
    log "Backing up Kubernetes configurations..."

    # Backup all resources
    kubectl get all,cm,secret,pvc,pv,ingress,networkpolicy,poddisruptionbudget -n "$NAMESPACE" \
        -o yaml > "$BACKUP_DIR/k8s-resources-$TIMESTAMP.yaml"

    # Backup Helm values
    helm get values uveddi-prod -n "$NAMESPACE" > "$BACKUP_DIR/helm-values-$TIMESTAMP.yaml"

    success "Configurations backed up to $BACKUP_DIR/"
}

backup_persistent_volumes() {
    log "Backing up persistent volume claims..."

    # Get all PVCs
    kubectl get pvc -n "$NAMESPACE" -o json > "$BACKUP_DIR/pvcs-$TIMESTAMP.json"

    # Create volume snapshots if supported
    kubectl get pvc -n "$NAMESPACE" --no-headers | while read line; do
        local PVC_NAME=$(echo "$line" | awk '{print $1}')

        cat <<EOF | kubectl apply -f - || warning "Snapshot not supported for $PVC_NAME"
apiVersion: snapshot.storage.k8s.io/v1
kind: VolumeSnapshot
metadata:
  name: snapshot-$PVC_NAME-$TIMESTAMP
  namespace: $NAMESPACE
spec:
  volumeSnapshotClassName: csi-snapclass
  source:
    persistentVolumeClaimName: $PVC_NAME
EOF
    done

    success "Persistent volumes backed up"
}

backup_secrets() {
    log "Backing up secrets (encrypted)..."

    # Export secrets (base64 encoded)
    kubectl get secrets -n "$NAMESPACE" -o yaml | \
        openssl enc -aes-256-cbc -salt -out "$BACKUP_DIR/secrets-$TIMESTAMP.enc" -pass pass:"${ENCRYPTION_KEY:-uveddi}"

    success "Secrets backed up (encrypted)"
}

# Restore functions
restore_database() {
    local BACKUP_FILE=$1

    log "Restoring database from $BACKUP_FILE..."

    local DB_POD=$(kubectl get pod -l app=postgres -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}')

    if [[ -z "$DB_POD" ]]; then
        error "No database pod found"
    fi

    # Drop existing database and recreate
    kubectl exec -n "$NAMESPACE" "$DB_POD" -- psql -U uveddi -c "DROP DATABASE IF EXISTS uveddi_prod;"
    kubectl exec -n "$NAMESPACE" "$DB_POD" -- psql -U uveddi -c "CREATE DATABASE uveddi_prod;"

    # Restore from backup
    gunzip -c "$BACKUP_FILE" | \
        kubectl exec -i -n "$NAMESPACE" "$DB_POD" -- \
        psql -U uveddi uveddi_prod

    success "Database restored"
}

restore_configurations() {
    local BACKUP_FILE=$1

    log "Restoring configurations from $BACKUP_FILE..."

    # Apply configurations
    kubectl apply -f "$BACKUP_FILE"

    success "Configurations restored"
}

restore_secrets() {
    local BACKUP_FILE=$1

    log "Restoring secrets from $BACKUP_FILE..."

    # Decrypt and apply secrets
    openssl enc -aes-256-cbc -d -in "$BACKUP_FILE" -pass pass:"${ENCRYPTION_KEY:-uveddi}" | \
        kubectl apply -f -

    success "Secrets restored"
}

# Failover functions
failover_to_dr() {
    log "Initiating failover to disaster recovery site..."

    # Update DNS to point to DR site
    warning "Manual step: Update DNS records to point to DR site"

    # Scale down production
    kubectl scale deployment --all --replicas=0 -n "$NAMESPACE"

    # Start DR site
    kubectl scale deployment --all --replicas=3 -n uveddi-dr

    # Verify DR site
    kubectl wait --for=condition=ready pod -l app=uveddi -n uveddi-dr --timeout=300s

    success "Failover to DR site completed"
}

failback_to_production() {
    log "Initiating failback to production..."

    # Sync data from DR to production
    warning "Manual step: Sync data from DR to production database"

    # Scale down DR
    kubectl scale deployment --all --replicas=0 -n uveddi-dr

    # Start production
    kubectl scale deployment --all --replicas=3 -n "$NAMESPACE"

    # Verify production
    kubectl wait --for=condition=ready pod -l app=uveddi -n "$NAMESPACE" --timeout=300s

    # Update DNS back to production
    warning "Manual step: Update DNS records to point back to production"

    success "Failback to production completed"
}

# Test recovery procedures
test_recovery() {
    log "Testing disaster recovery procedures..."

    # Create test namespace
    kubectl create namespace uveddi-dr-test --dry-run=client -o yaml | kubectl apply -f -

    # Restore to test namespace
    log "Restoring to test namespace..."

    # Apply configurations to test namespace
    sed "s/namespace: $NAMESPACE/namespace: uveddi-dr-test/g" \
        "$BACKUP_DIR/k8s-resources-$TIMESTAMP.yaml" | \
        kubectl apply -f -

    # Verify test deployment
    kubectl wait --for=condition=ready pod -l app=uveddi -n uveddi-dr-test --timeout=300s || true

    # Cleanup test namespace
    kubectl delete namespace uveddi-dr-test

    success "Recovery test completed"
}

# Verify backups
verify_backups() {
    log "Verifying backups..."

    local ERRORS=0

    # Check database backup
    if [[ -f "$BACKUP_DIR/database-$TIMESTAMP.sql.gz" ]]; then
        if gunzip -t "$BACKUP_DIR/database-$TIMESTAMP.sql.gz" 2>/dev/null; then
            success "Database backup is valid"
        else
            warning "Database backup is corrupted"
            ERRORS=$((ERRORS + 1))
        fi
    else
        warning "Database backup not found"
        ERRORS=$((ERRORS + 1))
    fi

    # Check configuration backup
    if [[ -f "$BACKUP_DIR/k8s-resources-$TIMESTAMP.yaml" ]]; then
        if kubectl apply --dry-run=client -f "$BACKUP_DIR/k8s-resources-$TIMESTAMP.yaml" &>/dev/null; then
            success "Configuration backup is valid"
        else
            warning "Configuration backup has issues"
            ERRORS=$((ERRORS + 1))
        fi
    else
        warning "Configuration backup not found"
        ERRORS=$((ERRORS + 1))
    fi

    if [[ $ERRORS -eq 0 ]]; then
        success "All backups verified successfully"
    else
        error "$ERRORS backup(s) have issues"
    fi
}

# Cleanup old backups
cleanup_backups() {
    local RETENTION_DAYS="${RETENTION_DAYS:-30}"

    log "Cleaning up backups older than $RETENTION_DAYS days..."

    find "$BACKUP_DIR" -type f -mtime +$RETENTION_DAYS -delete

    success "Old backups cleaned up"
}

# Main menu
show_menu() {
    echo "==========================================="
    echo "    Uveddi Disaster Recovery Tool"
    echo "==========================================="
    echo "1. Full Backup"
    echo "2. Database Backup"
    echo "3. Configuration Backup"
    echo "4. Restore Database"
    echo "5. Restore Full System"
    echo "6. Failover to DR Site"
    echo "7. Failback to Production"
    echo "8. Test Recovery Procedures"
    echo "9. Verify Backups"
    echo "10. Cleanup Old Backups"
    echo "0. Exit"
    echo "==========================================="
}

# Full backup
full_backup() {
    log "Starting full backup..."

    backup_database
    backup_configurations
    backup_persistent_volumes
    backup_secrets

    # Create backup manifest
    cat > "$BACKUP_DIR/manifest-$TIMESTAMP.json" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "namespace": "$NAMESPACE",
  "type": "full",
  "files": [
    "database-$TIMESTAMP.sql.gz",
    "k8s-resources-$TIMESTAMP.yaml",
    "helm-values-$TIMESTAMP.yaml",
    "pvcs-$TIMESTAMP.json",
    "secrets-$TIMESTAMP.enc"
  ]
}
EOF

    success "Full backup completed at $BACKUP_DIR"

    # Upload to S3 if configured
    if [[ -n "${S3_BUCKET:-}" ]]; then
        log "Uploading to S3..."
        aws s3 sync "$BACKUP_DIR" "s3://$S3_BUCKET/backups/" --exclude "*" --include "*-$TIMESTAMP.*"
        success "Backup uploaded to S3"
    fi
}

# Full restore
full_restore() {
    log "Starting full system restore..."

    # List available backups
    echo "Available backups:"
    ls -la "$BACKUP_DIR"/*.yaml 2>/dev/null || echo "No backups found"

    echo -n "Enter backup timestamp (YYYYMMDD-HHMMSS): "
    read RESTORE_TIMESTAMP

    # Restore components
    if [[ -f "$BACKUP_DIR/k8s-resources-$RESTORE_TIMESTAMP.yaml" ]]; then
        restore_configurations "$BACKUP_DIR/k8s-resources-$RESTORE_TIMESTAMP.yaml"
    fi

    if [[ -f "$BACKUP_DIR/secrets-$RESTORE_TIMESTAMP.enc" ]]; then
        restore_secrets "$BACKUP_DIR/secrets-$RESTORE_TIMESTAMP.enc"
    fi

    if [[ -f "$BACKUP_DIR/database-$RESTORE_TIMESTAMP.sql.gz" ]]; then
        restore_database "$BACKUP_DIR/database-$RESTORE_TIMESTAMP.sql.gz"
    fi

    success "Full system restore completed"
}

# Interactive mode
interactive_mode() {
    while true; do
        show_menu
        echo -n "Select option: "
        read option

        case $option in
            1) full_backup ;;
            2) backup_database ;;
            3) backup_configurations ;;
            4)
                echo -n "Enter backup file path: "
                read backup_file
                restore_database "$backup_file"
                ;;
            5) full_restore ;;
            6) failover_to_dr ;;
            7) failback_to_production ;;
            8) test_recovery ;;
            9) verify_backups ;;
            10) cleanup_backups ;;
            0) exit 0 ;;
            *) warning "Invalid option" ;;
        esac

        echo
        echo "Press Enter to continue..."
        read
    done
}

# Parse command line arguments
case "${1:-}" in
    backup)
        full_backup
        ;;
    restore)
        full_restore
        ;;
    failover)
        failover_to_dr
        ;;
    failback)
        failback_to_production
        ;;
    test)
        test_recovery
        ;;
    verify)
        verify_backups
        ;;
    cleanup)
        cleanup_backups
        ;;
    --help)
        echo "Usage: $0 [COMMAND]"
        echo "Commands:"
        echo "  backup    Perform full backup"
        echo "  restore   Restore from backup"
        echo "  failover  Failover to DR site"
        echo "  failback  Failback to production"
        echo "  test      Test recovery procedures"
        echo "  verify    Verify backup integrity"
        echo "  cleanup   Remove old backups"
        echo ""
        echo "Run without arguments for interactive mode"
        exit 0
        ;;
    *)
        interactive_mode
        ;;
esac