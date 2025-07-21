# Uveddi Incident Response Runbooks

## Table of Contents

1. [Overview](#overview)
2. [High Memory Usage Alert](#high-memory-usage-alert)
3. [High CPU Usage Alert](#high-cpu-usage-alert)
4. [Database Connection Issues](#database-connection-issues)
5. [WebSocket Connection Failures](#websocket-connection-failures)
6. [Analysis Queue Backup](#analysis-queue-backup)
7. [AI Service Unavailable](#ai-service-unavailable)
8. [Disk Space Critical](#disk-space-critical)
9. [Service Startup Failures](#service-startup-failures)
10. [Performance Degradation](#performance-degradation)
11. [Security Incidents](#security-incidents)
12. [Escalation Procedures](#escalation-procedures)

## Overview

This document provides step-by-step procedures for responding to common incidents in the Uveddi system. Each runbook follows a consistent format:

1. **Alert Description**: What triggered the alert
2. **Immediate Actions**: First steps to take (< 5 minutes)
3. **Investigation Steps**: Detailed diagnostic procedures (5-15 minutes)
4. **Resolution Actions**: How to fix the issue
5. **Prevention Measures**: How to prevent recurrence
6. **Escalation Criteria**: When to escalate

### Severity Levels

- 🔴 **Critical (P1)**: Service unavailable, data loss risk, security breach
- 🟡 **High (P2)**: Degraded performance, partial outage, resource exhaustion
- 🔵 **Medium (P3)**: Minor issues, configuration problems, capacity warnings
- ⚪ **Low (P4)**: Maintenance, informational alerts, optimization opportunities

## High Memory Usage Alert

**Severity**: 🟡 High (P2)  
**Alert Threshold**: Memory usage > 85% of available system memory

### Alert Description
Memory usage has exceeded the configured threshold during analysis operations, potentially leading to system instability or out-of-memory conditions.

### Immediate Actions (< 5 minutes)

#### 1. Assess Current Memory State
```bash
# Check current memory usage
free -h
htop -d 1

# Identify top memory consumers
ps aux --sort=-%mem | head -10

# Check for Uveddi processes specifically
pgrep -f uveddi | xargs ps -o pid,ppid,cmd,%mem,%cpu

# Check system memory pressure
cat /proc/pressure/memory
```

#### 2. Quick System Health Check
```bash
# Check system load
uptime

# Check for swap usage
swapon --show
cat /proc/swaps

# Check kernel messages for OOM killer activity
dmesg | grep -i "killed process\|out of memory"

# Check systemd journal for memory-related errors
journalctl -u uveddi --since "10 minutes ago" | grep -i "memory\|oom"
```

#### 3. Emergency Memory Relief (if critical)
```bash
# If memory usage > 95%, consider emergency actions:

# Reduce analysis concurrency
curl -X POST http://localhost:8080/api/v1/admin/config \
  -H "Content-Type: application/json" \
  -d '{"analysis": {"max_concurrent_analyses": 2}}'

# Cancel non-critical running analyses
curl -X POST http://localhost:8080/api/v1/admin/analyses/cancel \
  -d '{"priority": "low"}'

# Clear analysis cache if safe
curl -X POST http://localhost:8080/api/v1/admin/cache/clear \
  -d '{"cache_type": "analysis_results"}'
```

### Investigation Steps (5-15 minutes)

#### 1. Analyze Memory Usage Patterns
```bash
# Check historical memory usage
curl http://localhost:9090/api/v1/query?query=process_resident_memory_bytes[1h]

# Review current analysis workload
curl http://localhost:8080/api/v1/analysis/queue/status

# Check for memory leaks in metrics
curl http://localhost:9090/metrics | grep memory | sort -k2 -nr

# Analyze garbage collection metrics
curl http://localhost:9090/metrics | grep gc_duration
```

#### 2. Review Recent System Changes
```bash
# Check recent deployments
systemctl status uveddi | head -20

# Review configuration changes
diff /etc/uveddi/config.toml /etc/uveddi/config.toml.backup

# Check log files for errors
tail -1000 /var/log/uveddi/uveddi.log | grep -i "error\|warn\|panic"

# Check for recent large analysis jobs
curl http://localhost:8080/api/v1/analysis/history?limit=50 | \
  jq '.analyses[] | select(.memory_usage_mb > 2000)'
```

#### 3. Identify Root Cause
```bash
# Check if it's a specific analysis causing the issue
curl http://localhost:8080/api/v1/analysis/active | \
  jq '.analyses[] | {id, project_path, memory_usage_mb, duration_s}'

# Look for memory-intensive operations
ps -eo pid,ppid,cmd,pmem --sort=-pmem | head -20

# Check for plugin memory usage
curl http://localhost:8080/api/v1/plugins/stats | \
  jq '.plugins[] | select(.memory_usage_mb > 100)'

# Review AI service memory consumption
curl http://localhost:11434/api/stats 2>/dev/null || echo "Ollama not accessible"
```

### Resolution Actions

#### If Memory Leak Detected
```bash
# Graceful service restart (preserves analysis queue)
systemctl reload uveddi

# If unresponsive, force restart
systemctl restart uveddi

# Monitor memory after restart
watch -n 5 'free -h && ps -eo pid,cmd,pmem --sort=-pmem | head -10'
```

#### If Large Analysis Causing Issue
```bash
# Pause the problematic analysis
ANALYSIS_ID="analysis_id_here"
curl -X POST http://localhost:8080/api/v1/analysis/$ANALYSIS_ID/pause

# Move to low-memory analysis mode
curl -X POST http://localhost:8080/api/v1/admin/config \
  -H "Content-Type: application/json" \
  -d '{
    "analysis": {
      "streaming_mode": true,
      "batch_size": 10,
      "memory_limit_mb": 2048
    }
  }'

# Resume analysis with new settings
curl -X POST http://localhost:8080/api/v1/analysis/$ANALYSIS_ID/resume
```

#### If System Resource Exhaustion
```bash
# Scale horizontally (if load balancer available)
# Add additional Uveddi instances or
# Redirect traffic to other nodes

# Scale vertically (if possible)
# Increase system memory or
# Move to larger instance

# Enable swap if not available (temporary measure)
sudo dd if=/dev/zero of=/swapfile bs=1024 count=2097152
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### Prevention Measures

#### 1. Monitoring and Alerting
```bash
# Set up graded memory alerts
# 70% - Warning
# 80% - High
# 85% - Critical

# Configure alert rules in prometheus.yml
cat >> /etc/prometheus/uveddi_alerts.yml << EOF
- alert: MemoryUsageHigh
  expr: (1 - (node_memory_available_bytes / node_memory_total_bytes)) * 100 > 70
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High memory usage detected"

- alert: MemoryUsageCritical
  expr: (1 - (node_memory_available_bytes / node_memory_total_bytes)) * 100 > 85
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "Critical memory usage"
EOF
```

#### 2. Configuration Optimization
```toml
# /etc/uveddi/config.toml - Memory optimization
[analysis]
streaming_mode = true
memory_limit_mb = 4096  # Set based on system capacity
batch_size = 25         # Reduce for large projects

[cache]
max_size_mb = 1024      # Limit cache size
enable_compression = true

[performance]
gc_threshold_mb = 2048  # Trigger GC earlier
enable_memory_monitoring = true
```

#### 3. System Tuning
```bash
# Configure Linux memory overcommit
echo "vm.overcommit_memory = 1" >> /etc/sysctl.conf
echo "vm.swappiness = 10" >> /etc/sysctl.conf
sysctl -p

# Set up memory limits for systemd service
cat >> /etc/systemd/system/uveddi.service << EOF
[Service]
MemoryMax=6G
MemoryHigh=5G
OOMPolicy=continue
EOF

systemctl daemon-reload
```

---

## High CPU Usage Alert

**Severity**: 🟡 High (P2)  
**Alert Threshold**: CPU usage > 80% for more than 5 minutes

### Alert Description
CPU usage has remained consistently high, potentially causing performance degradation and increased response times.

### Immediate Actions (< 5 minutes)

#### 1. Assess CPU Load
```bash
# Check current CPU usage
top -n1 | head -5
htop -d 1

# Check system load average
uptime

# Identify CPU-intensive processes
ps aux --sort=-%cpu | head -10

# Check Uveddi-specific CPU usage
pgrep -f uveddi | xargs ps -o pid,ppid,cmd,%cpu,time
```

#### 2. Quick System Analysis
```bash
# Check CPU frequency scaling
cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq

# Check for thermal throttling
sensors | grep -i temp || echo "lm-sensors not installed"

# Check interrupt load
cat /proc/interrupts | head -10

# Check context switches
vmstat 1 3
```

#### 3. Emergency CPU Relief
```bash
# Reduce analysis concurrency
curl -X POST http://localhost:8080/api/v1/admin/config \
  -H "Content-Type: application/json" \
  -d '{"analysis": {"max_concurrent_analyses": 1, "parallel_processing": false}}'

# Pause low-priority analyses
curl -X GET http://localhost:8080/api/v1/analysis/active | \
  jq -r '.analyses[] | select(.priority == "low") | .id' | \
  xargs -I {} curl -X POST http://localhost:8080/api/v1/analysis/{}/pause
```

### Investigation Steps (5-15 minutes)

#### 1. Analyze CPU Usage Patterns
```bash
# Check historical CPU metrics
curl http://localhost:9090/api/v1/query?query=rate(process_cpu_seconds_total[5m])*100

# Review analysis queue and active jobs
curl http://localhost:8080/api/v1/analysis/queue/detailed

# Check for CPU-intensive detectors
curl http://localhost:8080/api/v1/metrics/detectors | \
  jq '.detectors[] | {name, cpu_time_ms, executions}' | \
  sort -k2 -nr

# Monitor specific thread usage
top -H -p $(pgrep uveddi | head -1)
```

#### 2. Identify Performance Bottlenecks
```bash
# Check for inefficient algorithms in logs
grep -i "performance\|slow\|timeout" /var/log/uveddi/uveddi.log | tail -50

# Analyze analysis complexity
curl http://localhost:8080/api/v1/analysis/active | \
  jq '.analyses[] | {id, files_count, estimated_complexity, cpu_usage_percent}'

# Check plugin CPU usage
curl http://localhost:8080/api/v1/plugins/performance | \
  jq '.plugins[] | select(.cpu_usage_percent > 20)'
```

### Resolution Actions

#### If Single Analysis Consuming CPU
```bash
# Identify the heavy analysis
HIGH_CPU_ANALYSIS=$(curl -s http://localhost:8080/api/v1/analysis/active | \
  jq -r '.analyses[] | select(.cpu_usage_percent > 50) | .id' | head -1)

# Move to background priority
curl -X POST http://localhost:8080/api/v1/analysis/$HIGH_CPU_ANALYSIS/priority \
  -d '{"priority": "background"}'

# Enable CPU throttling for this analysis
curl -X POST http://localhost:8080/api/v1/analysis/$HIGH_CPU_ANALYSIS/config \
  -d '{"cpu_limit_percent": 25}'
```

#### If System-Wide High CPU Usage
```bash
# Enable CPU throttling globally
curl -X POST http://localhost:8080/api/v1/admin/config \
  -H "Content-Type: application/json" \
  -d '{
    "performance": {
      "cpu_throttling_enabled": true,
      "max_cpu_percent": 70,
      "thread_pool_size": 4
    }
  }'

# Restart with CPU limits
systemctl edit uveddi
# Add:
# [Service]
# CPUQuota=70%
# CPUWeight=100

systemctl restart uveddi
```

#### If Algorithmic Performance Issue
```bash
# Switch to optimized detection algorithms
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{
    "detectors": {
      "use_optimized_algorithms": true,
      "enable_early_termination": true,
      "complexity_threshold": "medium"
    }
  }'

# Enable analysis result caching
curl -X POST http://localhost:8080/api/v1/admin/cache/config \
  -d '{"enable_result_caching": true, "cache_ttl_hours": 24}'
```

### Prevention Measures

#### 1. CPU Monitoring and Alerting
```yaml
# Prometheus alert rules
- alert: HighCPUUsage
  expr: rate(process_cpu_seconds_total[5m]) * 100 > 70
  for: 5m
  labels:
    severity: warning

- alert: CPUSaturation
  expr: rate(process_cpu_seconds_total[5m]) * 100 > 90
  for: 2m
  labels:
    severity: critical
```

#### 2. Performance Optimization
```toml
# Optimized configuration
[performance]
worker_threads = 8              # Match CPU cores
enable_cpu_affinity = true
cpu_usage_target = 70          # Target utilization
enable_adaptive_scheduling = true

[analysis]
enable_progressive_analysis = true
complexity_based_scheduling = true
early_termination_enabled = true
```

---

## Database Connection Issues

**Severity**: 🔴 Critical (P1)  
**Alert Description**: Unable to connect to the database or connection pool exhausted

### Immediate Actions (< 5 minutes)

#### 1. Check Database Status
```bash
# Check PostgreSQL service
systemctl status postgresql
sudo -u postgres pg_isready -h localhost -p 5432

# For Docker deployments
docker ps | grep postgres
docker logs uveddi-postgres --tail 50

# Check connection from application server
psql -h localhost -U uveddi -d uveddi_prod -c "SELECT 1;" || echo "Connection failed"
```

#### 2. Assess Connection Pool
```bash
# Check active connections
sudo -u postgres psql -c "
SELECT count(*) as active_connections 
FROM pg_stat_activity 
WHERE state = 'active';"

# Check connection pool status
curl http://localhost:8080/api/v1/admin/database/stats

# Check for connection leaks
sudo -u postgres psql -c "
SELECT pid, usename, application_name, client_addr, state, query_start 
FROM pg_stat_activity 
WHERE datname = 'uveddi_prod';"
```

#### 3. Emergency Actions
```bash
# Kill idle connections (if pool exhausted)
sudo -u postgres psql -c "
SELECT pg_terminate_backend(pid) 
FROM pg_stat_activity 
WHERE datname = 'uveddi_prod' 
  AND state = 'idle' 
  AND query_start < now() - interval '10 minutes';"

# Restart database connection pool
curl -X POST http://localhost:8080/api/v1/admin/database/pool/restart

# If critical, restart Uveddi service
systemctl restart uveddi
```

### Investigation Steps (5-15 minutes)

#### 1. Analyze Database Logs
```bash
# Check PostgreSQL logs
sudo tail -100 /var/log/postgresql/postgresql-15-main.log | grep -i "error\|fatal\|connection"

# Check for authentication issues
grep -i "authentication failed" /var/log/postgresql/postgresql-15-main.log

# Check for resource exhaustion
grep -i "too many connections\|out of memory" /var/log/postgresql/postgresql-15-main.log
```

#### 2. Review Connection Configuration
```bash
# Check PostgreSQL configuration
sudo -u postgres psql -c "SHOW max_connections;"
sudo -u postgres psql -c "SHOW shared_buffers;"

# Check application configuration
grep -A5 -B5 "database" /etc/uveddi/config.toml

# Verify network connectivity
telnet localhost 5432
nc -zv localhost 5432
```

### Resolution Actions

#### If Database Service Down
```bash
# Start PostgreSQL service
systemctl start postgresql

# If start fails, check disk space and permissions
df -h /var/lib/postgresql/
ls -la /var/lib/postgresql/15/main/

# Check for corruption
sudo -u postgres pg_controldata /var/lib/postgresql/15/main/
```

#### If Connection Pool Exhausted
```bash
# Increase connection limits temporarily
sudo -u postgres psql -c "ALTER SYSTEM SET max_connections = 200;"
sudo -u postgres psql -c "SELECT pg_reload_conf();"

# Update application pool size
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"database": {"pool_size": 15, "max_lifetime_seconds": 1800}}'

# Kill long-running queries
sudo -u postgres psql -c "
SELECT pg_cancel_backend(pid) 
FROM pg_stat_activity 
WHERE state = 'active' 
  AND query_start < now() - interval '30 minutes';"
```

---

## WebSocket Connection Failures

**Severity**: 🟡 High (P2)  
**Alert Description**: WebSocket connections failing or frequently disconnecting

### Immediate Actions (< 5 minutes)

#### 1. Test WebSocket Connectivity
```bash
# Test WebSocket endpoint directly
curl -i -N \
     -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" \
     -H "Sec-WebSocket-Version: 13" \
     http://localhost:8080/ws/monitoring

# Check WebSocket service status
curl http://localhost:8080/api/v1/admin/websocket/stats

# Test from external client
wscat -c ws://localhost:8080/ws/monitoring || echo "wscat not available"
```

#### 2. Check Network and Firewall
```bash
# Check port availability
netstat -tlnp | grep 8080
ss -tlnp | grep 8080

# Check firewall rules
ufw status | grep 8080
iptables -L | grep 8080

# Check proxy configuration (if using nginx/apache)
nginx -t && nginx -s reload || echo "Nginx not configured"
```

#### 3. Review Active Connections
```bash
# Check current WebSocket connections
curl http://localhost:8080/api/v1/admin/websocket/connections

# Check connection distribution
ss -an | grep :8080 | wc -l

# Monitor connection attempts
tail -f /var/log/uveddi/uveddi.log | grep -i websocket
```

### Investigation Steps (5-15 minutes)

#### 1. Analyze Connection Patterns
```bash
# Check historical connection metrics
curl http://localhost:9090/api/v1/query?query=websocket_connections_total[1h]

# Review client-side errors
# Check browser console or client logs

# Analyze connection duration
curl http://localhost:8080/api/v1/admin/websocket/stats | \
  jq '.connections[] | {id, duration_seconds, last_activity}'
```

#### 2. Check Resource Usage
```bash
# Check if hitting connection limits
ulimit -n
cat /proc/sys/fs/file-nr

# Check memory usage per connection
ps -eo pid,cmd,rss | grep uveddi

# Check for connection leaks
lsof -p $(pgrep uveddi) | grep -c socket
```

### Resolution Actions

#### If Connection Limit Reached
```bash
# Increase system limits
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# Restart service to apply limits
systemctl restart uveddi

# Configure application limits
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"websocket": {"max_connections": 1000, "idle_timeout_seconds": 300}}'
```

#### If Proxy Configuration Issues
```nginx
# Update nginx configuration for WebSocket support
location /ws/ {
    proxy_pass http://localhost:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_read_timeout 86400;
}
```

---

## Analysis Queue Backup

**Severity**: 🟡 High (P2)  
**Alert Description**: Analysis queue length exceeds threshold or processing stalled

### Immediate Actions (< 5 minutes)

#### 1. Check Queue Status
```bash
# Get queue statistics
curl http://localhost:8080/api/v1/analysis/queue/stats

# List queued analyses
curl http://localhost:8080/api/v1/analysis/queue | head -20

# Check active analyses
curl http://localhost:8080/api/v1/analysis/active
```

#### 2. Assess System Resources
```bash
# Check if resource constraints are causing slowdown
top -n1
df -h
free -h

# Check analysis performance metrics
curl http://localhost:9090/metrics | grep analysis_duration
```

#### 3. Emergency Queue Management
```bash
# Cancel stuck analyses (older than 1 hour)
curl -X POST http://localhost:8080/api/v1/admin/analyses/cleanup \
  -d '{"max_age_minutes": 60, "cancel_stuck": true}'

# Increase concurrent analysis limit temporarily
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"analysis": {"max_concurrent_analyses": 15}}'

# Prioritize critical analyses
curl -X POST http://localhost:8080/api/v1/admin/queue/reorder \
  -d '{"strategy": "priority_first"}'
```

### Resolution Actions

#### If Queue Processing Stalled
```bash
# Restart analysis worker pool
curl -X POST http://localhost:8080/api/v1/admin/workers/restart

# Check for deadlocks
curl http://localhost:8080/api/v1/admin/debug/deadlocks

# Force queue processing restart
systemctl reload uveddi
```

#### If Resource Constraints
```bash
# Scale horizontally (add worker nodes)
# Update load balancer configuration

# Scale vertically (increase resources)
# Adjust system configuration

# Implement queue throttling
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"queue": {"max_queue_size": 100, "throttle_submissions": true}}'
```

---

## AI Service Unavailable

**Severity**: 🟡 High (P2)  
**Alert Description**: AI service (Ollama/OpenAI) is unavailable or not responding

### Immediate Actions (< 5 minutes)

#### 1. Check AI Service Status
```bash
# Test Ollama service
curl http://localhost:11434/api/health || echo "Ollama not responding"
curl http://localhost:11434/api/tags || echo "Cannot list models"

# Check Ollama service logs
journalctl -u ollama --since "10 minutes ago" | tail -20

# Test OpenAI connectivity (if configured)
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models || echo "OpenAI not accessible"
```

#### 2. Switch to Fallback Mode
```bash
# Disable AI features temporarily
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"ai": {"enabled": false, "fallback_mode": true}}'

# Enable basic analysis without AI
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"analysis": {"enable_ai_default": false}}'

# Clear AI processing queue
curl -X POST http://localhost:8080/api/v1/admin/ai/queue/clear
```

### Investigation Steps (5-15 minutes)

#### 1. Diagnose AI Service Issues
```bash
# Check Ollama resource usage
docker stats ollama || systemctl status ollama

# Verify model availability
curl http://localhost:11434/api/tags | jq '.models[].name'

# Check GPU availability (if using GPU)
nvidia-smi || echo "No NVIDIA GPU detected"

# Test model inference
curl http://localhost:11434/api/generate \
  -d '{"model": "llama3.2:latest", "prompt": "Test", "stream": false}'
```

#### 2. Check Network and Configuration
```bash
# Verify network connectivity
telnet localhost 11434
ping ollama.docker.internal # for Docker deployments

# Check AI service configuration
grep -A10 "ai_provider" /etc/uveddi/config.toml

# Review AI request logs
tail -100 /var/log/uveddi/uveddi.log | grep -i "ai\|ollama\|openai"
```

### Resolution Actions

#### If Ollama Service Issues
```bash
# Restart Ollama service
systemctl restart ollama
# or for Docker
docker restart ollama

# Pull required models
ollama pull llama3.2:latest
ollama pull codellama:latest

# Check model status
ollama list
```

#### If Resource Constraints
```bash
# Monitor resource usage
watch -n 5 'nvidia-smi && free -h'

# Reduce model size or switch to smaller model
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"ai": {"ollama": {"model": "llama3.2:7b-instruct"}}}'

# Increase timeout settings
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"ai": {"request_timeout_seconds": 120}}'
```

---

## Disk Space Critical

**Severity**: 🔴 Critical (P1)  
**Alert Description**: Disk space usage > 90% on critical partitions

### Immediate Actions (< 5 minutes)

#### 1. Assess Disk Usage
```bash
# Check disk space across all partitions
df -h

# Find largest directories
du -h /var/lib/uveddi | sort -hr | head -20
du -h /var/log/uveddi | sort -hr | head -10

# Check for large files
find /var/lib/uveddi -type f -size +100M -exec ls -lh {} \; | sort -k5 -hr
```

#### 2. Emergency Cleanup
```bash
# Clean up old log files
find /var/log/uveddi -name "*.log" -mtime +7 -delete
journalctl --vacuum-time=3d

# Clean temporary files
rm -rf /tmp/uveddi-*
find /var/lib/uveddi/cache -name "*.tmp" -delete

# Compress old analysis results
find /var/lib/uveddi/results -name "*.json" -mtime +30 -exec gzip {} \;
```

#### 3. Reduce Cache Size
```bash
# Clear analysis result cache
curl -X POST http://localhost:8080/api/v1/admin/cache/clear \
  -d '{"cache_type": "analysis_results", "keep_recent_hours": 24}'

# Clear AST cache older than 7 days
curl -X POST http://localhost:8080/api/v1/admin/cache/cleanup \
  -d '{"max_age_days": 7}'

# Reduce cache size limits
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"cache": {"max_size_mb": 512}}'
```

### Investigation Steps (5-15 minutes)

#### 1. Identify Space Consumers
```bash
# Analyze disk usage by component
du -h /var/lib/uveddi/{cache,results,logs,plugins} | sort -hr

# Check for core dumps
find / -name "core.*" -size +10M 2>/dev/null

# Check database size
sudo -u postgres psql -c "
SELECT pg_size_pretty(pg_database_size('uveddi_prod')) as db_size;"

# Check for log file growth
ls -lah /var/log/uveddi/ | head -10
```

#### 2. Review Storage Trends
```bash
# Check historical disk usage
curl http://localhost:9090/api/v1/query?query=node_filesystem_free_bytes[24h]

# Identify growth patterns
grep "disk\|space" /var/log/uveddi/uveddi.log | tail -50

# Check backup size
ls -lah /var/backups/uveddi/ | head -10
```

### Resolution Actions

#### Immediate Space Recovery
```bash
# Archive old analysis results
tar -czf /backup/old_analyses_$(date +%Y%m%d).tar.gz \
  /var/lib/uveddi/results/*/*/*.json
rm -rf /var/lib/uveddi/results/2024/*/

# Move large files to external storage
# (Configure external storage mount first)
rsync -av /var/lib/uveddi/cache/ /external/storage/cache/
rm -rf /var/lib/uveddi/cache/*
```

#### Long-term Solutions
```bash
# Implement automated cleanup
cat > /etc/cron.daily/uveddi-cleanup << 'EOF'
#!/bin/bash
# Clean up files older than 30 days
find /var/lib/uveddi/results -name "*.json" -mtime +30 -delete
find /var/log/uveddi -name "*.log" -mtime +14 -delete
# Clean up cache
curl -X POST http://localhost:8080/api/v1/admin/cache/cleanup \
  -d '{"max_age_days": 30}'
EOF
chmod +x /etc/cron.daily/uveddi-cleanup

# Configure log rotation
cat > /etc/logrotate.d/uveddi << 'EOF'
/var/log/uveddi/*.log {
    daily
    missingok
    rotate 14
    compress
    delaycompress
    notifempty
    copytruncate
}
EOF
```

---

## Service Startup Failures

**Severity**: 🔴 Critical (P1)  
**Alert Description**: Uveddi service fails to start or crashes during startup

### Immediate Actions (< 5 minutes)

#### 1. Check Service Status
```bash
# Check systemd service status
systemctl status uveddi
systemctl status uveddi-rendering

# Check recent logs
journalctl -u uveddi --since "5 minutes ago" --no-pager

# Check for port conflicts
netstat -tlnp | grep -E "8080|3001|9090"
ss -tlnp | grep -E "8080|3001|9090"
```

#### 2. Verify Prerequisites
```bash
# Check database connectivity
pg_isready -h localhost -p 5432 -U uveddi

# Check configuration file syntax
uveddi --config /etc/uveddi/config.toml --check-config

# Verify file permissions
ls -la /var/lib/uveddi/
ls -la /etc/uveddi/
```

#### 3. Attempt Manual Start
```bash
# Try starting manually to see error output
sudo -u uveddi /usr/local/bin/uveddi --config /etc/uveddi/config.toml

# Check environment variables
sudo -u uveddi env | grep UVEDDI

# Verify binary integrity
file /usr/local/bin/uveddi
ldd /usr/local/bin/uveddi | grep "not found"
```

### Investigation Steps (5-15 minutes)

#### 1. Analyze Startup Logs
```bash
# Get detailed startup logs
journalctl -u uveddi --since "1 hour ago" -f

# Check for panic or crash logs
grep -i "panic\|segmentation\|abort" /var/log/uveddi/uveddi.log

# Check system messages
dmesg | grep -i uveddi

# Review configuration parsing errors
grep -i "config\|parse\|invalid" /var/log/uveddi/uveddi.log
```

#### 2. Check Dependencies
```bash
# Verify database connection
psql -h localhost -U uveddi -d uveddi_prod -c "SELECT 1;"

# Check required system libraries
ldd /usr/local/bin/uveddi

# Verify Rust runtime environment
rustc --version
cargo --version

# Check for missing files
test -f /etc/uveddi/config.toml && echo "Config exists" || echo "Config missing"
test -d /var/lib/uveddi && echo "Data dir exists" || echo "Data dir missing"
```

### Resolution Actions

#### If Configuration Issues
```bash
# Validate configuration file
uveddi --config /etc/uveddi/config.toml --validate-config

# Reset to default configuration
cp /etc/uveddi/config.toml /etc/uveddi/config.toml.backup
cp /etc/uveddi/config.default.toml /etc/uveddi/config.toml

# Fix common configuration issues
sed -i 's/localhost:5433/localhost:5432/' /etc/uveddi/config.toml
sed -i 's/127.0.0.1/0.0.0.0/' /etc/uveddi/config.toml
```

#### If Permission Issues
```bash
# Fix ownership and permissions
chown -R uveddi:uveddi /var/lib/uveddi
chown -R uveddi:uveddi /var/log/uveddi
chmod 750 /var/lib/uveddi
chmod 640 /etc/uveddi/config.toml

# Fix SELinux context (if enabled)
restorecon -R /var/lib/uveddi
restorecon -R /etc/uveddi
```

#### If Binary Issues
```bash
# Reinstall binary
wget "https://github.com/uveddi/uveddi/releases/download/v1.0.0/uveddi-v1.0.0-linux-x86_64.tar.gz"
tar -xzf uveddi-v1.0.0-linux-x86_64.tar.gz
sudo cp uveddi /usr/local/bin/
sudo chmod +x /usr/local/bin/uveddi
```

---

## Performance Degradation

**Severity**: 🟡 High (P2)  
**Alert Description**: Analysis response times > 2 seconds or throughput degradation

### Immediate Actions (< 5 minutes)

#### 1. Check Current Performance
```bash
# Check response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/api/v1/health

# Where curl-format.txt contains:
# time_namelookup:    %{time_namelookup}\n
# time_connect:       %{time_connect}\n
# time_appconnect:    %{time_appconnect}\n
# time_pretransfer:   %{time_pretransfer}\n
# time_redirect:      %{time_redirect}\n
# time_starttransfer: %{time_starttransfer}\n
# time_total:         %{time_total}\n

# Check analysis performance
curl http://localhost:8080/api/v1/admin/performance/stats

# Check system load
uptime
iostat 1 3
```

#### 2. Identify Bottlenecks
```bash
# Check for slow queries
curl http://localhost:8080/api/v1/admin/database/slow-queries

# Check cache hit rates
curl http://localhost:8080/api/v1/admin/cache/stats

# Check concurrent operations
curl http://localhost:8080/api/v1/analysis/active | jq length

# Monitor resource usage
htop -d 1
iotop -d 1
```

### Investigation Steps (5-15 minutes)

#### 1. Performance Profiling
```bash
# Enable detailed profiling
curl -X POST http://localhost:8080/api/v1/admin/debug/profiling \
  -d '{"enabled": true, "duration_seconds": 300}'

# Check historical performance metrics
curl http://localhost:9090/api/v1/query_range \
  -d 'query=histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))' \
  -d 'start=2024-07-21T10:00:00Z' \
  -d 'end=2024-07-21T11:00:00Z' \
  -d 'step=60s'

# Analyze request patterns
tail -1000 /var/log/uveddi/access.log | awk '{print $7}' | sort | uniq -c | sort -nr
```

#### 2. Check Dependencies Performance
```bash
# Database performance
sudo -u postgres psql -c "
SELECT query, mean_time, calls, total_time 
FROM pg_stat_statements 
WHERE mean_time > 1000 
ORDER BY mean_time DESC LIMIT 10;"

# AI service response times
curl -w "%{time_total}" http://localhost:11434/api/health

# File system performance
dd if=/dev/zero of=/tmp/test bs=1M count=100 oflag=direct
rm /tmp/test
```

### Resolution Actions

#### If Database Bottleneck
```bash
# Optimize database connections
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"database": {"pool_size": 25, "prepared_statements": true}}'

# Enable query result caching
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"cache": {"enable_query_cache": true, "query_cache_ttl": 300}}'

# Run database maintenance
sudo -u postgres psql -d uveddi_prod -c "VACUUM ANALYZE;"
sudo -u postgres psql -d uveddi_prod -c "REINDEX DATABASE uveddi_prod;"
```

#### If Analysis Performance Issues
```bash
# Enable performance optimizations
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{
    "analysis": {
      "enable_result_caching": true,
      "use_parallel_processing": true,
      "optimize_for_speed": true
    }
  }'

# Adjust analysis algorithms
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{
    "detectors": {
      "use_fast_algorithms": true,
      "skip_expensive_checks": true,
      "early_termination_threshold": 1000
    }
  }'
```

---

## Security Incidents

**Severity**: 🔴 Critical (P1)  
**Alert Description**: Suspicious activity, unauthorized access, or security breach detected

### Immediate Actions (< 5 minutes)

#### 1. Assess Threat Level
```bash
# Check for suspicious login attempts
grep -i "failed\|unauthorized\|invalid" /var/log/uveddi/security.log | tail -50

# Check active sessions
curl http://localhost:8080/api/v1/admin/security/sessions

# Review recent API calls
tail -500 /var/log/uveddi/access.log | grep -E "POST|PUT|DELETE"

# Check for privilege escalation attempts
grep -i "admin\|root\|sudo" /var/log/uveddi/uveddi.log
```

#### 2. Immediate Containment
```bash
# If breach confirmed, enable security lockdown
curl -X POST http://localhost:8080/api/v1/admin/security/lockdown \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# Revoke all active sessions except emergency admin
curl -X POST http://localhost:8080/api/v1/admin/security/revoke-sessions \
  -d '{"exclude_admin": true}'

# Enable additional logging
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"security": {"audit_level": "maximum", "log_all_requests": true}}'
```

#### 3. Block Suspicious Activity
```bash
# Block suspicious IP addresses
SUSPICIOUS_IPS="192.168.1.100 10.0.0.50"
for ip in $SUSPICIOUS_IPS; do
  iptables -A INPUT -s $ip -j DROP
  echo "Blocked $ip"
done

# Rate limit API endpoints
curl -X POST http://localhost:8080/api/v1/admin/security/rate-limit \
  -d '{"requests_per_minute": 10, "burst_size": 20}'
```

### Investigation Steps (5-15 minutes)

#### 1. Forensic Analysis
```bash
# Analyze access patterns
awk '{print $1}' /var/log/uveddi/access.log | sort | uniq -c | sort -nr | head -20

# Check for data exfiltration
grep -i "export\|download\|analysis.*large" /var/log/uveddi/uveddi.log

# Review configuration changes
diff /etc/uveddi/config.toml /etc/uveddi/config.toml.backup

# Check for unauthorized file access
find /var/lib/uveddi -newermt "1 hour ago" -type f
```

#### 2. Check System Integrity
```bash
# Verify binary integrity
sha256sum /usr/local/bin/uveddi > current_hash.txt
diff current_hash.txt /etc/uveddi/binary_hashes.txt

# Check for unusual processes
ps aux | grep -v -E "uveddi|postgres|systemd" | sort

# Check network connections
netstat -plant | grep -E "ESTABLISHED|LISTEN"

# Review user activities
lastlog | head -20
```

### Resolution Actions

#### If Unauthorized Access Confirmed
```bash
# Force password reset for all users
curl -X POST http://localhost:8080/api/v1/admin/security/force-password-reset

# Regenerate all API keys
curl -X POST http://localhost:8080/api/v1/admin/security/regenerate-keys

# Enable 2FA requirement
curl -X POST http://localhost:8080/api/v1/admin/config \
  -d '{"security": {"require_2fa": true, "jwt_expiration_hours": 1}}'

# Audit all user permissions
curl http://localhost:8080/api/v1/admin/users/audit > user_audit.json
```

#### If System Compromise Suspected
```bash
# Create forensic backup
tar -czf /backup/forensic_$(date +%Y%m%d_%H%M%S).tar.gz \
  /var/log/uveddi /etc/uveddi /var/lib/uveddi

# Isolate system
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -A INPUT -i lo -j ACCEPT
iptables -A INPUT -p tcp --dport 22 -j ACCEPT # Keep SSH for investigation

# Contact security team
echo "Security incident detected at $(date)" | \
  mail -s "URGENT: Uveddi Security Incident" security@company.com
```

---

## Escalation Procedures

### When to Escalate

#### Immediate Escalation (P1)
- Service completely unavailable > 15 minutes
- Data loss or corruption detected
- Security breach confirmed
- Critical infrastructure failure

#### Escalation within 1 hour (P2)
- Performance degradation > 50%
- Multiple component failures
- Resource exhaustion not resolved
- Recurring issues

#### Escalation within 4 hours (P3)
- Single component degradation
- Configuration issues affecting users
- Monitoring system failures

### Escalation Contacts

#### Primary On-Call Engineer
- Slack: @oncall-engineering
- Phone: +1-555-0123
- Email: oncall@company.com

#### Secondary (Senior Engineer)
- Slack: @senior-oncall
- Phone: +1-555-0124
- Email: senior-oncall@company.com

#### Management Escalation
- Engineering Manager: @eng-manager
- VP Engineering: @vp-engineering
- CTO: @cto (P1 incidents only)

### Escalation Checklist

Before escalating, ensure you have:
- [ ] Attempted immediate actions from relevant runbook
- [ ] Gathered diagnostic information
- [ ] Documented timeline of events
- [ ] Identified business impact
- [ ] Prepared summary for escalation

### Post-Incident Actions

After resolution:
1. **Document** the incident in detail
2. **Conduct** post-mortem within 48 hours
3. **Update** runbooks based on learnings
4. **Implement** preventive measures
5. **Communicate** resolution to stakeholders

---

This comprehensive incident response documentation provides structured approaches to handle common Uveddi operational issues, from immediate triage to long-term prevention measures.