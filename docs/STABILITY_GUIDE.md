# Uveddi Stability & Operations Guide

This guide provides comprehensive instructions for stable operation, deployment, and troubleshooting of the Uveddi platform.

## 🚀 **Quick Start for Stable Operations**

### Using the Process Manager (Recommended)

The easiest way to manage all Uveddi services:

```bash
# Start all services
./scripts/process-manager.sh start

# Check status
./scripts/process-manager.sh status

# Monitor with auto-restart
./scripts/process-manager.sh monitor

# View logs
./scripts/process-manager.sh logs api    # or frontend, rendering
```

### Service Health Monitoring

```bash
# Start monitoring
./scripts/monitoring.sh start

# Check monitoring status
./scripts/monitoring.sh status

# Generate health report
./scripts/monitoring.sh report
```

## 🛠 **System Architecture**

### Service Dependencies
```
Frontend (Port 8001)
    ↓
API Server (Port 8000)
    ↓
Rendering Service (Port 3001)
```

### Process Management Flow
1. **Rendering Service** starts first (provides diagrams)
2. **API Server** starts next (depends on rendering service)
3. **Frontend** starts last (depends on API server)

## 📋 **Common Operations**

### Starting Services

**Option 1: Process Manager (Recommended)**
```bash
./scripts/process-manager.sh start
```

**Option 2: Manual Startup**
```bash
# Terminal 1 - Rendering Service
cd rendering-service && npm start

# Terminal 2 - API Server  
cd api-server && node server.js

# Terminal 3 - Frontend
cd frontend && npm run dev
```

**Option 3: Docker Compose**
```bash
docker compose -f deploy/docker-compose.dev.yml up
```

### Stopping Services

```bash
# Graceful shutdown
./scripts/process-manager.sh stop

# Force stop if needed
pkill -f "node.*server.js"
pkill -f "npm.*dev"
```

### Health Checking

```bash
# Check all services
curl http://localhost:8000/health    # API
curl http://localhost:8001          # Frontend  
curl http://localhost:3001/health   # Rendering

# Using process manager
./scripts/process-manager.sh health
```

## 🔧 **Troubleshooting**

### Port Conflicts

**Problem**: "EADDRINUSE: address already in use"

**Solution**:
```bash
# Find what's using the port
lsof -i :8000
ss -tulpn | grep :8000

# Kill the process
kill <PID>

# Or use the process manager
./scripts/process-manager.sh stop
./scripts/process-manager.sh start
```

### Service Crashes

**Problem**: Services randomly stopping

**Solutions**:
1. **Use monitoring for auto-restart**:
   ```bash
   ./scripts/monitoring.sh start
   ```

2. **Check logs for errors**:
   ```bash
   ./scripts/process-manager.sh logs api
   tail -f logs/api.log
   ```

3. **Use systemd for production**:
   ```bash
   sudo cp systemd/uveddi.service /etc/systemd/system/
   sudo systemctl enable uveddi
   sudo systemctl start uveddi
   ```

### Memory Issues

**Problem**: High memory usage or OOM kills

**Solutions**:
1. **Monitor memory usage**:
   ```bash
   ./scripts/monitoring.sh status
   ```

2. **Restart services periodically**:
   ```bash
   ./scripts/process-manager.sh restart
   ```

3. **Configure resource limits** (Docker):
   ```yaml
   deploy:
     resources:
       limits:
         memory: 2G
   ```

### Network Connectivity

**Problem**: CORS errors or connection refused

**Solutions**:
1. **Check CORS configuration** in `api-server/server.js`:
   ```javascript
   const corsOptions = {
     origin: ['http://localhost:8001', 'http://127.0.0.1:8001'],
     credentials: true
   };
   ```

2. **Verify service health**:
   ```bash
   ./scripts/process-manager.sh health
   ```

3. **Check firewall/network**:
   ```bash
   sudo ufw status
   netstat -tulpn | grep -E ":(8000|8001|3001)"
   ```

## 🚢 **Deployment**

### Development Deployment

```bash
# Simple restart
./scripts/process-manager.sh restart

# Full rebuild and restart
cd frontend && npm run build
./scripts/process-manager.sh restart
```

### Production Deployment

```bash
# Automated deployment with validation
./scripts/deploy.sh

# Manual steps
./scripts/deploy.sh --skip-tests --no-auto-rollback
```

### Rollback

```bash
# Automatic rollback (if deployment fails)
# This happens automatically with ./scripts/deploy.sh

# Manual rollback
./scripts/deploy.sh --rollback
```

## 📊 **Monitoring & Alerting**

### Setting Up Monitoring

```bash
# Start continuous monitoring
./scripts/monitoring.sh start

# Set up webhook alerts (optional)
export ALERT_WEBHOOK="https://hooks.slack.com/your-webhook"
./scripts/monitoring.sh start
```

### Monitoring Dashboard

```bash
# Generate HTML report
./scripts/monitoring.sh report

# View report
open monitoring/report.html  # macOS
xdg-open monitoring/report.html  # Linux
```

### Key Metrics to Watch

1. **Response Times**:
   - API: < 2 seconds normal, > 5 seconds problematic
   - Frontend: < 1 second normal, > 3 seconds problematic

2. **Memory Usage**:
   - < 70% normal, > 90% critical

3. **Disk Usage**:
   - < 80% normal, > 95% critical

4. **Service Availability**:
   - 99.9% uptime target

## 🐳 **Docker Operations**

### Development Environment

```bash
# Start development stack
docker compose -f deploy/docker-compose.dev.yml up

# View logs
docker compose -f deploy/docker-compose.dev.yml logs -f

# Rebuild after changes
docker compose -f deploy/docker-compose.dev.yml up --build
```

### Production Environment

```bash
# Production deployment
docker compose -f deploy/docker-compose.yml up -d

# Health check
docker-compose ps
docker-compose exec uveddi curl http://localhost:8080/health
```

### Docker Troubleshooting

```bash
# Container logs
docker logs uveddi_api-server_1

# Enter container
docker exec -it uveddi_api-server_1 /bin/bash

# Resource usage
docker stats
```

## 🔐 **Security Considerations**

### Service Security

1. **Run services as non-root user**
2. **Configure proper file permissions**:
   ```bash
   chmod 750 scripts/*.sh
   chown -R uveddi:uveddi /opt/uveddi
   ```

3. **Use HTTPS in production**:
   ```bash
   # Configure nginx reverse proxy
   sudo cp nginx/nginx.conf /etc/nginx/sites-available/uveddi
   sudo ln -s /etc/nginx/sites-available/uveddi /etc/nginx/sites-enabled/
   ```

### Network Security

1. **Firewall configuration**:
   ```bash
   sudo ufw allow 80,443/tcp    # HTTP/HTTPS
   sudo ufw allow 22/tcp        # SSH
   sudo ufw deny 8000,8001,3001/tcp  # Block direct access
   ```

2. **API rate limiting** (already configured in server.js)

## 📁 **File Structure**

```
uveddi/
├── scripts/
│   ├── process-manager.sh     # Unified service management
│   ├── deploy.sh             # Automated deployment
│   ├── monitoring.sh         # Health monitoring
│   └── ...
├── systemd/
│   └── uveddi.service        # Systemd service configuration
├── deploy/
│   ├── docker-compose.dev.yml  # Development environment
│   └── docker-compose.yml      # Production environment
├── logs/                     # Service logs
├── monitoring/               # Monitoring data and reports
├── .pids/                    # Process ID files
└── ...
```

## 🆘 **Emergency Procedures**

### Complete System Restart

```bash
# Stop everything
./scripts/process-manager.sh stop
pkill -f node  # Force kill if needed

# Clear PIDs and locks
rm -f .pids/*.pid /tmp/uveddi_*.lock

# Restart
./scripts/process-manager.sh start
```

### Service Recovery

```bash
# Check what's broken
./scripts/monitoring.sh status

# Restart specific service
./scripts/process-manager.sh restart api

# Full system health check
./scripts/process-manager.sh health
```

### Data Recovery

```bash
# List available backups
ls -la /opt/uveddi/backups/

# Restore from backup
sudo cp /opt/uveddi/backups/deploy-XXXXXX/uveddi.db ./
./scripts/process-manager.sh restart
```

## 📞 **Getting Help**

### Log Locations

- **Process Manager**: `logs/*.log`
- **System Logs**: `journalctl -u uveddi -f`
- **Docker Logs**: `docker-compose logs`
- **Alert Logs**: `logs/alerts.log`

### Diagnostic Commands

```bash
# System overview
./scripts/monitoring.sh status

# Service details
./scripts/process-manager.sh status

# Resource usage
top
free -h
df -h

# Network status
ss -tulpn | grep -E ":(8000|8001|3001)"
```

### Support Information

When reporting issues, include:

1. **Error messages** from logs
2. **System status** output
3. **Reproduction steps**
4. **Environment details** (OS, Node version, etc.)

---

## 📋 **Quick Reference Commands**

```bash
# Essential commands for daily operations

# Start everything
./scripts/process-manager.sh start

# Check status  
./scripts/process-manager.sh status

# Monitor continuously
./scripts/monitoring.sh start

# Deploy changes
./scripts/deploy.sh

# Emergency restart
./scripts/process-manager.sh stop && ./scripts/process-manager.sh start

# View logs
./scripts/process-manager.sh logs api
```

This stability guide ensures reliable operation of Uveddi with minimal manual intervention and quick recovery from any issues.
