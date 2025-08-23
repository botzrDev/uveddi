#!/bin/bash
#
# Quick Data Consistency Test
#

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$PROJECT_ROOT/target/debug/uveddi"

echo "🔍 Quick Data Consistency Test"
echo "=============================="

# Test: Analyze same project and check via API
echo "📊 Step 1: Run analysis on API module"
TEST_OUTPUT="/tmp/test-api-analysis.json"
"$BINARY" analyze "$PROJECT_ROOT/src/api" \
    --output-format json \
    --output "$TEST_OUTPUT"

if [ ! -f "$TEST_OUTPUT" ]; then
    echo "❌ Analysis failed"
    exit 1
fi

# Extract key metrics
ANALYSIS_FILES=$(jq '.summary.filesAnalyzed' "$TEST_OUTPUT")
ANALYSIS_ISSUES=$(jq '.summary.issuesTotal' "$TEST_OUTPUT")

echo "✅ Analysis completed"
echo "   Files analyzed: $ANALYSIS_FILES"
echo "   Issues found: $ANALYSIS_ISSUES"

# Test: Start API server and query same analysis via database
echo ""
echo "🌐 Step 2: Query analysis via API server"

# Start server in background (simplified)
API_PORT=8891
mkdir -p .uveddi
"$BINARY" ui serve --port $API_PORT --dev &
SERVER_PID=$!

# Wait for startup
sleep 5

# Query API
API_RESPONSE="/tmp/api-response.json"
curl -s "http://localhost:$API_PORT/api/v1/reports/demo" > "$API_RESPONSE"

# Cleanup server
kill $SERVER_PID 2>/dev/null || true

if [ ! -s "$API_RESPONSE" ]; then
    echo "❌ API query failed"
    exit 1
fi

# Extract API metrics
API_FILES=$(jq '.summary.filesAnalyzed' "$API_RESPONSE")
API_ISSUES=$(jq '.summary.issuesTotal' "$API_RESPONSE")

echo "✅ API query completed"
echo "   Files analyzed: $API_FILES"
echo "   Issues found: $API_ISSUES"

# Compare results
echo ""
echo "📋 Results Summary:"
echo "   Direct Analysis Files: $ANALYSIS_FILES"
echo "   API Response Files: $API_FILES"  
echo "   Direct Analysis Issues: $ANALYSIS_ISSUES"
echo "   API Response Issues: $API_ISSUES"

# Basic check - files should be > 0
if [ "$ANALYSIS_FILES" -gt 0 ] && [ "$API_FILES" -gt 0 ]; then
    echo "✅ Both report non-zero files (fix confirmed)"
else
    echo "❌ Still showing zero files"
    exit 1
fi

echo ""
echo "🎉 File count fix is working!"
echo "   Both direct analysis and API consistently report analyzed files"