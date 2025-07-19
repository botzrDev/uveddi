# Security and RBAC Implementation Guide

## Overview

This guide provides comprehensive documentation for the Uveddi Security and RBAC system (UV-247), covering implementation details, configuration, and usage patterns for enterprise deployment.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Authentication System](#authentication-system)
3. [Authorization Engine](#authorization-engine)
4. [Audit Logging](#audit-logging)
5. [Rate Limiting](#rate-limiting)
6. [Configuration Management](#configuration-management)
7. [Security Middleware](#security-middleware)
8. [Deployment Guide](#deployment-guide)
9. [Troubleshooting](#troubleshooting)

## Architecture Overview

The Uveddi security system implements a comprehensive enterprise-grade security platform with the following components:

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Architecture                     │
├─────────────────────────────────────────────────────────────┤
│  HTTP Request                                               │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────┐    │
│  │ Security    │    │ Rate         │    │ Auth        │    │
│  │ Headers     │───▶│ Limiting     │───▶│ Middleware  │    │
│  │ Middleware  │    │ Middleware   │    │             │    │
│  └─────────────┘    └──────────────┘    └─────────────┘    │
│       │                                        │            │
│       ▼                                        ▼            │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────┐    │
│  │ Authorization│    │ Audit        │    │ Application │    │
│  │ Engine      │◀───│ Logger       │◀───│ Handler     │    │
│  │ (Casbin)    │    │              │    │             │    │
│  └─────────────┘    └──────────────┘    └─────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

- **Authentication Service**: JWT and API key authentication with OAuth/OIDC support
- **Authorization Engine**: Casbin-powered RBAC with domain support for multi-tenancy
- **Audit Logger**: Tamper-evident audit logging with real-time statistics
- **Rate Limiter**: Comprehensive DoS protection and abuse prevention
- **Security Middleware**: HTTP security headers and request validation
- **Configuration Manager**: Hierarchical configuration with secure defaults

## Authentication System

### JWT Authentication

The system supports JWT-based authentication with configurable expiration and secure key management.

#### Configuration

```toml
[authentication]
jwt_secret = "your-secure-32-character-secret-key"
jwt_expiry_hours = 24
jwt_issuer = "uveddi"
jwt_audience = "uveddi-api"
```

#### Usage Example

```rust
use uveddi::security::{AuthenticationService, User, UserRole};

// Initialize authentication service
let auth_service = AuthenticationService::new(config).await?;

// Create user
let user = User::new(
    "user123".to_string(),
    "user@example.com".to_string(),
    "John Doe".to_string(),
);

// Generate JWT token
let token = auth_service
    .generate_jwt_token(&user, vec![UserRole::Developer])
    .await?;

// Validate JWT token
let authenticated_user = auth_service
    .authenticate_jwt(&token)
    .await?;
```

### API Key Authentication

For service-to-service communication, the system supports secure API key authentication.

#### Usage Example

```rust
// Generate API key for service
let api_key = auth_service
    .generate_api_key(&service_user, vec![UserRole::Service])
    .await?;

// Authenticate with API key
let authenticated_service = auth_service
    .authenticate_api_key(&api_key)
    .await?;
```

### OAuth 2.0/OIDC Integration

The system supports integration with enterprise identity providers.

#### Configuration

```toml
[[authentication.oauth_providers]]
provider_name = "google"
client_id = "your-google-client-id"
client_secret = "your-google-client-secret"
redirect_url = "https://your-app.com/auth/callback"

[[authentication.oidc_providers]]
provider_name = "azure"
issuer_url = "https://login.microsoftonline.com/tenant-id/v2.0"
client_id = "your-azure-client-id"
client_secret = "your-azure-client-secret"
redirect_url = "https://your-app.com/auth/azure/callback"
```

## Authorization Engine

The authorization engine uses Casbin with a domain-based RBAC model supporting multi-tenancy.

### RBAC Model

The system implements a 4-parameter RBAC model with domain support:

```ini
[request_definition]
r = sub, dom, obj, act

[policy_definition]
p = sub, dom, obj, act

[role_definition]
g = _, _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && r.obj == p.obj && r.act == p.act
```

### User Roles

The system supports five primary user roles:

- **Admin**: Full system access and configuration
- **Developer**: Project-scoped access for development work
- **QA**: Team-level access for testing and validation
- **Manager**: Read-only access to reports and dashboards
- **Service**: API access for automated integrations

### Usage Example

```rust
use uveddi::security::{AuthorizationEngine, UserRole, AuthContext};

// Initialize authorization engine
let authz_engine = AuthorizationEngine::new().await?;

// Create authorization context
let context = AuthContext::new(
    user_id,
    Some("192.168.1.100".to_string()), // IP address
    Some("Mozilla/5.0...".to_string()), // User agent
    Some(serde_json::json!({"project_id": "proj123"})), // Additional context
);

// Check permission
let allowed = authz_engine
    .check_permission(
        &user_id,
        &UserRole::Developer,
        "projects",
        "read",
        &context
    )
    .await?;

if allowed {
    // Grant access
} else {
    // Deny access
}
```

### Role Management

```rust
// Add role for user
authz_engine
    .add_role_for_user("user123".to_string(), "Developer".to_string())
    .await?;

// Remove role for user
authz_engine
    .delete_role_for_user("user123".to_string(), "Developer".to_string())
    .await?;

// Check if user has role
let has_role = authz_engine
    .has_role_for_user("user123".to_string(), "Developer".to_string())
    .await?;
```

## Audit Logging

The audit logging system provides comprehensive security event tracking with tamper-evident logs.

### Configuration

```toml
[audit]
enabled = true
store_type = "database" # Options: memory, database, file
integrity_verification = true
retention_days = 365

[audit.database]
url = "postgresql://user:pass@localhost/audit_db"
table_name = "audit_events"

[audit.file]
directory = "/var/log/uveddi/audit"
max_file_size_mb = 100
rotation_policy = "daily"
```

### Usage Example

```rust
use uveddi::security::{AuditLogger, AuditEvent, AuditEventType, AuditOutcome};

// Initialize audit logger
let audit_logger = AuditLogger::new(audit_store).await?;

// Create audit event
let event = AuditEvent {
    id: Uuid::new_v4(),
    timestamp: Utc::now(),
    event_type: AuditEventType::Authentication,
    user_id: Some(user_id),
    session_id: Some(session_id),
    resource: "login".to_string(),
    action: "jwt_authentication".to_string(),
    outcome: AuditOutcome::Success,
    ip_address: Some("192.168.1.100".to_string()),
    user_agent: Some("Mozilla/5.0...".to_string()),
    additional_data: serde_json::json!({
        "login_method": "jwt",
        "user_agent": "web_browser"
    }),
    integrity_hash: String::new(), // Calculated by logger
};

// Log the event
audit_logger.log(event).await?;
```

### Audit Event Types

- **Authentication**: Login, logout, token generation/validation
- **Authorization**: Permission checks, role assignments
- **DataAccess**: Resource access, data modifications
- **ConfigurationChange**: System configuration updates
- **SecurityViolation**: Failed authentication, unauthorized access attempts
- **SystemEvent**: System startup, shutdown, maintenance operations

### Audit Statistics

```rust
// Get audit statistics
let start_time = Utc::now() - Duration::days(30);
let end_time = Utc::now();
let stats = audit_store.get_statistics(start_time, end_time).await?;

println!("Total events: {}", stats.total_events);
println!("Events by type: {:?}", stats.events_by_type);
println!("Events by outcome: {:?}", stats.events_by_outcome);
```

## Rate Limiting

The rate limiting system provides comprehensive DoS protection and abuse prevention.

### Configuration

```toml
[rate_limiting]
enabled = true
default_rate_limit = 100 # requests per window
window_seconds = 60
burst_size = 20
storage_type = "memory" # Options: memory, redis

[rate_limiting.redis]
url = "redis://localhost:6379"
key_prefix = "uveddi:ratelimit:"

# Endpoint-specific limits
[[rate_limiting.endpoint_limits]]
endpoint = "/api/auth/login"
rate_limit = 5
window_seconds = 300 # 5 requests per 5 minutes

[[rate_limiting.endpoint_limits]]
endpoint = "/api/admin/*"
rate_limit = 50
window_seconds = 60
```

### Usage Example

```rust
use uveddi::security::{RateLimiter, RateLimitingConfig};

// Initialize rate limiter
let rate_limiter = RateLimiter::new(storage, config).await?;

// Check rate limit
let identifier = "user123"; // or IP address
let endpoint = "/api/projects";

let allowed = rate_limiter
    .check_rate_limit(identifier, endpoint)
    .await?;

if !allowed {
    return Err(SecurityError::RateLimitExceeded);
}
```

### Rate Limit Information

```rust
// Get rate limit info
let info = rate_limiter
    .get_rate_limit_info(identifier, endpoint)
    .await?;

if let Some(info) = info {
    println!("Requests made: {}", info.requests_made);
    println!("Limit: {}", info.limit);
    println!("Reset time: {}", info.reset_time);
}
```

## Configuration Management

The configuration system supports hierarchical configuration with environment-specific overrides.

### Configuration Structure

```toml
# config/security/default.toml
[authentication]
jwt_secret = "default-development-secret-key-32chars"
jwt_expiry_hours = 24
jwt_issuer = "uveddi"
jwt_audience = "uveddi-api"

[authorization]
enabled = true
model_file = "security/rbac_model.conf"
policy_file = "security/rbac_policy.csv"

[audit]
enabled = true
store_type = "memory"
integrity_verification = true

[rate_limiting]
enabled = true
default_rate_limit = 100
window_seconds = 60
burst_size = 20
```

### Environment Overrides

```toml
# config/security/production.toml
[authentication]
jwt_secret = "${JWT_SECRET}" # From environment variable
jwt_expiry_hours = 8 # Shorter expiry in production

[audit]
store_type = "database"
[audit.database]
url = "${DATABASE_URL}"

[rate_limiting]
storage_type = "redis"
[rate_limiting.redis]
url = "${REDIS_URL}"
```

### Loading Configuration

```rust
use uveddi::security::{SecurityConfig, SecurityConfigLoader};

// Load configuration with environment overrides
let config = SecurityConfigLoader::new()
    .add_file("config/security/default.toml")?
    .add_file("config/security/production.toml")?
    .add_environment("UVEDDI_SECURITY")?
    .build()?;

// Validate configuration
config.validate()?;
```

### Environment Variables

```bash
# Authentication
export UVEDDI_SECURITY_AUTHENTICATION_JWT_SECRET="your-production-secret-key"
export UVEDDI_SECURITY_AUTHENTICATION_JWT_EXPIRY_HOURS="8"

# Database
export UVEDDI_SECURITY_AUDIT_DATABASE_URL="postgresql://user:pass@db:5432/audit"

# Redis
export UVEDDI_SECURITY_RATE_LIMITING_REDIS_URL="redis://redis:6379"
```

## Security Middleware

The security middleware provides HTTP security headers and request validation.

### Middleware Stack

```rust
use uveddi::security::middleware::{
    auth_middleware, authz_middleware, security_headers_middleware,
    rate_limiting_middleware, error_handling_middleware
};

// Create middleware stack
let app = Router::new()
    .route("/api/*", get(api_handler))
    .layer(error_handling_middleware)
    .layer(authz_middleware)
    .layer(auth_middleware)
    .layer(rate_limiting_middleware)
    .layer(security_headers_middleware);
```

### Security Headers

The middleware automatically adds security headers:

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- `Content-Security-Policy: default-src 'self'`
- `Referrer-Policy: strict-origin-when-cross-origin`

### Custom Middleware Configuration

```rust
// Configure endpoint-specific security
let protected_routes = Router::new()
    .route("/admin/*", get(admin_handler))
    .layer(require_role_middleware(UserRole::Admin))
    .layer(rate_limit_middleware(5, Duration::from_secs(60))); // 5 req/min
```

## Deployment Guide

### Production Deployment Checklist

#### 1. Configuration Security
- [ ] Generate secure JWT secret (32+ characters)
- [ ] Configure production database URLs
- [ ] Set up Redis for distributed rate limiting
- [ ] Configure OAuth/OIDC providers
- [ ] Set appropriate token expiry times

#### 2. Database Setup
```sql
-- Run security schema migration
\i migrations/V5__security_schema.sql

-- Create indexes for performance
CREATE INDEX idx_audit_events_timestamp ON audit_events(timestamp);
CREATE INDEX idx_audit_events_user_id ON audit_events(user_id);
CREATE INDEX idx_audit_events_event_type ON audit_events(event_type);
```

#### 3. Environment Variables
```bash
# Required production environment variables
export UVEDDI_SECURITY_AUTHENTICATION_JWT_SECRET="$(openssl rand -base64 32)"
export UVEDDI_SECURITY_AUDIT_DATABASE_URL="postgresql://audit_user:secure_pass@db:5432/audit_db"
export UVEDDI_SECURITY_RATE_LIMITING_REDIS_URL="redis://redis:6379/0"
```

#### 4. Monitoring Setup
```bash
# Configure log aggregation
export UVEDDI_SECURITY_AUDIT_FILE_DIRECTORY="/var/log/uveddi/audit"

# Set up metrics collection
export UVEDDI_SECURITY_METRICS_ENABLED="true"
export UVEDDI_SECURITY_METRICS_ENDPOINT="/metrics"
```

### Docker Deployment

```dockerfile
# Dockerfile security configuration
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features security

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/uveddi /usr/local/bin/
COPY config/ /app/config/

# Security: Run as non-root user
RUN useradd -r -s /bin/false uveddi
USER uveddi

EXPOSE 8080
CMD ["uveddi", "--config", "/app/config/security/production.toml"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uveddi-security
spec:
  replicas: 3
  selector:
    matchLabels:
      app: uveddi-security
  template:
    metadata:
      labels:
        app: uveddi-security
    spec:
      containers:
      - name: uveddi
        image: uveddi:latest
        ports:
        - containerPort: 8080
        env:
        - name: UVEDDI_SECURITY_AUTHENTICATION_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: uveddi-secrets
              key: jwt-secret
        - name: UVEDDI_SECURITY_AUDIT_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: uveddi-secrets
              key: database-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
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
```

## Troubleshooting

### Common Issues

#### 1. JWT Token Validation Failures
```
Error: SecurityError::InvalidToken
```

**Solutions**:
- Verify JWT secret is correctly configured
- Check token expiration settings
- Ensure clock synchronization between services
- Validate JWT issuer and audience claims

#### 2. Authorization Permission Denied
```
Error: SecurityError::PermissionDenied
```

**Solutions**:
- Verify user has required role assigned
- Check RBAC policy configuration
- Validate domain/tenant context
- Review permission scoping rules

#### 3. Rate Limiting Issues
```
Error: SecurityError::RateLimitExceeded
```

**Solutions**:
- Review rate limiting configuration
- Check if limits are too restrictive
- Verify rate limit storage (Redis) connectivity
- Consider implementing rate limit exemptions

#### 4. Audit Logging Failures
```
Error: SecurityError::AuditLogError
```

**Solutions**:
- Check database connectivity and permissions
- Verify audit table schema is up to date
- Review disk space for file-based logging
- Check audit configuration settings

### Performance Optimization

#### 1. Authorization Caching
```rust
// Enable permission caching
let authz_config = AuthorizationConfig {
    cache_enabled: true,
    cache_ttl_seconds: 300, // 5 minutes
    cache_max_entries: 10000,
};
```

#### 2. Database Optimization
```sql
-- Optimize audit queries
CREATE INDEX CONCURRENTLY idx_audit_events_composite 
ON audit_events(user_id, event_type, timestamp);

-- Partition large audit tables
CREATE TABLE audit_events_2024_01 PARTITION OF audit_events
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
```

#### 3. Rate Limiting Optimization
```toml
[rate_limiting]
# Use Redis for distributed rate limiting
storage_type = "redis"
# Optimize Redis connection pool
[rate_limiting.redis]
max_connections = 20
connection_timeout_ms = 5000
```

### Monitoring and Alerting

#### 1. Security Metrics
Monitor these key security metrics:
- Authentication success/failure rates
- Authorization denial rates
- Rate limiting trigger frequency
- Audit log volume and errors
- JWT token expiration patterns

#### 2. Alert Conditions
Set up alerts for:
- High authentication failure rates (> 10% in 5 minutes)
- Authorization denials for admin resources
- Rate limiting threshold breaches
- Audit logging failures
- Unusual access patterns

#### 3. Log Analysis
```bash
# Monitor authentication failures
grep "SecurityError::AuthenticationFailed" /var/log/uveddi/security.log

# Track authorization denials
grep "SecurityError::PermissionDenied" /var/log/uveddi/security.log

# Analyze rate limiting
grep "SecurityError::RateLimitExceeded" /var/log/uveddi/security.log
```

---

This implementation guide provides comprehensive documentation for deploying and managing the Uveddi security system in production environments. For additional support, refer to the API documentation and security best practices guide.