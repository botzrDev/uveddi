#!/bin/bash
set -euo pipefail

# Load Testing Script for Uveddi Production
# Comprehensive performance testing with various scenarios

# Configuration
NAMESPACE="${NAMESPACE:-uveddi-prod}"
BASE_URL="${BASE_URL:-https://uveddi.com}"
DURATION="${DURATION:-60}"  # seconds
CONCURRENT_USERS="${CONCURRENT_USERS:-50}"
RESULTS_DIR="${RESULTS_DIR:-./load-test-results}"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Create results directory
mkdir -p "$RESULTS_DIR"

# Logging functions
log() { echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warning() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."

    # Check for required tools
    local tools=("ab" "curl" "jq")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            warning "$tool is not installed, some tests may be skipped"
        fi
    done

    # Check if k6 is installed for advanced testing
    if command -v k6 &> /dev/null; then
        HAS_K6=true
        success "k6 found - advanced load testing available"
    else
        HAS_K6=false
        warning "k6 not found - install for advanced load testing"
    fi

    success "Prerequisites check completed"
}

# Basic load test with Apache Bench
basic_load_test() {
    log "Running basic load test..."

    local endpoints=(
        "/api/v1/health"
        "/api/v1/reports"
        "/api/v1/metrics"
    )

    for endpoint in "${endpoints[@]}"; do
        log "Testing $endpoint..."

        ab -n 1000 -c "$CONCURRENT_USERS" \
           -g "$RESULTS_DIR/ab-$endpoint-$TIMESTAMP.tsv" \
           "$BASE_URL$endpoint" > "$RESULTS_DIR/ab-$endpoint-$TIMESTAMP.txt" 2>&1 || true

        # Parse results
        if [[ -f "$RESULTS_DIR/ab-$endpoint-$TIMESTAMP.txt" ]]; then
            local req_per_sec=$(grep "Requests per second" "$RESULTS_DIR/ab-$endpoint-$TIMESTAMP.txt" | awk '{print $4}')
            local time_per_req=$(grep "Time per request.*mean" "$RESULTS_DIR/ab-$endpoint-$TIMESTAMP.txt" | head -1 | awk '{print $4}')
            local failed=$(grep "Failed requests" "$RESULTS_DIR/ab-$endpoint-$TIMESTAMP.txt" | awk '{print $3}')

            echo "  Requests/sec: $req_per_sec"
            echo "  Mean time/req: ${time_per_req}ms"
            echo "  Failed requests: $failed"
        fi
    done

    success "Basic load test completed"
}

# Create k6 test script
create_k6_script() {
    cat > "$RESULTS_DIR/k6-test-$TIMESTAMP.js" <<'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },  // Ramp up to 10 users
    { duration: '1m', target: 50 },   // Ramp up to 50 users
    { duration: '2m', target: 50 },   // Stay at 50 users
    { duration: '30s', target: 100 }, // Spike to 100 users
    { duration: '1m', target: 100 },  // Stay at 100 users
    { duration: '30s', target: 0 },   // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests under 500ms
    errors: ['rate<0.05'],             // Error rate under 5%
  },
};

const BASE_URL = __ENV.BASE_URL || 'https://uveddi.com';

// Test scenarios
export default function () {
  // Scenario 1: Health check
  let healthRes = http.get(`${BASE_URL}/api/v1/health`);
  check(healthRes, {
    'health check status is 200': (r) => r.status === 200,
    'health check response time < 200ms': (r) => r.timings.duration < 200,
  });
  errorRate.add(healthRes.status !== 200);

  sleep(1);

  // Scenario 2: Get reports
  let reportsRes = http.get(`${BASE_URL}/api/v1/reports`);
  check(reportsRes, {
    'reports status is 200': (r) => r.status === 200,
    'reports response time < 500ms': (r) => r.timings.duration < 500,
  });
  errorRate.add(reportsRes.status !== 200);

  sleep(2);

  // Scenario 3: Analysis request (if applicable)
  let analysisPayload = JSON.stringify({
    repository: 'https://github.com/example/repo',
    branch: 'main',
  });

  let analysisParams = {
    headers: { 'Content-Type': 'application/json' },
  };

  let analysisRes = http.post(
    `${BASE_URL}/api/v1/analyze`,
    analysisPayload,
    analysisParams
  );

  check(analysisRes, {
    'analysis accepted': (r) => r.status === 202 || r.status === 200,
    'analysis response time < 1000ms': (r) => r.timings.duration < 1000,
  });

  sleep(3);

  // Scenario 4: Metrics endpoint
  let metricsRes = http.get(`${BASE_URL}/metrics`);
  check(metricsRes, {
    'metrics available': (r) => r.status === 200,
  });

  sleep(1);
}

// Lifecycle hooks
export function setup() {
  console.log('Load test starting...');

  // Verify service is up
  let res = http.get(`${BASE_URL}/api/v1/health`);
  if (res.status !== 200) {
    throw new Error(`Service not healthy: ${res.status}`);
  }
}

export function teardown(data) {
  console.log('Load test completed');
}
EOF
}

# Advanced k6 load test
advanced_load_test() {
    if [[ "$HAS_K6" != "true" ]]; then
        warning "k6 not installed, skipping advanced load test"
        return
    fi

    log "Running advanced k6 load test..."

    # Create test script
    create_k6_script

    # Run k6 test
    k6 run \
        --out json="$RESULTS_DIR/k6-results-$TIMESTAMP.json" \
        --summary-export="$RESULTS_DIR/k6-summary-$TIMESTAMP.json" \
        -e BASE_URL="$BASE_URL" \
        "$RESULTS_DIR/k6-test-$TIMESTAMP.js"

    # Parse summary
    if [[ -f "$RESULTS_DIR/k6-summary-$TIMESTAMP.json" ]]; then
        log "Test Summary:"
        jq '.metrics.http_req_duration' "$RESULTS_DIR/k6-summary-$TIMESTAMP.json"
    fi

    success "Advanced load test completed"
}

# Stress test - find breaking point
stress_test() {
    log "Running stress test to find breaking point..."

    local users=10
    local increment=10
    local max_users=500
    local breaking_point=0

    while [[ $users -le $max_users ]]; do
        log "Testing with $users concurrent users..."

        # Run test
        local output=$(ab -n 100 -c "$users" -t 10 "$BASE_URL/api/v1/health" 2>&1)

        # Check for failures
        local failed=$(echo "$output" | grep "Failed requests" | awk '{print $3}')
        local error_rate=$(echo "$output" | grep "Non-2xx responses" | awk '{print $3}')

        if [[ "$failed" -gt 0 ]] || [[ "$error_rate" -gt 0 ]]; then
            breaking_point=$users
            warning "Breaking point found at $users concurrent users"
            break
        fi

        # Check response time
        local mean_time=$(echo "$output" | grep "Time per request.*mean" | head -1 | awk '{print $4}')
        if (( $(echo "$mean_time > 1000" | bc -l) )); then
            warning "Response time degraded at $users users (${mean_time}ms)"
        fi

        users=$((users + increment))
    done

    if [[ $breaking_point -eq 0 ]]; then
        success "System handled up to $max_users concurrent users"
    else
        warning "System breaking point: $breaking_point concurrent users"
    fi
}

# Endurance test
endurance_test() {
    log "Running endurance test for $DURATION seconds..."

    # Run sustained load
    ab -n 999999 -c "$CONCURRENT_USERS" -t "$DURATION" \
       "$BASE_URL/api/v1/health" > "$RESULTS_DIR/endurance-$TIMESTAMP.txt" 2>&1

    # Monitor resource usage during test
    if command -v kubectl &> /dev/null; then
        log "Monitoring resource usage..."

        for i in $(seq 1 $((DURATION / 10))); do
            kubectl top pods -n "$NAMESPACE" >> "$RESULTS_DIR/resource-usage-$TIMESTAMP.txt" 2>&1
            sleep 10
        done
    fi

    success "Endurance test completed"
}

# API endpoint test
api_test() {
    log "Testing API endpoints..."

    # Test different HTTP methods
    local endpoints=(
        "GET:/api/v1/health"
        "GET:/api/v1/reports"
        "GET:/api/v1/metrics"
        "POST:/api/v1/analyze"
        "GET:/api/v1/status"
    )

    for endpoint in "${endpoints[@]}"; do
        local method="${endpoint%%:*}"
        local path="${endpoint#*:}"

        log "Testing $method $path..."

        if [[ "$method" == "GET" ]]; then
            response=$(curl -s -o /dev/null -w "%{http_code},%{time_total}" "$BASE_URL$path")
        else
            response=$(curl -s -o /dev/null -w "%{http_code},%{time_total}" -X "$method" \
                      -H "Content-Type: application/json" \
                      -d '{"test": true}' \
                      "$BASE_URL$path")
        fi

        local status_code="${response%%,*}"
        local response_time="${response#*,}"

        if [[ "$status_code" -ge 200 ]] && [[ "$status_code" -lt 300 ]]; then
            success "$method $path: ${status_code} (${response_time}s)"
        elif [[ "$status_code" -eq 405 ]]; then
            warning "$method $path: Method not allowed"
        else
            error "$method $path: ${status_code} (${response_time}s)"
        fi
    done
}

# Generate report
generate_report() {
    log "Generating load test report..."

    local report_file="$RESULTS_DIR/report-$TIMESTAMP.md"

    cat > "$report_file" <<EOF
# Load Test Report

**Date:** $(date)
**Base URL:** $BASE_URL
**Duration:** $DURATION seconds
**Concurrent Users:** $CONCURRENT_USERS

## Test Results

### Basic Load Test
$(if [[ -f "$RESULTS_DIR/ab-/api/v1/health-$TIMESTAMP.txt" ]]; then
    grep -E "Requests per second|Time per request.*mean|Failed requests" "$RESULTS_DIR/ab-/api/v1/health-$TIMESTAMP.txt"
fi)

### Resource Usage
$(if [[ -f "$RESULTS_DIR/resource-usage-$TIMESTAMP.txt" ]]; then
    tail -10 "$RESULTS_DIR/resource-usage-$TIMESTAMP.txt"
fi)

## Recommendations

1. **Scaling:** Based on the load test results, consider scaling to handle peak traffic
2. **Caching:** Implement caching for frequently accessed endpoints
3. **Database:** Optimize database queries showing high latency
4. **Monitoring:** Set up alerts for response time degradation

## Files Generated

- Basic test results: ab-*-$TIMESTAMP.txt
- Resource usage: resource-usage-$TIMESTAMP.txt
- Endurance test: endurance-$TIMESTAMP.txt
$(if [[ "$HAS_K6" == "true" ]]; then
    echo "- k6 results: k6-results-$TIMESTAMP.json"
    echo "- k6 summary: k6-summary-$TIMESTAMP.json"
fi)

EOF

    success "Report generated: $report_file"
}

# Monitor during test
monitor_test() {
    log "Starting monitoring during load test..."

    # Monitor pods
    watch -n 5 "kubectl top pods -n $NAMESPACE" &
    local watch_pid=$!

    # Run load test
    basic_load_test

    # Stop monitoring
    kill $watch_pid 2>/dev/null

    success "Monitoring completed"
}

# Main menu
show_menu() {
    echo "==========================================="
    echo "       Uveddi Load Testing Suite"
    echo "==========================================="
    echo "1. Quick Test (Basic load test)"
    echo "2. Full Test Suite (All tests)"
    echo "3. Stress Test (Find breaking point)"
    echo "4. Endurance Test (Long-running)"
    echo "5. API Endpoint Test"
    echo "6. Advanced k6 Test"
    echo "7. Monitor During Test"
    echo "8. Generate Report"
    echo "0. Exit"
    echo "==========================================="
}

# Full test suite
full_test_suite() {
    log "Running full test suite..."

    check_prerequisites
    basic_load_test
    api_test
    stress_test
    endurance_test
    advanced_load_test
    generate_report

    success "Full test suite completed"
}

# Parse command line arguments
case "${1:-}" in
    quick)
        check_prerequisites
        basic_load_test
        generate_report
        ;;
    full)
        full_test_suite
        ;;
    stress)
        check_prerequisites
        stress_test
        ;;
    endurance)
        check_prerequisites
        endurance_test
        ;;
    api)
        api_test
        ;;
    k6)
        check_prerequisites
        advanced_load_test
        ;;
    monitor)
        monitor_test
        ;;
    report)
        generate_report
        ;;
    --help)
        echo "Usage: $0 [COMMAND]"
        echo "Commands:"
        echo "  quick     Run quick load test"
        echo "  full      Run full test suite"
        echo "  stress    Find breaking point"
        echo "  endurance Long-running test"
        echo "  api       Test API endpoints"
        echo "  k6        Advanced k6 test"
        echo "  monitor   Monitor during test"
        echo "  report    Generate report"
        echo ""
        echo "Environment variables:"
        echo "  BASE_URL           Target URL (default: https://uveddi.com)"
        echo "  CONCURRENT_USERS   Number of concurrent users (default: 50)"
        echo "  DURATION          Test duration in seconds (default: 60)"
        exit 0
        ;;
    *)
        # Interactive mode
        while true; do
            show_menu
            echo -n "Select option: "
            read option

            case $option in
                1) check_prerequisites; basic_load_test; generate_report ;;
                2) full_test_suite ;;
                3) check_prerequisites; stress_test ;;
                4) check_prerequisites; endurance_test ;;
                5) api_test ;;
                6) check_prerequisites; advanced_load_test ;;
                7) monitor_test ;;
                8) generate_report ;;
                0) exit 0 ;;
                *) warning "Invalid option" ;;
            esac

            echo
            echo "Press Enter to continue..."
            read
        done
        ;;
esac