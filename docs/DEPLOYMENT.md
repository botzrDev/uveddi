# Deployment Automation Guide

This guide covers the comprehensive deployment automation infrastructure for Uveddi, including containerization, Kubernetes deployment, CI/CD pipelines, infrastructure as code, and monitoring.

## Overview

The deployment automation provides:

- **Zero-downtime deployments** with automated rollbacks
- **Infrastructure as Code** using Terraform
- **Container-based deployment** with security hardening
- **Multi-environment support** (staging/production)
- **Comprehensive monitoring** and alerting
- **Automated testing** and validation
- **Security scanning** at every stage

## Quick Start

### Prerequisites

```bash
# Required tools
kubectl    # Kubernetes CLI
helm       # Helm package manager
docker     # Container runtime
terraform  # Infrastructure as Code
```

### Deploy to Staging

```bash
# Build and deploy to staging
./scripts/deployment/deploy.sh staging latest

# Run integration tests
./scripts/testing/integration-test.sh staging

# Check deployment status
kubectl get pods -n uveddi
```

### Deploy to Production

```bash
# Deploy tagged version to production
./scripts/deployment/deploy.sh production v1.2.3

# Monitor deployment
kubectl rollout status deployment/uveddi -n uveddi

# Validate with performance tests
./scripts/testing/performance-test.sh production
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CI/CD Pipeline│    │   Container     │    │   Kubernetes    │
│                 │    │   Registry      │    │   Cluster       │
│ GitHub Actions  ├───▶│   ghcr.io      ├───▶│   EKS/GKE      │
│ Security Scan   │    │   Multi-arch   │    │   Helm Charts  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Infrastructure│    │   Monitoring    │    │   Testing       │
│                 │    │                 │    │                 │
│ Terraform/AWS   │    │ Prometheus      │    │ Integration     │
│ Network/Storage │    │ Grafana/Alerts │    │ Performance     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Components

### 1. Container Images

#### Production Image (`Dockerfile.production`)
- **Multi-stage build** for minimal size
- **Alpine Linux** base for security
- **Non-root user** (UID 1001)
- **Security hardening** with minimal packages
- **Health checks** built-in

```bash
# Build production image
docker build -f Dockerfile.production -t uveddi:prod .

# Test image
docker run --rm -p 8080:8080 uveddi:prod
```

#### Development Image (`Dockerfile.development`)
- **Hot reload** with cargo watch
- **Debug tools** included
- **Development dependencies**

### 2. Kubernetes Manifests

Located in `k8s/base/`:

- **Namespace**: Isolated environment with security policies
- **ServiceAccount**: Minimal permissions with RBAC
- **ConfigMap**: Application configuration
- **Secret**: Sensitive data management
- **Deployment**: Application pods with security context
- **Service**: Internal load balancing
- **Ingress**: External access with TLS
- **HPA**: Auto-scaling based on CPU/memory
- **PDB**: Availability during maintenance
- **NetworkPolicy**: Network security

### 3. Helm Charts

Located in `helm/uveddi/`:

#### Chart Structure
```
helm/uveddi/
├── Chart.yaml              # Chart metadata
├── values.yaml             # Default values
├── values-staging.yaml     # Staging overrides
├── values-production.yaml  # Production overrides
└── templates/
    ├── deployment.yaml     # Main application
    ├── service.yaml        # Load balancer
    ├── ingress.yaml        # External access
    ├── configmap.yaml      # Configuration
    ├── secret.yaml         # Secrets
    └── monitoring/         # Prometheus/Grafana
```

#### Environment-Specific Deployments

**Staging**:
```bash
helm upgrade --install uveddi ./helm/uveddi \
  --namespace uveddi \
  --values ./helm/uveddi/values-staging.yaml \
  --set image.tag=latest
```

**Production**:
```bash
helm upgrade --install uveddi ./helm/uveddi \
  --namespace uveddi \
  --values ./helm/uveddi/values-production.yaml \
  --set image.tag=v1.2.3
```

### 4. CI/CD Pipeline

#### GitHub Actions Workflows

**Continuous Integration** (`.github/workflows/ci.yml`):
- ✅ Multi-version Rust testing
- ✅ Code formatting and linting
- ✅ Security auditing
- ✅ Container building
- ✅ Helm validation

**Deployment** (`.github/workflows/deploy.yml`):
- 🚀 Automated container builds
- 🔒 Security vulnerability scanning
- 🧪 Integration testing
- 📦 Multi-environment deployment
- 📊 Post-deployment validation

#### Pipeline Stages

1. **Build & Test**
   - Rust compilation with multiple feature sets
   - Unit and integration testing
   - Security audit with cargo-audit
   - Code coverage analysis

2. **Security Scanning**
   - Container vulnerability scanning (Trivy)
   - Code security analysis (Semgrep)
   - Dependency vulnerability checks

3. **Container Registry**
   - Multi-architecture builds (AMD64/ARM64)
   - Registry push to ghcr.io
   - Image signing and attestation

4. **Deployment**
   - Staging auto-deployment
   - Production manual approval
   - Rolling updates with health checks
   - Automated rollback on failure

### 5. Infrastructure as Code

#### Terraform Configuration

Located in `terraform/`:

```
terraform/
├── main.tf                 # Main infrastructure
├── variables.tf            # Input variables
├── outputs.tf              # Output values
├── environments/
│   ├── staging/           # Staging config
│   └── production/        # Production config
└── modules/
    ├── eks/               # Kubernetes cluster
    ├── rds/               # Database
    ├── redis/             # Cache
    ├── monitoring/        # CloudWatch/SNS
    └── networking/        # VPC/Subnets
```

#### Infrastructure Components

**Networking**:
- VPC with public/private subnets
- NAT Gateway for internet access
- Security Groups with least privilege
- VPC Endpoints for AWS services

**Kubernetes (EKS)**:
- Managed control plane
- Auto-scaling node groups
- IAM roles with IRSA
- Add-ons: ALB Controller, External DNS

**Database**:
- RDS PostgreSQL with encryption
- Multi-AZ for production
- Automated backups
- Performance Insights

**Monitoring**:
- CloudWatch log groups
- SNS topics for alerts
- IAM roles for metrics

#### Deploy Infrastructure

```bash
# Initialize Terraform
cd terraform/environments/production
terraform init

# Plan infrastructure
terraform plan -var-file="production.tfvars"

# Apply changes
terraform apply -var-file="production.tfvars"

# Configure kubectl
aws eks update-kubeconfig --region us-west-2 --name uveddi-production
```

### 6. Deployment Scripts

#### Deployment Script (`scripts/deployment/deploy.sh`)

Features:
- **Comprehensive validation** of prerequisites
- **Pre-deployment checks** and health validation
- **Database migration** execution
- **Zero-downtime deployment** with Helm
- **Post-deployment testing** and validation
- **Detailed logging** and error reporting

```bash
# Basic deployment
./scripts/deployment/deploy.sh staging latest

# Production deployment with validation
./scripts/deployment/deploy.sh production v1.2.3

# Dry-run deployment
./scripts/deployment/deploy.sh staging latest true

# Skip tests (for troubleshooting)
SKIP_TESTS=true ./scripts/deployment/deploy.sh staging

# Force deployment (skip health checks)
FORCE_DEPLOY=true ./scripts/deployment/deploy.sh staging
```

#### Rollback Script (`scripts/deployment/rollback.sh`)

Features:
- **Automated rollback** to previous or specific revision
- **Pre-rollback backup** of current state
- **Health validation** before and after rollback
- **Confirmation prompts** for production
- **Rollback validation** with smoke tests

```bash
# Rollback to previous revision
./scripts/deployment/rollback.sh staging

# Rollback to specific revision
./scripts/deployment/rollback.sh production 5

# Dry-run rollback
./scripts/deployment/rollback.sh staging 0 true

# Skip confirmation (automation)
CONFIRM=false ./scripts/deployment/rollback.sh staging
```

### 7. Testing Framework

#### Integration Testing (`scripts/testing/integration-test.sh`)

Tests:
- ✅ Health and readiness endpoints
- ✅ API endpoint availability
- ✅ Metrics endpoint functionality
- ✅ Database connectivity
- ✅ Resource usage validation
- ✅ Pod logs analysis
- ✅ Service endpoint configuration
- ✅ Basic load handling

```bash
# Run integration tests
./scripts/testing/integration-test.sh staging

# Verbose testing
VERBOSE=true ./scripts/testing/integration-test.sh staging

# Extended timeout
TEST_TIMEOUT=600 ./scripts/testing/integration-test.sh staging
```

#### Performance Testing (`scripts/testing/performance-test.sh`)

Tests:
- 📊 Response time analysis
- 🔄 Load testing with configurable users
- 📈 Stress testing with increasing load
- 📉 Resource utilization monitoring
- 🎯 Performance threshold validation

```bash
# Basic performance test
./scripts/testing/performance-test.sh staging

# Heavy load test
CONCURRENT_USERS=50 TEST_DURATION=300 ./scripts/testing/performance-test.sh

# Test against live URL
TARGET_URL=https://uveddi.com ./scripts/testing/performance-test.sh
```

### 8. Monitoring and Observability

#### Prometheus Monitoring

Configuration in `k8s/monitoring/`:

**ServiceMonitor**: Automatic metrics discovery
**PrometheusRules**: Alert definitions
**Grafana Dashboard**: Visualization

#### Key Metrics

- **Application**: Response time, error rate, throughput
- **Infrastructure**: CPU, memory, disk usage
- **Business**: Analysis jobs, queue size, user activity

#### Alert Rules

- 🚨 Application down (5min threshold)
- ⚠️ High error rate (>5% for 10min)
- ⚠️ High latency (>2s p99 for 10min)
- ⚠️ High resource usage (>80% for 15min)
- 🚨 Pod crash looping
- ⚠️ Database connectivity issues

## Environment Configuration

### Staging Environment

- **Resource limits**: 1Gi memory, 500m CPU
- **Replicas**: 2 (with HPA 2-5)
- **Database**: Embedded PostgreSQL
- **Monitoring**: Basic Prometheus
- **Domain**: staging.uveddi.example.com

### Production Environment

- **Resource limits**: 4Gi memory, 2000m CPU
- **Replicas**: 5 (with HPA 5-20)
- **Database**: External RDS PostgreSQL
- **Monitoring**: Full stack (Prometheus, Grafana, Alertmanager)
- **Domain**: uveddi.com

## Security

### Container Security
- ✅ Non-root user execution
- ✅ Read-only filesystem
- ✅ Dropped capabilities
- ✅ Security contexts
- ✅ No new privileges

### Kubernetes Security
- ✅ Pod Security Standards (restricted)
- ✅ Network policies (deny-all default)
- ✅ Service account with minimal permissions
- ✅ Secret management
- ✅ TLS everywhere

### CI/CD Security
- ✅ Vulnerability scanning (Trivy)
- ✅ Code security analysis (Semgrep)
- ✅ Dependency auditing
- ✅ Container registry scanning
- ✅ Infrastructure security scanning

## Troubleshooting

### Common Issues

**Deployment Stuck**:
```bash
# Check pod status
kubectl get pods -n uveddi

# Check events
kubectl get events -n uveddi --sort-by=.metadata.creationTimestamp

# Check logs
kubectl logs -n uveddi deployment/uveddi --previous
```

**Performance Issues**:
```bash
# Check resource usage
kubectl top pods -n uveddi

# Check HPA status
kubectl get hpa -n uveddi

# Run performance test
./scripts/testing/performance-test.sh
```

**Health Check Failures**:
```bash
# Port forward and test
kubectl port-forward -n uveddi service/uveddi 8080:80
curl http://localhost:8080/health

# Check configuration
kubectl get configmap -n uveddi uveddi-config -o yaml
```

**Rollback Issues**:
```bash
# Check Helm history
helm history uveddi -n uveddi

# Manual rollback
helm rollback uveddi 5 -n uveddi

# Check rollback status
kubectl rollout status deployment/uveddi -n uveddi
```

### Debug Commands

```bash
# Get comprehensive status
kubectl get all -n uveddi

# Describe problematic resources
kubectl describe deployment uveddi -n uveddi
kubectl describe pod <pod-name> -n uveddi

# Check resource quotas
kubectl describe resourcequota -n uveddi

# Network debugging
kubectl run debug --image=nicolaka/netshoot -n uveddi -it --rm
```

## Best Practices

### Deployment Best Practices

1. **Always test in staging** before production
2. **Use specific image tags** (not `latest`) for production
3. **Monitor deployment progress** and set timeouts
4. **Have rollback plan ready** before deploying
5. **Validate health checks** after deployment

### Security Best Practices

1. **Scan containers** before deployment
2. **Use least privilege** for service accounts
3. **Enable network policies** to restrict traffic
4. **Regularly update** base images and dependencies
5. **Monitor security events** and alerts

### Operational Best Practices

1. **Set up proper monitoring** and alerting
2. **Maintain deployment logs** for audit
3. **Document configuration changes**
4. **Test disaster recovery** procedures
5. **Keep infrastructure code** in version control

## Migration Guide

### From Manual to Automated Deployment

1. **Assess current deployment**:
   - Document existing configuration
   - Identify manual steps
   - Note custom configurations

2. **Prepare for automation**:
   - Set up container registry
   - Configure Kubernetes cluster
   - Install required tools

3. **Migrate gradually**:
   - Start with staging environment
   - Validate automated deployment
   - Migrate production with rollback plan

4. **Optimize and monitor**:
   - Fine-tune resource limits
   - Set up comprehensive monitoring
   - Train team on new processes

## Support

For deployment issues:

1. **Check logs**: All scripts generate detailed logs
2. **Review documentation**: This guide covers common scenarios
3. **Test in staging**: Always validate changes in non-production
4. **Use dry-run**: Test deployments without making changes
5. **Monitor resources**: Ensure cluster has adequate capacity

## Contributing

When modifying deployment automation:

1. **Test thoroughly** in staging environment
2. **Update documentation** for any configuration changes
3. **Validate security** implications of changes
4. **Test rollback procedures** after modifications
5. **Update monitoring** if adding new components