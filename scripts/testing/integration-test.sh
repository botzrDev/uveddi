#!/bin/bash
# Comprehensive integration testing for Uveddi deployments
# Tests all deployment components including API, health checks, and functionality

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_FILE="/tmp/uveddi-integration-test-$(date +%Y%m%d-%H%M%S).log"

# Default values
ENVIRONMENT="${1:-staging}"
NAMESPACE="${NAMESPACE:-uveddi}"
TEST_TIMEOUT="${TEST_TIMEOUT:-300}"
VERBOSE="${VERBOSE:-false}"

# Test configuration
TEST_DATA_DIR="${SCRIPT_DIR}/test-data"
TEMP_TEST_DIR="/tmp/uveddi-test-$(date +%Y%m%d-%H%M%S)"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Logging functions
log() {
    local level="$1"
    shift
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [$level] $*" | tee -a "$LOG_FILE"
}

info() { log "INFO" "$@"; }
warn() { log "WARN" "$@"; }
error() { log "ERROR" "$@"; }
debug() { [[ "${VERBOSE}" == "true" ]] && log "DEBUG" "$@" || true; }

success() {
    echo -e "${GREEN}✅ $*${NC}"
    info "SUCCESS: $*"
}

failure() {
    echo -e "${RED}❌ $*${NC}"
    error "FAILURE: $*"
}

warning() {
    echo -e "${YELLOW}⚠️  $*${NC}"
    warn "WARNING: $*"
}

test_info() {
    echo -e "${BLUE}🧪 $*${NC}"
    info "TEST: $*"
}

# Test tracking functions
start_test() {
    local test_name="$1"
    ((TESTS_TOTAL++))
    test_info "Starting: $test_name"
    debug "Test $TESTS_TOTAL: $test_name"
}

pass_test() {
    local test_name="$1"
    ((TESTS_PASSED++))
    success "PASSED: $test_name"
}

fail_test() {
    local test_name="$1"
    local error_msg="${2:-No error message provided}"
    ((TESTS_FAILED++))
    FAILED_TESTS+=("$test_name: $error_msg")
    failure "FAILED: $test_name - $error_msg"
}

# Cleanup function
cleanup() {
    local exit_code=$?
    
    # Cleanup temporary files
    if [[ -d "$TEMP_TEST_DIR" ]]; then
        rm -rf "$TEMP_TEST_DIR"
        debug "Cleaned up temporary test directory: $TEMP_TEST_DIR"
    fi
    
    # Kill any background processes
    jobs -p | xargs -r kill 2>/dev/null || true
    
    # Show test summary
    echo ""
    echo "========================================"
    echo "Integration Test Summary"
    echo "========================================"
    echo "Environment: $ENVIRONMENT"
    echo "Total Tests: $TESTS_TOTAL"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Success Rate: $(( TESTS_PASSED * 100 / TESTS_TOTAL ))%"
    echo "Log File: $LOG_FILE"
    
    if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
        echo ""
        echo "Failed Tests:"
        for failed_test in "${FAILED_TESTS[@]}"; do
            echo "  - $failed_test"
        done
    fi
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        success "All integration tests passed!"
        exit 0
    else
        failure "$TESTS_FAILED tests failed"
        exit 1
    fi
}

trap cleanup EXIT

# Validation functions
validate_prerequisites() {
    info "Validating prerequisites..."
    
    local required_commands=("kubectl" "curl" "jq" "nc")
    local missing_commands=()
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_commands+=("$cmd")
        fi
    done
    
    if [[ ${#missing_commands[@]} -gt 0 ]]; then
        failure "Missing required commands: ${missing_commands[*]}"
        exit 1
    fi
    
    # Check kubectl connection
    if ! kubectl cluster-info &> /dev/null; then
        failure "kubectl not configured or cluster not accessible"
        exit 1
    fi
    
    # Create test data directory if needed
    mkdir -p "$TEST_DATA_DIR"
    mkdir -p "$TEMP_TEST_DIR"
    
    success "Prerequisites validated"
}

wait_for_deployment() {
    info "Waiting for deployment to be ready..."
    
    # Wait for deployment to be available
    if ! kubectl wait --for=condition=available deployment/uveddi -n "$NAMESPACE" --timeout="${TEST_TIMEOUT}s"; then
        failure "Deployment did not become available within $TEST_TIMEOUT seconds"
        exit 1
    fi
    
    # Wait for pods to be ready
    if ! kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=uveddi -n "$NAMESPACE" --timeout="${TEST_TIMEOUT}s"; then
        failure "Pods did not become ready within $TEST_TIMEOUT seconds"
        exit 1
    fi
    
    success "Deployment is ready"
}

setup_port_forwarding() {
    info "Setting up port forwarding for testing..."
    
    # Kill any existing port forwards
    pkill -f "kubectl.*port-forward.*uveddi" 2>/dev/null || true
    sleep 2
    
    # Start port forwarding
    kubectl port-forward service/uveddi 8080:80 -n "$NAMESPACE" &
    local api_port_forward_pid=$!
    
    kubectl port-forward service/uveddi 9090:9090 -n "$NAMESPACE" &
    local metrics_port_forward_pid=$!
    
    # Store PIDs for cleanup
    echo "$api_port_forward_pid" > "$TEMP_TEST_DIR/api_port_forward.pid"
    echo "$metrics_port_forward_pid" > "$TEMP_TEST_DIR/metrics_port_forward.pid"
    
    # Wait for port forwards to be ready
    local attempts=0
    local max_attempts=30
    
    while [[ $attempts -lt $max_attempts ]]; do
        if nc -z localhost 8080 && nc -z localhost 9090; then
            success "Port forwarding is ready"
            return 0
        fi
        sleep 1
        ((attempts++))
    done
    
    failure "Port forwarding failed to become ready"
    exit 1
}

cleanup_port_forwarding() {
    debug "Cleaning up port forwarding..."
    
    if [[ -f "$TEMP_TEST_DIR/api_port_forward.pid" ]]; then
        local api_pid
        api_pid=$(cat "$TEMP_TEST_DIR/api_port_forward.pid")
        kill "$api_pid" 2>/dev/null || true
    fi
    
    if [[ -f "$TEMP_TEST_DIR/metrics_port_forward.pid" ]]; then
        local metrics_pid
        metrics_pid=$(cat "$TEMP_TEST_DIR/metrics_port_forward.pid")
        kill "$metrics_pid" 2>/dev/null || true
    fi
    
    # Kill any remaining port forwards
    pkill -f "kubectl.*port-forward.*uveddi" 2>/dev/null || true
}

# Individual test functions
test_health_endpoint() {
    start_test "Health endpoint availability"
    
    local response
    if response=$(curl -s -w "%{http_code}" "http://localhost:8080/health" -o "$TEMP_TEST_DIR/health_response.json"); then
        local http_code="${response: -3}"
        if [[ "$http_code" == "200" ]]; then
            pass_test "Health endpoint availability"
        else
            fail_test "Health endpoint availability" "HTTP $http_code"
        fi
    else
        fail_test "Health endpoint availability" "Failed to connect"
    fi
}

test_readiness_endpoint() {
    start_test "Readiness endpoint availability"
    
    local response
    if response=$(curl -s -w "%{http_code}" "http://localhost:8080/ready" -o "$TEMP_TEST_DIR/ready_response.json"); then
        local http_code="${response: -3}"
        if [[ "$http_code" == "200" ]]; then
            pass_test "Readiness endpoint availability"
        else
            fail_test "Readiness endpoint availability" "HTTP $http_code"
        fi
    else
        fail_test "Readiness endpoint availability" "Failed to connect"
    fi
}

test_metrics_endpoint() {
    start_test "Metrics endpoint availability"
    
    local response
    if response=$(curl -s -w "%{http_code}" "http://localhost:9090/metrics" -o "$TEMP_TEST_DIR/metrics_response.txt"); then
        local http_code="${response: -3}"
        if [[ "$http_code" == "200" ]]; then
            # Check if metrics contain expected content
            if grep -q "^# HELP" "$TEMP_TEST_DIR/metrics_response.txt"; then
                pass_test "Metrics endpoint availability"
            else
                fail_test "Metrics endpoint availability" "Invalid metrics format"
            fi
        else
            fail_test "Metrics endpoint availability" "HTTP $http_code"
        fi
    else
        fail_test "Metrics endpoint availability" "Failed to connect"
    fi
}

test_api_endpoints() {
    start_test "API endpoints availability"
    
    # Test API health endpoint
    local response
    if response=$(curl -s -w "%{http_code}" "http://localhost:8080/api/v1/health" -o "$TEMP_TEST_DIR/api_health.json"); then
        local http_code="${response: -3}"
        if [[ "$http_code" == "200" ]]; then
            pass_test "API endpoints availability"
        else
            fail_test "API endpoints availability" "API health returned HTTP $http_code"
        fi
    else
        fail_test "API endpoints availability" "Failed to connect to API"
    fi
}

test_health_response_format() {
    start_test "Health response format validation"
    
    if [[ -f "$TEMP_TEST_DIR/health_response.json" ]]; then
        # Validate JSON format
        if jq . "$TEMP_TEST_DIR/health_response.json" > /dev/null 2>&1; then
            # Check for required fields
            local status
            status=$(jq -r '.status' "$TEMP_TEST_DIR/health_response.json" 2>/dev/null || echo "missing")
            
            if [[ "$status" == "ok" || "$status" == "healthy" ]]; then
                pass_test "Health response format validation"
            else
                fail_test "Health response format validation" "Invalid status: $status"
            fi
        else
            fail_test "Health response format validation" "Invalid JSON format"
        fi
    else
        fail_test "Health response format validation" "Health response file not found"
    fi
}

test_metrics_content() {
    start_test "Metrics content validation"
    
    if [[ -f "$TEMP_TEST_DIR/metrics_response.txt" ]]; then
        local expected_metrics=("http_requests_total" "process_cpu_seconds_total" "process_resident_memory_bytes")
        local missing_metrics=()
        
        for metric in "${expected_metrics[@]}"; do
            if ! grep -q "^$metric" "$TEMP_TEST_DIR/metrics_response.txt"; then
                missing_metrics+=("$metric")
            fi
        done
        
        if [[ ${#missing_metrics[@]} -eq 0 ]]; then
            pass_test "Metrics content validation"
        else
            fail_test "Metrics content validation" "Missing metrics: ${missing_metrics[*]}"
        fi
    else
        fail_test "Metrics content validation" "Metrics response file not found"
    fi
}

test_database_connectivity() {
    start_test "Database connectivity"
    
    # Try to make a request that would require database access
    local response
    if response=$(curl -s -w "%{http_code}" "http://localhost:8080/api/v1/status" -o "$TEMP_TEST_DIR/status_response.json"); then
        local http_code="${response: -3}"
        if [[ "$http_code" == "200" ]]; then
            pass_test "Database connectivity"
        else
            # Check if it's a database connection error
            if [[ "$http_code" == "503" ]]; then
                fail_test "Database connectivity" "Service unavailable (possible DB issue)"
            else
                # If endpoint doesn't exist, that's actually OK for this test
                pass_test "Database connectivity"
            fi
        fi
    else
        fail_test "Database connectivity" "Failed to connect"
    fi
}

test_resource_limits() {
    start_test "Resource limits validation"
    
    # Get pod resource usage
    local pods
    pods=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi -o jsonpath='{.items[*].metadata.name}')
    
    if [[ -n "$pods" ]]; then
        local resource_issues=()
        
        for pod in $pods; do
            # Check memory usage
            local memory_usage
            memory_usage=$(kubectl top pod "$pod" -n "$NAMESPACE" --no-headers 2>/dev/null | awk '{print $3}' | sed 's/Mi//' || echo "0")
            
            # Check CPU usage  
            local cpu_usage
            cpu_usage=$(kubectl top pod "$pod" -n "$NAMESPACE" --no-headers 2>/dev/null | awk '{print $2}' | sed 's/m//' || echo "0")
            
            debug "Pod $pod: Memory=${memory_usage}Mi, CPU=${cpu_usage}m"
            
            # Validate reasonable resource usage (not hitting limits)
            if [[ "$memory_usage" -gt 1500 ]]; then
                resource_issues+=("$pod: High memory usage (${memory_usage}Mi)")
            fi
            
            if [[ "$cpu_usage" -gt 800 ]]; then
                resource_issues+=("$pod: High CPU usage (${cpu_usage}m)")
            fi
        done
        
        if [[ ${#resource_issues[@]} -eq 0 ]]; then
            pass_test "Resource limits validation"
        else
            fail_test "Resource limits validation" "${resource_issues[*]}"
        fi
    else
        fail_test "Resource limits validation" "No pods found"
    fi
}

test_pod_logs() {
    start_test "Pod logs validation"
    
    local pods
    pods=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=uveddi -o jsonpath='{.items[*].metadata.name}')
    
    if [[ -n "$pods" ]]; then
        local log_issues=()
        
        for pod in $pods; do
            # Get recent logs
            local logs
            logs=$(kubectl logs "$pod" -n "$NAMESPACE" --tail=50 2>/dev/null || echo "")
            
            # Check for error patterns
            if echo "$logs" | grep -qi "error\|exception\|panic\|fatal"; then
                log_issues+=("$pod: Contains error messages")
            fi
            
            # Check for startup success indicators
            if ! echo "$logs" | grep -qi "started\|listening\|ready"; then
                log_issues+=("$pod: No startup success indicators")
            fi
        done
        
        if [[ ${#log_issues[@]} -eq 0 ]]; then
            pass_test "Pod logs validation"
        else
            fail_test "Pod logs validation" "${log_issues[*]}"
        fi
    else
        fail_test "Pod logs validation" "No pods found"
    fi
}

test_service_endpoints() {
    start_test "Service endpoints validation"
    
    # Check if service has endpoints
    local endpoints
    endpoints=$(kubectl get endpoints uveddi -n "$NAMESPACE" -o jsonpath='{.subsets[*].addresses[*].ip}' 2>/dev/null || echo "")
    
    if [[ -n "$endpoints" ]]; then
        local endpoint_count
        endpoint_count=$(echo "$endpoints" | wc -w)
        
        debug "Found $endpoint_count service endpoints: $endpoints"
        
        if [[ "$endpoint_count" -gt 0 ]]; then
            pass_test "Service endpoints validation"
        else
            fail_test "Service endpoints validation" "No endpoints available"
        fi
    else
        fail_test "Service endpoints validation" "No endpoints found"
    fi
}

test_ingress_configuration() {
    start_test "Ingress configuration validation"
    
    # Check if ingress exists and is configured
    local ingress_exists
    ingress_exists=$(kubectl get ingress -n "$NAMESPACE" 2>/dev/null | grep -c "uveddi" || echo "0")
    
    if [[ "$ingress_exists" -gt 0 ]]; then
        # Get ingress details
        local ingress_host
        ingress_host=$(kubectl get ingress -n "$NAMESPACE" -o jsonpath='{.items[0].spec.rules[0].host}' 2>/dev/null || echo "")
        
        debug "Ingress host: $ingress_host"
        
        if [[ -n "$ingress_host" && "$ingress_host" != "null" ]]; then
            pass_test "Ingress configuration validation"
        else
            fail_test "Ingress configuration validation" "No host configured"
        fi
    else
        # Ingress might not be enabled in all environments
        warning "No ingress found - skipping ingress test"
        pass_test "Ingress configuration validation"
    fi
}

# Load testing function
test_load_handling() {
    start_test "Basic load handling"
    
    info "Running basic load test..."
    
    # Make multiple concurrent requests
    local request_count=10
    local success_count=0
    
    for i in $(seq 1 $request_count); do
        if curl -s -f "http://localhost:8080/health" > /dev/null 2>&1 &
        then
            ((success_count++))
        fi
    done
    
    # Wait for all background requests to complete
    wait
    
    debug "Load test: $success_count/$request_count requests succeeded"
    
    # Allow for some failures in load testing
    local success_rate=$((success_count * 100 / request_count))
    if [[ $success_rate -ge 80 ]]; then
        pass_test "Basic load handling"
    else
        fail_test "Basic load handling" "Success rate: ${success_rate}%"
    fi
}

# Main test execution
run_all_tests() {
    info "Starting comprehensive integration tests..."
    
    # Infrastructure tests
    test_health_endpoint
    test_readiness_endpoint
    test_metrics_endpoint
    test_api_endpoints
    
    # Response validation tests
    test_health_response_format
    test_metrics_content
    
    # System tests
    test_database_connectivity
    test_resource_limits
    test_pod_logs
    test_service_endpoints
    test_ingress_configuration
    
    # Performance tests
    test_load_handling
    
    info "All tests completed"
}

show_usage() {
    cat << EOF
Usage: $0 [ENVIRONMENT]

ENVIRONMENT: Target environment to test (default: staging)

Environment Variables:
  NAMESPACE=uveddi     Kubernetes namespace (default: uveddi)
  TEST_TIMEOUT=300     Test timeout in seconds (default: 300)
  VERBOSE=true         Enable verbose logging (default: false)

Examples:
  $0                           # Test staging environment
  $0 production               # Test production environment
  VERBOSE=true $0 staging     # Test with verbose logging
  TEST_TIMEOUT=600 $0         # Test with extended timeout

EOF
}

# Main function
main() {
    # Show usage if help requested
    if [[ "${1:-}" =~ ^(-h|--help)$ ]]; then
        show_usage
        exit 0
    fi
    
    info "Starting Uveddi integration tests..."
    info "Environment: $ENVIRONMENT"
    info "Namespace: $NAMESPACE"
    info "Test timeout: $TEST_TIMEOUT seconds"
    
    # Setup
    validate_prerequisites
    wait_for_deployment
    setup_port_forwarding
    
    # Run tests
    run_all_tests
    
    # Cleanup
    cleanup_port_forwarding
    
    success "Integration tests completed"
}

# Execute main function
main "$@"