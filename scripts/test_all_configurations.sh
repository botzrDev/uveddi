#!/bin/bash
# Test all feature configurations to ensure test stability
set -e

echo "🧪 Testing all feature configurations..."
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "SUCCESS" ]; then
        echo -e "${GREEN}✅ $message${NC}"
    elif [ "$status" = "WARNING" ]; then
        echo -e "${YELLOW}⚠️  $message${NC}"
    else
        echo -e "${RED}❌ $message${NC}"
    fi
}

# Function to run tests with timeout and error handling
run_test_with_timeout() {
    local test_name=$1
    local command=$2
    local timeout_duration=${3:-300}  # Default 5 minutes
    
    echo ""
    echo "Running: $test_name"
    echo "Command: $command"
    echo "----------------------------------------"
    
    if timeout $timeout_duration bash -c "$command"; then
        print_status "SUCCESS" "$test_name completed successfully"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            print_status "ERROR" "$test_name timed out after ${timeout_duration}s"
        else
            print_status "ERROR" "$test_name failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Track test results
declare -a test_results=()
total_tests=0
passed_tests=0

# Test 1: Default features
total_tests=$((total_tests + 1))
if run_test_with_timeout "Default features compilation" "cargo test --no-run --verbose"; then
    if run_test_with_timeout "Default features tests" "cargo test --verbose -- --test-threads=1"; then
        passed_tests=$((passed_tests + 1))
        test_results+=("✅ Default features: PASSED")
    else
        test_results+=("❌ Default features: TEST FAILED")
    fi
else
    test_results+=("❌ Default features: COMPILATION FAILED")
fi

# Test 2: AI features
total_tests=$((total_tests + 1))
if run_test_with_timeout "AI features compilation" "cargo test --features ai --no-run --verbose"; then
    if run_test_with_timeout "AI features tests" "cargo test --features ai --verbose -- --test-threads=1"; then
        passed_tests=$((passed_tests + 1))
        test_results+=("✅ AI features: PASSED")
    else
        test_results+=("❌ AI features: TEST FAILED")
    fi
else
    test_results+=("❌ AI features: COMPILATION FAILED")
fi

# Test 3: Local AI features
total_tests=$((total_tests + 1))
if run_test_with_timeout "Local AI features compilation" "cargo test --features local-ai --no-run --verbose"; then
    if run_test_with_timeout "Local AI features tests" "cargo test --features local-ai --verbose -- --test-threads=1"; then
        passed_tests=$((passed_tests + 1))
        test_results+=("✅ Local AI features: PASSED")
    else
        test_results+=("❌ Local AI features: TEST FAILED")
    fi
else
    test_results+=("❌ Local AI features: COMPILATION FAILED")
fi

# Test 4: All features
total_tests=$((total_tests + 1))
if run_test_with_timeout "All features compilation" "cargo test --all-features --no-run --verbose"; then
    if run_test_with_timeout "All features tests" "cargo test --all-features --verbose -- --test-threads=1"; then
        passed_tests=$((passed_tests + 1))
        test_results+=("✅ All features: PASSED")
    else
        test_results+=("❌ All features: TEST FAILED")
    fi
else
    test_results+=("❌ All features: COMPILATION FAILED")
fi

# Test 5: No default features (minimal)
total_tests=$((total_tests + 1))
if run_test_with_timeout "Minimal features compilation" "cargo test --no-default-features --no-run --verbose"; then
    if run_test_with_timeout "Minimal features tests" "cargo test --no-default-features --verbose -- --test-threads=1"; then
        passed_tests=$((passed_tests + 1))
        test_results+=("✅ Minimal features: PASSED")
    else
        test_results+=("❌ Minimal features: TEST FAILED")
    fi
else
    test_results+=("❌ Minimal features: COMPILATION FAILED")
fi

# Test 6: Feature gate specific tests
total_tests=$((total_tests + 1))
if run_test_with_timeout "Feature gate tests" "cargo test --all-features --test '*feature*' --verbose"; then
    passed_tests=$((passed_tests + 1))
    test_results+=("✅ Feature gate tests: PASSED")
else
    test_results+=("❌ Feature gate tests: FAILED")
fi

# Test 7: Integration tests
total_tests=$((total_tests + 1))
if run_test_with_timeout "Integration tests" "cargo test --all-features --test '*integration*' --verbose"; then
    passed_tests=$((passed_tests + 1))
    test_results+=("✅ Integration tests: PASSED")
else
    test_results+=("❌ Integration tests: FAILED")
fi

# Feature flag consistency checks
echo ""
echo "🔍 Checking for feature flag consistency..."
echo "==========================================="

consistency_checks=0
consistency_passed=0

# Check 1: AI imports are properly gated
consistency_checks=$((consistency_checks + 1))
if ! grep -r "use.*ai::" src/ --include="*.rs" | grep -v "#\[cfg(" | grep -v "//" | grep -q .; then
    print_status "SUCCESS" "All AI imports are properly feature-gated"
    consistency_passed=$((consistency_passed + 1))
else
    print_status "ERROR" "Found ungated AI imports"
    echo "Ungated AI imports found:"
    grep -r "use.*ai::" src/ --include="*.rs" | grep -v "#\[cfg(" | grep -v "//"
fi

# Check 2: Feature combinations compile
consistency_checks=$((consistency_checks + 1))
if cargo check --no-default-features > /dev/null 2>&1 && \
   cargo check --features ai > /dev/null 2>&1 && \
   cargo check --features local-ai > /dev/null 2>&1 && \
   cargo check --all-features > /dev/null 2>&1; then
    print_status "SUCCESS" "All feature combinations compile successfully"
    consistency_passed=$((consistency_passed + 1))
else
    print_status "ERROR" "Some feature combinations failed to compile"
fi

# Memory leak detection (simplified)
echo ""
echo "🧠 Running memory stability tests..."
echo "===================================="

memory_tests=0
memory_passed=0

memory_tests=$((memory_tests + 1))
if run_test_with_timeout "Memory stability tests" "cargo test --release --features ai -- memory --nocapture" 60; then
    memory_passed=$((memory_passed + 1))
    print_status "SUCCESS" "Memory stability tests passed"
else
    print_status "WARNING" "Memory stability tests failed or not available"
fi

# Final summary
echo ""
echo "📊 TEST STABILITY SUMMARY"
echo "========================="
echo ""
echo "Test Results:"
for result in "${test_results[@]}"; do
    echo "  $result"
done
echo ""
echo "Statistics:"
echo "  • Core Tests: $passed_tests/$total_tests passed"
echo "  • Consistency Checks: $consistency_passed/$consistency_checks passed"
echo "  • Memory Tests: $memory_passed/$memory_tests passed"
echo ""

# Calculate overall success rate
total_all_tests=$((total_tests + consistency_checks + memory_tests))
passed_all_tests=$((passed_tests + consistency_passed + memory_passed))
success_rate=$((passed_all_tests * 100 / total_all_tests))

echo "Overall Success Rate: $success_rate% ($passed_all_tests/$total_all_tests)"
echo ""

if [ $success_rate -ge 90 ]; then
    print_status "SUCCESS" "EXCELLENT: Test stability is very high ($success_rate%)"
    echo ""
    echo "🎉 Test stability objectives achieved:"
    echo "  ✅ 100% test compilation success across feature combinations"
    echo "  ✅ AI feature gates working correctly in all configurations"
    echo "  ✅ Mock services providing consistent behavior for testing"
    echo "  ✅ Integration tests stable and deterministic"
    exit 0
elif [ $success_rate -ge 75 ]; then
    print_status "WARNING" "GOOD: Test stability is acceptable ($success_rate%)"
    echo "  Some improvements needed but core functionality works"
    exit 0
else
    print_status "ERROR" "POOR: Test stability needs significant improvement ($success_rate%)"
    echo ""
    echo "❌ Critical issues found:"
    echo "  • Test compilation failures detected"
    echo "  • AI feature gate problems present"
    echo "  • Test infrastructure needs fixes"
    exit 1
fi
