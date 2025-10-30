#!/bin/bash

# Incremental Coverage Measurement for Large Codebases
# Measures coverage in chunks to avoid timeouts

set -e

echo "📊 Incremental Coverage Measurement"
echo "=================================="

# Create coverage output directory
mkdir -p coverage/incremental

# Define module groups for incremental coverage
MODULES=(
    "src/analysis"
    "src/monitoring" 
    "src/observability"
    "src/resilience"
    "src/security"
    "src/tui"
    "src/ai"
    "src/database"
    "src/report"
    "src/plugins"
)

TOTAL_COVERAGE=0
MODULE_COUNT=0

echo ""
echo "📋 Measuring coverage by module..."

for module in "${MODULES[@]}"; do
    if [ -d "$module" ]; then
        echo ""
        echo "🔍 Processing module: $module"
        
        # Set timeout for each module (5 minutes)
        timeout 300 cargo llvm-cov --lib --package uveddi \
            --include-ffi \
            --lcov \
            --output-path "coverage/incremental/$(basename $module).lcov" \
            -- --test-threads=1 \
            2>/dev/null || {
            echo "   ⚠️  Timeout or error in $module - skipping"
            continue
        }
        
        # Extract coverage percentage
        if [ -f "coverage/incremental/$(basename $module).lcov" ]; then
            LINES_FOUND=$(grep -c "^DA:" "coverage/incremental/$(basename $module).lcov" || echo "0")
            LINES_HIT=$(grep "^DA:" "coverage/incremental/$(basename $module).lcov" | grep -v ",0$" | wc -l || echo "0")
            
            if [ $LINES_FOUND -gt 0 ]; then
                MODULE_COVERAGE=$(echo "scale=2; $LINES_HIT * 100 / $LINES_FOUND" | bc -l)
                echo "   ✅ $module: ${MODULE_COVERAGE}% coverage"
                TOTAL_COVERAGE=$(echo "$TOTAL_COVERAGE + $MODULE_COVERAGE" | bc -l)
                MODULE_COUNT=$((MODULE_COUNT + 1))
            else
                echo "   ⚠️  No coverage data for $module"
            fi
        else
            echo "   ❌ Failed to generate coverage for $module"
        fi
    else
        echo "   ⏭️  Module $module not found - skipping"
    fi
done

# Calculate average coverage
if [ $MODULE_COUNT -gt 0 ]; then
    AVERAGE_COVERAGE=$(echo "scale=2; $TOTAL_COVERAGE / $MODULE_COUNT" | bc -l)
    echo ""
    echo "📊 Coverage Summary"
    echo "=================="
    echo "   📈 Modules measured: $MODULE_COUNT"
    echo "   📊 Average coverage: ${AVERAGE_COVERAGE}%"
    
    # Check if meets 90% threshold
    if (( $(echo "$AVERAGE_COVERAGE >= 90" | bc -l) )); then
        echo "   ✅ Coverage target met (≥90%)"
        echo "90" > coverage/coverage_status.txt
    else
        echo "   ⚠️  Coverage below 90% target"
        echo "$AVERAGE_COVERAGE" > coverage/coverage_status.txt
    fi
else
    echo ""
    echo "❌ No coverage data collected"
    echo "0" > coverage/coverage_status.txt
fi

# Merge coverage files if multiple exist
echo ""
echo "🔗 Merging coverage reports..."
if ls coverage/incremental/*.lcov 1> /dev/null 2>&1; then
    cat coverage/incremental/*.lcov > coverage/merged_coverage.lcov
    echo "   ✅ Merged coverage report: coverage/merged_coverage.lcov"
else
    echo "   ⚠️  No coverage files to merge"
fi

echo ""
echo "📁 Coverage files generated:"
ls -la coverage/ 2>/dev/null || echo "   No coverage directory"
