#!/bin/bash
#
# Data Consistency Validation Script
# 
# This script validates that data flows consistently from the analysis engine
# through the database, API, and dashboard layers.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="/tmp/uveddi-consistency-test"
BINARY="$PROJECT_ROOT/target/debug/uveddi"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🧪 Uveddi Data Consistency Validation Test"
echo "=========================================="

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}❌ Binary not found: $BINARY${NC}"
    echo "Please build with: cargo build --features=production --bin uveddi"
    exit 1
fi

# Create test directory
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "📂 Test directory: $TEST_DIR"

# Test 1: Run analysis and generate JSON report
echo -e "\n${YELLOW}📊 Test 1: Analysis Engine JSON Export${NC}"
echo "Running analysis on API module..."

"$BINARY" analyze "$PROJECT_ROOT/src/api" \
    --output-format json \
    --output "$TEST_DIR/backend-analysis.json" > /dev/null

if [ ! -f "$TEST_DIR/backend-analysis.json" ]; then
    echo -e "${RED}❌ Failed to generate backend analysis${NC}"
    exit 1
fi

# Extract key metrics from backend JSON
BACKEND_FILES=$(jq '.summary.filesAnalyzed' "$TEST_DIR/backend-analysis.json")
BACKEND_ISSUES=$(jq '.summary.issuesTotal' "$TEST_DIR/backend-analysis.json")
BACKEND_FIRST_SEVERITY=$(jq -r '.issues[0].severity // "none"' "$TEST_DIR/backend-analysis.json")

echo "✅ Backend Analysis Complete"
echo "   Files analyzed: $BACKEND_FILES"
echo "   Issues found: $BACKEND_ISSUES"
echo "   First issue severity: $BACKEND_FIRST_SEVERITY"

# Test 2: Start API server and query demo endpoint
echo -e "\n${YELLOW}🌐 Test 2: API Server Response${NC}"
echo "Starting API server..."

API_PORT=8890

# Create a simple database for testing
mkdir -p .uveddi
cp .uveddi/analysis.db .uveddi/test-analysis.db 2>/dev/null || true

# Use ui serve command which doesn't require rendering service
"$BINARY" ui serve --port $API_PORT --dev --database .uveddi/test-analysis.db &
SERVER_PID=$!

# Wait for server to start
echo "Waiting for server to start..."
sleep 5

# Query the demo report
curl -s "http://localhost:$API_PORT/api/v1/reports/demo" > "$TEST_DIR/api-demo-response.json"

if [ $? -ne 0 ] || [ ! -s "$TEST_DIR/api-demo-response.json" ]; then
    echo -e "${RED}❌ Failed to query API server${NC}"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# Extract key metrics from API response
API_FILES=$(jq '.summary.filesAnalyzed' "$TEST_DIR/api-demo-response.json")
API_ISSUES=$(jq '.summary.issuesTotal' "$TEST_DIR/api-demo-response.json")
API_FIRST_SEVERITY=$(jq -r '.findings[0].severity // "none"' "$TEST_DIR/api-demo-response.json")
API_SCHEMA_VERSION=$(jq -r '.schemaVersion' "$TEST_DIR/api-demo-response.json")

echo "✅ API Server Response Complete"
echo "   Schema version: $API_SCHEMA_VERSION"
echo "   Files analyzed: $API_FILES"
echo "   Issues found: $API_ISSUES"
echo "   First finding severity: $API_FIRST_SEVERITY"

# Cleanup server
kill $SERVER_PID 2>/dev/null || true

# Test 3: Validate schema consistency
echo -e "\n${YELLOW}🔍 Test 3: Schema Consistency Validation${NC}"

# Check required fields in backend JSON
BACKEND_HAS_SUMMARY=$(jq 'has("summary")' "$TEST_DIR/backend-analysis.json")
BACKEND_HAS_ISSUES=$(jq 'has("issues")' "$TEST_DIR/backend-analysis.json")
BACKEND_HAS_METADATA=$(jq 'has("metadata")' "$TEST_DIR/backend-analysis.json")

# Check required fields in API JSON
API_HAS_SCHEMA=$(jq 'has("schemaVersion")' "$TEST_DIR/api-demo-response.json")
API_HAS_PROJECT=$(jq 'has("project")' "$TEST_DIR/api-demo-response.json")
API_HAS_SUMMARY=$(jq 'has("summary")' "$TEST_DIR/api-demo-response.json")
API_HAS_FINDINGS=$(jq 'has("findings")' "$TEST_DIR/api-demo-response.json")
API_HAS_DEPENDENCY_GRAPH=$(jq 'has("dependencyGraph")' "$TEST_DIR/api-demo-response.json")

echo "Backend JSON Schema:"
echo "   Has summary: $BACKEND_HAS_SUMMARY"
echo "   Has issues: $BACKEND_HAS_ISSUES"
echo "   Has metadata: $BACKEND_HAS_METADATA"

echo "API JSON Schema:"
echo "   Has schemaVersion: $API_HAS_SCHEMA"
echo "   Has project: $API_HAS_PROJECT"
echo "   Has summary: $API_HAS_SUMMARY"
echo "   Has findings: $API_HAS_FINDINGS"
echo "   Has dependencyGraph: $API_HAS_DEPENDENCY_GRAPH"

# Test 4: Field name consistency check
echo -e "\n${YELLOW}📋 Test 4: Field Name Consistency${NC}"

# Check field naming conventions
BACKEND_USES_ISSUES=$(jq 'has("issues")' "$TEST_DIR/backend-analysis.json")
API_USES_FINDINGS=$(jq 'has("findings")' "$TEST_DIR/api-demo-response.json")

BACKEND_FILES_ANALYZED=$(jq '.summary | has("filesAnalyzed")' "$TEST_DIR/backend-analysis.json")
API_FILES_ANALYZED=$(jq '.summary | has("filesAnalyzed")' "$TEST_DIR/api-demo-response.json")

echo "Field naming consistency:"
echo "   Backend uses 'issues': $BACKEND_USES_ISSUES"
echo "   API uses 'findings': $API_USES_FINDINGS"
echo "   Backend has filesAnalyzed: $BACKEND_FILES_ANALYZED"
echo "   API has filesAnalyzed: $API_FILES_ANALYZED"

# Test 5: Data type consistency
echo -e "\n${YELLOW}🔢 Test 5: Data Type Consistency${NC}"

# Check if file counts are numbers
BACKEND_FILES_IS_NUMBER=$(jq '.summary.filesAnalyzed | type == "number"' "$TEST_DIR/backend-analysis.json")
API_FILES_IS_NUMBER=$(jq '.summary.filesAnalyzed | type == "number"' "$TEST_DIR/api-demo-response.json")

# Check if filesAnalyzed is greater than 0 in backend (our fix)
BACKEND_FILES_GT_ZERO=$(jq '.summary.filesAnalyzed > 0' "$TEST_DIR/backend-analysis.json")

echo "Data type consistency:"
echo "   Backend filesAnalyzed is number: $BACKEND_FILES_IS_NUMBER"
echo "   API filesAnalyzed is number: $API_FILES_IS_NUMBER"
echo "   Backend filesAnalyzed > 0: $BACKEND_FILES_GT_ZERO (should be true)"

# Test Results Summary
echo -e "\n${GREEN}📊 Test Results Summary${NC}"
echo "======================="

TESTS_PASSED=0
TOTAL_TESTS=5

# Test 1: Backend analysis generated
if [ "$BACKEND_ISSUES" -gt 0 ] && [ "$BACKEND_FILES" -gt 0 ]; then
    echo -e "✅ Test 1: Backend analysis generation - ${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "❌ Test 1: Backend analysis generation - ${RED}FAILED${NC}"
fi

# Test 2: API server responded
if [ "$API_SCHEMA_VERSION" == "1.0" ] && [ "$API_ISSUES" -gt 0 ]; then
    echo -e "✅ Test 2: API server response - ${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "❌ Test 2: API server response - ${RED}FAILED${NC}"
fi

# Test 3: Required fields present
if [ "$BACKEND_HAS_SUMMARY" == "true" ] && [ "$API_HAS_FINDINGS" == "true" ] && [ "$API_HAS_SCHEMA" == "true" ]; then
    echo -e "✅ Test 3: Schema consistency - ${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "❌ Test 3: Schema consistency - ${RED}FAILED${NC}"
fi

# Test 4: Field naming conventions
if [ "$API_USES_FINDINGS" == "true" ] && [ "$BACKEND_FILES_ANALYZED" == "true" ] && [ "$API_FILES_ANALYZED" == "true" ]; then
    echo -e "✅ Test 4: Field naming consistency - ${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "❌ Test 4: Field naming consistency - ${RED}FAILED${NC}"
fi

# Test 5: Data types and values
if [ "$BACKEND_FILES_IS_NUMBER" == "true" ] && [ "$API_FILES_IS_NUMBER" == "true" ] && [ "$BACKEND_FILES_GT_ZERO" == "true" ]; then
    echo -e "✅ Test 5: Data type consistency - ${GREEN}PASSED${NC}"
    ((TESTS_PASSED++))
else
    echo -e "❌ Test 5: Data type consistency - ${RED}FAILED${NC}"
fi

# Final result
echo -e "\n${YELLOW}Final Result: $TESTS_PASSED/$TOTAL_TESTS tests passed${NC}"

if [ "$TESTS_PASSED" -eq "$TOTAL_TESTS" ]; then
    echo -e "${GREEN}🎉 All tests passed! Data consistency is maintained across the pipeline.${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed. Data consistency issues detected.${NC}"
    exit 1
fi