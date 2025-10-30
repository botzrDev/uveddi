#!/bin/bash
# API Endpoint Integration Tests
# Tests all API endpoints to ensure they return appropriate responses

set -e

# Configuration
API_BASE_URL="http://localhost:8000"
API_V1_BASE="$API_BASE_URL/api/v1"
PASSED=0
FAILED=0
TEST_RESULTS=()

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Test helper function
test_endpoint() {
    local endpoint="$1"
    local expected_status="$2"
    local description="$3"
    local method="${4:-GET}"
    local additional_checks="$5"
    
    echo "Testing: $description"
    
    local response
    local http_status
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" "$endpoint")
    else
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" -X "$method" "$endpoint")
    fi
    
    http_status=$(echo "$response" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
    response_body=$(echo "$response" | sed -e 's/HTTPSTATUS:.*//g')
    
    if [ "$http_status" = "$expected_status" ]; then
        # Additional content checks if provided
        if [ -n "$additional_checks" ]; then
            if echo "$response_body" | grep -q "$additional_checks"; then
                log_success "✅ $description (HTTP $http_status, content verified)"
                PASSED=$((PASSED + 1))
                TEST_RESULTS+=("PASS: $description")
                return 0
            else
                log_error "❌ $description (HTTP $http_status OK, but content check failed)"
                log_error "   Expected content containing: $additional_checks"
                log_error "   Got: $response_body"
                FAILED=$((FAILED + 1))
                TEST_RESULTS+=("FAIL: $description - content check failed")
                return 1
            fi
        else
            log_success "✅ $description (HTTP $http_status)"
            PASSED=$((PASSED + 1))
            TEST_RESULTS+=("PASS: $description")
            return 0
        fi
    else
        log_error "❌ $description (HTTP $http_status, expected $expected_status)"
        log_error "   Response: $response_body"
        FAILED=$((FAILED + 1))
        TEST_RESULTS+=("FAIL: $description - HTTP $http_status instead of $expected_status")
        return 1
    fi
}

# Check if API server is running
check_server() {
    log_info "Checking if API server is running..."
    if curl -s "$API_BASE_URL/health" > /dev/null 2>&1; then
        log_success "API server is running on $API_BASE_URL"
        return 0
    else
        log_error "API server is not running on $API_BASE_URL"
        log_error "Please start the server with: ./scripts/process-manager.sh start"
        exit 1
    fi
}

# Main test function
run_tests() {
    echo "========================"
    echo "API Endpoint Integration Tests"
    echo "========================"
    echo ""
    
    check_server
    
    echo ""
    log_info "Running endpoint tests..."
    echo ""
    
    # Test health endpoints
    test_endpoint "$API_BASE_URL/health" "200" "Legacy Health endpoint" "GET" "status.*ok"
    test_endpoint "$API_V1_BASE/health" "200" "API v1 Health endpoint" "GET" "status.*healthy"
    
    # Test metrics endpoint
    test_endpoint "$API_V1_BASE/metrics" "200" "API v1 Metrics endpoint" "GET" "reportsServed"
    
    # Test reports endpoints
    test_endpoint "$API_V1_BASE/reports" "200" "API v1 Reports listing" "GET" "data.*reports"
    test_endpoint "$API_V1_BASE/reports/latest" "200" "API v1 Latest report" "GET" "schemaVersion"
    test_endpoint "$API_V1_BASE/reports/demo" "200" "API v1 Demo report" "GET" "schemaVersion"
    test_endpoint "$API_V1_BASE/reports/demo/graphs/dependency" "200" "Dependency graph endpoint" "GET" "data"
    
    # Test export endpoints
    test_endpoint "$API_V1_BASE/reports/latest/export?format=markdown" "200" "Latest report export (markdown)" "GET" "Uveddi Analysis Report"
    test_endpoint "$API_V1_BASE/reports/demo/export?format=markdown" "200" "Demo report export (markdown)" "GET" "Analysis Report"
    
    # Test security endpoints
    test_endpoint "$API_V1_BASE/security/sarif" "200" "SARIF export endpoint" "GET" "version.*2.1.0"
    
    # Test error cases
    test_endpoint "$API_V1_BASE/reports/nonexistent" "404" "Non-existent report" "GET" "error"
    test_endpoint "$API_V1_BASE/nonexistent" "404" "Non-existent endpoint" "GET" "error"
    
    # Test documentation endpoints
    test_endpoint "$API_BASE_URL/api/docs/structure" "200" "Documentation structure" "GET" "structure"
    test_endpoint "$API_BASE_URL/api/project/info" "200" "Project info endpoint" "GET" "name"
    test_endpoint "$API_BASE_URL/api/analysis/status" "200" "Analysis status endpoint" "GET" "status"
}

# Performance test function
run_performance_tests() {
    echo ""
    log_info "Running basic performance tests..."
    
    # Test response times for key endpoints
    local endpoints=(
        "$API_V1_BASE/health"
        "$API_V1_BASE/reports/latest" 
        "$API_V1_BASE/metrics"
        "$API_V1_BASE/reports"
    )
    
    for endpoint in "${endpoints[@]}"; do
        local response_time
        response_time=$(curl -s -w "%{time_total}" -o /dev/null "$endpoint")
        local response_time_ms
        response_time_ms=$(echo "$response_time * 1000" | bc -l | cut -d. -f1)
        
        if [ "$response_time_ms" -lt 500 ]; then
            log_success "⚡ $endpoint: ${response_time_ms}ms (good)"
        elif [ "$response_time_ms" -lt 1000 ]; then
            log_warning "⚠️  $endpoint: ${response_time_ms}ms (acceptable)"
        else
            log_error "🐌 $endpoint: ${response_time_ms}ms (slow)"
        fi
    done
}

# Concurrent request test
run_concurrent_tests() {
    echo ""
    log_info "Running concurrent request test..."
    
    # Test concurrent requests to health endpoint
    local concurrent_requests=5
    local pids=()
    
    for i in $(seq 1 $concurrent_requests); do
        curl -s "$API_V1_BASE/health" > /tmp/concurrent_test_$i.json &
        pids+=($!)
    done
    
    # Wait for all requests to complete
    local success_count=0
    for pid in "${pids[@]}"; do
        if wait "$pid"; then
            success_count=$((success_count + 1))
        fi
    done
    
    if [ "$success_count" -eq "$concurrent_requests" ]; then
        log_success "✅ Concurrent requests test: $success_count/$concurrent_requests successful"
    else
        log_error "❌ Concurrent requests test: $success_count/$concurrent_requests successful"
    fi
    
    # Cleanup
    rm -f /tmp/concurrent_test_*.json
}

# Generate test report
generate_report() {
    echo ""
    echo "========================"
    echo "Test Summary"
    echo "========================"
    echo ""
    echo "Total tests: $((PASSED + FAILED))"
    echo -e "Passed: ${GREEN}$PASSED${NC}"
    echo -e "Failed: ${RED}$FAILED${NC}"
    
    if [ $FAILED -eq 0 ]; then
        echo ""
        log_success "🎉 All tests passed! API endpoints are working correctly."
        echo ""
        echo "Frontend Integration Status:"
        echo "✅ All required endpoints are implemented"
        echo "✅ Response formats match frontend expectations"
        echo "✅ Error handling is consistent"
        echo "✅ Export functionality working"
        echo ""
        return 0
    else
        echo ""
        log_error "❌ Some tests failed. API endpoints need attention."
        echo ""
        echo "Failed Tests:"
        for result in "${TEST_RESULTS[@]}"; do
            if [[ $result == FAIL:* ]]; then
                echo "  - ${result#FAIL: }"
            fi
        done
        echo ""
        return 1
    fi
}

# Export validation function
validate_export_formats() {
    echo ""
    log_info "Validating export formats..."
    
    # Test markdown export
    local md_content
    md_content=$(curl -s "$API_V1_BASE/reports/demo/export?format=markdown")
    if echo "$md_content" | grep -q "# Uveddi Analysis Report"; then
        log_success "✅ Markdown export format is valid"
    else
        log_error "❌ Markdown export format is invalid"
        FAILED=$((FAILED + 1))
    fi
    
    # Test SARIF export
    local sarif_content
    sarif_content=$(curl -s "$API_V1_BASE/security/sarif")
    if echo "$sarif_content" | jq -e '.version == "2.1.0"' > /dev/null 2>&1; then
        log_success "✅ SARIF export format is valid JSON with correct version"
    else
        log_error "❌ SARIF export format is invalid"
        FAILED=$((FAILED + 1))
    fi
}

# Main execution
main() {
    # Check dependencies
    if ! command -v curl &> /dev/null; then
        log_error "curl is required but not installed"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        log_warning "jq is not installed - some validation tests will be skipped"
    fi
    
    if ! command -v bc &> /dev/null; then
        log_warning "bc is not installed - performance timing will be limited"
    fi
    
    # Run test suites
    run_tests
    
    if command -v bc &> /dev/null; then
        run_performance_tests
    fi
    
    run_concurrent_tests
    
    if command -v jq &> /dev/null; then
        validate_export_formats
    fi
    
    generate_report
    
    # Exit with appropriate code
    if [ $FAILED -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--help|--performance-only|--basic-only]"
        echo ""
        echo "Options:"
        echo "  --help           Show this help message"
        echo "  --performance-only    Run only performance tests"
        echo "  --basic-only     Run only basic endpoint tests"
        echo ""
        echo "Examples:"
        echo "  $0                    # Run all tests"
        echo "  $0 --basic-only       # Quick endpoint validation"
        echo "  $0 --performance-only # Check response times"
        exit 0
        ;;
    --performance-only)
        check_server
        run_performance_tests
        exit 0
        ;;
    --basic-only)
        check_server
        run_tests
        generate_report
        exit $?
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac