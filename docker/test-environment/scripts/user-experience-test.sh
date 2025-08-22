#!/bin/bash
# User Experience Testing for Uveddi TUI Integration
# Simulates first-time user interactions and validates UX

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Test results
UX_SCORE=0
MAX_UX_SCORE=0

print_header() {
    echo -e "${BOLD}${BLUE}$1${NC}"
    echo -e "${BLUE}$(echo $1 | sed 's/./=/g')${NC}"
}

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

# UX scoring functions
ux_pass() {
    ((UX_SCORE++))
    ((MAX_UX_SCORE++))
    print_status "$1"
}

ux_fail() {
    ((MAX_UX_SCORE++))
    print_error "$1"
}

ux_info() {
    print_info "$1"
}

print_header "👤 Uveddi TUI User Experience Testing"
echo "Simulating first-time user interactions to validate UX design"
echo ""

# Scenario 1: Command Discovery
print_header "Scenario 1: First-time user discovers commands"

ux_info "User runs 'uveddi --help' to discover available commands"
help_output=$(uveddi --help 2>&1)

# Check if help is clear and well-organized
if echo "$help_output" | grep -q "Commands:"; then
    ux_pass "Help output has clear command section"
else
    ux_fail "Help output lacks clear command organization"
fi

# Check if TUI is prominently featured
if echo "$help_output" | grep -A 3 -B 3 "tui" | grep -qi "terminal\|interactive"; then
    ux_pass "TUI command is clearly described as interactive"
else
    ux_fail "TUI command description is unclear"
fi

# Check for logical command grouping
if echo "$help_output" | grep -q "analyze" && echo "$help_output" | grep -q "tui" && echo "$help_output" | grep -q "serve"; then
    ux_pass "Main user commands (analyze, tui, serve) are all present"
else
    ux_fail "Missing key user commands in help"
fi

echo ""

# Scenario 2: TUI Command Discovery
print_header "Scenario 2: User explores TUI-specific options"

ux_info "User runs 'uveddi tui --help' to learn about TUI options"
tui_help=$(uveddi tui --help 2>&1)

# Check for clear option descriptions
if echo "$tui_help" | grep -q "PATH.*Project path"; then
    ux_pass "PATH argument clearly explained"
else
    ux_fail "PATH argument explanation unclear"
fi

if echo "$tui_help" | grep -q "skip-analysis.*Skip"; then
    ux_pass "Skip analysis option clearly explained"
else
    ux_fail "Skip analysis option explanation unclear"
fi

if echo "$tui_help" | grep -q "debug.*debug"; then
    ux_pass "Debug option clearly explained"
else
    ux_fail "Debug option explanation unclear"
fi

# Check for reasonable defaults
if echo "$tui_help" | grep -q "default.*\."; then
    ux_pass "Default path (current directory) specified"
else
    ux_fail "Default path not clearly indicated"
fi

echo ""

# Scenario 3: Error Handling and Guidance
print_header "Scenario 3: User tries TUI in wrong environment"

ux_info "User attempts to run TUI in non-interactive environment"
cd /home/testuser/test-workspace

# Capture error output
error_output=$(uveddi tui ./projects/rust-sample 2>&1 || true)

# Check error message quality
if echo "$error_output" | grep -qi "interactive terminal"; then
    ux_pass "Error clearly explains interactive terminal requirement"
else
    ux_fail "Error doesn't explain terminal requirement clearly"
fi

# Check for helpful suggestions
if echo "$error_output" | grep -q "uveddi analyze"; then
    ux_pass "Error suggests analyze command alternative"
else
    ux_fail "Error missing analyze command suggestion"
fi

if echo "$error_output" | grep -q "uveddi serve"; then
    ux_pass "Error suggests web interface alternative"
else
    ux_fail "Error missing web interface suggestion"
fi

# Check for example commands
if echo "$error_output" | grep -q "\-\-output-format"; then
    ux_pass "Error provides example analyze command"
else
    ux_fail "Error lacks concrete example commands"
fi

echo ""

# Scenario 4: Path Validation Feedback
print_header "Scenario 4: User provides invalid path"

ux_info "User tries TUI with non-existent path"
path_error=$(uveddi tui ./non-existent-path 2>&1 || true)

if echo "$path_error" | grep -qi "does not exist"; then
    ux_pass "Clear feedback for non-existent path"
else
    ux_fail "Unclear feedback for invalid path"
fi

if echo "$path_error" | grep -qi "valid.*path"; then
    ux_pass "Error suggests providing valid path"
else
    ux_fail "Error lacks guidance on path requirements"
fi

echo ""

# Scenario 5: Results File Validation
print_header "Scenario 5: User provides invalid results file"

ux_info "User tries to load non-existent results file"
results_error=$(uveddi tui ./projects/rust-sample --load-results ./missing.json 2>&1 || true)

if echo "$results_error" | grep -qi "results file.*not exist"; then
    ux_pass "Clear feedback for missing results file"
else
    ux_fail "Unclear feedback for missing results file"
fi

if echo "$results_error" | grep -qi "JSON.*results"; then
    ux_pass "Error explains expected file type"
else
    ux_fail "Error doesn't explain expected file format"
fi

echo ""

# Scenario 6: Command Workflow Discovery
print_header "Scenario 6: User discovers analysis workflow"

ux_info "Testing logical command workflow discovery"

# Test that analyze command is obvious
analyze_help=$(uveddi analyze --help 2>&1)
if echo "$analyze_help" | grep -q "output-format"; then
    ux_pass "Analyze command shows output format options"
else
    ux_fail "Analyze command output options unclear"
fi

# Test serve command visibility
serve_help=$(uveddi serve --help 2>&1)
if echo "$serve_help" | grep -q "port"; then
    ux_pass "Serve command shows port configuration"
else
    ux_fail "Serve command configuration unclear"
fi

echo ""

# Scenario 7: Feature Discovery
print_header "Scenario 7: User explores advanced features"

ux_info "Testing advanced feature discoverability"

# Check if debug mode is documented
if echo "$tui_help" | grep -q "debug"; then
    ux_pass "Debug mode is discoverable"
else
    ux_fail "Debug mode not easily discoverable"
fi

# Check if force flag exists with warning
if echo "$tui_help" | grep -i "force.*not recommended\|force.*override"; then
    ux_pass "Force flag includes appropriate warning"
else
    ux_fail "Force flag lacks safety warning"
fi

echo ""

# Scenario 8: Integration Verification
print_header "Scenario 8: User verifies installation"

ux_info "Testing that users can verify proper installation"

# Check version information
version_output=$(uveddi --version 2>&1 || echo "No version info")
if echo "$version_output" | grep -q -E "[0-9]+\.[0-9]+"; then
    ux_pass "Version information available"
else
    ux_fail "Version information missing or unclear"
fi

# Check command completeness
main_help=$(uveddi --help 2>&1)
expected_commands=("analyze" "config" "ui" "ci" "tui" "serve")
missing_commands=()

for cmd in "${expected_commands[@]}"; do
    if echo "$main_help" | grep -q "$cmd"; then
        ux_pass "Command present: $cmd"
    else
        ux_fail "Command missing: $cmd"
        missing_commands+=("$cmd")
    fi
done

echo ""

# Scenario 9: Documentation and Help Consistency
print_header "Scenario 9: Documentation consistency check"

ux_info "Verifying help documentation consistency"

# Check that 'help tui' works
help_tui_output=$(uveddi help tui 2>&1 || echo "Help command failed")
if echo "$help_tui_output" | grep -q "Terminal User Interface\|TUI\|interactive"; then
    ux_pass "Help system supports TUI command"
else
    ux_fail "Help system doesn't properly document TUI"
fi

# Verify consistent terminology
main_tui_description=$(echo "$main_help" | grep "tui" | head -1)
if echo "$main_tui_description" | grep -qi "terminal.*user.*interface\|interactive"; then
    ux_pass "Consistent TUI terminology used"
else
    ux_fail "Inconsistent TUI terminology"
fi

echo ""

# Scenario 10: Real-world usage patterns
print_header "Scenario 10: Common usage patterns"

ux_info "Testing common user workflows"

# Test quick analysis workflow
if uveddi analyze ./projects/rust-sample --output-format json --output ./results/ux-test.json >/dev/null 2>&1; then
    ux_pass "Quick analysis workflow works"
else
    ux_fail "Quick analysis workflow broken"
fi

# Test different output formats for user choice
formats=("json" "html" "markdown")
for format in "${formats[@]}"; do
    if uveddi analyze ./projects/rust-sample --output-format "$format" --output "./results/ux-test.$format" >/dev/null 2>&1; then
        ux_pass "Output format works: $format"
    else
        ux_fail "Output format broken: $format"
    fi
done

# Test that serve command starts without errors (for web UI users)
# (We'll just test help since actually starting requires more setup)
if uveddi serve --help | grep -q "dashboard"; then
    ux_pass "Web dashboard option documented"
else
    ux_fail "Web dashboard option not clear"
fi

echo ""

# UX Assessment
print_header "📊 User Experience Assessment"

echo "UX Score: $UX_SCORE / $MAX_UX_SCORE"
percentage=$((UX_SCORE * 100 / MAX_UX_SCORE))
echo "UX Quality: $percentage%"

if [ $percentage -ge 90 ]; then
    echo -e "${GREEN}🏆 EXCELLENT UX! Users will have a smooth experience.${NC}"
    ux_rating="EXCELLENT"
elif [ $percentage -ge 80 ]; then
    echo -e "${GREEN}✅ GOOD UX! Minor improvements possible.${NC}"
    ux_rating="GOOD"
elif [ $percentage -ge 70 ]; then
    echo -e "${YELLOW}⚠️  FAIR UX. Some improvements needed.${NC}"
    ux_rating="FAIR"
else
    echo -e "${RED}❌ POOR UX. Significant improvements required.${NC}"
    ux_rating="POOR"
fi

echo ""
echo "📋 UX Evaluation Summary:"
echo "========================="
echo "• Command Discovery: $(if echo "$main_help" | grep -q "tui.*Terminal"; then echo "✅ Good"; else echo "❌ Needs work"; fi)"
echo "• Error Messages: $(if echo "$error_output" | grep -q "interactive terminal"; then echo "✅ Helpful"; else echo "❌ Unclear"; fi)"
echo "• Help Documentation: $(if echo "$tui_help" | grep -q "PATH.*Project"; then echo "✅ Clear"; else echo "❌ Confusing"; fi)"
echo "• Workflow Integration: $(if [ $UX_SCORE -gt $((MAX_UX_SCORE * 3 / 4)) ]; then echo "✅ Smooth"; else echo "❌ Fragmented"; fi)"

echo ""
echo "💡 UX Recommendations:"
echo "======================"

if [ $percentage -lt 90 ]; then
    echo "• Improve error message clarity and provide more specific guidance"
    echo "• Enhance help documentation with examples"
    echo "• Consider adding more interactive features discovery"
fi

if [ $percentage -lt 80 ]; then
    echo "• Add command completion suggestions"
    echo "• Improve terminology consistency across all commands"
    echo "• Consider adding a 'quick start' guide in help"
fi

if [ $percentage -lt 70 ]; then
    echo "• Major revision of command structure needed"
    echo "• Error handling requires significant improvement"
    echo "• Documentation needs comprehensive review"
fi

echo ""
echo "👥 Target User Validation:"
echo "========================="
echo "• Beginner developers: $(if [ $percentage -ge 80 ]; then echo "✅ Supported"; else echo "❌ May struggle"; fi)"
echo "• Experienced developers: $(if [ $percentage -ge 70 ]; then echo "✅ Supported"; else echo "❌ May find confusing"; fi)"
echo "• CI/CD integration: $(if uveddi analyze --help | grep -q "output-format"; then echo "✅ Well supported"; else echo "❌ Poorly supported"; fi)"

# Final assessment
if [ $percentage -ge 80 ]; then
    echo ""
    echo -e "${GREEN}🎯 RECOMMENDATION: Ready for user testing and deployment${NC}"
    exit 0
else
    echo ""
    echo -e "${YELLOW}🎯 RECOMMENDATION: Address UX issues before broader deployment${NC}"
    exit 1
fi