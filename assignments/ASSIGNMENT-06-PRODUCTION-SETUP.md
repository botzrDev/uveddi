# Assignment 6: Production Environment Setup 🏭

**Estimated Time:** 1 week  
**Priority:** MEDIUM - Deployment readiness  
**Assigned Developer:** [Your Name Here]  
**Status:** Not Started  
**Prerequisites:** Assignments 1-5 must be completed

## Problem Statement

While Uveddi has sophisticated infrastructure components (Kubernetes configs, monitoring setup, Docker containers), the production deployment process needs to be validated, documented, and made reliable. The production environment setup should leverage existing infrastructure while ensuring security, monitoring, and backup procedures are operational.

**Current State:**
- Kubernetes Helm charts exist but deployment status unknown
- Monitoring infrastructure configured but not verified in production
- SSL/TLS and security hardening status unclear
- Database production setup and backup procedures need validation
- Production environment variables and secrets management needs verification

## Task Description

Set up a reliable production deployment using existing infrastructure configurations, with comprehensive monitoring, security hardening, and operational procedures.

## Specific Actions

### 1. Validate Existing Infrastructure Configuration

**Assess Kubernetes deployment configuration:**
```bash
# Review existing Helm charts
ls -la k8s/helm/
ls -la k8s/helm/uveddi/

# Check Helm chart structure  
find k8s/helm/uveddi -name "*.yaml" | head -10

# Validate Helm chart syntax
helm lint k8s/helm/uveddi
```

**Review monitoring setup:**
```bash
# Check monitoring configuration
ls -la scripts/monitoring.sh
ls -la monitoring/

# Look for Prometheus/Grafana configs
find . -name "*prometheus*" -o -name "*grafana*" | head -10
```

**Assess Docker configuration:**
```bash
# Review Docker setup
ls -la Dockerfile* docker-compose*.yml
find . -name "Dockerfile*" | head -5
```

### 2. Deploy to Production-Like Environment

**Deploy using existing Kubernetes configurations:**
```bash
# Create namespace for production
kubectl create namespace uveddi-prod

# Install using Helm chart
helm install uveddi-prod k8s/helm/uveddi \
  --namespace uveddi-prod \
  --set environment=production \
  --set image.tag=latest \
  --dry-run --debug

# If dry-run succeeds, deploy for real
helm install uveddi-prod k8s/helm/uveddi \
  --namespace uveddi-prod \
  --set environment=production
```

**Verify deployment:**
```bash
# Check pod status
kubectl get pods -n uveddi-prod

# Check services and ingress
kubectl get svc,ingress -n uveddi-prod

# View logs
kubectl logs -l app=uveddi-api -n uveddi-prod
kubectl logs -l app=uveddi-frontend -n uveddi-prod
```

### 3. Configure Production Monitoring

**Set up monitoring stack:**
```bash
# Use existing monitoring setup
./scripts/monitoring.sh start

# Configure alert webhooks (example for Slack)
export ALERT_WEBHOOK="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"

# Test monitoring endpoints
curl http://localhost:9090  # Prometheus
curl http://localhost:3000  # Grafana
```

**Configure alerting rules:**
```yaml
# monitoring/alerts/uveddi-alerts.yml
groups:
- name: uveddi.rules
  rules:
  - alert: UveddiAPIDown
    expr: up{job="uveddi-api"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Uveddi API is down"
      description: "Uveddi API has been down for more than 1 minute"

  - alert: UveddiFrontendDown  
    expr: up{job="uveddi-frontend"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Uveddi Frontend is down"

  - alert: UveddiHighErrorRate
    expr: rate(http_requests_total{job="uveddi-api",status=~"5.."}[5m]) > 0.1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High error rate in Uveddi API"
```

### 4. Configure SSL/TLS and Security

**Set up SSL certificates:**
```bash
# If using Let's Encrypt with cert-manager
kubectl apply --validate=false -f https://github.com/jetstack/cert-manager/releases/download/v1.5.0/cert-manager.yaml

# Create certificate issuer
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@domain.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

**Update ingress with SSL:**
```yaml
# k8s/helm/uveddi/templates/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: uveddi-ingress
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - your-domain.com
    secretName: uveddi-tls
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: uveddi-api
            port:
              number: 8000
      - path: /
        pathType: Prefix
        backend:
          service:
            name: uveddi-frontend  
            port:
              number: 8001
```

### 5. Set Up Production Database

**Configure PostgreSQL for production:**
```bash
# If using managed database service (recommended)
# Configure connection secrets

# If self-hosting, set up with proper configuration
kubectl create secret generic postgresql-secret \
  --from-literal=username=uveddi_user \
  --from-literal=password=secure_random_password \
  --from-literal=database=uveddi_prod

# Create persistent volume for data
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 100Gi
EOF
```

**Set up database backup procedures:**
```bash
# Create backup script
cat <<'EOF' > scripts/backup-database.sh
#!/bin/bash
set -e

BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="uveddi_prod"

# Create backup directory
mkdir -p $BACKUP_DIR

# Perform backup
pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME | gzip > "$BACKUP_DIR/uveddi_backup_$DATE.sql.gz"

# Cleanup old backups (keep last 7 days)
find $BACKUP_DIR -name "uveddi_backup_*.sql.gz" -mtime +7 -delete

echo "Backup completed: uveddi_backup_$DATE.sql.gz"
EOF

chmod +x scripts/backup-database.sh
```

**Schedule automated backups:**
```yaml
# k8s/cronjobs/backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: database-backup
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:13
            command: ["/scripts/backup-database.sh"]
            env:
            - name: DB_HOST
              value: "postgres-service"
            - name: DB_USER
              valueFrom:
                secretKeyRef:
                  name: postgresql-secret
                  key: username
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
```

### 6. Production Environment Configuration

**Set up environment-specific configurations:**
```bash
# Create production environment file
cat <<EOF > .env.production
NODE_ENV=production
API_URL=https://api.your-domain.com/api/v1
FRONTEND_URL=https://your-domain.com
DATABASE_URL=postgresql://user:pass@postgres:5432/uveddi_prod
REDIS_URL=redis://redis:6379
LOG_LEVEL=info
METRICS_ENABLED=true
PROMETHEUS_PORT=9090
EOF
```

**Configure secrets management:**
```bash
# Create Kubernetes secrets
kubectl create secret generic uveddi-secrets \
  --from-literal=jwt-secret=your-jwt-secret \
  --from-literal=api-key=your-api-key \
  --from-literal=database-url=your-database-url \
  --namespace uveddi-prod
```

### 7. Performance and Load Testing

**Test production deployment performance:**
```bash
# Load test API endpoints
ab -n 1000 -c 50 https://your-domain.com/api/v1/health
ab -n 500 -c 25 https://your-domain.com/api/v1/reports/latest

# Monitor resource usage during load test
kubectl top pods -n uveddi-prod
kubectl top nodes
```

**Set up continuous monitoring:**
```bash
# Configure resource limits and requests
# Update Helm values for production
cat <<EOF > values-production.yaml
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

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
EOF
```

## Acceptance Criteria

- [ ] Kubernetes deployment succeeds and pods are running
- [ ] All services are accessible via HTTPS with valid SSL certificates
- [ ] Monitoring stack (Prometheus/Grafana) is operational with alerts
- [ ] Database is configured with automated backup procedures
- [ ] Production environment variables and secrets are properly managed
- [ ] Load testing shows acceptable performance (response times < 500ms)
- [ ] Resource limits and autoscaling are configured appropriately
- [ ] Health checks pass for all deployed services
- [ ] Logs are properly aggregated and searchable
- [ ] Rollback procedures are tested and documented

## Detailed Implementation Approach

### Days 1-2: Infrastructure Assessment and Setup
- Review and validate existing Kubernetes/Helm configurations
- Set up production namespace and basic deployment
- Configure SSL/TLS certificates and ingress
- Verify monitoring stack deployment

### Days 3-4: Database and Security Configuration  
- Set up production database with proper security
- Configure automated backup procedures
- Implement secrets management and environment configuration
- Test SSL certificate generation and renewal

### Days 5-6: Performance and Reliability
- Configure resource limits, requests, and autoscaling
- Set up comprehensive alerting rules
- Perform load testing and performance optimization  
- Configure log aggregation and monitoring

### Day 7: Documentation and Procedures
- Document deployment procedures
- Create runbooks for common operational tasks
- Test disaster recovery and rollback procedures
- Final production readiness validation

## Technical Considerations

**Security Hardening:**
- Use non-root containers
- Enable network policies for pod-to-pod communication
- Regular security scanning of container images
- Proper RBAC configuration for Kubernetes access

**High Availability:**
- Multiple replicas for all services
- Proper readiness and liveness probes
- Database replication or managed service
- Load balancing and failover procedures

**Observability:**
- Comprehensive logging with structured formats
- Distributed tracing for complex requests
- Business metrics in addition to infrastructure metrics
- Regular SLO monitoring and reporting

## Risk Assessment

**Risk Level:** HIGH - Production deployments can have complex failure modes

**Potential Issues:**
- Infrastructure configuration issues may cause deployment failures
- SSL certificate generation may fail in production environment
- Database migration issues could cause data loss
- Performance under real load may be inadequate
- Monitoring/alerting may not cover all failure scenarios

**Mitigation Strategies:**
- Test deployment in staging environment first
- Use infrastructure-as-code for reproducible deployments
- Implement comprehensive backup and recovery procedures
- Plan for gradual rollout with quick rollback capability

## Dependencies

- **Completed:** All previous assignments (1-5)
- **Required:** Production Kubernetes cluster access
- **Required:** Domain name and DNS configuration
- **Required:** SSL certificate authority access (Let's Encrypt or commercial)
- **Optional:** Managed database service
- **Optional:** CDN for frontend assets

## Success Metrics

- **Availability:** 99.9% uptime for production services
- **Performance:** API response times under 500ms for 95th percentile
- **Security:** All security scans pass, SSL rating A+ 
- **Monitoring:** Zero false positives/negatives in alerting
- **Recovery:** Database restore procedures tested and documented

## Production Readiness Checklist

- [ ] All services deployed and running in production
- [ ] SSL certificates valid and auto-renewal configured
- [ ] Monitoring and alerting fully operational
- [ ] Backup procedures tested and automated
- [ ] Performance benchmarks established and monitored
- [ ] Security hardening implemented and verified
- [ ] Disaster recovery procedures documented and tested
- [ ] Production support procedures established
- [ ] User documentation updated for production features

## Next Steps After Completion

Once production environment is operational:
- Begin user acceptance testing with real users
- Implement gradual feature rollouts and A/B testing
- Establish ongoing maintenance and update procedures
- Plan for scaling and capacity management
- Set up incident response and on-call procedures

---
**Assignment Created:** 2025-09-08  
**Last Updated:** 2025-09-08  
**Estimated Completion:** TBD