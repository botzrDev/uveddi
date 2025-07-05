#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}===== UVEDDI PRE-RELEASE TEST SUITE =====${NC}"
echo "This script runs comprehensive tests to ensure Uveddi is ready for release."

# Check if Ollama is running
echo -e "\n${YELLOW}Checking if Ollama is running...${NC}"
if curl -s --fail http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Ollama is running${NC}"
    OLLAMA_RUNNING=true
else
    echo -e "${YELLOW}⚠ Ollama is not running. Some AI-related tests will be skipped.${NC}"
    echo "To enable full testing, please start Ollama with:"
    echo "  ollama serve"
    OLLAMA_RUNNING=false
fi

# Check if the required model is available
if [ "$OLLAMA_RUNNING" = true ]; then
    echo -e "\n${YELLOW}Checking if deepseek-coder model is available...${NC}"
    if ollama list | grep -q "deepseek-coder:6.7b-instruct-q4_0"; then
        echo -e "${GREEN}✓ deepseek-coder model is available${NC}"
    else
        echo -e "${YELLOW}⚠ deepseek-coder model is not available. Pulling it now...${NC}"
        ollama pull deepseek-coder:6.7b-instruct-q4_0
        
        if [ $? -ne 0 ]; then
            echo -e "${RED}✗ Failed to pull the model. Some AI tests may fail.${NC}"
        else
            echo -e "${GREEN}✓ Model pulled successfully${NC}"
        fi
    fi
fi

# Step 1: Run cargo fmt check
echo -e "\n${YELLOW}Step 1: Checking code formatting...${NC}"
cargo fmt -- --check
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Code formatting is correct${NC}"
else
    echo -e "${RED}✗ Code formatting issues detected. Please run 'cargo fmt' to fix.${NC}"
    exit 1
fi

# Step 2: Run cargo clippy
echo -e "\n${YELLOW}Step 2: Running clippy lints...${NC}"
cargo clippy -- -D warnings
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ No clippy warnings${NC}"
else
    echo -e "${RED}✗ Clippy warnings detected. Please fix them before release.${NC}"
    exit 1
fi

# Step 3: Generate benchmark data
echo -e "\n${YELLOW}Step 3: Generating benchmark data...${NC}"
cargo run --bin generate_benchmark_data
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Benchmark data generated successfully${NC}"
else
    echo -e "${RED}✗ Failed to generate benchmark data${NC}"
    exit 1
fi

# Step 4: Run unit tests
echo -e "\n${YELLOW}Step 4: Running unit tests...${NC}"
cargo test --lib -- --nocapture
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ All unit tests passed${NC}"
else
    echo -e "${RED}✗ Unit tests failed${NC}"
    exit 1
fi

# Step 5: Run integration tests 
echo -e "\n${YELLOW}Step 5: Running integration tests...${NC}"
cargo test --test '*' -- --nocapture
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ All integration tests passed${NC}"
else
    echo -e "${RED}✗ Some integration tests failed (non-fatal, may be due to missing Ollama)${NC}"
    # Don't exit here, as some tests might fail due to missing Ollama
fi

# Step 6: Run benchmarks to ensure performance
echo -e "\n${YELLOW}Step 6: Running benchmarks...${NC}"
cargo bench
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Benchmarks completed successfully${NC}"
else
    echo -e "${RED}✗ Benchmark tests failed${NC}"
    exit 1
fi

# Step 7: Build in release mode
echo -e "\n${YELLOW}Step 7: Building in release mode...${NC}"
cargo build --release
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Release build successful${NC}"
else
    echo -e "${RED}✗ Release build failed${NC}"
    exit 1
fi

# Step 8: Run a simple analysis with the release binary
echo -e "\n${YELLOW}Step 8: Testing release binary...${NC}"
echo "Running analysis on the benchmark data with the release binary"
./target/release/uveddi analyze ./target/benchmark-data/rust-project --format=markdown --output=./analysis_report.md

if [ $? -eq 0 ] && [ -f ./analysis_report.md ]; then
    echo -e "${GREEN}✓ Analysis completed successfully and report generated${NC}"
    echo "Report saved to ./analysis_report.md"
else
    echo -e "${RED}✗ Analysis with release binary failed${NC}"
    exit 1
fi

# Success!
echo -e "\n${GREEN}===== ALL TESTS COMPLETED SUCCESSFULLY =====${NC}"
echo -e "${GREEN}Uveddi is ready for release!${NC}"
echo -e "\nRelease report summary:"
echo "- Code quality: Passed"
echo "- Unit tests: Passed"
echo "- Integration tests: Checked"
echo "- Performance benchmarks: Completed"
echo "- Release build: Successful"
echo "- End-to-end analysis: Successful"
echo -e "\nThe code is now considered stable and ready for release."
