# Uveddi Deployment Guide

## Overview

This guide covers deployment strategies for Uveddi across different environments, from local development to production cloud deployments. It includes configuration management, security considerations, and monitoring setup.

## Deployment Environments

### Development Environment

**Purpose**: Local development and testing

**Setup**:
```bash
# Clone and setup
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
./scripts/setup-dev-environment.sh

# Run locally
cargo run -- analyze ./test-project
```

**Configuration**:
```bash
# .env.local
RUST_LOG=debug
RUST_BACKTRACE=1
OLLAMA_API_URL=http://localhost:11434
```

### Staging Environment

**Purpose**: Pre-production testing and validation

**Infrastructure**:
- Docker containers for consistent environment
- PostgreSQL database for backend
- CI/CD integration for automated deployment

**Setup**:
```bash
# Using Docker Compose
cd backend
./docker-compose.sh up

# Or manual deployment
make setup migrate run
```

### Production Environment

**Purpose**: Live system serving end users

**Infrastructure Options**:
1. **Cloud Native** (Recommended)
   - Google Cloud Run + Cloud SQL
   - AWS ECS + RDS
   - Azure Container Instances + Azure Database

2. **Self-Hosted**
   - Kubernetes cluster
   - Docker Swarm
   - Traditional VMs

## Cloud Deployment

### Google Cloud Platform

#### Prerequisites

```bash
# Install Google Cloud SDK
curl https://sdk.cloud.google.com | bash
exec -l $SHELL

# Authenticate
gcloud auth login
gcloud config set project YOUR_PROJECT_ID
```

#### Database Setup

```bash
# Create Cloud SQL instance
gcloud sql instances create uveddi-db \
    --database-version=POSTGRES_15 \
    --tier=db-f1-micro \
    --region=us-central1

# Create database
gcloud sql databases create uveddi --instance=uveddi-db

# Create user
gcloud sql users create uveddi-user \
    --instance=uveddi-db \
    --password=SECURE_PASSWORD
```

#### Backend Deployment

```bash
# Build and push container
cd backend
gcloud builds submit --tag gcr.io/YOUR_PROJECT_ID/uveddi-backend

# Deploy to Cloud Run
gcloud run deploy uveddi-backend \
    --image gcr.io/YOUR_PROJECT_ID/uveddi-backend \
    --platform managed \
    --region us-central1 \
    --add-cloudsql-instances YOUR_PROJECT_ID:us-central1:uveddi-db \
    --set-env-vars "DB_POSTGRES_HOST=127.0.0.1,DB_POSTGRES_DB=uveddi,DB_POSTGRES_USER=uveddi-user" \
    --set-secrets="DB_POSTGRES_PASSWORD=db-password:latest" \
    --allow-unauthenticated \
    --memory 2Gi \
    --cpu 2 \
    --max-instances 10
```

#### CLI Distribution

```bash
# Build release binaries
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin

# Upload to Cloud Storage
gsutil cp target/release/uveddi gs://your-bucket/releases/v0.1.0/uveddi-linux
gsutil cp target/release/uveddi.exe gs://your-bucket/releases/v0.1.0/uveddi-windows.exe
gsutil cp target/release/uveddi gs://your-bucket/releases/v0.1.0/uveddi-macos
```

### AWS Deployment

#### Infrastructure as Code (Terraform)

```hcl
# infrastructure/aws/main.tf
provider "aws" {
  region = var.aws_region
}

# ECS Cluster
resource "aws_ecs_cluster" "uveddi" {
  name = "uveddi-cluster"
  
  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

# RDS Database
resource "aws_db_instance" "uveddi" {
  identifier = "uveddi-db"
  engine     = "postgres"
  engine_version = "15.4"
  instance_class = "db.t3.micro"
  allocated_storage = 20
  
  db_name  = "uveddi"
  username = "uveddi_user"
  password = var.db_password
  
  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = aws_db_subnet_group.uveddi.name
  
  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  skip_final_snapshot = true
}

# ECS Service
resource "aws_ecs_service" "uveddi_backend" {
  name            = "uveddi-backend"
  cluster         = aws_ecs_cluster.uveddi.id
  task_definition = aws_ecs_task_definition.uveddi_backend.arn
  desired_count   = 2
  
  load_balancer {
    target_group_arn = aws_lb_target_group.uveddi.arn
    container_name   = "uveddi-backend"
    container_port   = 8000
  }
}
```

#### Deployment Commands

```bash
# Initialize Terraform
cd infrastructure/aws
terraform init

# Plan deployment
terraform plan -var="db_password=SECURE_PASSWORD"

# Deploy infrastructure
terraform apply

# Deploy application
aws ecs update-service --cluster uveddi-cluster --service uveddi-backend --force-new-deployment
```

### Kubernetes Deployment

#### Namespace and ConfigMap

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: uveddi

---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: uveddi-config
  namespace: uveddi
data:
  DB_POSTGRES_HOST: "postgres-service"
  DB_POSTGRES_DB: "uveddi"
  DB_POSTGRES_USER: "uveddi_user"
  RUST_LOG: "info"
```

#### Database Deployment

```yaml
# k8s/postgres.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: uveddi
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15
        env:
        - name: POSTGRES_DB
          value: "uveddi"
        - name: POSTGRES_USER
          value: "uveddi_user"
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc

---
apiVersion: v1
kind: Service
metadata:
  name: postgres-service
  namespace: uveddi
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

#### Backend Deployment

```yaml
# k8s/backend.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uveddi-backend
  namespace: uveddi
spec:
  replicas: 3
  selector:
    matchLabels:
      app: uveddi-backend
  template:
    metadata:
      labels:
        app: uveddi-backend
    spec:
      containers:
      - name: uveddi-backend
        image: your-registry/uveddi-backend:latest
        ports:
        - containerPort: 8000
        envFrom:
        - configMapRef:
            name: uveddi-config
        env:
        - name: DB_POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: uveddi-backend-service
  namespace: uveddi
spec:
  selector:
    app: uveddi-backend
  ports:
  - port: 80
    targetPort: 8000
  type: LoadBalancer
```

#### Deployment Commands

```bash
# Apply configurations
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/secrets.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/backend.yaml

# Check deployment status
kubectl get pods -n uveddi
kubectl get services -n uveddi

# View logs
kubectl logs -f deployment/uveddi-backend -n uveddi
```

## Configuration Management

### Environment Variables

**Core Configuration**:
```bash
# Database
DB_POSTGRES_HOST=localhost
DB_POSTGRES_PORT=5432
DB_POSTGRES_DB=uveddi
DB_POSTGRES_USER=uveddi_user
DB_POSTGRES_PASSWORD=secure_password

# AI Providers
OPENAI_API_KEY=sk-your-key-here
ANTHROPIC_API_KEY=sk-ant-your-key-here
OLLAMA_API_URL=http://localhost:11434

# Application
RUST_LOG=info
RUST_BACKTRACE=1
UVEDDI_CONFIG_PATH=/etc/uveddi/config.toml

# Security
JWT_SECRET=your-jwt-secret
ENCRYPTION_KEY=your-encryption-key
```

**Environment-Specific Overrides**:
```bash
# Development
RUST_LOG=debug
DB_POSTGRES_HOST=localhost

# Staging
RUST_LOG=info
DB_POSTGRES_HOST=staging-db.internal

# Production
RUST_LOG=warn
DB_POSTGRES_HOST=prod-db.internal
ENABLE_METRICS=true
ENABLE_TRACING=true
```

### Configuration Files

**Main Configuration** (`config.toml`):
```toml
[database]
host = "localhost"
port = 5432
database = "uveddi"
user = "uveddi_user"
max_connections = 10
connection_timeout = 30

[ai]
default_provider = "openai"
timeout = 30
max_retries = 3

[ai.providers.openai]
model = "gpt-4"
max_tokens = 2000

[ai.providers.anthropic]
model = "claude-3-opus-20240229"
max_tokens = 2000

[ai.providers.ollama]
url = "http://localhost:11434"
model = "deepseek-coder:6.7b-instruct-q4_0"

[analysis]
max_file_size = 1048576  # 1MB
timeout = 300  # 5 minutes
parallel_jobs = 4

[security]
enable_audit_log = true
max_request_size = 10485760  # 10MB
rate_limit_requests_per_minute = 100
```

### Secrets Management

#### Google Cloud Secret Manager

```bash
# Store secrets
gcloud secrets create db-password --data-file=password.txt
gcloud secrets create openai-api-key --data-file=openai-key.txt

# Grant access
gcloud secrets add-iam-policy-binding db-password \
    --member="serviceAccount:uveddi-backend@PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/secretmanager.secretAccessor"
```

#### AWS Secrets Manager

```bash
# Store secrets
aws secretsmanager create-secret \
    --name "uveddi/db-password" \
    --secret-string "your-secure-password"

aws secretsmanager create-secret \
    --name "uveddi/openai-api-key" \
    --secret-string "sk-your-openai-key"
```

#### Kubernetes Secrets

```yaml
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: postgres-secret
  namespace: uveddi
type: Opaque
data:
  password: <base64-encoded-password>

---
apiVersion: v1
kind: Secret
metadata:
  name: ai-secrets
  namespace: uveddi
type: Opaque
data:
  openai-api-key: <base64-encoded-key>
  anthropic-api-key: <base64-encoded-key>
```

## Security Considerations

### Network Security

**Firewall Rules**:
```bash
# Allow only necessary ports
# HTTP/HTTPS for web traffic
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Database access (internal only)
iptables -A INPUT -p tcp --dport 5432 -s 10.0.0.0/8 -j ACCEPT

# SSH for administration
iptables -A INPUT -p tcp --dport 22 -s ADMIN_IP -j ACCEPT

# Drop all other traffic
iptables -A INPUT -j DROP
```

**TLS Configuration**:
```nginx
# nginx.conf
server {
    listen 443 ssl http2;
    server_name api.uveddi.com;
    
    ssl_certificate /etc/ssl/certs/uveddi.crt;
    ssl_certificate_key /etc/ssl/private/uveddi.key;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
    
    location / {
        proxy_pass http://uveddi-backend:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Application Security

**Input Validation**:
```rust
// Validate file uploads
pub fn validate_upload(file: &UploadedFile) -> Result<(), ValidationError> {
    // Check file size
    if file.size > MAX_FILE_SIZE {
        return Err(ValidationError::FileTooLarge);
    }
    
    // Check file type
    let allowed_extensions = ["rs", "py", "js", "ts"];
    let extension = file.extension().ok_or(ValidationError::InvalidFileType)?;
    if !allowed_extensions.contains(&extension) {
        return Err(ValidationError::InvalidFileType);
    }
    
    // Scan for malicious content
    if contains_malicious_patterns(&file.content) {
        return Err(ValidationError::MaliciousContent);
    }
    
    Ok(())
}
```

**API Security**:
```rust
// Rate limiting middleware
pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, StatusCode> {
    let client_ip = get_client_ip(&req);
    
    if is_rate_limited(&client_ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    let response = next.run(req).await;
    record_request(&client_ip).await;
    
    Ok(response)
}
```

### Data Protection

**Encryption at Rest**:
```bash
# Database encryption
ALTER DATABASE uveddi SET default_table_access_method = 'heap';
CREATE EXTENSION IF NOT EXISTS pgcrypto;

# File system encryption (Linux)
cryptsetup luksFormat /dev/sdb
cryptsetup luksOpen /dev/sdb uveddi-data
mkfs.ext4 /dev/mapper/uveddi-data
```

**Encryption in Transit**:
```rust
// TLS configuration for database connections
let config = tokio_postgres::Config::new()
    .host("database.internal")
    .port(5432)
    .user("uveddi_user")
    .password(&password)
    .dbname("uveddi")
    .ssl_mode(SslMode::Require);
```

## Monitoring and Observability

### Health Checks

**Application Health**:
```rust
// Health check endpoint
#[get("/health")]
pub async fn health_check(db: web::Data<Database>) -> impl Responder {
    let mut status = HealthStatus::new();
    
    // Check database connectivity
    match db.ping().await {
        Ok(_) => status.add_check("database", "healthy"),
        Err(e) => status.add_check("database", &format!("unhealthy: {}", e)),
    }
    
    // Check AI providers
    for provider in &["openai", "anthropic", "ollama"] {
        match check_ai_provider(provider).await {
            Ok(_) => status.add_check(provider, "healthy"),
            Err(e) => status.add_check(provider, &format!("degraded: {}", e)),
        }
    }
    
    HttpResponse::Ok().json(status)
}
```

**Infrastructure Health**:
```bash
# System health script
#!/bin/bash
check_disk_space() {
    usage=$(df / | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ $usage -gt 80 ]; then
        echo "WARNING: Disk usage at ${usage}%"
    fi
}

check_memory() {
    available=$(free | awk 'NR==2{printf "%.2f", $7/$2*100}')
    if (( $(echo "$available < 20" | bc -l) )); then
        echo "WARNING: Low memory: ${available}% available"
    fi
}

check_disk_space
check_memory
```

### Logging

**Structured Logging**:
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(db))]
pub async fn analyze_project(
    project_path: &Path,
    db: &Database,
) -> Result<AnalysisReport, AnalysisError> {
    info!(
        project_path = %project_path.display(),
        "Starting project analysis"
    );
    
    let start_time = Instant::now();
    
    match perform_analysis(project_path).await {
        Ok(report) => {
            info!(
                duration_ms = start_time.elapsed().as_millis(),
                issues_found = report.issues.len(),
                "Analysis completed successfully"
            );
            Ok(report)
        }
        Err(e) => {
            error!(
                error = %e,
                duration_ms = start_time.elapsed().as_millis(),
                "Analysis failed"
            );
            Err(e)
        }
    }
}
```

**Log Aggregation** (ELK Stack):
```yaml
# docker-compose.logging.yml
version: '3.8'
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.8.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    ports:
      - "9200:9200"
    
  logstash:
    image: docker.elastic.co/logstash/logstash:8.8.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf
    ports:
      - "5044:5044"
    
  kibana:
    image: docker.elastic.co/kibana/kibana:8.8.0
    ports:
      - "5601:5601"
    environment:
      - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
```

### Metrics and Alerting

**Prometheus Metrics**:
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref ANALYSIS_REQUESTS: Counter = register_counter!(
        "uveddi_analysis_requests_total",
        "Total number of analysis requests"
    ).unwrap();
    
    static ref ANALYSIS_DURATION: Histogram = register_histogram!(
        "uveddi_analysis_duration_seconds",
        "Duration of analysis requests"
    ).unwrap();
}

pub async fn analyze_with_metrics(project: &Path) -> Result<AnalysisReport, AnalysisError> {
    ANALYSIS_REQUESTS.inc();
    let timer = ANALYSIS_DURATION.start_timer();
    
    let result = analyze_project(project).await;
    timer.observe_duration();
    
    result
}
```

**Grafana Dashboard**:
```json
{
  "dashboard": {
    "title": "Uveddi Monitoring",
    "panels": [
      {
        "title": "Analysis Requests",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(uveddi_analysis_requests_total[5m])",
            "legendFormat": "Requests/sec"
          }
        ]
      },
      {
        "title": "Analysis Duration",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(uveddi_analysis_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      }
    ]
  }
}
```

## Backup and Disaster Recovery

### Database Backup

**Automated Backup Script**:
```bash
#!/bin/bash
# backup-database.sh

BACKUP_DIR="/backups/uveddi"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/uveddi_backup_$TIMESTAMP.sql"

# Create backup directory
mkdir -p $BACKUP_DIR

# Perform backup
pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME > $BACKUP_FILE

# Compress backup
gzip $BACKUP_FILE

# Upload to cloud storage
aws s3 cp $BACKUP_FILE.gz s3://uveddi-backups/database/

# Clean up old backups (keep last 30 days)
find $BACKUP_DIR -name "*.sql.gz" -mtime +30 -delete
```

**Backup Verification**:
```bash
#!/bin/bash
# verify-backup.sh

LATEST_BACKUP=$(ls -t /backups/uveddi/*.sql.gz | head -1)

# Test restore to temporary database
createdb uveddi_test
gunzip -c $LATEST_BACKUP | psql -d uveddi_test

# Verify data integrity
psql -d uveddi_test -c "SELECT COUNT(*) FROM analysis_runs;"

# Clean up
dropdb uveddi_test
```

### Application State Backup

**Configuration Backup**:
```bash
#!/bin/bash
# backup-config.sh

CONFIG_BACKUP_DIR="/backups/config"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p $CONFIG_BACKUP_DIR

# Backup configuration files
tar -czf $CONFIG_BACKUP_DIR/config_$TIMESTAMP.tar.gz \
    /etc/uveddi/ \
    /opt/uveddi/config/ \
    ~/.uveddi/

# Upload to cloud storage
aws s3 cp $CONFIG_BACKUP_DIR/config_$TIMESTAMP.tar.gz s3://uveddi-backups/config/
```

### Disaster Recovery Plan

**Recovery Procedures**:

1. **Database Recovery**:
   ```bash
   # Restore from latest backup
   LATEST_BACKUP=$(aws s3 ls s3://uveddi-backups/database/ | sort | tail -1 | awk '{print $4}')
   aws s3 cp s3://uveddi-backups/database/$LATEST_BACKUP ./
   gunzip $LATEST_BACKUP
   psql -d uveddi < ${LATEST_BACKUP%.gz}
   ```

2. **Application Recovery**:
   ```bash
   # Redeploy application
   kubectl apply -f k8s/
   kubectl rollout status deployment/uveddi-backend -n uveddi
   ```

3. **Configuration Recovery**:
   ```bash
   # Restore configuration
   LATEST_CONFIG=$(aws s3 ls s3://uveddi-backups/config/ | sort | tail -1 | awk '{print $4}')
   aws s3 cp s3://uveddi-backups/config/$LATEST_CONFIG ./
   tar -xzf $LATEST_CONFIG -C /
   ```

**Recovery Testing**:
```bash
#!/bin/bash
# test-disaster-recovery.sh

echo "Starting disaster recovery test..."

# Create test environment
kubectl create namespace uveddi-dr-test

# Deploy from backups
kubectl apply -f k8s/ -n uveddi-dr-test

# Restore database
# ... restoration commands ...

# Verify functionality
curl -f http://uveddi-dr-test/health || exit 1

echo "Disaster recovery test completed successfully"

# Cleanup
kubectl delete namespace uveddi-dr-test
```

---

*This deployment guide provides comprehensive coverage for deploying Uveddi in various environments. Adapt the configurations based on your specific infrastructure requirements and security policies.*