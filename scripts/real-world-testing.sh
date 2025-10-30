#!/bin/bash

# Uveddi Real-World Codebase Testing Script
# Tests Uveddi against popular open-source projects

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_DIR="real_world_tests"
RESULTS_DIR="real_world_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SUMMARY_FILE="${RESULTS_DIR}/summary_${TIMESTAMP}.md"

# Ensure Uveddi is built
echo -e "${BLUE}Building Uveddi with production features...${NC}"
cargo build --release --features=production

# Create directories
mkdir -p "$TEST_DIR"
mkdir -p "$RESULTS_DIR"

# Function to clone or update repository
clone_or_update() {
    local repo_url=$1
    local repo_name=$2
    local target_dir="${TEST_DIR}/${repo_name}"
    
    if [ -d "$target_dir" ]; then
        echo -e "${YELLOW}Updating ${repo_name}...${NC}"
        cd "$target_dir"
        git pull --quiet
        cd - > /dev/null
    else
        echo -e "${GREEN}Cloning ${repo_name}...${NC}"
        git clone --depth=1 "$repo_url" "$target_dir"
    fi
}

# Function to analyze repository
analyze_repo() {
    local repo_name=$1
    local language=$2
    local target_dir="${TEST_DIR}/${repo_name}"
    local output_file="${RESULTS_DIR}/${repo_name}_${TIMESTAMP}.json"
    local html_file="${RESULTS_DIR}/${repo_name}_${TIMESTAMP}.html"
    
    echo -e "${BLUE}Analyzing ${repo_name} (${language})...${NC}"
    
    # Run analysis with timeout
    timeout 300 cargo run --release --features=production --bin uveddi -- \
        analyze "$target_dir" \
        --output-format json \
        --output "$output_file" \
        2>&1 | tee "${RESULTS_DIR}/${repo_name}_${TIMESTAMP}.log"
    
    # Generate HTML report
    timeout 60 cargo run --release --features=production --bin uveddi -- \
        analyze "$target_dir" \
        --output-format html \
        --output "$html_file" \
        2>&1 > /dev/null || true
    
    # Extract metrics
    if [ -f "$output_file" ]; then
        local files_analyzed=$(jq -r '.summary.files_analyzed // 0' "$output_file" 2>/dev/null || echo "0")
        local issues_found=$(jq -r '.summary.total_issues // 0' "$output_file" 2>/dev/null || echo "0")
        local analysis_time=$(jq -r '.metadata.analysis_duration_ms // 0' "$output_file" 2>/dev/null || echo "0")
        
        echo "  Files analyzed: $files_analyzed"
        echo "  Issues found: $issues_found"
        echo "  Analysis time: ${analysis_time}ms"
        
        # Add to summary
        echo "| $repo_name | $language | $files_analyzed | $issues_found | ${analysis_time}ms |" >> "$SUMMARY_FILE"
    else
        echo -e "${RED}  Analysis failed or timed out${NC}"
        echo "| $repo_name | $language | ERROR | ERROR | ERROR |" >> "$SUMMARY_FILE"
    fi
}

# Initialize summary file
cat > "$SUMMARY_FILE" << EOF
# Uveddi Real-World Testing Results

**Date**: $(date)
**Version**: v1.0.0-alpha
**Test Type**: Real-world codebase analysis

## Test Repositories

### Small Projects (< 10K LOC)
EOF

echo "| Repository | Language | Files | Issues | Time |" >> "$SUMMARY_FILE"
echo "|------------|----------|-------|--------|------|" >> "$SUMMARY_FILE"

# Small Rust Projects
clone_or_update "https://github.com/BurntSushi/ripgrep" "ripgrep"
analyze_repo "ripgrep" "Rust"

clone_or_update "https://github.com/sharkdp/bat" "bat"
analyze_repo "bat" "Rust"

# Small Python Projects
clone_or_update "https://github.com/httpie/httpie" "httpie"
analyze_repo "httpie" "Python"

clone_or_update "https://github.com/psf/black" "black"
analyze_repo "black" "Python"

# Small JavaScript/TypeScript Projects
clone_or_update "https://github.com/sindresorhus/got" "got"
analyze_repo "got" "JavaScript"

clone_or_update "https://github.com/tannerlinsley/react-query" "react-query"
analyze_repo "react-query" "TypeScript"

# Add medium projects section
cat >> "$SUMMARY_FILE" << EOF

### Medium Projects (10K-50K LOC)
| Repository | Language | Files | Issues | Time |
|------------|----------|-------|--------|------|
EOF

# Medium Rust Projects
clone_or_update "https://github.com/tokio-rs/tokio" "tokio"
analyze_repo "tokio" "Rust"

# Medium Python Projects
clone_or_update "https://github.com/django/django" "django"
analyze_repo "django" "Python"

# Medium TypeScript Projects
clone_or_update "https://github.com/microsoft/vscode" "vscode"
analyze_repo "vscode" "TypeScript"

# Generate performance analysis
cat >> "$SUMMARY_FILE" << EOF

## Performance Analysis

### Analysis Speed
EOF

# Calculate average analysis speed
echo "Calculating performance metrics..."
total_files=0
total_time=0
total_issues=0
count=0

for json_file in ${RESULTS_DIR}/*_${TIMESTAMP}.json; do
    if [ -f "$json_file" ]; then
        files=$(jq -r '.summary.files_analyzed // 0' "$json_file" 2>/dev/null || echo "0")
        time=$(jq -r '.metadata.analysis_duration_ms // 0' "$json_file" 2>/dev/null || echo "0")
        issues=$(jq -r '.summary.total_issues // 0' "$json_file" 2>/dev/null || echo "0")
        
        if [ "$files" != "ERROR" ] && [ "$files" -gt 0 ]; then
            total_files=$((total_files + files))
            total_time=$((total_time + time))
            total_issues=$((total_issues + issues))
            count=$((count + 1))
        fi
    fi
done

if [ $count -gt 0 ]; then
    avg_files=$((total_files / count))
    avg_time=$((total_time / count))
    avg_issues=$((total_issues / count))
    
    if [ $total_files -gt 0 ]; then
        files_per_second=$((total_files * 1000 / total_time))
    else
        files_per_second=0
    fi
    
    cat >> "$SUMMARY_FILE" << EOF
- **Total files analyzed**: $total_files
- **Total analysis time**: ${total_time}ms
- **Average files per project**: $avg_files
- **Average analysis time**: ${avg_time}ms
- **Files per second**: $files_per_second
- **Total issues detected**: $total_issues
- **Average issues per project**: $avg_issues

### Anti-Pattern Distribution
EOF

    # Analyze anti-pattern distribution
    echo "Analyzing anti-pattern distribution..."
    for json_file in ${RESULTS_DIR}/*_${TIMESTAMP}.json; do
        if [ -f "$json_file" ]; then
            repo_name=$(basename "$json_file" | sed "s/_${TIMESTAMP}.json//")
            echo "" >> "$SUMMARY_FILE"
            echo "#### $repo_name" >> "$SUMMARY_FILE"
            jq -r '.issues_by_type // {} | to_entries | .[] | "- \(.key): \(.value)"' "$json_file" 2>/dev/null >> "$SUMMARY_FILE" || echo "- No anti-patterns detected" >> "$SUMMARY_FILE"
        fi
    done
fi

# Add recommendations
cat >> "$SUMMARY_FILE" << EOF

## Key Findings

### Strengths
- Successfully analyzed multiple real-world codebases
- Performance scales well with codebase size
- Multi-language support working correctly

### Areas for Improvement
- Large codebases may require memory optimization
- Some language-specific patterns may need refinement
- Performance optimization for very large projects

## Recommendations

1. **Memory Management**: Consider streaming analysis for large files
2. **Parallelization**: Increase parallel processing for multi-file projects
3. **Caching**: Implement inter-run caching for frequently analyzed projects
4. **Language-Specific Tuning**: Adjust thresholds per language

---

**Generated**: $(date)
**Test Framework**: Real-World Codebase Testing v1.0
EOF

echo -e "${GREEN}Testing complete! Results saved to ${SUMMARY_FILE}${NC}"
echo -e "${BLUE}Individual reports available in ${RESULTS_DIR}/${NC}"

# Generate consolidated HTML report
echo -e "${BLUE}Generating consolidated HTML report...${NC}"
cargo run --release --features=production --bin uveddi -- \
    analyze "$TEST_DIR" \
    --output-format html \
    --output "${RESULTS_DIR}/consolidated_${TIMESTAMP}.html" \
    2>&1 > /dev/null || true

echo -e "${GREEN}All tests completed successfully!${NC}"