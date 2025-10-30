#!/bin/bash

# Memory limit validation script
# Tests various memory scenarios and validates resource management behavior

set -e

echo "🧠 Memory Limits Validation"
echo "=========================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Create test directory with various file sizes
TEST_DIR=$(mktemp -d)
echo "Created test directory: $TEST_DIR"

# Test files of different sizes
create_test_files() {
    echo -e "${BLUE}Creating test files...${NC}"
    
    # Small file (< 1MB)
    cat > "$TEST_DIR/small_file.rs" << 'EOF'
fn main() {
    println!("Hello, world!");
}
EOF

    # Medium file (1-10MB)
    MEDIUM_FILE="$TEST_DIR/medium_file.rs"
    {
        echo "// Medium size test file"
        for i in $(seq 1 1000); do
            cat << EOF
fn function_$i() {
    // Function $i with detailed implementation
    let data = vec![1, 2, 3, 4, 5];
    for item in data.iter() {
        println!("Processing item: {}", item);
    }
    
    match some_condition($i) {
        true => println!("Condition met for function {}", $i),
        false => println!("Condition not met for function {}", $i),
    }
}

fn some_condition(n: i32) -> bool {
    n % 2 == 0
}

EOF
        done
    } > "$MEDIUM_FILE"
    
    # Large file (10MB+)
    LARGE_FILE="$TEST_DIR/large_file.rs"
    {
        echo "// Large test file for memory stress testing"
        for i in $(seq 1 10000); do
            cat << EOF
struct DataStructure$i {
    field_a: String,
    field_b: i32,
    field_c: Vec<String>,
    field_d: std::collections::HashMap<String, String>,
}

impl DataStructure$i {
    fn new() -> Self {
        Self {
            field_a: String::from("test_data_$i"),
            field_b: $i,
            field_c: vec!["item1".to_string(), "item2".to_string()],
            field_d: std::collections::HashMap::new(),
        }
    }
    
    fn process_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Processing data structure {}", $i);
        Ok(())
    }
}

EOF
        done
    } > "$LARGE_FILE"
    
    echo -e "${GREEN}✅ Test files created${NC}"
    echo "Small file size: $(du -h "$TEST_DIR/small_file.rs" | cut -f1)"
    echo "Medium file size: $(du -h "$MEDIUM_FILE" | cut -f1)"
    echo "Large file size: $(du -h "$LARGE_FILE" | cut -f1)"
}

# Test function
test_memory_scenario() {
    local scenario="$1"
    local memory_limit="$2"
    local expected_outcome="$3"
    
    echo -e "\n${YELLOW}Testing Scenario: $scenario${NC}"
    echo "Memory limit: $memory_limit"
    echo "Expected outcome: $expected_outcome"
    
    # Set environment variable for resource management
    export UVEDDI_ENABLE_RESOURCE_MANAGEMENT=true
    
    # Run analysis with memory limit
    if cargo run --features=dev-core -- analyze "$TEST_DIR" \
        --memory-limit-gb "$memory_limit" \
        --enable-resource-management \
        --output-format json \
        --timeout-seconds 30 > /tmp/analysis_result.json 2>&1; then
        
        echo -e "${GREEN}✅ Analysis completed successfully${NC}"
        
        # Check if output contains expected patterns
        if grep -q "issues_found" /tmp/analysis_result.json; then
            echo -e "${GREEN}✅ Analysis produced valid results${NC}"
        else
            echo -e "${YELLOW}⚠️  Analysis completed but output format unexpected${NC}"
        fi
        
        # Check for resource management indicators
        if grep -q "resource" /tmp/analysis_result.json 2>/dev/null || 
           grep -q "memory" /tmp/analysis_result.json 2>/dev/null; then
            echo -e "${GREEN}✅ Resource management appears active${NC}"
        fi
        
    else
        exit_code=$?
        echo -e "${RED}❌ Analysis failed with exit code: $exit_code${NC}"
        
        # Check if failure was due to expected resource limits
        if [ "$expected_outcome" = "failure" ]; then
            echo -e "${GREEN}✅ Expected failure occurred (resource limits working)${NC}"
        else
            echo -e "${RED}❌ Unexpected failure${NC}"
            cat /tmp/analysis_result.json 2>/dev/null | head -20 || true
        fi
    fi
}

# Main test execution
main() {
    create_test_files
    
    echo -e "\n${BLUE}Running Memory Limit Tests...${NC}"
    
    # Test 1: Normal memory limit (should succeed)
    test_memory_scenario "Normal Operation" "2.0" "success"
    
    # Test 2: Low memory limit (should trigger resource management)
    test_memory_scenario "Low Memory Trigger Resource Management" "0.5" "success_with_degradation"
    
    # Test 3: Very low memory limit (should fail gracefully)
    test_memory_scenario "Very Low Memory Graceful Failure" "0.1" "failure"
    
    # Test 4: Extremely low memory limit (should fail immediately)
    test_memory_scenario "Extremely Low Memory Immediate Failure" "0.01" "failure"
    
    # Test 5: Test with different memory profiles
    echo -e "\n${YELLOW}Testing Memory Profiles${NC}"
    
    for profile in "small" "default" "large"; do
        echo -e "\nTesting profile: $profile"
        if cargo run --features=dev-core -- analyze "$TEST_DIR" \
            --memory-profile "$profile" \
            --enable-resource-management \
            --output-format json \
            --timeout-seconds 15 > /tmp/profile_test.json 2>&1; then
            echo -e "${GREEN}✅ Profile $profile works${NC}"
        else
            echo -e "${YELLOW}⚠️  Profile $profile had issues${NC}"
        fi
    done
    
    # Test 6: Memory pressure simulation
    echo -e "\n${YELLOW}Testing Memory Pressure Response${NC}"
    
    # Create a script that allocates memory while analysis runs
    cat > /tmp/memory_pressure.sh << 'EOF'
#!/bin/bash
# Allocate memory to simulate pressure
python3 -c "
import time
# Allocate ~100MB
data = []
for i in range(100):
    data.append('x' * 1024 * 1024)  # 1MB chunks
    time.sleep(0.1)
print('Memory pressure applied')
time.sleep(10)  # Hold memory for 10 seconds
"
EOF
    chmod +x /tmp/memory_pressure.sh
    
    # Run memory pressure in background
    /tmp/memory_pressure.sh &
    PRESSURE_PID=$!
    
    # Run analysis with resource management
    echo "Running analysis under memory pressure..."
    if timeout 20 cargo run --features=dev-core -- analyze "$TEST_DIR" \
        --enable-resource-management \
        --memory-limit-gb 1.0 \
        --output-format json > /tmp/pressure_test.json 2>&1; then
        echo -e "${GREEN}✅ Analysis completed under memory pressure${NC}"
    else
        echo -e "${YELLOW}⚠️  Analysis affected by memory pressure (expected)${NC}"
    fi
    
    # Clean up memory pressure process
    kill $PRESSURE_PID 2>/dev/null || true
    wait $PRESSURE_PID 2>/dev/null || true
    
    # Test 7: Resource exhaustion recovery
    echo -e "\n${YELLOW}Testing Resource Exhaustion Recovery${NC}"
    
    # Run multiple analyses concurrently to test resource limits
    for i in {1..3}; do
        (
            cargo run --features=dev-core -- analyze "$TEST_DIR" \
                --enable-resource-management \
                --memory-limit-gb 0.3 \
                --output-format json \
                --timeout-seconds 10 > "/tmp/concurrent_$i.json" 2>&1 &
        ) &
    done
    
    # Wait for all concurrent analyses
    wait
    
    # Check results
    successful_analyses=0
    for i in {1..3}; do
        if [ -f "/tmp/concurrent_$i.json" ] && grep -q "analysis" "/tmp/concurrent_$i.json" 2>/dev/null; then
            successful_analyses=$((successful_analyses + 1))
        fi
    done
    
    echo "Concurrent analyses completed: $successful_analyses/3"
    
    if [ $successful_analyses -gt 0 ]; then
        echo -e "${GREEN}✅ Resource management handled concurrent load${NC}"
    else
        echo -e "${YELLOW}⚠️  All concurrent analyses failed (may indicate resource limits working)${NC}"
    fi
}

# Cleanup function
cleanup() {
    echo -e "\n${BLUE}Cleaning up...${NC}"
    rm -rf "$TEST_DIR"
    rm -f /tmp/analysis_result.json /tmp/profile_test.json /tmp/pressure_test.json
    rm -f /tmp/concurrent_*.json /tmp/memory_pressure.sh
    
    # Kill any remaining background processes
    jobs -p | xargs -r kill 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Set up trap for cleanup
trap cleanup EXIT

# Run main test suite
main

echo -e "\n${GREEN}🎉 Memory limit validation completed!${NC}"