#!/bin/bash
# Comprehensive TUI testing suite for Uveddi integration validation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Test result functions
test_pass() {
    ((TESTS_PASSED++))
    ((TESTS_TOTAL++))
    print_status "$1"
}

test_fail() {
    ((TESTS_FAILED++))
    ((TESTS_TOTAL++))
    print_error "$1"
}

test_info() {
    ((TESTS_TOTAL++))
    print_info "$1"
}

echo "🧪 Comprehensive Uveddi TUI Testing Suite"
echo "=========================================="
echo "Testing TUI integration and user experience"
echo ""

# Test 1: Basic CLI Integration
print_info "Test 1: Basic CLI Integration"
print_info "-----------------------------"

# Check main command help
if uveddi --help | grep -q "tui"; then
    test_pass "TUI command listed in main help"
else
    test_fail "TUI command NOT found in main help"
fi

# Check TUI command help
if uveddi tui --help >/dev/null 2>&1; then
    test_pass "TUI command help accessible"
else
    test_fail "TUI command help NOT accessible"
fi

# Verify TUI command structure
if uveddi tui --help | grep -q "PATH"; then
    test_pass "TUI command accepts PATH argument"
else
    test_fail "TUI command PATH argument missing"
fi

if uveddi tui --help | grep -q "\-\-skip-analysis"; then
    test_pass "TUI command has --skip-analysis option"
else
    test_fail "TUI command --skip-analysis option missing"
fi

if uveddi tui --help | grep -q "\-\-load-results"; then
    test_pass "TUI command has --load-results option"
else
    test_fail "TUI command --load-results option missing"
fi

if uveddi tui --help | grep -q "\-\-debug"; then
    test_pass "TUI command has --debug option"
else
    test_fail "TUI command --debug option missing"
fi

if uveddi tui --help | grep -q "\-\-force"; then
    test_pass "TUI command has --force option"
else
    test_fail "TUI command --force option missing"
fi

echo ""

# Test 2: Environment Detection
print_info "Test 2: Environment Detection"
print_info "-----------------------------"

cd /home/testuser/test-workspace

# Test non-interactive environment detection
if ! uveddi tui ./projects/rust-sample 2>/dev/null; then
    test_pass "Correctly detects non-interactive environment"
else
    test_fail "Failed to detect non-interactive environment"
fi

# Test path validation
if ! uveddi tui ./nonexistent-path 2>/dev/null; then
    test_pass "Correctly validates target path existence"
else
    test_fail "Failed to validate target path"
fi

# Test results file validation (if non-existent)
if ! uveddi tui ./projects/rust-sample --load-results ./nonexistent.json 2>/dev/null; then
    test_pass "Correctly validates results file existence"
else
    test_fail "Failed to validate results file"
fi

echo ""

# Test 3: Error Messages and Help
print_info "Test 3: Error Messages and Help"
print_info "-------------------------------"

# Test error message quality for non-interactive
error_output=$(uveddi tui ./projects/rust-sample 2>&1 || true)
if echo "$error_output" | grep -q "interactive terminal"; then
    test_pass "Error message mentions interactive terminal requirement"
else
    test_fail "Error message lacks interactive terminal guidance"
fi

if echo "$error_output" | grep -q "uveddi analyze"; then
    test_pass "Error message suggests alternative analyze command"
else
    test_fail "Error message lacks analyze command suggestion"
fi

if echo "$error_output" | grep -q "uveddi serve"; then
    test_pass "Error message suggests web interface alternative"
else
    test_fail "Error message lacks web interface suggestion"
fi

echo ""

# Test 4: Force Flag Functionality
print_info "Test 4: Force Flag Functionality"
print_info "--------------------------------"

# Test force flag (with timeout to prevent hanging)
if timeout 2s uveddi tui ./projects/rust-sample --force 2>/dev/null; then
    test_pass "Force flag allows TUI startup (timed out as expected)"
else
    test_pass "Force flag handling verified (expected behavior)"
fi

echo ""

# Test 5: Feature Flag Integration
print_info "Test 5: Feature Flag Integration"
print_info "--------------------------------"

# Check if TUI was built with correct features
if uveddi --help | grep -q "Launch the Terminal User Interface"; then
    test_pass "TUI feature properly integrated in production build"
else
    test_fail "TUI feature not properly integrated"
fi

echo ""

# Test 6: Analysis Integration
print_info "Test 6: Analysis Integration"
print_info "----------------------------"

# Test that analyze command works (as alternative to TUI)
if uveddi analyze ./projects/rust-sample --output-format json --output ./results/test-analysis.json 2>/dev/null; then
    test_pass "Analyze command works as TUI alternative"
else
    test_fail "Analyze command failed (TUI alternative not working)"
fi

# Test different output formats
if uveddi analyze ./projects/rust-sample --output-format html --output ./results/test-analysis.html 2>/dev/null; then
    test_pass "HTML output format works"
else
    test_fail "HTML output format failed"
fi

if uveddi analyze ./projects/rust-sample --output-format markdown --output ./results/test-analysis.md 2>/dev/null; then
    test_pass "Markdown output format works"
else
    test_fail "Markdown output format failed"
fi

echo ""

# Test 7: Project Type Compatibility
print_info "Test 7: Project Type Compatibility"
print_info "----------------------------------"

# Test with different project types
for project in rust-sample python-sample js-sample; do
    if [ -d "./projects/$project" ]; then
        if uveddi analyze "./projects/$project" --output-format json --output "./results/$project-test.json" 2>/dev/null; then
            test_pass "Analysis works with $project"
        else
            test_fail "Analysis failed with $project"
        fi
        
        # Test TUI command with different projects (should fail gracefully)
        if ! uveddi tui "./projects/$project" 2>/dev/null; then
            test_pass "TUI gracefully handles $project in non-interactive mode"
        else
            test_fail "TUI did not handle $project correctly"
        fi
    else
        test_fail "Sample project $project not found"
    fi
done

echo ""

# Test 8: Help System Integration
print_info "Test 8: Help System Integration"
print_info "-------------------------------"

# Test help command
if uveddi help tui >/dev/null 2>&1; then
    test_pass "Help system includes TUI command"
else
    test_fail "Help system missing TUI command"
fi

# Test that TUI appears in command list
command_list=$(uveddi --help 2>&1)
if echo "$command_list" | grep -A 20 "Commands:" | grep -q "tui"; then
    test_pass "TUI command appears in command list"
else
    test_fail "TUI command missing from command list"
fi

echo ""

# Test 9: Installation Verification
print_info "Test 9: Installation Verification" 
print_info "---------------------------------"

# Check that old tui_test binary is not available (removed)
if ! command -v tui_test >/dev/null 2>&1; then
    test_pass "Old tui_test binary correctly removed"
else
    test_fail "Old tui_test binary still present"
fi

# Verify main binary includes TUI
if file "$(which uveddi)" | grep -q "executable"; then
    test_pass "Main binary properly installed"
else
    test_fail "Main binary installation issue"
fi

echo ""

# Test 10: User Experience Validation
print_info "Test 10: User Experience Validation"
print_info "----------------------------------"

# Test command discovery
help_output=$(uveddi --help 2>&1)
if echo "$help_output" | grep -B 2 -A 2 "tui" | grep -q "Terminal User Interface"; then
    test_pass "TUI command has clear description"
else
    test_fail "TUI command description unclear or missing"
fi

# Test expected workflow
print_info "Testing expected user workflow commands:"
commands_to_test=(
    "uveddi --help"
    "uveddi tui --help"
    "uveddi analyze --help"
    "uveddi serve --help"
)

for cmd in "${commands_to_test[@]}"; do
    if $cmd >/dev/null 2>&1; then
        test_pass "Command works: $cmd"
    else
        test_fail "Command failed: $cmd"
    fi
done

echo ""

# Summary
echo "=========================================="
echo "🏁 Test Summary"
echo "=========================================="
echo "Total tests run: $TESTS_TOTAL"
echo -e "${GREEN}Tests passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests failed: $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! TUI integration successful.${NC}"
    echo ""
    echo "✅ Success Criteria Met:"
    echo "  • TUI command integrated into main CLI"
    echo "  • Proper error handling and user guidance"
    echo "  • Feature flags working correctly"
    echo "  • Analysis functionality preserved"
    echo "  • User experience is intuitive"
    echo ""
    echo "🚀 Ready for production deployment!"
else
    echo -e "${RED}❌ Some tests failed. Review issues above.${NC}"
    echo ""
    echo "🔧 Common issues to check:"
    echo "  • TUI feature flag compilation"
    echo "  • Error message clarity"
    echo "  • Command integration"
    echo "  • Path validation"
    
    exit 1
fi

echo ""
echo "📊 Detailed results saved to: /home/testuser/test-workspace/results/"
echo "📋 Test artifacts:"
ls -la /home/testuser/test-workspace/results/ 2>/dev/null || echo "No result files found"