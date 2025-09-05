#!/bin/bash

# Uveddi Production Monitoring Setup Script
# This script sets up the complete monitoring infrastructure for Uveddi

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PROMETHEUS_VERSION="2.40.7"
GRAFANA_VERSION="9.3.2"
POSTGRES_EXPORTER_VERSION="0.11.1"
JAEGER_VERSION="1.40.0"

# Directories
MONITORING_DIR="/opt/monitoring"
CONFIG_DIR="/etc/uveddi"
LOG_DIR="/var/log/uveddi"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if running as root
    if [[ $EUID -eq 0 ]]; then
        log_error "This script should not be run as root for security reasons"
        exit 1
    fi
    
    # Check if sudo is available
    if ! command -v sudo &> /dev/null; then
        log_error "sudo is required but not installed"
        exit 1
    fi
    
    # Check available disk space (require at least 10GB)
    available_space=$(df / | tail -1 | awk '{print $4}')
    required_space=10485760  # 10GB in KB
    
    if [[ $available_space -lt $required_space ]]; then
        log_error "Insufficient disk space. Require at least 10GB free space"
        exit 1
    fi
    
    log_info "Prerequisites check passed"
}

create_users() {
    log_info "Creating system users..."
    
    # Create prometheus user
    if ! id "prometheus" &>/dev/null; then
        sudo useradd --no-create-home --shell /bin/false prometheus
        log_info "Created prometheus user"
    fi
    
    # Create grafana user
    if ! id "grafana" &>/dev/null; then
        sudo useradd --system --home-dir /var/lib/grafana --shell /bin/false grafana
        log_info "Created grafana user"
    fi
}

create_directories() {
    log_info "Creating directories..."
    
    sudo mkdir -p "$MONITORING_DIR"/{prometheus,grafana,alertmanager}
    sudo mkdir -p "$CONFIG_DIR"
    sudo mkdir -p "$LOG_DIR"
    sudo mkdir -p /var/lib/{prometheus,grafana}
    sudo mkdir -p /etc/prometheus/{rules,alerting}
    sudo mkdir -p /etc/grafana/dashboards
    
    # Set permissions
    sudo chown -R prometheus:prometheus /var/lib/prometheus
    sudo chown -R prometheus:prometheus /etc/prometheus
    sudo chown -R grafana:grafana /var/lib/grafana
    sudo chown -R grafana:grafana /etc/grafana
    
    log_info "Directories created successfully"
}

install_prometheus() {
    log_info "Installing Prometheus..."
    
    # Download and extract Prometheus
    cd /tmp
    wget "https://github.com/prometheus/prometheus/releases/download/v${PROMETHEUS_VERSION}/prometheus-${PROMETHEUS_VERSION}.linux-amd64.tar.gz"
    tar xvf "prometheus-${PROMETHEUS_VERSION}.linux-amd64.tar.gz"
    
    # Install binaries
    sudo cp "prometheus-${PROMETHEUS_VERSION}.linux-amd64/prometheus" /usr/local/bin/
    sudo cp "prometheus-${PROMETHEUS_VERSION}.linux-amd64/promtool" /usr/local/bin/
    sudo chown prometheus:prometheus /usr/local/bin/prometheus*
    
    # Copy console files
    sudo cp -r "prometheus-${PROMETHEUS_VERSION}.linux-amd64/consoles" /etc/prometheus/
    sudo cp -r "prometheus-${PROMETHEUS_VERSION}.linux-amd64/console_libraries" /etc/prometheus/
    sudo chown -R prometheus:prometheus /etc/prometheus/consoles*
    
    # Create configuration file
    sudo tee /etc/prometheus/prometheus.yml > /dev/null <<EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

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

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']

  - job_name: 'postgres'
    static_configs:
      - targets: ['localhost:9187']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - localhost:9093
EOF

    sudo chown prometheus:prometheus /etc/prometheus/prometheus.yml
    
    # Create systemd service
    sudo tee /etc/systemd/system/prometheus.service > /dev/null <<EOF
[Unit]
Description=Prometheus
Wants=network-online.target
After=network-online.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/usr/local/bin/prometheus \\
    --config.file /etc/prometheus/prometheus.yml \\
    --storage.tsdb.path /var/lib/prometheus/ \\
    --web.console.templates=/etc/prometheus/consoles \\
    --web.console.libraries=/etc/prometheus/console_libraries \\
    --web.listen-address=0.0.0.0:9090 \\
    --web.enable-lifecycle \\
    --storage.tsdb.retention.time=15d

[Install]
WantedBy=multi-user.target
EOF
    
    sudo systemctl daemon-reload
    sudo systemctl enable prometheus
    
    log_info "Prometheus installed successfully"
    
    # Cleanup
    rm -rf "/tmp/prometheus-${PROMETHEUS_VERSION}.linux-amd64"*
}

install_grafana() {
    log_info "Installing Grafana..."
    
    # Add Grafana repository
    sudo apt-get update
    sudo apt-get install -y software-properties-common wget
    sudo wget -q -O /usr/share/keyrings/grafana.key https://packages.grafana.com/gpg.key
    echo "deb [signed-by=/usr/share/keyrings/grafana.key] https://packages.grafana.com/oss/deb stable main" | sudo tee -a /etc/apt/sources.list.d/grafana.list
    
    # Install Grafana
    sudo apt-get update
    sudo apt-get install -y grafana
    
    # Configure Grafana
    sudo tee /etc/grafana/grafana.ini > /dev/null <<EOF
[security]
admin_user = admin
admin_password = admin123

[server]
http_port = 3000
domain = localhost

[database]
type = sqlite3
path = /var/lib/grafana/grafana.db

[session]
provider = memory

[analytics]
reporting_enabled = false
check_for_updates = false

[log]
mode = console file
level = info

[paths]
data = /var/lib/grafana
temp_data_lifetime = 24h
logs = /var/log/grafana
plugins = /var/lib/grafana/plugins
provisioning = /etc/grafana/provisioning
EOF

    # Create provisioning directories
    sudo mkdir -p /etc/grafana/provisioning/{datasources,dashboards,notifiers}
    
    # Configure Prometheus datasource
    sudo tee /etc/grafana/provisioning/datasources/prometheus.yml > /dev/null <<EOF
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://localhost:9090
    isDefault: true
    editable: true
EOF

    # Configure dashboard provisioning
    sudo tee /etc/grafana/provisioning/dashboards/uveddi.yml > /dev/null <<EOF
apiVersion: 1

providers:
  - name: 'uveddi'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/dashboards
EOF

    sudo systemctl enable grafana-server
    
    log_info "Grafana installed successfully"
}

install_node_exporter() {
    log_info "Installing Node Exporter..."
    
    # Download and install node_exporter
    cd /tmp
    wget "https://github.com/prometheus/node_exporter/releases/download/v1.5.0/node_exporter-1.5.0.linux-amd64.tar.gz"
    tar xvf node_exporter-1.5.0.linux-amd64.tar.gz
    sudo cp node_exporter-1.5.0.linux-amd64/node_exporter /usr/local/bin/
    
    # Create user
    sudo useradd --no-create-home --shell /bin/false node_exporter || true
    sudo chown node_exporter:node_exporter /usr/local/bin/node_exporter
    
    # Create systemd service
    sudo tee /etc/systemd/system/node_exporter.service > /dev/null <<EOF
[Unit]
Description=Node Exporter
Wants=network-online.target
After=network-online.target

[Service]
User=node_exporter
Group=node_exporter
Type=simple
ExecStart=/usr/local/bin/node_exporter

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable node_exporter
    
    log_info "Node Exporter installed successfully"
    
    # Cleanup
    rm -rf /tmp/node_exporter-1.5.0.linux-amd64*
}

install_postgres_exporter() {
    log_info "Installing PostgreSQL Exporter..."
    
    # Download and install postgres_exporter
    cd /tmp
    wget "https://github.com/prometheus-community/postgres_exporter/releases/download/v${POSTGRES_EXPORTER_VERSION}/postgres_exporter-${POSTGRES_EXPORTER_VERSION}.linux-amd64.tar.gz"
    tar xvf "postgres_exporter-${POSTGRES_EXPORTER_VERSION}.linux-amd64.tar.gz"
    sudo cp "postgres_exporter-${POSTGRES_EXPORTER_VERSION}.linux-amd64/postgres_exporter" /usr/local/bin/
    
    # Create user
    sudo useradd --no-create-home --shell /bin/false postgres_exporter || true
    sudo chown postgres_exporter:postgres_exporter /usr/local/bin/postgres_exporter
    
    # Create environment file
    sudo tee /etc/default/postgres_exporter > /dev/null <<EOF
DATA_SOURCE_NAME="postgresql://postgres_exporter:password@localhost:5432/postgres?sslmode=disable"
EOF

    # Create systemd service
    sudo tee /etc/systemd/system/postgres_exporter.service > /dev/null <<EOF
[Unit]
Description=PostgreSQL Exporter
Wants=network-online.target
After=network-online.target

[Service]
User=postgres_exporter
Group=postgres_exporter
Type=simple
EnvironmentFile=/etc/default/postgres_exporter
ExecStart=/usr/local/bin/postgres_exporter
Restart=always

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable postgres_exporter
    
    log_info "PostgreSQL Exporter installed successfully"
    
    # Cleanup
    rm -rf "/tmp/postgres_exporter-${POSTGRES_EXPORTER_VERSION}.linux-amd64"*
}

install_jaeger() {
    log_info "Installing Jaeger (using Docker)..."
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        log_warn "Docker is not installed. Installing Docker..."
        curl -fsSL https://get.docker.com -o get-docker.sh
        sudo sh get-docker.sh
        sudo usermod -aG docker $USER
        rm get-docker.sh
    fi
    
    # Create docker-compose file for Jaeger
    sudo tee /opt/monitoring/docker-compose.jaeger.yml > /dev/null <<EOF
version: '3.7'

services:
  jaeger-all-in-one:
    image: jaegertracing/all-in-one:${JAEGER_VERSION}
    ports:
      - "16686:16686"  # Jaeger UI
      - "14268:14268"  # HTTP collector
      - "6831:6831/udp"  # Agent
      - "6832:6832/udp"  # Agent
    environment:
      - COLLECTOR_ZIPKIN_HTTP_PORT=9411
    restart: unless-stopped
    container_name: jaeger
    volumes:
      - jaeger-data:/badger
    networks:
      - monitoring

volumes:
  jaeger-data:

networks:
  monitoring:
    external: true
EOF

    # Create monitoring network
    sudo docker network create monitoring 2>/dev/null || true
    
    log_info "Jaeger configuration created. Start with: docker-compose -f /opt/monitoring/docker-compose.jaeger.yml up -d"
}

create_alert_rules() {
    log_info "Creating Prometheus alert rules..."
    
    sudo tee /etc/prometheus/rules/uveddi.yml > /dev/null <<EOF
groups:
- name: uveddi_alerts
  rules:
  
  # Service availability alerts
  - alert: UveddiDown
    expr: up{job="uveddi"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Uveddi service is down"
      description: "Uveddi has been down for more than 1 minute"

  # Error rate alerts
  - alert: HighErrorRate
    expr: rate(uveddi_errors_total[5m]) / rate(uveddi_requests_total[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ \$value | humanizePercentage }} over the last 5 minutes"

  # Latency alerts
  - alert: HighLatency
    expr: histogram_quantile(0.99, rate(uveddi_request_duration_seconds_bucket[5m])) > 5
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High request latency"
      description: "99th percentile latency is {{ \$value }}s"

  # Memory usage alerts
  - alert: HighMemoryUsage
    expr: (uveddi_memory_usage_bytes / (1024*1024*1024)) > 4
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage"
      description: "Memory usage is {{ \$value }}GB"

  - alert: CriticalMemoryUsage
    expr: (uveddi_memory_usage_bytes / (1024*1024*1024)) > 6
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Critical memory usage"
      description: "Memory usage is {{ \$value }}GB"

  # Analysis-specific alerts
  - alert: AnalysisQueueBacklog
    expr: uveddi_analysis_queue_length > 50
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Analysis queue backlog"
      description: "Analysis queue has {{ \$value }} pending requests"

  - alert: AnalysisFailureRate
    expr: rate(uveddi_analysis_requests_total{status="failure"}[5m]) / rate(uveddi_analysis_requests_total[5m]) > 0.2
    for: 10m
    labels:
      severity: critical
    annotations:
      summary: "High analysis failure rate"
      description: "Analysis failure rate is {{ \$value | humanizePercentage }}"

  # Database alerts
  - alert: DatabaseDown
    expr: up{job="postgres"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Database is down"
      description: "PostgreSQL database has been down for more than 1 minute"

  - alert: DatabaseConnectionsHigh
    expr: sum(pg_stat_database_numbackends) > 80
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High database connections"
      description: "Database has {{ \$value }} active connections"

  # System resource alerts
  - alert: HighCPUUsage
    expr: 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High CPU usage"
      description: "CPU usage is {{ \$value }}%"

  - alert: LowDiskSpace
    expr: (node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 10
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Low disk space"
      description: "Disk usage is above 90% on {{ \$labels.device }}"
EOF

    sudo chown prometheus:prometheus /etc/prometheus/rules/uveddi.yml
    log_info "Alert rules created successfully"
}

create_grafana_dashboards() {
    log_info "Creating Grafana dashboards..."
    
    # Overview dashboard
    sudo tee /etc/grafana/dashboards/uveddi-overview.json > /dev/null <<'EOF'
{
  "dashboard": {
    "id": null,
    "title": "Uveddi Overview",
    "tags": ["uveddi"],
    "style": "dark",
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(uveddi_requests_total[5m])",
            "legendFormat": "Requests/sec"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "yAxes": [
          {"label": "Requests/sec"},
          {"show": false}
        ]
      },
      {
        "id": 2,
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(uveddi_errors_total[5m]) / rate(uveddi_requests_total[5m])",
            "legendFormat": "Error Rate"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "yAxes": [
          {"label": "Rate", "unit": "percentunit"},
          {"show": false}
        ]
      },
      {
        "id": 3,
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(uveddi_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(uveddi_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(uveddi_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p99"
          }
        ],
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 8},
        "yAxes": [
          {"label": "Response Time", "unit": "s"},
          {"show": false}
        ]
      }
    ],
    "time": {"from": "now-1h", "to": "now"},
    "refresh": "30s"
  }
}
EOF

    log_info "Grafana dashboards created successfully"
}

setup_systemd_services() {
    log_info "Setting up systemd services..."
    
    # Reload systemd daemon
    sudo systemctl daemon-reload
    
    # Enable and start services
    services=("prometheus" "grafana-server" "node_exporter")
    
    for service in "${services[@]}"; do
        log_info "Starting $service..."
        sudo systemctl enable "$service"
        sudo systemctl start "$service"
        
        # Check if service started successfully
        if sudo systemctl is-active --quiet "$service"; then
            log_info "$service started successfully"
        else
            log_error "Failed to start $service"
            sudo systemctl status "$service"
        fi
    done
}

configure_firewall() {
    log_info "Configuring firewall rules..."
    
    # Check if UFW is installed and active
    if command -v ufw &> /dev/null; then
        # Prometheus
        sudo ufw allow 9090/tcp comment 'Prometheus'
        
        # Grafana  
        sudo ufw allow 3000/tcp comment 'Grafana'
        
        # Node Exporter (local only)
        sudo ufw allow from 127.0.0.1 to any port 9100
        
        # PostgreSQL Exporter (local only)
        sudo ufw allow from 127.0.0.1 to any port 9187
        
        # Jaeger (if using)
        sudo ufw allow 16686/tcp comment 'Jaeger UI'
        sudo ufw allow 14268/tcp comment 'Jaeger Collector'
        
        log_info "Firewall rules configured"
    else
        log_warn "UFW not installed. Please configure firewall manually"
    fi
}

create_monitoring_user() {
    log_info "Creating monitoring administrative user..."
    
    # Create monitoring group
    sudo groupadd monitoring 2>/dev/null || true
    
    # Add current user to monitoring group
    sudo usermod -a -G monitoring "$USER"
    
    log_info "User $USER added to monitoring group"
}

validate_installation() {
    log_info "Validating installation..."
    
    # Check if services are running
    services=("prometheus" "grafana-server" "node_exporter")
    for service in "${services[@]}"; do
        if sudo systemctl is-active --quiet "$service"; then
            log_info "✓ $service is running"
        else
            log_error "✗ $service is not running"
        fi
    done
    
    # Check endpoints
    endpoints=(
        "http://localhost:9090/-/healthy"
        "http://localhost:3000/api/health"
        "http://localhost:9100/metrics"
    )
    
    for endpoint in "${endpoints[@]}"; do
        if curl -s -f "$endpoint" > /dev/null; then
            log_info "✓ $endpoint is accessible"
        else
            log_warn "✗ $endpoint is not accessible"
        fi
    done
}

print_summary() {
    echo
    echo "======================================="
    echo "  Uveddi Monitoring Setup Complete!"
    echo "======================================="
    echo
    echo "Services installed and configured:"
    echo "• Prometheus: http://localhost:9090"
    echo "• Grafana: http://localhost:3000 (admin/admin123)"
    echo "• Node Exporter: http://localhost:9100/metrics"
    echo
    echo "Next steps:"
    echo "1. Configure Uveddi to enable metrics endpoint"
    echo "2. Import additional Grafana dashboards"
    echo "3. Set up alerting channels (Slack, PagerDuty, etc.)"
    echo "4. Configure backup and retention policies"
    echo "5. Review and adjust alert thresholds"
    echo
    echo "Documentation:"
    echo "• Setup guide: docs/monitoring-setup-guide.md"
    echo "• Configuration: monitoring-config.toml"
    echo "• Dashboards: /etc/grafana/dashboards/"
    echo
    echo "For support, see the monitoring setup guide or contact the DevOps team."
}

# Main execution
main() {
    echo "======================================="
    echo "  Uveddi Monitoring Setup Script"
    echo "======================================="
    echo
    
    check_prerequisites
    create_users
    create_directories
    install_prometheus
    install_grafana
    install_node_exporter
    install_postgres_exporter
    install_jaeger
    create_alert_rules
    create_grafana_dashboards
    setup_systemd_services
    configure_firewall
    create_monitoring_user
    validate_installation
    print_summary
    
    log_info "Monitoring setup completed successfully!"
}

# Run main function
main "$@"