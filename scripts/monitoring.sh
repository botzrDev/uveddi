#!/bin/bash
# Comprehensive monitoring setup for Uveddi
# Provides health monitoring, alerting, and performance tracking

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MONITOR_DIR="$PROJECT_ROOT/monitoring"
LOG_DIR="$PROJECT_ROOT/logs"
ALERT_WEBHOOK="${ALERT_WEBHOOK:-}"
CHECK_INTERVAL=30
ALERT_COOLDOWN=300 # 5 minutes

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${BLUE}[MONITOR]${NC} $(date '+%H:%M:%S') - $1"; }
log_success() { echo -e "${GREEN}[MONITOR]${NC} $(date '+%H:%M:%S') - $1"; }
log_warning() { echo -e "${YELLOW}[MONITOR]${NC} $(date '+%H:%M:%S') - $1"; }
log_error() { echo -e "${RED}[MONITOR]${NC} $(date '+%H:%M:%S') - $1"; }

# Initialize monitoring
init_monitoring() {
    mkdir -p "$MONITOR_DIR" "$LOG_DIR"
    
    # Create monitoring state file
    cat > "$MONITOR_DIR/state.json" <<EOF
{
  "lastCheck": "$(date -Iseconds)",
  "services": {
    "api": {"status": "unknown", "lastAlert": 0},
    "frontend": {"status": "unknown", "lastAlert": 0},
    "rendering": {"status": "unknown", "lastAlert": 0}
  },
  "metrics": {
    "checks": 0,
    "failures": 0,
    "alerts": 0
  }
}
EOF

    log_info "Monitoring initialized"
}

# Service health check
check_service_health() {
    local service="$1"
    local url="$2"
    local timeout="${3:-10}"
    
    local start_time=$(date +%s%3N)
    local response_code
    local response_time
    
    if response_code=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 --max-time "$timeout" "$url" 2>/dev/null); then
        response_time=$(($(date +%s%3N) - start_time))
        
        if [ "$response_code" = "200" ]; then
            echo "healthy $response_time"
            return 0
        else
            echo "unhealthy $response_time $response_code"
            return 1
        fi
    else
        response_time=$(($(date +%s%3N) - start_time))
        echo "unreachable $response_time"
        return 1
    fi
}

# System metrics collection
collect_system_metrics() {
    local metrics_file="$MONITOR_DIR/metrics.json"
    local timestamp=$(date -Iseconds)
    
    # CPU usage
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    
    # Memory usage
    local memory_info=$(free -m | awk 'NR==2{printf "%.1f %.1f %.1f", $3*100/$2, $3, $2}')
    read -r mem_percent mem_used mem_total <<< "$memory_info"
    
    # Disk usage
    local disk_info=$(df "$PROJECT_ROOT" | awk 'NR==2{printf "%.1f %.1f %.1f", $5, $3/1024/1024, $2/1024/1024}')
    read -r disk_percent disk_used disk_total <<< "$disk_info"
    
    # Network connections
    local connections=$(ss -tuln | wc -l)
    
    # Process counts
    local api_processes=$(pgrep -f "node.*server.js" | wc -l)
    local frontend_processes=$(pgrep -f "npm.*dev" | wc -l)
    local rendering_processes=$(pgrep -f "node.*rendering" | wc -l)
    
    # Create metrics entry
    cat >> "$metrics_file" <<EOF
{
  "timestamp": "$timestamp",
  "system": {
    "cpu_usage": $cpu_usage,
    "memory": {
      "usage_percent": $mem_percent,
      "used_mb": $mem_used,
      "total_mb": $mem_total
    },
    "disk": {
      "usage_percent": ${disk_percent%\%},
      "used_gb": $disk_used,
      "total_gb": $disk_total
    },
    "connections": $connections
  },
  "processes": {
    "api": $api_processes,
    "frontend": $frontend_processes,
    "rendering": $rendering_processes
  }
}
EOF
    
    # Keep only last 1000 entries
    tail -n 1000 "$metrics_file" > "$metrics_file.tmp" && mv "$metrics_file.tmp" "$metrics_file"
}

# Send alert
send_alert() {
    local severity="$1"
    local service="$2"
    local message="$3"
    local timestamp=$(date -Iseconds)
    
    # Log alert
    local alert_log="$LOG_DIR/alerts.log"
    echo "[$timestamp] $severity: $service - $message" >> "$alert_log"
    
    # Console notification
    case "$severity" in
        "CRITICAL")
            log_error "ALERT: $service - $message"
            ;;
        "WARNING")
            log_warning "ALERT: $service - $message"
            ;;
        *)
            log_info "ALERT: $service - $message"
            ;;
    esac
    
    # Webhook notification
    if [ -n "$ALERT_WEBHOOK" ]; then
        local payload=$(cat <<EOF
{
  "timestamp": "$timestamp",
  "severity": "$severity",
  "service": "$service",
  "message": "$message",
  "host": "$(hostname)"
}
EOF
)
        curl -s -X POST -H "Content-Type: application/json" -d "$payload" "$ALERT_WEBHOOK" || true
    fi
    
    # Desktop notification (if available)
    if command -v notify-send >/dev/null 2>&1; then
        notify-send "Uveddi Alert" "$severity: $service - $message" || true
    fi
}

# Check if alert should be sent (cooldown logic)
should_send_alert() {
    local service="$1"
    local current_time=$(date +%s)
    local state_file="$MONITOR_DIR/state.json"
    
    if [ ! -f "$state_file" ]; then
        return 0 # Send alert if no state file
    fi
    
    local last_alert=$(jq -r ".services.$service.lastAlert" "$state_file" 2>/dev/null || echo "0")
    local time_diff=$((current_time - last_alert))
    
    if [ "$time_diff" -gt "$ALERT_COOLDOWN" ]; then
        # Update last alert time
        local temp_file=$(mktemp)
        jq ".services.$service.lastAlert = $current_time" "$state_file" > "$temp_file" && mv "$temp_file" "$state_file"
        return 0
    fi
    
    return 1
}

# Monitor services
monitor_services() {
    local services=(
        "api:http://localhost:8000/health"
        "frontend:http://localhost:8001"
        "rendering:http://localhost:3001/health"
    )
    
    local all_healthy=true
    local status_summary=""
    
    for service_config in "${services[@]}"; do
        IFS=':' read -r service_name service_url <<< "$service_config"
        
        local health_result
        health_result=$(check_service_health "$service_name" "$service_url")
        local health_status=$?
        
        IFS=' ' read -r status response_time extra <<< "$health_result"
        
        if [ $health_status -eq 0 ]; then
            status_summary+="✅ $service_name ($response_time ms) "
        else
            status_summary+="❌ $service_name ($status) "
            all_healthy=false
            
            # Send alert if cooldown period has passed
            if should_send_alert "$service_name"; then
                local message="Service $service_name is $status (response time: ${response_time}ms)"
                if [ "$status" = "unreachable" ]; then
                    send_alert "CRITICAL" "$service_name" "$message"
                else
                    send_alert "WARNING" "$service_name" "$message"
                fi
            fi
        fi
    done
    
    # Overall status
    if [ "$all_healthy" = true ]; then
        printf "\r%s[%s] %s" "$(tput el)" "$(date '+%H:%M:%S')" "$status_summary"
    else
        echo
        log_error "Some services are unhealthy: $status_summary"
    fi
}

# Performance monitoring
monitor_performance() {
    # Check response times
    local api_response=$(check_service_health "api" "http://localhost:8000/health")
    IFS=' ' read -r status response_time <<< "$api_response"
    
    if [ "$status" = "healthy" ] && [ "$response_time" -gt 5000 ]; then # 5 seconds
        if should_send_alert "api-performance"; then
            send_alert "WARNING" "api" "Slow response time: ${response_time}ms"
        fi
    fi
    
    # Check memory usage
    local memory_usage=$(free | awk 'NR==2{printf "%.1f", $3*100/$2}')
    if (( $(echo "$memory_usage > 90" | bc -l) )); then
        if should_send_alert "memory"; then
            send_alert "WARNING" "system" "High memory usage: ${memory_usage}%"
        fi
    fi
    
    # Check disk usage
    local disk_usage=$(df "$PROJECT_ROOT" | awk 'NR==2{print $5}' | cut -d'%' -f1)
    if [ "$disk_usage" -gt 85 ]; then
        if should_send_alert "disk"; then
            send_alert "WARNING" "system" "High disk usage: ${disk_usage}%"
        fi
    fi
}

# Generate monitoring report
generate_report() {
    local report_file="$MONITOR_DIR/report.html"
    local timestamp=$(date)
    
    # Get latest metrics
    local latest_metrics=$(tail -n 1 "$MONITOR_DIR/metrics.json" 2>/dev/null || echo '{}')
    
    cat > "$report_file" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Uveddi Monitoring Report</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .card { background: white; padding: 20px; margin: 10px 0; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .status-healthy { color: #28a745; }
        .status-warning { color: #ffc107; }
        .status-error { color: #dc3545; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background: #f8f9fa; border-radius: 4px; }
        .header { text-align: center; color: #333; }
        .timestamp { color: #666; font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="container">
        <div class="card">
            <h1 class="header">Uveddi Monitoring Dashboard</h1>
            <p class="timestamp">Generated: $timestamp</p>
        </div>
        
        <div class="card">
            <h2>Service Status</h2>
            <div id="service-status">
                <!-- Will be populated by JavaScript -->
            </div>
        </div>
        
        <div class="card">
            <h2>System Metrics</h2>
            <div id="system-metrics">
                <!-- Will be populated by JavaScript -->
            </div>
        </div>
        
        <div class="card">
            <h2>Recent Alerts</h2>
            <div id="recent-alerts">
                <pre>$(tail -n 20 "$LOG_DIR/alerts.log" 2>/dev/null || echo "No alerts found")</pre>
            </div>
        </div>
    </div>
    
    <script>
        // Auto-refresh every 30 seconds
        setTimeout(() => window.location.reload(), 30000);
        
        // Update timestamp
        document.querySelector('.timestamp').textContent = 'Generated: ' + new Date().toLocaleString();
    </script>
</body>
</html>
EOF
    
    log_info "Monitoring report generated: $report_file"
}

# Continuous monitoring loop
continuous_monitoring() {
    log_info "Starting continuous monitoring (interval: ${CHECK_INTERVAL}s)"
    log_info "Press Ctrl+C to stop monitoring"
    
    while true; do
        collect_system_metrics
        monitor_services
        monitor_performance
        
        # Generate report every 10 checks
        local check_count=$(jq -r '.metrics.checks' "$MONITOR_DIR/state.json" 2>/dev/null || echo "0")
        if [ $((check_count % 10)) -eq 0 ]; then
            generate_report
        fi
        
        # Update check count
        if [ -f "$MONITOR_DIR/state.json" ]; then
            local temp_file=$(mktemp)
            jq ".metrics.checks += 1 | .lastCheck = \"$(date -Iseconds)\"" "$MONITOR_DIR/state.json" > "$temp_file" && mv "$temp_file" "$MONITOR_DIR/state.json"
        fi
        
        sleep "$CHECK_INTERVAL"
    done
}

# Show monitoring status
show_status() {
    if [ ! -f "$MONITOR_DIR/state.json" ]; then
        log_error "Monitoring not initialized. Run: $0 start"
        return 1
    fi
    
    echo "Uveddi Monitoring Status"
    echo "========================"
    
    local last_check=$(jq -r '.lastCheck' "$MONITOR_DIR/state.json")
    local total_checks=$(jq -r '.metrics.checks' "$MONITOR_DIR/state.json")
    local total_failures=$(jq -r '.metrics.failures' "$MONITOR_DIR/state.json")
    local total_alerts=$(jq -r '.metrics.alerts' "$MONITOR_DIR/state.json")
    
    echo "Last Check: $last_check"
    echo "Total Checks: $total_checks"
    echo "Total Failures: $total_failures"
    echo "Total Alerts: $total_alerts"
    echo
    
    # Service status
    echo "Service Status:"
    for service in api frontend rendering; do
        local status=$(jq -r ".services.$service.status" "$MONITOR_DIR/state.json")
        printf "  %-12s %s\n" "$service:" "$status"
    done
    
    # Latest metrics
    if [ -f "$MONITOR_DIR/metrics.json" ]; then
        echo
        echo "Latest System Metrics:"
        local latest=$(tail -n 1 "$MONITOR_DIR/metrics.json")
        echo "  CPU Usage: $(echo "$latest" | jq -r '.system.cpu_usage')%"
        echo "  Memory: $(echo "$latest" | jq -r '.system.memory.usage_percent')%"
        echo "  Disk: $(echo "$latest" | jq -r '.system.disk.usage_percent')%"
    fi
}

# Main command handler
main() {
    case "${1:-status}" in
        "start")
            init_monitoring
            continuous_monitoring
            ;;
        "stop")
            pkill -f "monitoring.sh" || true
            log_info "Monitoring stopped"
            ;;
        "status")
            show_status
            ;;
        "report")
            generate_report
            ;;
        "alert-test")
            send_alert "INFO" "test" "This is a test alert"
            ;;
        *)
            echo "Usage: $0 {start|stop|status|report|alert-test}"
            echo
            echo "Commands:"
            echo "  start      - Start continuous monitoring"
            echo "  stop       - Stop monitoring"
            echo "  status     - Show monitoring status"
            echo "  report     - Generate HTML report"
            echo "  alert-test - Send test alert"
            echo
            echo "Environment variables:"
            echo "  ALERT_WEBHOOK - Webhook URL for alerts"
            exit 1
            ;;
    esac
}

# Handle cleanup on exit
trap 'log_info "Monitoring interrupted"; exit 0' SIGINT SIGTERM

main "$@"
