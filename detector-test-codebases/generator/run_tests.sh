#!/usr/bin/env bash
# Complete automated testing script for Uveddi detectors
# This script generates extreme test files, clones real repos, and runs comprehensive tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(dirname "$SCRIPT_DIR")"
REPO_ROOT="$(dirname "$BASE_DIR")"

echo "=== UVEDDI EXTREME DETECTOR TESTING SUITE ==="
echo "Base directory: $BASE_DIR"
echo "Repository root: $REPO_ROOT"

# Step 1: Generate extreme test files
echo ""
echo "Step 1: Generating extreme test files..."
python3 "$SCRIPT_DIR/generate.py" --all-extreme --outdir "$BASE_DIR/benchmark-codebases/extreme_suite"

# Step 2: Clone real repositories (commented out by default - uncomment to enable)
echo ""
echo "Step 2: Cloning real repositories..."
# mkdir -p "$BASE_DIR/benchmark-codebases/real_repos"
# "$SCRIPT_DIR/clone_repos.sh" "$SCRIPT_DIR/repos.example.txt" "$BASE_DIR/benchmark-codebases/real_repos"

# Step 3: Run Uveddi analysis on all test cases
echo ""
echo "Step 3: Running Uveddi analysis..."

# Check if Uveddi binary exists
if [ -f "$REPO_ROOT/target/release/uveddi" ]; then
    UVEDDI_BIN="$REPO_ROOT/target/release/uveddi"
elif [ -f "$REPO_ROOT/target/debug/uveddi" ]; then
    UVEDDI_BIN="$REPO_ROOT/target/debug/uveddi"
else
    echo "Building Uveddi..."
    cd "$REPO_ROOT"
    cargo build --release
    UVEDDI_BIN="$REPO_ROOT/target/release/uveddi"
fi

echo "Using Uveddi binary: $UVEDDI_BIN"

# Analyze extreme test suite
echo "Analyzing extreme test suite..."
RESULTS_FILE="$BASE_DIR/STRESS_TEST_RESULTS.md"
echo "# Uveddi Stress Test Results" > "$RESULTS_FILE"
echo "Date: $(date)" >> "$RESULTS_FILE"
echo "Branch: $(cd "$REPO_ROOT" && git rev-parse --abbrev-ref HEAD)" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Test each language's extreme cases
for lang_dir in "$BASE_DIR/benchmark-codebases/extreme_suite"/*; do
    if [ -d "$lang_dir" ]; then
        lang_name=$(basename "$lang_dir")
        echo "Testing $lang_name extreme cases..."
        echo "## $lang_name Results" >> "$RESULTS_FILE"
        
        # Run Uveddi analysis and capture results
        if "$UVEDDI_BIN" analyze "$lang_dir" --output-format json > "$BASE_DIR/temp_results.json" 2>&1; then
            echo "✅ Analysis completed for $lang_name" >> "$RESULTS_FILE"
            
            # Count detections by type
            if command -v jq >/dev/null 2>&1; then
                god_objects=$(jq '.detections[] | select(.detector_type == "GodObject") | length' "$BASE_DIR/temp_results.json" 2>/dev/null || echo "0")
                dead_code=$(jq '.detections[] | select(.detector_type == "DeadCode") | length' "$BASE_DIR/temp_results.json" 2>/dev/null || echo "0")
                large_classes=$(jq '.detections[] | select(.detector_type == "LargeClass") | length' "$BASE_DIR/temp_results.json" 2>/dev/null || echo "0")
                
                echo "- God Objects detected: $god_objects" >> "$RESULTS_FILE"
                echo "- Dead Code detected: $dead_code" >> "$RESULTS_FILE"  
                echo "- Large Classes detected: $large_classes" >> "$RESULTS_FILE"
            fi
            
            rm -f "$BASE_DIR/temp_results.json"
        else
            echo "❌ Analysis failed for $lang_name" >> "$RESULTS_FILE"
            echo "Error details logged to stress_test_errors.log" >> "$RESULTS_FILE"
        fi
        
        echo "" >> "$RESULTS_FILE"
    fi
done

echo ""
echo "=== STRESS TESTING COMPLETE ==="
echo "Results written to: $RESULTS_FILE"
echo ""
echo "Summary of generated test files:"
find "$BASE_DIR/benchmark-codebases/extreme_suite" -name "*.rs" -o -name "*.py" -o -name "*.ts" | wc -l | xargs echo "Generated files:"

echo ""
echo "To run specific tests:"
echo "  python3 $SCRIPT_DIR/generate.py --lang rust --type god_object --fields 100 --methods 200 --outdir /tmp/custom_test"
echo "  $UVEDDI_BIN analyze /tmp/custom_test"
