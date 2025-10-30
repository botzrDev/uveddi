#!/bin/bash
# Comprehensive Uveddi CLI Command Testing Suite
# Tests every CLI action and command variation
# Created: 2025-10-03

set -e  # Exit on error

echo "🧪 Uveddi Comprehensive CLI Testing Suite"
echo "=========================================="
echo ""

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Binary path
UVEDDI_BIN="${UVEDDI_BIN:-./target/debug/uveddi}"

# Test directory
TEST_DIR="/tmp/uveddi-cli-test-$$"
mkdir -p "$TEST_DIR"

# Function to print test status
test_pass() {
    echo -e "${GREEN}✅ PASS:${NC} $1"
    ((TESTS_PASSED++))
}

test_fail() {
    echo -e "${RED}❌ FAIL:${NC} $1"
    ((TESTS_FAILED++))
}

test_skip() {
    echo -e "${YELLOW}⏭️  SKIP:${NC} $1"
    ((TESTS_SKIPPED++))
}

test_info() {
    echo -e "${BLUE}ℹ️  INFO:${NC} $1"
}

# Function to run a test command
run_test() {
    local test_name="$1"
    local command="$2"
    local expected_exit_code="${3:-0}"

    test_info "Running: $test_name"

    if eval "$command" > "$TEST_DIR/test_output.log" 2>&1; then
        actual_exit_code=0
    else
        actual_exit_code=$?
    fi

    if [ "$actual_exit_code" -eq "$expected_exit_code" ]; then
        test_pass "$test_name"
        return 0
    else
        test_fail "$test_name (exit code: $actual_exit_code, expected: $expected_exit_code)"
        cat "$TEST_DIR/test_output.log" | tail -10
        return 1
    fi
}

# Function to run a test and check output
run_test_output() {
    local test_name="$1"
    local command="$2"
    local expected_output="$3"

    test_info "Running: $test_name"

    if eval "$command" > "$TEST_DIR/test_output.log" 2>&1; then
        if grep -q "$expected_output" "$TEST_DIR/test_output.log"; then
            test_pass "$test_name"
            return 0
        else
            test_fail "$test_name (output mismatch)"
            echo "Expected to find: $expected_output"
            echo "Actual output:"
            cat "$TEST_DIR/test_output.log" | tail -10
            return 1
        fi
    else
        test_fail "$test_name (command failed)"
        cat "$TEST_DIR/test_output.log" | tail -10
        return 1
    fi
}

# Ensure binary exists
if [ ! -f "$UVEDDI_BIN" ]; then
    echo -e "${RED}Error: Uveddi binary not found at $UVEDDI_BIN${NC}"
    echo "Please build first: cargo build --bin uveddi --features standard"
    exit 1
fi

echo "📦 Binary: $UVEDDI_BIN"
echo "📁 Test directory: $TEST_DIR"
echo ""

# ============================================================================
# SECTION 1: Basic CLI Options
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 1: Basic CLI Options"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Version flag (--version)" \
    "$UVEDDI_BIN --version" \
    "uveddi 1.0.0"

run_test_output "Version flag (-V)" \
    "$UVEDDI_BIN -V" \
    "uveddi 1.0.0"

run_test_output "Help flag (--help)" \
    "$UVEDDI_BIN --help" \
    "Usage: uveddi"

run_test_output "Help flag (-h)" \
    "$UVEDDI_BIN -h" \
    "Usage: uveddi"

# ============================================================================
# SECTION 2: Analyze Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 2: Analyze Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Analyze help (--help)" \
    "$UVEDDI_BIN analyze --help" \
    "Perform comprehensive code analysis"

run_test_output "Analyze help (short form: a --help)" \
    "$UVEDDI_BIN a --help" \
    "Perform comprehensive code analysis"

# Create test project
mkdir -p "$TEST_DIR/test-project"
cat > "$TEST_DIR/test-project/main.rs" << 'EOF'
fn main() {
    println!("Hello, world!");
}

fn unused_function() {
    // This should be detected as dead code
}
EOF

run_test "Analyze basic Rust file" \
    "timeout 60 $UVEDDI_BIN analyze $TEST_DIR/test-project --output-format json"

run_test "Analyze with JSON output to file" \
    "timeout 60 $UVEDDI_BIN analyze $TEST_DIR/test-project --output-format json --output $TEST_DIR/report.json"

run_test "Analyze with Markdown output" \
    "timeout 60 $UVEDDI_BIN analyze $TEST_DIR/test-project --output-format markdown"

run_test "Analyze with HTML output" \
    "timeout 60 $UVEDDI_BIN analyze $TEST_DIR/test-project --output-format html --output $TEST_DIR/report.html"

run_test "Analyze with timeout option" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --timeout 10"

run_test "Analyze with verbose flag" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --verbose"

run_test "Analyze with progress format terminal" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --progress-format terminal"

run_test "Analyze with progress format json" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --progress-format json"

run_test "Analyze with progress format silent" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --progress-format silent"

run_test "Analyze with Mermaid-only diagrams" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --mermaid-only"

run_test "Analyze with no diagrams" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --no-diagrams"

run_test "Analyze with max diagrams limit" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --max-diagrams 5"

# Memory optimization tests
run_test "Analyze with memory optimization disabled" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --disable-memory-optimization"

run_test "Analyze with memory limit" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --memory-limit-gb 2.0"

run_test "Analyze with memory profile small" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --memory-profile small"

run_test "Analyze with memory profile default" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --memory-profile default"

run_test "Analyze with memory profile large" \
    "timeout 30 $UVEDDI_BIN analyze $TEST_DIR/test-project --memory-profile large"

# ============================================================================
# SECTION 3: Config Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 3: Config Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Config help" \
    "$UVEDDI_BIN config --help" \
    "Manage configuration settings"

run_test_output "Config help (short form: cfg)" \
    "$UVEDDI_BIN cfg --help" \
    "Manage configuration settings"

# Create test config file
cat > "$TEST_DIR/test-config.toml" << 'EOF'
ollama_model = "deepseek-coder:6.7b"
EOF

run_test "Config show from environment" \
    "$UVEDDI_BIN config show"

run_test "Config show from file" \
    "$UVEDDI_BIN config show --file $TEST_DIR/test-config.toml"

run_test "Config set value" \
    "$UVEDDI_BIN config set ollama_model 'deepseek-coder:latest' --file $TEST_DIR/test-config-new.toml"

run_test "Config validate" \
    "$UVEDDI_BIN config validate --file $TEST_DIR/test-config.toml"

run_test "Config validate with suggestions" \
    "$UVEDDI_BIN config validate --file $TEST_DIR/test-config.toml --suggestions"

run_test "Config validate with JSON format" \
    "$UVEDDI_BIN config validate --file $TEST_DIR/test-config.toml --format json"

# ============================================================================
# SECTION 4: Doctor Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 4: Doctor Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Doctor help" \
    "$UVEDDI_BIN doctor --help" \
    "Run health diagnostics"

run_test_output "Doctor help (short form: dr)" \
    "$UVEDDI_BIN dr --help" \
    "Run health diagnostics"

run_test "Doctor check" \
    "timeout 30 $UVEDDI_BIN doctor"

# ============================================================================
# SECTION 5: Help Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 5: Help Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Help command" \
    "$UVEDDI_BIN help" \
    "Usage: uveddi"

run_test_output "Help for analyze command" \
    "$UVEDDI_BIN help analyze" \
    "Perform comprehensive code analysis"

run_test_output "Help for config command" \
    "$UVEDDI_BIN help config" \
    "Manage configuration settings"

# ============================================================================
# SECTION 6: Hooks Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 6: Hooks Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Hooks help" \
    "$UVEDDI_BIN hooks --help" \
    "Git hooks management"

# Create a test git repo
mkdir -p "$TEST_DIR/git-repo"
cd "$TEST_DIR/git-repo"
git init > /dev/null 2>&1
cd - > /dev/null

run_test "Hooks list" \
    "cd $TEST_DIR/git-repo && $UVEDDI_BIN hooks list"

run_test "Hooks install pre-commit" \
    "cd $TEST_DIR/git-repo && $UVEDDI_BIN hooks install pre-commit"

run_test "Hooks uninstall pre-commit" \
    "cd $TEST_DIR/git-repo && $UVEDDI_BIN hooks uninstall pre-commit"

# ============================================================================
# SECTION 7: Init Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 7: Init Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Init help" \
    "$UVEDDI_BIN init --help" \
    "Initialize Uveddi configuration"

mkdir -p "$TEST_DIR/init-test"

run_test "Init in directory" \
    "cd $TEST_DIR/init-test && $UVEDDI_BIN init --non-interactive"

# ============================================================================
# SECTION 8: UI Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 8: UI Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "UI help" \
    "$UVEDDI_BIN ui --help" \
    "Launch the UI dashboard"

# UI command requires server, skip actual launch
test_skip "UI launch (requires running server)"

# ============================================================================
# SECTION 9: CI Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 9: CI Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "CI help" \
    "$UVEDDI_BIN ci --help" \
    "CI/CD integration"

run_test "CI check (dry run)" \
    "timeout 30 $UVEDDI_BIN ci --dry-run"

# ============================================================================
# SECTION 10: TUI Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 10: TUI Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "TUI help" \
    "$UVEDDI_BIN tui --help" \
    "Terminal User Interface"

# TUI requires terminal, skip actual launch
test_skip "TUI launch (requires interactive terminal)"

# ============================================================================
# SECTION 11: Plugin Command (feature-gated)
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 11: Plugin Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if wasm-plugins feature is enabled
if $UVEDDI_BIN plugin --help > /dev/null 2>&1; then
    run_test_output "Plugin help" \
        "$UVEDDI_BIN plugin --help" \
        "Manage WASM plugins"

    run_test "Plugin list" \
        "$UVEDDI_BIN plugin list"
else
    test_skip "Plugin command (wasm-plugins feature not enabled)"
fi

# ============================================================================
# SECTION 12: Serve Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 12: Serve Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test_output "Serve help" \
    "$UVEDDI_BIN serve --help" \
    "Start the web dashboard"

# Test server startup with timeout (will start and then timeout)
test_info "Testing server startup on port 9876..."
timeout 5 $UVEDDI_BIN serve --port 9876 > "$TEST_DIR/serve.log" 2>&1 &
SERVER_PID=$!
sleep 2

if kill -0 $SERVER_PID 2>/dev/null; then
    test_pass "Server startup on custom port"
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
else
    # Check if it's a feature issue or actual failure
    if grep -q "feature" "$TEST_DIR/serve.log"; then
        test_skip "Serve command (service-orchestration feature not enabled)"
    else
        test_fail "Server startup on custom port"
        cat "$TEST_DIR/serve.log" | tail -10
    fi
fi

# ============================================================================
# SECTION 13: Error Handling Tests
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 13: Error Handling Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

run_test "Invalid command" \
    "$UVEDDI_BIN invalid-command" \
    2

run_test "Analyze non-existent path" \
    "timeout 10 $UVEDDI_BIN analyze /non/existent/path" \
    1

run_test "Analyze with invalid output format" \
    "timeout 10 $UVEDDI_BIN analyze $TEST_DIR/test-project --output-format invalid-format" \
    1

run_test "Config with non-existent file" \
    "$UVEDDI_BIN config show --file /non/existent/file.toml" \
    1

# ============================================================================
# SECTION 14: Edge Cases and Special Options
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 14: Edge Cases and Special Options"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create empty directory
mkdir -p "$TEST_DIR/empty-dir"

run_test "Analyze empty directory" \
    "timeout 10 $UVEDDI_BIN analyze $TEST_DIR/empty-dir" \
    1

# Create directory with non-source files
mkdir -p "$TEST_DIR/no-source"
echo "Not source code" > "$TEST_DIR/no-source/readme.txt"

run_test "Analyze directory with no source files" \
    "timeout 10 $UVEDDI_BIN analyze $TEST_DIR/no-source" \
    1

# Test with extremely long path
LONG_PATH="$TEST_DIR/$(printf 'a%.0s' {1..100})"
mkdir -p "$LONG_PATH"
echo "fn main() {}" > "$LONG_PATH/main.rs"

run_test "Analyze with long path" \
    "timeout 30 $UVEDDI_BIN analyze $LONG_PATH"

# ============================================================================
# Cleanup
# ============================================================================
echo ""
echo "🧹 Cleaning up test artifacts..."
rm -rf "$TEST_DIR"

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Tests Passed:  $TESTS_PASSED${NC}"
echo -e "${RED}❌ Tests Failed:  $TESTS_FAILED${NC}"
echo -e "${YELLOW}⏭️  Tests Skipped: $TESTS_SKIPPED${NC}"
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))
PASS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))

echo "Total Tests: $TOTAL_TESTS"
echo "Pass Rate: ${PASS_RATE}%"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed. Please review the output above.${NC}"
    exit 1
fi
