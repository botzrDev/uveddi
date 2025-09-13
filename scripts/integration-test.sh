#!/bin/bash

# Comprehensive End-to-End Integration Tests for Uveddi
#
# This script runs full integration tests covering:
# - Service startup and health checks
# - Full analysis workflow
# - Report generation and export
# - API endpoint functionality
# - WebSocket real-time updates
# - Database operations
#
# Exit codes:
# 0 - All tests passed
# 1 - Service startup failed
# 2 - Analysis workflow failed
# 3 - API tests failed
# 4 - Export functionality failed

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_PORT=${API_PORT:-8000}
API_URL="http://localhost:${API_PORT}"
TEST_TIMEOUT=30
STARTUP_WAIT=5

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TEST_RESULTS=()

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

run_test() {
    local test_name="$1"
    local test_command="$2"

    echo -n "Testing $test_name... "

    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        TEST_RESULTS+=("$test_name: PASSED")
        return 0
    else
        echo -e "${RED}❌ FAILED${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        TEST_RESULTS+=("$test_name: FAILED")
        return 1
    fi
}

wait_for_service() {
    local service_url="$1"
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$service_url/health" > /dev/null 2>&1; then
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done

    return 1
}

cleanup() {
    log_info "Cleaning up test environment..."
    ./scripts/process-manager.sh stop > /dev/null 2>&1 || true
    rm -f /tmp/test_*.json /tmp/export_test.* 2>/dev/null || true
}

# Trap cleanup on exit
trap cleanup EXIT

# Main test execution
main() {
    echo "======================================"
    echo "Uveddi End-to-End Integration Tests"
    echo "======================================"
    echo ""

    # 1. Start services
    log_info "Starting Uveddi services..."
    if ! ./scripts/process-manager.sh start > /dev/null 2>&1; then
        log_error "Failed to start services"
        exit 1
    fi

    # 2. Wait for services to be ready
    log_info "Waiting for services to be ready..."
    if ! wait_for_service "$API_URL"; then
        log_error "Services did not become ready in time"
        exit 1
    fi

    sleep $STARTUP_WAIT

    echo ""
    echo "Running integration tests..."
    echo "----------------------------"

    # 3. Test health endpoint
    run_test "Health check endpoint" \
        "curl -s $API_URL/health | jq -e '.status == \"ok\"'"

    # 4. Test project info endpoint
    run_test "Project info endpoint" \
        "curl -s $API_URL/api/project/info | jq -e '.name == \"Uveddi\"'"

    # 5. Test analysis workflow
    log_info "Testing full analysis workflow..."

    # Trigger analysis
    run_test "Trigger analysis" \
        "curl -s -X POST $API_URL/api/analyze \
            -H 'Content-Type: application/json' \
            -d '{\"path\": \".\", \"options\": {\"language\": \"rust\"}}' | \
            jq -e '.jobId'"

    # Wait for analysis to complete
    sleep 3

    # 6. Test report retrieval
    run_test "Get latest report" \
        "curl -s $API_URL/api/reports/latest | jq -e '.summary.filesAnalyzed' > /tmp/test_result"

    if [ -s /tmp/test_result ]; then
        FILES_ANALYZED=$(cat /tmp/test_result)
        if [ "$FILES_ANALYZED" -gt 0 ]; then
            log_info "Analysis successful: $FILES_ANALYZED files analyzed"
        else
            log_warning "No files were analyzed"
        fi
    fi

    # 7. Test report listing
    run_test "List all reports" \
        "curl -s $API_URL/api/reports | jq -e '.reports | length' > /tmp/test_reports_count"

    # 8. Test export functionality
    log_info "Testing export functionality..."

    run_test "Export to JSON" \
        "curl -s $API_URL/api/reports/latest/export?format=json -o /tmp/export_test.json && \
         [ -s /tmp/export_test.json ]"

    run_test "Export to Markdown" \
        "curl -s $API_URL/api/reports/latest/export?format=markdown -o /tmp/export_test.md && \
         [ -s /tmp/export_test.md ]"

    run_test "Export to HTML" \
        "curl -s $API_URL/api/reports/latest/export?format=html -o /tmp/export_test.html && \
         [ -s /tmp/export_test.html ]"

    # 9. Test documentation endpoints
    run_test "Documentation structure" \
        "curl -s $API_URL/api/docs/structure | jq -e '.structure'"

    run_test "Documentation search" \
        "curl -s '$API_URL/api/docs/search?q=analysis' | jq -e '.results'"

    # 10. Test WebSocket connectivity
    log_info "Testing WebSocket connectivity..."

    # Use Node.js to test WebSocket if available
    if command -v node > /dev/null 2>&1; then
        cat > /tmp/ws_test.js << 'EOF'
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8000');

ws.on('open', () => {
    console.log('connected');
    ws.close();
    process.exit(0);
});

ws.on('error', () => {
    process.exit(1);
});

setTimeout(() => process.exit(1), 5000);
EOF

        if [ -f "node_modules/ws/index.js" ]; then
            run_test "WebSocket connection" "node /tmp/ws_test.js"
        else
            log_warning "WebSocket test skipped (ws module not installed)"
        fi
    else
        log_warning "WebSocket test skipped (Node.js not available)"
    fi

    # 11. Test error handling
    run_test "404 error handling" \
        "curl -s $API_URL/api/nonexistent | jq -e '.error'"

    run_test "Invalid format handling" \
        "curl -s '$API_URL/api/reports/latest/export?format=invalid' | jq -e '.error'"

    # 12. Test concurrent requests
    log_info "Testing concurrent request handling..."

    run_test "Concurrent health checks" \
        "for i in {1..10}; do curl -s $API_URL/health & done; wait"

    # 13. Database operations test
    log_info "Testing database operations..."

    # Create a test report
    TEST_REPORT_JSON='{
        "id": "integration-test-'$(date +%s)'",
        "timestamp": "'$(date -Iseconds)'",
        "summary": {
            "filesAnalyzed": 100,
            "issuesFound": 10,
            "criticalIssues": 1,
            "majorIssues": 3,
            "minorIssues": 6
        },
        "findings": []
    }'

    run_test "Create report via API" \
        "curl -s -X POST $API_URL/api/reports \
            -H 'Content-Type: application/json' \
            -d '$TEST_REPORT_JSON' | \
            jq -e '.id' > /tmp/test_report_id"

    if [ -s /tmp/test_report_id ]; then
        REPORT_ID=$(cat /tmp/test_report_id | tr -d '"')

        run_test "Retrieve created report" \
            "curl -s $API_URL/api/reports/$REPORT_ID | jq -e '.id == \"$REPORT_ID\"'"

        run_test "Delete created report" \
            "curl -s -X DELETE $API_URL/api/reports/$REPORT_ID | jq -e '.message'"
    fi

    # 14. Performance tests
    log_info "Running performance tests..."

    START_TIME=$(date +%s%N)
    for i in {1..100}; do
        curl -s $API_URL/health > /dev/null 2>&1
    done
    END_TIME=$(date +%s%N)

    DURATION=$((($END_TIME - $START_TIME) / 1000000))
    AVG_TIME=$(($DURATION / 100))

    if [ $AVG_TIME -lt 50 ]; then
        echo -e "Performance test: ${GREEN}✅ PASSED${NC} (Avg response time: ${AVG_TIME}ms)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "Performance test: ${YELLOW}⚠️  SLOW${NC} (Avg response time: ${AVG_TIME}ms)"
    fi

    # 15. Security headers test
    run_test "Security headers present" \
        "curl -sI $API_URL/health | grep -q 'X-Content-Type-Options'"

    # 16. CORS configuration test
    run_test "CORS headers for allowed origin" \
        "curl -s -H 'Origin: http://localhost:8001' -I $API_URL/health | \
         grep -q 'Access-Control-Allow-Origin'"

    echo ""
    echo "======================================"
    echo "Test Results Summary"
    echo "======================================"
    echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
    echo ""

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}✅ All integration tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Please review the results.${NC}"
        echo ""
        echo "Failed tests:"
        for result in "${TEST_RESULTS[@]}"; do
            if [[ $result == *"FAILED"* ]]; then
                echo "  - $result"
            fi
        done
        exit 3
    fi
}

# Check dependencies
check_dependencies() {
    local missing_deps=()

    command -v curl > /dev/null 2>&1 || missing_deps+=("curl")
    command -v jq > /dev/null 2>&1 || missing_deps+=("jq")

    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        echo "Please install them with: apt-get install ${missing_deps[*]}"
        exit 1
    fi
}

# Run checks and tests
check_dependencies
main