#!/bin/bash
set -euo pipefail

# Automated GitHub Repository Testing Framework for Uveddi Alpha
# Tests Uveddi on diverse GitHub repositories to gather feedback and validate detection capabilities

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Configuration
FEATURES="alpha"
TEST_ROOT="/tmp/uveddi_github_tests"
RESULTS_DIR="$PROJECT_ROOT/automated_test_results"
MAX_CONCURRENT=3
TIMEOUT_PER_REPO=300  # 5 minutes per repository
ANALYSIS_TIMEOUT=120  # 2 minutes for analysis itself

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_repo() { echo -e "${CYAN}[REPO]${NC} $1"; }

# Setup directories
setup_environment() {
    log_info "Setting up test environment..."
    
    rm -rf "$TEST_ROOT" "$RESULTS_DIR"
    mkdir -p "$TEST_ROOT" "$RESULTS_DIR"/{reports,logs,metrics,summaries}
    
    # Build Uveddi with alpha features
    log_info "Building Uveddi with alpha features..."
    if ! cargo build --release --features="$FEATURES"; then
        log_error "Failed to build Uveddi"
        exit 1
    fi
    
    log_success "Environment setup complete"
}

# Test repository definition - organized by language and expected patterns
declare -A TEST_REPOSITORIES=(
    # Python repositories with known architectural issues
    ["python_django_project"]="https://github.com/django/django.git|python|Large framework with complex architecture"
    ["python_flask_complex"]="https://github.com/pallets/flask.git|python|Web framework with potential patterns"
    ["python_requests"]="https://github.com/psf/requests.git|python|Popular HTTP library"
    ["python_pandas"]="https://github.com/pandas-dev/pandas.git|python|Data analysis library with complex codebase"
    ["python_scrapy"]="https://github.com/scrapy/scrapy.git|python|Web scraping framework"
    
    # JavaScript/TypeScript repositories
    ["js_express"]="https://github.com/expressjs/express.git|javascript|Node.js web framework"
    ["js_lodash"]="https://github.com/lodash/lodash.git|javascript|Utility library"
    ["js_axios"]="https://github.com/axios/axios.git|javascript|HTTP client library"
    ["ts_typescript_compiler"]="https://github.com/microsoft/TypeScript.git|typescript|TypeScript compiler itself"
    ["ts_vscode"]="https://github.com/microsoft/vscode.git|typescript|Large TypeScript application"
    
    # Rust repositories for validation (known to work well)
    ["rust_tokio"]="https://github.com/tokio-rs/tokio.git|rust|Async runtime validation"
    ["rust_serde"]="https://github.com/serde-rs/serde.git|rust|Serialization library"
    
    # Mixed language repositories
    ["mixed_react"]="https://github.com/facebook/react.git|mixed|React library (JS/TypeScript)"
    ["mixed_node_js"]="https://github.com/nodejs/node.git|mixed|Node.js runtime (C++/JS)"
    
    # Test repositories known to have issues
    ["test_legacy_python"]="https://github.com/python/cpython.git|python|Large legacy Python codebase"
    ["test_complex_js"]="https://github.com/webpack/webpack.git|javascript|Complex build tool"
)

# Clone repository with depth limit
clone_repository() {
    local repo_key="$1"
    local repo_info="${TEST_REPOSITORIES[$repo_key]}"
    local repo_url=$(echo "$repo_info" | cut -d'|' -f1)
    local repo_lang=$(echo "$repo_info" | cut -d'|' -f2)
    local repo_desc=$(echo "$repo_info" | cut -d'|' -f3)
    
    local repo_dir="$TEST_ROOT/$repo_key"
    
    log_repo "Cloning $repo_key ($repo_lang): $repo_desc"
    
    # Clone with shallow depth to save space and time
    if timeout "$TIMEOUT_PER_REPO" git clone --depth=1 --single-branch "$repo_url" "$repo_dir" &>/dev/null; then
        log_success "Successfully cloned $repo_key"
        echo "$repo_lang|$repo_desc" > "$repo_dir/.uveddi_test_meta"
        return 0
    else
        log_error "Failed to clone $repo_key"
        return 1
    fi
}

# Analyze repository with Uveddi
analyze_repository() {
    local repo_key="$1"
    local repo_dir="$TEST_ROOT/$repo_key"
    local binary="$PROJECT_ROOT/target/release/uveddi"
    
    if [[ ! -d "$repo_dir" ]]; then
        log_error "Repository directory not found: $repo_dir"
        return 1
    fi
    
    local meta_info=$(cat "$repo_dir/.uveddi_test_meta" 2>/dev/null || echo "unknown|No description")
    local repo_lang=$(echo "$meta_info" | cut -d'|' -f1)
    local repo_desc=$(echo "$meta_info" | cut -d'|' -f2)
    
    log_repo "Analyzing $repo_key ($repo_lang)"
    
    local start_time=$(date +%s)
    local output_file="$RESULTS_DIR/reports/${repo_key}_analysis.json"
    local log_file="$RESULTS_DIR/logs/${repo_key}_analysis.log"
    local metrics_file="$RESULTS_DIR/metrics/${repo_key}_metrics.json"
    
    # Run analysis with comprehensive options
    local analysis_cmd="timeout $ANALYSIS_TIMEOUT $binary analyze '$repo_dir' \
        --output-format=json \
        --output='$output_file' \
        --dead-code-confidence=0.7 \
        --large-classes-max-loc=100 \
        --large-classes-max-methods=20 \
        --large-classes-max-fields=15"
    
    # Capture both stdout and stderr
    if eval "$analysis_cmd" &>"$log_file"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        # Create metrics file
        create_metrics_file "$repo_key" "$repo_lang" "$repo_desc" "$duration" "success" "$output_file" "$metrics_file"
        
        log_success "Analysis completed for $repo_key in ${duration}s"
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        # Create metrics file for failure
        create_metrics_file "$repo_key" "$repo_lang" "$repo_desc" "$duration" "failed" "" "$metrics_file"
        
        log_error "Analysis failed for $repo_key after ${duration}s"
        return 1
    fi
}

# Create metrics file for analysis results
create_metrics_file() {
    local repo_key="$1"
    local repo_lang="$2"
    local repo_desc="$3"
    local duration="$4"
    local status="$5"
    local output_file="$6"
    local metrics_file="$7"
    
    local file_count=0
    local issues_found=0
    local detectors_run=0
    
    # Count files in repository
    if [[ -d "$TEST_ROOT/$repo_key" ]]; then
        case "$repo_lang" in
            "python")
                file_count=$(find "$TEST_ROOT/$repo_key" -name "*.py" | wc -l)
                ;;
            "javascript")
                file_count=$(find "$TEST_ROOT/$repo_key" -name "*.js" | wc -l)
                ;;
            "typescript")
                file_count=$(find "$TEST_ROOT/$repo_key" -name "*.ts" -o -name "*.tsx" | wc -l)
                ;;
            "rust")
                file_count=$(find "$TEST_ROOT/$repo_key" -name "*.rs" | wc -l)
                ;;
            "mixed")
                file_count=$(find "$TEST_ROOT/$repo_key" \( -name "*.py" -o -name "*.js" -o -name "*.ts" -o -name "*.tsx" -o -name "*.rs" \) | wc -l)
                ;;
        esac
    fi
    
    # Extract issues from analysis output if successful
    if [[ "$status" == "success" && -f "$output_file" ]]; then
        if command -v jq &>/dev/null; then
            issues_found=$(jq '.issues | length' "$output_file" 2>/dev/null || echo 0)
            detectors_run=$(jq '.summary.detectors_run // 0' "$output_file" 2>/dev/null || echo 0)
        fi
    fi
    
    # Create comprehensive metrics
    cat > "$metrics_file" << EOF
{
    "repository": "$repo_key",
    "language": "$repo_lang",
    "description": "$repo_desc",
    "analysis": {
        "status": "$status",
        "duration_seconds": $duration,
        "timestamp": "$(date -Iseconds)"
    },
    "codebase": {
        "file_count": $file_count,
        "files_analyzed": $file_count
    },
    "results": {
        "issues_found": $issues_found,
        "detectors_run": $detectors_run
    }
}
EOF
}

# Process single repository (clone and analyze)
process_repository() {
    local repo_key="$1"
    
    log_repo "Processing repository: $repo_key"
    
    # Clone repository
    if clone_repository "$repo_key"; then
        # Analyze repository
        if analyze_repository "$repo_key"; then
            log_success "Successfully processed $repo_key"
        else
            log_warning "Analysis failed for $repo_key but repository was cloned"
        fi
        
        # Clean up repository to save space
        rm -rf "$TEST_ROOT/$repo_key"
    else
        log_error "Failed to process $repo_key - could not clone"
    fi
}

# Run tests in parallel batches
run_parallel_tests() {
    log_info "Starting parallel repository testing (max concurrent: $MAX_CONCURRENT)"
    
    local repo_keys=($(printf '%s\n' "${!TEST_REPOSITORIES[@]}" | sort))
    local total_repos=${#repo_keys[@]}
    local completed=0
    
    # Process repositories in batches
    for ((i=0; i<total_repos; i+=MAX_CONCURRENT)); do
        local batch_pids=()
        local batch_repos=()
        
        # Start batch of repositories
        for ((j=0; j<MAX_CONCURRENT && i+j<total_repos; j++)); do
            local repo_key="${repo_keys[i+j]}"
            batch_repos+=("$repo_key")
            
            log_info "Starting analysis of $repo_key ($(($i+$j+1))/$total_repos)"
            process_repository "$repo_key" &
            batch_pids+=($!)
        done
        
        # Wait for batch to complete
        for k in "${!batch_pids[@]}"; do
            local pid="${batch_pids[k]}"
            local repo="${batch_repos[k]}"
            
            if wait "$pid"; then
                log_success "Batch job completed: $repo"
            else
                log_warning "Batch job failed: $repo"
            fi
            ((completed++))
        done
        
        log_info "Batch completed. Progress: $completed/$total_repos repositories"
    done
    
    log_success "All repositories processed!"
}

# Generate comprehensive test summary
generate_summary_report() {
    log_info "Generating comprehensive test summary..."
    
    local summary_file="$RESULTS_DIR/summaries/test_summary_$(date +%Y%m%d_%H%M%S).json"
    local markdown_file="$RESULTS_DIR/summaries/test_summary_$(date +%Y%m%d_%H%M%S).md"
    
    # Collect all metrics
    local total_repos=0
    local successful_analyses=0
    local failed_analyses=0
    local total_issues=0
    local total_files=0
    local avg_duration=0
    local by_language=()
    
    # Process metrics files
    for metrics_file in "$RESULTS_DIR/metrics"/*.json; do
        if [[ -f "$metrics_file" ]]; then
            ((total_repos++))
            
            if command -v jq &>/dev/null; then
                local status=$(jq -r '.analysis.status' "$metrics_file" 2>/dev/null || echo "unknown")
                local issues=$(jq -r '.results.issues_found' "$metrics_file" 2>/dev/null || echo "0")
                local files=$(jq -r '.codebase.file_count' "$metrics_file" 2>/dev/null || echo "0")
                local duration=$(jq -r '.analysis.duration_seconds' "$metrics_file" 2>/dev/null || echo "0")
                local language=$(jq -r '.language' "$metrics_file" 2>/dev/null || echo "unknown")
                
                if [[ "$status" == "success" ]]; then
                    ((successful_analyses++))
                    total_issues=$((total_issues + issues))
                else
                    ((failed_analyses++))
                fi
                
                total_files=$((total_files + files))
                avg_duration=$((avg_duration + duration))
                
                by_language+=("$language:$status:$issues:$files:$duration")
            fi
        fi
    done
    
    if [[ $total_repos -gt 0 ]]; then
        avg_duration=$((avg_duration / total_repos))
    fi
    
    # Create JSON summary
    cat > "$summary_file" << EOF
{
    "test_run": {
        "timestamp": "$(date -Iseconds)",
        "total_repositories": $total_repos,
        "successful_analyses": $successful_analyses,
        "failed_analyses": $failed_analyses,
        "success_rate": $(echo "scale=2; $successful_analyses * 100 / $total_repos" | bc -l 2>/dev/null || echo "0")
    },
    "aggregate_metrics": {
        "total_issues_found": $total_issues,
        "total_files_analyzed": $total_files,
        "average_analysis_duration": $avg_duration
    },
    "language_breakdown": [
$(for lang_data in "${by_language[@]}"; do
    IFS=':' read -r lang status issues files duration <<< "$lang_data"
    echo "        {\"language\": \"$lang\", \"status\": \"$status\", \"issues\": $issues, \"files\": $files, \"duration\": $duration},"
done | sed '$ s/,$//')
    ]
}
EOF
    
    # Create Markdown report
    cat > "$markdown_file" << EOF
# Uveddi Alpha Testing Report

**Generated:** $(date)

## Executive Summary

- **Total Repositories Tested:** $total_repos
- **Successful Analyses:** $successful_analyses
- **Failed Analyses:** $failed_analyses
- **Success Rate:** $(echo "scale=1; $successful_analyses * 100 / $total_repos" | bc -l 2>/dev/null || echo "0")%
- **Total Issues Found:** $total_issues
- **Total Files Analyzed:** $total_files
- **Average Analysis Time:** ${avg_duration}s

## Language Breakdown

| Language | Status | Issues Found | Files | Duration (s) |
|----------|--------|--------------|-------|--------------|
$(for lang_data in "${by_language[@]}"; do
    IFS=':' read -r lang status issues files duration <<< "$lang_data"
    echo "| $lang | $status | $issues | $files | $duration |"
done)

## Test Results by Repository

$(for metrics_file in "$RESULTS_DIR/metrics"/*.json; do
    if [[ -f "$metrics_file" ]] && command -v jq &>/dev/null; then
        local repo=$(jq -r '.repository' "$metrics_file" 2>/dev/null)
        local lang=$(jq -r '.language' "$metrics_file" 2>/dev/null)
        local status=$(jq -r '.analysis.status' "$metrics_file" 2>/dev/null)
        local issues=$(jq -r '.results.issues_found' "$metrics_file" 2>/dev/null)
        local desc=$(jq -r '.description' "$metrics_file" 2>/dev/null)
        echo "### $repo ($lang)"
        echo "- **Status:** $status"
        echo "- **Issues Found:** $issues"
        echo "- **Description:** $desc"
        echo ""
    fi
done)

## Recommendations

### Alpha Release Readiness
$(if [[ $successful_analyses -gt $((total_repos * 7 / 10)) ]]; then
    echo "✅ **READY** - Success rate above 70%, good for alpha release"
else
    echo "⚠️ **NEEDS WORK** - Success rate below 70%, requires investigation"
fi)

### Next Steps
1. Review failed analyses for common patterns
2. Investigate performance issues if average duration > 60s
3. Validate issue detection accuracy with manual review
4. Consider expanding test repository set based on results

---
*Generated by Uveddi Automated Testing Framework*
EOF
    
    log_success "Summary report generated:"
    log_info "  JSON: $summary_file"
    log_info "  Markdown: $markdown_file"
}

# Main execution function
main() {
    local start_time=$(date +%s)
    
    log_info "Starting Uveddi Alpha GitHub Repository Testing Framework"
    log_info "Testing ${#TEST_REPOSITORIES[@]} repositories across multiple languages"
    
    # Setup
    setup_environment
    
    # Run tests
    run_parallel_tests
    
    # Generate reports
    generate_summary_report
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    log_success "Automated testing completed in ${total_duration} seconds"
    log_info "Results available in: $RESULTS_DIR"
    
    # Display quick summary
    local successful=$(find "$RESULTS_DIR/reports" -name "*.json" | wc -l)
    local total=${#TEST_REPOSITORIES[@]}
    
    echo ""
    echo "=========================================="
    echo "         QUICK SUMMARY"
    echo "=========================================="
    echo "Repositories tested: $total"
    echo "Successful analyses: $successful"
    echo "Success rate: $(echo "scale=1; $successful * 100 / $total" | bc -l 2>/dev/null || echo "0")%"
    echo "Total duration: ${total_duration}s"
    echo "Results directory: $RESULTS_DIR"
    echo "=========================================="
}

# Handle command line arguments
case "${1:-full}" in
    "setup")
        setup_environment
        ;;
    "single")
        if [[ -z "${2:-}" ]]; then
            echo "Usage: $0 single <repository_key>"
            echo "Available repositories:"
            printf '%s\n' "${!TEST_REPOSITORIES[@]}" | sort
            exit 1
        fi
        setup_environment
        process_repository "$2"
        ;;
    "python")
        # Test only Python repositories
        declare -A FILTERED_REPOS
        for key in "${!TEST_REPOSITORIES[@]}"; do
            if [[ "${TEST_REPOSITORIES[$key]}" == *"|python|"* ]]; then
                FILTERED_REPOS["$key"]="${TEST_REPOSITORIES[$key]}"
            fi
        done
        TEST_REPOSITORIES=()
        for key in "${!FILTERED_REPOS[@]}"; do
            TEST_REPOSITORIES["$key"]="${FILTERED_REPOS[$key]}"
        done
        main
        ;;
    "javascript"|"js")
        # Test only JavaScript repositories
        declare -A FILTERED_REPOS
        for key in "${!TEST_REPOSITORIES[@]}"; do
            if [[ "${TEST_REPOSITORIES[$key]}" == *"|javascript|"* ]]; then
                FILTERED_REPOS["$key"]="${TEST_REPOSITORIES[$key]}"
            fi
        done
        TEST_REPOSITORIES=()
        for key in "${!FILTERED_REPOS[@]}"; do
            TEST_REPOSITORIES["$key"]="${FILTERED_REPOS[$key]}"
        done
        main
        ;;
    "typescript"|"ts")
        # Test only TypeScript repositories
        declare -A FILTERED_REPOS
        for key in "${!TEST_REPOSITORIES[@]}"; do
            if [[ "${TEST_REPOSITORIES[$key]}" == *"|typescript|"* ]]; then
                FILTERED_REPOS["$key"]="${TEST_REPOSITORIES[$key]}"
            fi
        done
        TEST_REPOSITORIES=()
        for key in "${!FILTERED_REPOS[@]}"; do
            TEST_REPOSITORIES["$key"]="${FILTERED_REPOS[$key]}"
        done
        main
        ;;
    "rust")
        # Test only Rust repositories (validation)
        declare -A FILTERED_REPOS
        for key in "${!TEST_REPOSITORIES[@]}"; do
            if [[ "${TEST_REPOSITORIES[$key]}" == *"|rust|"* ]]; then
                FILTERED_REPOS["$key"]="${TEST_REPOSITORIES[$key]}"
            fi
        done
        TEST_REPOSITORIES=()
        for key in "${!FILTERED_REPOS[@]}"; do
            TEST_REPOSITORIES["$key"]="${FILTERED_REPOS[$key]}"
        done
        main
        ;;
    "report")
        generate_summary_report
        ;;
    "full"|"")
        main
        ;;
    "list")
        echo "Available repositories for testing:"
        for key in $(printf '%s\n' "${!TEST_REPOSITORIES[@]}" | sort); do
            info="${TEST_REPOSITORIES[$key]}"
            lang=$(echo "$info" | cut -d'|' -f2)
            desc=$(echo "$info" | cut -d'|' -f3)
            echo "  $key ($lang): $desc"
        done
        ;;
    *)
        echo "Usage: $0 [setup|single <repo>|python|javascript|typescript|rust|report|full|list]"
        echo ""
        echo "Commands:"
        echo "  setup     - Setup test environment only"
        echo "  single    - Test a single repository"
        echo "  python    - Test only Python repositories"
        echo "  javascript - Test only JavaScript repositories" 
        echo "  typescript - Test only TypeScript repositories"
        echo "  rust      - Test only Rust repositories"
        echo "  report    - Generate summary report from existing results"
        echo "  full      - Run full test suite (default)"
        echo "  list      - List all available test repositories"
        exit 1
        ;;
esac