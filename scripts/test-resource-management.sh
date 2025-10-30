#!/bin/bash

# Test script for resource management functionality
# Validates memory limits, resource monitoring, and graceful degradation

set -e

echo "🧪 Resource Management Test Suite"
echo "================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test function wrapper
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${YELLOW}Testing: $test_name${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Test 1: Basic Memory Tracking
run_test "Memory Tracker Basic Operations" "cargo test --test resource_management_tests memory_tracker_tests::test_memory_allocation_and_release -- --nocapture"

# Test 2: Memory Exhaustion Handling
run_test "Memory Exhaustion Protection" "cargo test --test resource_management_tests memory_tracker_tests::test_memory_exhaustion -- --nocapture"

# Test 3: Memory Pressure Calculation
run_test "Memory Pressure Monitoring" "cargo test --test resource_management_tests memory_tracker_tests::test_memory_pressure_calculation -- --nocapture"

# Test 4: Peak Memory Tracking
run_test "Peak Memory Tracking" "cargo test --test resource_management_tests memory_tracker_tests::test_peak_memory_tracking -- --nocapture"

# Test 5: Component Memory Tracking
run_test "Component Memory Tracking" "cargo test --test resource_management_tests memory_tracker_tests::test_component_tracking -- --nocapture"

# Test 6: File Processing - Small Files
run_test "Small File In-Memory Processing" "cargo test --test resource_management_tests streaming_file_processor_tests::test_small_file_in_memory_processing -- --nocapture"

# Test 7: File Processing - Large Files (Streaming)
run_test "Large File Streaming Processing" "cargo test --test resource_management_tests streaming_file_processor_tests::test_large_file_streaming -- --nocapture"

# Test 8: File Type Filtering
run_test "File Type Filtering" "cargo test --test resource_management_tests streaming_file_processor_tests::test_file_type_filtering -- --nocapture"

# Test 9: Binary Data Detection
run_test "Binary Data Detection" "cargo test --test resource_management_tests streaming_file_processor_tests::test_binary_data_detection -- --nocapture"

# Test 10: Concurrent Analysis Limiting
run_test "Concurrent Analysis Limits" "cargo test --test resource_management_tests analysis_orchestrator_tests::test_concurrent_analysis_limiting -- --nocapture"

# Test 11: Memory-Based Analysis Rejection
run_test "Memory-Based Analysis Rejection" "cargo test --test resource_management_tests analysis_orchestrator_tests::test_memory_based_analysis_rejection -- --nocapture"

# Test 12: Analysis Priority Ordering
run_test "Analysis Priority Handling" "cargo test --test resource_management_tests analysis_orchestrator_tests::test_analysis_priority_ordering -- --nocapture"

# Test 13: Degradation Level Progression
run_test "Degradation Level Progression" "cargo test --test resource_management_tests degradation_manager_tests::test_degradation_level_progression -- --nocapture"

# Test 14: Feature State Management
run_test "Feature State Management" "cargo test --test resource_management_tests degradation_manager_tests::test_feature_state_management -- --nocapture"

# Test 15: Emergency Cleanup
run_test "Emergency Cleanup" "cargo test --test resource_management_tests degradation_manager_tests::test_emergency_cleanup -- --nocapture"

# Test 16: Degradation History
run_test "Degradation History Tracking" "cargo test --test resource_management_tests degradation_manager_tests::test_degradation_history -- --nocapture"

# Test 17: Resource Monitoring
run_test "Resource Monitor Creation" "cargo test --test resource_management_tests resource_monitor_tests::test_resource_monitoring_creation -- --nocapture"

# Test 18: Alert Generation
run_test "Memory Pressure Alert Generation" "cargo test --test resource_management_tests resource_monitor_tests::test_memory_pressure_alert_generation -- --nocapture"

# Test 19: Alert Acknowledgment
run_test "Alert Acknowledgment" "cargo test --test resource_management_tests resource_monitor_tests::test_alert_acknowledgment -- --nocapture"

# Test 20: Emergency Alert
run_test "Emergency Cleanup Alert" "cargo test --test resource_management_tests resource_monitor_tests::test_emergency_cleanup_alert -- --nocapture"

# Test 21: Full Lifecycle Integration
run_test "Full Resource Management Lifecycle" "cargo test --test resource_management_tests integration_tests::test_full_resource_management_lifecycle -- --nocapture"

# Test 22: Stress Test
run_test "Memory Allocation Stress Test" "cargo test --test resource_management_tests integration_tests::test_stress_test_memory_allocation -- --nocapture"

# Test 23: Degradation Under Pressure
run_test "Degradation Under Pressure" "cargo test --test resource_management_tests integration_tests::test_degradation_under_pressure -- --nocapture"

# Memory exhaustion simulation test
echo -e "\n${YELLOW}Running Memory Exhaustion Simulation${NC}"
run_test "Memory Exhaustion Simulation" 'cargo run --features=dev-core -- analyze ./src --enable-resource-management --memory-limit-gb 0.1 --output-format json | grep -q "analysis"'

# Large file processing test
echo -e "\n${YELLOW}Running Large File Processing Test${NC}"
# Create a temporary large file
TEMP_DIR=$(mktemp -d)
LARGE_FILE="$TEMP_DIR/large_test_file.rs"

echo "Creating large test file..."
{
    echo "// Large test file for resource management testing"
    for i in $(seq 1 5000); do
        echo "fn test_function_$i() {"
        echo "    // This is test function number $i"
        echo "    println!(\"Function $i executing\");"
        echo "}"
        echo ""
    done
} > "$LARGE_FILE"

run_test "Large File Processing" "UVEDDI_ENABLE_RESOURCE_MANAGEMENT=true cargo run --features=dev-core -- analyze $TEMP_DIR --output-format json | grep -q 'issues_found'"

# Cleanup
rm -rf "$TEMP_DIR"

# Test Summary
echo -e "\n${YELLOW}Test Summary${NC}"
echo "============="
echo "Total Tests:  $TOTAL_TESTS"
echo -e "Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:       ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All resource management tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. See output above for details.${NC}"
    exit 1
fi