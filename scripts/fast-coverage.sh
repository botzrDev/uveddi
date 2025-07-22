#!/bin/bash

# Fast Coverage Measurement using cargo-tarpaulin
# Lighter alternative to llvm-cov for timeout-prone environments

set -e

echo "⚡ Fast Coverage Measurement"
echo "=========================="

# Install tarpaulin if not available
if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin --timeout 300 || {
        echo "❌ Failed to install cargo-tarpaulin"
        exit 1
    }
fi

echo ""
echo "📊 Running fast coverage analysis..."

# Create coverage directory
mkdir -p coverage

# Run tarpaulin with optimizations for large codebases
timeout 600 cargo tarpaulin \
    --verbose \
    --timeout 120 \
    --jobs 2 \
    --exclude-files "target/*" \
    --exclude-files "tests/*" \
    --exclude-files "benches/*" \
    --exclude-files "examples/*" \
    --out Html \
    --out Lcov \
    --output-dir coverage \
    --skip-clean \
    --force-clean \
    2>&1 | tee coverage/tarpaulin.log || {
    
    echo "⚠️  Full coverage timed out, trying core modules only..."
    
    # Fallback: measure only core modules
    timeout 300 cargo tarpaulin \
        --verbose \
        --timeout 60 \
        --jobs 1 \
        --packages uveddi \
        --exclude-files "target/*" \
        --exclude-files "tests/*" \
        --exclude-files "benches/*" \
        --lib \
        --out Html \
        --out Lcov \
        --output-dir coverage \
        --skip-clean \
        2>&1 | tee coverage/tarpaulin_core.log || {
        
        echo "❌ Coverage measurement failed - build timeout persists"
        echo "0" > coverage/coverage_status.txt
        exit 1
    }
}

# Extract coverage percentage from output
if [ -f "coverage/tarpaulin-report.html" ]; then
    COVERAGE=$(grep -o '[0-9]*\.[0-9]*%' coverage/tarpaulin-report.html | head -1 | sed 's/%//')
    echo ""
    echo "📊 Coverage Result: ${COVERAGE}%"
    echo "$COVERAGE" > coverage/coverage_status.txt
    
    if (( $(echo "$COVERAGE >= 90" | bc -l) )); then
        echo "✅ Coverage target met (≥90%)"
    else
        echo "⚠️  Coverage below 90% target"
    fi
else
    echo "❌ No coverage report generated"
    echo "0" > coverage/coverage_status.txt
fi

echo ""
echo "📁 Coverage files:"
ls -la coverage/ 2>/dev/null || echo "No coverage files"
