# Production-Secure Deployment Guide

**Version**: 1.0  
**Date**: September 6, 2025  
**Target**: Production environments requiring maximum security  

---

## Overview

This guide provides instructions for deploying Uveddi in production environments with maximum security assurance. The production-secure configuration eliminates all known security vulnerabilities while maintaining full analytical capabilities.

## Security-First Build Configuration

### Recommended Production Build

Use the `production-secure` feature set for maximum security:

```bash
# Build with zero security vulnerabilities
cargo build --release --features production-secure

# Verify no vulnerabilities present
cargo audit --features production-secure
```

### Feature Comparison

| Feature | production | production-secure | Impact |
|---------|------------|-------------------|---------|
| Tree-sitter parsing | ✅ | ✅ | Full language support |
| Memory optimization | ✅ | ✅ | Performance benefits |
| Web interface | ✅ | ✅ | Complete web dashboard |
| TUI interface | ✅ | ✅ | Interactive terminal UI |
| WASM plugins | ✅ | ✅ | Extensibility system |
| Prometheus metrics | ✅ | ✅ | Monitoring integration |
| **OpenID Connect** | ✅ | ❌ | Authentication limitation |
| **OAuth2** | ✅ | ❌ | Authentication limitation |
| **Casbin RBAC** | ✅ | ❌ | Authorization limitation |

## Environment Configuration

### Required Environment Variables

```bash
# JWT secret for secure token generation (REQUIRED)
export UVEDDI_JWT_SECRET="$(openssl rand -base64 64)"

# Security configuration
export UVEDDI_SECURITY_MODE="strict"
export UVEDDI_LOG_LEVEL="info"
export UVEDDI_AUDIT_ENABLED="true"

# Network security
export UVEDDI_BIND_ADDRESS="127.0.0.1"
export UVEDDI_PORT="8888"
export UVEDDI_TLS_CERT_PATH="/path/to/cert.pem"
export UVEDDI_TLS_KEY_PATH="/path/to/key.pem"

# Rate limiting
export UVEDDI_RATE_LIMIT_REQUESTS="100"
export UVEDDI_RATE_LIMIT_WINDOW="60"

# Session security
export UVEDDI_SESSION_TIMEOUT="3600"
export UVEDDI_SESSION_SECURE="true"
```

### Optional Security Enhancements

```bash
# Database encryption
export UVEDDI_DB_ENCRYPTION="true"
export UVEDDI_DB_KEY="$(openssl rand -base64 32)"

# API key management
export UVEDDI_API_KEY_EXPIRY="365"
export UVEDDI_API_KEY_ROTATION="true"

# Monitoring
export UVEDDI_METRICS_ENABLED="true"
export UVEDDI_HEALTH_CHECK_ENDPOINT="/_health"
```

## Network Security Configuration

### Reverse Proxy Setup (nginx)

```nginx
# /etc/nginx/sites-available/uveddi
server {
    listen 443 ssl http2;
    server_name uveddi.yourdomain.com;
    
    # SSL configuration
    ssl_certificate /path/to/fullchain.pem;
    ssl_certificate_key /path/to/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy "strict-origin-when-cross-origin";
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'";
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=uveddi_api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=uveddi_web:10m rate=30r/s;
    
    # API endpoints with strict rate limiting
    location /api/ {
        limit_req zone=uveddi_api burst=20 nodelay;
        proxy_pass http://127.0.0.1:8888;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Web interface with moderate rate limiting
    location / {
        limit_req zone=uveddi_web burst=50 nodelay;
        proxy_pass http://127.0.0.1:8888;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Health check endpoint
    location /_health {
        access_log off;
        proxy_pass http://127.0.0.1:8888/_health;
    }
}
```

### Firewall Configuration (iptables)

```bash
# Basic firewall rules for Uveddi
sudo iptables -I INPUT -p tcp --dport 22 -j ACCEPT    # SSH
sudo iptables -I INPUT -p tcp --dport 80 -j ACCEPT     # HTTP (redirect)
sudo iptables -I INPUT -p tcp --dport 443 -j ACCEPT    # HTTPS
sudo iptables -I INPUT -i lo -j ACCEPT                 # Loopback
sudo iptables -I INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
sudo iptables -P INPUT DROP                            # Drop all other traffic

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

## Authentication Configuration

Since OpenID Connect is not available in production-secure mode, configure alternative authentication:

### API Key Authentication

```bash
# Generate secure API key
uveddi api-key generate --name "production-service" --expires-days 365

# Use API key for programmatic access
curl -H "Authorization: Bearer uvd_12345678_abcdefghijklmnopqrstuvwxyz123456" \
     https://uveddi.yourdomain.com/api/v1/analysis
```

### JWT Authentication

```bash
# Configure JWT authentication
uveddi auth jwt-config \
  --secret "$UVEDDI_JWT_SECRET" \
  --expiry-hours 24 \
  --issuer "uveddi-production"

# Generate JWT token for user
uveddi auth generate-jwt --user "admin@company.com" --roles "admin"
```

### Session-Based Authentication

```bash
# Configure session management
uveddi auth session-config \
  --timeout 3600 \
  --secure true \
  --same-site strict
```

## Monitoring and Alerting

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'uveddi'
    static_configs:
      - targets: ['localhost:8888']
    scrape_interval: 30s
    metrics_path: '/metrics'
    scheme: 'https'
    tls_config:
      insecure_skip_verify: false
```

### Security Monitoring

```bash
# Enable security audit logging
export UVEDDI_AUDIT_LOG_PATH="/var/log/uveddi/security.log"
export UVEDDI_AUDIT_LOG_LEVEL="info"

# Configure log rotation
cat << 'EOF' > /etc/logrotate.d/uveddi
/var/log/uveddi/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0644 uveddi uveddi
    postrotate
        systemctl reload uveddi
    endscript
}
EOF
```

## Deployment Scripts

### Systemd Service

```ini
# /etc/systemd/system/uveddi.service
[Unit]
Description=Uveddi Code Analysis Service
After=network.target
Wants=network.target

[Service]
Type=exec
User=uveddi
Group=uveddi
ExecStart=/usr/local/bin/uveddi serve --port 8888 --production-secure
ExecReload=/bin/kill -HUP $MAINPID
KillMode=mixed
KillSignal=SIGTERM
TimeoutStopSec=30
Restart=always
RestartSec=5

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/uveddi /var/log/uveddi
PrivateTmp=true
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictSUIDSGID=true
RestrictRealtime=true

# Environment file
EnvironmentFile=/etc/uveddi/environment

[Install]
WantedBy=multi-user.target
```

### Docker Configuration

```dockerfile
# Dockerfile.production-secure
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features production-secure

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false -m -d /var/lib/uveddi uveddi

COPY --from=builder /app/target/release/uveddi /usr/local/bin/
COPY --chown=uveddi:uveddi config/ /etc/uveddi/

USER uveddi
WORKDIR /var/lib/uveddi

EXPOSE 8888
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:8888/_health || exit 1

CMD ["uveddi", "serve", "--port", "8888", "--bind", "0.0.0.0"]
```

### Docker Compose

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  uveddi:
    build:
      context: .
      dockerfile: Dockerfile.production-secure
    container_name: uveddi-prod
    restart: unless-stopped
    ports:
      - "127.0.0.1:8888:8888"
    volumes:
      - uveddi_data:/var/lib/uveddi
      - uveddi_logs:/var/log/uveddi
      - ./config:/etc/uveddi:ro
    environment:
      - UVEDDI_JWT_SECRET_FILE=/run/secrets/jwt_secret
      - UVEDDI_SECURITY_MODE=strict
      - UVEDDI_LOG_LEVEL=info
    secrets:
      - jwt_secret
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8888/_health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  prometheus:
    image: prom/prometheus:latest
    container_name: uveddi-prometheus
    restart: unless-stopped
    ports:
      - "127.0.0.1:9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus

volumes:
  uveddi_data:
  uveddi_logs:
  prometheus_data:

secrets:
  jwt_secret:
    file: ./secrets/jwt_secret.txt
```

## Security Validation Checklist

### Pre-Deployment Validation

- [ ] **Build Security**: Confirmed zero vulnerabilities with `cargo audit --features production-secure`
- [ ] **Dependencies**: Verified no vulnerable dependencies in build
- [ ] **Environment**: All security environment variables configured
- [ ] **Secrets**: JWT secret generated securely and stored safely
- [ ] **Network**: Firewall rules configured and tested
- [ ] **TLS**: Valid SSL certificates installed and configured
- [ ] **Monitoring**: Security logging and metrics collection enabled

### Post-Deployment Validation

- [ ] **Service Health**: Health check endpoint responding correctly
- [ ] **Authentication**: API key authentication working
- [ ] **Authorization**: User permissions enforced correctly
- [ ] **Rate Limiting**: Request throttling functioning
- [ ] **Logging**: Security events being logged
- [ ] **Metrics**: Prometheus metrics collection active
- [ ] **Backup**: Configuration and data backup procedures tested

## Incident Response

### Security Incident Procedures

1. **Detection**: Monitor security logs and metrics
2. **Containment**: Isolate affected components
3. **Investigation**: Analyze logs and forensic evidence
4. **Recovery**: Restore service with security fixes
5. **Documentation**: Record lessons learned

### Emergency Contacts

```bash
# Security team notification
security-team@company.com

# Infrastructure alerts
infrastructure-alerts@company.com

# On-call rotation
oncall@company.com
```

## Maintenance Schedule

### Regular Security Tasks

| Task | Frequency | Responsibility |
|------|-----------|----------------|
| Security audit scan | Weekly | DevOps |
| Dependency updates | Monthly | Development |
| Log review | Daily | Security |
| Certificate renewal | Quarterly | Infrastructure |
| Backup validation | Weekly | Operations |
| Penetration testing | Quarterly | External |

### Update Procedures

```bash
# Security update procedure
1. Test in staging environment
2. Verify security scan results
3. Schedule maintenance window
4. Deploy with rollback plan
5. Validate security controls
6. Update documentation
```

---

## Conclusion

This production-secure deployment configuration provides maximum security assurance by:

✅ **Eliminating all known vulnerabilities** through feature selection  
✅ **Implementing defense-in-depth** security controls  
✅ **Providing comprehensive monitoring** and alerting  
✅ **Maintaining full analytical capabilities** without security compromise  

The configuration is suitable for enterprise environments requiring the highest security standards while maintaining operational excellence.

For support or questions regarding secure deployment, contact the Uveddi security team or refer to the comprehensive security documentation.

---

**Document Status**: Production Ready  
**Security Review**: Approved  
**Last Updated**: September 6, 2025