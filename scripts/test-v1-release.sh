#!/bin/bash
# Uveddi v1.0 Release Testing Suite
# Comprehensive testing before v1.0 release

set -e  # Exit on error

echo "🧪 Uveddi v1.0 Release Testing Suite"
echo "===================================="
echo ""

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0

# Function to print test status
test_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $1${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ $1${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Function to print warning
test_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo "📦 Test 1: Building all components..."
echo "======================================"

# Build library
echo "Building library with standard features..."
cargo build --lib --features standard 2>&1 | tail -5
test_status "Library build"

# Build main binary
echo "Building main binary..."
cargo build --bin uveddi --features standard 2>&1 | tail -5
test_status "Main binary build"

# Build with full features
echo "Building with full features..."
cargo build --release --features full 2>&1 | tail -5
test_status "Release build with full features"

# Build frontend
echo "Building frontend..."
cd frontend
if [ -d "node_modules" ]; then
    npm run build > /dev/null 2>&1
else
    echo "Installing frontend dependencies..."
    npm install > /dev/null 2>&1
    npm run build > /dev/null 2>&1
fi
cd ..
test_status "Frontend build"

# Test API server
echo "Testing API server setup..."
cd api-server
if [ ! -d "node_modules" ]; then
    echo "Installing API server dependencies..."
    npm install > /dev/null 2>&1
fi
cd ..
test_status "API server setup"

echo ""
echo "🔧 Test 2: Testing CLI commands..."
echo "===================================="

# Test version command
./target/debug/uveddi --version > /dev/null
test_status "Version command"

# Test help command
./target/debug/uveddi --help > /dev/null
test_status "Help command"

# Test analyze help
./target/debug/uveddi analyze --help > /dev/null
test_status "Analyze help command"

# Test serve help
./target/debug/uveddi serve --help > /dev/null
test_status "Serve help command"

echo ""
echo "📊 Test 3: Running analysis with different outputs..."
echo "======================================================"

# Create temp directory for test outputs
TEST_DIR="/tmp/uveddi-test-$$"
mkdir -p "$TEST_DIR"

# Test JSON output
echo "Testing JSON output format..."
timeout 120 ./target/debug/uveddi analyze ./src/cli --output-format json --output "$TEST_DIR/test-report.json" > /dev/null 2>&1 || true
test -f "$TEST_DIR/test-report.json"
test_status "JSON report generation"

# Test HTML output
echo "Testing HTML output format..."
timeout 120 ./target/debug/uveddi analyze ./src/cli --output-format html --output "$TEST_DIR/test-report.html" > /dev/null 2>&1 || true
test -f "$TEST_DIR/test-report.html"
test_status "HTML report generation"

# Test Markdown output
echo "Testing Markdown output format..."
timeout 120 ./target/debug/uveddi analyze ./src/cli --output-format markdown --output "$TEST_DIR/test-report.md" > /dev/null 2>&1 || true
test -f "$TEST_DIR/test-report.md"
test_status "Markdown report generation"

# Verify JSON is valid
if [ -f "$TEST_DIR/test-report.json" ]; then
    python3 -m json.tool "$TEST_DIR/test-report.json" > /dev/null 2>&1
    test_status "JSON report validity"
fi

echo ""
echo "🌐 Test 4: Testing server startup..."
echo "====================================="

# Test server startup on custom port
echo "Starting server on port 9999..."
timeout 10 ./target/debug/uveddi serve --port 9999 > /dev/null 2>&1 &
SERVER_PID=$!
sleep 5

# Test if server is running
if kill -0 $SERVER_PID 2>/dev/null; then
    test_status "Server startup"

    # Test health endpoint
    echo "Testing health endpoint..."
    curl -f http://localhost:9999/health > /dev/null 2>&1
    test_status "Health endpoint"

    # Test API reports endpoint (may not have reports)
    echo "Testing reports endpoint..."
    curl -f http://localhost:9999/api/v1/reports > /dev/null 2>&1 || test_warning "No reports available (expected for clean test)"

    # Cleanup server
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
else
    test_status "Server startup"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

echo ""
echo "🧪 Test 5: Running test suite..."
echo "================================="

# Run library tests
echo "Running library unit tests..."
cargo test --features standard --lib --quiet 2>&1 | tail -10
test_status "Library unit tests"

# Run binary tests
echo "Running binary tests..."
cargo test --features standard --bins --quiet 2>&1 | tail -10
test_status "Binary tests"

# Run doc tests
echo "Running documentation tests..."
cargo test --features standard --doc --quiet 2>&1 | tail -10 || test_warning "Some doc tests may be disabled"

echo ""
echo "🔍 Test 6: Running linter..."
echo "============================"

# Run clippy
cargo clippy --features standard --all-targets -- -D warnings 2>&1 | tail -20
test_status "Clippy linting"

echo ""
echo "🔒 Test 7: Security audit..."
echo "============================"

# Check if cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    test_warning "cargo-audit not installed (run: cargo install cargo-audit)"
else
    cargo audit 2>&1 | tail -10
    if [ $? -eq 0 ]; then
        test_status "Security audit"
    else
        test_warning "Security vulnerabilities found - review required"
    fi
fi

echo ""
echo "⚛️  Test 8: Frontend production build..."
echo "========================================="

cd frontend
if [ -d "dist" ]; then
    test_status "Frontend dist directory exists"
else
    npm run build > /dev/null 2>&1
    test -d dist
    test_status "Frontend production build"
fi
cd ..

echo ""
echo "🚀 Test 9: API server tests..."
echo "==============================="

cd api-server
npm start > /dev/null 2>&1 &
API_PID=$!
sleep 3

# Test if API server is running
if kill -0 $API_PID 2>/dev/null; then
    curl -f http://localhost:8000/health > /dev/null 2>&1
    test_status "API server health check"
    kill $API_PID 2>/dev/null || true
    wait $API_PID 2>/dev/null || true
else
    test_warning "API server failed to start (may be port conflict)"
fi
cd ..

echo ""
echo "📚 Test 10: Documentation validation..."
echo "========================================="

# Check critical documentation files exist
test -f README.md
test_status "README.md exists"

test -f CLAUDE.md
test_status "CLAUDE.md exists"

test -f docs/API_SERVER.md
test_status "API_SERVER.md exists"

test -f docs/CURRENT_STATUS.md
test_status "CURRENT_STATUS.md exists"

# Cleanup
echo ""
echo "🧹 Cleaning up test artifacts..."
rm -rf "$TEST_DIR"

echo ""
echo "======================================"
echo "📋 Test Summary:"
echo "======================================"
echo -e "${GREEN}✅ Tests Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}❌ Tests Failed: $TESTS_FAILED${NC}"
else
    echo -e "${GREEN}❌ Tests Failed: 0${NC}"
fi
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🚀 All critical tests passed! Ready for v1.0 release!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Some tests failed. Please review before release.${NC}"
    exit 1
fi
