#!/bin/bash
# Uveddi v1.0 Performance Benchmarks
# Measures analysis performance across different codebase sizes

set -e

echo "⚡ Uveddi v1.0 Performance Benchmarks"
echo "====================================="
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Build optimized binary
echo "🔨 Building optimized release binary..."
cargo build --release --features full 2>&1 | tail -5
echo -e "${GREEN}✅ Build complete${NC}"
echo ""

# Create benchmark output directory
BENCH_DIR="/tmp/uveddi-benchmarks-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BENCH_DIR"

echo "📊 Benchmark Results will be saved to: $BENCH_DIR"
echo ""

# Function to run benchmark
run_benchmark() {
    local name=$1
    local path=$2
    local output_file="$BENCH_DIR/bench-${name}.json"

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}📊 Benchmark: $name${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo "Path: $path"
    echo "Output: $output_file"
    echo ""

    # Count files
    if [ -d "$path" ]; then
        FILE_COUNT=$(find "$path" -type f \( -name "*.rs" -o -name "*.py" -o -name "*.js" -o -name "*.ts" \) | wc -l)
        LOC=$(find "$path" -type f \( -name "*.rs" -o -name "*.py" -o -name "*.js" -o -name "*.ts" \) -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
        echo "Files: $FILE_COUNT"
        echo "Lines of Code: ~$LOC"
        echo ""
    fi

    # Time the analysis
    echo "Running analysis..."
    /usr/bin/time -v ./target/release/uveddi analyze "$path" --output-format json --output "$output_file" 2>&1 | \
        grep -E "(Elapsed|Maximum resident|Percent of CPU)" || true

    echo ""

    # Report file size
    if [ -f "$output_file" ]; then
        REPORT_SIZE=$(du -h "$output_file" | cut -f1)
        echo -e "${GREEN}✅ Report generated: $REPORT_SIZE${NC}"
    else
        echo -e "${YELLOW}⚠️  Report not generated${NC}"
    fi

    echo ""
}

# Benchmark 1: Small codebase
echo "======================================"
echo "Benchmark 1: Small Codebase"
echo "======================================"
if [ -d "./examples/simple_demo" ]; then
    run_benchmark "small" "./examples/simple_demo"
elif [ -d "./src/cli" ]; then
    run_benchmark "small" "./src/cli"
else
    echo -e "${YELLOW}⚠️  Skipping small codebase benchmark (no suitable directory)${NC}"
fi

# Benchmark 2: Medium codebase
echo "======================================"
echo "Benchmark 2: Medium Codebase"
echo "======================================"
if [ -d "./src" ]; then
    run_benchmark "medium" "./src"
else
    echo -e "${YELLOW}⚠️  Skipping medium codebase benchmark${NC}"
fi

# Benchmark 3: Large codebase (if available)
echo "======================================"
echo "Benchmark 3: Large Codebase"
echo "======================================"
if [ -d "./tests" ] && [ -d "./src" ]; then
    # Combine src and tests for larger benchmark
    run_benchmark "large" "./"
else
    echo -e "${YELLOW}⚠️  Skipping large codebase benchmark${NC}"
fi

# Memory profiling on medium codebase
echo "======================================"
echo "Memory Profiling"
echo "======================================"
echo "Analyzing memory usage on medium codebase..."
echo ""

if [ -d "./src" ]; then
    /usr/bin/time -v ./target/release/uveddi analyze ./src --output "$BENCH_DIR/memory-test.json" 2>&1 | \
        grep -E "(Maximum resident|Average resident|Average total|Page size)" || true
else
    echo -e "${YELLOW}⚠️  Skipping memory profiling${NC}"
fi

echo ""

# CPU profiling (if flamegraph is available)
echo "======================================"
echo "CPU Profiling"
echo "======================================"

if command -v cargo-flamegraph &> /dev/null; then
    echo "Generating flamegraph..."
    if [ -d "./src/cli" ]; then
        cargo flamegraph --bin uveddi --features full -- analyze ./src/cli --output "$BENCH_DIR/flamegraph-test.json" 2>&1 | tail -5 || true
        if [ -f "flamegraph.svg" ]; then
            mv flamegraph.svg "$BENCH_DIR/"
            echo -e "${GREEN}✅ Flamegraph saved to $BENCH_DIR/flamegraph.svg${NC}"
        fi
    fi
else
    echo -e "${YELLOW}⚠️  cargo-flamegraph not installed${NC}"
    echo "Install with: cargo install flamegraph"
fi

echo ""

# Performance targets check
echo "======================================"
echo "Performance Targets Validation"
echo "======================================"
echo ""
echo "Expected Performance Targets:"
echo "┌──────────────────────┬──────────────┬──────────────┐"
echo "│ Codebase Size        │ Time Target  │ Memory Target│"
echo "├──────────────────────┼──────────────┼──────────────┤"
echo "│ Small (< 1k LOC)     │ < 5s         │ < 200MB      │"
echo "│ Medium (10k-50k LOC) │ < 30s        │ < 1GB        │"
echo "│ Large (> 100k LOC)   │ < 5min       │ < 4GB        │"
echo "└──────────────────────┴──────────────┴──────────────┘"
echo ""

# Summary
echo "======================================"
echo "Summary"
echo "======================================"
echo -e "${GREEN}✅ Benchmarks complete!${NC}"
echo ""
echo "Results saved to: $BENCH_DIR"
echo ""
echo "Generated files:"
ls -lh "$BENCH_DIR" | tail -n +2

echo ""
echo "📈 To view detailed results:"
echo "  cat $BENCH_DIR/bench-*.json | python3 -m json.tool"
echo ""
echo "🔥 To view flamegraph (if generated):"
echo "  firefox $BENCH_DIR/flamegraph.svg"
echo ""
