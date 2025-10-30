# Uveddi Production Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying Uveddi to a production Kubernetes environment with full monitoring, security, and backup procedures.

## Prerequisites

### Required Tools
- Kubernetes cluster (v1.28+)
- kubectl CLI configured
- Helm 3.x
- Docker
- SSL certificate domain

### Required Access
- Kubernetes cluster admin access
- Container registry credentials
- Domain DNS management
- SSL certificate authority (Let's Encrypt)

## Quick Start

Deploy Uveddi to production with a single command:

```bash
./scripts/deploy-production.sh --namespace uveddi-prod --environment production
```

## Deployment Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Internet Traffic                      │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │  Ingress/SSL  │
                    │  (NGINX/ALB)  │
                    └───────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Frontend   │  │     API      │  │  Rendering   │
│  (3 replicas)│  │ (3 replicas) │  │  Service     │
└──────────────┘  └──────────────┘  └──────────────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  PostgreSQL  │  │    Redis     │  │ Prometheus/  │
│   Database   │  │    Cache     │  │   Grafana    │
└──────────────┘  └──────────────┘  └──────────────┘
```

## Step-by-Step Deployment

### 1. Environment Setup

```bash
# Set environment variables
export NAMESPACE=uveddi-prod
export ENVIRONMENT=production
export DOMAIN=uveddi.com
export SSL_EMAIL=admin@uveddi.com
```

### 2. Create Namespace and RBAC

```bash
# Create namespace
kubectl create namespace $NAMESPACE

# Apply RBAC configuration
kubectl apply -f k8s/base/serviceaccount.yaml -n $NAMESPACE
```

### 3. Configure Secrets

```bash
# Generate secure passwords
export DB_PASSWORD=$(openssl rand -base64 32)
export JWT_SECRET=$(openssl rand -base64 32)
export API_KEY=$(openssl rand -base64 32)

# Create Kubernetes secrets
kubectl create secret generic uveddi-secrets \
  --from-literal=database-password="$DB_PASSWORD" \
  --from-literal=jwt-secret="$JWT_SECRET" \
  --from-literal=api-key="$API_KEY" \
  --namespace=$NAMESPACE
```

### 4. Deploy with Helm

```bash
# Update dependencies
helm dependency update helm/uveddi

# Deploy Uveddi
helm upgrade --install uveddi-prod helm/uveddi \
  --namespace $NAMESPACE \
  --values helm/uveddi/values-production.yaml \
  --set global.domain=$DOMAIN \
  --set environment=production \
  --wait --timeout 10m
```

### 5. Configure SSL/TLS

```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create Let's Encrypt issuer
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: $SSL_EMAIL
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

### 6. Setup Monitoring

```bash
# Deploy Prometheus and Grafana
kubectl apply -f k8s/monitoring/ -n $NAMESPACE

# Import Grafana dashboards
kubectl exec -n $NAMESPACE deployment/grafana -- \
  grafana-cli admin reset-admin-password newpassword
```

### 7. Configure Backups

```bash
# Create backup CronJob
kubectl apply -f k8s/base/backup-cronjob.yaml -n $NAMESPACE

# Test backup manually
kubectl create job --from=cronjob/database-backup manual-backup -n $NAMESPACE
```

## Health Checks

Run comprehensive health checks:

```bash
./scripts/health-check.sh --namespace uveddi-prod
```

Expected output:
```
=== Pod Status ===
✓ uveddi-api-5d9f8c6b7d-abc123: 1/1 (Running)
✓ uveddi-frontend-7b8d9f5c4-def456: 1/1 (Running)
✓ postgres-0: 1/1 (Running)
✓ redis-master-0: 1/1 (Running)

=== Service Health Checks ===
Checking uveddi-api... ✓ Healthy (HTTP 200)
Checking database... ✓ Healthy (HTTP 200)
Checking metrics... ✓ Healthy (HTTP 200)
```

## Performance Tuning

### Resource Limits

Configure appropriate resource limits in `values-production.yaml`:

```yaml
resources:
  api:
    limits:
      cpu: 2000m
      memory: 4Gi
    requests:
      cpu: 1000m
      memory: 2Gi
  frontend:
    limits:
      cpu: 500m
      memory: 1Gi
    requests:
      cpu: 250m
      memory: 512Mi
```

### Autoscaling

Enable horizontal pod autoscaling:

```yaml
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80
```

### Database Optimization

```sql
-- Optimize PostgreSQL settings
ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
```

## Security Configuration

### Network Policies

Apply network policies to restrict pod-to-pod communication:

```bash
kubectl apply -f k8s/base/networkpolicy.yaml -n $NAMESPACE
```

### Pod Security Standards

Enforce security standards:

```yaml
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 1001
  fsGroup: 1001
  seccompProfile:
    type: RuntimeDefault

securityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop:
    - ALL
```

### Secret Rotation

Rotate secrets regularly:

```bash
# Generate new secrets
NEW_JWT_SECRET=$(openssl rand -base64 32)

# Update Kubernetes secret
kubectl patch secret uveddi-secrets -n $NAMESPACE \
  --type='json' \
  -p='[{"op": "replace", "path": "/data/jwt-secret", "value": "'$(echo -n $NEW_JWT_SECRET | base64)'"}]'

# Restart pods to pick up new secret
kubectl rollout restart deployment/uveddi-api -n $NAMESPACE
```

## Disaster Recovery

### Backup Procedures

1. **Database Backups** (automated daily):
```bash
# Manual backup
kubectl exec -n $NAMESPACE postgres-0 -- \
  pg_dump -U uveddi uveddi_prod | gzip > backup-$(date +%Y%m%d).sql.gz
```

2. **Configuration Backups**:
```bash
# Backup all configurations
kubectl get all,cm,secret,pvc,ingress -n $NAMESPACE -o yaml > namespace-backup.yaml
```

3. **Persistent Volume Backups**:
```bash
# Snapshot PVCs
kubectl get pvc -n $NAMESPACE -o json > pvc-backup.json
```

### Restore Procedures

1. **Database Restore**:
```bash
# Restore from backup
gunzip -c backup-20240101.sql.gz | \
  kubectl exec -i -n $NAMESPACE postgres-0 -- \
  psql -U uveddi uveddi_prod
```

2. **Full Namespace Restore**:
```bash
# Restore namespace from backup
kubectl apply -f namespace-backup.yaml
```

### Rollback Procedures

```bash
# List Helm releases
helm list -n $NAMESPACE

# Rollback to previous version
helm rollback uveddi-prod -n $NAMESPACE

# Rollback to specific revision
helm rollback uveddi-prod 3 -n $NAMESPACE
```

## Monitoring and Alerts

### Key Metrics to Monitor

- **Application Metrics**:
  - Request rate and latency
  - Error rate
  - Analysis completion time
  - Active connections

- **Infrastructure Metrics**:
  - CPU and memory usage
  - Disk I/O and space
  - Network throughput
  - Pod restart count

### Alert Configuration

Configure alerts in Prometheus:

```yaml
groups:
- name: uveddi-alerts
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
    for: 5m
    annotations:
      summary: "High error rate detected"

  - alert: PodMemoryUsage
    expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.9
    for: 5m
    annotations:
      summary: "Pod memory usage above 90%"
```

### Grafana Dashboards

Access Grafana dashboards:

```bash
# Get Grafana URL
kubectl get ingress grafana -n monitoring

# Default credentials
Username: admin
Password: <configured-password>
```

## Troubleshooting

### Common Issues

1. **Pods not starting**:
```bash
# Check pod logs
kubectl logs -f deployment/uveddi-api -n $NAMESPACE

# Describe pod for events
kubectl describe pod <pod-name> -n $NAMESPACE
```

2. **Database connection issues**:
```bash
# Test database connectivity
kubectl exec -n $NAMESPACE deployment/uveddi-api -- \
  pg_isready -h postgres-service -U uveddi
```

3. **SSL certificate issues**:
```bash
# Check certificate status
kubectl get certificate -n $NAMESPACE
kubectl describe certificate uveddi-tls -n $NAMESPACE
```

4. **Performance issues**:
```bash
# Check resource usage
kubectl top pods -n $NAMESPACE
kubectl top nodes
```

### Debug Mode

Enable debug logging:

```bash
# Update deployment with debug logging
kubectl set env deployment/uveddi-api RUST_LOG=debug -n $NAMESPACE
```

## Maintenance

### Regular Tasks

- **Daily**:
  - Check health status
  - Review error logs
  - Monitor resource usage

- **Weekly**:
  - Review metrics and alerts
  - Check backup integrity
  - Update documentation

- **Monthly**:
  - Security updates
  - Performance analysis
  - Capacity planning

### Upgrade Procedures

```bash
# Test upgrade in staging first
helm upgrade --install uveddi-staging helm/uveddi \
  --namespace uveddi-staging \
  --values helm/uveddi/values-staging.yaml \
  --set image.tag=new-version

# If successful, upgrade production
helm upgrade uveddi-prod helm/uveddi \
  --namespace $NAMESPACE \
  --values helm/uveddi/values-production.yaml \
  --set image.tag=new-version \
  --wait --timeout 10m
```

## Support

For production support:
- GitHub Issues: https://github.com/botzrDev/uveddi/issues
- Documentation: https://docs.uveddi.dev
- Email: support@uveddi.dev

## Appendix

### Environment Variables Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `JWT_SECRET` | JWT signing secret | Required |
| `API_KEY` | API authentication key | Required |
| `LOG_LEVEL` | Logging level | `info` |
| `METRICS_ENABLED` | Enable Prometheus metrics | `true` |

### Port Reference

| Service | Port | Protocol | Description |
|---------|------|----------|-------------|
| API | 8080 | HTTP | Main API endpoint |
| Metrics | 9090 | HTTP | Prometheus metrics |
| Frontend | 8001 | HTTP | Web interface |
| Rendering | 3001 | HTTP | Rendering service |
| PostgreSQL | 5432 | TCP | Database |
| Redis | 6379 | TCP | Cache |

### Checklist

Production deployment checklist:

- [ ] Kubernetes cluster ready
- [ ] Domain configured
- [ ] SSL certificates configured
- [ ] Secrets created
- [ ] Database deployed
- [ ] Monitoring configured
- [ ] Backups scheduled
- [ ] Health checks passing
- [ ] Load testing completed
- [ ] Documentation updated
- [ ] Team notified