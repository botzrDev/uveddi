# Task Assignment: Deployment Automation and Infrastructure

## Priority: 🟡 HIGH PRIORITY - Operations Requirement

## Problem Statement
Current manual deployment process is error-prone and not suitable for production releases. The system needs automated deployment pipelines, infrastructure as code, and reliable rollback mechanisms for safe production operations.

## Objective
Implement comprehensive deployment automation with infrastructure as code, automated testing, and reliable rollback capabilities to ensure safe and consistent production deployments.

## Scope of Work

### Current Deployment Challenges:
- Manual build and deployment process
- No infrastructure as code
- No automated rollback mechanisms
- Missing deployment validation
- No blue-green or canary deployment options

### Deployment Automation Components:

#### 1. Docker Container Strategy
```dockerfile
# Dockerfile.production
FROM rust:1.70-alpine AS builder

# Install system dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    sqlite-dev

WORKDIR /app
COPY . .

# Build with production features
RUN cargo build --release --features production

# Runtime image
FROM alpine:3.18
RUN apk add --no-cache \
    ca-certificates \
    sqlite \
    openssl

WORKDIR /app
COPY --from=builder /app/target/release/uveddi /usr/local/bin/uveddi
COPY --from=builder /app/config /app/config

# Create non-root user
RUN addgroup -g 1001 -S uveddi && \
    adduser -S -D -H -u 1001 -h /app -s /sbin/nologin -G uveddi uveddi
USER uveddi

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD /usr/local/bin/uveddi health || exit 1

CMD ["uveddi", "serve", "--config", "/app/config/production.toml"]
```

```dockerfile
# Dockerfile.development
FROM rust:1.70

WORKDIR /app

# Install development dependencies
RUN apt-get update && apt-get install -y \
    nodejs \
    npm \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install development tools
RUN cargo install cargo-watch cargo-tarpaulin

COPY . .

# Build with development features
RUN cargo build --features dev-core

EXPOSE 8080 3000
CMD ["cargo", "watch", "-x", "run -- serve --development"]
```

#### 2. Kubernetes Deployment Manifests
```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uveddi-api
  labels:
    app: uveddi
    component: api
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
      component: api
  template:
    metadata:
      labels:
        app: uveddi
        component: api
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
    spec:
      serviceAccountName: uveddi-api
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        fsGroup: 1001
      containers:
      - name: api
        image: uveddi/api:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: uveddi-secrets
              key: database-url
        - name: RUST_LOG
          value: "info"
        - name: PROMETHEUS_ENDPOINT
          value: "0.0.0.0:9090"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: storage
          mountPath: /app/data
      volumes:
      - name: config
        configMap:
          name: uveddi-config
      - name: storage
        persistentVolumeClaim:
          claimName: uveddi-storage
```

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: uveddi-api-service
  labels:
    app: uveddi
    component: api
spec:
  type: ClusterIP
  ports:
  - port: 80
    targetPort: http
    protocol: TCP
    name: http
  - port: 9090
    targetPort: metrics
    protocol: TCP
    name: metrics
  selector:
    app: uveddi
    component: api
---
apiVersion: v1
kind: Service
metadata:
  name: uveddi-api-headless
  labels:
    app: uveddi
    component: api
spec:
  clusterIP: None
  ports:
  - port: 80
    targetPort: http
    protocol: TCP
  selector:
    app: uveddi
    component: api
```

#### 3. Helm Chart Structure
```yaml
# helm/uveddi/Chart.yaml
apiVersion: v2
name: uveddi
description: Architectural Analysis Tool
type: application
version: 0.9.0
appVersion: "0.9.0-alpha"

dependencies:
- name: postgresql
  version: "11.9.13"
  repository: "https://charts.bitnami.com/bitnami"
  condition: postgresql.enabled
- name: redis
  version: "17.3.7"
  repository: "https://charts.bitnami.com/bitnami"
  condition: redis.enabled
```

```yaml
# helm/uveddi/values.yaml
replicaCount: 3

image:
  repository: uveddi/api
  pullPolicy: IfNotPresent
  tag: ""

serviceAccount:
  create: true
  annotations: {}
  name: ""

podAnnotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "9090"

podSecurityContext:
  fsGroup: 1001
  runAsNonRoot: true
  runAsUser: 1001

securityContext:
  allowPrivilegeEscalation: false
  capabilities:
    drop:
    - ALL
  readOnlyRootFilesystem: true

service:
  type: ClusterIP
  port: 80

ingress:
  enabled: true
  className: "nginx"
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
  hosts:
  - host: uveddi.example.com
    paths:
    - path: /
      pathType: Prefix
  tls:
  - secretName: uveddi-tls
    hosts:
    - uveddi.example.com

resources:
  limits:
    cpu: 1000m
    memory: 2Gi
  requests:
    cpu: 250m
    memory: 512Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

persistence:
  enabled: true
  size: 10Gi
  storageClass: ""

postgresql:
  enabled: true
  auth:
    postgresPassword: ""
    database: uveddi
  primary:
    persistence:
      enabled: true
      size: 20Gi

redis:
  enabled: true
  auth:
    enabled: false
  master:
    persistence:
      enabled: true
      size: 8Gi
```

#### 4. CI/CD Pipeline (GitHub Actions)
```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'staging'
        type: choice
        options:
        - staging
        - production

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    outputs:
      image: ${{ steps.image.outputs.image }}
      digest: ${{ steps.build.outputs.digest }}
    steps:
    - name: Checkout
      uses: actions/checkout@v4
      
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
      
    - name: Log in to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
        
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v5
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        
    - name: Build and push Docker image
      id: build
      uses: docker/build-push-action@v5
      with:
        context: .
        file: ./Dockerfile.production
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max
        
    - name: Output image
      id: image
      run: |
        echo "image=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}@${{ steps.build.outputs.digest }}" >> $GITHUB_OUTPUT

  security-scan:
    needs: build
    runs-on: ubuntu-latest
    steps:
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        image-ref: ${{ needs.build.outputs.image }}
        format: 'sarif'
        output: 'trivy-results.sarif'
        
    - name: Upload Trivy scan results
      uses: github/codeql-action/upload-sarif@v2
      if: always()
      with:
        sarif_file: 'trivy-results.sarif'

  deploy-staging:
    if: github.event_name == 'workflow_dispatch' && github.event.inputs.environment == 'staging'
    needs: [build, security-scan]
    runs-on: ubuntu-latest
    environment: staging
    steps:
    - name: Deploy to staging
      run: |
        echo "Deploying ${{ needs.build.outputs.image }} to staging"
        # Add actual deployment commands here
        
  deploy-production:
    if: startsWith(github.ref, 'refs/tags/v') || (github.event_name == 'workflow_dispatch' && github.event.inputs.environment == 'production')
    needs: [build, security-scan]
    runs-on: ubuntu-latest
    environment: production
    steps:
    - name: Deploy to production
      run: |
        echo "Deploying ${{ needs.build.outputs.image }} to production"
        # Add actual deployment commands here
```

#### 5. Terraform Infrastructure
```hcl
# terraform/main.tf
terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
  }
  
  backend "s3" {
    bucket = "uveddi-terraform-state"
    key    = "production/terraform.tfstate"
    region = "us-west-2"
  }
}

provider "aws" {
  region = var.aws_region
}

# EKS Cluster
module "eks" {
  source = "terraform-aws-modules/eks/aws"
  version = "~> 19.0"
  
  cluster_name    = "uveddi-${var.environment}"
  cluster_version = "1.27"
  
  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets
  
  eks_managed_node_groups = {
    uveddi_nodes = {
      desired_size = 3
      max_size     = 10
      min_size     = 3
      
      instance_types = ["t3.medium"]
      capacity_type  = "ON_DEMAND"
      
      labels = {
        Environment = var.environment
        Application = "uveddi"
      }
      
      taints = {
        dedicated = {
          key    = "CriticalAddonsOnly"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      }
    }
  }
  
  tags = {
    Environment = var.environment
    Terraform   = "true"
    Application = "uveddi"
  }
}

# RDS Database
resource "aws_db_instance" "postgresql" {
  identifier = "uveddi-${var.environment}-db"
  
  engine         = "postgres"
  engine_version = "15.3"
  instance_class = "db.t3.micro"
  
  allocated_storage     = 20
  max_allocated_storage = 100
  storage_encrypted     = true
  
  db_name  = "uveddi"
  username = var.db_username
  password = var.db_password
  
  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = aws_db_subnet_group.default.name
  
  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  skip_final_snapshot = var.environment != "production"
  
  tags = {
    Name        = "uveddi-${var.environment}-db"
    Environment = var.environment
  }
}

# Redis Cache
resource "aws_elasticache_subnet_group" "redis" {
  name       = "uveddi-${var.environment}-cache-subnet"
  subnet_ids = module.vpc.private_subnets
}

resource "aws_elasticache_replication_group" "redis" {
  replication_group_id         = "uveddi-${var.environment}-cache"
  description                  = "Redis cache for Uveddi ${var.environment}"
  
  node_type                    = "cache.t3.micro"
  port                         = 6379
  parameter_group_name         = "default.redis7"
  
  num_cache_clusters           = 2
  automatic_failover_enabled   = true
  multi_az_enabled            = true
  
  subnet_group_name           = aws_elasticache_subnet_group.redis.name
  security_group_ids          = [aws_security_group.redis.id]
  
  at_rest_encryption_enabled  = true
  transit_encryption_enabled  = true
  
  tags = {
    Name        = "uveddi-${var.environment}-cache"
    Environment = var.environment
  }
}
```

#### 6. Deployment Scripts
```bash
#!/bin/bash
# scripts/deploy.sh

set -euo pipefail

ENVIRONMENT=${1:-staging}
IMAGE_TAG=${2:-latest}
DRY_RUN=${3:-false}

echo "Deploying Uveddi to $ENVIRONMENT with image tag $IMAGE_TAG"

# Pre-deployment checks
echo "Running pre-deployment checks..."
./scripts/pre-deployment-checks.sh $ENVIRONMENT

# Database migration
echo "Running database migrations..."
if [[ $DRY_RUN == "false" ]]; then
    kubectl apply -f k8s/migrations/job.yaml
    kubectl wait --for=condition=complete job/uveddi-migration --timeout=300s
fi

# Deploy application
echo "Deploying application..."
helm upgrade --install uveddi ./helm/uveddi \
    --namespace uveddi \
    --create-namespace \
    --set image.tag=$IMAGE_TAG \
    --set environment=$ENVIRONMENT \
    --values ./helm/uveddi/values-$ENVIRONMENT.yaml \
    ${DRY_RUN:+--dry-run}

# Post-deployment validation
echo "Running post-deployment validation..."
./scripts/post-deployment-checks.sh $ENVIRONMENT

echo "Deployment completed successfully!"
```

```bash
#!/bin/bash
# scripts/rollback.sh

set -euo pipefail

ENVIRONMENT=${1:-staging}
REVISION=${2:-0}  # 0 means previous revision

echo "Rolling back Uveddi in $ENVIRONMENT to revision $REVISION"

# Get current revision info
CURRENT_REVISION=$(helm history uveddi -n uveddi --max 1 -o json | jq -r '.[0].revision')
echo "Current revision: $CURRENT_REVISION"

if [[ $REVISION -eq 0 ]]; then
    REVISION=$((CURRENT_REVISION - 1))
fi

echo "Rolling back to revision: $REVISION"

# Perform rollback
helm rollback uveddi $REVISION -n uveddi

# Validate rollback
echo "Validating rollback..."
./scripts/post-deployment-checks.sh $ENVIRONMENT

echo "Rollback completed successfully!"
```

## Expected Outcome
- Automated deployment pipeline with zero-downtime deployments
- Infrastructure as code for reproducible environments
- Reliable rollback mechanisms with validation
- Container-based deployment with security scanning
- Comprehensive monitoring and alerting during deployments
- Blue-green deployment capability for production

## Time Estimate: 2-3 weeks

## Dependencies:
- Kubernetes cluster or cloud platform setup
- Container registry access
- Terraform/cloud provider credentials
- CI/CD platform configuration

## Testing Required:
```bash
# Test container builds
docker build -f Dockerfile.production -t uveddi:test .

# Test Kubernetes manifests
kubectl apply --dry-run=client -f k8s/

# Test Helm chart
helm template ./helm/uveddi --values ./helm/uveddi/values-staging.yaml

# Test deployment pipeline
./scripts/test-deployment-pipeline.sh

# Test rollback procedures
./scripts/test-rollback.sh
```

## Implementation Phases:

### Week 1: Containerization and Basic Deployment
- [ ] Create production Docker images
- [ ] Implement Kubernetes manifests
- [ ] Create basic deployment scripts
- [ ] Set up container registry

### Week 2: Infrastructure as Code
- [ ] Implement Terraform infrastructure
- [ ] Create Helm charts
- [ ] Set up CI/CD pipeline
- [ ] Configure environment-specific settings

### Week 3: Production Hardening
- [ ] Implement security scanning
- [ ] Add deployment validation
- [ ] Create rollback procedures
- [ ] Set up monitoring and alerting

## Success Metrics:
- [ ] Zero-downtime deployments achieved
- [ ] <5 minute deployment time from trigger to live
- [ ] <30 second rollback time with validation
- [ ] 100% deployment success rate in staging
- [ ] Comprehensive security scanning with no critical vulnerabilities