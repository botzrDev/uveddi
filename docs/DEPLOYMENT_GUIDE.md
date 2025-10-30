# Deployment Guide - Production Ready

## Overview

This guide provides comprehensive instructions for deploying Uveddi's high-performance analysis engine in production environments. The deployment supports horizontal scaling, comprehensive monitoring, and delivers 2-30x performance improvements through intelligent caching.

## Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 4 cores (8 threads recommended)
- **Memory**: 8GB RAM (16GB recommended)
- **Storage**: 100GB SSD with 1000+ IOPS
- **Network**: 1Gbps connection
- **OS**: Ubuntu 20.04+ or equivalent Linux distribution

#### Recommended Production Requirements
- **CPU**: 8+ cores (16+ threads)
- **Memory**: 32GB+ RAM
- **Storage**: NVMe SSD with 3000+ IOPS
- **Network**: 10Gbps connection
- **OS**: Ubuntu 22.04 LTS

#### Large-Scale Enterprise Requirements
- **CPU**: 16+ cores (32+ threads)
- **Memory**: 64GB+ RAM
- **Storage**: High-performance NVMe SSD array
- **Network**: 25Gbps+ connection
- **Load Balancer**: Application load balancer with WebSocket support

### Software Dependencies
- **Docker**: 20.10+
- **Kubernetes**: 1.21+ (for orchestrated deployment)
- **Prometheus**: 2.30+ (for monitoring)
- **Grafana**: 8.0+ (for dashboards)

## Docker Deployment

### Production Dockerfile

```dockerfile
# Multi-stage build for optimal security and size
FROM rust:1.70-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY src/dummy.rs src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm src/main.rs

# Copy source code
COPY src ./src
COPY templates ./templates
COPY assets ./assets

# Build application
RUN cargo build --release --features="production,analysis-cache,ast-cache"

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r uveddi && useradd -r -g uveddi uveddi

# Create directories
RUN mkdir -p /var/lib/uveddi/cache \
    && mkdir -p /etc/uveddi \
    && mkdir -p /var/log/uveddi \
    && chown -R uveddi:uveddi /var/lib/uveddi \
    && chown -R uveddi:uveddi /var/log/uveddi

# Copy binary and assets
COPY --from=builder /app/target/release/uveddi /usr/local/bin/
COPY --from=builder /app/templates /usr/local/share/uveddi/templates
COPY --from=builder /app/assets /usr/local/share/uveddi/assets

# Copy configuration template
COPY docker/production.toml /etc/uveddi/config.toml

# Set permissions
RUN chmod +x /usr/local/bin/uveddi

# Switch to app user
USER uveddi

# Expose ports
EXPOSE 3000 3001 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Set default command
CMD ["uveddi", "serve", "--config", "/etc/uveddi/config.toml"]
```

### Production Configuration

Create `docker/production.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3000
websocket_port = 3001
metrics_port = 9090
max_connections = 1000
request_timeout_secs = 300

[logging]
level = "info"
format = "json"
file = "/var/log/uveddi/uveddi.log"
max_file_size_mb = 100
max_files = 10

[database]
url = "postgresql://uveddi:${POSTGRES_PASSWORD}@postgres:5432/uveddi"
max_connections = 20
connection_timeout_secs = 30

[cache]
enabled = true
data_dir = "/var/lib/uveddi/cache"

[cache.ast]
enabled = true
max_entries = 50000
max_memory_mb = 8192
eviction_policy = "lru_with_ttl"
ttl_seconds = 3600

[cache.analysis]
enabled = true
max_entries = 25000
max_memory_mb = 16384
eviction_policy = "adaptive"
persist_on_shutdown = true

[cache.graph]
enabled = true
max_entries = 5000
max_memory_mb = 4096
query_cache_ttl_secs = 300

[cache.file_watcher]
enabled = true
poll_interval_secs = 2
watch_patterns = [
    "**/*.rs",
    "**/*.py", 
    "**/*.js",
    "**/*.ts",
    "**/*.tsx",
    "**/*.jsx"
]
debounce_ms = 1000

[cache.metrics]
enabled = true
collection_interval_secs = 30
detailed_logging = false
export_prometheus = true

[cache.memory]
max_total_memory_mb = 32768
enable_pressure_monitoring = true
pressure_threshold = 0.85
emergency_threshold = 0.95
```

### Docker Compose

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  uveddi:
    image: uveddi:latest
    container_name: uveddi-analysis-engine
    restart: unless-stopped
    ports:
      - "3000:3000"   # REST API
      - "3001:3001"   # WebSocket
      - "9090:9090"   # Metrics
    environment:
      - RUST_LOG=info
      - RUST_BACKTRACE=1
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    volumes:
      - ./config/production.toml:/etc/uveddi/config.toml:ro
      - cache_data:/var/lib/uveddi/cache
      - log_data:/var/log/uveddi
    depends_on:
      - postgres
      - redis
    networks:
      - uveddi_network
    deploy:
      resources:
        limits:
          memory: 32G
          cpus: '16.0'
        reservations:
          memory: 16G
          cpus: '8.0'
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  postgres:
    image: postgres:14-alpine
    container_name: uveddi-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_DB=uveddi
      - POSTGRES_USER=uveddi
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
      - POSTGRES_INITDB_ARGS=--auth-host=scram-sha-256
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    networks:
      - uveddi_network
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '2.0'

  redis:
    image: redis:7-alpine
    container_name: uveddi-redis
    restart: unless-stopped
    command: redis-server --appendonly yes --maxmemory 2gb --maxmemory-policy allkeys-lru
    volumes:
      - redis_data:/data
    networks:
      - uveddi_network
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '1.0'

  prometheus:
    image: prom/prometheus:latest
    container_name: uveddi-prometheus
    restart: unless-stopped
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'
    networks:
      - uveddi_network

  grafana:
    image: grafana/grafana:latest
    container_name: uveddi-grafana
    restart: unless-stopped
    ports:
      - "3002:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    networks:
      - uveddi_network

  nginx:
    image: nginx:alpine
    container_name: uveddi-nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
    depends_on:
      - uveddi
    networks:
      - uveddi_network

volumes:
  cache_data:
    driver: local
  log_data:
    driver: local
  postgres_data:
    driver: local
  redis_data:
    driver: local
  prometheus_data:
    driver: local
  grafana_data:
    driver: local

networks:
  uveddi_network:
    driver: bridge
```

### Deployment Commands

```bash
# Create environment file
cat > .env << 'EOF'
POSTGRES_PASSWORD=your_secure_password_here
GRAFANA_PASSWORD=your_grafana_password_here
EOF

# Build production image
docker build -t uveddi:latest -f deploy/Dockerfile.production .

# Deploy with docker-compose
docker-compose -f docker-compose.prod.yml up -d

# Verify deployment
docker-compose -f docker-compose.prod.yml ps
docker-compose -f docker-compose.prod.yml logs uveddi

# Check health
curl http://localhost:3000/health
curl http://localhost:3000/api/v1/cache/health
```

## Kubernetes Deployment

### Namespace and ConfigMap

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: uveddi
  labels:
    name: uveddi
    monitoring: enabled

---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: uveddi-config
  namespace: uveddi
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 3000
    websocket_port = 3001
    metrics_port = 9090
    max_connections = 2000
    
    [logging]
    level = "info"
    format = "json"
    
    [cache]
    enabled = true
    
    [cache.ast]
    enabled = true
    max_entries = 100000
    max_memory_mb = 16384
    eviction_policy = "lru_with_ttl"
    
    [cache.analysis]
    enabled = true
    max_entries = 50000
    max_memory_mb = 32768
    eviction_policy = "adaptive"
    
    [cache.memory]
    max_total_memory_mb = 65536
    enable_pressure_monitoring = true
```

### Deployment Manifest

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uveddi-analysis-engine
  namespace: uveddi
  labels:
    app: uveddi
    component: analysis-engine
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: uveddi
      component: analysis-engine
  template:
    metadata:
      labels:
        app: uveddi
        component: analysis-engine
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: uveddi
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - uveddi
              topologyKey: kubernetes.io/hostname
      containers:
      - name: uveddi
        image: uveddi:latest
        imagePullPolicy: Always
        ports:
        - name: http-api
          containerPort: 3000
          protocol: TCP
        - name: websocket
          containerPort: 3001
          protocol: TCP
        - name: metrics
          containerPort: 9090
          protocol: TCP
        env:
        - name: RUST_LOG
          value: "info"
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: uveddi-secrets
              key: postgres-password
        resources:
          requests:
            memory: "16Gi"
            cpu: "4000m"
            ephemeral-storage: "10Gi"
          limits:
            memory: "64Gi"
            cpu: "16000m"
            ephemeral-storage: "100Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: http-api
          initialDelaySeconds: 60
          periodSeconds: 30
          timeoutSeconds: 10
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: http-api
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 2
        startupProbe:
          httpGet:
            path: /health
            port: http-api
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 6
        volumeMounts:
        - name: cache-storage
          mountPath: /var/lib/uveddi/cache
        - name: config
          mountPath: /etc/uveddi
        - name: logs
          mountPath: /var/log/uveddi
      volumes:
      - name: cache-storage
        persistentVolumeClaim:
          claimName: uveddi-cache-pvc
      - name: config
        configMap:
          name: uveddi-config
      - name: logs
        emptyDir: {}
      terminationGracePeriodSeconds: 60
```

### Services and Ingress

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: uveddi-api-service
  namespace: uveddi
  labels:
    app: uveddi
    component: api
spec:
  type: ClusterIP
  ports:
  - name: http-api
    port: 3000
    targetPort: http-api
    protocol: TCP
  - name: websocket
    port: 3001
    targetPort: websocket
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: metrics
    protocol: TCP
  selector:
    app: uveddi
    component: analysis-engine

---
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: uveddi-ingress
  namespace: uveddi
  annotations:
    kubernetes.io/ingress.class: nginx
    nginx.ingress.kubernetes.io/proxy-read-timeout: "300"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "300"
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "300"
    nginx.ingress.kubernetes.io/websocket-services: "uveddi-api-service"
    nginx.ingress.kubernetes.io/upstream-hash-by: "$request_uri"
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.uveddi.example.com
    secretName: uveddi-tls
  rules:
  - host: api.uveddi.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: uveddi-api-service
            port:
              number: 3000
      - path: /ws
        pathType: Prefix
        backend:
          service:
            name: uveddi-api-service
            port:
              number: 3001
```

### Horizontal Pod Autoscaler

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: uveddi-hpa
  namespace: uveddi
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: uveddi-analysis-engine
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
  - type: Pods
    pods:
      metric:
        name: analysis_queue_size
      target:
        type: AverageValue
        averageValue: "10"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

### Persistent Storage

```yaml
# k8s/storage.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: uveddi-cache-pvc
  namespace: uveddi
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 500Gi

---
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/aws-ebs
parameters:
  type: gp3
  iops: "3000"
  throughput: "125"
  fsType: ext4
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### Deployment Commands

```bash
# Create namespace and secrets
kubectl apply -f k8s/namespace.yaml

kubectl create secret generic uveddi-secrets \
  --from-literal=postgres-password=your_secure_password \
  -n uveddi

# Deploy configuration
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/storage.yaml

# Deploy application
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/hpa.yaml
kubectl apply -f k8s/ingress.yaml

# Verify deployment
kubectl get pods -n uveddi
kubectl get services -n uveddi
kubectl get hpa -n uveddi

# Check logs
kubectl logs -f deployment/uveddi-analysis-engine -n uveddi

# Port forward for testing
kubectl port-forward service/uveddi-api-service 3000:3000 -n uveddi
```

## Monitoring and Observability

### Prometheus Configuration

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'uveddi-production'

rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'uveddi'
    static_configs:
      - targets: ['uveddi:9090']
    scrape_interval: 10s
    metrics_path: /metrics
    
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
```

### Alert Rules

```yaml
# monitoring/alert_rules.yml
groups:
  - name: uveddi.performance
    rules:
      - alert: CacheHitRateLow
        expr: cache_hit_rate < 0.7
        for: 5m
        labels:
          severity: warning
          service: uveddi
        annotations:
          summary: "Uveddi cache hit rate is low"
          description: "Cache hit rate is {{ $value | humanizePercentage }}, below 70% threshold"

      - alert: AnalysisSpeedupLow
        expr: analysis_speedup < 2
        for: 10m
        labels:
          severity: critical
          service: uveddi
        annotations:
          summary: "Analysis speedup below minimum threshold"
          description: "Current speedup is {{ $value }}x, below required 2x minimum"

      - alert: MemoryUsageHigh
        expr: cache_memory_bytes / cache_memory_limit_bytes > 0.9
        for: 2m
        labels:
          severity: warning
          service: uveddi
        annotations:
          summary: "Uveddi memory usage approaching limit"
          description: "Memory usage at {{ $value | humanizePercentage }} of limit"

      - alert: AnalysisQueueBacklog
        expr: analysis_queue_size > 50
        for: 5m
        labels:
          severity: warning
          service: uveddi
        annotations:
          summary: "Analysis queue backlog is high"
          description: "{{ $value }} analyses queued, consider scaling up"

  - name: uveddi.availability
    rules:
      - alert: UveddiDown
        expr: up{job="uveddi"} == 0
        for: 1m
        labels:
          severity: critical
          service: uveddi
        annotations:
          summary: "Uveddi service is down"
          description: "Uveddi has been down for more than 1 minute"

      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
          service: uveddi
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"
```

### Grafana Dashboards

Create comprehensive dashboards for monitoring:

#### Main Performance Dashboard
```json
{
  "dashboard": {
    "title": "Uveddi Performance Dashboard",
    "panels": [
      {
        "title": "Analysis Speedup",
        "type": "stat",
        "targets": [
          {
            "expr": "analysis_speedup",
            "legendFormat": "Speedup Factor"
          }
        ]
      },
      {
        "title": "Cache Hit Rates",
        "type": "graph",
        "targets": [
          {
            "expr": "cache_hit_rate{cache_type=\"ast\"}",
            "legendFormat": "AST Cache"
          },
          {
            "expr": "cache_hit_rate{cache_type=\"analysis\"}",
            "legendFormat": "Analysis Cache"
          },
          {
            "expr": "cache_hit_rate{cache_type=\"graph\"}",
            "legendFormat": "Graph Cache"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "cache_memory_bytes / 1024 / 1024",
            "legendFormat": "Cache Memory (MB)"
          }
        ]
      }
    ]
  }
}
```

### Health Checks

Implement comprehensive health checks:

```rust
// src/api/health.rs
use axum::{response::Json, http::StatusCode};
use serde_json::{json, Value};

pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    // Check cache service health
    let cache_healthy = check_cache_health().await;
    
    // Check database connectivity
    let db_healthy = check_database_health().await;
    
    // Check file watcher status
    let watcher_healthy = check_file_watcher_health().await;
    
    let overall_healthy = cache_healthy && db_healthy && watcher_healthy;
    
    let response = json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "cache": if cache_healthy { "healthy" } else { "unhealthy" },
            "database": if db_healthy { "healthy" } else { "unhealthy" },
            "file_watcher": if watcher_healthy { "healthy" } else { "unhealthy" }
        },
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": get_uptime_seconds(),
        "memory_usage_mb": get_memory_usage_mb()
    });
    
    if overall_healthy {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub async fn readiness_check() -> Result<Json<Value>, StatusCode> {
    // Check if service is ready to accept requests
    let cache_ready = is_cache_ready().await;
    let database_ready = is_database_ready().await;
    
    if cache_ready && database_ready {
        Ok(Json(json!({
            "status": "ready",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
```

## Security Configuration

### SSL/TLS Configuration

```nginx
# nginx/nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream uveddi_api {
        server uveddi:3000;
        keepalive 32;
    }
    
    upstream uveddi_ws {
        server uveddi:3001;
        keepalive 32;
    }

    server {
        listen 80;
        server_name api.uveddi.example.com;
        return 301 https://$server_name$request_uri;
    }

    server {
        listen 443 ssl http2;
        server_name api.uveddi.example.com;
        
        ssl_certificate /etc/nginx/ssl/uveddi.crt;
        ssl_certificate_key /etc/nginx/ssl/uveddi.key;
        
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
        ssl_prefer_server_ciphers off;
        ssl_session_cache shared:SSL:10m;
        
        location / {
            proxy_pass http://uveddi_api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 300s;
            proxy_send_timeout 300s;
            proxy_read_timeout 300s;
        }
        
        location /ws {
            proxy_pass http://uveddi_ws;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

### Authentication Configuration

```toml
# Add to production.toml
[security]
enable_authentication = true
jwt_secret = "${JWT_SECRET}"
jwt_expiry_hours = 24

[security.rate_limiting]
enable = true
requests_per_minute = 100
burst_size = 50

[security.cors]
allowed_origins = [
    "https://dashboard.uveddi.example.com",
    "https://app.uveddi.example.com"
]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Authorization", "Content-Type"]
```

## Scaling and Performance

### Horizontal Scaling

For high-load environments:

```yaml
# Scale deployment
kubectl scale deployment uveddi-analysis-engine --replicas=10 -n uveddi

# Update HPA for higher scale
kubectl patch hpa uveddi-hpa -n uveddi --patch '
spec:
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 60
'
```

### Cache Optimization for Scale

```toml
# High-scale cache configuration
[cache.memory]
max_total_memory_mb = 131072  # 128GB
enable_distributed_cache = true
redis_url = "redis://redis-cluster:6379"

[cache.distributed]
enable_cluster_mode = true
cluster_nodes = [
    "redis-node-1:6379",
    "redis-node-2:6379", 
    "redis-node-3:6379"
]
replication_factor = 2
```

### Performance Tuning

```bash
# System-level optimizations
echo 'vm.swappiness=1' >> /etc/sysctl.conf
echo 'net.core.somaxconn=65535' >> /etc/sysctl.conf
echo 'net.ipv4.ip_local_port_range=1024 65535' >> /etc/sysctl.conf
sysctl -p

# Container resource limits for high performance
docker run -d \
  --name uveddi \
  --memory=64g \
  --cpus=32 \
  --ulimit nofile=1048576:1048576 \
  --ulimit memlock=-1:-1 \
  uveddi:latest
```

## Backup and Disaster Recovery

### Cache Backup Strategy

```bash
#!/bin/bash
# scripts/backup_cache.sh

BACKUP_DIR="/backups/uveddi/cache"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR/$DATE"

# Backup cache data
kubectl exec deployment/uveddi-analysis-engine -n uveddi -- \
  tar czf - /var/lib/uveddi/cache | \
  cat > "$BACKUP_DIR/$DATE/cache_data.tar.gz"

# Backup cache statistics
curl -s http://localhost:3000/api/v1/cache/stats > \
  "$BACKUP_DIR/$DATE/cache_stats.json"

# Cleanup old backups (keep last 7 days)
find "$BACKUP_DIR" -type d -mtime +7 -exec rm -rf {} \;

echo "Cache backup completed: $BACKUP_DIR/$DATE"
```

### Disaster Recovery Plan

1. **Service Recovery**: Automatic restart via Kubernetes
2. **Data Recovery**: Restore from persistent volume snapshots
3. **Cache Rebuild**: Automatic cache warming from source
4. **Configuration Recovery**: GitOps-managed configurations

## Troubleshooting

### Common Issues

#### High Memory Usage
```bash
# Check cache statistics
curl http://localhost:3000/api/v1/cache/stats

# Enable aggressive eviction
kubectl patch configmap uveddi-config -n uveddi --patch '
data:
  config.toml: |
    [cache.memory]
    enable_aggressive_eviction = true
    pressure_threshold = 0.75
'

# Restart deployment to apply changes
kubectl rollout restart deployment/uveddi-analysis-engine -n uveddi
```

#### Cache Performance Issues
```bash
# Check file watcher status
curl http://localhost:3000/api/v1/cache/health

# Clear cache if needed
curl -X POST http://localhost:3000/api/v1/cache/control \
  -H "Content-Type: application/json" \
  -d '{"operation":"clear","target":"all","confirm":true}'

# Warm cache with common patterns
curl -X POST http://localhost:3000/api/v1/cache/warmup \
  -H "Content-Type: application/json" \
  -d '{
    "target":{"type":"path","path":"/common/codebase"},
    "strategy":"common_patterns"
  }'
```

#### Service Discovery Issues
```bash
# Check service endpoints
kubectl get endpoints uveddi-api-service -n uveddi

# Verify pod readiness
kubectl get pods -n uveddi -o wide

# Check logs for connection issues
kubectl logs deployment/uveddi-analysis-engine -n uveddi --tail=100
```

This deployment guide provides a comprehensive foundation for running Uveddi's high-performance analysis engine in production with enterprise-grade reliability, monitoring, and scaling capabilities.