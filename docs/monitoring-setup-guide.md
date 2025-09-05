# Uveddi Production Monitoring and Observability Setup Guide

## Overview

This guide provides comprehensive instructions for setting up production-ready monitoring and observability for Uveddi. The observability stack includes metrics collection, distributed tracing, health monitoring, alerting, and centralized logging.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Monitoring Infrastructure Setup](#monitoring-infrastructure-setup)
- [Configuration](#configuration)
- [Dashboard Setup](#dashboard-setup)
- [Alerting Configuration](#alerting-configuration)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

## Prerequisites

### Required Services

- **Prometheus** (v2.40+) - Metrics collection and storage
- **Grafana** (v9.0+) - Metrics visualization and dashboards
- **Jaeger** (v1.40+) - Distributed tracing (optional but recommended)
- **PostgreSQL** (v13+) - Primary database with monitoring extensions
- **Redis** (v6.0+) - Caching layer with monitoring

### Optional Services

- **Elasticsearch + Kibana** - Centralized log aggregation and search
- **AlertManager** - Advanced alert routing and silencing
- **PagerDuty** - Incident management integration

## Monitoring Infrastructure Setup

### 1. Prometheus Setup

```bash
# Install Prometheus
wget https://github.com/prometheus/prometheus/releases/latest/download/prometheus-*-linux-amd64.tar.gz
tar xvfz prometheus-*-linux-amd64.tar.gz
sudo mv prometheus-*-linux-amd64 /opt/prometheus
sudo useradd --no-create-home --shell /bin/false prometheus
sudo chown -R prometheus:prometheus /opt/prometheus
```

Create `/etc/prometheus/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "uveddi_rules.yml"

scrape_configs:
  - job_name: 'uveddi'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
    metrics_path: '/metrics'
    
  - job_name: 'uveddi-health'
    static_configs:
      - targets: ['localhost:8888']
    scrape_interval: 30s
    metrics_path: '/health'
    
  - job_name: 'postgresql'
    static_configs:
      - targets: ['localhost:9187']
    scrape_interval: 30s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - localhost:9093
```

### 2. Grafana Setup

```bash
# Install Grafana
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install grafana

# Start Grafana
sudo systemctl enable grafana-server
sudo systemctl start grafana-server
```

### 3. Jaeger Setup (Optional)

```bash
# Using Docker for simplicity
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 14268:14268 \
  jaegertracing/all-in-one:latest
```

### 4. PostgreSQL Monitoring Setup

```bash
# Install postgres_exporter
wget https://github.com/prometheus-community/postgres_exporter/releases/latest/download/postgres_exporter-*-linux-amd64.tar.gz
tar xvfz postgres_exporter-*-linux-amd64.tar.gz
sudo mv postgres_exporter-*-linux-amd64/postgres_exporter /usr/local/bin/

# Create monitoring user in PostgreSQL
sudo -u postgres psql -c "CREATE USER postgres_exporter WITH PASSWORD 'password';"
sudo -u postgres psql -c "ALTER USER postgres_exporter SET SEARCH_PATH TO postgres_exporter,pg_catalog;"
sudo -u postgres psql -c "GRANT CONNECT ON DATABASE uveddi TO postgres_exporter;"
sudo -u postgres psql -c "GRANT pg_monitor TO postgres_exporter;"
```

## Configuration

### 1. Uveddi Configuration

Copy the monitoring configuration:

```bash
cp monitoring-config.toml /etc/uveddi/observability.toml
```

Key configuration sections:

```toml
[observability.metrics]
enabled = true
port = 9090
endpoint_path = "/metrics"

[observability.health_check] 
enabled = true
check_interval = "30s"

[observability.tracing]
enabled = true
sampling_rate = 0.1  # 10% sampling in production
jaeger_endpoint = "http://localhost:14268/api/traces"
```

### 2. Environment-Specific Configuration

**Development:**
```bash
export UVEDDI_ENV=development
export UVEDDI_LOG_LEVEL=debug
export UVEDDI_METRICS_ENABLED=false
```

**Production:**
```bash
export UVEDDI_ENV=production
export UVEDDI_LOG_LEVEL=info
export UVEDDI_METRICS_ENABLED=true
export UVEDDI_TRACING_SAMPLING_RATE=0.1
```

### 3. Security Configuration

Set up authentication and secure endpoints:

```toml
[observability.security]
enable_audit_integration = true
export_security_metrics = true
event_sampling_rate = 1.0

[observability.logging.pii_redaction]
enabled = true
strategy = "mask"
```

## Dashboard Setup

### 1. Import Uveddi Dashboards

Grafana dashboards are available in the `monitoring/dashboards/` directory:

- **overview.json** - System overview and health
- **analysis.json** - Analysis performance metrics
- **security.json** - Security monitoring
- **performance.json** - Detailed performance metrics

Import via Grafana UI:
1. Go to Dashboards → Import
2. Upload JSON files or use dashboard IDs
3. Select Prometheus as data source

### 2. Key Metrics to Monitor

#### System Health Metrics
- `uveddi_requests_total` - Total HTTP requests
- `uveddi_request_duration_seconds` - Request latency
- `uveddi_errors_total` - Error count by type
- `uveddi_memory_usage_bytes` - Memory consumption
- `uveddi_cpu_utilization_percent` - CPU usage

#### Analysis Metrics
- `uveddi_analysis_requests_total` - Analysis requests
- `uveddi_analysis_duration_seconds` - Analysis time
- `uveddi_analysis_bugs_found_total` - Issues detected
- `uveddi_analysis_queue_length` - Queue backlog

#### Database Metrics
- `pg_up` - PostgreSQL availability
- `pg_connections_active` - Active connections
- `pg_stat_database_tup_returned_total` - Query performance

### 3. Custom Dashboard Creation

Create custom dashboards using PromQL queries:

```promql
# Request rate
rate(uveddi_requests_total[5m])

# Error rate
rate(uveddi_errors_total[5m]) / rate(uveddi_requests_total[5m])

# P99 latency
histogram_quantile(0.99, rate(uveddi_request_duration_seconds_bucket[5m]))

# Analysis throughput
rate(uveddi_analysis_requests_total{status="success"}[5m])
```

## Alerting Configuration

### 1. Prometheus Alert Rules

Create `/etc/prometheus/uveddi_rules.yml`:

```yaml
groups:
- name: uveddi_alerts
  rules:
  - alert: HighErrorRate
    expr: rate(uveddi_errors_total[5m]) / rate(uveddi_requests_total[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value | humanizePercentage }}"

  - alert: HighMemoryUsage
    expr: uveddi_memory_usage_bytes / (1024*1024*1024) > 4
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage"
      description: "Memory usage is {{ $value }}GB"

  - alert: AnalysisQueueBacklog
    expr: uveddi_analysis_queue_length > 50
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Analysis queue backlog"
      description: "Queue has {{ $value }} pending requests"

  - alert: ServiceDown
    expr: up{job="uveddi"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Uveddi service is down"
      description: "Uveddi has been down for more than 1 minute"
```

### 2. Slack Integration

Configure Slack webhook in `monitoring-config.toml`:

```toml
[[notification_channels]]
name = "slack"
type = "slack"
enabled = true
webhook_url = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
```

### 3. PagerDuty Integration

For critical alerts:

```toml
[[notification_channels]]
name = "pagerduty"
type = "pagerduty"
enabled = true
integration_key = "your-pagerduty-integration-key"
```

## Health Checks

### 1. Application Health Endpoints

Uveddi exposes several health check endpoints:

- `GET /health` - Overall system health
- `GET /health/detailed` - Component-level health
- `GET /health/ready` - Readiness probe
- `GET /health/live` - Liveness probe

### 2. Kubernetes Health Probes

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: uveddi
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8888
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /health/ready  
        port: 8888
      initialDelaySeconds: 5
      periodSeconds: 5
```

## Log Management

### 1. Structured Logging

Uveddi uses structured JSON logging in production:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "message": "Analysis completed",
  "trace_id": "abc123...",
  "project_id": "proj_456",
  "duration_ms": 1500,
  "issues_found": 12
}
```

### 2. Log Aggregation with ELK Stack

If using Elasticsearch:

```bash
# Configure Filebeat to ship logs
sudo apt-get install filebeat
```

Filebeat configuration:
```yaml
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/uveddi/*.log
  json.keys_under_root: true

output.elasticsearch:
  hosts: ["localhost:9200"]
  index: "uveddi-logs-%{+yyyy.MM.dd}"
```

## Security Monitoring

### 1. Authentication Failures

Monitor failed login attempts:

```promql
increase(uveddi_security_auth_failures_total[5m]) > 10
```

### 2. Rate Limiting

Track rate limit violations:

```promql
rate(uveddi_security_rate_limit_exceeded_total[5m]) > 1
```

### 3. Audit Events

Security events are logged with special markers:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "WARN",
  "event_type": "security_event",
  "action": "permission_denied",
  "user_id": "user_123",
  "resource": "/api/admin/users"
}
```

## Performance Monitoring

### 1. Key Performance Indicators

Monitor these critical metrics:

- **Availability**: > 99.9% uptime
- **Latency**: P99 < 2 seconds
- **Error Rate**: < 0.1%
- **Throughput**: Analysis requests per minute

### 2. Database Performance

```sql
-- Slow query monitoring
SELECT query, mean_time, calls, total_time
FROM pg_stat_statements
WHERE mean_time > 1000
ORDER BY mean_time DESC;
```

### 3. Memory and CPU Monitoring

Use system metrics to track resource utilization:

```promql
# CPU usage
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Memory usage
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100
```

## Troubleshooting

### Common Issues

#### 1. Metrics Not Appearing

**Symptoms**: Empty dashboards, missing metrics
**Solutions**:
- Check Prometheus targets: `http://localhost:9090/targets`
- Verify Uveddi metrics endpoint: `curl http://localhost:9090/metrics`
- Check firewall rules and network connectivity

#### 2. High Memory Usage Alerts

**Symptoms**: Memory alerts firing frequently
**Solutions**:
- Check for memory leaks in analysis pipeline
- Adjust JVM/Rust memory settings
- Scale horizontally if needed

#### 3. Tracing Not Working

**Symptoms**: No traces in Jaeger
**Solutions**:
- Verify Jaeger collector endpoint
- Check sampling rate configuration
- Ensure trace context propagation

### Debug Commands

```bash
# Check Uveddi health
curl http://localhost:8888/health

# Verify Prometheus metrics
curl http://localhost:9090/metrics | grep uveddi

# Check Jaeger traces
curl http://localhost:16686/api/traces?service=uveddi

# Test alerting
curl -X POST http://localhost:9093/api/v1/alerts
```

## Best Practices

### 1. Monitoring Best Practices

- **Use the RED method**: Rate, Errors, Duration
- **Implement SLIs/SLOs**: Define service level indicators and objectives
- **Monitor user experience**: Track business metrics, not just technical metrics
- **Use proper alerting**: Alert on symptoms, not causes

### 2. Dashboard Design

- **Keep it simple**: Focus on the most important metrics
- **Use consistent time ranges**: Align all panels to same time window
- **Include context**: Add annotations for deployments and incidents
- **Make it actionable**: Include links to runbooks and documentation

### 3. Alert Configuration

- **Reduce noise**: Use proper thresholds and time windows
- **Escalate appropriately**: Match severity to response requirements
- **Provide context**: Include relevant metrics and troubleshooting steps
- **Test regularly**: Verify alerts work as expected

### 4. Security Monitoring

- **Monitor authentication**: Track failed logins and suspicious patterns
- **Watch for privilege escalation**: Monitor admin action attempts
- **Log security events**: Maintain audit trail for compliance
- **Implement anomaly detection**: Detect unusual behavior patterns

## Maintenance and Updates

### 1. Regular Tasks

- **Review dashboards**: Update and optimize monthly
- **Test alerting**: Verify alerts work correctly
- **Clean up metrics**: Remove unused or redundant metrics
- **Update retention policies**: Balance storage costs and data needs

### 2. Capacity Planning

Monitor resource usage trends:
- CPU and memory utilization
- Disk space growth
- Network bandwidth usage
- Database connection pool utilization

### 3. Security Updates

- Keep monitoring tools updated
- Rotate authentication credentials
- Review access permissions
- Update SSL certificates

## Support and Resources

### Internal Resources

- **Runbooks**: `/docs/runbooks/`
- **Architecture**: `/docs/architecture/observability.md`
- **API Documentation**: `/docs/api/monitoring-endpoints.md`

### External Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/)

### Getting Help

- **Internal Support**: Slack #uveddi-monitoring
- **Issue Tracking**: GitHub Issues
- **Documentation**: This guide and linked resources

---

For questions or issues with this monitoring setup, contact the DevOps team or create an issue in the Uveddi repository.