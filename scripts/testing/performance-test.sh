#!/bin/bash
# Performance and load testing for Uveddi deployments
# Tests application performance under various load conditions

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="/tmp/uveddi-performance-test-$(date +%Y%m%d-%H%M%S).log"

# Default test parameters
ENVIRONMENT="${1:-staging}"
NAMESPACE="${NAMESPACE:-uveddi}"
CONCURRENT_USERS="${CONCURRENT_USERS:-10}"
TEST_DURATION="${TEST_DURATION:-60}"
RAMP_UP_TIME="${RAMP_UP_TIME:-30}"
TARGET_URL="${TARGET_URL:-http://localhost:8080}"

# Performance thresholds
MAX_RESPONSE_TIME="${MAX_RESPONSE_TIME:-2000}"  # milliseconds
MIN_SUCCESS_RATE="${MIN_SUCCESS_RATE:-95}"      # percentage
MAX_ERROR_RATE="${MAX_ERROR_RATE:-5}"           # percentage

# Test results
RESULTS_DIR="/tmp/uveddi-perf-results-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    local level="$1"
    shift
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [$level] $*" | tee -a "$LOG_FILE"
}

info() { log "INFO" "$@"; }
warn() { log "WARN" "$@"; }
error() { log "ERROR" "$@"; }
success() { echo -e "${GREEN}✅ $*${NC}"; info "SUCCESS: $*"; }
failure() { echo -e "${RED}❌ $*${NC}"; error "FAILURE: $*"; }
warning() { echo -e "${YELLOW}⚠️  $*${NC}"; warn "WARNING: $*"; }

# Cleanup function
cleanup() {
    local exit_code=$?
    
    # Kill any background processes
    jobs -p | xargs -r kill 2>/dev/null || true
    
    # Cleanup port forwarding
    pkill -f "kubectl.*port-forward.*uveddi" 2>/dev/null || true
    
    # Generate final report
    generate_final_report
    
    if [[ $exit_code -eq 0 ]]; then
        success "Performance tests completed successfully"
    else
        failure "Performance tests failed"
    fi
    
    info "Results directory: $RESULTS_DIR"
    info "Log file: $LOG_FILE"
}

trap cleanup EXIT

validate_prerequisites() {
    info "Validating prerequisites for performance testing..."
    
    local required_commands=("kubectl" "curl" "jq" "nc")
    local missing_commands=()
    
    # Check for load testing tools (prefer hey, fallback to ab)
    if command -v hey &> /dev/null; then
        LOAD_TOOL="hey"
    elif command -v ab &> /dev/null; then
        LOAD_TOOL="ab"
    else
        missing_commands+=("hey or ab (Apache Bench)")
    fi
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_commands+=("$cmd")
        fi
    done
    
    if [[ ${#missing_commands[@]} -gt 0 ]]; then
        failure "Missing required commands: ${missing_commands[*]}"
        info "Install missing tools:"
        info "  hey: go install github.com/rakyll/hey@latest"
        info "  ab: apt-get install apache2-utils (Ubuntu/Debian)"
        exit 1
    fi
    
    # Check kubectl connection
    if ! kubectl cluster-info &> /dev/null; then
        failure "kubectl not configured or cluster not accessible"
        exit 1
    fi
    
    success "Prerequisites validated (using $LOAD_TOOL for load testing)"
}

setup_port_forwarding() {
    info "Setting up port forwarding for performance testing..."
    
    # Kill existing port forwards
    pkill -f "kubectl.*port-forward.*uveddi" 2>/dev/null || true
    sleep 2
    
    # Start port forwarding
    kubectl port-forward service/uveddi 8080:80 -n "$NAMESPACE" &
    local port_forward_pid=$!
    
    # Wait for port forward to be ready
    local attempts=0
    local max_attempts=30
    
    while [[ $attempts -lt $max_attempts ]]; do
        if nc -z localhost 8080; then
            success "Port forwarding is ready"
            return 0
        fi
        sleep 1
        ((attempts++))
    done
    
    failure "Port forwarding failed to become ready"
    exit 1
}

get_baseline_metrics() {
    info "Collecting baseline metrics..."
    
    # Get current pod metrics
    local pods
    pods=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi -o jsonpath='{.items[*].metadata.name}')
    
    echo "Baseline Metrics:" > "$RESULTS_DIR/baseline-metrics.txt"
    echo "=================" >> "$RESULTS_DIR/baseline-metrics.txt"
    echo "Timestamp: $(date)" >> "$RESULTS_DIR/baseline-metrics.txt"
    echo "" >> "$RESULTS_DIR/baseline-metrics.txt"
    
    for pod in $pods; do
        echo "Pod: $pod" >> "$RESULTS_DIR/baseline-metrics.txt"
        kubectl top pod "$pod" -n "$NAMESPACE" --no-headers 2>/dev/null >> "$RESULTS_DIR/baseline-metrics.txt" || echo "Metrics unavailable" >> "$RESULTS_DIR/baseline-metrics.txt"
        echo "" >> "$RESULTS_DIR/baseline-metrics.txt"
    done
    
    # Test basic connectivity
    local response_time
    response_time=$(curl -w "%{time_total}" -s -o /dev/null "$TARGET_URL/health" || echo "failed")
    
    echo "Baseline Response Time: ${response_time}s" >> "$RESULTS_DIR/baseline-metrics.txt"
    
    success "Baseline metrics collected"
}

run_health_endpoint_test() {
    info "Running health endpoint performance test..."
    
    local test_name="health-endpoint"
    local endpoint="$TARGET_URL/health"
    
    if [[ "$LOAD_TOOL" == "hey" ]]; then
        hey -n 1000 -c 10 -t 5 -o csv "$endpoint" > "$RESULTS_DIR/${test_name}-hey.csv" 2>/dev/null || {
            failure "Health endpoint test failed"
            return 1
        }
        
        # Parse hey results
        local avg_response_time
        avg_response_time=$(tail -n +2 "$RESULTS_DIR/${test_name}-hey.csv" | awk -F',' '{sum+=$1; count++} END {print sum/count*1000}')
        
        echo "Health Endpoint Test Results:" > "$RESULTS_DIR/${test_name}-summary.txt"
        echo "Average Response Time: ${avg_response_time}ms" >> "$RESULTS_DIR/${test_name}-summary.txt"
        
    elif [[ "$LOAD_TOOL" == "ab" ]]; then
        ab -n 1000 -c 10 -g "$RESULTS_DIR/${test_name}-ab.tsv" "$endpoint" > "$RESULTS_DIR/${test_name}-ab.txt" 2>&1 || {
            failure "Health endpoint test failed"
            return 1
        }
        
        # Parse ab results
        local avg_response_time
        avg_response_time=$(grep "Time per request:" "$RESULTS_DIR/${test_name}-ab.txt" | head -1 | awk '{print $4}')
        
        echo "Health Endpoint Test Results:" > "$RESULTS_DIR/${test_name}-summary.txt"
        echo "Average Response Time: ${avg_response_time}ms" >> "$RESULTS_DIR/${test_name}-summary.txt"
    fi
    
    success "Health endpoint performance test completed"
}

run_load_test() {
    info "Running main load test..."
    info "Parameters: $CONCURRENT_USERS users, ${TEST_DURATION}s duration"
    
    local test_name="main-load-test"
    local endpoint="$TARGET_URL/health"  # Using health endpoint for load test
    
    # Start resource monitoring in background
    monitor_resources &
    local monitor_pid=$!
    
    if [[ "$LOAD_TOOL" == "hey" ]]; then
        # Calculate total requests based on duration and concurrency
        local total_requests=$((TEST_DURATION * CONCURRENT_USERS))
        
        info "Starting load test with hey..."
        hey -n "$total_requests" -c "$CONCURRENT_USERS" -t 30 -o csv "$endpoint" > "$RESULTS_DIR/${test_name}-hey.csv" 2>/dev/null || {
            failure "Main load test failed"
            kill "$monitor_pid" 2>/dev/null || true
            return 1
        }
        
        # Parse results
        analyze_hey_results "$test_name"
        
    elif [[ "$LOAD_TOOL" == "ab" ]]; then
        local total_requests=$((TEST_DURATION * CONCURRENT_USERS))
        
        info "Starting load test with Apache Bench..."
        ab -n "$total_requests" -c "$CONCURRENT_USERS" -g "$RESULTS_DIR/${test_name}-ab.tsv" "$endpoint" > "$RESULTS_DIR/${test_name}-ab.txt" 2>&1 || {
            failure "Main load test failed"
            kill "$monitor_pid" 2>/dev/null || true
            return 1
        }
        
        # Parse results
        analyze_ab_results "$test_name"
    fi
    
    # Stop resource monitoring
    kill "$monitor_pid" 2>/dev/null || true
    wait "$monitor_pid" 2>/dev/null || true
    
    success "Main load test completed"
}

monitor_resources() {
    info "Starting resource monitoring..."
    
    local monitor_file="$RESULTS_DIR/resource-monitoring.txt"
    echo "Resource Monitoring Log" > "$monitor_file"
    echo "======================" >> "$monitor_file"
    
    while true; do
        {
            echo "Timestamp: $(date)"
            echo "Pod Resources:"
            kubectl top pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi --no-headers 2>/dev/null || echo "Metrics unavailable"
            echo "Node Resources:"
            kubectl top nodes --no-headers 2>/dev/null || echo "Node metrics unavailable"
            echo "---"
        } >> "$monitor_file"
        
        sleep 10
    done
}

analyze_hey_results() {
    local test_name="$1"
    local csv_file="$RESULTS_DIR/${test_name}-hey.csv"
    local summary_file="$RESULTS_DIR/${test_name}-summary.txt"
    
    if [[ ! -f "$csv_file" ]]; then
        failure "Hey results file not found: $csv_file"
        return 1
    fi
    
    # Calculate statistics from CSV
    local total_requests
    total_requests=$(tail -n +2 "$csv_file" | wc -l)
    
    local avg_response_time
    avg_response_time=$(tail -n +2 "$csv_file" | awk -F',' '{sum+=$1} END {print sum/NR*1000}')
    
    local p95_response_time
    p95_response_time=$(tail -n +2 "$csv_file" | awk -F',' '{print $1*1000}' | sort -n | awk -v p=95 'NR==int(NR*p/100){print $0}')
    
    local p99_response_time
    p99_response_time=$(tail -n +2 "$csv_file" | awk -F',' '{print $1*1000}' | sort -n | awk -v p=99 'NR==int(NR*p/100){print $0}')
    
    local success_count
    success_count=$(tail -n +2 "$csv_file" | awk -F',' '$7==200 {count++} END {print count+0}')
    
    local success_rate
    success_rate=$(echo "scale=2; $success_count * 100 / $total_requests" | bc -l)
    
    # Write summary
    {
        echo "Load Test Results Summary"
        echo "========================"
        echo "Test: $test_name"
        echo "Tool: hey"
        echo "Timestamp: $(date)"
        echo ""
        echo "Test Parameters:"
        echo "  Concurrent Users: $CONCURRENT_USERS"
        echo "  Total Requests: $total_requests"
        echo "  Target URL: $TARGET_URL"
        echo ""
        echo "Results:"
        echo "  Success Rate: ${success_rate}%"
        echo "  Average Response Time: ${avg_response_time}ms"
        echo "  95th Percentile: ${p95_response_time}ms"
        echo "  99th Percentile: ${p99_response_time}ms"
        echo ""
        echo "Thresholds:"
        echo "  Max Response Time: ${MAX_RESPONSE_TIME}ms"
        echo "  Min Success Rate: ${MIN_SUCCESS_RATE}%"
        echo ""
        echo "Status:"
    } > "$summary_file"
    
    # Validate against thresholds
    local test_passed=true
    
    if (( $(echo "$avg_response_time > $MAX_RESPONSE_TIME" | bc -l) )); then
        echo "  ❌ FAIL: Average response time exceeds threshold" >> "$summary_file"
        test_passed=false
    else
        echo "  ✅ PASS: Average response time within threshold" >> "$summary_file"
    fi
    
    if (( $(echo "$success_rate < $MIN_SUCCESS_RATE" | bc -l) )); then
        echo "  ❌ FAIL: Success rate below threshold" >> "$summary_file"
        test_passed=false
    else
        echo "  ✅ PASS: Success rate meets threshold" >> "$summary_file"
    fi
    
    if [[ "$test_passed" == "true" ]]; then
        echo "  ✅ OVERALL: PASSED" >> "$summary_file"
        success "Load test passed all thresholds"
    else
        echo "  ❌ OVERALL: FAILED" >> "$summary_file"
        failure "Load test failed one or more thresholds"
    fi
}

analyze_ab_results() {
    local test_name="$1"
    local results_file="$RESULTS_DIR/${test_name}-ab.txt"
    local summary_file="$RESULTS_DIR/${test_name}-summary.txt"
    
    if [[ ! -f "$results_file" ]]; then
        failure "Apache Bench results file not found: $results_file"
        return 1
    fi
    
    # Extract metrics from ab output
    local total_requests
    total_requests=$(grep "Complete requests:" "$results_file" | awk '{print $3}')
    
    local failed_requests
    failed_requests=$(grep "Failed requests:" "$results_file" | awk '{print $3}')
    
    local avg_response_time
    avg_response_time=$(grep "Time per request:" "$results_file" | head -1 | awk '{print $4}')
    
    local requests_per_sec
    requests_per_sec=$(grep "Requests per second:" "$results_file" | awk '{print $4}')
    
    local success_rate
    success_rate=$(echo "scale=2; ($total_requests - $failed_requests) * 100 / $total_requests" | bc -l)
    
    # Write summary
    {
        echo "Load Test Results Summary"
        echo "========================"
        echo "Test: $test_name"
        echo "Tool: Apache Bench"
        echo "Timestamp: $(date)"
        echo ""
        echo "Test Parameters:"
        echo "  Concurrent Users: $CONCURRENT_USERS"
        echo "  Total Requests: $total_requests"
        echo "  Target URL: $TARGET_URL"
        echo ""
        echo "Results:"
        echo "  Success Rate: ${success_rate}%"
        echo "  Failed Requests: $failed_requests"
        echo "  Average Response Time: ${avg_response_time}ms"
        echo "  Requests per Second: $requests_per_sec"
        echo ""
        echo "Thresholds:"
        echo "  Max Response Time: ${MAX_RESPONSE_TIME}ms"
        echo "  Min Success Rate: ${MIN_SUCCESS_RATE}%"
        echo ""
        echo "Status:"
    } > "$summary_file"
    
    # Validate against thresholds
    local test_passed=true
    
    if (( $(echo "$avg_response_time > $MAX_RESPONSE_TIME" | bc -l) )); then
        echo "  ❌ FAIL: Average response time exceeds threshold" >> "$summary_file"
        test_passed=false
    else
        echo "  ✅ PASS: Average response time within threshold" >> "$summary_file"
    fi
    
    if (( $(echo "$success_rate < $MIN_SUCCESS_RATE" | bc -l) )); then
        echo "  ❌ FAIL: Success rate below threshold" >> "$summary_file"
        test_passed=false
    else
        echo "  ✅ PASS: Success rate meets threshold" >> "$summary_file"
    fi
    
    if [[ "$test_passed" == "true" ]]; then
        echo "  ✅ OVERALL: PASSED" >> "$summary_file"
        success "Load test passed all thresholds"
    else
        echo "  ❌ OVERALL: FAILED" >> "$summary_file"
        failure "Load test failed one or more thresholds"
    fi
}

run_stress_test() {
    info "Running stress test with increasing load..."
    
    local stress_levels=(5 10 20 50)
    local stress_summary="$RESULTS_DIR/stress-test-summary.txt"
    
    echo "Stress Test Summary" > "$stress_summary"
    echo "==================" >> "$stress_summary"
    echo "Timestamp: $(date)" >> "$stress_summary"
    echo "" >> "$stress_summary"
    
    for level in "${stress_levels[@]}"; do
        info "Running stress test with $level concurrent users..."
        
        local test_name="stress-test-${level}users"
        local endpoint="$TARGET_URL/health"
        local test_requests=$((level * 20))  # 20 requests per user
        
        if [[ "$LOAD_TOOL" == "hey" ]]; then
            hey -n "$test_requests" -c "$level" -t 10 -o csv "$endpoint" > "$RESULTS_DIR/${test_name}-hey.csv" 2>/dev/null || {
                echo "Stress Level $level users: FAILED" >> "$stress_summary"
                continue
            }
            
            # Quick analysis
            local avg_time
            avg_time=$(tail -n +2 "$RESULTS_DIR/${test_name}-hey.csv" | awk -F',' '{sum+=$1} END {print sum/NR*1000}')
            
            echo "Stress Level $level users: Average ${avg_time}ms" >> "$stress_summary"
            
        elif [[ "$LOAD_TOOL" == "ab" ]]; then
            ab -n "$test_requests" -c "$level" "$endpoint" > "$RESULTS_DIR/${test_name}-ab.txt" 2>&1 || {
                echo "Stress Level $level users: FAILED" >> "$stress_summary"
                continue
            }
            
            local avg_time
            avg_time=$(grep "Time per request:" "$RESULTS_DIR/${test_name}-ab.txt" | head -1 | awk '{print $4}')
            
            echo "Stress Level $level users: Average ${avg_time}ms" >> "$stress_summary"
        fi
        
        # Brief pause between stress levels
        sleep 5
    done
    
    success "Stress test completed"
}

generate_final_report() {
    info "Generating final performance report..."
    
    local final_report="$RESULTS_DIR/final-performance-report.txt"
    
    {
        echo "Uveddi Performance Test Report"
        echo "=============================="
        echo "Generated: $(date)"
        echo "Environment: $ENVIRONMENT"
        echo "Namespace: $NAMESPACE"
        echo ""
        echo "Test Configuration:"
        echo "  Target URL: $TARGET_URL"
        echo "  Concurrent Users: $CONCURRENT_USERS"
        echo "  Test Duration: ${TEST_DURATION}s"
        echo "  Load Tool: $LOAD_TOOL"
        echo ""
        echo "Performance Thresholds:"
        echo "  Max Response Time: ${MAX_RESPONSE_TIME}ms"
        echo "  Min Success Rate: ${MIN_SUCCESS_RATE}%"
        echo "  Max Error Rate: ${MAX_ERROR_RATE}%"
        echo ""
        echo "Test Results:"
        echo "============="
    } > "$final_report"
    
    # Include summaries from individual tests
    for summary_file in "$RESULTS_DIR"/*-summary.txt; do
        if [[ -f "$summary_file" ]]; then
            echo "" >> "$final_report"
            cat "$summary_file" >> "$final_report"
        fi
    done
    
    # Add resource monitoring summary if available
    if [[ -f "$RESULTS_DIR/resource-monitoring.txt" ]]; then
        echo "" >> "$final_report"
        echo "Resource Usage Summary:" >> "$final_report"
        echo "======================" >> "$final_report"
        tail -n 20 "$RESULTS_DIR/resource-monitoring.txt" >> "$final_report"
    fi
    
    # Add recommendations
    {
        echo ""
        echo "Recommendations:"
        echo "==============="
        echo "1. Monitor response times during peak usage"
        echo "2. Set up alerts for response time > ${MAX_RESPONSE_TIME}ms"
        echo "3. Consider scaling if success rate drops below ${MIN_SUCCESS_RATE}%"
        echo "4. Review resource utilization trends"
        echo "5. Run performance tests regularly after deployments"
        echo ""
        echo "Files Generated:"
        echo "==============="
        echo "All test results are available in: $RESULTS_DIR"
        ls -la "$RESULTS_DIR"
    } >> "$final_report"
    
    success "Final performance report generated: $final_report"
}

show_usage() {
    cat << EOF
Usage: $0 [ENVIRONMENT]

ENVIRONMENT: Target environment to test (default: staging)

Environment Variables:
  NAMESPACE=uveddi              Kubernetes namespace (default: uveddi)
  CONCURRENT_USERS=10           Number of concurrent users (default: 10)
  TEST_DURATION=60              Test duration in seconds (default: 60)
  TARGET_URL=http://localhost:8080  Target URL (default: localhost with port forward)
  MAX_RESPONSE_TIME=2000        Max acceptable response time in ms (default: 2000)
  MIN_SUCCESS_RATE=95           Min acceptable success rate % (default: 95)

Examples:
  $0                                        # Test staging with defaults
  $0 production                             # Test production
  CONCURRENT_USERS=20 TEST_DURATION=120 $0  # Extended load test
  TARGET_URL=https://uveddi.com $0          # Test against live URL

EOF
}

# Main function
main() {
    if [[ "${1:-}" =~ ^(-h|--help)$ ]]; then
        show_usage
        exit 0
    fi
    
    info "Starting Uveddi performance tests..."
    info "Environment: $ENVIRONMENT"
    info "Concurrent users: $CONCURRENT_USERS"
    info "Test duration: ${TEST_DURATION}s"
    info "Target URL: $TARGET_URL"
    
    # Setup
    validate_prerequisites
    
    # Setup port forwarding only if using localhost
    if [[ "$TARGET_URL" == "http://localhost:8080" ]]; then
        setup_port_forwarding
    fi
    
    # Run tests
    get_baseline_metrics
    run_health_endpoint_test
    run_load_test
    run_stress_test
    
    info "Performance testing completed"
    info "Results available in: $RESULTS_DIR"
}

# Execute main function
main "$@"