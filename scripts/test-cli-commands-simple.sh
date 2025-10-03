#!/bin/bash
# Simplified Uveddi CLI Command Testing Suite
# Fast, reliable tests for all CLI commands
# Created: 2025-10-03

set -euo pipefail

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
PASS=0
FAIL=0
SKIP=0

# Binary and test directory
UVEDDI_BIN="${UVEDDI_BIN:-./target/debug/uveddi}"
TEST_DIR="/tmp/uveddi-cli-test-$$"
mkdir -p "$TEST_DIR"

# Suppress logging noise by setting environment variable
export RUST_LOG=error

echo "🧪 Uveddi CLI Test Suite (Simple)"
echo "=================================="
echo ""
echo "Binary: $UVEDDI_BIN"
echo "Test Dir: $TEST_DIR"
echo ""

# Test helper
test_cmd() {
    local name="$1"
    local cmd="$2"
    local expect_fail="${3:-0}"

    if timeout 10 bash -c "$cmd" > /dev/null 2>&1; then
        if [ "$expect_fail" -eq 0 ]; then
            echo -e "${GREEN}✅ PASS:${NC} $name"
            ((PASS++))
        else
            echo -e "${RED}❌ FAIL:${NC} $name (should have failed)"
            ((FAIL++))
        fi
    else
        if [ "$expect_fail" -eq 1 ]; then
            echo -e "${GREEN}✅ PASS:${NC} $name (failed as expected)"
            ((PASS++))
        else
            echo -e "${RED}❌ FAIL:${NC} $name"
            ((FAIL++))
        fi
    fi
}

# ============================================================================
# Basic CLI Tests
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 1: Basic CLI Options"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Version (--version)" "$UVEDDI_BIN --version"
test_cmd "Version (-V)" "$UVEDDI_BIN -V"
test_cmd "Help (--help)" "$UVEDDI_BIN --help"
test_cmd "Help (-h)" "$UVEDDI_BIN -h"

# ============================================================================
# Analyze Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 2: Analyze Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create test project
mkdir -p "$TEST_DIR/project"
cat > "$TEST_DIR/project/main.rs" << 'EOF'
fn main() {
    println!("test");
}
EOF

test_cmd "Analyze --help" "$UVEDDI_BIN analyze --help"
test_cmd "Analyze alias (a --help)" "$UVEDDI_BIN a --help"
test_cmd "Analyze basic" "$UVEDDI_BIN analyze $TEST_DIR/project --output-format json"
test_cmd "Analyze with Markdown" "$UVEDDI_BIN analyze $TEST_DIR/project --output-format markdown"
test_cmd "Analyze with timeout" "$UVEDDI_BIN analyze $TEST_DIR/project --timeout 5"
test_cmd "Analyze non-existent path" "$UVEDDI_BIN analyze /nonexistent" 1

# ============================================================================
# Config Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 3: Config Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Config --help" "$UVEDDI_BIN config --help"
test_cmd "Config alias (cfg --help)" "$UVEDDI_BIN cfg --help"
test_cmd "Config show" "$UVEDDI_BIN config show"

# ============================================================================
# Doctor Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 4: Doctor Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Doctor --help" "$UVEDDI_BIN doctor --help"
test_cmd "Doctor alias (dr --help)" "$UVEDDI_BIN dr --help"
test_cmd "Doctor check" "$UVEDDI_BIN doctor"

# ============================================================================
# Help Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 5: Help Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Help command" "$UVEDDI_BIN help"
test_cmd "Help analyze" "$UVEDDI_BIN help analyze"
test_cmd "Help config" "$UVEDDI_BIN help config"

# ============================================================================
# Hooks Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 6: Hooks Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Hooks --help" "$UVEDDI_BIN hooks --help"

# Create git repo for hooks tests
mkdir -p "$TEST_DIR/git-repo"
cd "$TEST_DIR/git-repo"
git init > /dev/null 2>&1
cd - > /dev/null

test_cmd "Hooks list" "cd $TEST_DIR/git-repo && $UVEDDI_BIN hooks list"

# ============================================================================
# Init Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 7: Init Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Init --help" "$UVEDDI_BIN init --help"

mkdir -p "$TEST_DIR/init-test"
test_cmd "Init non-interactive" "cd $TEST_DIR/init-test && $UVEDDI_BIN init --non-interactive"

# ============================================================================
# UI Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 8: UI Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "UI --help" "$UVEDDI_BIN ui --help"

# ============================================================================
# CI Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 9: CI Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "CI --help" "$UVEDDI_BIN ci --help"

# ============================================================================
# TUI Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 10: TUI Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "TUI --help" "$UVEDDI_BIN tui --help"

# ============================================================================
# Serve Command
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 11: Serve Command"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Serve --help" "$UVEDDI_BIN serve --help"

# ============================================================================
# Error Handling
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SECTION 12: Error Handling"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_cmd "Invalid command" "$UVEDDI_BIN invalid-command" 1
test_cmd "Analyze invalid format" "$UVEDDI_BIN analyze $TEST_DIR/project --output-format invalid" 1

# ============================================================================
# Cleanup
# ============================================================================
rm -rf "$TEST_DIR"

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Passed: $PASS${NC}"
echo -e "${RED}❌ Failed: $FAIL${NC}"
echo -e "${YELLOW}⏭️  Skipped: $SKIP${NC}"
echo ""

TOTAL=$((PASS + FAIL + SKIP))
if [ $TOTAL -gt 0 ]; then
    RATE=$((PASS * 100 / TOTAL))
    echo "Total: $TOTAL | Pass Rate: ${RATE}%"
fi
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed${NC}"
    exit 1
fi
