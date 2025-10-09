#!/bin/bash
# Run comprehensive analysis on test corpus and collect metrics
#
# Usage: ./scripts/calibrate_detectors.sh [corpus_dir]
#
# Example:
#   ./scripts/calibrate_detectors.sh test-codebases

set -e

CORPUS_DIR="${1:-test-codebases}"
OUTPUT_DIR="reports/calibration-$(date +%Y-%m-%d)"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Uveddi Detector Calibration ===${NC}"
echo "Corpus directory: $CORPUS_DIR"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to analyze a codebase
analyze_repo() {
    local repo_path=$1
    local repo_name=$(basename "$repo_path")
    local output_file="$OUTPUT_DIR/${repo_name}-analysis.json"

    echo -e "${YELLOW}Analyzing:${NC} $repo_name"

    # Run analysis with timeout to prevent hanging
    timeout 300 cargo run --release --features standard -- analyze "$repo_path" \
        --output-format json \
        --output "$output_file" \
        --verbose 2>&1 | tee "$OUTPUT_DIR/${repo_name}-log.txt" || {
            echo "  ⚠️  Analysis timed out or failed for $repo_name"
            return 1
        }

    echo -e "${GREEN}✓ Completed:${NC} $repo_name"

    # Extract quick stats
    if [ -f "$output_file" ]; then
        local issue_count=$(jq '.issues | length' "$output_file" 2>/dev/null || echo "unknown")
        local file_count=$(jq '.metadata.files_analyzed // 0' "$output_file" 2>/dev/null || echo "unknown")
        echo "  Issues found: $issue_count (files: $file_count)"
    fi

    return 0
}

# Counter for tracking
total_repos=0
successful_repos=0
failed_repos=0

# Analyze all repos in corpus
for size in small medium large; do
    for lang in rust python javascript typescript; do
        corpus_path="$CORPUS_DIR/$size/$lang"
        if [ -d "$corpus_path" ]; then
            echo -e "\n${YELLOW}=== Processing: $size/$lang ===${NC}"
            for repo in "$corpus_path"/*; do
                if [ -d "$repo" ]; then
                    total_repos=$((total_repos + 1))
                    if analyze_repo "$repo"; then
                        successful_repos=$((successful_repos + 1))
                    else
                        failed_repos=$((failed_repos + 1))
                    fi
                fi
            done
        fi
    done
done

# Also analyze repos in root of test-codebases if they exist
if [ -d "$CORPUS_DIR" ]; then
    for repo in "$CORPUS_DIR"/*; do
        if [ -d "$repo" ] && [ ! -d "$repo/rust" ] && [ ! -d "$repo/python" ]; then
            # It's a repo, not a category directory
            total_repos=$((total_repos + 1))
            if analyze_repo "$repo"; then
                successful_repos=$((successful_repos + 1))
            else
                failed_repos=$((failed_repos + 1))
            fi
        fi
    done
fi

# Summary
echo ""
echo -e "${YELLOW}=== Analysis Complete ===${NC}"
echo "Total repositories: $total_repos"
echo -e "${GREEN}Successful: $successful_repos${NC}"
if [ $failed_repos -gt 0 ]; then
    echo -e "\033[0;31mFailed: $failed_repos${NC}"
fi
echo ""
echo "Results directory: $OUTPUT_DIR"
echo ""
echo "Next steps:"
echo "  1. Annotate ground truth:"
echo "     python3 scripts/annotate_ground_truth.py $OUTPUT_DIR/<repo>-analysis.json"
echo ""
echo "  2. Calculate metrics:"
echo "     python3 scripts/calculate_metrics.py $OUTPUT_DIR/<repo>-analysis_ground_truth.json"
echo ""
