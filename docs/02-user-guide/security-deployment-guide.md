# Security System Deployment Guide

## Overview

This guide provides step-by-step instructions for deploying the Uveddi Security and RBAC system (UV-247) in production environments. The security system is enterprise-ready with 100% test coverage and SOC 2/ISO 27001 compliance features.

## Pre-Deployment Requirements

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+ or RHEL 8+)
- **Memory**: Minimum 2GB RAM, Recommended 4GB+
- **CPU**: Minimum 2 cores, Recommended 4+ cores
- **Storage**: Minimum 10GB, Recommended 50GB+ for audit logs
- **Network**: HTTPS/TLS 1.2+ required for production

### Dependencies

- **Database**: PostgreSQL 13+ (for audit logging and user management)
- **Cache**: Redis 6+ (for distributed rate limiting and session storage)
- **TLS Certificates**: Valid SSL/TLS certificates for HTTPS
- **Identity Provider**: OAuth 2.0/OIDC provider (optional but recommended)

## Deployment Steps

### Step 1: Environment Preparation

#### 1.1 Create Deployment User
```bash
# Create dedicated user for security service
sudo useradd -r -s /bin/false -d /opt/uveddi uveddi
sudo mkdir -p /opt/uveddi/{bin,config,logs,data}
sudo chown -R uveddi:uveddi /opt/uveddi
```

#### 1.2 Install System Dependencies
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y postgresql-client redis-tools openssl curl

# RHEL/CentOS
sudo yum install -y postgresql redis openssl curl
```

#### 1.3 Configure Firewall
```bash
# Allow HTTPS traffic
sudo ufw allow 443/tcp
sudo ufw allow 80/tcp  # For Let's Encrypt if needed

# Allow internal service communication
sudo ufw allow from 10.0.0.0/8 to any port 8080  # Adjust for your network
```

### Step 2: Database Setup

#### 2.1 Create Security Database
```sql
-- Connect as PostgreSQL superuser
CREATE DATABASE uveddi_security;
CREATE USER uveddi_security WITH PASSWORD 'secure_random_password';
GRANT ALL PRIVILEGES ON DATABASE uveddi_security TO uveddi_security;

-- Connect to uveddi_security database
\c uveddi_security;

-- Create audit schema
CREATE SCHEMA audit;
GRANT ALL ON SCHEMA audit TO uveddi_security;
```

#### 2.2 Run Security Migrations
```bash
# Copy migration files
sudo cp migrations/V5__security_schema.sql /opt/uveddi/
sudo chown uveddi:uveddi /opt/uveddi/V5__security_schema.sql

# Run migration
sudo -u uveddi psql -h localhost -U uveddi_security -d uveddi_security -f /opt/uveddi/V5__security_schema.sql
```

#### 2.3 Create Database Indexes
```sql
-- Performance indexes for audit queries
CREATE INDEX CONCURRENTLY idx_audit_events_timestamp ON audit_events(timestamp);
CREATE INDEX CONCURRENTLY idx_audit_events_user_id ON audit_events(user_id);
CREATE INDEX CONCURRENTLY idx_audit_events_event_type ON audit_events(event_type);
CREATE INDEX CONCURRENTLY idx_audit_events_outcome ON audit_events(outcome);

-- Composite index for common queries
CREATE INDEX CONCURRENTLY idx_audit_events_composite 
ON audit_events(user_id, event_type, timestamp);

-- Index for session management
CREATE INDEX CONCURRENTLY idx_sessions_user_id ON sessions(user_id);
CREATE INDEX CONCURRENTLY idx_sessions_expires_at ON sessions(expires_at);

-- Index for API keys
CREATE INDEX CONCURRENTLY idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX CONCURRENTLY idx_api_keys_expires_at ON api_keys(expires_at);
```

### Step 3: Redis Configuration

#### 3.1 Configure Redis for Security
```bash
# Edit Redis configuration
sudo nano /etc/redis/redis.conf

# Add security configurations:
# requirepass your_redis_password
# maxmemory 1gb
# maxmemory-policy allkeys-lru
# save 900 1
# save 300 10
# save 60 10000

# Restart Redis
sudo systemctl restart redis
sudo systemctl enable redis
```

#### 3.2 Test Redis Connection
```bash
redis-cli -a your_redis_password ping
# Should return: PONG
```

### Step 4: Security Configuration

#### 4.1 Generate Secure Secrets
```bash
# Generate JWT secret (32+ characters)
JWT_SECRET=$(openssl rand -base64 32)
echo "JWT Secret: $JWT_SECRET"

# Generate API encryption key
API_KEY=$(openssl rand -base64 32)
echo "API Key: $API_KEY"

# Generate audit integrity key
AUDIT_KEY=$(openssl rand -base64 32)
echo "Audit Key: $AUDIT_KEY"
```

#### 4.2 Create Production Configuration
```bash
# Create configuration directory
sudo mkdir -p /opt/uveddi/config/security
sudo chown uveddi:uveddi /opt/uveddi/config/security
```

Create `/opt/uveddi/config/security/production.toml`:
```toml
[authentication]
jwt_secret = "YOUR_JWT_SECRET_HERE"
jwt_expiry_hours = 8
jwt_issuer = "uveddi-production"
jwt_audience = "uveddi-api"
session_timeout_hours = 24

[authorization]
enabled = true
cache_enabled = true
cache_ttl_seconds = 300
cache_max_entries = 10000

[audit]
enabled = true
store_type = "database"
integrity_verification = true
retention_days = 2555  # 7 years for compliance

[audit.database]
url = "postgresql://uveddi_security:PASSWORD@localhost:5432/uveddi_security"
table_name = "audit_events"
connection_pool_size = 10

[rate_limiting]
enabled = true
default_rate_limit = 1000
window_seconds = 60
burst_size = 100
storage_type = "redis"

[rate_limiting.redis]
url = "redis://:PASSWORD@localhost:6379/0"
key_prefix = "uveddi:ratelimit:"
max_connections = 20

# Endpoint-specific rate limits
[[rate_limiting.endpoint_limits]]
endpoint = "/api/auth/login"
rate_limit = 5
window_seconds = 300

[[rate_limiting.endpoint_limits]]
endpoint = "/api/admin/*"
rate_limit = 100
window_seconds = 60

[monitoring]
metrics_enabled = true
metrics_endpoint = "/metrics"
health_check_endpoint = "/health"

[security]
cors_allowed_origins = ["https://your-frontend-domain.com"]
cors_allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
cors_allowed_headers = ["Authorization", "Content-Type", "X-Requested-With"]
```

#### 4.3 Set File Permissions
```bash
sudo chown uveddi:uveddi /opt/uveddi/config/security/production.toml
sudo chmod 600 /opt/uveddi/config/security/production.toml
```

### Step 5: Application Deployment

#### 5.1 Deploy Application Binary
```bash
# Copy application binary
sudo cp target/release/uveddi /opt/uveddi/bin/
sudo chown uveddi:uveddi /opt/uveddi/bin/uveddi
sudo chmod 755 /opt/uveddi/bin/uveddi
```

#### 5.2 Create Systemd Service
Create `/etc/systemd/system/uveddi-security.service`:
```ini
[Unit]
Description=Uveddi Security Service
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=uveddi
Group=uveddi
WorkingDirectory=/opt/uveddi
ExecStart=/opt/uveddi/bin/uveddi --config /opt/uveddi/config/security/production.toml
Restart=always
RestartSec=10

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/uveddi/logs /opt/uveddi/data

# Environment variables
Environment=RUST_LOG=info
Environment=UVEDDI_SECURITY_AUTHENTICATION_JWT_SECRET=YOUR_JWT_SECRET
Environment=UVEDDI_SECURITY_AUDIT_DATABASE_URL=postgresql://uveddi_security:PASSWORD@localhost:5432/uveddi_security
Environment=UVEDDI_SECURITY_RATE_LIMITING_REDIS_URL=redis://:PASSWORD@localhost:6379/0

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

#### 5.3 Enable and Start Service
```bash
sudo systemctl daemon-reload
sudo systemctl enable uveddi-security
sudo systemctl start uveddi-security

# Check status
sudo systemctl status uveddi-security
```

### Step 6: TLS/SSL Configuration

#### 6.1 Obtain SSL Certificate
```bash
# Using Let's Encrypt (recommended)
sudo apt install certbot
sudo certbot certonly --standalone -d your-domain.com

# Or use your existing certificates
sudo mkdir -p /opt/uveddi/ssl
sudo cp your-cert.pem /opt/uveddi/ssl/
sudo cp your-key.pem /opt/uveddi/ssl/
sudo chown -R uveddi:uveddi /opt/uveddi/ssl
sudo chmod 600 /opt/uveddi/ssl/*
```

#### 6.2 Configure Reverse Proxy (Nginx)
Create `/etc/nginx/sites-available/uveddi-security`:
```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    
    # SSL Security
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

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=auth:10m rate=1r/s;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /api/auth/ {
        limit_req zone=auth burst=5 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://127.0.0.1:8080;
        access_log off;
    }
}
```

Enable the site:
```bash
sudo ln -s /etc/nginx/sites-available/uveddi-security /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Step 7: Monitoring and Logging

#### 7.1 Configure Log Rotation
Create `/etc/logrotate.d/uveddi-security`:
```
/opt/uveddi/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 uveddi uveddi
    postrotate
        systemctl reload uveddi-security
    endscript
}
```

#### 7.2 Set Up Monitoring
```bash
# Install monitoring tools
sudo apt install prometheus-node-exporter

# Configure Prometheus scraping
# Add to /etc/prometheus/prometheus.yml:
# - job_name: 'uveddi-security'
#   static_configs:
#   - targets: ['localhost:8080']
#   metrics_path: '/metrics'
```

#### 7.3 Configure Alerting
Create monitoring alerts for:
- High authentication failure rates
- Authorization denial spikes
- Rate limiting threshold breaches
- Database connection failures
- Redis connectivity issues

### Step 8: Backup and Recovery

#### 8.1 Database Backup
```bash
# Create backup script
cat > /opt/uveddi/backup-security-db.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/opt/uveddi/backups"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# Backup security database
pg_dump -h localhost -U uveddi_security -d uveddi_security > $BACKUP_DIR/security_db_$DATE.sql

# Compress backup
gzip $BACKUP_DIR/security_db_$DATE.sql

# Remove backups older than 30 days
find $BACKUP_DIR -name "security_db_*.sql.gz" -mtime +30 -delete
EOF

chmod +x /opt/uveddi/backup-security-db.sh
chown uveddi:uveddi /opt/uveddi/backup-security-db.sh
```

#### 8.2 Schedule Backups
```bash
# Add to crontab for uveddi user
sudo -u uveddi crontab -e

# Add this line for daily backups at 2 AM
0 2 * * * /opt/uveddi/backup-security-db.sh
```

### Step 9: Security Hardening

#### 9.1 System Hardening
```bash
# Disable unnecessary services
sudo systemctl disable apache2 2>/dev/null || true
sudo systemctl disable sendmail 2>/dev/null || true

# Configure fail2ban for SSH protection
sudo apt install fail2ban
sudo systemctl enable fail2ban
sudo systemctl start fail2ban
```

#### 9.2 Application Security
```bash
# Set up audit logging for system access
echo "audit:x:999:uveddi" | sudo tee -a /etc/group
sudo usermod -a -G audit uveddi

# Configure audit rules
echo "-w /opt/uveddi/config/ -p wa -k uveddi-config" | sudo tee -a /etc/audit/rules.d/uveddi.rules
sudo systemctl restart auditd
```

### Step 10: Validation and Testing

#### 10.1 Health Check
```bash
# Test service health
curl -k https://your-domain.com/health

# Expected response:
# {"status": "healthy", "timestamp": "2024-01-15T10:30:00Z"}
```

#### 10.2 Authentication Test
```bash
# Test JWT authentication
curl -X POST https://your-domain.com/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "test@example.com", "password": "test_password"}'

# Should return JWT token
```

#### 10.3 Rate Limiting Test
```bash
# Test rate limiting
for i in {1..10}; do
  curl -w "%{http_code}\n" -o /dev/null -s https://your-domain.com/api/auth/login
done

# Should show 429 (Too Many Requests) after limit exceeded
```

#### 10.4 Security Headers Test
```bash
# Test security headers
curl -I https://your-domain.com/

# Should include:
# Strict-Transport-Security: max-age=31536000; includeSubDomains
# X-Content-Type-Options: nosniff
# X-Frame-Options: DENY
```

## Post-Deployment Checklist

### Security Verification
- [ ] All secrets are properly configured and secured
- [ ] Database connections are encrypted
- [ ] Redis is password-protected
- [ ] TLS/SSL certificates are valid and properly configured
- [ ] Security headers are present in all responses
- [ ] Rate limiting is functioning correctly
- [ ] Audit logging is capturing all security events

### Performance Verification
- [ ] Authentication latency < 50ms
- [ ] Authorization checks < 10ms
- [ ] Database queries are optimized with proper indexes
- [ ] Redis connections are pooled and efficient
- [ ] Memory usage is within expected limits

### Monitoring Verification
- [ ] Health checks are responding correctly
- [ ] Metrics are being collected
- [ ] Log rotation is configured
- [ ] Backup scripts are working
- [ ] Alerting is configured for critical events

### Compliance Verification
- [ ] Audit logs include all required security events
- [ ] Data retention policies are configured
- [ ] Access controls are properly enforced
- [ ] Security configurations meet compliance requirements

## Troubleshooting

### Common Issues

#### Service Won't Start
```bash
# Check service logs
sudo journalctl -u uveddi-security -f

# Check configuration
sudo -u uveddi /opt/uveddi/bin/uveddi --config /opt/uveddi/config/security/production.toml --validate-config
```

#### Database Connection Issues
```bash
# Test database connection
sudo -u uveddi psql -h localhost -U uveddi_security -d uveddi_security -c "SELECT 1;"

# Check database logs
sudo tail -f /var/log/postgresql/postgresql-*.log
```

#### Redis Connection Issues
```bash
# Test Redis connection
redis-cli -a your_redis_password ping

# Check Redis logs
sudo tail -f /var/log/redis/redis-server.log
```

### Performance Issues

#### High Memory Usage
```bash
# Check memory usage
ps aux | grep uveddi
free -h

# Adjust configuration if needed
# Reduce cache sizes, connection pools
```

#### Slow Database Queries
```sql
-- Check slow queries
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;

-- Add missing indexes if needed
```

## Maintenance

### Regular Maintenance Tasks

#### Weekly
- Review security logs for anomalies
- Check system resource usage
- Verify backup integrity
- Update security patches

#### Monthly
- Rotate JWT secrets (if required by policy)
- Review and clean up old audit logs
- Performance optimization review
- Security configuration audit

#### Quarterly
- Full security audit
- Penetration testing
- Compliance review
- Disaster recovery testing

---

This deployment guide ensures a secure, production-ready deployment of the Uveddi Security and RBAC system with enterprise-grade security, monitoring, and compliance features.