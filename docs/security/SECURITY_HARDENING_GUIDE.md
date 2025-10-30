# Uveddi Security Hardening Guide

## Overview

This guide documents the comprehensive security hardening measures implemented in Uveddi as part of UV-94: Security Hardening initiative. It covers HTTP client security, container security, testing protocols, and compliance validation.

## Table of Contents

1. [HTTP Client Security](#http-client-security)
2. [Container Security Hardening](#container-security-hardening)
3. [Security Testing Framework](#security-testing-framework)
4. [Compliance Validation](#compliance-validation)
5. [Security Configuration](#security-configuration)
6. [Incident Response](#incident-response)
7. [Security Monitoring](#security-monitoring)
8. [Best Practices](#best-practices)

## HTTP Client Security

### SecureHttpClient Implementation

Uveddi implements a hardened HTTP client (`SecureHttpClient`) that enforces security best practices:

#### Key Features:
- **HTTPS Enforcement**: Blocks all HTTP requests in production
- **Certificate Validation**: Strict TLS certificate verification
- **Timeout Management**: Configurable request, connection, and read timeouts
- **Rate Limiting**: Built-in rate limiting to prevent abuse
- **User Agent Control**: Configurable user agent strings

#### Configuration:
```rust
HttpSecurityConfig {
    enforce_https: true,           // Force HTTPS in production
    timeout_seconds: 30,           // Request timeout
    connect_timeout_seconds: 10,   // Connection timeout
    read_timeout_seconds: 30,      // Read timeout
    user_agent: "Uveddi/1.0",     // Custom user agent
    max_redirects: 3,              // Maximum redirects
}
```

#### Usage:
```rust
use uveddi::security::{SecureHttpClient, HttpSecurityConfig};

let config = HttpSecurityConfig::default();
let client = SecureHttpClient::new(config)?;
let response = client.get("https://api.example.com").await?;
```

### Security Considerations:
- All HTTP clients in the codebase have been updated to use `SecureHttpClient`
- Development builds allow HTTP for localhost testing
- Production builds strictly enforce HTTPS
- Certificate pinning can be enabled for critical endpoints

## Container Security Hardening

### Dockerfile Security Measures

The Uveddi container implements multiple security layers:

#### Multi-Stage Build:
```dockerfile
# Builder stage - minimal build dependencies
FROM rust:1.70-slim AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates libsqlite3-dev git

# Runtime stage - minimal runtime image
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 libsqlite3-0 curl
```

#### Non-Root User Execution:
```dockerfile
# Create dedicated user with minimal privileges
RUN groupadd -r -g 1001 uveddi && \
    useradd -r -g uveddi -u 1001 -s /bin/false -M uveddi

# Switch to non-root user
USER uveddi
```

#### Secure File Permissions:
```dockerfile
# Create directories with proper ownership
RUN mkdir -p /app/data /app/cache /app/logs && \
    chown -R uveddi:uveddi /app

# Set executable permissions
RUN chmod +x /docker-entrypoint.sh && \
    chmod +x /usr/local/bin/uveddi
```

### Docker Compose Security Constraints

The `deploy/docker-compose.yml` bundle implements comprehensive security controls:

```yaml
services:
  uveddi:
    security_opt:
      - no-new-privileges:true    # Prevent privilege escalation
      - apparmor:docker-default   # Enable AppArmor
    read_only: true              # Read-only filesystem
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
      - /var/tmp:noexec,nosuid,size=50m
    cap_drop:
      - ALL                      # Drop all capabilities
    cap_add:
      - NET_BIND_SERVICE         # Only required capabilities
    user: "1001:1001"           # Non-root user
    deploy:
      resources:
        limits:
          memory: 2G             # Memory limits
          cpus: '1.0'           # CPU limits
```

### Security Entrypoint Script

The `deploy/docker-entrypoint.sh` script implements additional security measures:

#### Signal Handling:
```bash
cleanup() {
    log "Received shutdown signal, cleaning up..."
    if [ ! -z "$UVEDDI_PID" ]; then
        kill -TERM "$UVEDDI_PID" 2>/dev/null || true
        wait "$UVEDDI_PID" 2>/dev/null || true
    fi
    exit 0
}
trap cleanup SIGTERM SIGINT
```

#### Security Validation:
```bash
# Ensure non-root execution
if [ "$(id -u)" = "0" ]; then
    log "ERROR: Container should not run as root user"
    exit 1
fi

# Validate required directories
for dir in "/app/data" "/app/cache" "/app/logs"; do
    if [ ! -w "$dir" ]; then
        log "ERROR: Directory $dir is not writable"
        exit 1
    fi
done
```

## Security Testing Framework

### Test Categories

#### 1. Comprehensive Security Validation
Location: `tests/security/comprehensive_security_validation.rs`

Tests include:
- HTTP client security enforcement
- Certificate validation
- Input validation security
- File access security
- Rate limiting enforcement
- Audit logging functionality
- Encryption configuration
- Container security validation

#### 2. Compliance Validation
Location: `tests/security/compliance_validation.rs`

OWASP Top 10 Tests:
- **A01**: Broken Access Control
- **A02**: Cryptographic Failures
- **A03**: Injection attacks
- **A04**: Insecure Design
- **A05**: Security Misconfiguration
- **A06**: Vulnerable Components
- **A07**: Authentication Failures
- **A08**: Integrity Failures
- **A09**: Logging/Monitoring Failures
- **A10**: Server-Side Request Forgery

### Running Security Tests

```bash
# Run all security tests
cargo test --test comprehensive_security_validation
cargo test --test compliance_validation

# Run specific security test categories
cargo test security:: -- --nocapture
cargo test input_validation -- --nocapture
cargo test path_traversal -- --nocapture
```

## Compliance Validation

### OWASP Top 10 Compliance

Uveddi implements protections against all OWASP Top 10 vulnerabilities:

1. **Broken Access Control (A01)**
   - Role-based access control
   - Default deny principle
   - Session timeout enforcement

2. **Cryptographic Failures (A02)**
   - Strong encryption algorithms (AES-256-GCM)
   - Minimum 256-bit keys
   - No hardcoded secrets

3. **Injection (A03)**
   - Input validation and sanitization
   - Parameterized queries
   - Command injection prevention

4. **Insecure Design (A04)**
   - Secure defaults
   - Defense in depth
   - Threat modeling

5. **Security Misconfiguration (A05)**
   - Hardened containers
   - Secure configuration management
   - Regular security reviews

6. **Vulnerable Components (A06)**
   - Dependency scanning (cargo-audit)
   - Automated vulnerability detection
   - Regular updates

7. **Authentication Failures (A07)**
   - Strong authentication requirements
   - Account lockout mechanisms
   - Secure session management

8. **Integrity Failures (A08)**
   - Container image verification
   - Dependency integrity checks
   - Configuration validation

9. **Logging/Monitoring Failures (A09)**
   - Comprehensive audit logging
   - Security event monitoring
   - Log retention policies

10. **Server-Side Request Forgery (A10)**
    - URL validation
    - Network access restrictions
    - Input sanitization

### Container Security Standards

- **Non-root execution**: UID/GID 1001
- **Minimal attack surface**: Multi-stage builds
- **Capability dropping**: ALL capabilities dropped
- **Resource limits**: Memory and CPU constraints
- **Read-only filesystem**: Write access only to specific directories
- **Security options**: no-new-privileges, AppArmor

## Security Configuration

### Main Security Configuration

```rust
SecurityConfig {
    authentication: AuthenticationConfig {
        enabled: true,
        max_failed_attempts: 5,
        lockout_duration_minutes: 15,
        session_timeout_minutes: 240,
        require_secure_cookies: true,
    },
    authorization: AuthorizationConfig {
        enabled: true,
        default_role: "guest",
    },
    encryption: EncryptionConfig {
        enabled: true,
        algorithm: "AES-256-GCM",
        key_length: 256,
    },
    rate_limiting: RateLimitingConfig {
        enabled: true,
        requests_per_minute: 100,
        burst_size: 10,
    },
    audit_logging: AuditLoggingConfig {
        enabled: true,
        log_file_path: "/app/logs/audit.log",
        max_file_size_mb: 100,
        max_files: 10,
        log_failed_auth: true,
        log_access_violations: true,
        log_config_changes: true,
    },
}
```

### Environment Variables

Security-sensitive configuration should use environment variables:

```bash
# Required security configuration
export SECURITY_ENCRYPTION_KEY_FILE=/secrets/encryption.key
export AUDIT_LOG_PATH=/app/logs/audit.log
export RATE_LIMIT_REQUESTS_PER_MINUTE=100
export SESSION_TIMEOUT_MINUTES=240
```

## Incident Response

### Security Incident Classification

**Critical (P0)**:
- Confirmed data breach
- System compromise
- Privilege escalation

**High (P1)**:
- Suspected data breach
- Authentication bypass
- Injection attacks

**Medium (P2)**:
- Failed authentication patterns
- Configuration drift
- Dependency vulnerabilities

**Low (P3)**:
- Security warnings
- Compliance violations
- Monitoring alerts

### Response Procedures

1. **Detection**: Automated monitoring and manual reporting
2. **Assessment**: Severity classification and impact analysis
3. **Containment**: Immediate threat mitigation
4. **Investigation**: Root cause analysis
5. **Recovery**: System restoration and hardening
6. **Lessons Learned**: Post-incident review and improvements

## Security Monitoring

### Continuous Security Scanning

The GitHub Actions security workflow provides continuous monitoring:

- **Dependency Scanning**: cargo-audit, cargo-deny
- **Static Code Analysis**: Clippy with security lints, Semgrep
- **Container Scanning**: Trivy, Docker Scout
- **Secret Detection**: truffleHog
- **Compliance Validation**: Automated OWASP Top 10 checks

### Security Metrics

Key security metrics tracked:
- Failed authentication attempts
- Rate limiting violations
- Certificate validation failures
- Container security violations
- Dependency vulnerabilities
- Compliance score

### Alerting

Security alerts are configured for:
- Critical security events
- Compliance violations
- Failed security tests
- Dependency vulnerabilities
- Configuration drift

## Best Practices

### Development Guidelines

1. **Secure Coding**:
   - Use `SecureHttpClient` for all HTTP requests
   - Validate all inputs
   - Follow principle of least privilege
   - Implement defense in depth

2. **Testing**:
   - Run security tests before deployment
   - Validate compliance requirements
   - Test failure scenarios
   - Monitor security metrics

3. **Configuration**:
   - Use secure defaults
   - Environment-specific configuration
   - Regular security reviews
   - Automated compliance checks

4. **Deployment**:
   - Container security scanning
   - Non-root user execution
   - Resource limits
   - Security monitoring

### Security Checklist

- [ ] All HTTP clients use `SecureHttpClient`
- [ ] HTTPS enforcement enabled in production
- [ ] Container runs as non-root user (UID 1001)
- [ ] Security constraints applied in docker-compose
- [ ] All security tests passing
- [ ] Dependency vulnerabilities resolved
- [ ] Compliance requirements validated
- [ ] Security monitoring configured
- [ ] Incident response procedures documented
- [ ] Security training completed

## Updates and Maintenance

### Regular Security Tasks

**Weekly**:
- Review security alerts
- Check dependency updates
- Monitor compliance scores

**Monthly**:
- Security configuration review
- Penetration testing
- Compliance assessment

**Quarterly**:
- Security architecture review
- Incident response testing
- Security training updates

### Security Contacts

- **Security Team**: security@uveddi.dev
- **Incident Response**: incident-response@uveddi.dev
- **Compliance**: compliance@uveddi.dev

## Conclusion

This security hardening guide provides comprehensive protection against modern security threats. Regular review and updates ensure continued security posture improvement. For questions or concerns, contact the security team.

---

**Document Version**: 1.0  
**Last Updated**: 2024-01-24  
**Next Review**: 2024-04-24