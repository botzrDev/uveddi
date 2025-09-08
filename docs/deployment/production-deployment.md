# Uveddi Production Deployment Guide

> **Status**: v1.0-alpha Production Readiness  
> **Last Updated**: September 2025  
> **Target Audience**: DevOps Engineers, SREs, Production Administrators

## Table of Contents

1. [Production Readiness Overview](#production-readiness-overview)
2. [Pre-Production Checklist](#pre-production-checklist)
3. [Production Deployment Methods](#production-deployment-methods)
4. [Environment Configuration](#environment-configuration)
5. [Security Hardening](#security-hardening)
6. [Monitoring & Observability](#monitoring--observability)
7. [Backup & Recovery](#backup--recovery)
8. [Scaling & Performance](#scaling--performance)
9. [Troubleshooting Production Issues](#troubleshooting-production-issues)
10. [Maintenance & Operations](#maintenance--operations)

---

## Production Readiness Overview

### Current Production Status

**Uveddi v1.0-alpha** is production-ready for **core code analysis workloads** with the following caveats:

✅ **Production-Ready Components:**
- CLI-based static code analysis
- JSON/HTML/Markdown report generation
- Anti-pattern detection with tree-sitter
- Multi-language support (Rust, Python, JavaScript)
- PostgreSQL database integration
- Basic API server functionality

⚠️ **Alpha Components (Use with Caution):**
- Web Dashboard UI (functional but not fully stable)
- WASM Plugin system (working but limited ecosystem)
- TypeScript analysis (basic support only)
- AI integration features (experimental)
- Real-time WebSocket features

❌ **Not Production-Ready:**
- Advanced web features (complex dashboard interactions)
- Enterprise authentication/authorization
- Multi-tenant deployments

### Recommended Production Scenarios

**Ideal for Production:**
- CI/CD pipeline integration for code quality checks
- Automated code review workflows
- Scheduled analysis reporting
- Command-line analysis tools for development teams
- Basic web-based report viewing

**Not Recommended for Production:**
- User-facing interactive web applications
- Real-time collaborative features
- Mission-critical enterprise workflows requiring 99.99% uptime
- Large-scale multi-tenant SaaS deployments

---

## Pre-Production Checklist

### Core Functionality Validation

```bash
# 1. Verify installation and basic functionality
uveddi --version
uveddi doctor

# 2. Test core analysis capabilities
uveddi analyze ./sample-project --output json
uveddi analyze ./sample-project --output html
uveddi analyze ./sample-project --features tree-sitter

# 3. Verify database connectivity
uveddi db migrate
uveddi db health-check

# 4. Test API server functionality
uveddi serve --port 8080 &
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/status
```

### Security Validation

```bash
# 1. Verify secure configuration
uveddi config validate --environment production

# 2. Check for exposed secrets
grep -r "password\|secret\|key" /etc/uveddi/ || echo "No hardcoded secrets found"

# 3. Test authentication (if enabled)
curl -H "Authorization: Bearer $UVEDDI_JWT_TOKEN" http://localhost:8080/api/v1/protected

# 4. Validate SSL/TLS configuration
openssl s_client -connect your-domain.com:443 -verify_hostname your-domain.com
```

### Performance Validation

```bash
# 1. Memory usage under load
ulimit -v 8388608  # 8GB limit
uveddi analyze ./large-codebase --timeout 600000

# 2. Concurrent analysis capability
for i in {1..5}; do
  uveddi analyze ./project-$i --output json &
done
wait

# 3. Database performance
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U $DB_USER -d $DB_NAME -c "\timing on; SELECT COUNT(*) FROM analysis_results;"
```

### Production Environment Checklist

- [ ] **System Requirements Met**
  - [ ] Minimum 8GB RAM, 16GB recommended
  - [ ] SSD storage with >100GB free space
  - [ ] Ubuntu 22.04 LTS or RHEL 9
  - [ ] Dedicated user account (`uveddi`)
  - [ ] Proper file system permissions

- [ ] **Network Configuration**
  - [ ] Firewall rules configured
  - [ ] Reverse proxy (Nginx/Apache) configured
  - [ ] SSL certificates installed and valid
  - [ ] DNS resolution working
  - [ ] Load balancer configured (if applicable)

- [ ] **Database Setup**
  - [ ] PostgreSQL 14+ installed and configured
  - [ ] Database user with minimal required permissions
  - [ ] Connection pooling configured
  - [ ] Backup strategy implemented
  - [ ] Monitoring and alerting enabled

- [ ] **Monitoring & Logging**
  - [ ] Log rotation configured
  - [ ] Prometheus metrics collection
  - [ ] Grafana dashboards deployed
  - [ ] Alert rules configured
  - [ ] Log aggregation (ELK/Loki) setup

- [ ] **Security Hardening**
  - [ ] Service running as non-root user
  - [ ] Environment variables secured
  - [ ] API authentication enabled
  - [ ] Rate limiting configured
  - [ ] Security headers implemented

---

## Production Deployment Methods

### Method 1: Containerized Deployment (Recommended)

Create a production-ready Docker setup:

```dockerfile
# Dockerfile.production
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features production

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/uveddi /usr/local/bin/
COPY config/production.toml /etc/uveddi/config.toml

RUN useradd --system --shell /bin/false uveddi
USER uveddi
EXPOSE 8080 9090

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["uveddi", "serve", "--config", "/etc/uveddi/config.toml"]
```

```yaml
# docker-compose.production.yml
version: '3.8'

services:
  uveddi:
    build:
      context: .
      dockerfile: Dockerfile.production
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - UVEDDI_ENV=production
      - UVEDDI_DATABASE_URL=postgresql://uveddi:${DB_PASSWORD}@postgres:5432/uveddi
      - UVEDDI_LOG_LEVEL=info
      - UVEDDI_JWT_SECRET=${JWT_SECRET}
    volumes:
      - ./data:/var/lib/uveddi
      - ./logs:/var/log/uveddi
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 8G
          cpus: '4'
        reservations:
          memory: 2G
          cpus: '1'

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=uveddi
      - POSTGRES_USER=uveddi
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '2'

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped
    command: redis-server --appendonly yes --maxmemory 1gb --maxmemory-policy allkeys-lru

  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - ./nginx/production.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/ssl/certs:ro
    depends_on:
      - uveddi
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

### Method 2: Kubernetes Deployment

```yaml
# k8s/production/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: uveddi-production
  labels:
    name: uveddi-production
---
# k8s/production/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: uveddi-config
  namespace: uveddi-production
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    max_connections = 1000
    
    [database]
    url = "postgresql://uveddi:password@postgres:5432/uveddi"
    pool_size = 20
    
    [logging]
    level = "info"
    format = "json"
    
    [analysis]
    max_concurrent = 10
    timeout_ms = 300000
---
# k8s/production/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uveddi
  namespace: uveddi-production
  labels:
    app: uveddi
spec:
  replicas: 3
  selector:
    matchLabels:
      app: uveddi
  template:
    metadata:
      labels:
        app: uveddi
    spec:
      containers:
      - name: uveddi
        image: uveddi:1.0-production
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: UVEDDI_ENV
          value: "production"
        - name: UVEDDI_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: uveddi-secrets
              key: database-url
        - name: UVEDDI_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: uveddi-secrets
              key: jwt-secret
        volumeMounts:
        - name: config
          mountPath: /etc/uveddi
        - name: data
          mountPath: /var/lib/uveddi
        resources:
          requests:
            memory: "2Gi"
            cpu: "500m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: uveddi-config
      - name: data
        persistentVolumeClaim:
          claimName: uveddi-data
---
# k8s/production/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: uveddi-service
  namespace: uveddi-production
spec:
  selector:
    app: uveddi
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
---
# k8s/production/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: uveddi-ingress
  namespace: uveddi-production
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - uveddi.yourdomain.com
    secretName: uveddi-tls
  rules:
  - host: uveddi.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: uveddi-service
            port:
              number: 80
```

### Method 3: Binary Deployment with SystemD

```bash
#!/bin/bash
# deploy-production.sh

set -euo pipefail

# Configuration
UVEDDI_VERSION="1.0.0"
UVEDDI_USER="uveddi"
INSTALL_DIR="/opt/uveddi"
CONFIG_DIR="/etc/uveddi"
DATA_DIR="/var/lib/uveddi"
LOG_DIR="/var/log/uveddi"

# Create directories and user
sudo mkdir -p $INSTALL_DIR $CONFIG_DIR $DATA_DIR $LOG_DIR
sudo useradd --system --home $DATA_DIR --shell /bin/false $UVEDDI_USER 2>/dev/null || true

# Download and install binary
wget "https://github.com/uveddi/uveddi/releases/download/v${UVEDDI_VERSION}/uveddi-linux-x86_64.tar.gz"
tar -xzf uveddi-linux-x86_64.tar.gz
sudo cp uveddi $INSTALL_DIR/
sudo chmod +x $INSTALL_DIR/uveddi
sudo chown $UVEDDI_USER:$UVEDDI_USER $INSTALL_DIR/uveddi

# Set up configuration
sudo cp config/production.toml $CONFIG_DIR/
sudo chown $UVEDDI_USER:$UVEDDI_USER $CONFIG_DIR/production.toml
sudo chmod 600 $CONFIG_DIR/production.toml

# Set up systemd service
sudo tee /etc/systemd/system/uveddi.service > /dev/null <<EOF
[Unit]
Description=Uveddi Code Analysis Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$UVEDDI_USER
Group=$UVEDDI_USER
WorkingDirectory=$DATA_DIR
ExecStart=$INSTALL_DIR/uveddi serve --config $CONFIG_DIR/production.toml
Restart=always
RestartSec=5
TimeoutStopSec=30

# Security settings
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=$DATA_DIR $LOG_DIR
PrivateTmp=yes
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes

# Resource limits
LimitNOFILE=65536
LimitNPROC=32768
MemoryLimit=8G

[Install]
WantedBy=multi-user.target
EOF

# Start and enable service
sudo systemctl daemon-reload
sudo systemctl enable uveddi
sudo systemctl start uveddi
sudo systemctl status uveddi
```

---

## Environment Configuration

### Production Environment Variables

```bash
# /etc/uveddi/environment.production
# Core Configuration
export UVEDDI_ENV=production
export UVEDDI_CONFIG_PATH=/etc/uveddi/production.toml
export UVEDDI_LOG_LEVEL=info
export UVEDDI_LOG_FORMAT=json

# Server Configuration
export UVEDDI_HOST=0.0.0.0
export UVEDDI_PORT=8080
export UVEDDI_MAX_CONNECTIONS=1000
export UVEDDI_REQUEST_TIMEOUT=60s
export UVEDDI_GRACEFUL_SHUTDOWN_TIMEOUT=30s

# Database Configuration (Use secrets management in production)
export UVEDDI_DATABASE_URL="postgresql://uveddi:$(cat /etc/uveddi/secrets/db_password)@localhost:5432/uveddi_prod"
export UVEDDI_DATABASE_POOL_SIZE=20
export UVEDDI_DATABASE_MAX_LIFETIME=3600s
export UVEDDI_DATABASE_CONNECT_TIMEOUT=10s

# Performance Configuration
export UVEDDI_WORKER_THREADS=8
export UVEDDI_MEMORY_LIMIT_MB=8192
export UVEDDI_ENABLE_PARALLEL_PROCESSING=true
export UVEDDI_MAX_CONCURRENT_ANALYSES=10

# Caching Configuration
export UVEDDI_CACHE_PATH=/var/lib/uveddi/cache
export UVEDDI_CACHE_SIZE_MB=2048
export UVEDDI_CACHE_TTL_HOURS=24

# Security Configuration
export UVEDDI_JWT_SECRET="$(cat /etc/uveddi/secrets/jwt_secret)"
export UVEDDI_JWT_EXPIRATION_HOURS=8
export UVEDDI_REQUIRE_AUTHENTICATION=true
export UVEDDI_RATE_LIMIT_REQUESTS_PER_MINUTE=100

# Monitoring Configuration
export UVEDDI_METRICS_PORT=9090
export UVEDDI_HEALTH_CHECK_INTERVAL=30s
export UVEDDI_ENABLE_TRACING=true
```

### Production TOML Configuration

```toml
# /etc/uveddi/production.toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 1000
request_timeout = "60s"
graceful_shutdown_timeout = "30s"
enable_cors = false
allowed_origins = ["https://yourdomain.com"]

[database]
url = "${UVEDDI_DATABASE_URL}"
pool_size = 20
max_lifetime = "3600s"
connect_timeout = "10s"
enable_logging = false

[logging]
level = "info"
format = "json"
file = "/var/log/uveddi/uveddi.log"
max_size = "100MB"
max_age = "30d"
max_backups = 10
compress = true

[analysis]
max_concurrent = 10
timeout_ms = 300000
enable_ai_default = false
default_features = ["tree-sitter"]
cache_results = true

[security]
jwt_secret = "${UVEDDI_JWT_SECRET}"
jwt_expiration_hours = 8
require_authentication = true
rate_limit_rpm = 100
enable_cors = false
allowed_origins = ["https://yourdomain.com"]

[cache]
path = "/var/lib/uveddi/cache"
size_mb = 2048
ttl_hours = 24

[monitoring]
enable_metrics = true
metrics_port = 9090
enable_tracing = true
health_check_interval = "30s"

[plugins]
enabled = false
path = "/var/lib/uveddi/plugins"
timeout_ms = 30000
```

---

## Security Hardening

### System-Level Security

```bash
#!/bin/bash
# security-hardening.sh

# 1. Firewall Configuration
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 443/tcp  # HTTPS
sudo ufw allow from 10.0.0.0/8 to any port 8080  # Internal API access only
sudo ufw allow from 10.0.0.0/8 to any port 9090  # Internal metrics access only
sudo ufw --force enable

# 2. Secure file permissions
sudo chmod 600 /etc/uveddi/secrets/*
sudo chmod 750 /var/lib/uveddi
sudo chown -R uveddi:uveddi /var/lib/uveddi
sudo chown -R uveddi:uveddi /var/log/uveddi

# 3. Disable unused services
sudo systemctl disable bluetooth
sudo systemctl disable cups
sudo systemctl disable avahi-daemon

# 4. Set up fail2ban
sudo apt-get install -y fail2ban
sudo tee /etc/fail2ban/jail.d/uveddi.conf > /dev/null <<EOF
[uveddi]
enabled = true
port = 8080
filter = uveddi
logpath = /var/log/uveddi/uveddi.log
maxretry = 10
bantime = 600
findtime = 60
EOF
```

### Application Security

```nginx
# /etc/nginx/sites-available/uveddi.conf
server {
    listen 443 ssl http2;
    server_name uveddi.yourdomain.com;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/uveddi.crt;
    ssl_certificate_key /etc/ssl/private/uveddi.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-Frame-Options DENY always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self';" always;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/m;
    limit_req zone=api burst=20 nodelay;

    # Hide server information
    server_tokens off;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        proxy_buffering off;
    }

    location /metrics {
        # Restrict metrics access
        allow 10.0.0.0/8;
        deny all;
        proxy_pass http://127.0.0.1:9090;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name uveddi.yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

### Secrets Management

```bash
#!/bin/bash
# setup-secrets.sh

# Create secrets directory
sudo mkdir -p /etc/uveddi/secrets
sudo chmod 700 /etc/uveddi/secrets

# Generate JWT secret
openssl rand -hex 32 | sudo tee /etc/uveddi/secrets/jwt_secret > /dev/null

# Create database password
openssl rand -base64 32 | sudo tee /etc/uveddi/secrets/db_password > /dev/null

# Set appropriate permissions
sudo chown root:uveddi /etc/uveddi/secrets/*
sudo chmod 640 /etc/uveddi/secrets/*

# For production, consider using:
# - AWS Secrets Manager
# - HashiCorp Vault
# - Kubernetes Secrets
# - Azure Key Vault
```

---

## Monitoring & Observability

### Prometheus Configuration

```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alerts/uveddi.yml"

scrape_configs:
  - job_name: 'uveddi'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
    metrics_path: /metrics

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']

  - job_name: 'postgres-exporter'
    static_configs:
      - targets: ['localhost:9187']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Alert Rules

```yaml
# alerts/uveddi.yml
groups:
- name: uveddi
  rules:
  - alert: UveddiServiceDown
    expr: up{job="uveddi"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Uveddi service is down"
      description: "Uveddi service has been down for more than 1 minute"

  - alert: UveddiHighMemoryUsage
    expr: process_resident_memory_bytes{job="uveddi"} / 1024 / 1024 / 1024 > 6
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Uveddi high memory usage"
      description: "Uveddi is using more than 6GB of memory"

  - alert: UveddiHighErrorRate
    expr: rate(uveddi_http_requests_total{status=~"5.."}[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Uveddi high error rate"
      description: "Error rate is above 10% for 2 minutes"

  - alert: UveddiDatabaseConnectionFailures
    expr: rate(uveddi_database_connection_failures_total[5m]) > 0.01
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Uveddi database connection failures"
      description: "Database connection failure rate is above 1%"

  - alert: UveddiLongRunningAnalysis
    expr: uveddi_analysis_duration_seconds > 600
    for: 0m
    labels:
      severity: warning
    annotations:
      summary: "Long running analysis detected"
      description: "Analysis has been running for more than 10 minutes"
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "id": null,
    "title": "Uveddi Production Dashboard",
    "tags": ["uveddi", "production"],
    "timezone": "browser",
    "panels": [
      {
        "title": "Service Status",
        "type": "stat",
        "targets": [
          {
            "expr": "up{job=\"uveddi\"}",
            "legendFormat": "Service Up"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "mappings": [
              {"value": 0, "text": "DOWN", "color": "red"},
              {"value": 1, "text": "UP", "color": "green"}
            ]
          }
        }
      },
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(uveddi_http_requests_total[5m])",
            "legendFormat": "{{method}} {{status}}"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "process_resident_memory_bytes{job=\"uveddi\"} / 1024 / 1024 / 1024",
            "legendFormat": "Memory GB"
          }
        ]
      },
      {
        "title": "Active Analyses",
        "type": "graph",
        "targets": [
          {
            "expr": "uveddi_active_analyses",
            "legendFormat": "Active Analyses"
          }
        ]
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "5s"
  }
}
```

---

## Backup & Recovery

### Automated Backup Strategy

```bash
#!/bin/bash
# backup-production.sh

set -euo pipefail

BACKUP_DIR="/var/backups/uveddi"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RETENTION_DAYS=30

# Database backup
pg_dump -h localhost -U uveddi -W -F custom -f "$BACKUP_DIR/db_backup_$TIMESTAMP.dump" uveddi_prod

# Application data backup
tar -czf "$BACKUP_DIR/data_backup_$TIMESTAMP.tar.gz" \
    /var/lib/uveddi \
    /etc/uveddi \
    --exclude="/var/lib/uveddi/cache/*" \
    --exclude="/var/lib/uveddi/logs/*"

# Configuration backup
tar -czf "$BACKUP_DIR/config_backup_$TIMESTAMP.tar.gz" \
    /etc/uveddi \
    /etc/systemd/system/uveddi.service \
    /etc/nginx/sites-available/uveddi.conf

# Upload to S3 (optional)
if command -v aws &> /dev/null; then
    aws s3 sync $BACKUP_DIR s3://your-backup-bucket/uveddi/ \
        --exclude "*.tmp" \
        --delete
fi

# Cleanup old backups
find $BACKUP_DIR -type f -mtime +$RETENTION_DAYS -delete

# Test backup integrity
pg_restore --list "$BACKUP_DIR/db_backup_$TIMESTAMP.dump" > /dev/null
tar -tzf "$BACKUP_DIR/data_backup_$TIMESTAMP.tar.gz" > /dev/null

echo "Backup completed successfully: $TIMESTAMP"
```

### Disaster Recovery Procedures

```bash
#!/bin/bash
# disaster-recovery.sh

set -euo pipefail

BACKUP_DIR="/var/backups/uveddi"
RESTORE_TIMESTAMP=${1:-latest}

if [ "$RESTORE_TIMESTAMP" == "latest" ]; then
    RESTORE_TIMESTAMP=$(ls -t $BACKUP_DIR/db_backup_*.dump | head -n1 | grep -o '[0-9]\{8\}_[0-9]\{6\}')
fi

echo "Restoring from backup: $RESTORE_TIMESTAMP"

# 1. Stop services
sudo systemctl stop uveddi
sudo systemctl stop nginx

# 2. Restore database
dropdb -h localhost -U postgres uveddi_prod
createdb -h localhost -U postgres -O uveddi uveddi_prod
pg_restore -h localhost -U uveddi -d uveddi_prod "$BACKUP_DIR/db_backup_$RESTORE_TIMESTAMP.dump"

# 3. Restore application data
sudo rm -rf /var/lib/uveddi/*
sudo tar -xzf "$BACKUP_DIR/data_backup_$RESTORE_TIMESTAMP.tar.gz" -C /

# 4. Restore configuration
sudo tar -xzf "$BACKUP_DIR/config_backup_$RESTORE_TIMESTAMP.tar.gz" -C /

# 5. Fix permissions
sudo chown -R uveddi:uveddi /var/lib/uveddi
sudo chmod 600 /etc/uveddi/secrets/*

# 6. Start services
sudo systemctl start uveddi
sudo systemctl start nginx

# 7. Verify recovery
sleep 10
curl -f http://localhost:8080/health || {
    echo "Health check failed after recovery"
    exit 1
}

echo "Disaster recovery completed successfully"
```

---

## Scaling & Performance

### Load Balancer Configuration

```nginx
# /etc/nginx/upstream.conf
upstream uveddi_backend {
    # Load balancing method
    least_conn;
    
    # Backend servers
    server 10.0.1.10:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.12:8080 max_fails=3 fail_timeout=30s;
    
    # Health check (nginx plus)
    # health_check interval=10s uri=/health;
}

server {
    listen 443 ssl http2;
    server_name uveddi.yourdomain.com;
    
    location / {
        proxy_pass http://uveddi_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Connection pooling
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        
        # Timeouts
        proxy_connect_timeout 10s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
    
    # Sticky sessions (if needed)
    # location / {
    #     proxy_pass http://uveddi_backend;
    #     hash $remote_addr consistent;
    # }
}
```

### Auto-Scaling Configuration

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: uveddi-hpa
  namespace: uveddi-production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: uveddi
  minReplicas: 3
  maxReplicas: 20
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
  - type: Object
    object:
      metric:
        name: uveddi_active_analyses
      target:
        type: AverageValue
        averageValue: "5"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

### Performance Tuning

```bash
#!/bin/bash
# performance-tuning.sh

# 1. System-level optimizations
echo 'vm.swappiness=10' | sudo tee -a /etc/sysctl.conf
echo 'net.core.somaxconn=65535' | sudo tee -a /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog=65535' | sudo tee -a /etc/sysctl.conf
echo 'fs.file-max=1000000' | sudo tee -a /etc/sysctl.conf

# 2. Increase limits
echo 'uveddi soft nofile 65535' | sudo tee -a /etc/security/limits.conf
echo 'uveddi hard nofile 65535' | sudo tee -a /etc/security/limits.conf
echo 'uveddi soft nproc 32768' | sudo tee -a /etc/security/limits.conf
echo 'uveddi hard nproc 32768' | sudo tee -a /etc/security/limits.conf

# 3. Database optimizations
sudo -u postgres psql -c "ALTER SYSTEM SET max_connections = 200;"
sudo -u postgres psql -c "ALTER SYSTEM SET shared_buffers = '4GB';"
sudo -u postgres psql -c "ALTER SYSTEM SET effective_cache_size = '12GB';"
sudo -u postgres psql -c "ALTER SYSTEM SET maintenance_work_mem = '1GB';"
sudo -u postgres psql -c "ALTER SYSTEM SET checkpoint_completion_target = 0.9;"
sudo -u postgres psql -c "ALTER SYSTEM SET wal_buffers = '16MB';"
sudo -u postgres psql -c "SELECT pg_reload_conf();"

# 4. Apply changes
sudo sysctl -p
```

---

## Troubleshooting Production Issues

### Common Issues and Solutions

#### 1. High Memory Usage

```bash
# Monitor memory usage
sudo -u uveddi uveddi monitor --memory

# Check for memory leaks
ps aux | grep uveddi
cat /proc/$(pgrep uveddi)/status | grep -E "(VmSize|VmRSS|VmPeak)"

# Solutions:
# - Increase memory limits in systemd
# - Enable memory optimization features
# - Check for large analysis targets
# - Restart service if memory leak detected
```

#### 2. Database Connection Issues

```bash
# Check database connectivity
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U $DB_USER -d $DB_NAME -c "SELECT 1;"

# Check connection pool
curl http://localhost:8080/debug/database/pool

# Solutions:
# - Increase database connection limits
# - Check network connectivity
# - Verify database credentials
# - Restart database service
```

#### 3. Service Startup Failures

```bash
# Check service logs
sudo journalctl -u uveddi -n 50 --no-pager

# Check configuration
sudo -u uveddi uveddi config validate --config /etc/uveddi/production.toml

# Check file permissions
ls -la /var/lib/uveddi /etc/uveddi /var/log/uveddi

# Solutions:
# - Fix configuration errors
# - Correct file permissions
# - Check environment variables
# - Verify dependencies are running
```

#### 4. Performance Issues

```bash
# Check system resources
htop
iotop
netstat -tuln

# Check application metrics
curl http://localhost:9090/metrics | grep uveddi

# Profile application (if debug enabled)
curl http://localhost:8080/debug/pprof/heap > heap.prof
curl http://localhost:8080/debug/pprof/profile?seconds=30 > cpu.prof

# Solutions:
# - Scale horizontally
# - Optimize database queries
# - Increase system resources
# - Enable caching
```

### Health Check Scripts

```bash
#!/bin/bash
# health-check.sh

set -euo pipefail

HEALTHY=true

# Check service status
if ! systemctl is-active --quiet uveddi; then
    echo "ERROR: Uveddi service is not running"
    HEALTHY=false
fi

# Check HTTP endpoints
if ! curl -f -s http://localhost:8080/health > /dev/null; then
    echo "ERROR: Health endpoint not responding"
    HEALTHY=false
fi

# Check database connectivity
if ! PGPASSWORD=$DB_PASSWORD psql -h localhost -U uveddi -d uveddi_prod -c "SELECT 1;" > /dev/null 2>&1; then
    echo "ERROR: Database not accessible"
    HEALTHY=false
fi

# Check disk space
DISK_USAGE=$(df /var/lib/uveddi | tail -1 | awk '{print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 90 ]; then
    echo "WARNING: Disk usage is ${DISK_USAGE}%"
    HEALTHY=false
fi

# Check memory usage
MEMORY_USAGE=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
if [ "$MEMORY_USAGE" -gt 90 ]; then
    echo "WARNING: Memory usage is ${MEMORY_USAGE}%"
    HEALTHY=false
fi

# Check log errors
ERROR_COUNT=$(sudo tail -n 100 /var/log/uveddi/uveddi.log | grep -c "ERROR" || echo 0)
if [ "$ERROR_COUNT" -gt 10 ]; then
    echo "WARNING: ${ERROR_COUNT} errors in recent logs"
    HEALTHY=false
fi

if [ "$HEALTHY" = true ]; then
    echo "OK: All health checks passed"
    exit 0
else
    echo "CRITICAL: Health checks failed"
    exit 1
fi
```

---

## Maintenance & Operations

### Regular Maintenance Tasks

```bash
#!/bin/bash
# maintenance.sh

# Daily tasks
daily_maintenance() {
    # Backup
    /opt/uveddi/scripts/backup-production.sh
    
    # Log rotation
    sudo logrotate /etc/logrotate.d/uveddi
    
    # Health check
    /opt/uveddi/scripts/health-check.sh
    
    # Update metrics
    curl -X POST http://localhost:8080/admin/collect-metrics
}

# Weekly tasks
weekly_maintenance() {
    # Database maintenance
    sudo -u postgres psql -d uveddi_prod -c "VACUUM ANALYZE;"
    
    # Clean old cache files
    sudo find /var/lib/uveddi/cache -type f -mtime +7 -delete
    
    # Security updates (requires approval)
    sudo apt list --upgradable | grep -i security
    
    # Certificate renewal check
    openssl x509 -in /etc/ssl/certs/uveddi.crt -noout -checkend $((30*24*3600)) || echo "Certificate expires in <30 days"
}

# Monthly tasks
monthly_maintenance() {
    # Performance report
    /opt/uveddi/scripts/performance-report.sh
    
    # Capacity planning report
    /opt/uveddi/scripts/capacity-report.sh
    
    # Security audit
    /opt/uveddi/scripts/security-audit.sh
    
    # Backup verification
    /opt/uveddi/scripts/verify-backups.sh
}

case "${1:-daily}" in
    daily)   daily_maintenance ;;
    weekly)  weekly_maintenance ;;
    monthly) monthly_maintenance ;;
    *)       echo "Usage: $0 {daily|weekly|monthly}" ;;
esac
```

### Upgrade Procedures

```bash
#!/bin/bash
# upgrade-production.sh

set -euo pipefail

NEW_VERSION=${1:?"Usage: $0 <new_version>"}
CURRENT_VERSION=$(uveddi --version | awk '{print $2}')

echo "Upgrading Uveddi from $CURRENT_VERSION to $NEW_VERSION"

# 1. Pre-upgrade backup
echo "Creating pre-upgrade backup..."
/opt/uveddi/scripts/backup-production.sh

# 2. Download new version
echo "Downloading new version..."
wget "https://github.com/uveddi/uveddi/releases/download/v${NEW_VERSION}/uveddi-linux-x86_64.tar.gz"
tar -xzf uveddi-linux-x86_64.tar.gz

# 3. Test new binary
echo "Testing new binary..."
./uveddi --version
./uveddi config validate --config /etc/uveddi/production.toml

# 4. Stop services
echo "Stopping services..."
sudo systemctl stop uveddi

# 5. Install new binary
echo "Installing new binary..."
sudo cp ./uveddi /opt/uveddi/uveddi
sudo chown uveddi:uveddi /opt/uveddi/uveddi
sudo chmod +x /opt/uveddi/uveddi

# 6. Run database migrations
echo "Running database migrations..."
sudo -u uveddi /opt/uveddi/uveddi db migrate --config /etc/uveddi/production.toml

# 7. Start services
echo "Starting services..."
sudo systemctl start uveddi

# 8. Health check
echo "Performing health check..."
sleep 10
/opt/uveddi/scripts/health-check.sh

# 9. Verify upgrade
NEW_VERSION_ACTUAL=$(sudo -u uveddi /opt/uveddi/uveddi --version | awk '{print $2}')
if [ "$NEW_VERSION_ACTUAL" = "$NEW_VERSION" ]; then
    echo "Upgrade completed successfully: $CURRENT_VERSION -> $NEW_VERSION"
else
    echo "ERROR: Version mismatch after upgrade"
    exit 1
fi

# 10. Cleanup
rm -f uveddi-linux-x86_64.tar.gz uveddi

echo "Upgrade completed successfully!"
```

### Rollback Procedures

```bash
#!/bin/bash
# rollback-production.sh

set -euo pipefail

BACKUP_TIMESTAMP=${1:?"Usage: $0 <backup_timestamp>"}

echo "Rolling back to backup: $BACKUP_TIMESTAMP"

# 1. Stop services
sudo systemctl stop uveddi

# 2. Restore from backup
/opt/uveddi/scripts/disaster-recovery.sh $BACKUP_TIMESTAMP

# 3. Verify rollback
echo "Verifying rollback..."
/opt/uveddi/scripts/health-check.sh

echo "Rollback completed successfully!"
```

---

## Production Readiness Summary

### ✅ Ready for Production

- **Core Analysis Features**: Static analysis, anti-pattern detection, reporting
- **CLI Interface**: Full command-line functionality
- **Database Integration**: PostgreSQL with connection pooling
- **Basic API Server**: REST endpoints for integration
- **Monitoring**: Prometheus metrics and health checks
- **Security**: Basic authentication, HTTPS support
- **Performance**: Multi-threaded analysis, caching

### ⚠️ Use with Caution

- **Web Dashboard**: Functional but not fully stable
- **Plugin System**: Working but limited ecosystem
- **TypeScript Analysis**: Basic support only
- **Auto-scaling**: Manual configuration required

### ❌ Not Production-Ready

- **Enterprise Features**: Multi-tenancy, advanced RBAC
- **Advanced Web UI**: Complex interactive features
- **Real-time Features**: WebSocket-based functionality

### Recommended Next Steps

1. **Start with CLI/API integration** in your CI/CD pipeline
2. **Use HTML/JSON reports** rather than interactive dashboard
3. **Implement monitoring and alerting** before production load
4. **Test with your actual codebase** before going live
5. **Have rollback plan ready** for initial deployment
6. **Plan for gradual feature adoption** as stability improves

---

*This guide reflects Uveddi v1.0-alpha production capabilities as of September 2025. Always test thoroughly with your specific environment and requirements.*