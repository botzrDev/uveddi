# Uveddi Production Deployment Guide

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Installation Methods](#installation-methods)
4. [Configuration](#configuration)
5. [Database Setup](#database-setup)
6. [Service Deployment](#service-deployment)
7. [Monitoring and Observability](#monitoring-and-observability)
8. [Security Configuration](#security-configuration)
9. [Performance Optimization](#performance-optimization)
10. [Backup and Recovery](#backup-and-recovery)
11. [Scaling Strategies](#scaling-strategies)
12. [Maintenance Procedures](#maintenance-procedures)

## Prerequisites

### System Requirements

#### Minimum Requirements
```
CPU:     2 cores (x86_64 or ARM64)
Memory:  4GB RAM
Disk:    20GB available space
OS:      Linux (Ubuntu 20.04+, CentOS 8+, RHEL 8+)
Network: Internet access for dependencies
```

#### Recommended Requirements (Production)
```
CPU:     8+ cores (x86_64)
Memory:  16GB+ RAM (32GB for large codebases)
Disk:    100GB+ SSD storage
OS:      Ubuntu 22.04 LTS or RHEL 9
Network: Dedicated network interface, load balancer support
```

#### High-Performance Requirements (Enterprise)
```
CPU:     16+ cores (x86_64)
Memory:  64GB+ RAM
Disk:    500GB+ NVMe SSD (RAID 10 recommended)
OS:      Ubuntu 22.04 LTS
Network: 10Gbps network, clustering support
```

### Software Dependencies

#### Core Dependencies
```bash
# Rust toolchain (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update

# Node.js (18+) for rendering service
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Additional tools
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    postgresql-client \
    git \
    curl \
    wget \
    unzip
```

#### Optional Dependencies
```bash
# Docker for containerized deployment
sudo apt-get install -y docker.io docker-compose
sudo systemctl enable docker
sudo usermod -aG docker $USER

# Kubernetes tools (for cluster deployment)
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Monitoring tools
sudo apt-get install -y prometheus grafana
```

## Environment Setup

### Environment Variables

Create a comprehensive environment configuration:

```bash
# /etc/uveddi/environment
# Core Configuration
UVEDDI_ENV=production
UVEDDI_LOG_LEVEL=info
UVEDDI_LOG_FORMAT=json
UVEDDI_CONFIG_PATH=/etc/uveddi/config.toml

# Server Configuration
UVEDDI_HOST=0.0.0.0
UVEDDI_PORT=8080
UVEDDI_MAX_CONNECTIONS=1000
UVEDDI_REQUEST_TIMEOUT=30s
UVEDDI_SHUTDOWN_TIMEOUT=10s

# Database Configuration
UVEDDI_DATABASE_URL="postgresql://uveddi:secure_password@localhost:5432/uveddi_prod"
UVEDDI_DATABASE_POOL_SIZE=20
UVEDDI_DATABASE_MAX_LIFETIME=3600s
UVEDDI_DATABASE_CONNECT_TIMEOUT=10s

# Cache Configuration
UVEDDI_CACHE_PATH="/var/lib/uveddi/cache"
UVEDDI_CACHE_SIZE_MB=1024
UVEDDI_CACHE_TTL_HOURS=24

# Analysis Configuration
UVEDDI_MAX_CONCURRENT_ANALYSES=10
UVEDDI_DEFAULT_TIMEOUT_MS=300000
UVEDDI_ENABLE_AI_DEFAULT=true
UVEDDI_AI_PROVIDER=ollama

# Monitoring Configuration
UVEDDI_METRICS_PORT=9090
UVEDDI_WEBSOCKET_PORT=8080
UVEDDI_HEALTH_CHECK_INTERVAL=30s
UVEDDI_ALERT_WEBHOOK_URL="https://alerts.yourcompany.com/webhook"

# Security Configuration
UVEDDI_JWT_SECRET="your-256-bit-secret-here"
UVEDDI_JWT_EXPIRATION_HOURS=24
UVEDDI_REQUIRE_AUTHENTICATION=true
UVEDDI_ENABLE_RBAC=true
UVEDDI_RATE_LIMIT_REQUESTS_PER_MINUTE=100

# Performance Configuration
UVEDDI_WORKER_THREADS=8
UVEDDI_MEMORY_LIMIT_MB=8192
UVEDDI_ENABLE_PARALLEL_PROCESSING=true
UVEDDI_STREAMING_BUFFER_SIZE=8192

# External Services
UVEDDI_SMTP_HOST=smtp.yourcompany.com
UVEDDI_SMTP_PORT=587
UVEDDI_SMTP_USERNAME=uveddi-alerts@yourcompany.com
UVEDDI_SMTP_PASSWORD=secure_smtp_password
```

### System User Setup

Create a dedicated system user for security:

```bash
# Create uveddi user and group
sudo groupadd --system uveddi
sudo useradd --system --gid uveddi --shell /bin/false --home /var/lib/uveddi uveddi

# Create necessary directories
sudo mkdir -p /var/lib/uveddi/{cache,logs,data,plugins}
sudo mkdir -p /etc/uveddi
sudo mkdir -p /var/log/uveddi

# Set permissions
sudo chown -R uveddi:uveddi /var/lib/uveddi
sudo chown -R uveddi:uveddi /var/log/uveddi
sudo chmod 750 /var/lib/uveddi
sudo chmod 640 /etc/uveddi/environment
```

## Installation Methods

### Method 1: Binary Installation (Recommended)

```bash
# Download latest release
UVEDDI_VERSION="1.0.0"
wget "https://github.com/uveddi/uveddi/releases/download/v${UVEDDI_VERSION}/uveddi-v${UVEDDI_VERSION}-linux-x86_64.tar.gz"

# Extract and install
tar -xzf uveddi-v${UVEDDI_VERSION}-linux-x86_64.tar.gz
sudo mv uveddi /usr/local/bin/
sudo chmod +x /usr/local/bin/uveddi

# Install rendering service
sudo mkdir -p /opt/uveddi/rendering-service
sudo cp -r rendering-service/* /opt/uveddi/rendering-service/
cd /opt/uveddi/rendering-service
sudo npm install --production

# Verify installation
uveddi --version
```

### Method 2: Docker Deployment

#### Docker Compose Setup

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  uveddi-api:
    image: uveddi/uveddi:latest
    container_name: uveddi-api
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - UVEDDI_ENV=production
      - UVEDDI_DATABASE_URL=postgresql://uveddi:${DB_PASSWORD}@postgres:5432/uveddi
      - UVEDDI_CACHE_PATH=/data/cache
      - UVEDDI_LOG_LEVEL=info
    volumes:
      - uveddi-data:/data
      - uveddi-logs:/var/log/uveddi
      - ./config:/etc/uveddi:ro
    depends_on:
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  postgres:
    image: postgres:15
    container_name: uveddi-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_DB=uveddi
      - POSTGRES_USER=uveddi
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U uveddi"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: uveddi-redis
    restart: unless-stopped
    volumes:
      - redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 3

  rendering-service:
    image: uveddi/rendering-service:latest
    container_name: uveddi-rendering
    restart: unless-stopped
    ports:
      - "3001:3001"
    environment:
      - NODE_ENV=production
      - PORT=3001
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  nginx:
    image: nginx:alpine
    container_name: uveddi-nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl/private:ro
      - uveddi-logs:/var/log/nginx
    depends_on:
      - uveddi-api
      - rendering-service

volumes:
  uveddi-data:
  postgres-data:
  redis-data:
  uveddi-logs:
```

#### Deploy with Docker Compose

```bash
# Create environment file
cat > .env << EOF
DB_PASSWORD=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 64)
EOF

# Start services
docker-compose -f docker-compose.prod.yml up -d

# Verify deployment
docker-compose ps
docker-compose logs -f uveddi-api
```

### Method 3: Kubernetes Deployment

#### Kubernetes Manifests

```yaml
# k8s/namespace.yml
apiVersion: v1
kind: Namespace
metadata:
  name: uveddi-prod
  labels:
    name: uveddi-prod

---
# k8s/configmap.yml
apiVersion: v1
kind: ConfigMap
metadata:
  name: uveddi-config
  namespace: uveddi-prod
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    max_connections = 1000
    
    [database]
    url = "postgresql://uveddi:password@postgres:5432/uveddi"
    pool_size = 20
    
    [monitoring]
    metrics_port = 9090
    enable_prometheus = true
    
    [analysis]
    max_concurrent = 10
    default_timeout_ms = 300000

---
# k8s/secret.yml
apiVersion: v1
kind: Secret
metadata:
  name: uveddi-secrets
  namespace: uveddi-prod
type: Opaque
data:
  db-password: <base64-encoded-password>
  jwt-secret: <base64-encoded-jwt-secret>

---
# k8s/deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uveddi-api
  namespace: uveddi-prod
  labels:
    app: uveddi-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: uveddi-api
  template:
    metadata:
      labels:
        app: uveddi-api
    spec:
      containers:
      - name: uveddi-api
        image: uveddi/uveddi:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: UVEDDI_ENV
          value: "production"
        - name: UVEDDI_DATABASE_URL
          value: "postgresql://uveddi:$(DB_PASSWORD)@postgres:5432/uveddi"
        - name: UVEDDI_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: uveddi-secrets
              key: jwt-secret
        envFrom:
        - secretRef:
            name: uveddi-secrets
        volumeMounts:
        - name: config
          mountPath: /etc/uveddi
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
      volumes:
      - name: config
        configMap:
          name: uveddi-config
      - name: data
        persistentVolumeClaim:
          claimName: uveddi-data

---
# k8s/service.yml
apiVersion: v1
kind: Service
metadata:
  name: uveddi-api-service
  namespace: uveddi-prod
spec:
  selector:
    app: uveddi-api
  ports:
    - name: api
      port: 8080
      targetPort: 8080
    - name: metrics
      port: 9090
      targetPort: 9090
  type: ClusterIP

---
# k8s/ingress.yml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: uveddi-ingress
  namespace: uveddi-prod
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/websocket-services: "uveddi-api-service"
spec:
  tls:
  - hosts:
    - uveddi.yourcompany.com
    secretName: uveddi-tls
  rules:
  - host: uveddi.yourcompany.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: uveddi-api-service
            port:
              number: 8080
```

#### Deploy to Kubernetes

```bash
# Apply manifests
kubectl apply -f k8s/

# Verify deployment
kubectl get pods -n uveddi-prod
kubectl get services -n uveddi-prod
kubectl logs -f deployment/uveddi-api -n uveddi-prod
```

## Configuration

### Production Configuration File

```toml
# /etc/uveddi/config.toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 1000
request_timeout = "30s"
shutdown_timeout = "10s"

[database]
url = "postgresql://uveddi:password@localhost:5432/uveddi_prod"
pool_size = 20
max_lifetime = "3600s"
connection_timeout = "10s"
enable_migrations = false

[cache]
path = "/var/lib/uveddi/cache"
max_size_mb = 2048
ttl_hours = 24
compression_enabled = true

[analysis]
max_concurrent_analyses = 10
default_timeout_ms = 300000
enable_ai_default = true
ai_provider = "ollama"
parallel_processing = true
streaming_mode = true
memory_limit_mb = 8192

[monitoring]
metrics_port = 9090
enable_prometheus = true
websocket_port = 8080
health_check_interval = "30s"
retention_days = 30

[security]
jwt_secret = "your-secure-256-bit-secret"
jwt_expiration_hours = 24
require_authentication = true
enable_rbac = true
rate_limit_per_minute = 100
cors_origins = ["https://uveddi.yourcompany.com"]

[logging]
level = "info"
format = "json"
file = "/var/log/uveddi/uveddi.log"
max_size_mb = 100
max_files = 10

[performance]
worker_threads = 8
enable_jemalloc = true
gc_threshold_mb = 1024
batch_size = 50

[alerts]
webhook_url = "https://alerts.yourcompany.com/webhook"
smtp_host = "smtp.yourcompany.com"
smtp_port = 587
smtp_username = "uveddi-alerts@yourcompany.com"
email_recipients = ["ops-team@yourcompany.com"]

[plugins]
enable_wasm = true
plugin_dir = "/var/lib/uveddi/plugins"
max_execution_time_ms = 10000
memory_limit_mb = 256
```

### Environment-Specific Configurations

#### Development Environment
```toml
# config/development.toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite:/tmp/uveddi_dev.db"

[security]
require_authentication = false

[logging]
level = "debug"
format = "pretty"
```

#### Staging Environment
```toml
# config/staging.toml  
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://uveddi:password@staging-db:5432/uveddi_staging"

[security]
require_authentication = true
jwt_expiration_hours = 8

[analysis]
max_concurrent_analyses = 5

[logging]
level = "info"
format = "json"
```

## Database Setup

### PostgreSQL Installation and Configuration

#### Install PostgreSQL
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y postgresql postgresql-contrib

# CentOS/RHEL
sudo yum install -y postgresql-server postgresql-contrib
sudo postgresql-setup initdb
sudo systemctl enable postgresql
sudo systemctl start postgresql
```

#### Create Database and User
```bash
# Switch to postgres user
sudo -u postgres psql

-- Create database and user
CREATE DATABASE uveddi_prod;
CREATE USER uveddi WITH PASSWORD 'secure_password_here';
GRANT ALL PRIVILEGES ON DATABASE uveddi_prod TO uveddi;
ALTER USER uveddi CREATEDB;

-- Configure connection limits
ALTER USER uveddi CONNECTION LIMIT 50;

-- Exit psql
\q
```

#### Configure PostgreSQL

```bash
# Edit postgresql.conf
sudo vim /etc/postgresql/15/main/postgresql.conf

# Key settings for production:
listen_addresses = '*'
port = 5432
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100

# Edit pg_hba.conf for authentication
sudo vim /etc/postgresql/15/main/pg_hba.conf

# Add lines for Uveddi connection:
host    uveddi_prod    uveddi    10.0.0.0/8     md5
host    uveddi_prod    uveddi    172.16.0.0/12  md5
host    uveddi_prod    uveddi    192.168.0.0/16 md5

# Restart PostgreSQL
sudo systemctl restart postgresql
```

#### Run Database Migrations

```bash
# Using the Uveddi binary
uveddi migrate --config /etc/uveddi/config.toml

# Or using SQL files directly
sudo -u postgres psql -d uveddi_prod -f migrations/V1__initial_schema.sql
sudo -u postgres psql -d uveddi_prod -f migrations/V2__add_monitoring_tables.sql
sudo -u postgres psql -d uveddi_prod -f migrations/V3__add_security_tables.sql
```

### Database Backup Configuration

```bash
# Create backup script
cat > /usr/local/bin/backup-uveddi-db.sh << 'EOF'
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/var/backups/uveddi"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="uveddi_prod"
DB_USER="uveddi"

mkdir -p "$BACKUP_DIR"

# Create backup
pg_dump -h localhost -U "$DB_USER" -d "$DB_NAME" \
    --no-password --compress=9 \
    > "$BACKUP_DIR/uveddi_backup_$DATE.sql.gz"

# Keep only last 30 days of backups
find "$BACKUP_DIR" -name "uveddi_backup_*.sql.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_DIR/uveddi_backup_$DATE.sql.gz"
EOF

chmod +x /usr/local/bin/backup-uveddi-db.sh

# Add to cron
echo "0 2 * * * /usr/local/bin/backup-uveddi-db.sh" | sudo crontab -
```

## Service Deployment

### Systemd Service Configuration

```ini
# /etc/systemd/system/uveddi.service
[Unit]
Description=Uveddi Code Analysis Service
Documentation=https://docs.uveddi.com
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=exec
User=uveddi
Group=uveddi
ExecStart=/usr/local/bin/uveddi --config /etc/uveddi/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5
TimeoutStopSec=30

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/uveddi /var/log/uveddi /tmp
PrivateTmp=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096
LimitMEMLOCK=64M

# Environment
EnvironmentFile=/etc/uveddi/environment
WorkingDirectory=/var/lib/uveddi

[Install]
WantedBy=multi-user.target
```

### Rendering Service Configuration

```ini
# /etc/systemd/system/uveddi-rendering.service
[Unit]
Description=Uveddi Rendering Service
Documentation=https://docs.uveddi.com
After=network.target redis.service
Wants=redis.service

[Service]
Type=exec
User=uveddi
Group=uveddi
ExecStart=/usr/bin/node /opt/uveddi/rendering-service/server.js
Restart=on-failure
RestartSec=5
TimeoutStopSec=15

# Environment
Environment=NODE_ENV=production
Environment=PORT=3001
Environment=REDIS_URL=redis://localhost:6379
WorkingDirectory=/opt/uveddi/rendering-service

# Resource limits
LimitNOFILE=65536
LimitNPROC=2048

[Install]
WantedBy=multi-user.target
```

### Service Management

```bash
# Install and start services
sudo systemctl daemon-reload
sudo systemctl enable uveddi
sudo systemctl enable uveddi-rendering

sudo systemctl start uveddi
sudo systemctl start uveddi-rendering

# Check status
sudo systemctl status uveddi
sudo systemctl status uveddi-rendering

# View logs
sudo journalctl -u uveddi -f
sudo journalctl -u uveddi-rendering -f
```

## Monitoring and Observability

### Prometheus Configuration

```yaml
# /etc/prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "uveddi_alerts.yml"

scrape_configs:
  - job_name: 'uveddi-api'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
    metrics_path: /metrics
    
  - job_name: 'uveddi-system'
    static_configs:
      - targets: ['localhost:9100']
    
  - job_name: 'postgresql'
    static_configs:
      - targets: ['localhost:9187']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "id": null,
    "title": "Uveddi Production Monitoring",
    "panels": [
      {
        "title": "API Requests per Second",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(uveddi_http_requests_total[5m])",
            "legendFormat": "{{method}} {{status}}"
          }
        ]
      },
      {
        "title": "Analysis Queue Length",
        "type": "singlestat",
        "targets": [
          {
            "expr": "uveddi_analysis_queue_length",
            "legendFormat": "Queue Length"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "process_resident_memory_bytes",
            "legendFormat": "Memory Usage"
          }
        ]
      },
      {
        "title": "Active WebSocket Connections",
        "type": "graph",
        "targets": [
          {
            "expr": "uveddi_websocket_connections_active",
            "legendFormat": "Active Connections"
          }
        ]
      }
    ]
  }
}
```

### Log Management

#### Configure Log Rotation

```bash
# /etc/logrotate.d/uveddi
/var/log/uveddi/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    copytruncate
    postrotate
        systemctl reload uveddi > /dev/null 2>&1 || true
    endscript
}
```

#### ELK Stack Integration

```yaml
# filebeat.yml
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/uveddi/*.log
  fields:
    service: uveddi
    environment: production
  fields_under_root: true
  json.keys_under_root: true
  json.message_key: message

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "uveddi-logs-%{+yyyy.MM.dd}"

setup.template.name: "uveddi-logs"
setup.template.pattern: "uveddi-logs-*"
```

## Security Configuration

### SSL/TLS Configuration

```bash
# Generate SSL certificates (Let's Encrypt)
sudo apt-get install -y certbot
sudo certbot certonly --standalone -d uveddi.yourcompany.com

# Or use custom certificates
sudo mkdir -p /etc/ssl/private
sudo cp your-certificate.crt /etc/ssl/certs/uveddi.crt
sudo cp your-private-key.key /etc/ssl/private/uveddi.key
sudo chmod 600 /etc/ssl/private/uveddi.key
```

### Nginx Reverse Proxy

```nginx
# /etc/nginx/sites-available/uveddi
server {
    listen 80;
    server_name uveddi.yourcompany.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name uveddi.yourcompany.com;
    
    ssl_certificate /etc/ssl/certs/uveddi.crt;
    ssl_certificate_key /etc/ssl/private/uveddi.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload";
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Rate limiting
        limit_req zone=api burst=20 nodelay;
    }
    
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # API-specific settings
        proxy_read_timeout 300s;
        client_max_body_size 100M;
    }
    
    location /metrics {
        proxy_pass http://127.0.0.1:9090;
        allow 10.0.0.0/8;
        allow 172.16.0.0/12;
        allow 192.168.0.0/16;
        deny all;
    }
}
```

### Firewall Configuration

```bash
# Configure UFW firewall
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow internal monitoring (adjust IP ranges as needed)
sudo ufw allow from 10.0.0.0/8 to any port 9090
sudo ufw allow from 172.16.0.0/12 to any port 9090
sudo ufw allow from 192.168.0.0/16 to any port 9090

# Enable firewall
sudo ufw --force enable
sudo ufw status verbose
```

## Performance Optimization

### System-Level Optimizations

```bash
# Kernel parameters for high-performance systems
cat > /etc/sysctl.d/99-uveddi.conf << EOF
# Network optimizations
net.core.rmem_default = 262144
net.core.rmem_max = 16777216
net.core.wmem_default = 262144
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_congestion_control = bbr

# File system optimizations
fs.file-max = 1048576
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5

# Process limits
kernel.pid_max = 4194304
EOF

# Apply settings
sudo sysctl -p /etc/sysctl.d/99-uveddi.conf
```

### Application Tuning

```toml
# Performance-optimized config section
[performance]
# Use all available CPU cores
worker_threads = 0  # 0 = auto-detect

# Memory management
enable_jemalloc = true
gc_threshold_mb = 2048
arena_size_mb = 512

# Analysis optimizations
batch_size = 100
streaming_buffer_size = 16384
parallel_detector_execution = true

# Cache optimizations
cache_preload = true
cache_compression_level = 6
cache_write_batch_size = 1000

# Network optimizations
tcp_nodelay = true
keep_alive_timeout = 65
max_keep_alive_requests = 1000
```

### Database Performance Tuning

```sql
-- PostgreSQL performance tuning
-- Run as superuser

-- Adjust shared_buffers (25% of total RAM)
ALTER SYSTEM SET shared_buffers = '4GB';

-- Adjust effective_cache_size (75% of total RAM)  
ALTER SYSTEM SET effective_cache_size = '12GB';

-- Optimize for analysis workloads
ALTER SYSTEM SET work_mem = '256MB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';

-- Write-ahead log settings
ALTER SYSTEM SET wal_buffers = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;

-- Query planner settings
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;

-- Apply settings
SELECT pg_reload_conf();
```

## Backup and Recovery

### Automated Backup Strategy

```bash
# Comprehensive backup script
cat > /usr/local/bin/uveddi-backup.sh << 'EOF'
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/var/backups/uveddi"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

# Create backup directory
mkdir -p "$BACKUP_DIR"/{database,config,data,logs}

echo "Starting Uveddi backup - $DATE"

# Database backup
echo "Backing up database..."
pg_dump -h localhost -U uveddi -d uveddi_prod \
    --no-password --compress=9 \
    > "$BACKUP_DIR/database/db_backup_$DATE.sql.gz"

# Configuration backup
echo "Backing up configuration..."
tar -czf "$BACKUP_DIR/config/config_backup_$DATE.tar.gz" \
    /etc/uveddi /etc/systemd/system/uveddi*.service

# Data backup (cache and analysis results)
echo "Backing up data..."
tar -czf "$BACKUP_DIR/data/data_backup_$DATE.tar.gz" \
    /var/lib/uveddi --exclude="*.tmp" --exclude="*.lock"

# Log backup
echo "Backing up logs..."
tar -czf "$BACKUP_DIR/logs/logs_backup_$DATE.tar.gz" \
    /var/log/uveddi

# Cleanup old backups
find "$BACKUP_DIR" -name "*backup_*.gz" -mtime +$RETENTION_DAYS -delete
find "$BACKUP_DIR" -name "*backup_*.tar.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed successfully"

# Verify backups
echo "Verifying backups..."
for backup_file in "$BACKUP_DIR"/*/"*backup_$DATE*"; do
    if [ -f "$backup_file" ]; then
        echo "✓ $(basename "$backup_file") - $(stat -c%s "$backup_file" | numfmt --to=iec-i --suffix=B)"
    fi
done

# Upload to remote storage (optional)
if command -v aws &> /dev/null; then
    echo "Uploading to S3..."
    aws s3 sync "$BACKUP_DIR" s3://your-backup-bucket/uveddi/ --delete
fi

echo "Backup process completed - $DATE"
EOF

chmod +x /usr/local/bin/uveddi-backup.sh

# Schedule daily backups
echo "0 3 * * * /usr/local/bin/uveddi-backup.sh" | sudo crontab -
```

### Disaster Recovery Procedures

```bash
# Recovery script
cat > /usr/local/bin/uveddi-restore.sh << 'EOF'
#!/bin/bash
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <backup_date>"
    echo "Example: $0 20240721_030000"
    exit 1
fi

BACKUP_DATE="$1"
BACKUP_DIR="/var/backups/uveddi"

echo "Starting Uveddi recovery - $BACKUP_DATE"

# Stop services
echo "Stopping Uveddi services..."
systemctl stop uveddi uveddi-rendering

# Restore database
echo "Restoring database..."
dropdb uveddi_prod
createdb uveddi_prod
gunzip -c "$BACKUP_DIR/database/db_backup_$BACKUP_DATE.sql.gz" | \
    psql -h localhost -U uveddi -d uveddi_prod

# Restore configuration
echo "Restoring configuration..."
tar -xzf "$BACKUP_DIR/config/config_backup_$BACKUP_DATE.tar.gz" -C /

# Restore data
echo "Restoring data..."
rm -rf /var/lib/uveddi/*
tar -xzf "$BACKUP_DIR/data/data_backup_$BACKUP_DATE.tar.gz" -C /

# Set permissions
chown -R uveddi:uveddi /var/lib/uveddi

# Start services
echo "Starting Uveddi services..."
systemctl start uveddi uveddi-rendering

# Verify recovery
sleep 10
if systemctl is-active --quiet uveddi; then
    echo "✓ Uveddi API service is running"
else
    echo "✗ Uveddi API service failed to start"
fi

if systemctl is-active --quiet uveddi-rendering; then
    echo "✓ Rendering service is running"
else
    echo "✗ Rendering service failed to start"
fi

echo "Recovery completed - $BACKUP_DATE"
EOF

chmod +x /usr/local/bin/uveddi-restore.sh
```

## Scaling Strategies

### Horizontal Scaling (Load Balancer)

```bash
# HAProxy configuration
cat > /etc/haproxy/haproxy.cfg << EOF
global
    daemon
    maxconn 4096
    log stdout local0

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    option httplog
    
frontend uveddi_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/uveddi.pem
    redirect scheme https if !{ ssl_fc }
    default_backend uveddi_backend

backend uveddi_backend
    balance roundrobin
    option httpchk GET /api/v1/health
    server uveddi1 10.0.1.10:8080 check
    server uveddi2 10.0.1.11:8080 check
    server uveddi3 10.0.1.12:8080 check
EOF
```

### Vertical Scaling Configuration

```bash
# High-performance single-node setup
cat > /etc/uveddi/high-performance.toml << EOF
[server]
max_connections = 5000
worker_threads = 32

[analysis] 
max_concurrent_analyses = 50
memory_limit_mb = 32768

[cache]
max_size_mb = 16384

[performance]
enable_numa_awareness = true
cpu_affinity = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
memory_pool_size_mb = 8192
EOF
```

## Maintenance Procedures

### Regular Maintenance Tasks

```bash
# Weekly maintenance script
cat > /usr/local/bin/uveddi-maintenance.sh << 'EOF'
#!/bin/bash
set -euo pipefail

echo "Starting weekly Uveddi maintenance - $(date)"

# Update system packages
apt-get update && apt-get upgrade -y

# Clean up old logs
find /var/log/uveddi -name "*.log" -mtime +30 -delete
journalctl --vacuum-time=30d

# Clean up temporary files
find /tmp -name "uveddi-*" -mtime +7 -delete
find /var/lib/uveddi/cache -name "*.tmp" -delete

# Optimize database
sudo -u postgres psql -d uveddi_prod -c "VACUUM ANALYZE;"
sudo -u postgres psql -d uveddi_prod -c "REINDEX DATABASE uveddi_prod;"

# Update Docker images (if using Docker)
if command -v docker &> /dev/null; then
    docker pull uveddi/uveddi:latest
    docker pull uveddi/rendering-service:latest
fi

# Check service health
systemctl status uveddi
systemctl status uveddi-rendering

# Generate health report
curl -s http://localhost:8080/api/v1/health | jq

echo "Maintenance completed - $(date)"
EOF

chmod +x /usr/local/bin/uveddi-maintenance.sh

# Schedule weekly maintenance
echo "0 1 * * 0 /usr/local/bin/uveddi-maintenance.sh" | sudo crontab -
```

### Health Monitoring Script

```bash
cat > /usr/local/bin/uveddi-health-check.sh << 'EOF'
#!/bin/bash
set -euo pipefail

HEALTH_URL="http://localhost:8080/api/v1/health"
ALERT_EMAIL="ops-team@yourcompany.com"

# Check API health
if ! curl -f -s "$HEALTH_URL" > /dev/null; then
    echo "CRITICAL: Uveddi API is not responding" | \
        mail -s "Uveddi Health Alert" "$ALERT_EMAIL"
    exit 1
fi

# Check disk space
DISK_USAGE=$(df /var/lib/uveddi | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 85 ]; then
    echo "WARNING: Disk usage is at ${DISK_USAGE}%" | \
        mail -s "Uveddi Disk Space Alert" "$ALERT_EMAIL"
fi

# Check memory usage
MEMORY_USAGE=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')
if [ "$MEMORY_USAGE" -gt 90 ]; then
    echo "WARNING: Memory usage is at ${MEMORY_USAGE}%" | \
        mail -s "Uveddi Memory Alert" "$ALERT_EMAIL"
fi

echo "Health check passed - $(date)"
EOF

chmod +x /usr/local/bin/uveddi-health-check.sh

# Run health checks every 5 minutes
echo "*/5 * * * * /usr/local/bin/uveddi-health-check.sh" | crontab -
```

This comprehensive deployment guide provides all the necessary information and scripts for successfully deploying Uveddi in production environments, from basic installations to enterprise-grade high-availability setups with monitoring, security, and maintenance procedures.