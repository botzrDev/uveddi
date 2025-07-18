# GPT Development Prompt: UV-247 - Security and RBAC Implementation

## 🎯 **Objective**

You are a senior Rust security engineer tasked with implementing a comprehensive security and RBAC (Role-Based Access Control) system for Uveddi, a static code analysis platform. This implementation must meet enterprise-grade security requirements including SOC 2 and ISO 27001 compliance standards.

## 📋 **Task Context**

**Issue**: UV-247 - Phase 4.1: Security and RBAC Implementation  
**Priority**: High  
**Story Points**: 13  
**Current Status**: Research Complete  
**Type**: Epic Implementation

## 🏗️ **Architecture Overview**

Based on comprehensive research (docs/06-research/Specialized/UV-247/UV-247_Research.md), implement a **hybrid RBAC/ABAC system** with the following components:

### **Core Architecture Pillars**
1. **Hybrid RBAC/ABAC Authorization**: Role-based permissions with attribute-based fine-grained control
2. **Federated Authentication**: OAuth 2.0/OIDC integration with enterprise identity providers
3. **API Security**: Secure API key management for service-to-service communication
4. **Environment-Specific Configuration**: Hierarchical configuration with secure secret management
5. **Comprehensive Audit Logging**: Immutable audit trail for compliance

## 🎯 **User Roles to Implement**

```rust
// Core user roles with specific permissions
pub enum UserRole {
    Admin,      // Full system access and configuration
    Developer,  // Test data access for owned projects only
    QA,         // Test execution and failure analysis access
    Manager,    // Read-only access to reports and dashboards
    Service,    // API access for automated integrations
}
```

## 🔧 **Implementation Requirements**

### **Phase 1: Core RBAC Foundation (Priority 1)**

#### **1.1 Database Schema Implementation**
Create the core RBAC tables in your migration system:

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255) UNIQUE NOT NULL, -- From IdP
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    is_system_role BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Permissions table
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource VARCHAR(100) NOT NULL, -- e.g., 'projects', 'reports'
    action VARCHAR(50) NOT NULL,    -- e.g., 'read', 'write', 'delete'
    scope VARCHAR(100),             -- e.g., 'own', 'team', 'all'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(resource, action, scope)
);

-- User-Role assignments
CREATE TABLE user_roles (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    PRIMARY KEY (user_id, role_id)
);

-- Role-Permission assignments
CREATE TABLE role_permissions (
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);
```

#### **1.2 Core Authorization Engine**
Implement using **casbin-rs** as recommended in research:

```rust
// src/security/authorization.rs
use casbin::{Enforcer, Model, FileAdapter};
use uuid::Uuid;

pub struct AuthorizationEngine {
    enforcer: Enforcer,
}

impl AuthorizationEngine {
    pub async fn new() -> Result<Self, SecurityError> {
        // Initialize casbin with RBAC model
        let model = Model::from_str(RBAC_MODEL).await?;
        let adapter = FileAdapter::new("security/policies.csv");
        let enforcer = Enforcer::new(model, adapter).await?;
        
        Ok(Self { enforcer })
    }
    
    pub async fn check_permission(
        &self,
        user_id: &Uuid,
        resource: &str,
        action: &str,
        context: &AuthContext,
    ) -> Result<bool, SecurityError> {
        // Implement hybrid RBAC/ABAC logic
        // 1. Check role-based permissions (RBAC)
        // 2. Apply attribute-based restrictions (ABAC)
        todo!("Implement authorization logic")
    }
}

// RBAC model configuration for casbin
const RBAC_MODEL: &str = r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
"#;
```

#### **1.3 Authentication Integration**
Implement OAuth 2.0/OIDC using **oauth2** and **openidconnect** crates:

```rust
// src/security/authentication.rs
use oauth2::{AuthorizationCode, ClientId, ClientSecret, RedirectUrl};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    IssuerUrl, Nonce, TokenResponse,
};

pub struct AuthenticationService {
    oidc_clients: HashMap<String, CoreClient>,
    jwt_validator: JwtValidator,
}

impl AuthenticationService {
    pub async fn authenticate_oidc(
        &self,
        provider: &str,
        auth_code: &AuthorizationCode,
        nonce: &Nonce,
    ) -> Result<AuthenticatedUser, SecurityError> {
        // Implement OIDC authentication flow
        todo!("Implement OIDC authentication")
    }
    
    pub async fn authenticate_api_key(
        &self,
        api_key: &str,
    ) -> Result<AuthenticatedUser, SecurityError> {
        // Implement API key authentication for services
        todo!("Implement API key authentication")
    }
}
```

### **Phase 2: API Security and Middleware (Priority 2)**

#### **2.1 Security Middleware**
Implement authentication and authorization middleware:

```rust
// src/security/middleware.rs
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(auth_service): State<AuthenticationService>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract and validate authentication token
    let auth_header = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let user = auth_service
        .authenticate_token(auth_header)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add user context to request
    request.extensions_mut().insert(user);
    
    Ok(next.run(request).await)
}

pub async fn rbac_middleware(
    State(authz_engine): State<AuthorizationEngine>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract user and check permissions
    let user = request.extensions().get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let resource = extract_resource_from_path(request.uri().path());
    let action = extract_action_from_method(request.method());
    
    let allowed = authz_engine
        .check_permission(&user.id, &resource, &action, &AuthContext::from_request(&request))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !allowed {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(request).await)
}
```

#### **2.2 Rate Limiting**
Implement rate limiting using **tower-governor**:

```rust
// src/security/rate_limiting.rs
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use std::time::Duration;

pub fn create_rate_limiter() -> GovernorLayer<'static, (), axum::extract::ConnectInfo<std::net::SocketAddr>> {
    let config = GovernorConfigBuilder::default()
        .per_second(10) // 10 requests per second
        .burst_size(20) // Allow bursts up to 20 requests
        .finish()
        .unwrap();
    
    GovernorLayer::new(&config)
}
```

### **Phase 3: Configuration and Secrets Management (Priority 3)**

#### **3.1 Hierarchical Configuration**
Implement environment-specific security configuration:

```rust
// src/security/config.rs
use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub authentication: AuthConfig,
    pub authorization: AuthzConfig,
    pub api_security: ApiSecurityConfig,
    pub audit: AuditConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
    pub oidc_providers: Vec<OidcProviderConfig>,
    pub jwt_secret: String, // Should be loaded from secure store
    pub token_expiry: Duration,
}

impl SecurityConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::builder()
            .add_source(File::with_name("config/security/default"))
            .add_source(File::with_name(&format!(
                "config/security/{}",
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            )).required(false))
            .add_source(Environment::with_prefix("UVEDDI_SECURITY"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

#### **3.2 Secure Secret Management**
Integrate with external secret management:

```rust
// src/security/secrets.rs
pub trait SecretStore: Send + Sync {
    async fn get_secret(&self, key: &str) -> Result<String, SecurityError>;
    async fn set_secret(&self, key: &str, value: &str) -> Result<(), SecurityError>;
}

pub struct HashiCorpVaultStore {
    client: vault::Client,
}

pub struct AwsSecretsManagerStore {
    client: aws_sdk_secretsmanager::Client,
}

// Implement SecretStore for both providers
```

### **Phase 4: Audit Logging (Priority 4)**

#### **4.1 Comprehensive Audit System**
Implement immutable audit logging:

```rust
// src/security/audit.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub resource: String,
    pub action: String,
    pub outcome: AuditOutcome,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub additional_data: serde_json::Value,
    pub integrity_hash: String, // For tamper detection
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    ConfigurationChange,
    SecurityViolation,
    SystemEvent,
}

pub struct AuditLogger {
    store: Box<dyn AuditStore>,
    hasher: Blake3Hasher,
}

impl AuditLogger {
    pub async fn log_event(&self, event: AuditEvent) -> Result<(), SecurityError> {
        // Calculate integrity hash
        let mut event_with_hash = event;
        event_with_hash.integrity_hash = self.calculate_hash(&event_with_hash);
        
        // Store immutably
        self.store.store_event(event_with_hash).await
    }
}
```

## ✅ **Acceptance Criteria**

Your implementation must ensure:

### **Core Functionality**
- [ ] RBAC system controls access to sensitive monitoring data correctly
- [ ] API authentication prevents unauthorized access
- [ ] Environment-specific configurations supported
- [ ] Credentials managed securely with no hardcoded secrets
- [ ] Audit logs track all system access and modifications
- [ ] OAuth integration with existing identity systems working
- [ ] Role permissions enforced at API and UI levels

### **Security Standards**
- [ ] All authentication flows follow OAuth 2.0/OIDC standards
- [ ] API keys are securely generated, stored, and validated
- [ ] Rate limiting prevents abuse and DoS attacks
- [ ] Audit logs are tamper-evident and immutable
- [ ] Configuration supports secure secret management
- [ ] All security events are properly logged

### **Performance Requirements**
- [ ] Authentication adds < 50ms latency to API requests
- [ ] Authorization checks complete in < 10ms
- [ ] Rate limiting doesn't impact legitimate usage
- [ ] Audit logging is asynchronous and non-blocking

## 🛠 **Required Dependencies**

Add these to your `Cargo.toml`:

```toml
[dependencies]
# Authorization
casbin = "2.0"
oso = "0.27" # Alternative authorization engine

# Authentication
oauth2 = "4.4"
openidconnect = "3.0"
jsonwebtoken = "9.0"

# Security
argon2 = "0.5" # Password hashing
blake3 = "1.5" # Integrity hashing
ring = "0.17" # Cryptographic operations

# Rate limiting
tower-governor = "0.1"

# Configuration
config = "0.14"
serde = { version = "1.0", features = ["derive"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Database
sqlx = { version = "0.7", features = ["postgres", "uuid", "chrono"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# HTTP
axum = "0.7"
tower = "0.4"
tower-http = "0.5"
```

## 🧪 **Testing Strategy**

### **Unit Tests**
- Authorization engine policy evaluation
- Token validation and parsing
- Rate limiting logic
- Audit event generation

### **Integration Tests**
- End-to-end authentication flows
- RBAC permission enforcement
- API security middleware
- Configuration loading

### **Security Tests**
- Penetration testing scenarios
- Token manipulation attempts
- Rate limiting bypass attempts
- Audit log integrity verification

## 📁 **File Structure**

Create this security module structure:

```
src/security/
├── mod.rs                 # Public API and re-exports
├── authentication.rs     # OAuth/OIDC and API key auth
├── authorization.rs      # RBAC/ABAC engine
├── middleware.rs         # HTTP middleware
├── config.rs             # Security configuration
├── secrets.rs            # Secret management
├── audit.rs              # Audit logging
├── rate_limiting.rs      # Rate limiting
├── models.rs             # Security data models
└── errors.rs             # Security-specific errors

config/security/
├── default.toml          # Default security config
├── development.toml      # Dev environment overrides
├── staging.toml          # Staging environment overrides
└── production.toml       # Production environment overrides

migrations/
└── V5__security_schema.sql # Database schema for RBAC
```

## 🚨 **Security Considerations**

### **Critical Security Requirements**
- Never log sensitive data (passwords, tokens, secrets)
- Use secure random number generation for all cryptographic operations
- Implement proper session management with secure cookies
- Validate all inputs to prevent injection attacks
- Use HTTPS only in production environments
- Implement proper CORS policies

### **Compliance Requirements (SOC 2/ISO 27001)**
- All authentication attempts must be logged
- Failed authorization attempts must trigger alerts
- User privilege changes must be audited
- System configuration changes must be logged
- Data access must be tracked and auditable

## 🎯 **Success Metrics**

- **Zero security vulnerabilities** in static analysis scans
- **< 50ms authentication latency** for API requests
- **100% audit coverage** for security-relevant events
- **Multi-provider OIDC support** working correctly
- **Comprehensive test coverage** (>90% for security modules)

## 📝 **Implementation Notes**

### **Phase 1 Priority**
Start with the core RBAC foundation - this unblocks other development work.

### **Integration Points**
- Integrate with existing `src/analysis/engine.rs` for resource access control
- Hook into existing API endpoints in your web framework
- Integrate with the monitoring system for security metrics

### **Future Extensibility**
- Design for multi-tenancy support
- Plan for additional identity providers
- Consider future compliance requirements (GDPR, HIPAA)

---

**Remember**: Security is not just about implementing features - it's about building a robust, auditable, and compliant system that enterprises can trust with their sensitive data. Focus on defense in depth and assume that every component could be compromised.

This implementation will establish Uveddi as an enterprise-ready platform with best-in-class security architecture.